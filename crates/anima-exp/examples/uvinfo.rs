//! V2.1 u-information reproducibility instrument (read-only).
//! Answers: does the persistent state u carry sensory-specific
//! (A-vs-C) information after the stimulus is gone?
//! For each presentation in S1 (pattern known), capture u-vectors at:
//!   during: last snapshot at/before presentation end (t+500)
//!   post:   first snapshot at/or after t+550 (immediate post-offset)
//!   late:   off-window snapshot at t+2000 (next presentation start)
//! Outputs per window class: mean u vector over presentations of each
//! pattern; cosine(A,C), L2 distance(A,C), norms, top-10 overlap.
//! Also endogenous off-window rate + Fano.
use std::collections::BTreeMap;

fn main() {
    for dir in std::env::args().skip(1) {
        let name = dir.rsplit('/').next().unwrap().to_string();
        let tdir = std::path::Path::new(&dir).join("telemetry");
        let reader = match anima_telemetry::TelemetryReader::open(&tdir) {
            Ok(r) => r, Err(_) => { println!("{name}\tno-telemetry"); continue; }
        };
        let idx = reader.chunk_index();
        let mut pres: Vec<(String, u64)> = Vec::new();
        let mut spikes: Vec<(u64, u32)> = Vec::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                match row.kind {
                    5 => { if let Some(env) = row.envelope("e").ok() {
                        if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } = env.payload {
                            if stage == "S1" { pres.push((pattern_id, env.t)); } } } }
                    3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                    _ => {}
                }
            }
        }
        pres.sort_by_key(|p| p.1);
        spikes.sort_by_key(|s| s.0);
        let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        if snaps.is_empty() { println!("{name}\tno-snaps"); continue; }
        let ticks: Vec<u64> = snaps.iter().map(|s| s.tick).collect();
        let uvec = |t: u64| -> Option<Vec<f32>> {
            let st = ticks.iter().rev().find(|&&x| x <= t)?;
            let s = snaps.iter().find(|s| s.tick == *st)?;
            Some(s.neurons.iter().filter(|n| n.id >= 24).map(|n| n.u_slow.unwrap_or(0.0)).collect())
        };
        let snap_after = |t: u64| -> Option<Vec<f32>> {
            let st = ticks.iter().find(|&&x| x >= t)?;
            let s = snaps.iter().find(|s| s.tick == *st)?;
            Some(s.neurons.iter().filter(|n| n.id >= 24).map(|n| n.u_slow.unwrap_or(0.0)).collect())
        };
        // per-presentation u snapshots at 3 windows, bucketed by pattern
        let mut buckets: BTreeMap<(String, &str), Vec<Vec<f32>>> = BTreeMap::new();
        for (p, t) in &pres {
            let end = t + 500;
            if let Some(v) = uvec(end) { buckets.entry((p.clone(), "during")).or_default().push(v); }
            if let Some(v) = snap_after(end + 50) { buckets.entry((p.clone(), "post")).or_default().push(v); }
            if let Some(v) = uvec(end + 1500) { buckets.entry((p.clone(), "late")).or_default().push(v); }
        }
        let meanv = |vs: &Vec<Vec<f32>>| -> Vec<f32> {
            let n = vs.len().max(1);
            (0..52).map(|i| vs.iter().map(|v| v[i]).sum::<f32>() / n as f32).collect()
        };
        let cos = |a: &[f32], b: &[f32]| -> f64 {
            let d: f64 = a.iter().zip(b).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
            let na = (a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
            let nb = (b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
            if na > 0.0 && nb > 0.0 { d / (na * nb) } else { 0.0 }
        };
        let norm = |v: &[f32]| -> f64 { (v.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt() };
        let top10 = |v: &[f32]| -> Vec<u32> {
            let mut ids: Vec<(u32, f32)> = v.iter().enumerate().map(|(i, &x)| (i as u32 + 24, x)).collect();
            ids.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            ids.iter().take(10).map(|(i, _)| *i).collect()
        };
        println!("RUN {name}");
        println!("  pattern counts: A={} C={} (S1 presentations)", pres.iter().filter(|(p, _)| p == "A").count(), pres.iter().filter(|(p, _)| p == "C").count());
        for w in ["during", "post", "late"] {
            let va = buckets.get(&("A".into(), w));
            let vc = buckets.get(&("C".into(), w));
            match (va, vc) {
                (Some(a), Some(c)) if !a.is_empty() && !c.is_empty() => {
                    let (ma, mc) = (meanv(a), meanv(c));
                    let d: f64 = ma.iter().zip(&mc).map(|(x, y)| (x - y).powi(2)).sum::<f32>() as f64;
                    let ta = top10(&ma); let tc = top10(&mc);
                    let inter = ta.iter().filter(|x| tc.contains(x)).count();
                    println!("  [{w}] u-cos(A,C)={:.4} u-dist={:.2} |A|={:.3} |C|={:.3} norm-ratio={:.3} top10-overlap(J)={:.3} (nA={} nC={})",
                        cos(&ma, &mc), d.sqrt(), norm(&ma), norm(&mc), norm(&ma) / norm(&mc).max(1e-9),
                        inter as f64 / 10.0, a.len(), c.len());
                }
                _ => println!("  [{w}] insufficient data"),
            }
        }
        // endogenous activity after last presentation: S2 silence
        let last_end = pres.last().map(|(_, t)| *t + 500).unwrap_or(0);
        let end = spikes.last().map(|s| s.0 + 1).unwrap_or(last_end);
        let sil: Vec<u64> = spikes.iter().filter(|(t, n)| *t >= last_end + 2000 && *n >= 24).map(|(t, _)| *t).collect();
        let sil_len = end.saturating_sub(last_end + 2000).max(1);
        let rate = sil.len() as f64 / (sil_len as f64 / 1000.0);
        let nb = (sil_len / 1000).max(1) as usize;
        let mut bins = vec![0.0f64; nb];
        for t in &sil { let b = ((t - (last_end + 2000)) / 1000) as usize; if b < bins.len() { bins[b] += 1.0; } }
        let m = bins.iter().sum::<f64>() / bins.len() as f64;
        let fano = if m > 0.0 { bins.iter().map(|x| (x - m).powi(2)).sum::<f64>() / bins.len() as f64 / m } else { 0.0 };
        println!("  endogenous silence (post S1): rate={rate:.2} sp/s, Fano={fano:.3}, spikes={}", sil.len());
        println!();
    }
}