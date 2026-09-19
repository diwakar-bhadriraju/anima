//! V2.2 Stage-2 analysis instrument: per run emits
//! seed, arm, curriculum, end, M, S10, regime, topu, latchset@drive-end,
//! latchset@+2s (Jaccard vs drive-end), max_silence_hz, latch_frac@drive-end.
//! C-S2 = Jaccard >= 0.8; C-S3 = max_hz < 300.
use std::collections::BTreeMap;

fn main() {
    for dir in std::env::args().skip(1) {
        let name = dir.rsplit('/').next().unwrap().to_string();
        // v22s2-{arm}-s{seed}-{cur}-{ts}
        let parts: Vec<&str> = name.split('-').collect();
        let arm = parts[1].to_string();
        let seed = parts[2].trim_start_matches('s').to_string();
        let cur = parts[3].to_string();
        let reader = match anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")) {
            Ok(r) => r,
            Err(_) => { println!("{seed}\t{arm}\t{cur}\tNO-TELEMETRY"); continue; }
        };
        let idx = reader.chunk_index();
        let mut last_end = 0u64;
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
        let fail = std::fs::read_to_string(std::path::Path::new(&dir).join("metrics.json")).ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .map(|m| {
                let f = m.get("failures").cloned().unwrap_or(serde_json::Value::Null);
                if f.is_array() && !f.as_array().unwrap().is_empty() { "abort".to_string() } else { "complete".to_string() }
            }).unwrap_or_else(|| "?".to_string());

        let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let snap_at = |t: u64| snaps.iter().rev().find(|s| s.tick <= t);
        let s_end = snap_at(last_end).unwrap();
        // latch set = non-input neurons with z_latch == Some(1)
        let latchset = |s: &anima_telemetry::recorder::NetworkStateSnapshot| -> Vec<u32> {
            s.neurons.iter().filter(|n| n.id >= 24 && n.z_latch == Some(1)).map(|n| n.id).collect()
        };
        let le = latchset(s_end);
        let s2 = snap_at(last_end + 2000).unwrap();
        let l2 = latchset(s2);
        let inter = le.iter().filter(|x| l2.contains(x)).count();
        let union = le.len() + l2.len() - inter;
        let jac = if union == 0 { 1.0 } else { inter as f64 / union as f64 };
        let lf = le.len() as f64 / 52.0;

        let end = spikes.last().map(|s| s.0 + 1).unwrap_or(last_end);
        let sil: Vec<(u64, u32)> = spikes.iter().filter(|(t, n)| *t >= last_end && *n >= 24).cloned().collect();
        let s10 = sil.iter().filter(|(t, _)| *t < last_end + 10_000).count();
        let mut per: BTreeMap<u32, u64> = BTreeMap::new();
        for &(_, n) in &sil { *per.entry(n).or_default() += 1; }
        let sil_ms = end.saturating_sub(last_end).max(1);
        let max_hz = per.values().map(|&c| c as f64 * 1000.0 / sil_ms as f64).fold(0.0, f64::max);
        let regime = if sil.is_empty() { "silent" }
            else if per.len() <= 8 && max_hz > 48.0 { "pacemaker" }
            else if per.len() <= 8 { "sparse-core" } else { "irregular" };
        let mut topu: Vec<(u32, f32)> = s_end.neurons.iter().map(|n| (n.id, n.u_slow.unwrap_or(0.0))).collect();
        topu.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let m = (s10 as f64 + 1.0).log10();
        let m_str = if fail == "abort" { "inf".to_string() } else { format!("{m:.4}") };
        println!("{seed}\t{arm}\t{cur}\t{fail}\t{m_str}\t{s10}\t{regime}\tn{}:{:.2}\t{}\t{jac:.3}\t{max_hz:.0}\t{lf:.2}",
            topu[0].0, topu[0].1, le.len());
    }
}
