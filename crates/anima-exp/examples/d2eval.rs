//! d2eval: Phase II-A read-only evaluation (docs/x-phase2-a-protocol.md §10).
//! Per run, per snapshot: per-cohort PROTECTED mass (A pre<8, C 8..16,
//! B 16..24 — measurement-only grouping; the mechanism never reads it),
//! total P, per-track P (t2 = track 1, when D-core state exists),
//! prototype-pair cosine at drive end (BLUR metric), per-track working
//! shares, and the block-boundary snapshots used by the S1 criterion
//! (first at t=44,000, second at t=84,000 — the e24 convention).
//!
//! Output TSV:
//!   H <run>
//!   S <tick> <pA> <pC> <pB> <pTot> <pT0> <pT1> <wTot>
//!   B <mid-pA> <mid-pC> <mid-first-cohort> <end-pA> <end-pC>
//!   C <blur-cos> <protos-nonzero-neurons>
use anima_telemetry::recorder::read_snapshots;
use anima_telemetry::TelemetryReader;
use anima_telemetry::events::Payload;

fn main() {
    for dir in std::env::args().skip(1) {
        let snaps = read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let reader = TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        // determine block order from stimulus events (measurement only)
        let mut first_pat = String::new();
        let mut second_pat = String::new();
        let mut seen: Vec<String> = Vec::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(env) = row.envelope("e") {
                    if let Payload::StimulusPresented { pattern_id, stage } = &env.payload {
                        if stage == "S1" && !seen.contains(pattern_id) {
                            seen.push(pattern_id.clone());
                        }
                    }
                }
            }
        }
        if seen.len() >= 2 {
            first_pat = seen[0].clone();
            second_pat = seen[1].clone();
        }
        let cohort = |pre: u32| -> usize {
            if pre < 8 { 0 } else if pre < 16 { 1 } else { 2 }
        };
        let name = dir.rsplit('/').next().unwrap().to_string();
        println!("H\t{name}\tfirst={first_pat}\tsecond={second_pat}");
        for snap in &snaps {
            let mut p = [0.0f32; 3];
            let mut p_t0 = 0.0f32;
            let mut p_t1 = 0.0f32;
            let mut w_tot = 0.0f32;
            for s in &snap.synapses {
                if s.post >= 76 || s.w.is_none() { continue; }
                let w = s.w.unwrap();
                if s.plastic {
                    if s.consolidated {
                        if s.pre < 24 {
                            p[cohort(s.pre)] += w;
                        }
                        p_t0 += w * (1.0 - s.track as f32);
                        p_t1 += w * s.track as f32;
                    } else if s.pre < 24 {
                        w_tot += w;
                    }
                }
            }
            println!("S\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}",
                snap.tick, p[0], p[1], p[2], p[0] + p[1] + p[2], p_t0, p_t1, w_tot);
        }
        // block boundary numbers
        let mid = snaps.iter().find(|s| s.tick == 44_000).or_else(|| snaps.get(snaps.len() / 2));
        let end = snaps.last();
        let (mut mA_m, mut mC_m, mut mA_e, mut mC_e) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        let (mut first_m, mut first_e) = (0.0f32, 0.0f32);
        if let Some(s) = mid {
            for syn in &s.synapses {
                if syn.consolidated && syn.pre < 24 {
                    let w = syn.w.unwrap_or(0.0);
                    let cx = cohort(syn.pre);
                    if cx == 0 { mA_m += w; } else if cx == 1 { mC_m += w; }
                    if !first_pat.is_empty() && ((first_pat == "A" && cx == 0) || (first_pat == "C" && cx == 1)) {
                        first_m += w;
                    }
                }
            }
        }
        if let Some(s) = end {
            for syn in &s.synapses {
                if syn.consolidated && syn.pre < 24 {
                    let w = syn.w.unwrap_or(0.0);
                    let cx = cohort(syn.pre);
                    if cx == 0 { mA_e += w; } else if cx == 1 { mC_e += w; }
                    if !second_pat.is_empty() && ((second_pat == "A" && cx == 0) || (second_pat == "C" && cx == 1)) {
                        first_e += w;
                    }
                    if !first_pat.is_empty() && ((first_pat == "A" && cx == 0) || (first_pat == "C" && cx == 1)) {
                        first_e += 0.0; // second-block cohort only below
                    }
                }
            }
            // second-block cohort mass at drive end
            let mut second_e = 0.0f32;
            for syn in &s.synapses {
                if syn.consolidated && syn.pre < 24 {
                    let w = syn.w.unwrap_or(0.0);
                    let cx = cohort(syn.pre);
                    if !second_pat.is_empty() && ((second_pat == "A" && cx == 0) || (second_pat == "C" && cx == 1)) {
                        second_e += w;
                    }
                }
            }
            println!("B\t{mA_m:.3}\t{mC_m:.3}\t{first_m:.3}\t{mA_e:.3}\t{mC_e:.3}\t{second_e:.3}");
        }
        // raw drive-end masses (F1-style: ALL live exc afferents by channel
        // group, consolidated or not) + per-track protected max
        let last2 = snaps.last().unwrap();
        let mut ra = [0.0f32; 3];
        let mut pmax = [0.0f32; 3]; // per neuron per track max
        let mut ptrack = vec![[0.0f32; 3]; last2.neurons.len()];
        for syn in &last2.synapses {
            if syn.post >= 76 || syn.w.is_none() { continue; }
            if !syn.plastic { continue; }
            let w = syn.w.unwrap();
            if syn.pre < 24 {
                ra[cohort(syn.pre)] += w;
            }
            if syn.consolidated && syn.post < 76 {
                let t = (syn.track as usize).min(2);
                ptrack[syn.post as usize][t] += w;
            }
        }
        for i in 0..3 {
            pmax[i] = ptrack.iter().map(|p| p[i]).fold(0.0f32, f32::max);
        }
        let p_tot_max = ptrack.iter().map(|p| p[0] + p[1]).fold(0.0f32, f32::max);
        println!("R\t{:.3}\t{:.3}\t{:.3}", ra[0], ra[1], ra[2]);
        println!("T\t{:.3}\t{:.3}\t{:.3}", pmax[0], pmax[1], p_tot_max);
        // blur: mean pairwise cosine of the two prototypes over neurons
        // that have both (45-degree pair structure from the last snapshot)
        let last = snaps.last().unwrap();
        let mut cos_sum = 0.0f32;
        let mut cos_n = 0usize;
        for n in last.neurons.iter().filter(|n| !n.ctx_protos.as_ref().is_none_or(|v| v.is_empty())) {
            if let Some(v) = &n.ctx_protos {
                if v.len() >= 48 {
                    let (a, b) = (&v[0..24], &v[24..48]);
                    let mut num = 0.0; let mut na = 0.0; let mut nb = 0.0;
                    for (x, y) in a.iter().zip(b.iter()) {
                        num += x * y; na += x * x; nb += y * y;
                    }
                    if na > 0.0 && nb > 0.0 {
                        cos_sum += num / (na.sqrt() * nb.sqrt());
                        cos_n += 1;
                    }
                }
            }
        }
        let blur = if cos_n > 0 { cos_sum / cos_n as f32 } else { -1.0 };
        println!("C\t{blur:.3}\t{cos_n}");
        println!();
    }
}