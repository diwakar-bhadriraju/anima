//! E16 REV1-window process-activity probe (audit stage; read-only).
//! No thresholds, no causal claims. Reports raw footprints per
//! mechanism in (REV1-pre, REV1-post] and at the registered instants.
//!
//! 1. STDP: kind 8/9 (Strengthened/Weakened) events in the interval,
//!    counts by sign, ids intersecting E15 created ids.
//! 2. M3: candidate-permanence creations (kind 6, reason) in interval.
//! 3. M4: competitive-prune events (kind 7) in interval.
//! 4. M5: budget-eviction events in interval + occupancy vs B_e/B_i.
//! 5. M2: per-neuron excitatory incoming sum distribution at T0/pre/
//!    post (post-normalize invariant raw stats).
//! 6. M6: inhibitory (plastic=false) endpoint dW stats pre->post.
//! 7. Network dynamics: Spike counts during the REV1 presentation
//!    window (internal), per window-quartile (raw).
//!
//! Usage: e16_activity <run-dir> <seed>

const T0: u64 = 364_000;

fn main() {
    let dir = std::env::args().nth(1).expect("run dir");
    let seed = std::env::args().nth(2).expect("seed");
    let (pre_tick, post_tick, sb) = if seed == "20260912" {
        (T0, 366_000u64, 365_000u64)
    } else {
        (368_000, 370_000, 369_000)
    };
    let tdir = std::path::Path::new(&dir).join("telemetry");
    let reader = anima_telemetry::TelemetryReader::open(&tdir).unwrap();
    let idx = reader.chunk_index();

    let mut strengthened: Vec<(u64, u32, f32)> = Vec::new();
    let mut weakened: Vec<(u64, u32, f32)> = Vec::new();
    let mut created: Vec<(u64, u32, u32, u32, String)> = Vec::new();
    let mut pruned: Vec<(u64, u32, String)> = Vec::new();
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                3 => {
                    if row.n.map(|n| n >= 24).unwrap_or(false) {
                        spikes.push((row.t, row.n.unwrap() as u32));
                    }
                }
                6 => {
                    if let Ok(e) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::SynapseCreated { syn, pre, post, reason, .. } = e.payload {
                            created.push((e.t, syn.0, pre.0, post.0, reason.trigger));
                        }
                    }
                }
                7 => {
                    if let Ok(e) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::SynapsePruned { syn, reason } = e.payload {
                            pruned.push((e.t, syn.0, reason.trigger));
                        }
                    }
                }
                8 => {
                    if let Ok(e) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::SynapseStrengthened { syn, delta } = e.payload {
                            strengthened.push((e.t, syn.0, delta.unwrap_or(0.0)));
                        }
                    }
                }
                9 => {
                    if let Ok(e) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::SynapseWeakened { syn, delta } = e.payload {
                            weakened.push((e.t, syn.0, delta.unwrap_or(0.0)));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let snaps = anima_telemetry::recorder::read_snapshots(
        std::path::Path::new(&dir).join("snapshots.bin.zst").as_path(),
    )
    .expect("snapshots");
    let syn_at = |tick: u64| {
        snaps
            .iter()
            .find(|s| s.tick == tick)
            .unwrap()
            .synapses
            .iter()
            .map(|s| (s.id, s.pre, s.post, s.w.unwrap_or(0.0), s.plastic))
            .collect::<Vec<_>>()
    };

    println!("== seed {seed}  interval ({pre_tick}, {post_tick}]  REV1-B window=[{sb}, {})", sb + 500);
    let in_int = |t: u64| t > pre_tick && t <= post_tick;

    // 1. STDP footprints
    let st: Vec<_> = strengthened.iter().filter(|(t, _, _)| in_int(*t)).collect();
    let wk: Vec<_> = weakened.iter().filter(|(t, _, _)| in_int(*t)).collect();
    println!("STDP (kind 8/9, |dW|>0.01 per tick, emitted): strengthened={} weakened={}", st.len(), wk.len());
    let sum = |v: &[&(u64, u32, f32)]| v.iter().map(|e| e.2 as f64).sum::<f64>();
    let pre_ids: std::collections::BTreeSet<u32> = syn_at(pre_tick).iter().map(|s| s.0).collect();
    let post_ids: std::collections::BTreeSet<u32> = syn_at(post_tick).iter().map(|s| s.0).collect();
    let born_int: Vec<u32> = created.iter().filter(|(t, _, _, _, _)| in_int(*t)).map(|(_, id, _, _, _)| *id).collect();
    let st_ids: std::collections::BTreeSet<u32> = st.iter().map(|e| e.1).collect();
    let wk_ids: std::collections::BTreeSet<u32> = wk.iter().map(|e| e.1).collect();
    println!("  summed deltas: strengthened {:.6}, weakened {:.6}", sum(&st), sum(&wk));
    println!("  distinct synapses touched: strengthened-ids={} weakened-ids={} of alive-pre={}",
        st_ids.len(), wk_ids.len(), pre_ids.len());
    println!("  synergies: strengthened∩born={} weakened∩born={} strengthened∩pruned={} weakened∩pruned={}",
        born_int.iter().filter(|i| st_ids.contains(i)).count(),
        born_int.iter().filter(|i| wk_ids.contains(i)).count(),
        pruned.iter().filter(|(t, id, _)| in_int(*t) && st_ids.contains(id)).count(),
        pruned.iter().filter(|(t, id, _)| in_int(*t) && wk_ids.contains(id)).count());
    println!("  strengthened∩weakened (same id in interval): {}", st_ids.intersection(&wk_ids).count());

    // 2. M3 / 3. M4 / 4. M5
    let cr: Vec<_> = created.iter().filter(|(t, _, _, _, _)| in_int(*t)).collect();
    let pr: Vec<_> = pruned.iter().filter(|(t, _, _)| in_int(*t)).collect();
    println!("M3 maturations (candidate-permanence) in interval: {}", cr.iter().filter(|e| e.4 == "candidate-permanence").count());
    println!("M4 competitive-prune in interval: {}", pr.iter().filter(|e| e.2 == "competitive-prune").count());
    println!("M5 budget-eviction in interval: {}", pr.iter().filter(|e| e.2 == "budget-eviction").count());
    println!("  other prune reasons: {:?}", pr.iter().filter(|e| e.2 != "competitive-prune" && e.2 != "budget-eviction").map(|e| e.2.clone()).collect::<Vec<_>>());

    // 5. M2 invariant distribution at the three instants
    for tick in [T0, pre_tick, post_tick] {
        let mut sums: Vec<f64> = Vec::new();
        let mut by_post: std::collections::BTreeMap<u32, f64> = std::collections::BTreeMap::new();
        let mut inh: std::collections::BTreeMap<u32, f64> = std::collections::BTreeMap::new();
        for (_, _, post, w, plastic) in syn_at(tick) {
            if post < 24 {
                continue;
            }
            if plastic {
                *by_post.entry(post).or_insert(0.0) += w as f64;
            } else {
                *inh.entry(post).or_insert(0.0) += w as f64;
            }
        }
        for (_, s) in &by_post {
            sums.push(*s);
        }
        // only internal posts with at least one exc afferent
        if !sums.is_empty() {
            let mean_dev = sums.iter().map(|s| (s - 0.8).abs()).sum::<f64>() / sums.len() as f64;
            let max_dev = sums.iter().map(|s| (s - 0.8).abs()).fold(0.0f64, f64::max);
            println!("M2 invariant at t={tick}: neurons_with_exc={} mean|sum-0.8|={:.6} max|sum-0.8|={:.6}",
                sums.len(), mean_dev, max_dev);
        } else {
            println!("M2 invariant at t={tick}: no exc posts");
        }
    }

    // 6. M6 endpoint dW (plastic=false, alive both)
    let pre_map: std::collections::BTreeMap<u32, (u32, f32)> =
        syn_at(pre_tick).iter().map(|s| (s.0, (s.2, s.3))).collect();
    let post_map: std::collections::BTreeMap<u32, (u32, f32)> =
        syn_at(post_tick).iter().map(|s| (s.0, (s.2, s.3))).collect();
    let mut n_changed = 0usize;
    let mut n_total = 0usize;
    let mut dsum = 0.0f64;
    let pre_full = syn_at(pre_tick);
    let mut n_changed = 0usize;
    let mut n_total = 0usize;
    let mut dsum = 0.0f64;
    for (id, (_, w)) in &pre_map {
        let s_pre = pre_full.iter().find(|s| s.0 == *id).unwrap();
        if s_pre.4 {
            continue; // excitatory — counted elsewhere
        }
        n_total += 1;
        if let Some((_, w2)) = post_map.get(id) {
            let d = (*w2 - *w) as f64;
            if d != 0.0 {
                n_changed += 1;
                dsum += d;
            }
        }
    }
    println!("M6 endpoint (inhibitory plastic=false alive-both): changed={n_changed}/{n_total}, net dW sum={dsum:.6}");

    // 7. Network dynamics during the presentation window (internal spikes)
    let win_spikes: Vec<&(u64, u32)> = spikes.iter().filter(|(t, _)| *t >= sb && *t < sb + 500).collect();
    let quart = |lo: u64, hi: u64| win_spikes.iter().filter(|(t, _)| *t >= lo && *t < hi).count();
    println!("network dynamics: internal spikes in REV1 window={} (q1 {} q2 {} q3 {} q4 {})",
        win_spikes.len(),
        quart(sb, sb + 125),
        quart(sb + 125, sb + 250),
        quart(sb + 250, sb + 375),
        quart(sb + 375, sb + 500));
}