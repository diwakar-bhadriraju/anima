//! Evidence-pass analysis: per-pattern response vectors from chunk telemetry.
//! Computes late-S1 cross-pattern cosine per pair (A-B, B-C, A-C), matching
//! the E1 evidence-pass method.
use std::collections::BTreeMap;

fn main() {
    let dir = std::env::args().nth(1).expect("run dir");
    let tdir = std::path::Path::new(&dir).join("telemetry");
    let reader = anima_telemetry::TelemetryReader::open(&tdir).unwrap();
    let idx = reader.chunk_index();

    // Presentations: (pattern, stage, start, end).
    let mut pres: Vec<(String, String, u64, u64)> = Vec::new();
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                5 => {
                    if let Some(env) = row.envelope("e").ok() {
                        if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } = env.payload {
                            pres.push((pattern_id, stage, env.t, env.t + 500));
                        }
                    }
                }
                3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                _ => {}
            }
        }
    }
    pres.sort_by_key(|p| p.2);
    let s1: Vec<_> = pres.iter().filter(|p| p.1 == "S1").collect();
    let late_start = s1[s1.len() / 2].2;
    let input_n = 24u32;

    // Response vector per late-S1 presentation: neuron -> spike count.
    let mut vecs: Vec<(String, BTreeMap<u32, u64>)> = Vec::new();
    for p in &s1 {
        if p.2 < late_start { continue; }
        let mut v: BTreeMap<u32, u64> = BTreeMap::new();
        for &(t, n) in &spikes {
            if t >= p.2 && t < p.3 && n >= input_n {
                *v.entry(n).or_insert(0) += 1;
            }
        }
        vecs.push((p.0.clone(), v));
    }
    // Mean cross-cosine per pair.
    let mut pairs: BTreeMap<(String, String), Vec<f32>> = BTreeMap::new();
    for i in 0..vecs.len() {
        for j in (i+1)..vecs.len() {
            if vecs[i].0 == vecs[j].0 { continue; }
            let mut key = [vecs[i].0.clone(), vecs[j].0.clone()];
            key.sort();
            let c = cosine(&vecs[i].1, &vecs[j].1);
            if let Some(c) = c {
                pairs.entry((key[0].clone(), key[1].clone())).or_default().push(c);
            }
        }
    }
    for ((a, b), cs) in &pairs {
        let mean = cs.iter().sum::<f32>() / cs.len() as f32;
        println!("cross {a}-{b}: {mean:.3} (n={})", cs.len());
    }
    // Within for reference.
    let mut wc = Vec::new();
    for i in 0..vecs.len() {
        for j in (i+1)..vecs.len() {
            if vecs[i].0 != vecs[j].0 { continue; }
            if let Some(c) = cosine(&vecs[i].1, &vecs[j].1) { wc.push(c); }
        }
    }
    if !wc.is_empty() {
        println!("within-pattern mean: {:.3} (n={})", wc.iter().sum::<f32>() / wc.len() as f32, wc.len());
    }
}

fn cosine(a: &BTreeMap<u32, u64>, b: &BTreeMap<u32, u64>) -> Option<f32> {
    let keys: std::collections::BTreeSet<u32> = a.keys().chain(b.keys()).copied().collect();
    let dot: f64 = keys.iter().map(|k| (*a.get(k).unwrap_or(&0) * *b.get(k).unwrap_or(&0)) as f64).sum();
    let na: f64 = a.values().map(|&x| (x * x) as f64).sum::<f64>().sqrt();
    let nb: f64 = b.values().map(|&x| (x * x) as f64).sum::<f64>().sqrt();
    if na == 0.0 || nb == 0.0 { return None; }
    Some((dot / (na * nb)) as f32)
}
