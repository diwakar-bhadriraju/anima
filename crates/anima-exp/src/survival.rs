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
use crate::reflex;
use anima_core::resources::{Failure as ResFailure, ResourceConfig, ResourceMonitor};
use anima_core::structural_v2::V2Plasticity;
use anima_core::structural::{make_trigger, BirthTrigger, Signals, StructuralMonitor};

const BEAT_MS: u64 = 500;

// ---- D-59 closed-loop motor world (docs/phase3/d59-reflex-integration-protocol.md).
// PRE-REGISTERED constants, v1 world; NOT tuned on outcomes (if
// consequence-novelty is uninformative, record and stop - do not re-fit).
/// Motor drive gain: the next beat's afferent rate is scaled by
/// 1.0 +/- drive * MOTOR_GAIN (drive in [0,1]).
const MOTOR_GAIN: f32 = 0.2;
/// Effect clamp on the sensed consequence.
const MOTOR_EFF_LO: f32 = 0.5;
const MOTOR_EFF_HI: f32 = 2.0;
/// D-59 fault injection (verification only, registered): for beats >=
/// D59_FAULT_BEAT the world responds OPPOSITE to the action (eff =
/// 1.0 - drive*GAIN). A working organism-side detector must fire.
const D59_FAULT_BEAT: u64 = 60;
/// Harness-side ground-truth tracker: EMA alpha (expected drive + residual
/// sigma) and the 3-sigma violation rule. Instrumentation only - explicitly
/// NOT organism knowledge (cross-check for the organism-side signal).
const WORLD_EMA_ALPHA: f32 = 0.2;
const WORLD_SIGMA_K: f32 = 3.0;

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
    /// D-53b: fraction of NOVEL beats the organism DETECTED as novel
    /// (act = NOVEL/UNSURE, below th_known). The honest novelty metric -
    /// novel_recognized_frac was a tautology (always 0) and proved
    /// nothing about detection.
    pub novel_detected_frac: f32,
    /// D-53b: count of novel beats that decoded to a KNOWN symbol
    /// (contamination - org misattributed the novel pattern).
    pub novel_contaminated: u32,
    /// D-59: reflex-CIRCUIT novelty detection on D beats (min-L2 of the
    /// live reflex-band response vs per-symbol templates > REFLEX_TH_FAM).
    /// Runs ALONGSIDE the codec metric (novel_detected_frac) - the honest
    /// per-structure comparison asked by the user (the D-58 rule now lives
    /// in real life). None when reflex_k == 0.
    pub reflex_novel_detected_frac: Option<f32>,
    /// D-59: ORGANISM-side consequence-novelty fraction (motor mode): known
    /// beats whose reflex response violates the symbol's existing template
    /// - the organism's own reflex circuit registering "this does NOT do
    /// that". None unless motor_mode && reflex_k > 0.
    pub consequence_novel_frac: Option<f32>,
    /// D-59: HARNESS-side ground-truth tracker (instrumentation only, NOT
    /// the claim): known beats whose motor drive deviated > 3sigma from the
    /// per-symbol EMA expectation. Cross-checks consequence_novel_frac.
    /// None unless motor_mode && reflex_k > 0.
    pub world_consequence_violation_frac: Option<f32>,
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
        v2, window_ticks, None, None, false, None, "", 0, false)
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
    // D-50: i/o alphabet mode ("" = legacy 8ch A/C/D; d50 = 6ch A/C/E/D)
    mode: &str,
    // D-59 (docs/phase3/d59-reflex-integration-protocol.md): reflex band
    // size (0 = off, byte-identical baseline; must equal the network's
    // cfg.d58_reflex).
    reflex_k: usize,
    // D-59: closed-loop motor world (motor command -> sensed consequence ->
    // organism-side expectation check). false = off, byte-identical.
    motor_mode: bool,
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
    let mut novel_detected: u32 = 0; // D-53b: honest novelty-DETECTION count
    let rw = spec.r_window.max(1) as usize;
    let mut rcog: std::collections::VecDeque<bool> = std::collections::VecDeque::with_capacity(rw);
    let mut ob_window: std::collections::VecDeque<bool> = std::collections::VecDeque::with_capacity(rw);
    let mut tick = net.tick;
    // ---- D-59 state (reflex band + closed-loop motor world) ----
    // Reflex band = the last `reflex_k` neurons at loop entry (the nodes
    // appended at construction; mid-life births append AFTER them, so the
    // band cannot shift under growth).
    let reflex_base: usize = if reflex_k > 0 && net.cfg.d58_reflex == reflex_k {
        net.neurons.len() - reflex_k
    } else { 0 };
    let mut reflex_vec = vec![0.0f32; reflex_k];
    let mut templates: std::collections::BTreeMap<String, Vec<f32>> = Default::default();
    let (mut reflex_novel_detected, mut reflex_novel_n) = (0u32, 0u32);
    let (mut consequence_novel, mut consequence_novel_n) = (0u32, 0u32);
    let (mut world_viol, mut world_n) = (0u32, 0u32);
    // harness-side ground-truth: per-symbol EMA expected drive + residual sigma
    let mut exp_drive: std::collections::BTreeMap<String, f32> = Default::default();
    let mut sig_drive: std::collections::BTreeMap<String, f32> = Default::default();
    let mut exp_n: std::collections::BTreeMap<String, u32> = Default::default();
    let mut prev_drive: f32 = 0.0; // scalar drive of the previous beat's action
    let fault = std::env::var("D59_FAULT").is_ok();
    // diagnostics only (per-beat min-L2 vs templates); no behavior change
    let d59dbg = std::env::var("D59_DEBUG").is_ok();
    // D-60a (docs/phase3/d60-slow-parity-protocol.md): reflex-path
    // slow-state parity - clamp the BAND's own u_slow at beat end so the
    // next beat starts state-matched. Identity when unset. Registered.
    let d60_parity = std::env::var("D60_PARITY").is_ok();
    // D-60b (docs/phase3/d60b-input-parity-protocol.md): input-afferent
    // slow-state parity - clamp the INPUT neurons' u_slow at beat end.
    // Identity when unset. Registered.
    let d60b_parity = std::env::var("D60B_PARITY").is_ok();
    // D-62 (docs/phase3/d62-band-start-parity-protocol.md): full band
    // start-state reset at beat end (u_slow, v, z_latch, i_syn) - the
    // band's response becomes a deterministic function of the train.
    // Identity when unset. Registered.
    let d62_parity = std::env::var("D62_PARITY").is_ok();

    while beat < spec.beats && died.is_none() {
        // D-59: the previous action's consequence - motor drive modulates
        // THIS beat's afferent rate (eff); D59_FAULT reverses the effect
        // for beats >= D59_FAULT_BEAT (verification-only fault injection).
        let eff = if motor_mode {
            let d0 = if fault && beat >= D59_FAULT_BEAT { -prev_drive } else { prev_drive };
            (1.0 + d0 * MOTOR_GAIN).clamp(MOTOR_EFF_LO, MOTOR_EFF_HI)
        } else { 1.0 };
        let tr = if motor_mode {
            io::symbol_trains_mode_eff(&cur, mode, world_seed, eff)
        } else {
            io::symbol_trains_mode(&cur, mode, world_seed)
        };
        // D-50: known-set derived from refs (mode-agnostic: [A,C] or
        // [A,C,E]). A symbol is 'known' iff it has a captured ref.
        let is_known = refs.iter().any(|(p, _)| *p == cur);
        // D-60c (diagnostic only): band PRE-beat state snapshot - which
        // carry-over variable correlates with the beat-to-beat min-L2
        // variance (u_slow, g_drive afferent-EMA, v residual, i_syn tail).
        let (mut pre_u, mut pre_g, mut pre_v, mut pre_i) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        if reflex_k > 0 {
            for n in net.neurons.iter().skip(reflex_base).take(reflex_k) {
                pre_u += n.u_slow;
                pre_g += n.g_drive;
                pre_v += n.v;
                pre_i += n.i_syn;
            }
        }
        reflex_vec.iter_mut().for_each(|x| *x = 0.0);
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
                // D-59: accumulate the reflex band in the SAME pass (no
                // extra probe presentations - riding the real beats).
                if reflex_k > 0 {
                    let ci = c.0 as usize;
                    if ci >= reflex_base && ci < reflex_base + reflex_k {
                        reflex_vec[ci - reflex_base] += 1.0;
                    }
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
                let pe = if !is_known { 1.0 } else { 0.2 }; // birth pressure from novel/unrecognized beats
                if let Some((m, s)) = pe_state.as_mut() {
                    *m += (pe - *m) * 0.05;
                    *s = (*s + (pe - *m).abs()) * 0.5;
                }
                let sig = Signals {
                    prediction_error: pe,
                    pe_mean: pe_state.map(|x| x.0).unwrap_or(0.0),
                    pe_std: pe_state.map(|x| x.1).unwrap_or(0.0),
                    novelty: if !is_known { 1.0 } else { 0.0 },
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
        // D-60a: band slow-state parity at beat end (see protocol doc).
        if d60_parity && reflex_k > 0 {
            for n in net.neurons.iter_mut().skip(reflex_base).take(reflex_k) {
                n.u_slow = 0.0;
            }
        }
        // D-60b: input-afferent slow-state parity at beat end (see
        // protocol doc) - ids 0..channels.len() are the neurons each
        // InputChannel drives; the band's afferent source.
        if d60b_parity {
            let n_in = net.channels.len();
            for n in net.neurons.iter_mut().take(n_in) {
                n.u_slow = 0.0;
            }
        }
        // D-62: full band start-state reset at beat end (see protocol
        // doc) - u_slow, v, z_latch, i_syn so the next beat starts
        // state-clean and the band's response is a deterministic
        // function of the train.
        if d62_parity && reflex_k > 0 {
            let v_rest = net.cfg.lif.v_rest;
            for n in net.neurons.iter_mut().skip(reflex_base).take(reflex_k) {
                n.u_slow = 0.0;
                n.v = v_rest;
                n.z_latch = 0;
                n.i_syn = 0.0;
            }
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
        // D-59: motor command (linear proportional: activation -> drive)
        // and scalar drive from THIS beat's output.
        let drive = if motor_mode {
            let mc = io::motor(&out);
            (mc.rates_hz.iter().sum::<f32>() / mc.rates_hz.len().max(1) as f32 / 1000.0).clamp(0.0, 1.0)
        } else { 0.0 };
        // D-59 reflex verdicts + template maintenance. ORDERING RULE: a beat
        // is judged against the PRE-beat expectation FIRST, then the
        // template is overwritten (the organism's own just-experienced
        // response is the new per-symbol expectation; state-matched by
        // construction).
        if reflex_k > 0 {
            let min_l = if templates.is_empty() { f32::MAX } else {
                templates.values().map(|t| reflex::l2(&reflex_vec, t)).fold(f32::MAX, f32::min)
            };
            if d59dbg {
                let dists: Vec<(String, f32)> = templates
                    .iter().map(|(s, t)| (s.clone(), reflex::l2(&reflex_vec, t))).collect();
                // ownL2: distance to the symbol's OWN template (-1 when
                // none exists yet = first occurrence; only own-template
                // beats are eligible for the known-side familiarity
                // metric).
                let own_l2 = templates.get(&cur).map(|t| reflex::l2(&reflex_vec, t)).unwrap_or(-1.0);
                eprintln!("D59DBG beat={} cur={} is_known={} minL2={:.1} ownL2={:.1} dists={:?} rv={:?} pre_u={:.3} pre_g={:.3} pre_v={:.3} pre_i={:.3}",
                    beat, cur, is_known, min_l, own_l2, dists, reflex_vec, pre_u, pre_g, pre_v, pre_i);
            }
            if !is_known && !templates.is_empty() {
                // D beat vs known templates: NOVEL iff min-L2 > th (the
                // verified D-58 rule, now riding real beats).
                reflex_novel_n += 1;
                if min_l > reflex::REFLEX_TH_FAM { reflex_novel_detected += 1; }
            } else if is_known {
                if motor_mode && templates.contains_key(&cur) {
                    // organism-side consequence check ("knows this does
                    // that exactly"): the reflex response to a known symbol
                    // must match its template.
                    consequence_novel_n += 1;
                    if min_l > reflex::REFLEX_TH_FAM { consequence_novel += 1; }
                }
                templates.insert(cur.clone(), reflex_vec.clone());
            }
        }
        // D-59 harness-side ground-truth tracker (instrumentation ONLY,
        // explicitly NOT organism knowledge): per-symbol EMA expected
        // drive; a known beat whose drive deviates > 3sigma is a
        // world-side consequence violation (cross-check for the
        // organism-side signal). First sample initializes expectation;
        // violations judged from sample 3 on (sigma then has an update).
        if motor_mode && is_known {
            world_n += 1;
            let e = exp_drive.entry(cur.clone()).or_insert(0.0);
            let n0 = exp_n.entry(cur.clone()).or_insert(0);
            if *n0 == 0 {
                *e = drive;
            } else {
                let residual = (drive - *e).abs();
                let s = sig_drive.entry(cur.clone()).or_insert(0.0);
                if *n0 >= 2 && residual > WORLD_SIGMA_K * *s { world_viol += 1; }
                *s += WORLD_EMA_ALPHA * (residual - *s);
                *e += WORLD_EMA_ALPHA * (drive - *e);
            }
            *n0 += 1;
        }
        prev_drive = drive;
        if motor_mode {
            eprintln!("MOTOR beat={} drive={:.3} eff={:.3} cons_novel={} world_viol={} cons_frac={:.3} world_frac={:.3}",
                beat, drive, eff, consequence_novel, world_viol,
                consequence_novel as f32 / consequence_novel_n.max(1) as f32,
                world_viol as f32 / world_n.max(1) as f32);
        }
        // recognition: a known beat is recognized ONLY if it decodes to
        // its own ref (exact match). A mismatched known beat (cur=A decoded
        // C) is a recognition FAILURE, not a hit.
        let recognized = is_known && act == cur;
        rcog.push_back(recognized);
        if rcog.len() > rw { rcog.pop_front(); }
        if is_known { known_n += 1; if recognized { known_ok += 1; } } else {
            novel_n += 1;
            // Legacy semantics UNCHANGED (D-53b): novel_ok += recognized
            // stays tautologically 0 (is_known=false -> recognized=false)
            // so historical novel_recognized_frac JSONs stay interpretable.
            if recognized { novel_ok += 1; }
            // D-53b honest detection: a novel beat is correctly HANDLED
            // iff act is NOVEL/UNSURE (below th_known) - i.e. the org
            // DETECTED it is not any known symbol. Track separately.
            if act == "NOVEL" || act == "UNSURE" { novel_detected += 1; }
        }
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
            // approach-known: the decoded known symbol is re-presented;
            // withdraw/novel/QUIET -> move AWAY to a DIFFERENT known ref
            // (D-23 semantics, generalized to N known via refs).
            if refs.iter().any(|(p, _)| *p == act.as_str()) {
                act.clone()
            } else {
                let knowns: Vec<&str> = refs.iter().map(|(p, _)| p.as_str()).collect();
                let some_known = knowns.iter().find(|&&k| k != cur).copied().unwrap_or("A");
                some_known.to_string()
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
        novel_detected_frac: novel_detected as f32 / novel_n.max(1) as f32,
        novel_contaminated: novel_n - novel_detected,
        reflex_novel_detected_frac: if reflex_k > 0 {
            Some(reflex_novel_detected as f32 / reflex_novel_n.max(1) as f32)
        } else { None },
        consequence_novel_frac: if motor_mode && reflex_k > 0 {
            Some(consequence_novel as f32 / consequence_novel_n.max(1) as f32)
        } else { None },
        world_consequence_violation_frac: if motor_mode && reflex_k > 0 {
            Some(world_viol as f32 / world_n.max(1) as f32)
        } else { None },
        known_beats: known_n, novel_beats: novel_n,
    }
}