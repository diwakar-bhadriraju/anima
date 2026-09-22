//! Phase III Level-4 (docs/phase3/level4-decision.md): prediction index PI
//! on an alternating (il) run.
//!
//! For each presentation of pattern p (successor s = the alternating other),
//! compute the LATE-gap internal state (last ~500 ms before the next
//! presentation) as a 52-dim per-neuron spike-count vector. Reference
//! vectors: mean per-neuron rate vector during p presentations (p_ref) and
//! during s presentations (s_ref). Prediction index
//!   PI(p) = <gap_after_p, s_ref> - <gap_after_p, p_ref>
//! positive when the gap state ANTICIPATES the imminent successor.
//! Reported per stage-group (early vs late plateau) and per seed.
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

fn main() {
    for dir in std::env::args().skip(1) {
        let reader = TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut pres: Vec<(u64, String)> = Vec::new(); // (t, pattern_id)
        let mut spk: Vec<(u64, u32)> = Vec::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(e) = row.envelope("e") {
                    match &e.payload {
                        Payload::StimulusPresented { pattern_id, .. } => {
                            pres.push((row.t, pattern_id.clone()));
                        }
                        Payload::Spike { n } => {
                            if (24..76).contains(&n.0) { spk.push((row.t, n.0)); }
                        }
                        _ => {}
                    }
                }
            }
        }
        pres.sort_by_key(|p| p.0);
        // references: mean per-neuron vector over presentation windows
        let mut refs: std::collections::BTreeMap<String, Vec<f32>> = Default::default();
        let mut counts: std::collections::BTreeMap<String, u32> = Default::default();
        for (t, p) in &pres {
            let mut v = vec![0.0f32; 52];
            for (st, n) in &spk {
                if *st >= *t && *st < t + 500 { let j = (n - 24) as usize; if j < 52 { v[j] += 1.0; } }
            }
            let e = refs.entry(p.clone()).or_insert_with(|| vec![0.0f32; 52]);
            for i in 0..52 { e[i] += v[i]; }
            *counts.entry(p.clone()).or_insert(0) += 1;
        }
        for (p, v) in refs.iter_mut() { let c = counts[p] as f32; for x in v.iter_mut() { *x /= c; } }
        // pairs: for each presentation with a successor within 2000ms
        let mut alist: Vec<f32> = Vec::new();
        let mut clist: Vec<f32> = Vec::new();
        for i in 0..pres.len().saturating_sub(1) {
            let (t0, p0) = (pres[i].0, pres[i].1.clone());
            let (t1, p1) = (pres[i + 1].0, pres[i + 1].1.clone());
            if t1 <= t0 || t1 - t0 > 2100 || p0 == p1 { continue; }
            // late gap = last 500ms before the next presentation
            let v = {
                let mut v = vec![0.0f32; 52];
                for (st, n) in &spk {
                    if *st >= t1.saturating_sub(500) && *st < t1 {
                        let j = (n - 24) as usize; if j < 52 { v[j] += 1.0; }
                    }
                }
                v
            };
            let pr = refs.get(&p0).cloned().unwrap_or_else(|| vec![0.0f32; 52]);
            let sr = refs.get(&p1).cloned().unwrap_or_else(|| vec![0.0f32; 52]);
            let pi = cos(&v, &sr) - cos(&v, &pr);
            if il_rank(&p0) % 2 == 0 { alist.push(pi); } else { clist.push(pi); }
        }
        // diagnostics: late-gap spike density + ref magnitudes
        let mut gap_spikes = 0u64; let mut n_gaps = 0u64;
        for i in 0..pres.len().saturating_sub(1) {
            let (t0, p0) = (pres[i].0, pres[i].1.clone());
            let (t1, _) = (pres[i + 1].0, pres[i + 1].1.clone());
            if t1 <= t0 || t1 - t0 > 2100 || p0 == pres[i + 1].1 { continue; }
            for (st, _) in &spk { if *st >= t1.saturating_sub(500) && *st < t1 { gap_spikes += 1; } }
            n_gaps += 1;
        }
        let refmag: f32 = refs.values().map(|v| v.iter().sum::<f32>()).sum();
        let name = dir.rsplit('/').next().unwrap().to_string();
        println!("  [diag] late-gap internal spikes: {gap_spikes}/{n_gaps} gaps = {:.1}/gap (refmag={refmag:.1})",
            gap_spikes as f32 / n_gaps.max(1) as f32);
        let rep = |v: &[f32]| -> String {
            if v.is_empty() { return "n/a".into() }
            let m = v.iter().sum::<f32>() / v.len() as f32;
            format!("n={} mean={m:+.4}", v.len())
        };
        println!("{name}");
        println!("  PI(after A -> C anticipate): {}", rep(&alist));
        println!("  PI(after C -> A anticipate): {}", rep(&clist));
        let all: Vec<f32> = alist.iter().chain(clist.iter()).cloned().collect();
        println!("  PI all: {}", rep(&all));
    }
}

/// Alternating cadence rank (odd/even) — placeholder mapping first
/// distinct pattern to rank 0. Pattern ids are "A"/"C" in these runs.
fn il_rank(p: &str) -> usize {
    match p {
        "A" => 0,
        "C" => 1,
        _ => 0,
    }
}