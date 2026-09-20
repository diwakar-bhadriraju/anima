
fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
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
    let mut va: std::collections::BTreeMap<u32, f64> = std::collections::BTreeMap::new();
    let mut vc: std::collections::BTreeMap<u32, f64> = std::collections::BTreeMap::new();
    for (p, t) in &pres {
        let v = if p == "A" { &mut va } else { &mut vc };
        for &(ts, n) in &spikes {
            if ts >= *t && ts < *t + 500 && n >= 24 { *v.entry(n).or_default() += 1.0; }
        }
    }
    let cos = |a: &std::collections::BTreeMap<u32,f64>, b: &std::collections::BTreeMap<u32,f64>| -> f64 {
        let keys: std::collections::BTreeSet<u32> = a.keys().chain(b.keys()).copied().collect();
        let d: f64 = keys.iter().map(|k| a.get(k).unwrap_or(&0.0) * b.get(k).unwrap_or(&0.0)).sum();
        let na = a.values().map(|x| x.powi(2)).sum::<f64>().sqrt();
        let nb = b.values().map(|x| x.powi(2)).sum::<f64>().sqrt();
        if na > 0.0 && nb > 0.0 { d / (na * nb) } else { 0.0 }
    };
    println!("{} A-cos-C during-window (full): {:.4}  |A|={} |C|={}", dir.rsplit('/').next().unwrap(), cos(&va, &vc), va.len(), vc.len());
}
