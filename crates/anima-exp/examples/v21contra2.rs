//! Post-V2.1 exploration P2b: antecedent-conditioned POST-presentation responses during drive.
//!
//! In xp2 runs, each A or C presentation (500 ms) is followed by 1500 ms of no input.
//! The spikes in that off-window are driven by internal state (u, recurrence), not by
//! immediate afferent drive. Readouts:
//!  (1) post-window spike count per antecedent class (A-off vs C-off), late-drive half;
//!  (2) post-window population VECTOR (which neurons fire) — cosine separation A-off vs C-off;
//!  (3) trajectory over drive: does separation grow with presentations?
//! Deterministic, read-only.
use std::collections::BTreeMap;

fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
    let idx = reader.chunk_index();
    let mut pres: Vec<(String, u64)> = Vec::new();
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                5 => {
                    if let Some(env) = row.envelope("e").ok() {
                        if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } = env.payload {
                            if stage == "S1" { pres.push((pattern_id, env.t)); }
                        }
                    }
                }
                3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                _ => {}
            }
        }
    }
    pres.sort_by_key(|p| p.1);
    spikes.sort_by_key(|s| s.0);

    // Off-window per presentation: [t+500, t+2000), non-input neurons.
    let mut recs: Vec<(String, u64, BTreeMap<u32, u64>)> = Vec::new();
    for (p, t) in &pres {
        let mut v: BTreeMap<u32, u64> = BTreeMap::new();
        for &(ts, n) in &spikes {
            if ts >= t + 500 && ts < t + 2000 && n >= 24 { *v.entry(n).or_default() += 1; }
        }
        recs.push((p.clone(), *t, v));
    }
    let half = recs.len() / 2;
    for (label, slice) in [("early", &recs[..half]), ("late", &recs[half..])] {
        let a: Vec<&BTreeMap<u32,u64>> = slice.iter().filter(|(p, _, _)| p == "A").map(|(_, _, v)| v).collect();
        let c: Vec<&BTreeMap<u32,u64>> = slice.iter().filter(|(p, _, _)| p == "C").map(|(_, _, v)| v).collect();
        let tot = |vs: &Vec<&BTreeMap<u32,u64>>| -> f64 {
            let n = vs.len().max(1) as f64;
            vs.iter().map(|v| v.values().sum::<u64>() as f64).sum::<f64>() / n
        };
        let mean_vec = |vs: &Vec<&BTreeMap<u32,u64>>| -> BTreeMap<u32, f64> {
            let mut m: BTreeMap<u32, f64> = BTreeMap::new();
            for v in vs.iter() { for (&k, &x) in v.iter() { *m.entry(k).or_default() += x as f64; } }
            let n = vs.len().max(1) as f64;
            for x in m.values_mut() { *x /= n; }
            m
        };
        let cos = |a: &BTreeMap<u32,f64>, b: &BTreeMap<u32,f64>| -> f64 {
            let keys: std::collections::BTreeSet<u32> = a.keys().chain(b.keys()).copied().collect();
            let d: f64 = keys.iter().map(|k| a.get(k).unwrap_or(&0.0) * b.get(k).unwrap_or(&0.0)).sum();
            let na = a.values().map(|x| x.powi(2)).sum::<f64>().sqrt();
            let nb = b.values().map(|x| x.powi(2)).sum::<f64>().sqrt();
            if na > 0.0 && nb > 0.0 { d / (na * nb) } else { 0.0 }
        };
        let (ma, mc) = (mean_vec(&a), mean_vec(&c));
        println!("[{label}] n=({},{}) | off-window spikes/rep: A={:.1} C={:.1} | vec-cos(A,C)={:.4}",
            a.len(), c.len(), tot(&a), tot(&c), cos(&ma, &mc));
        // trajectory in quarters
        let q = slice.len() / 4;
        for i in 0..4 {
            let s = &slice[i * q..(i + 1) * q.min(slice.len() - i * q).max(1)];
            let a: Vec<&BTreeMap<u32,u64>> = s.iter().filter(|(p, _, _)| p == "A").map(|(_, _, v)| v).collect();
            let c: Vec<&BTreeMap<u32,u64>> = s.iter().filter(|(p, _, _)| p == "C").map(|(_, _, v)| v).collect();
            if a.is_empty() || c.is_empty() { continue; }
            println!("   q{}: A={:.1} C={:.1} cos={:.4}", i + 1, tot(&a), tot(&c), cos(&mean_vec(&a), &mean_vec(&c)));
        }
    }
}
