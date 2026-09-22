//! Phase III Level-4 (docs/phase3/level4-decision.md): prediction index PI
//! on an alternating (il) run, measured separately for the POOL (internal
//! 24..64) and the OUTPUT (64..76).
//!
//! For each presentation of pattern p (successor s = the alternating
//! other), the LATE-gap state (last ~500 ms before the next presentation)
//! is a per-neuron spike-count vector. References: mean vector during
//! p-presentations (p_ref) and s-presentations (s_ref). Prediction index
//!   PI(group) = <gap_after_p, s_ref> - <gap_after_p, p_ref>
//! positive = the gap state ANTICIPATES the imminent successor.
//! Reported per pattern and group, plus gap-spike density.
use anima_telemetry::TelemetryReader;
use anima_telemetry::events::Payload;

fn cos(a: &[f32], b: &[f32]) -> f32 {
    let (mut n, mut na, mut nb) = (0.0f32, 0.0f32, 0.0f32);
    for (x, y) in a.iter().zip(b.iter()) {
        n += x * y;
        na += x * x;
        nb += y * y;
    }
    if na <= 0.0 || nb <= 0.0 { 0.0 } else { n / (na.sqrt() * nb.sqrt()) }
}

fn val(v: &[f32]) -> f32 { v.iter().sum() }

fn main() {
    for dir in std::env::args().skip(1) {
        let reader = TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut pres: Vec<(u64, String)> = Vec::new();
        let mut spk: Vec<(u64, u32)> = Vec::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(e) = row.envelope("e") {
                    match &e.payload {
                        Payload::StimulusPresented { pattern_id, .. } => {
                            pres.push((row.t, pattern_id.clone()));
                        }
                        Payload::Spike { n } => spk.push((row.t, n.0)),
                        _ => {}
                    }
                }
            }
        }
        pres.sort_by_key(|p| p.0);
        let name = dir.rsplit('/').next().unwrap().to_string();
        for (glabel, lo, hi) in [("pool", 24u32, 64u32), ("out", 64u32, 76u32)] {
            let n = (hi - lo) as usize;
            // references
            let mut refs: std::collections::BTreeMap<String, Vec<f32>> = Default::default();
            let mut cnt: std::collections::BTreeMap<String, u32> = Default::default();
            for (t, p) in &pres {
                let mut v = vec![0.0f32; n];
                for (st, s) in &spk {
                    if *st >= *t && *st < t + 500 && *s >= lo && *s < hi { v[(s - lo) as usize] += 1.0; }
                }
                let e = refs.entry(p.clone()).or_insert_with(|| vec![0.0f32; n]);
                for i in 0..n { e[i] += v[i]; }
                *cnt.entry(p.clone()).or_insert(0) += 1;
            }
            for (p, v) in refs.iter_mut() { let c = cnt[p] as f32; for x in v.iter_mut() { *x /= c; } }
            // per-pattern PI + late-gap density
            let mut pi: std::collections::BTreeMap<String, Vec<f32>> = Default::default();
            let mut gap_spikes = 0u64; let mut n_gaps = 0u64;
            for i in 0..pres.len().saturating_sub(1) {
                let (t0, p0) = (pres[i].0, pres[i].1.clone());
                let (t1, p1) = (pres[i + 1].0, pres[i + 1].1.clone());
                if t1 <= t0 || t1 - t0 > 2100 || p0 == p1 { continue; }
                let mut v = vec![0.0f32; n];
                for (st, s) in &spk {
                    if *st >= t1.saturating_sub(500) && *st < t1 && *s >= lo && *s < hi {
                        v[(s - lo) as usize] += 1.0;
                        gap_spikes += 1;
                    }
                }
                n_gaps += 1;
                let pr = refs.get(&p0).cloned().unwrap_or_else(|| vec![0.0f32; n]);
                let sr = refs.get(&p1).cloned().unwrap_or_else(|| vec![0.0f32; n]);
                pi.entry(p0.clone()).or_default().push(cos(&v, &sr) - cos(&v, &pr));
            }
            let fmt = |v: &Vec<f32>| if v.is_empty() { "n/a".into() } else { format!("n={} mean={:+.4}", v.len(), v.iter().sum::<f32>() / v.len() as f32) };
            for (p, v) in &pi { println!("{name}[{glabel}] PI(after {p} -> next): {}", fmt(v)); }
            let all: Vec<f32> = pi.values().flatten().cloned().collect();
            println!("{name}[{glabel}] PI all: {}   | late-gap spikes {gap_spikes}/{n_gaps} = {:.1}/gap",
                if all.is_empty() { "n/a".into() } else { format!("n={} mean={:+.4}", all.len(), all.iter().sum::<f32>() / all.len() as f32) },
                gap_spikes as f32 / n_gaps.max(1) as f32);
        }
        println!();
    }
}