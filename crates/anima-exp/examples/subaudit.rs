//! Future-memory substrate preservation audit (read-only, 2026-09-21).
//! Per committed run:
//!   - per snapshot: live unconsolidated afferent count + working mass by
//!     channel cohort (A: pre 0-7, C: 8-15, O: 16-23); consolidated counts
//!     by cohort; per-neuron W/P for M2 factor reconstruction.
//!   - SynapsePruned events: resolve the pruned synapse's pre cohort and
//!     weight-at-previous-snapshot (M2-crushed before prune, or not).
//!   - SynapseCreated candidate-permanence events: per cohort.
//! Output: SNAP/PRUNE/PERM rows.
use std::collections::BTreeMap;
fn main() {
    for dir in std::env::args().skip(1) {
        let snaps = anima_telemetry::recorder::read_snapshots(
            &std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let reader = anima_telemetry::TelemetryReader::open(
            &std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        // snapshot weight lookup by synapse id (latest snapshot at or before tick)
        let mut snap_at: Vec<(u64, BTreeMap<u32, f32>)> = Vec::new();
        for s in &snaps {
            let mut m = BTreeMap::new();
            for syn in &s.synapses {
                if let Some(w) = syn.w { m.insert(syn.id, w); }
            }
            snap_at.push((s.tick, m));
        }
        fn cohort(pre: u32) -> char {
            if pre < 8 { 'A' } else if pre < 16 { 'C' } else { 'O' }
        }
        println!("RUN {}", dir.rsplit('/').next().unwrap());
        for s in &snaps {
            let mut uc = [0u32; 3]; let mut um = [0.0f32; 3];
            let mut cc = [0u32; 3]; let mut cm = [0.0f32; 3];
            for syn in &s.synapses {
                if syn.pre >= 24 || syn.post < 24 || syn.post >= 76 { continue; }
                let c = (if syn.pre < 8 { 0 } else if syn.pre < 16 { 1 } else { 2 }) as usize;
                let w = syn.w.unwrap_or(0.0);
                if syn.consolidated { cc[c] += 1; cm[c] += w; }
                else { uc[c] += 1; um[c] += w; }
            }
            println!("SNAP\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}",
                s.tick, uc[0], uc[1], uc[2], um[0], um[1], um[2], cc[0], cc[1], cc[2], cm[0], cm[1], cm[2]);
        }
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(env) = row.envelope("e") {
                    match &env.payload {
                        anima_telemetry::events::Payload::SynapsePruned { syn, reason } => {
                            // find weight in the latest snapshot <= row.t
                            let mut w = -1.0f32; let mut pre = 999u32;
                            for (t, m) in snap_at.iter().rev() {
                                if *t <= row.t { if let Some(&wv) = m.get(&syn.0) { w = wv; } break; }
                            }
                            // pre from the same snapshot map: need synapse list; scan snaps backwards
                            for (t, s) in snaps.iter().enumerate().rev() {
                                if s.tick <= row.t {
                                    if let Some(sy) = s.synapses.iter().find(|x| x.id == syn.0) {
                                        pre = sy.pre; break;
                                    }
                                    break;
                                }
                            }
                            if pre == 999 { continue; } // no record (pre-v2)
                            println!("PRUNE\t{}\t{}\t{}\t{}\t{:.4}", row.t, cohort(pre), pre, reason.trigger, w);
                        }
                        anima_telemetry::events::Payload::SynapseCreated { syn: _, pre, post: _, w: _, reason } => {
                            if reason.trigger == "candidate-permanence" {
                                println!("PERM\t{}\t{}\t{}", row.t, cohort(pre.0), pre.0);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        println!();
    }
}
