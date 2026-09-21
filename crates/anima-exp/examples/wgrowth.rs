//! Read-only second-block weight-growth decomposition (docs authority: the
//! accepted source-correction record). Per committed run:
//!   PERM  t pre post      - candidate-permanence (full synapse identity)
//!   WTRAJ t pre post w res  - per-synapse weight from snapshots (live)
//!   PST   pres pres_t pat nspikes   - post spikes per presentation (neuron id 0 = count below)
//!   PRUNE t pre           - prune events affecting input channels (full rows)
//!   SW    t syn pre post d   - strengthen/weaken events (synapse identity via snapshot map)
use std::collections::BTreeMap;
fn main() {
    for dir in std::env::args().skip(1) {
        let snaps = anima_telemetry::recorder::read_snapshots(
            &std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let reader = anima_telemetry::TelemetryReader::open(
            &std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        // snapshot synapse map: id -> (pre, post, w, cons) at latest tick <= t
        let mut pres: Vec<(String, u64)> = Vec::new();
        let mut perms: Vec<(u64, u32, u32)> = Vec::new();
        let mut prunes: Vec<(u64, u32, String)> = Vec::new();
        let mut sw: Vec<(u64, u32, f32)> = Vec::new(); // (t, syn, delta)
        let mut spikes: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(env) = row.envelope("e") {
                    match &env.payload {
                        anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } => {
                            if stage == "S1" { pres.push((pattern_id.clone(), row.t)); }
                        }
                        anima_telemetry::events::Payload::SynapseCreated { syn: _, pre, post, w: _, reason } => {
                            if reason.trigger == "candidate-permanence" {
                                perms.push((row.t, pre.0, post.0));
                            }
                        }
                        anima_telemetry::events::Payload::SynapsePruned { syn, reason } => {
                            prunes.push((row.t, syn.0, reason.trigger.clone()));
                        }
                        anima_telemetry::events::Payload::SynapseStrengthened { syn, delta } => {
                            sw.push((row.t, syn.0, delta.unwrap_or(0.0)));
                        }
                        anima_telemetry::events::Payload::SynapseWeakened { syn, delta } => {
                            sw.push((row.t, syn.0, delta.unwrap_or(0.0)));
                        }
                        anima_telemetry::events::Payload::Spike { n } => {
                            spikes.entry(row.t).or_default().push(n.0);
                        }
                        _ => {}
                    }
                }
            }
        }
        pres.sort_by_key(|p| p.1);
        println!("RUN {}", dir.rsplit('/').next().unwrap());
        for (t, pre, post) in &perms {
            println!("PERM\t{}\t{}\t{}", t, pre, post);
        }
        // per-synapse weight trajectory: for input-channel synapses (pre<24),
        // sample w at every snapshot
        for s in &snaps {
            let mut line = Vec::new();
            for syn in &s.synapses {
                if syn.pre < 24 {
                    line.push(format!("{}:{}:{:.4}:{}", syn.pre, syn.post, syn.w.unwrap_or(0.0),
                        if syn.consolidated { 1 } else { 0 }));
                }
            }
            if !line.is_empty() {
                println!("WT\t{}\t{}", s.tick, line.join(","));
            }
        }
        for (t, syn, d) in &sw {
            // resolve syn pre/post from nearest snapshot at or before t
            let mut pre = 999u32; let mut post = 999u32;
            for s in snaps.iter().rev() {
                if s.tick <= *t {
                    if let Some(sy) = s.synapses.iter().find(|x| x.id == *syn) {
                        pre = sy.pre; post = sy.post;
                    }
                    break;
                }
            }
            if pre != 999 { println!("SW\t{}\t{}\t{}\t{}\t{:.4}", t, syn, pre, post, d); }
        }
        for (t, syn, reason) in &prunes {
            // pre from snapshot
            let mut pre = 999u32;
            for s in snaps.iter().rev() {
                if s.tick <= *t {
                    if let Some(sy) = s.synapses.iter().find(|x| x.id == *syn) {
                        pre = sy.pre;
                    }
                    break;
                }
            }
            println!("PRUNE\t{}\t{}\t{}", t, pre, reason);
        }
        // post spikes per presentation (internal neuron total)
        for (i, (pat, t0)) in pres.iter().enumerate() {
            let mut n = 0u32;
            for t in *t0..t0 + 500 {
                if let Some(ids) = spikes.get(&t) {
                    n += ids.iter().filter(|&&x| x >= 24 && x < 76).count() as u32;
                }
            }
            println!("PST\t{}\t{}\t{}\t{}", pat, i + 1, t0, n);
        }
        // C-channel pre spikes per presentation (channels 8..16 firing count)
        for (i, (pat, t0)) in pres.iter().enumerate() {
            let mut n = 0u32;
            for t in *t0..t0 + 500 {
                if let Some(ids) = spikes.get(&t) {
                    n += ids.iter().filter(|&&x| x >= 8 && x < 16).count() as u32;
                }
            }
            println!("PRE_C\t{}\t{}\t{}", pat, i + 1, n);
        }
        println!();
    }
}
