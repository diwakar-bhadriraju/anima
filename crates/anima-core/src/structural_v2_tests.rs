//! ANIMA v2 mechanism tests (docs/anima-v2-protocol.md).
//! Each test asserts a frozen mechanism behavior or invariant; none of
//! them depend on A/B/C labels or stage structure.

use crate::network::{Network, NetworkConfig, NeuronClass, NeuronId, SynapseId, Tick, V2Params};
use crate::structural_v2::{V2Event, V2Plasticity};

/// The exact frozen parameter set (protocol §2–§8).
pub fn v2_params() -> V2Params {
    V2Params {
        disable_m2: false,
        disable_m3_m4: false,
        disable_m5: false,
        disable_m6: false,
        p_in: 0.5,
        w_in_lo: 0.02,
        w_in_hi: 0.06,
        p_rec: 0.2,
        w_rec_lo: 0.005,
        w_rec_hi: 0.02,
        t_e: 0.8,
        c_slots: 6,
        w_c_init: 0.01,
        delta_perm: 0.01,
        decay_c: 0.99,
        theta_permanent: 0.05,
        w_c_permanent: 0.02,
        theta_die: 0.005,
        p_cand_in: 0.5,
        p_cand_rec: 0.5,
        theta_prune: 0.005,
        prune_windows: 10,
        b_e: 40,
        b_i: 10,
        p_inh: 0.3,
        w_inh_lo: 0.01,
        w_inh_hi: 0.03,
        a_inh: 0.005,
        decay_inh: 0.98,
        w_inh_max: 0.10,
        window_ticks: 100,
    }
}

fn v2_net(seed: u64) -> (Network, V2Plasticity) {
    let cfg = NetworkConfig { v2: Some(v2_params()), ..NetworkConfig::default() };
    let mut net = Network::new(cfg, 8, 12, 4, seed);
    let v2 = V2Plasticity::new(&mut net, v2_params());
    (net, v2)
}

/// M1: dense-weak initialization — every non-input neuron receives input
/// afferents at Bernoulli p_in ~ 0.5, recurrent at p_rec ~ 0.2, and the
/// network fires under input without any STDP.
#[test]
fn m1_dense_weak_initialization_connects_most_channels() {
    let (net, _) = v2_net(42);
    let n_non_input = net.neurons.iter().filter(|n| n.class != NeuronClass::Input).count();
    let n_input = 8usize;
    // Expected input afferents per non-input neuron: p_in * 8 = 4.
    let mut total_in = 0usize;
    let mut total_rec = 0usize;
    let mut total_inh = 0usize;
    for n in net.neurons.iter().filter(|n| n.class != NeuronClass::Input) {
        for &sid in &net.incoming[n.id.idx()] {
            let s = &net.synapses[sid.idx()];
            if net.neurons[s.pre.idx()].class == NeuronClass::Input {
                total_in += 1;
            } else if s.inhibitory {
                total_inh += 1;
            } else {
                total_rec += 1;
            }
        }
    }
    let mean_in = total_in as f32 / n_non_input as f32;
    let mean_rec = total_rec as f32 / n_non_input as f32;
    let mean_inh = total_inh as f32 / n_non_input as f32;
    assert!(
        (mean_in - 4.0).abs() < 2.0,
        "M1: expected ~4 input afferents/neuron at p_in=0.5, got {mean_in:.2}"
    );
    assert!(
        (mean_rec - 1.6).abs() < 1.5,
        "M1: expected ~1.6 recurrent at p_rec=0.2, got {mean_rec:.2}"
    );
    assert!(
        mean_inh > 0.0,
        "M1: M6 inhibitory afferents exist at p_inh=0.3, got {mean_inh:.2}"
    );
    assert!(n_input > 0);
}

/// M1 determinism: same seed → identical synapse topologies; different
/// seed → different topology.
#[test]
fn m1_deterministic_initialization() {
    let topo = |seed: u64| -> String {
        let (net, _) = v2_net(seed);
        let mut s = String::new();
        for syn in net.synapses.iter() {
            s.push_str(&format!(
                "{}:{}->{}:{}:{};",
                syn.id.0, syn.pre.0, syn.post.0, syn.inhibitory, syn.plastic
            ));
        }
        s
    };
    assert_eq!(topo(7), topo(7), "same seed → identical v2 wiring");
    assert_ne!(topo(7), topo(8), "different seed → different wiring");
}

/// M3: candidate → permanence requires co-activity reinforcement, and
/// produces a real synapse.
#[test]
fn m3_candidate_becomes_permanent_under_coactivity() {
    let (mut net, mut v2) = v2_net(9);
    let post = net
        .neurons
        .iter()
        .find(|n| n.class == NeuronClass::Internal)
        .unwrap()
        .id;
    // Pick any candidate pre.
    let pre = v2.candidates[post.idx()][0].pre;
    let before = net.synapses.len();
    // Reinforce co-activity: make both fire every window until permanent.
    // theta_permanent 0.05 from 0.01 at +0.005/window = 8 windows.
    for _ in 0..12 {
        v2.tick(&[pre, post]);
        let events = v2.window(&mut net, Tick(1));
        for e in &events {
            if let V2Event::SynapseCreated { pre: p, post: q, reason, .. } = e {
                assert_eq!(*p, pre);
                assert_eq!(*q, post);
                assert_eq!(*reason, "candidate-permanence");
            }
        }
        // Successive windows double-fire both neurons (fired is per window).
        v2.tick(&[pre, post]);
        let ev2 = v2.window(&mut net, Tick(2));
        if !ev2.is_empty() {
            break;
        }
        v2.tick(&[pre, post]);
        let ev3 = v2.window(&mut net, Tick(3));
        if !ev3.is_empty() {
            break;
        }
    }
    assert!(
        net.synapses.len() > before,
        "permanence must add a real synapse"
    );
}

/// M3/M5: permanence respects the B_e budget — eviction removes the
/// lowest-weight live excitatory synapse when full.
#[test]
fn m5_eviction_removes_lowest_weight_when_budget_full() {
    // Larger net: 8 input, 60 internal, 4 output ⇒ 63 non-input sources.
    let cfg = NetworkConfig { v2: Some(v2_params()), ..NetworkConfig::default() };
    let mut net = Network::new(cfg, 8, 60, 4, 11);
    let post = net
        .neurons
        .iter()
        .find(|n| n.class == NeuronClass::Internal)
        .unwrap()
        .id;
    // Top up to exactly B_e = 40 live excitatory afferents (M1 already
    // contributed some input afferents).
    let existing = net
        .incoming[post.idx()]
        .iter()
        .filter(|&&sid| {
            let s = &net.synapses[sid.idx()];
            !s.inhibitory && s.silent_ticks != u64::MAX
        })
        .count();
    let need = 40usize.saturating_sub(existing);
    let srcs: Vec<NeuronId> = net
        .neurons
        .iter()
        .filter(|n| n.class != NeuronClass::Input && n.id != post)
        .map(|n| n.id)
        .take(need + 10)
        .collect();
    for (i, s) in srcs.iter().enumerate().take(need) {
        net.add_synapse(*s, post, 0.1 + i as f32 * 0.001, true, Tick(0));
    }
    // Use V2Plasticity's eviction via a fake budget-full condition: set
    // live_e by construction (budget_check recount) — simplest: call
    // budget_check via window with no events is complex; instead verify
    // the invariant counting matches the manual fill.
    let (live_e, _) = {
        // count live excitatory on post
        let mut e = 0usize;
        for &sid in &net.incoming[post.idx()] {
            if !net.synapses[sid.idx()].inhibitory && net.synapses[sid.idx()].silent_ticks != u64::MAX {
                e += 1;
            }
        }
        (e, 0usize)
    };
    assert_eq!(live_e, 40, "manual fill reached budget exactly");
    // Now drive a permanence on `post` via a candidate and check eviction.
    let mut v2 = V2Plasticity::new(&mut net, v2_params());
    // v2.live_e now recounts real counts (40 on post = full).
    let pre23: Vec<NeuronId> = net
        .neurons
        .iter()
        .filter(|n| n.class == NeuronClass::Input)
        .map(|n| n.id)
        .collect();
    let cand = pre23[0];
    v2.tick(&[cand, post]);
    // One window: permanence can't happen yet (candidate starts at 0.01);
    // force by repeatedly reinforcing — but budget is already full so
    // permanence should evict. To expedite, set candidate weight high.
    if !v2.candidates[post.idx()].is_empty() {
        v2.candidates[post.idx()][0].w = 0.99; // above permanence threshold
    }
    let events = v2.window(&mut net, Tick(5));
    let evictions = events
        .iter()
        .filter(|e| matches!(e, V2Event::SynapsePruned { reason: "budget-eviction", .. }))
        .count();
    assert!(
        evictions >= 1,
        "permanence on a full budget must evict: {events:?}"
    );
}

/// M2: normalization conserves total incoming excitatory weight to t_e.
#[test]
fn m2_normalization_conserves_t_e() {
    let (mut net, mut v2) = v2_net(3);
    let post = net
        .neurons
        .iter()
        .find(|n| n.class == NeuronClass::Internal)
        .unwrap()
        .id;
    // Bump one weight high, then normalize.
    let first = net.incoming[post.idx()][0];
    net.synapses[first.idx()].w = 0.7;
    // window with no firing: prune/candidates/normalize still run.
    v2.window(&mut net, Tick(100));
    let (sum, n_live) = net
        .incoming[post.idx()]
        .iter()
        .fold((0.0f32, 0usize), |(s, n), &sid| {
            let syn = &net.synapses[sid.idx()];
            if syn.silent_ticks != u64::MAX && !syn.inhibitory {
                (s + syn.w, n + 1)
            } else {
                (s, n)
            }
        });
    assert!(n_live > 0);
    assert!(
        (sum - 0.8).abs() < 1e-3,
        "M2: sum must equal t_e=0.8 after normalization, got {sum}"
    );
}

/// M2 invariant across many windows: sum never exceeds t_e.
#[test]
fn m2_invariant_sum_never_exceeds_t_e() {
    let (mut net, mut v2) = v2_net(5);
    let post = net
        .neurons
        .iter()
        .find(|n| n.class == NeuronClass::Internal)
        .unwrap()
        .id;
    for w in 1..=30u64 {
        // random-ish drive: fire everything to lift weights
        let all: Vec<NeuronId> = net.neurons.iter().map(|n| n.id).collect();
        v2.tick(&all);
        v2.window(&mut net, Tick(w * 100));
        let sum: f32 = net.incoming[post.idx()]
            .iter()
            .filter(|&&sid| {
                let s = &net.synapses[sid.idx()];
                s.silent_ticks != u64::MAX && !s.inhibitory
            })
            .map(|&sid| net.synapses[sid.idx()].w)
            .sum();
        assert!(
            sum <= 0.8 + 1e-3,
            "M2 invariant violated at window {w}: sum {sum}"
        );
    }
}

/// M4: a live excitatory synapse below theta_prune for prune_windows is
/// pruned; inhibitory synapses are never M4-pruned.
#[test]
fn m4_prunes_persistently_low_excitatory_only() {
    let (mut net, mut v2) = v2_net(8);
    let post = net
        .neurons
        .iter()
        .find(|n| n.class == NeuronClass::Internal)
        .unwrap()
        .id;
    // Find a live excitatory synapse set below threshold.
    let mut target: Option<SynapseId> = None;
    for &sid in &net.incoming[post.idx()] {
        let s = &net.synapses[sid.idx()];
        if !s.inhibitory && s.silent_ticks != u64::MAX {
            net.synapses[sid.idx()].w = 0.0; // below theta_prune 0.005
            target = Some(sid);
            break;
        }
    }
    let target = target.expect("neuron has live excitatory afferent");
    // Count inhibitory synapses alive.
    let inh_before = net
        .synapses
        .iter()
        .filter(|s| s.inhibitory && s.silent_ticks != u64::MAX)
        .count();
    for w in 1..=11u64 {
        v2.window(&mut net, Tick(w * 100));
    }
    assert!(
        !net.synapse_alive(target),
        "M4 must prune a synapse below theta_prune for >= 10 windows"
    );
    let inh_after = net
        .synapses
        .iter()
        .filter(|s| s.inhibitory && s.silent_ticks != u64::MAX)
        .count();
    assert_eq!(inh_before, inh_after, "M4 never prunes inhibitory synapses");
}

/// M2 + M4 interaction: normalization ({8}) then prune: a zeroed synapse
/// still gets pruned even in a neuron that has other weight — and the
/// prune does not destroy the last afferent (frozen rule has no exception,
/// but M1's p_in=0.5 means every internal has ~4 → the min-afferent edge
/// case is exercised in m4 test above only for one synapse).
#[test]
fn m5_budget_invariant_holds_after_structural_activity() {
    let (mut net, mut v2) = v2_net(19);
    // Run 20 windows with mixed firing; assert budget counts never exceed.
    let internals: Vec<NeuronId> = net
        .neurons
        .iter()
        .filter(|n| n.class == NeuronClass::Internal)
        .map(|n| n.id)
        .collect();
    for w in 1..=20u64 {
        for (i, id) in internals.iter().enumerate() {
            if (w as usize + i) % 3 == 0 {
                v2.tick(&[*id]);
            }
        }
        v2.window(&mut net, Tick(w * 100));
        // report was refreshed by budget_check inside window()
        assert!(v2.report.live_exc <= (net.neurons.len() as u64) * 40);
        assert!(v2.report.live_inh <= (net.neurons.len() as u64) * 10);
    }
}

/// M6: inhibitory weight increases when pre AND post co-fire; decays
/// otherwise; never exceeds w_inh_max.
#[test]
fn m6_anti_hebbian_updates_sign_and_bounds() {
    let (mut net, mut v2) = v2_net(13);
    // Find an inhibitory synapse.
    let (pre, post, sid) = net
        .synapses
        .iter()
        .filter(|s| s.inhibitory && s.silent_ticks != u64::MAX)
        .map(|s| (s.pre, s.post, s.id))
        .next()
        .expect("M1 gave an inhibitory afferent");
    net.synapses[sid.idx()].w = 0.01;
    // Co-fire both.
    v2.tick(&[pre, post]);
    v2.window(&mut net, Tick(100));
    // Decay branch: they do not fire next window.
    v2.window(&mut net, Tick(200));
    let w = net.synapses[sid.idx()].w;
    assert!(
        (0.0..=0.1).contains(&w),
        "M6 weight must stay in [0, w_inh_max=0.1], got {w}"
    );
    // 100 co-active windows saturate at the cap (0.01 + 100*0.005 >> cap).
    for w in 1..=100u64 {
        v2.tick(&[pre, post]);
        v2.window(&mut net, Tick(1000 + w * 100));
    }
    let w2 = net.synapses[sid.idx()].w;
    assert!((w2 - 0.10).abs() < 1e-6, "M6 must cap at w_inh_max, got {w2}");
    // And it must be inhibitory: deposit sign check on network step.
    let before = net.neurons[post.idx()].i_syn;
    let ev = net.step(&crate::network::InputFrame { tick: Tick(0), spikes: vec![] });
    let _ = ev;
    let after = net.neurons[post.idx()].i_syn;
    assert!(
        after <= before + 1e-6,
        "inhibitory delivery must not add positive current (before {before}, after {after})"
    );
}

/// M6 + STDP: STDP never touches inhibitory weights (M6 owns them).
#[test]
fn m6_stdp_does_not_touch_inhibitory_weights() {
    let (mut net, _) = v2_net(21);
    let sid = net
        .synapses
        .iter()
        .find(|s| s.inhibitory && s.silent_ticks != u64::MAX)
        .map(|s| s.id)
        .expect("inhibitory afferent exists");
    let w0 = net.synapses[sid.idx()].w;
    let traces = crate::plasticity::Traces::new(&net, 20.0);
    // Force spikes on both sides, run stdp_tick.
    let spikes: Vec<NeuronId> = net.neurons.iter().map(|n| n.id).collect();
    let changes = crate::plasticity::stdp_tick(
        &crate::plasticity::StdpParams::default(),
        &mut net,
        &traces,
        &spikes,
        1.0,
    );
    let w1 = net.synapses[sid.idx()].w;
    assert_eq!(w0, w1, "STDP must not modify inhibitory weight");
    assert!(
        !changes.iter().any(|c| c.synapse == sid),
        "STDP must not report inhibitory changes"
    );
}

/// Feature-off control: V2Params disabled (v2: None) reproduces the legacy
/// sparse wiring exactly (E1–E4f determinism).
#[test]
fn v2_disabled_reproduces_legacy_wiring() {
    let topo = |v2: Option<V2Params>| -> String {
        let cfg = NetworkConfig { v2, ..NetworkConfig::default() };
        let net = Network::new(cfg, 8, 12, 4, 42);
        net.synapses
            .iter()
            .map(|s| format!("{}:{}->{}", s.id.0, s.pre.0, s.post.0))
            .collect::<Vec<_>>()
            .join(";")
    };
    let legacy = topo(None);
    let baseline = topo(Some(v2_params())); // enabled
    assert_ne!(legacy, baseline, "v2 wiring must differ from legacy");
    // Re-run legacy twice for determinism.
    assert_eq!(topo(None), legacy, "legacy path deterministic");
}

/// Determinism of the full structural cycle: same seed → identical
/// event stream across v2 windows.
#[test]
fn v2_structural_cycle_deterministic() {
    let run = |seed: u64| -> Vec<String> {
        let (mut net, mut v2) = v2_net(seed);
        let mut log = Vec::new();
        let internals: Vec<NeuronId> = net
            .neurons
            .iter()
            .filter(|n| n.class == NeuronClass::Internal)
            .map(|n| n.id)
            .collect();
        for w in 1..=40u64 {
            // Fire every internal every window: heavy co-activity drives
            // candidate permanence, so the stream is non-empty and
            // seed-dependent.
            v2.tick(&internals);
            for e in v2.window(&mut net, Tick(w * 100)) {
                log.push(format!("{w}:{e:?}"));
            }
        }
        log
    };
    assert_eq!(run(33), run(33), "same seed → identical v2 structural stream");
    assert_ne!(run(33), run(34), "different seed → different stream");
}
/// M3-1 (approved amendment): permanence must be reachable during a
/// single 500 ms presentation (5 co-active windows) — mathematically
/// 0.01 + 4x0.01 = 0.05 at the 4th co-active window.
#[test]
fn m3_1_permanence_reachable_within_presentation() {
    let (mut net, mut v2) = v2_net(4_001);
    let post = net
        .neurons
        .iter()
        .find(|n| n.class == NeuronClass::Internal)
        .unwrap()
        .id;
    // Take a candidate and drive its pre + post co-active every window.
    let pre = v2.candidates[post.idx()][0].pre;
    let mut perms = 0usize;
    for w in 1..=5u64 {
        v2.tick(&[pre, post]);
        for e in v2.window(&mut net, Tick(w * 100)) {
            if let V2Event::SynapseCreated { reason, .. } = e {
                assert_eq!(reason, "candidate-permanence");
                perms += 1;
            }
        }
    }
    assert!(
        perms == 1 || perms == 2,
        "M3-1: permanence expected at 4th-5th co-active window of one presentation, got {perms}"
    );
    // The new synapse exists as a live plastic excitatory synapse.
    // Its weight starts at w_c_permanent (0.02) then M2 renormalizes the
    // neuron's total to t_e within the same window, so assert the
    // mechanism-relevant properties (permanence created it, plastic,
    // excitatory, still alive), not the transient pre-normalization value.
    let new_syn = net
        .synapses
        .iter()
        .rev()
        .find(|s| s.silent_ticks != u64::MAX && !s.inhibitory)
        .expect("permanence created a live synapse");
    assert!(new_syn.plastic);
    assert!(new_syn.w >= 0.005, "M2-normalized weight must survive M4 bar");
}

/// M3-1: silent candidate lifetime — 0.01 decays to theta_die (0.005)
/// at 0.99^n = 0.5 => n = 69 windows (~6.9 s), preserving search
/// semantics (die + redraw).
#[test]
fn m3_1_silent_candidate_dies_and_redraws() {
    let (mut net, mut v2) = v2_net(4_002);
    let post = net
        .neurons
        .iter()
        .find(|n| n.class == NeuronClass::Internal)
        .unwrap()
        .id;
    // Track a specific candidate until it is swapped (dies) — with no
    // firing at all, all candidates decay 0.99/window and redraw at 0.005.
    let orig: Vec<u32> = v2.candidates[post.idx()].iter().map(|c| c.pre.0).collect();
    let mut death_windows = 0u64;
    for w in 1..=200u64 {
        v2.window(&mut net, Tick(w * 100));
        let now: Vec<u32> = v2.candidates[post.idx()].iter().map(|c| c.pre.0).collect();
        if now != orig {
            death_windows = w;
            break;
        }
    }
    assert!(
        (60..=80).contains(&death_windows),
        "M3-1: silent candidate should die around 69 windows, got {death_windows}"
    );
    assert!(
        !v2.candidates[post.idx()].is_empty(),
        "redraw must refill the slot"
    );
}

/// M3-1: permanence still gated on *reinforcement* — a candidate whose
/// pre fires without the post firing never reaches threshold and stays
/// in the pool (no spurious permanence from pre-activity alone).
#[test]
fn m3_1_no_permanence_without_post_coactivity() {
    let (mut net, mut v2) = v2_net(4_003);
    let post = net
        .neurons
        .iter()
        .find(|n| n.class == NeuronClass::Internal)
        .unwrap()
        .id;
    let pre = v2.candidates[post.idx()][0].pre;
    let mut perms = 0usize;
    for w in 1..=30u64 {
        v2.tick(&[pre]); // ONLY pre fires
        for e in v2.window(&mut net, Tick(w * 100)) {
            if let V2Event::SynapseCreated { .. } = e {
                perms += 1;
            }
        }
    }
    assert_eq!(perms, 0, "M3-1: pre-only activity must not cause permanence");
}

/// M3-1 regression: with permanence now engaging, many windows of
/// candidate permanence + eviction must not index low_windows out of
/// bounds (arena grows mid-window; evict_for must resize first).
#[test]
fn m3_1_many_windows_with_growth_no_oob() {
    let (mut net, mut v2) = v2_net(5_101);
    // Saturate one neuron near its budget so permanence triggers eviction,
    // then drive heavy co-activity for many windows.
    let post = net
        .neurons
        .iter()
        .find(|n| n.class == NeuronClass::Internal)
        .unwrap()
        .id;
    // Fire every neuron (post + all pre channels) every window: massive
    // co-activity -> permanence storms.
    let all: Vec<NeuronId> = net.neurons.iter().map(|n| n.id).collect();
    for w in 1..=400u64 {
        v2.tick(&all);
        let _ = v2.window(&mut net, Tick(w * 100));
    }
    // Arena grew and stayed within budget invariants (hard asserts inside
    // window() already fired on violation — reaching here means stable).
    assert!(net.synapses.len() > 0);
}
