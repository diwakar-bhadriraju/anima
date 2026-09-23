//! Phase III sY survival loop (docs/phase3/s-survival-protocol.md
//! D-22..D-24). CLOSED-LOOP stage: after the static schedule (S1
//! formation), the next stimulus is chosen by the organism's OWN output
//! vote through the frozen io codebook (D-23/D-25), a homeostatic drive
//! scores viability v(t) = a(t)*r(t)*s(t), and death = viability
//! collapse. Runs the LIVE harness network (V2Plasticity + STDP).
use anima_core::network::{InputFrame, Network, Tick};
use anima_core::plasticity::{stdp_tick, StdpParams, Traces};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::io;
use anima_core::resources::{Failure as ResFailure, ResourceConfig, ResourceMonitor};
use anima_core::structural_v2::V2Plasticity;
use anima_core::structural::{make_trigger, BirthTrigger, Signals, StructuralMonitor};

const BEAT_MS: u64 = 500;

pub struct SurvivalOutcome {
    pub beats: u64,
    pub died_at: Option<u64>,
    pub mean_viability: f32,
    pub known_vs_novel_diff: Option<f32>,
    pub a_actions: u64,
    pub c_actions: u64,
    pub withdraw_actions: u64,
    pub quiet_actions: u64,
    pub known_recognized_frac: f32,
    pub novel_recognized_frac: f32,
    pub known_beats: u32,
    pub novel_beats: u32,
    /// In-loop resource failure (runaway-activity / resource-exhaustion),
    /// if the monitor tripped before death-by-viability. Advisory D-38:
    /// evolve's life must detect runaway the same way the harness does,
    /// else registration is blind to the very failure it registers.
    pub failed: Option<String>,
}

/// Run the closed loop after S1: `beats` beats; each presents the current
/// world stimulus (deterministic trains via io), runs the live network
/// with STDP ON (D-24 plasticity), decodes the 12-dim output to an action
/// via the frozen codebook, world-updates, accumulates viability, and
/// detects death (recognition collapse or activity leave [a_lo,a_hi]).
pub fn run(
    net: &mut Network,
    seed: u64,
    refs: &[(String, Vec<f32>)],
    spec: &crate::config::SurvivalSpec,
    params: &StdpParams,
    traces: &mut Traces,
) -> SurvivalOutcome {
    run_world(net, seed, seed, refs, spec, params, traces)
}

/// `world_seed` controls the forced-novelty schedule INDEPENDENTLY of the
/// organism seed: pass the same world_seed for every organism in a
/// generation and they all face the IDENTICAL exam (same D-beat schedule),
/// so fitness differences reflect the brain, not different world
/// sequences (D-32 fairness requirement).
#[allow(clippy::too_many_arguments)]
pub fn run_world(
    net: &mut Network,
    _org_seed: u64,
    world_seed: u64,
    refs: &[(String, Vec<f32>)],
    spec: &crate::config::SurvivalSpec,
    params: &StdpParams,
    traces: &mut Traces,
) -> SurvivalOutcome {
    run_world_v2(net, _org_seed, world_seed, refs, spec, params, traces, None, 100)
}

/// Variant with the V2Plasticity mechanism (M3 candidate -> self-constructed
/// permanence) ACTIVE, so the organism builds its own new connections from
/// experience during its life (D-36). v2 None = identity (no self-construction).
#[allow(clippy::too_many_arguments)]
pub fn run_world_v2(
    net: &mut Network,
    _org_seed: u64,
    world_seed: u64,
    refs: &[(String, Vec<f32>)],
    spec: &crate::config::SurvivalSpec,
    params: &StdpParams,
    traces: &mut Traces,
    mut v2: Option<&mut V2Plasticity>,
    window_ticks: u64,
) -> SurvivalOutcome {
    run_world_full(net, _org_seed, world_seed, refs, spec, params, traces,
        v2, window_ticks, None, None, false, None)
}

#[allow(clippy::too_many_arguments)]
pub fn run_world_full(
    net: &mut Network,
    _org_seed: u64,
    world_seed: u64,
    refs: &[(String, Vec<f32>)],
    spec: &crate::config::SurvivalSpec,
    params: &StdpParams,
    traces: &mut Traces,
    mut v2: Option<&mut V2Plasticity>,
    window_ticks: u64,
    mut structural: Option<(StructuralMonitor, Box<dyn BirthTrigger>)>,
    mut pe_state: Option<(f32, f32)>, // (pe_mean, pe_std) for persistent error
    _birth_probe: bool,
    // in-loop resource monitor (runaway + caps); None = no detection
    mut mon: Option<ResourceMonitor>,
) -> SurvivalOutcome {
    let mut cur = "A".to_string(); // world starts on a known pattern
    let mut beat = 0u64;
    let mut died: Option<u64> = None;
    let mut fail_kind: Option<String> = None;
    let mut v_sum = 0.0f32;
    let (mut a_act, mut c_act, mut wd_act, mut qt_act) = (0u64, 0u64, 0u64, 0u64);
    let mut known_v: Vec<f32> = Vec::new();
    let mut novel_v: Vec<f32> = Vec::new();
    let (mut known_ok, mut known_n, mut novel_ok, mut novel_n) = (0u32, 0u32, 0u32, 0u32);
    let rw = spec.r_window.max(1) as usize;
    let mut rcog: std::collections::VecDeque<bool> = std::collections::VecDeque::with_capacity(rw);
    let mut ob_window: std::collections::VecDeque<bool> = std::collections::VecDeque::with_capacity(rw);
    let mut tick = net.tick;

    while beat < spec.beats && died.is_none() {
        let tr = io::symbol_trains(&cur, world_seed);
        let is_known = cur == "A" || cur == "C";
        let mut out = vec![0.0f32; 12];
        for t in 0..BEAT_MS {
            let frame = InputFrame {
                tick,
                spikes: tr.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect(),
            };
            let ev = net.step(&frame);
            for c in &ev.spikes {
                if (io::OUTPUT_LO..io::OUTPUT_HI).contains(&c.0) {
                    out[(c.0 - io::OUTPUT_LO) as usize] += 1.0;
                }
            }
            // in-loop resource detection (D-38): runaway / exhaustion =
            // death, same standard as the harness monitor.
            if let Some(m) = mon.as_mut() {
                let (_, fail) = m.step(net, ev.spikes.len());
                if let Some(f) = fail {
                    if died.is_none() {
                        died = Some(beat);
                        fail_kind = Some(format!("{}: {}", f.kind, f.detail));
                    }
                }
            }
            // V2 self-construction machinery (D-36): tick + structural window
            // (M3 candidate -> permanence) so the organism builds its OWN new
            // connections from experience. None = no self-construction.
            if let Some(v2ref) = v2.as_deref_mut() {
                v2ref.tick(&ev.spikes);
                v2ref.accumulate_input_current(net, &ev.spikes); // unblocks
                // candidate accumulation (alloc_residual gate) - the missing
                // per-tick bookkeeping that was zeroing delta_perm
                if net.tick.0 % window_ticks == 0 && net.tick.0 > 0 {
                    let _ = v2ref.window(net, tick);
                }
            }
            traces.step(&net, &ev.spikes);
            // STDP live (plasticity ON in-loop, D-24)
            let changes = stdp_tick(params, net, traces, &ev.spikes, 1.0, None);
            let _ = changes.len();
            // birth trigger (optional): organism self-builds NEW neurons via
            // the organism's own signals (prediction error = unrecognized
            // or novel beats). D-37 gate probe.
            if let Some((monitor, trigger)) = structural.as_mut() {
                let pe = if cur != "A" && cur != "C" { 1.0 } else { 0.2 }; // birth pressure from novel/unrecognized beats
                if let Some((m, s)) = pe_state.as_mut() {
                    *m += (pe - *m) * 0.05;
                    *s = (*s + (pe - *m).abs()) * 0.5;
                }
                let sig = Signals {
                    prediction_error: pe,
                    pe_mean: pe_state.map(|x| x.0).unwrap_or(0.0),
                    pe_std: pe_state.map(|x| x.1).unwrap_or(0.0),
                    novelty: if cur != "A" && cur != "C" { 1.0 } else { 0.0 },
                };
                let n_before = net.neurons.len();
                let ev = monitor.step(net, trigger.as_mut(), &sig);
                if !ev.births.is_empty() {
                    if let Some(v2ref) = v2.as_deref_mut() {
                        // keep V2Plasticity bookkeeping in sync with the born neuron
                        v2ref.on_neuron_appended(net);
                    }
                }
            }
            tick = Tick(tick.0 + 1);
        }
        // decode action via frozen codebook (plain-cos argmax + amp floor)
        let act = io::decode(&out, refs, spec.q_floor, spec.th_known);
        match act.as_str() {
            "A" => a_act += 1,
            "C" => c_act += 1,
            "NOVEL" | "UNSURE" => wd_act += 1,
            "QUIET" => qt_act += 1,
            _ => {}
        }
        // recognition: a known beat is recognized ONLY if it decodes to
        // its own ref (exact match). A mismatched known beat (cur=A decoded
        // C) is a recognition FAILURE, not a hit.
        let recognized = is_known && act == cur;
        rcog.push_back(recognized);
        if rcog.len() > rw { rcog.pop_front(); }
        if is_known { known_n += 1; if recognized { known_ok += 1; } } else { novel_n += 1; if recognized { novel_ok += 1; } }
        let r = rcog.iter().filter(|x| **x).count() as f32 / rcog.len().max(1) as f32;
        // activity boundedness (pool 24..64 mean rate, Hz)
        let rate = net.neurons.iter().skip(24).take(40).map(|n| n.rate_hz).sum::<f32>() / 40.0;
        let a = if rate >= spec.a_bounds[0] && rate <= spec.a_bounds[1] { 1.0 } else { 0.0 };
        if std::env::var("DBG_SURV").is_ok() { eprintln!("BEAT {beat} cur={cur} out={out:?} rate={rate:.1} a={a} recognized={recognized} r={r:.2} act={act}"); }
        let s = 1.0; // no in-loop failure recorded (runaway guarded at harness)
        let v = a * r * s;
        v_sum += v;
        if is_known { known_v.push(v); } else { novel_v.push(v); }
        // death: recognition collapse (full window unrecognized AND window
        // full) OR pool activity out of bounds
        // death requires SUSTAINED condition over a full window (D-28):
        // recognition collapse over full r_window, OR activity out of
        // bounds over full r_window (a single quiet/fast beat is not death)
        ob_window.push_back(a == 0.0);
        if ob_window.len() > rw { ob_window.pop_front(); }
        let ob_sustained = ob_window.len() == rw && ob_window.iter().all(|x| *x);
        let rcog_full = rcog.len() == rw && rcog.iter().all(|x| !*x);
        if (rcog_full || ob_sustained) && died.is_none() {
            died = Some(beat);
            if std::env::var("DBG_DEATH").is_ok() {
                eprintln!("DEATH beat {beat}: ob_sustained={ob_sustained} (rate={rate:.1}) rcog_full={rcog_full} (r={r:.2}) n_neurons={}", net.neurons.len());
            }
        }
        // world-update (D-23 approach-known, D-30 forced-novelty):
        // with prob p_novel present the NEVER-TRAINED D probe (guarantees
        // the known-vs-novel falsifier is measurable); else follow the
        // organism's own action (approach known keeps it, novel->gap).
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(world_seed ^ (beat as u64).wrapping_mul(0x9E3779B97F4A7C15));
        let force_novel = rng.gen::<f32>() < spec.p_novel;
        cur = if force_novel {
            "D".into() // forced-novelty probe (D-30): the ONLY source of D
        } else {
            match act.as_str() {
                "A" => "A".into(), // approach known keeps it
                "C" => "C".into(),
                _ => if cur == "A" { "C".into() } else { "A".into() },
                // withdraw/QUIET/novel -> GAP then move AWAY to the OTHER
                // known pattern (D-23), never re-present D from the
                // organism's own state (that deadlocks: novel->silence->D).
            }
        };
        // D-29: inter-beat REST gap (off_ms) - preserves the trained
        // stimulus+rest cadence; without it the pool self-saturates.
        for _ in 0..spec.off_ms {
            let frame = InputFrame { tick, spikes: vec![] };
            let ev = net.step(&frame);
            let _ = stdp_tick(params, net, traces, &ev.spikes, 1.0, None);
            tick = Tick(tick.0 + 1);
        }
        beat += 1;
    }
    let nb = beat.max(1);
    let mean_v = v_sum / nb as f32;
    let km = known_v.iter().sum::<f32>() / known_v.len().max(1) as f32;
    let nm = novel_v.iter().sum::<f32>() / novel_v.len().max(1) as f32;
    let kvs = if novel_n == 0 { None } else { Some(km - nm) };
    SurvivalOutcome {
        beats: beat, died_at: died, mean_viability: mean_v, failed: fail_kind,
        known_vs_novel_diff: kvs, a_actions: a_act, c_actions: c_act,
        withdraw_actions: wd_act, quiet_actions: qt_act,
        known_recognized_frac: known_ok as f32 / known_n.max(1) as f32,
        novel_recognized_frac: novel_ok as f32 / novel_n.max(1) as f32,
        known_beats: known_n, novel_beats: novel_n,
    }
}