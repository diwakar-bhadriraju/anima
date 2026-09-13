//! ANIMA v2 analysis instrument (measurement only, no mechanism access):
//! - established structural changes from chunk telemetry (candidate-
//!   permanence created at t, still alive at t + 10_000 ms)
//! - receptive-field statistics from the snapshot nearest t = 715_000 ms:
//!   per-internal-neuron input-channel weight vector entropy (H),
//!   quasi-private classification (H <= 1.5 AND top channels > 0.5*max in
//!   a single channel group; groups = ch / group_size), top-channel map
//! - S1 internal mean-rate range (snapshots in [5000, 725000])
//! Usage: v2_analysis <run-dir> <group_size>

use std::collections::BTreeMap;
use anima_telemetry::events::Payload;

fn main() {
    let run_dir = std::env::args().nth(1).expect("run dir");
    let group_size: usize = std::env::args().nth(2).expect("group_size").parse().unwrap();
    let tdir = std::path::Path::new(&run_dir).join("telemetry");

    // ---- established structural changes ----
    let reader = anima_telemetry::TelemetryReader::open(&tdir).unwrap();
    let idx = reader.chunk_index();
    let mut created: Vec<(u64, u64)> = Vec::new(); // (t, synapse_id)
    let mut pruned: Vec<(u64, u64)> = Vec::new();
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            if let Ok(e) = row.envelope("v2") {
                match &e.payload {
                    Payload::SynapseCreated { syn, reason, .. } => {
                        if reason.trigger == "candidate-permanence" {
                            created.push((e.t, syn.0 as u64));
                        }
                    }
                    Payload::SynapsePruned { syn, reason } => {
                        if reason.trigger == "competitive-prune"
                            || reason.trigger == "budget-eviction"
                        {
                            pruned.push((e.t, syn.0 as u64));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    let total_permanence = created.len();
    let established: Vec<(u64, u64)> = created
        .iter()
        .filter(|&&(t, s)| {
            !pruned.iter().any(|&(pt, ps)| ps == s && pt > t && pt <= t + 10_000)
        })
        .copied()
        .collect();
    println!("== structural changes ==");
    println!("candidate-permanence events: {total_permanence}");
    println!("established (alive at t+10s): {}", established.len());

    // ---- RF stats from snapshots ----
    let snaps = anima_telemetry::recorder::read_snapshots(
        std::path::Path::new(&run_dir).join("snapshots.bin.zst").as_path(),
    )
    .expect("snapshots");
    let target = 715_000u64;
    let snap = snaps
        .iter()
        .min_by_key(|s| s.tick.abs_diff(target))
        .expect("snapshots non-empty");
    println!("== RF snapshot ==");
    println!("snapshot tick = {} (target {})", snap.tick, target);

    // neuron class map: ids < 24 input? use class strings from snapshot.
    let n_in: usize = 24; // E1 fixed by config
    let internal_ids: Vec<u32> = snap
        .neurons
        .iter()
        .filter(|n| n.class == "internal")
        .map(|n| n.id)
        .collect();
    // channel group from channel id: group = ch / group_size
    // build weight vector per internal: input-channel afferents only
    // (skip inhibitory — snapshot SynapseState lacks the flag, so weight
    //  vectors count only synapses whose pre is an input neuron; v2 M6
    //  synapses never originate from input neurons (D8), so no filtering
    //  is needed beyond pre < n_in).
    let mut per_neuron: BTreeMap<u32, Vec<f64>> = BTreeMap::new();
    for id in &internal_ids {
        per_neuron.insert(*id, vec![0.0; n_in]);
    }
    for s in &snap.synapses {
        if s.pre < n_in as u32 {
            if let Some(v) = per_neuron.get_mut(&s.post) {
                v[s.pre as usize] = s.w.unwrap_or(0.0) as f64;
            }
        }
    }
    let mut h_vals = Vec::new();
    let mut quasi_private = 0usize;
    let mut per_group = vec![0usize; 4]; // A B C D groups
    let mut top_map: BTreeMap<u32, u32> = BTreeMap::new();
    for (id, v) in &per_neuron {
        let sum: f64 = v.iter().sum();
        let mut h = 0.0f64;
        if sum > 0.0 {
            for &w in v.iter() {
                let p = w / sum;
                if p > 0.0 {
                    h -= p * p.log2();
                }
            }
        }
        h_vals.push(h);
        let max = v.iter().cloned().fold(0.0, f64::max);
        let top: Vec<usize> = v
            .iter()
            .enumerate()
            .filter(|&(_, &w)| w > 0.5 * max)
            .map(|(i, _)| i)
            .collect();
        if let Some(&t) = top.first() {
            top_map.insert(*id, t as u32);
        }
        if h <= 1.5 && !top.is_empty() {
            // all top channels in one group?
            let groups: std::collections::BTreeSet<usize> =
                top.iter().map(|&ch| ch / group_size).collect();
            if groups.len() == 1 {
                quasi_private += 1;
                let g = *groups.iter().next().unwrap();
                if g < per_group.len() {
                    per_group[g] += 1;
                }
            }
        }
    }
    let n_neurons = internal_ids.len();
    let mean_h: f64 = h_vals.iter().sum::<f64>() / h_vals.len().max(1) as f64;
    println!("internal neurons: {n_neurons}");
    println!("mean H: {mean_h:.3}");
    println!("quasi-private: {quasi_private} / {n_neurons}");
    println!("quasi-private per group [A,B,C,D+]: {:?}", per_group);
    let frac = quasi_private as f64 / n_neurons.max(1) as f64;
    println!("quasi-private fraction: {frac:.3}");
    // top-channel map for P4 Jaccard
    println!("top-channel map: {:?}", top_map);
    println!("mean-H per arm output above");

    // ---- S1 mean-rate range ----
    let mut s1_means = Vec::new();
    for s in &snaps {
        if s.tick >= 5_000 && s.tick <= 725_000 {
            let rates: Vec<f32> = s
                .neurons
                .iter()
                .filter(|n| n.class == "internal")
                .filter_map(|n| n.rate_hz)
                .collect();
            if !rates.is_empty() {
                s1_means.push(rates.iter().sum::<f32>() / rates.len() as f32);
            }
        }
    }
    let min = s1_means.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = s1_means.iter().cloned().fold(0.0f32, f32::max);
    println!("== S1 internal mean-rate ==");
    println!("snapshots={} min={min:.1} max={max:.1} Hz", s1_means.len());
}
