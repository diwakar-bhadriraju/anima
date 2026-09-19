//! During-presentation response vectors: A-window vs C-window population
//! vectors (which non-input neurons fire during the 500ms presentation),
//! cosine separation, per quarter of the drive. Mixed-arm runs only.
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
    let half = pres.len() / 2;
    for (label, slice) in [("early", &pres[..half]), ("late", &pres[half..])] {
        let mut va: BTreeMap<u32, f64> = BTreeMap::new();
        let mut vc: BTreeMap<u32, f64> = BTreeMap::new();
        for (p, t) in slice {
            let v = if p == "A" { &mut va } else { &mut vc };
            for &(ts, n) in &spikes {
                if ts >= *t && ts < *t + 500 && n >= 24 { *v.entry(n).or_default() += 1.0; }
            }
        }
        let keys: std::collections::BTreeSet<u32> = va.keys().chain(vc.keys()).copied().collect();
        let d: f64 = keys.iter().map(|k| va.get(k).unwrap_or(&0.0) * vc.get(k).unwrap_or(&0.0)).sum();
        let na = (va.values().map(|x| x.powi(2)).sum::<f64>()).sqrt();
        let nc = (vc.values().map(|x| x.powi(2)).sum::<f64>()).sqrt();
        let cos = if na > 0.0 && nc > 0.0 { d / (na * nc) } else { 0.0 };
        // active-set overlap
        let sa: std::collections::BTreeSet<u32> = va.keys().copied().collect();
        let sc: std::collections::BTreeSet<u32> = vc.keys().copied().collect();
        let inter = sa.intersection(&sc).count();
        println!("[{label}] during-window A-vs-C cos={cos:.4} | |A set|={} |C set|={} overlap={}",
            sa.len(), sc.len(), inter);
    }
}
