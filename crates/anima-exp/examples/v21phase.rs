//! Post-V2.1 exploration P1: slow-state phase diagram on the wedge flank.
//!
//! Read-only instrument over committed/uncommitted v21probe-style runs.
//! Per run: classify the silence period (S2) activity regime.
//!   pacemaker   — tiny core at refractory ceiling (W1/W2 signature)
//!   irregular   — many neurons, high Fano (variable, distributed)
//!   silent      — no endogenous firing
//! Also: endogenous spike count, active-neuron count, max rate, mean Fano
//! of top-5 neurons, population-rate lag-1 autocorrelation.
use std::collections::BTreeMap;

fn main() {
    let dirs: Vec<String> = std::env::args().skip(1).collect();
    println!("run\tsil_spk\tactive\tmax_hz\tfano5\tlag1\tregime");
    for dir in dirs {
        let tdir = std::path::Path::new(&dir).join("telemetry");
        let reader = match anima_telemetry::TelemetryReader::open(&tdir) {
            Ok(r) => r,
            Err(_) => { eprintln!("ERR {}", dir); println!("{}\tERR", dir); continue; }
        };
        let idx = reader.chunk_index();
        // stage markers: kind 4 = StageBoundary? find silence window from env events.
        // We use StimulusPresented markers: silence = after last presentation + 500ms.
        let mut last_end: u64 = 0;
        let mut spikes: Vec<(u64, u32)> = Vec::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                match row.kind {
                    5 => {
                        if let Some(env) = row.envelope("e").ok() {
                            if let anima_telemetry::events::Payload::StimulusPresented { .. } = env.payload {
                                last_end = (env.t + 500).max(last_end);
                            }
                        }
                    }
                    3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                    _ => {}
                }
            }
        }
        spikes.sort_by_key(|s| s.0);
        let input_n = 24u32;
        let end = spikes.last().map(|s| s.0 + 1).unwrap_or(last_end);
        let sil: Vec<(u64, u32)> = spikes.iter().filter(|&&(t, n)| t >= last_end && n >= input_n).cloned().collect();
        // per-neuron counts + 1s-binned counts for Fano
        let mut per: BTreeMap<u32, u64> = BTreeMap::new();
        for &(_, n) in &sil { *per.entry(n).or_insert(0) += 1; }
        // Clamp: a silent run has a last presentation but no spikes; end falls
        // back to last_end, and aborted runs may end near last_end. Derive the
        // silence length defensively instead of trusting (end - last_end).
        let sil_ms = end.saturating_sub(last_end).min(120_000);
        let n_secs = ((sil_ms as f64 / 1000.0).ceil() as u64).max(1);
        let active = per.len();
        let sil_spk = sil.len();
        let max_hz = per.values().map(|&c| c as f64 / n_secs as f64).fold(0.0, f64::max);
        // Fano of top-5 by count
        let mut top: Vec<u32> = per.keys().copied().collect();
        top.sort_by_key(|&n| std::cmp::Reverse(per[&n]));
        top.truncate(5);
        let mut fanos: Vec<f64> = Vec::new();
        for &n in &top {
            let mut bins = vec![0f64; n_secs as usize];
            let sil0 = last_end;
            for &(t, m) in &sil {
                if m == n { let b = ((t - sil0) / 1000) as usize; if b < bins.len() { bins[b] += 1.0; } }
            }
            let mean = bins.iter().sum::<f64>() / bins.len() as f64;
            if mean > 0.0 {
                let var = bins.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / bins.len() as f64;
                fanos.push(var / mean);
            }
        }
        let fano5 = if fanos.is_empty() { f64::NAN } else { fanos.iter().sum::<f64>() / fanos.len() as f64 };
        // population rate lag-1 autocorr over 100ms bins
        let n_bins = (((end.saturating_sub(last_end)).min(120_000)) / 100).max(1) as usize;
        let mut pr = vec![0f64; n_bins];
        for &(t, _) in &sil { let b = ((t - last_end) / 100) as usize; if b < pr.len() { pr[b] += 1.0; } }
        let lag1 = if n_bins > 2 {
            let m = pr.iter().sum::<f64>() / n_bins as f64;
            let num: f64 = (0..n_bins - 1).map(|i| (pr[i] - m) * (pr[i + 1] - m)).sum();
            let den: f64 = pr.iter().map(|x| (x - m).powi(2)).sum();
            if den > 0.0 { num / den } else { 0.0 }
        } else { 0.0 };
        // regime classification
        let regime = if sil_spk == 0 {
            "silent".to_string()
        } else if active <= 8 && max_hz > 480.0 {
            "pacemaker".to_string()
        } else if active <= 8 {
            "sparse-core".to_string()
        } else {
            "irregular".to_string()
        };
        println!("{}\t{}\t{}\t{:.1}\t{:.3}\t{:.3}\t{}",
            dir.rsplit('/').next().unwrap(), sil_spk, active, max_hz, fano5, lag1, regime);
    }
}
