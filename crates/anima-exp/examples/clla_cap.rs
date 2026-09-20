//! CLLA capability measurement (read-only, docs/anima-clla-protocol.md).
//! Computes from committed run artifacts:
//!   - per-snapshot per-neuron protected P, working W (and Ptotal, Wtotal)
//!   - drive-end (t=84000) per-neuron raw A/C channel masses (F1)
//!   - per-presentation during-window [t, t+500) internal count vectors
//!     (reps 1..N per pattern; F2/F6 material)
//!   - per-snapshot total protected A-mass and C-mass (F7: peak vs drive-end)
//!   - drive-end per-neuron P and W columns for F3b (median computations)
//! Output: TSV rows; analysis in python.
use std::collections::BTreeMap;

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
                            if n.0 >= 24 && n.0 < 76 {
                                spk.entry(row.t).or_default().push(n.0 - 24);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        pres.sort_by_key(|p| p.1);
        // Per-pattern ordinal -> window vector [t, t+500)
        let mut pat_ord: BTreeMap<String, u32> = BTreeMap::new();
        println!("VECT\t{}", dir.rsplit('/').next().unwrap());
        for (pat, t0) in &pres {
            let ord = pat_ord.entry(pat.clone()).or_insert(0);
            *ord += 1;
            let mut v = vec![0u32; 52];
            for t in *t0..t0 + 500 {
                if let Some(ids) = spk.get(&t) {
                    for &i in ids { v[i as usize] += 1; }
                }
            }
            println!("{}\t{}\t{}", pat, ord, v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","));
        }
        // Snapshots: per-neuron P/W at every snapshot + protected A/C totals
        for s in &snaps {
            let mut p = vec![0.0f32; 52];
            let mut w = vec![0.0f32; 52];
            let mut pa = 0.0f32;
            let mut pc = 0.0f32;
            for syn in &s.synapses {
                if syn.plastic && syn.post >= 24 && syn.post < 76 {
                    let i = syn.post as usize - 24;
                    let wv = syn.w.unwrap_or(0.0);
                    if syn.consolidated {
                        p[i] += wv;
                        if syn.pre < 8 { pa += wv; }
                        else if syn.pre < 16 { pc += wv; }
                    } else {
                        w[i] += wv;
                    }
                }
            }
            let ptot: f32 = p.iter().sum();
            let wtot: f32 = w.iter().sum();
            println!("SNAP\t{}\t{ptot:.4}\t{wtot:.4}\t{pa:.4}\t{pc:.4}\t{}", s.tick,
                p.iter().map(|x| format!("{x:.4}")).collect::<Vec<_>>().join(","),
            );
            let _ = w;
        }
        println!();
    }
}