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
    let mut cur = "A".to_string(); // world starts on a known pattern
    let mut beat = 0u64;
    let mut died: Option<u64> = None;
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
        let tr = io::symbol_trains(&cur, _org_seed);
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
            traces.step(&net, &ev.spikes);
            // STDP live (plasticity ON in-loop, D-24)
            let changes = stdp_tick(params, net, traces, &ev.spikes, 1.0, None);
            let _ = changes.len();
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
        if (rcog_full || ob_sustained) && died.is_none() { died = Some(beat); }
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
        beats: beat, died_at: died, mean_viability: mean_v,
        known_vs_novel_diff: kvs, a_actions: a_act, c_actions: c_act,
        withdraw_actions: wd_act, quiet_actions: qt_act,
        known_recognized_frac: known_ok as f32 / known_n.max(1) as f32,
        novel_recognized_frac: novel_ok as f32 / novel_n.max(1) as f32,
        known_beats: known_n, novel_beats: novel_n,
    }
}