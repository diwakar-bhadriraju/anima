//! m1audit: read-only M1-track addressing audit (docs/x-phase2-m1-audit.md).
//! Per run, per relevant snapshot: per-cohort live WORKING weight sums
//! (A pre<8, C 8..16), per-track working sums, P_tot, and the surviving
//! second-cohort working mass split by tag (the M1-default-0 consequence).
//! Plus the first-exposure window's working CURRENT per cohort (from
//! telemetry spikes + the snapshot at/before the presentation) — the
//! quantities the retag counterfactual needs.
use anima_telemetry::recorder::read_snapshots;
use anima_telemetry::TelemetryReader;
use anima_telemetry::events::Payload;

fn main() {
    for dir in std::env::args().skip(1) {
        let snaps = read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let reader = TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut pres: Vec<(u64, String)> = Vec::new();
        let mut spk: std::collections::BTreeMap<u64, Vec<u32>> = std::collections::BTreeMap::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(env) = row.envelope("e") {
                    match &env.payload {
                        Payload::StimulusPresented { pattern_id, stage } => {
                            if stage == "S1" { pres.push((row.t, pattern_id.clone())); }
                        }
                        Payload::Spike { n } => {
                            spk.entry(row.t).or_default().push(n.0);
                        }
                        _ => {}
                    }
                }
            }
        }
        pres.sort_by_key(|p| p.0);
        let name = dir.rsplit('/').next().unwrap().to_string();
        println!("H\t{name}");
        let cohort = |pre: u32| -> usize { if pre < 8 { 0 } else if pre < 16 { 1 } else { 2 } };
        // snapshots of interest: 44000 (pre second block) and each pres start
        let mut taus = vec![44_000u64];
        for (t, _) in pres.iter().filter(|(_, p)| p == "A" || p == "C" || p == "B") {
            if pres.iter().position(|x| x.0 == *t).unwrap_or(99) == 21 {
                taus.push(*t);
            }
        }
        for &t in &taus {
            let Some(snap) = snaps.iter().rev().find(|s| s.tick <= t) else { continue };
            let mut wc = [0.0f32; 3];       // cohort working sums
            let mut wt = [0.0f32; 2];       // track working sums
            let mut wc_tag = [[0.0f32; 2]; 3]; // cohort x track working sums
            let mut p_tot = 0.0f32;
            for s in &snap.synapses {
                if s.post >= 76 || s.w.is_none() { continue; }
                let w = s.w.unwrap();
                if !s.plastic { continue; }
                if s.consolidated {
                    if s.pre < 24 { p_tot += w; }
                    continue;
                }
                if s.pre < 24 {
                    let c = cohort(s.pre);
                    let tag = (s.track as usize).min(1);
                    wc[c] += w;
                    wt[tag] += w;
                    wc_tag[c][tag] += w;
                }
            }
            println!("M\t{t}\tA_w={:.3}\tC_w={:.3}\tB_w={:.3}\tT0={:.3}\tT1={:.3}\tC_tag0={:.3}\tC_tag1={:.3}\tP={:.3}",
                wc[0], wc[1], wc[2], wt[0], wt[1], wc_tag[1][0], wc_tag[1][1], p_tot);
        }
        if pres.len() > 20 {
            let (t0, pat) = (pres[20].0, pres[20].1.clone());
            let snap = snaps.iter().rev().find(|s| s.tick <= t0).unwrap();
            let amp = 52.0f32;
            let mut cur = [0.0f32; 3];
            let mut cur_tag = [[0.0f32; 2]; 3];
            for t in t0..t0 + 500 {
                if let Some(ids) = spk.get(&t) {
                    for &n in ids {
                        if n >= 24 { continue; }
                        for sid in net_outgoing_of(snap, n) {
                            let s = snap_syn(snap, sid);
                            if s.w.is_none() || !s.plastic || s.inhibitory() { continue; }
                            if s.post >= 76 { continue; }
                            let c = cohort(s.pre as u32);
                            let w = s.w.unwrap();
                            cur[c] += amp * w;
                            cur_tag[c][(s.track as usize).min(1)] += amp * w;
                        }
                    }
                }
            }
            let _ = amp;
            println!("F\t{t0}\t{pat}\tI_c = {:.3}\tI_c_tag0 = {:.3}\tI_c_tag1 = {:.3}\tI_a = {:.3}",
                cur[1], cur_tag[1][0], cur_tag[1][1], cur[0]);
        }
        println!();
    }
}

// minimal snapshot accessors (SynapseState has pre/post/w/plastic/consolidated/track)
fn net_outgoing_of(snap: &anima_telemetry::recorder::NetworkStateSnapshot, _pre: u32) -> Vec<u32> {
    // pre is an input NEURON id; build outgoing by scanning synapses (small)
    snap.synapses.iter().filter(|s| s.pre == _pre && !s.inhibitory()).map(|s| s.id).collect()
}
fn snap_syn(snap: &anima_telemetry::recorder::NetworkStateSnapshot, id: u32) -> &anima_telemetry::recorder::SynapseState {
    snap.synapses.iter().find(|s| s.id == id).unwrap()
}
trait Inhib { fn inhibitory(&self) -> bool; }
impl Inhib for anima_telemetry::recorder::SynapseState {
    fn inhibitory(&self) -> bool { false } // snapshots omit inhibitory; all afferents here are excitatory
}