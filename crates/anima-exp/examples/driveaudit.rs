//! Read-only first-exposure drive decomposition (2026-09-21).
//! Per presentation, per neuron (24..76), from the snapshot at or before
//! the presentation tick + the tick spike record:
//!   IpA/IwA/IpC/IwC : afferent current from fired input channels, split
//!                    by cohort (A<8, C 8..16) and consolidated flag
//!   Irec : current from LIVE EXCITATORY internal pres that fired in the
//!          presentation window (plastic=true)
//!   Iinh : current from INHIBITORY (M6, plastic=false) internal pres that
//!          fired (delivered negative)
//!   Ialb : total afferent (Ip+Iw, all cohorts); posts: internal spikes
//!   vmed : median membrane potential of internal neurons at the snapshot
//!          (coarse, 1 s cadence — after-window state; reported for context
//!          only, NOT used as the primary response measure)
//! Totals are means over the 52 internal/output neurons.
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
            let snap = &snaps[si];
            // neurons that fired within the window (internal)
            let mut internal_fired: BTreeSet<u32> = BTreeSet::new();
            let mut ch_fired: BTreeSet<u32> = BTreeSet::new();
            for t in *t0..t0 + 500 {
                if let Some(ids) = spk.get(&t) {
                    for &n in ids {
                        if n < 24 { ch_fired.insert(n); }
                        else if n < 76 { internal_fired.insert(n); }
                    }
                }
            }
            let mut ipA = vec![0.0f32; 52]; let mut iwA = vec![0.0f32; 52];
            let mut ipC = vec![0.0f32; 52]; let mut iwC = vec![0.0f32; 52];
            let mut irec = vec![0.0f32; 52]; let mut iinh = vec![0.0f32; 52];
            for syn in &snap.synapses {
                if syn.post < 24 || syn.post >= 76 { continue; }
                let i = syn.post as usize - 24;
                // afferent: pre < 24, fired channel
                if syn.pre < 24 {
                    if !ch_fired.contains(&syn.pre) { continue; }
                    let w = syn.w.unwrap_or(0.0);
                    if syn.consolidated { if syn.pre < 8 { ipA[i] += amp*w; } else { ipC[i] += amp*w; } }
                    else { if syn.pre < 8 { iwA[i] += amp*w; } else { iwC[i] += amp*w; } }
                } else if syn.pre < 76 && internal_fired.contains(&syn.pre) {
                    let w = syn.w.unwrap_or(0.0);
                    if syn.plastic { irec[i] += amp*w; } else { iinh[i] -= amp*w; }
                }
            }
            let n = 52.0f32;
            let m = |v: &Vec<f32>| v.iter().sum::<f32>() / n;
            let posts = internal_fired.len() as f32 / n;
            let vs: Vec<f32> = snap.neurons.iter()
                .filter(|x| x.id >= 24 && x.id < 76)
                .map(|x| x.v.unwrap_or(0.0)).collect();
            let mut vs = vs; vs.sort_by(|a,b| a.partial_cmp(b).unwrap());
            let vmed = vs[vs.len()/2];
            println!("D\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}",
                pat, pi+1, t0,
                m(&ipA), m(&iwA), m(&ipC), m(&iwC),
                m(&irec), m(&iinh),
                m(&ipA)+m(&iwA)+m(&ipC)+m(&iwC),
                posts, vmed);
        }
        println!();
    }
}
