//! rg8eval: per-run endpoint ingredients for the frozen k_g=8 recruitment
//! experiment (docs/x-clla-recruitment-design-review.md §10).
//!
//! Output rows (TSV):
//!   H <run-name>
//!   P <pat> <ord> <t0> <unique-n> <spikes> <burst-hz> <ipA> <iwA> <ipB> <iwB> <ipC> <iwC> <irec> <iinh>
//!       per presentation: unique internal neurons fired /52, total internal
//!       spikes, burst-hz = spikes/52/0.5s, and per-neuron mean current
//!       decomposition (protected/working per cohort A 0-8 / B 16-24 /
//!       C 8-16, recurrent, inhibitory) from the snapshot at/before t0.
//!   G <gap-start> <gap-end> <internal-spikes>
//!       inter-presentation gap activity (no-input windows).
//!   E <t> <cohort> <perm-count>          candidate-permanence events
//!   X <t> <cohort> <prune-count>         prune events (all reasons)
//!   S <last-t> <c-count> <c-sum> <c-mean> <b-count> <b-sum> <b-mean> <a-count> <a-sum> <a-mean> <maxP>
//!       last snapshot: consolidated-cohort synapse counts/sums/means and
//!       max per-neuron protected mass across ALL cohorts.
use std::collections::{BTreeMap, BTreeSet};
use anima_telemetry::events::Payload;

fn cohort(pre: u32) -> &'static str {
    if pre < 8 { "A" } else if pre < 16 { "C" } else { "B" }
}

fn main() {
    for dir in std::env::args().skip(1) {
        let snaps = anima_telemetry::recorder::read_snapshots(
            &std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let reader = anima_telemetry::TelemetryReader::open(
            &std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut pres: Vec<(String, u64)> = Vec::new();
        let mut spk: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
        let mut perms: Vec<(u64, u32)> = Vec::new();
        let mut prunes: Vec<(u64, u32)> = Vec::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(env) = row.envelope("e") {
                    match &env.payload {
                        Payload::StimulusPresented { pattern_id, stage } => {
                            if stage == "S1" { pres.push((pattern_id.clone(), row.t)); }
                        }
                        Payload::Spike { n } => {
                            spk.entry(row.t).or_default().push(n.0);
                        }
                        Payload::SynapseCreated { pre, reason, .. } => {
                            if reason.trigger == "candidate-permanence" {
                                perms.push((row.t, pre.0));
                            }
                        }
                        Payload::SynapsePruned { syn, .. } => {
                            // resolve pre via any snapshot (weights vec) — use
                            // snapshot at t=0 as pre may persist; fall back to
                            // scanning the latest snapshot before t.
                            let mut pre = u32::MAX;
                            let mut si = 0;
                            while si + 1 < snaps.len() && snaps[si + 1].tick <= row.t { si += 1; }
                            for s in &snaps[si].synapses {
                                if s.id == syn.0 { pre = s.pre; break; }
                            }
                            prunes.push((row.t, pre));
                        }
                        _ => {}
                    }
                }
            }
        }
        pres.sort_by_key(|p| p.1);
        let name = dir.rsplit('/').next().unwrap().to_string();
        println!("H\t{name}");
        let amp = 52.0f32;
        let mut si = 0usize;
        for (pi, (pat, t0)) in pres.iter().enumerate() {
            while si + 1 < snaps.len() && snaps[si + 1].tick <= *t0 { si += 1; }
            let snap = &snaps[si];
            let mut internal_fired: BTreeSet<u32> = BTreeSet::new();
            let mut ch_fired: BTreeSet<u32> = BTreeSet::new();
            let mut n_spikes = 0u32;
            for t in *t0..t0 + 500 {
                if let Some(ids) = spk.get(&t) {
                    for &n in ids {
                        if n < 24 { ch_fired.insert(n); }
                        else if n < 76 { internal_fired.insert(n); n_spikes += 1; }
                    }
                }
            }
            let mut ip = [0.0f32; 3]; let mut iw = [0.0f32; 3];
            let mut irec = 0.0f32; let mut iinh = 0.0f32;
            for syn in &snap.synapses {
                if syn.post < 24 || syn.post >= 76 { continue; }
                if syn.pre < 24 {
                    if !ch_fired.contains(&syn.pre) { continue; }
                    let w = syn.w.unwrap_or(0.0);
                    let ci = if syn.pre < 8 { 0 } else if syn.pre < 16 { 1 } else { 2 };
                    if syn.consolidated { ip[ci] += amp * w; } else { iw[ci] += amp * w; }
                } else if syn.pre < 76 && internal_fired.contains(&syn.pre) {
                    let w = syn.w.unwrap_or(0.0);
                    if syn.plastic { irec += amp * w; } else { iinh -= amp * w; }
                }
            }
            let uniq = internal_fired.len() as f32 / 52.0;
            let burst = n_spikes as f32 / 52.0 / 0.5;
            println!("P\t{pat}\t{}\t{t0}\t{uniq:.2}\t{n_spikes}\t{burst:.1}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}",
                pi + 1, ip[0], iw[0], ip[1], iw[1], ip[2], iw[2], irec, iinh);
        }
        // gaps
        for w in pres.windows(2) {
            let (_, a) = &w[0]; let (_, b) = &w[1];
            let start = a + 500;
            if start >= *b { continue; }
            let mut g = 0u32;
            for t in start..*b {
                if let Some(ids) = spk.get(&t) {
                    g += ids.iter().copied().filter(|n| (24..76).contains(n)).count() as u32;
                }
            }
            println!("G\t{start}\t{b}\t{g}");
        }
        for (t, pre) in &perms { println!("E\t{t}\t{}\t1", cohort(*pre)); }
        for (t, pre) in &prunes { println!("X\t{t}\t{}\t1", cohort(*pre)); }
        // last snapshot stats
        let last = snaps.last().unwrap();
        let mut c = [0u32; 3]; let mut sum = [0.0f32; 3];
        let mut psum = vec![0.0f32; 76];
        for s in &last.synapses {
            if s.pre >= 24 || s.post >= 76 { continue; }
            if !s.consolidated { continue; }
            let w = s.w.unwrap_or(0.0);
            let ci = if s.pre < 8 { 0 } else if s.pre < 16 { 1 } else { 2 };
            c[ci] += 1; sum[ci] += w;
            psum[s.post as usize] += w;
        }
        let maxp = psum.iter().cloned().fold(0.0f32, f32::max);
        println!("S\t{}\t{}\t{:.3}\t{:.3}\t{}\t{:.3}\t{:.3}\t{}\t{:.3}\t{:.3}\t{:.3}",
            last.tick, c[0], sum[0], if c[0] > 0 { sum[0] / c[0] as f32 } else { 0.0 },
            c[1], sum[1], if c[1] > 0 { sum[1] / c[1] as f32 } else { 0.0 },
            c[2], sum[2], if c[2] > 0 { sum[2] / c[2] as f32 } else { 0.0 }, maxp);
        println!();
    }
}