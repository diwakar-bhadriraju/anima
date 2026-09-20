//! Read-only: per-presentation per-neuron protected-current ratio
//! R = Ip/(Ip+Iw) where Ip = current via consolidated afferents,
//! Iw = current via unconsolidated (working) afferents, computed from
//! total input current (all fired-channel exc afferents x amplitude w).
//! Reports MEAN over neurons and 25/50/75 percentiles.
use std::collections::{BTreeMap, BTreeSet};
fn main() {
    for dir in std::env::args().skip(1) {
        let snaps = anima_telemetry::recorder::read_snapshots(
            &std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let reader = anima_telemetry::TelemetryReader::open(
            &std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut pres: Vec<(String, u64)> = Vec::new();
        let mut spk: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(env) = row.envelope("e") {
                    match &env.payload {
                        anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } => {
                            if stage == "S1" { pres.push((pattern_id.clone(), row.t)); }
                        }
                        anima_telemetry::events::Payload::Spike { n } => {
                            spk.entry(row.t).or_default().push(n.0);
                        }
                        _ => {}
                    }
                }
            }
        }
        pres.sort_by_key(|p| p.1);
        println!("RUN {}", dir.rsplit('/').next().unwrap());
        let mut si = 0usize;
        let amp = 52.0f32;
        for (pi, (pat, t0)) in pres.iter().enumerate() {
            while si + 1 < snaps.len() && snaps[si + 1].tick <= *t0 { si += 1; }
            let mut fired: BTreeSet<u32> = BTreeSet::new();
            for t in *t0..t0 + 500 {
                if let Some(ids) = spk.get(&t) { for &n in ids { if n < 24 { fired.insert(n); } } }
            }
            let snap = &snaps[si];
            let mut ip = vec![0.0f32; 52];
            let mut itot = vec![0.0f32; 52];
            for syn in &snap.synapses {
                if !syn.plastic || syn.post < 24 || syn.post >= 76 { continue; }
                if fired.contains(&syn.pre) {
                    let i = syn.post as usize - 24;
                    let w = syn.w.unwrap_or(0.0);
                    itot[i] += amp * w;
                    if syn.consolidated { ip[i] += amp * w; }
                }
            }
            let mut r = Vec::new();
            for i in 0..52 {
                let t = itot[i];
                if t > 0.0 { r.push(ip[i] / t); }
            }
            if r.is_empty() { continue; }
            r.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let q = |q: f32| r[((r.len() - 1) as f32 * q) as usize];
            let mean = r.iter().sum::<f32>() / r.len() as f32;
            println!("RA\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{}", pat, pi + 1, q(0.25), q(0.5), q(0.75), mean, r.len());
        }
        println!();
    }
}
