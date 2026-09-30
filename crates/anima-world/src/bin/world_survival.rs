//! 3D world organism goals (plan: local://3d-organism-goals-plan.md):
//! food/energy/death, aimed movement under gravity, the event-level
//! novelty + fault alarms, 6-generation evolution, and a watch-only
//! live viewer.
//!
//! Loop (per beat): encode (48 vision + 8 proprio cells) -> network
//! (56/40/12 + reflex band K=8, STDP on, V2 on + band exclusivity) ->
//! verdict (novelty vs rolling W=5 window, judge BEFORE update) ->
//! motor -> world.step -> GRAVITY rule (this bin ONLY; World::step
//! untouched) -> touch_food (refill then drain) -> telemetry.
//!
//! All 12 motor channels stay LIVE; the organism gets no body
//! instructions. Environment rules (registered, NOT tuned on outcomes):
//! gravity 9.81 m/s^2, energy 100 start / 1.0 drain per beat / +30.0
//! refill within radius 6.0 of the food (green pyramid), death exactly
//! when the tank hits 0, fault negation from beat 30 (WL2_FAULT=1) with
//! a 3-beat zero-rate freeze on the first post-30 alarm, fitness =
//! final_energy + 3.0 * food_touches.
//!
//! Usage: world_survival [world_seed] [net_seed] [beats=60] [--serve PORT]
//!        world_survival --evolve            (seeds 20260924 and 7, 6 gens)
//! env: WL2_FAULT=1 (negate motor demand from beat 30; alarm freezes 3
//!      beats), WL2_MOTOR=0 (zero rates; registered diagnostic).
//!
//! Determinism: same (world_seed, net_seed) -> byte-identical WL2
//! stream; --serve is pure reads (frames never feed back).

use std::collections::{BTreeMap, HashSet};

use anima_core::network::{InputChannelId, InputFrame, Network, NetworkConfig, NeuronId, SynapseId, Tick};
use anima_core::plasticity::{stdp_tick, StdpParams, Traces};
use anima_core::structural_v2::V2Plasticity;
use anima_exp::encoders::SensoryEncoder;
use anima_exp::io;
use anima_world::camera::rasterize;
use anima_world::{MotorDrive, RetinaEncoder, World};
use anima_world::world::Vec3;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

/// Reflex band size (the novelty alarm's readout), frozen D-70.3 stack.
const K: usize = 8;
/// Novelty threshold + template window size (frozen D-70.3 numbers).
const TH: f32 = 60.0;
const W: usize = 5;
/// Environment gravity (m/s^2), applied in THIS bin only (registered,
/// never tuned).
const GRAVITY: f32 = 9.81;
/// WL2_FAULT=1 negates the motor demand from this beat on.
const FAULT_T: u64 = 30;
/// Alarm reaction: body freezes (zero rates) for 3 beats after the
/// first event-level alarm at/after FAULT_T (registered consequence).
const FREEZE_BEATS: u64 = 3;
/// Evolution (--evolve): N=4 organisms, 6 generations, these seeds.
const N_POP: usize = 4;
const GENS: u64 = 6;
const EVOLVE_SEEDS: [u64; 2] = [20260924, 7];
/// Weight inheritance: +/-10% mutation with p=0.1 (mirror evolve.rs).
const W_MUT_P: f32 = 0.10;
const W_MUT_AMP: f32 = 0.10;
/// Fitness tiebreaker: +3.0 per food touch (registered).
const TOUCH_BONUS: f32 = 3.0;
/// Exploration / curriculum constants (registered, NOT tuned on outcomes).
const BEATS_DEFAULT: u64 = 60;
const BEATS_LONG: u64 = 600;           // longer lives: single-life --explore watch
const BEATS_EVOLVE: u64 = 100;         // evolve+explore: short enough that gen-0 survival is reachable (~1s/life) — with the speed metabolic cost, a cruiser survives ~100 beats on a few touches; long lives wipe gen-0 flat (dead→0)
const MOTOR_NOISE: f32 = 0.15;         // motor noise stddev (frac of rate)
const FOOD_START_DIST: f32 = 8.0;      // initial food distance from spawn
const FOOD_MAX_DIST: f32 = 40.0;       // cap on food distance
const FOOD_MIN_DIST: f32 = 8.0;        // floor (>6 eat radius — never inside the mouth)
const FOOD_STEP: f32 = 2.0;            // distance increase per touch
const FOOD_CONTRACT: f32 = 3.0;        // distance decrease on failure
const FAILURES_BEFORE_CONTRACT: u32 = 3; // failures before contracting
/// Food-smell channels (--smell): body-relative direction+range, so
/// "food left -> steer left" is a learnable STDP mapping. 6 channels
/// after the 48 vision + 8 proprio (input ids 56..62).
const SMELL_CHANNELS: usize = 6;
/// 'Ultimate organism' bar: a champion that records ≥2 food touches on
/// NATURAL food (curriculum stripped) is declared the ultimate forager
/// and that seed stops mating (--generalize champion probing).
const ULTIMATE_BAR: u32 = 2;
/// Eligibility-based credit (food is THE reward, per operator): a
/// decaying trace over recently-active output (steering) neurons; on a
/// FOOD TOUCH the reward potentiates the afferents of those eligible
/// outputs. Reward value is purely food — actions/novelty are never
/// rewarded directly.
const ELIG_DECAY: f32 = 0.5;  // trace decay per beat (window ~3-4 beats)
const ELIG_BOOST: f32 = 0.03; // per-touch potentiation, xelig
/// Replay consolidation count: the touch-time eligibility trace is
/// re-applied this many times (dopamine-like consolidation) so one meal
/// meaningfully etches the approach mapping instead of a single weak tap.
const REPLAY_N: u32 = 8;
/// Intrinsic motivation (--intrinsic via WL2_INTR): reward on NOVEL beats.
/// Per-novel-beat potentiation (attenuated vs a food touch since novelty
/// is frequent); gated on the now de-saturated brain where novelty has
/// behavioral variance to act on.
const INTR_GAIN: f32 = 0.06;
/// Reward-gated (dopamine-like) plasticity window: after a reward, normal
/// STDP is paused for this many beats so the just-etched potentiation
/// survives — the credit dominates instead of being swamped by ongoing
/// unsupervised Hebbian drift (the root of all within-life failures).
const REWARD_WIN: usize = 3;
/// R-STDP (SL2_RSTDPP=1): a food touch emits a dopamine pulse that
/// multiplies the ongoing STDP gain for a few beats — reward decides
/// which coincident activity gets etched (three-factor rule) instead of
/// the legacy one-shot weight bump. Registered pre-run values.
const RSTDP_GAIN: f32 = 3.0;
const RSTDP_GAIN_CAP: f32 = 4.0;
const RSTDP_DECAY: f32 = 0.6;
/// A — SL2_DENSITY=1: grade evolution on SUSTAINED foraging rather than
/// terminal energy. Density term = touches^2 / beats (count x rate):
/// rewards touches spread across the life, giving the GA a continuous
/// gradient past the point where a survivor reaches food (the 137.5
/// saturation). Sized to the energy scale (design choice, not tuned):
/// DENSITY_BONUS=20 makes ~30 touches over a 100-beat evolve life
/// (touches^2/beats = 9) worth ~180 = ~1.8 full start tanks, so sustained
/// feeding outranks a single-burst survivor; sparse touches stay dwarfed
/// by energy exactly as before. (The R-STDP lesson: keep the learning
/// signal on the same scale as the selector or it vanishes.)
const DENSITY_BONUS: f32 = 20.0;
/// SL2_ARS — area-restricted-search selection term. persistence = sum over
/// touches of exp(-gap/ARS_TAU) where gap is beats since the previous
/// counted touch: QUICK re-approach after each success (staying in the
/// feeding locale) adds more. Fitness adds PERSIST_BONUS * persistence.
/// Registered values (design scale, not outcome-tuned): ARS_TAU=20 so a
/// ~5-beat retouch adds ~0.78; 30 quick touches ≈ persistence 23, worth
/// ~92 ≈ a full start tank — energy-commensurate like DENSITY_BONUS.
const ARS_TAU: f32 = 20.0;
const PERSIST_BONUS: f32 = 4.0;
/// Body-to-food refill radius (world.rs touch_food uses d < 6.0). Used by
/// the SL2_STASIS re-entry guard to detect a genuine leave-and-return.
const EAT_RADIUS: f32 = 6.0;
/// SL2_NEST — rest-recovery home. Stamina is a motor CAPABILITY constraint
/// (not a homing bias): spent by movement, restored at the nest radius,
/// and rates scale by min(1, stamina/NEST_CAP_FLOOR) so exhaustion
/// immobilizes — the organism must find the nest via its own senses/memory.
/// FATIGUE is per VELOCITY-unit (|vel| ~5 units/sec here), NOT per
/// distance-unit. Registered at 0.4 -> ~2 stamina/beat at full tilt,
/// ~50 continuous beats to exhaust: a 28-out + 28-back + meal-contingency
/// shuttle fits with running-room but still demands periodic rest.
/// (Corrected from 1.2, which mis-scaled to ~6/beat -> instant exhaustion.)
const NEST_MAX_STAMINA: f32 = 100.0;
const NEST_FATIGUE: f32 = 0.4;      // stamina lost per velocity-unit per beat
const NEST_REST: f32 = 60.0;        // stamina regained per beat at the nest
const NEST_RADIUS: f32 = 8.0;       // home radius (nest landmark)
const NEST_CAP_FLOOR: f32 = 25.0;   // full ability while stamina >= this
/// SL2_NESTFIT — reward genuine returns home so the GA is pushed toward
/// the feed->return shuttle. Without it the nest is only an affliction
/// (exhaust if you wander) and evolution prefers parked-at-home survivors
/// (measured: elite_ret=0). Bonus per nest-return, energy-commensurate.
const NEST_RETURN_BONUS: f32 = 40.0;
/// SL2_MB — mushroom-body-motif learning (evidence-tagged, operator-
/// approved). (1) Sparse expansion: internal pool 256 KC from ~62 input
/// channels (fly: ~50 PN -> ~2000 KC, ~40x, fan-in ~5-7; downscaled 4x
/// for single-threaded compute), connectivity ~0.10 (~6 inputs/KC, the
/// biological sparse fan-in). (2) Credit-locus change: reward-gated
/// potentiation routed to the KC INPUT synapses (DAN-style dopamine gates
/// incoming weights where sparse separation happens) instead of the output
/// band — the measured failure was credit landing on the wrong locus, not
/// magnitude (elig bump writes ~0.24/synapse, ~100x the 1e-3 floor).
const MB_POOL: usize = 256;
const MB_CONN: f32 = 0.10;
/// A — Hunger drive (internal need): H = 1 - energy/start rises as the
/// tank falls; when hungry it amplifies the food-smell channel's gain so
/// "smell food" drives the network harder (endogenous motivation).
const HUNGER_GAIN: f32 = 1.0;
const HUNGER_THRESH: f32 = 0.3;
/// C — Efference copy: external-novelty threshold. A tiny LMS predictor
/// learns how the organism's own motor command changes the scene; its
/// prediction error is EXTERNAL novelty (world did something new), the
/// self-induced change is discounted (agency).
const EXT_NOV_THRESH: f32 = 0.015;
/// D — Place-cell + habit-value architecture (hippocampus+striatum).
const PLACE_N: usize = 64;   // place prototypes ("where am I")
const H_A: usize = 8;        // discrete steering actions
const H_EPS0: f32 = 0.40;    // exploration prob (decays toward 0 by ~0.5/life)
const H_ALPHA: f32 = 0.20;   // TD learning rate
const H_GAMMA: f32 = 0.90;   // TD discount
const H_LAMBDA: f32 = 0.80;  // TD(λ) eligibility decay
const H_PHI: f32 = 0.05;     // place-prototype competitive update rate
/// Discretized steering actions -> 12-channel motor rates.
const ACTION_A: [[f32; 12]; H_A] = [
    [100.0, 0.0, 60.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // straight
    [100.0, 0.0, 60.0, 150.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // turn L
    [100.0, 0.0, 60.0, 0.0, 150.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // turn R
    [30.0, 0.0, 150.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],  // climb
    [20.0, 0.0, 0.0, 0.0, 0.0, 150.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],  // pitch down
    [0.0, 0.0, 60.0, 0.0, 0.0, 0.0, 0.0, 150.0, 0.0, 0.0, 0.0, 0.0],   // brake
    [80.0, 150.0, 60.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0], // strafe L
    [80.0, -150.0, 60.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],// strafe R
];
/// Option 3 — continuous odor-gradient→steering chemotaxis controller.
/// yaw ∝ (right−left), pitch ∝ (up−down), thrust ∝ scent — a gradient
/// response that exists at ANY distance (general by construction, unlike
/// discrete state-values). Learned: per-bearing-channel sensitivity gains
/// reinforce on food reward (smell→steer association is rewarded), so the
/// response IS food-shaped. Bacteria do reactive chemotaxis; we make the
/// gains learnable.
const GRAD_STEER: f32 = 2.5;   // yaw/pitch response gain — enough authority that pure pursuit CONVERGES (orbit radius v/omega < 6) instead of circling
const GRAD_THRUST_BASE: f32 = 45.0; // slower approach -> convergent pursuit
const GRAD_THRUST_SCALE: f32 = 1.5;
const GRAD_LR: f32 = 0.3;       // reward-modulated sensitivity learning
// Option-2 task-side lever: persistent natural homing (proximity-brake).
const GRAD_NEAR_THRESH: f32 = 35.0;  // near-smell above this = very close (<~8u): brake
/// Coverage grid: arena divided into COV_N x COV_N cells; an organism
/// "explores" by visiting distinct cells (novelty-drive falsifier).
const COV_N: usize = 20;
/// Progressive-survival mode (SL2_PROG=1): the organism starts with a
/// large lifespan and food starts CLOSE; every WIN pushes the next food
/// +PROG_STEP farther (wins escalate difficulty, failures do not
/// contract). It dies only when the food recedes past its reach —
/// measuring the escalation level reached.
const PROG_START: f32 = 10.0;
const PROG_STEP: f32 = 3.0;
const PROG_LIFE: f32 = 4000.0; // lifespan if not overridden by SL2_START
/// Metabolic cost of action (explore): the faster the organism MOVES,
/// the faster energy burns — so a full-tilt buzzer starves while a
/// precise cruiser saves. Speed is the signal (motor demand is always
/// saturated ~1800 here, so it would be a flat handicap). Magnitude kept
/// ENABLING: gen-0 bodies only know max throttle (binding problem has no
/// throttle control yet), so a strong cost wipes everyone before they
/// can learn to cruise; 0.15 (V=5 -> +0.75/beat ≈ 75% over base) builds
/// a real gradient AND lets today's 7-touch gen-0 survive.
const METABOLIC_SPEED: f32 = 0.15; // /beat per unit speed (V=5 -> +0.75)
/// Fix B (anti premature-convergence): if the same elite has been best
/// for ≥ this many consecutive gens it is carried WITH mutation (not
/// frozen) so the population can't collapse onto one unreachable genome
/// — the 361/137/232 exact-pin that recurred in every prior config.
const ELITE_STALL_GENS: u32 = 5;
/// Heritable-synapse growth cap (scalability guard): a child stops
/// inheriting new parent-born V2 synapses once it reaches this many —
/// keeps per-beat cost bounded across long runs (the ~7x crawl).
fn max_synapses() -> usize {
    std::env::var("SL2_SYNCAP").ok().and_then(|v| v.parse().ok()).unwrap_or(60_000)
}
/// Exploration floor for the habit (option 2): keeps ε from annealing
/// away so the trained habit still explores in a new (natural-food)
/// distribution instead of repeating one rigid trajectory. Default 0.05;
/// raise for transfer tests (SL2_EPSFLOOR=0.25).
fn eps_floor() -> f32 {
    std::env::var("SL2_EPSFLOOR").ok().and_then(|v| v.parse().ok()).unwrap_or(0.05)
}


fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut world_seed: u64 = 20260924;
    let mut net_seed: u64 = 77;
    let mut beats: u64 = 60;
    let mut evolve = false;
    let mut serve: Option<u16> = None;
    let mut long = false;
    let mut explore = false;
    let mut generalize = false;
    let mut smell = false;
    let mut transfer = false;
    // --until <N>: run evolution until the first generation with a food
    // touch, capped at N generations (operator-requested long run;
    // absent = the registered 6-gen run, byte-identical).
    let mut until: Option<u64> = None;
    let mut pos = 0usize;
    let mut i = 0usize;
    while i < args.len() {
        match args[i].as_str() {
            "--evolve" => evolve = true,
            "--until" => {
                i += 1;
                until = args.get(i).and_then(|s| s.parse().ok());
                if until.is_none() {
                    eprintln!("--until needs a generation cap");
                }
            }
            "--serve" => {
                i += 1;
                serve = args.get(i).and_then(|s| s.parse().ok());
                if serve.is_none() {
                    eprintln!("--serve needs a port");
                }
            }
            "--long" => long = true,
            "--explore" => explore = true,
            "--generalize" => generalize = true,
            "--smell" => smell = true,
            "--transfer" => transfer = true,
            s => match pos {
                0 => {
                    world_seed = s.parse().unwrap_or(20260924);
                    pos = 1;
                }
                1 => {
                    net_seed = s.parse().unwrap_or(77);
                    pos = 2;
                }
                2 => {
                    beats = s.parse().unwrap_or(60);
                    pos = 3;
                }
                _ => eprintln!("ignoring extra arg {s}"),
            },
        }
        i += 1;
    }
    let fault = std::env::var("WL2_FAULT").is_ok();
    let motor_zero = std::env::var("WL2_MOTOR").map(|v| v == "0").unwrap_or(false);
    let viewer = serve.map(Server::start);
    let max_gens = until.unwrap_or(GENS).max(1);
    let beats = if long { BEATS_LONG } else if evolve && explore { BEATS_EVOLVE } else { beats };
    if transfer {
        run_transfer(world_seed, net_seed, fault, motor_zero, viewer.as_ref(), smell);
    } else if evolve {
        evolve_run(max_gens, beats, fault, motor_zero, viewer.as_ref(), explore, generalize, smell);
    } else if explore {
        run_explore(world_seed, net_seed, fault, motor_zero, viewer.as_ref(), smell);
    } else {
        let r = run_life(world_seed, net_seed, beats, fault, motor_zero, viewer.as_ref(), false, smell);
        println!(
            "WL2 RESULT beats={} died_at={} final_energy={:.1} food_touches={} novel_flags={}",
            r.beats,
            r.died_at.map(|d| d.to_string()).unwrap_or_else(|| "none".to_string()),
            r.final_energy,
            r.food_touches,
            r.novel_flags
        );
    }
    // Keep the process alive for viewers (serve never returns); killed
    // via SIGTERM (4i).
    if let Some(v) = viewer {
        v.join();
    }
}

/// One organism's life outcome (world-survival fitness inputs).
struct LifeResult {
    beats: u64,
    died_at: Option<u64>,
    final_energy: f32,
    food_touches: u32,
    novel_flags: u32,
    coverage: u32,
    ext_news: u32,
    /// Final body position (x, z) — behavioral characterization for
    /// novelty-search selection (SL2_NOVELTY).
    final_pos: [f32; 2],
    /// ARS persistence (SL2_ARS): sum of exp(-inter-touch-gap/TAU) over
    /// touches — rewards quick re-approach (staying in the locale).
    persistence: f32,
    /// SL2_NEST: number of genuine returns to the nest (re-entries into
    /// the nest radius, false->true). Distinguishes a real forage->return
    /// shuttle from park-at-home. Attach to the elite for the verdict.
    nest_returns: u32,
}

/// Fresh 56/40/12 network with the K-neuron reflex band (frozen shape).
/// Input layer size: 48 vision + 8 proprio, plus the optional 6
/// food-smell channels (--smell). Default stays 56 (byte-identical).
fn net_inputs(smell: bool) -> usize {
    56 + if smell { SMELL_CHANNELS } else { 0 }
}

/// Networking override helpers (all env-gated; absent → 40/0.0, matching
/// every byte-identical default). -- experiment: SL2_POOL=128 SL2_INH=0.5
/// gives the bigger, laterally-inhibited internal pool that can ACTUALLY
/// bind smell/vision to steering (fix el 2).
fn pool_size() -> usize {
    std::env::var("SL2_POOL").ok().and_then(|v| v.parse().ok()).unwrap_or(40)
}
fn inh_gain() -> f32 {
    std::env::var("SL2_INH").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0)
}

fn build_net(n_in: usize, seed: u64) -> Network {
    // Output inhibition (D-46: same-tick pairwise between output
    // neurons): with it, outputs COMPETE instead of all firing at max —
    // the motor demand gains a real dynamic range so the organism can
    // steer. 0.15 = evolve.rs's calibrated known-good range. Env-gated
    // (WL2_OINH=1) so the default stays byte-identical.
    let oinh = if std::env::var("WL2_OINH").is_ok() { 0.15 } else { 0.0 };
    // SL2_IINH: internal-band lateral inhibition gain (registered first
    // value 0.5; parsed as a float, 0/absent = identity).
    let iinh = std::env::var("SL2_IINH").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    // SL2_THETA: global theta_rel_mean override (deferred-contingency
    // probe — bounds whether threshold sparsification has ANY effect on
    // the saturated internal band before committing to a per-band branch).
    let theta = std::env::var("SL2_THETA").ok().and_then(|v| v.parse().ok()).unwrap_or(1.0);
    // SL2_AMPL: input-amplitude override — the measured saturation source
    // is the 52.0 input drive (every KC driven over threshold every tick).
    // Scales it down to find where the internal band de-saturates.
    let ampl = std::env::var("SL2_AMPL").ok().and_then(|v| v.parse().ok()).unwrap_or(52.0);
    // SL2_HET: KC-excitability heterogeneity (theta/plateau/tau sd) — the
    // biological mechanism for sparse partial activation (uniform scaling
    // is a 0%/100% cliff, measured). All sd fields default 0.0 (identity).
    let het = std::env::var("SL2_HET").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let mb = std::env::var("SL2_MB").is_ok();
    let n_int = if mb { MB_POOL } else { pool_size() };
    let conn = if mb { MB_CONN } else { 0.038 };
    let cfg = NetworkConfig {
        connectivity: conn, w_init: 0.2, amplitude: ampl, adaptation_tau_ms: 200.0,
        adaptation_gain: 0.05, inhibition_gain: inh_gain(), slow_state_beta: 0.0046875,
        slow_state_tau_ms: 5000.0, slow_state_beta_drive: false, latch_enable: true,
        theta_rel_mean: theta, theta_rel_sd: het, u_plateau_rel_mean: 1.0, u_plateau_rel_sd: het,
        tau_het_rel_sd: het, phi_rel: 0.5, eta_rel: 0.0,
        output_inhibition_gain: oinh,
        internal_inhibition_gain: iinh,
        d58_reflex: K,
        ..NetworkConfig::default()
    };
    Network::new(cfg, n_in, n_int, 12, seed)
}

/// V2 params mirroring evolve's v2_params() (frozen D-70.3).
fn v2_params() -> anima_core::network::V2Params {
    anima_core::network::V2Params {
        p_in: 0.5, w_in_lo: 0.02, w_in_hi: 0.06, p_rec: 0.2, w_rec_lo: 0.005, w_rec_hi: 0.02,
        t_e: 0.8, assembly_protect: true, p_max_frac: 0.75, w_consolidate_min: 0.05,
        alloc_residual: true, dormant_reserve: false, recruit_gain: false,
        d_core: true, d_claim: true, d_sparse: false, d_elig: false, d_elig_ro: false, d_ing: false,
        c_slots: 6, w_c_init: 0.01, delta_perm: 0.01, decay_c: 0.99, theta_permanent: 0.05,
        w_c_permanent: 0.02, theta_die: 0.005, p_cand_in: 0.5, p_cand_rec: 0.5,
        theta_prune: 0.005, prune_windows: 10, b_e: 40, b_i: 10, p_inh: 0.3, w_inh_lo: 0.01,
        w_inh_hi: 0.03, a_inh: 0.005, decay_inh: 0.98, w_inh_max: 0.10, window_ticks: 100,
        m2_buckets: 1, m2_epoch_windows: 1, disable_m2: false, disable_m3_m4: false,
        disable_m5: false, disable_m6: false,
    }
}

/// Single-run mode: fresh network, one life. `explore` turns on the
/// exploration curriculum (food starts close, motor noise, curiosity
/// reward) — see life_with.
fn run_life(
    world_seed: u64, net_seed: u64, beats: u64, fault: bool, motor_zero: bool,
    viewer: Option<&Server>, explore: bool, smell: bool,
) -> LifeResult {
    let mut net = build_net(net_inputs(smell), net_seed);
    let (r, _) = life_with(&mut net, world_seed, beats, fault, motor_zero, viewer, None, None, explore, explore, smell, FOOD_START_DIST, None);
    r
}

/// One organism's life over `beats` beats (shared by single-run and
/// evolution). `gen`/`org` only label observer frames. When `explore`
/// is set the loop runs the exploration curriculum: food starts CLOSE
/// ahead and walks in/out with success/failure (snake-style), motor
/// rates get deterministic noise, and novelty (the flag) pays a small
/// intrinsic energy reward + boosts next-beat STDP (one-beat-delayed
/// credit, reward after the fact).
fn life_with(
    net: &mut Network, world_seed: u64, beats: u64, fault: bool, motor_zero: bool,
    viewer: Option<&Server>, gen: Option<u64>, org: Option<u64>, explore: bool,
    curriculum: bool, smell: bool, curriculum_start: f32, hb_in: Option<Habit>,
) -> (LifeResult, Option<Habit>) {
    let mut world = World::new(world_seed);
    // Learning-horizon override (SL2_START): start with a much larger
    // energy budget so a single life spans thousands of beats, giving
    // within-life plasticity (STDP / replayed eligibility / novelty)
    // enough reward events to compound — earlier lives died in ~53-68
    // beats, truncating all learning. Default unset = registered 100.
    if let Ok(v) = std::env::var("SL2_START") {
        if let Ok(s) = v.parse::<f32>() {
            world.body.energy = s;
            world.body.alive = true;
        }
    }
    let start_energy = world.body.energy;
    // A — hunger state + C — efference-copy LMS predictor (self-motion →
    // scene-brightness mapping learned online; error = external novelty).
    let mut hunger: f32 = 1.0 - (world.body.energy / start_energy.max(1.0)).clamp(0.0, 1.0);
    let mut prev_ret_mean: f32 = 0.0; // last beat's scene mean luminance
    let mut pf_bias: f32 = 2.0;
    let mut pf_ret: f32 = 0.0;        // predicts mean_lum(t) from mean_lum(t-1)
    let mut pf_mot: Vec<f32> = vec![0.0; 12]; // ... from each motor rate
    let mut ext_news: u32 = 0;
    let mut out_last: Vec<f32> = vec![0.0; 12]; // previous beat's motor rates (efference)
    // D — place+habit controller (SL2_HABIT=1): the LIF network still
    // senses, but motors are driven by ε-greedy TD(λ) over place codes.
    let habit = std::env::var("SL2_HABIT").is_ok() || hb_in.is_some();
    let mut hb = hb_in.or_else(|| {
        habit.then(|| Habit::new(12, io::derive_seed64(world_seed, io::hash_str("habit"), net_seed_at_start(net))))
    });
    // Option 3 — continuous chemotaxis controller (SL2_GRAD=1).
    let grad = std::env::var("SL2_GRAD").is_ok();
    let mut gb: Option<GradientBrain> = grad.then(GradientBrain::new);
    let mut grad_last_sm: [f32; 4] = [0.0; 4];
    let mut last_reward: Option<f32> = None;
    // Env rescale (SL2_FOODR): bounded food radius — makes natural
    // (no-curriculum) homing reachable at this body's scale.
    let food_radius: f32 = std::env::var("SL2_FOODR").ok().and_then(|v| v.parse().ok()).unwrap_or(f32::MAX);
    // Progressive-survival: large lifespan + food escalates on each win.
    let prog = std::env::var("SL2_PROG").is_ok();
    // B — SL2_STASIS=1: food does NOT teleport on touch; it stays in
    // place, so a forager that finds food once can eat repeatedly. This
    // within-life food stability is what makes learning/memory pay and
    // lets sustained feeding accumulate into selection-visible fitness.
    let stasis = std::env::var("SL2_STASIS").is_ok();
    // SL2_NEST — rest-recovery home (stamina capability constraint). The
    // nest is a visible NON-food landmark primitive in the scene (so the
    // retina can see it and the organism must learn to return via its own
    // senses/memory — NO directional homing bias). Stamina drains with
    // movement and is restored at the nest radius.
    let nest = std::env::var("SL2_NEST").is_ok();
    let nest_pos = if nest {
        // The non-food landmark NEAREST the spawn point (deterministic),
        // so the nest is always reachable for a fresh body. The raw first
        // non-food primitive was seed-luckily-far (34 units on 20260924)
        // and geometrically unreachable — corrected.
        world
            .primitives
            .iter()
            .filter(|p| !p.food)
            .min_by(|a, b| {
                let da = (a.pos.x - world.body.pos.x).powi(2) + (a.pos.z - world.body.pos.z).powi(2);
                let db = (b.pos.x - world.body.pos.x).powi(2) + (b.pos.z - world.body.pos.z).powi(2);
                da.partial_cmp(&db).unwrap()
            })
            .map(|p| p.pos)
    } else {
        None
    };
    let mut stamina: f32 = NEST_MAX_STAMINA;
    let mut nest_returns: u32 = 0;
    let mut was_at_nest: bool = true; // spawned inside the nest radius
    if nest && std::env::var("NEST_PROBE").is_ok() {
        let sp = world.body.pos;
        let np = nest_pos.unwrap_or(sp);
        let d0 = ((sp.x - np.x).powi(2) + (sp.z - np.z).powi(2)).sqrt();
        eprintln!(
            "NEST_PROBE spawn=({:.1},{:.1}) nest_prim=({:.1},{:.1}) dist={:.1}",
            sp.x, sp.z, np.x, np.z, d0
        );
        for (i, p) in world.primitives.iter().enumerate() {
            let d = ((sp.x - p.pos.x).powi(2) + (sp.z - p.pos.z).powi(2)).sqrt();
            eprintln!(
                "  prim[{i}] food={} color={:?} pos=({:.1},{:.1}) dist_spawn={:.1}",
                p.food, p.color, p.pos.x, p.pos.z, d
            );
        }
    }
    // Re-entry anti-farm flag (SL2_STASIS): set true when the body leaves
    // the food's eat radius; only a genuine re-approach (reenter true at
    // refill) counts as a touch — a parked grazing body never counts.
    let mut reenter = false;
    // ARS persistence tracker (persistence rewards quick re-approach).
    let mut persistence: f32 = 0.0;
    let mut last_touch_beat: i64 = -1;
    if prog && std::env::var("SL2_START").is_err() {
        world.body.energy = PROG_LIFE;
    }
    // Tank ceiling for the SL2_STASIS energy cap — the energy the body
    // actually started this life with (prog raises it to PROG_LIFE AFTER
    // the earlier start_energy capture, so re-read here).
    let tank_cap = world.body.energy;
    let food_idx = world.primitives.iter().position(|p| p.food).expect("scene has food");
    // Snake-mode food respawner: ONE stream per (world_seed, net_seed) —
    // deterministic, and (deliberately) the SAME stream regardless of
    // --serve (the observer never perturbs the sim).
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(io::derive_seed64(world_seed, io::hash_str("snake"), net_seed_at_start(net)));
    // Exploration curriculum (--explore): food starts CLOSE ahead on the
    // forward axis, then walks outward on each success and inward on a
    // failure streak, so the organism is nudged to reach for food and
    // then pursue it as it moves. Stripped for the generalization test
    // (food stays at the world's natural seed position).
    let mut food_distance: f32 = if prog { PROG_START } else { curriculum_start };
    let mut failures_since_touch: u32 = 0;
    if curriculum {
        // Curriculum places food on a RANDOM heading at the generation's
        // start distance (option 2: train in the test distribution — food
        // appears at all bearings, not just directly ahead, so the value
        // map covers unseen placements).
        let ang = rng.gen::<f32>() * std::f32::consts::TAU;
        let fp = Vec3::new(
            world.body.pos.x + ang.cos() * curriculum_start,
            2.0,
            world.body.pos.z + ang.sin() * curriculum_start,
        );
        world.primitives[food_idx].pos = fp;
    }
    // Env rescale: bounded food radius overrides start placement so the
    // task is solvable at this organism's scale.
    if food_radius < f32::MAX {
        respawn_near(&mut world, food_idx, &mut rng, food_radius);
    }
    // Progressive-survival: food starts close; each WIN escalates the
    // distance (measured here by food_start being dynamic).
    if prog {
        world.primitives[food_idx].pos = Vec3::new(
            world.body.pos.x + PROG_START,
            2.0,
            world.body.pos.z,
        );
    }
    // Curiosity credit: reward = FOOD only (eligibility), never novelty.
    if let Some(v) = viewer {
        v.push_scene(render_scene_live(&world, Some(spawn_xz(world_seed))));
    }
    // Output/reflex-band bounds derived from the ACTUAL anatomy so any
    // pool size works: outputs are the n_out neurons just before the
    // K-neuron reflex band at the very end.
    let net_n = net.neurons.len();
    let n_out = 12usize;
    // SL2_MB: the internal (KC) band = inputs..outputs. Reward-gated
    // plasticity reroutes here (DAN-style input-locus credit).
    let mb = std::env::var("SL2_MB").is_ok();
    let n_ins = net_n - K - n_out - if mb { MB_POOL } else { pool_size() };
    let mb_lo = n_ins;
    let mb_hi = mb_lo + if mb { MB_POOL } else { pool_size() };
    let mut kc_spike_set: HashSet<u32> = HashSet::new();
    let kc_every: u64 = std::env::var("KCPROBE_EVERY").ok().and_then(|v| v.parse().ok()).unwrap_or(50);
    let band_lo = net_n - K;      // reflex band = last K
    let out_hi = band_lo;         // outputs = the n_out before the band
    let out_lo = out_hi - n_out;
    let p = StdpParams {
        tau_plus: 20.0, tau_minus: 20.0, a_plus: 0.005, a_minus: 0.0053,
        decay: 1e-6, w_min: 0.0, w_max: 1.0,
    };
    let mut traces = Traces::new(net, 20.0);
    let mut v2 = V2Plasticity::new(net, v2_params(), None);

    // Rolling template window: band responses of the last W beats.
    // ORDERING RULE (D-59): judge the beat BEFORE pushing it.
    let mut window: Vec<Vec<f32>> = Vec::new();
    let mut food_touches = 0u32;
    let mut novel_flags = 0u32;
    let mut died_at: Option<u64> = None;
    let mut fault_fired = false;
    let mut freeze_left: u64 = 0;
    // Intrinsic motivation (WL2_INTR=1): novelty is the reward; the
    // food-touch synaptic reward is disabled. Needs the de-saturated
    // brain (WL2_OINH) to have variable motor output to act on.
    let intrinsic = std::env::var("WL2_INTR").is_ok();
    let no_reward = std::env::var("WL2_NOREWARD").is_ok();
    // SL2_HMEXP — hunger-modulated exploration (drive -> motor-noise gain,
    // same rule class as hunger->smell-gain): well-fed = SETTLED (below
    // baseline noise, stay in the patch), hungry = normal/wide search.
    // noise_scale = SATIATED_NOISE + (1 - SATIATED_NOISE) * hunger, so fed
    // (hunger 0) = 0.4x (settled) and hungry (hunger 1) = 1.0x (roam).
    // A plain (1 + gain*hunger) arm only widens when hungry and never
    // settles — registered correction to the stated settled-when-fed goal.
    let hmexp = std::env::var("SL2_HMEXP").is_ok();
    const SATIATED_NOISE: f32 = 0.4; // fed noise fraction of the baseline
    // Reward-gated plasticity: while gate_left > 0, normal STDP is paused
    // so the reward-etched potentiation survives.
    let mut gate_left: usize = 0;
    // R-STDP (SL2_RSTDPP=1): global dopamine pulse at food-touch
    // modulates the STDP gain for a decaying window (three-factor rule).
    let rstdp = std::env::var("SL2_RSTDPP").is_ok();
    let mut dopamine: f32 = 0.0;
    if rstdp && std::env::var("RSTDP_PROBE").is_ok() {
        eprintln!("RSTDP_PROBE active in life_with (rstdp=true)");
    }
    // SW_PROBE: identity-safe gross weight-change instrumentation. Tracks
    // per-beat sum of |Δw| over all synapses, split by whether dopamine
    // was > 0 that beat, to separate "inert by saturation" (4x0=0) from
    // "weak rule" — reported once at life end.
    let sw_probe = std::env::var("SW_PROBE").is_ok();
    let mut sw_boost: f64 = 0.0;
    let mut sw_base: f64 = 0.0;
    let mut sw_boost_beats: u32 = 0;
    // Coverage grid: distinct arena cells visited (novelty-drive metric).
    let mut visited = vec![false; COV_N * COV_N];
    let mut coverage = 0u32;
    // Eligibility trace (per-neuron): output/steering neurons reset to
    // 1.0 on firing, decay each beat; a food touch potentiates the
    // afferents of recently-active outputs.
    let mut elig = vec![0.0f32; net.neurons.len()];

    for beat in 0..beats {
        if rstdp {
            dopamine *= RSTDP_DECAY;
        }
        let w_snap: Vec<f32> = if sw_probe {
            net.synapses.iter().map(|s| s.w).collect()
        } else {
            Vec::new()
        };
        let beat_boosted = sw_probe && dopamine > 0.3;
        let enc = RetinaEncoder::from_world(&world);
        // Observer-only pure read: the luminance the organism literally
        // sees this beat (frames never feed back into the sim).
        let lum_now = viewer.map(|_| rasterize(&world).luminance);
        let mut train = enc.frames(world_seed);
        // A — hunger: recompute from the current tank; amplify the food
        // smell when hungry (gated to explore).
        hunger = 1.0 - (world.body.energy / start_energy.max(1.0)).clamp(0.0, 1.0);
        let smell_gain = if explore && hunger > HUNGER_THRESH { 1.0 + HUNGER_GAIN * hunger } else { 1.0 };
        // SL2_HMEXP: drive-modulated exploration gain (settle when fed).
        let noise_scale = if hmexp { SATIATED_NOISE + (1.0 - SATIATED_NOISE) * hunger } else { 1.0 };
        if smell {
            let fp = world.primitives[food_idx].pos;
            train.extend(smell_trains(&world, fp, world_seed, smell_gain));
            train.sort_unstable_by_key(|(t, _)| *t);
        }
        // C — efference copy: predict this beat's scene brightness from
        // the previous beat's (brightness + own motor command); the
        // prediction error is EXTERNAL novelty (world did something new).
        let lum = rasterize(&world).luminance;
        let ret_mean = lum.iter().sum::<f32>() / 48.0;
        let mut ext_this = false;
        if explore {
            let predicted = pf_bias + pf_ret * prev_ret_mean + pf_mot.iter().zip(out_last.iter()).map(|(w, r)| w * r).sum::<f32>();
            let error = (ret_mean - predicted).abs();
            ext_this = error > EXT_NOV_THRESH;
            if ext_this {
                ext_news += 1;
            }
            // LMS update: learn the self-motion→brightness mapping.
            let lr = 0.02;
            let e = ret_mean - predicted;
            pf_bias += lr * e;
            pf_ret += lr * e * prev_ret_mean;
            for (w, r) in pf_mot.iter_mut().zip(out_last.iter()) {
                *w += lr * e * r;
            }
        }
        prev_ret_mean = ret_mean;
        // Intrinsic reward uses EXTERNAL novelty rather than raw flag
        // (which includes self-induced change).
        let mut out = vec![0.0f32; n_out];
        let mut rv = vec![0.0f32; K];
        // Plain STDP (reward = FOOD only, delivered by eligibility below).
        for t in 0..io::BEAT_MS {
            let frame = InputFrame {
                tick: net.tick,
                spikes: train.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect(),
            };
            let ev = net.step(&frame);
            // KCPROBE (MB): distinct internal-band spikers this beat —
            // measures de-saturation (saturated = all 256 fire every beat).
            if mb && std::env::var("KCPROBE").is_ok() && kc_every > 0 && beat % kc_every == 0 {
                if t == 0 {
                    kc_spike_set.clear();
                }
                for c in &ev.spikes {
                    let ci = c.0 as usize;
                    if ci >= mb_lo && ci < mb_hi {
                        kc_spike_set.insert(ci as u32);
                    }
                }
            }
            for c in &ev.spikes {
                let ci = c.0 as usize;
                if ci >= out_lo && ci < out_hi {
                    out[ci - out_lo] += 1.0;
                    elig[ci] = 1.0; // steering neuron active -> eligibility reset
                } else if mb && ci >= mb_lo && ci < mb_hi {
                    elig[ci] = 1.0; // MB: KC active -> its input synapses eligible
                } else if ci >= band_lo && ci < band_lo + K {
                    rv[ci - band_lo] += 1.0;
                }
            }
            traces.step(net, &ev.spikes);
            // Reward-gated plasticity: pausing STDP after a reward lets
            // the etched potentiation survive; resumes after the window.
            if gate_left == 0 {
                let gain = if rstdp {
                    (1.0 + RSTDP_GAIN * dopamine).min(RSTDP_GAIN_CAP)
                } else {
                    1.0
                };
                let _ = stdp_tick(&p, net, &traces, &ev.spikes, gain, None);
            }
            v2.tick(&ev.spikes);
            if net.tick.0 % 100 == 0 && net.tick.0 > 0 {
                let tk = net.tick;
                let _ = v2.window(net, tk);
            }
            net.tick = Tick(net.tick.0 + 1);
        }
        if mb && std::env::var("KCPROBE").is_ok() && kc_every > 0 && beat % kc_every == 0 {
            let n = kc_spike_set.len();
            eprintln!("KCPROBE beat={beat} kc_spikers={n}/{}", MB_POOL);
            kc_spike_set.clear();
        }
        // Verdict BEFORE window update (ordering rule, D-59).
        let min_l = if window.is_empty() {
            f32::MAX
        } else {
            window.iter().map(|w| reflex_l2(&rv, w)).fold(f32::MAX, f32::min)
        };
        let flagged = min_l > TH;
        if beat >= 10 && flagged {
            novel_flags += 1;
        }
        // Intrinsic motivation: a NOVEL beat is itself the reward — via
        // the same eligibility machinery (credits the prior steering that
        // produced this novel view). Food-touch reward is disabled below.
        if intrinsic && !no_reward && explore && ext_this {
            for o in out_lo..out_hi {
                let e = elig[o];
                if e <= 0.0 {
                    continue;
                }
                let ids: Vec<SynapseId> = net.incoming[o].clone();
                let inc = INTR_GAIN * e;
                for sid in ids {
                    let s = &mut net.synapses[sid.idx()];
                    if s.silent_ticks != u64::MAX {
                        s.w = (s.w + inc).min(1.0);
                    }
                }
            }
            gate_left = REWARD_WIN; // freeze STDP so this credit survives
        }
        let mc = MotorDrive::from_counts(&out);
        // D — habit controller: locate place, feed last reward, pick an
        // action, set rates from the action basis (+ explore noise).
        let mut habit_rates: Option<Vec<f32>> = None;
        if grad {
            // Option 3: continuous odor-gradient -> steering (general).
            let sm = smell_rates(&world, world.primitives[food_idx].pos);
            let sm4 = [sm[0], sm[1], sm[2], sm[3]];
            grad_last_sm = sm4;
            let mut r = gb.as_mut().unwrap().act(sm4, sm[4]); // near = proximity
            if explore {
                for rr in r.iter_mut() {
                    let n = (rng.gen::<f32>() * 2.0 - 1.0) * MOTOR_NOISE * noise_scale * 150.0;
                    *rr = (*rr + n).clamp(-150.0, 150.0);
                }
            }
            habit_rates = Some(r);
        } else if habit {
            let h = hb.as_mut().unwrap();
            // Distance-INVARIANT place sense: food bearing (right/left/
            // up/down — same at 8 or 28 units) + proprio. NOT the retina
            // (whose object-size changes with distance → near/far food
            // would get different place codes and block value transfer).
            let sm = smell_rates(&world, world.primitives[food_idx].pos);
            let prop = enc.proprio_rates();
            let mut sense = Vec::with_capacity(12);
            sense.extend_from_slice(&sm[..4]); // bearing only
            sense.extend_from_slice(&prop);
            let s = h.loc(&sense);
            if let Some(r) = last_reward.take() {
                h.reward(r, s);
            }
            let a = h.pick(s);
            h.set_prev(s, a);
            let mut r = ACTION_A[a].to_vec();
            if explore {
                for rr in r.iter_mut() {
                    let n = (rng.gen::<f32>() * 2.0 - 1.0) * MOTOR_NOISE * noise_scale * 150.0;
                    *rr = (*rr + n).clamp(-150.0, 150.0);
                }
            }
            habit_rates = Some(r);
        }
        // Fault reaction (goal 5): the first event-level alarm at/after
        // FAULT_T freezes the body for FREEZE_BEATS beats (zero rates).
        if fault && beat >= FAULT_T && flagged && !fault_fired {
            fault_fired = true;
            freeze_left = FREEZE_BEATS;
        }
        let negate = fault && beat >= FAULT_T;
        let mut rates: Vec<f32> = if freeze_left > 0 {
            freeze_left -= 1;
            vec![0.0f32; n_out]
        } else if motor_zero {
            vec![0.0f32; n_out]
        } else if let Some(hr) = habit_rates {
            hr
        } else {
            // Clamp the demand to the body's ±150 swing FIRST, then add
            // exploration noise and clamp again. (Adding noise to the raw
            // ~1932 Hz demand was swallowed by the ±150 clamp — saturated
            // channels stayed pinned and the organism flew one fixed
            // diagonal forever.)
            let mut r: Vec<f32> =
                mc.rates_hz.iter().map(|r2| (if negate { -r2 } else { *r2 }).clamp(-150.0, 150.0)).collect();
            if explore {
                for rr in r.iter_mut() {
                    let n = (rng.gen::<f32>() * 2.0 - 1.0) * MOTOR_NOISE * noise_scale * 150.0;
                    *rr = (*rr + n).clamp(-150.0, 150.0);
                }
            }
            r
        };
        // SL2_NEST: capability constraint — exhausted (stamina below
        // NEST_CAP_FLOOR) scales all motor rates down, immobilizing a
        // spent body away from the nest so it must rest to recover. No
        // directional homing bias: the body must reach the nest itself.
        if nest {
            let c = (stamina / NEST_CAP_FLOOR).min(1.0).max(0.0);
            let probe = nest && std::env::var("NEST_PROBE").is_ok() && beat < 80;
            let pre: f32 = if probe {
                rates.iter().map(|r| r.abs()).sum::<f32>() / rates.len() as f32
            } else {
                0.0
            };
            if c < 1.0 {
                for rr in rates.iter_mut() {
                    *rr *= c;
                }
            }
            let post: f32 = if probe {
                rates.iter().map(|r| r.abs()).sum::<f32>() / rates.len() as f32
            } else {
                0.0
            };
            if probe && (c < 1.0 || post > 0.0) {
                eprintln!("NEST_RATES beat={beat} c={c:.2} pre={pre:.1} post={post:.1}");
            }
        }
        world.step(&rates);
        // Gravity: environment physics, applied in this bin ONLY
        // (registered 9.81 m/s^2; World::step keeps its recorded
        // behavior). Ground clamp re-applied after the pull.
        world.body.vel.y -= GRAVITY * anima_world::world::DT;
        if world.body.pos.y < 0.0 {
            world.body.pos.y = 0.0;
            world.body.vel.y = 0.0;
        }
        // Metabolic cost of action (explore): speed-proportional burn. A
        // full-tilt body (V≈5) costs +1.5/beat vs ~0 for a hover — a real
        // efficiency gradient under the food-is-reward regime.
        if explore && world.body.alive {
            let speed = (world.body.vel.x * world.body.vel.x
                + world.body.vel.y * world.body.vel.y
                + world.body.vel.z * world.body.vel.z)
            .sqrt();
            let cost = METABOLIC_SPEED * speed;
            world.body.energy -= cost;
            if world.body.energy <= 0.0 {
                world.body.energy = 0.0;
                world.body.alive = false;
            }
        }
        // SL2_NEST: stamina drains with movement, restored at the nest.
        if nest {
            if let Some(np) = nest_pos {
                let dx = world.body.pos.x - np.x;
                let dz = world.body.pos.z - np.z;
                let at = dx * dx + dz * dz < NEST_RADIUS * NEST_RADIUS;
                let s2 = (world.body.vel.x * world.body.vel.x
                    + world.body.vel.y * world.body.vel.y
                    + world.body.vel.z * world.body.vel.z)
                    .sqrt();
                if at {
                    stamina = (stamina + NEST_REST).min(NEST_MAX_STAMINA);
                    if !was_at_nest {
                        nest_returns += 1; // genuine return (re-entry)
                    }
                } else {
                    stamina = (stamina - NEST_FATIGUE * s2).max(0.0);
                }
                was_at_nest = at;
                if nest && std::env::var("NEST_PROBE").is_ok() && beat < 40 {
                    eprintln!("NEST_PROBE beat={beat} stamina={stamina:.1} speed={s2:.2} at_nest={at}");
                }
            }
        }
        // Energy (goal 1): refill on food contact, then the 1.0/beat
        // drain; death exactly when the tank reaches 0. A touch ALSO
        // respawns the food (snake mode) — the refill radius guarantees
        // the new food never spawns inside the organism's mouth.
        let energy_before = world.body.energy;
        world.touch_food();
        let touched = if stasis {
            // Re-entry anti-farm + tank bound. Under stasis the food is
            // stationary, so a parked body would refill every beat (energy
            // is uncapped). Fix: (1) cap the tank at start_energy so a
            // grazing body cannot accumulate unbounded energy; (2) count a
            // touch only if the body actually LEFT the eat radius since
            // the last touch (reenter) — a parked body never leaves, so it
            // never counts. Genuine foragers travel away (energy drops,
            // reenter set on exit) and their return is a counted touch.
            world.body.energy = world.body.energy.min(tank_cap);
            let fp = world.primitives[food_idx].pos;
            let dx = world.body.pos.x - fp.x;
            let dz = world.body.pos.z - fp.z;
            let fd = (dx * dx + dz * dz).sqrt();
            if fd > EAT_RADIUS {
                reenter = true;
            }
            let t = world.body.energy > energy_before && reenter;
            if t {
                reenter = false;
            }
            t
        } else {
            world.body.energy > energy_before
        };
        if touched {
            food_touches += 1;
            failures_since_touch = 0;
            // ARS persistence: quick re-approach (few beats since last
            // touch) adds exp(-gap/ARS_TAU); the first touch (no prior)
            // contributes nothing.
            if last_touch_beat >= 0 {
                let gap = beat as i64 - last_touch_beat;
                persistence += (-(gap as f32) / ARS_TAU).exp();
            }
            last_touch_beat = beat as i64;
            // FOOD is the reward: potentiate the afferents feeding the
            // steering neurons active in the preceding few beats (their
            // eligibility), weighted by decayed trace. The reward value
            // is the food contact only — the eligibility trace merely
            // decides WHICH output synapses get strengthened. REPLAY
            // consolidation re-applies the same trace REPLAY_N times so
            // one meal etches the association. Opening the reward gate
            // pauses STDP for REWARD_WIN beats so the credit survives
            // (dopamine-gated learning). Disabled in intrinsic or
            // no-reward mode.
            if rstdp {
                // R-STDP: emit the dopamine pulse; the network's own
                // STDP (now gain-modulated) etches the coincident
                // activity instead of this legacy weight bump.
                if std::env::var("RSTDP_PROBE").is_ok() {
                    eprintln!("RSTDP_PROBE touch at beat {beat} dop-set, gain={:.2}", (1.0 + RSTDP_GAIN * 1.0).min(RSTDP_GAIN_CAP));
                }
                dopamine = 1.0;
            } else if explore && !intrinsic && !no_reward {
                // MB (SL2_MB): reward gates the KC INPUT synapses (the
                // sparse expansion locus) instead of the output band —
                // the measured defect was credit at the wrong locus.
                let tgt_hi = if mb { mb_hi } else { out_hi };
                let tgt_lo = if mb { mb_lo } else { out_lo };
                let mb_probe = mb && std::env::var("MB_PROBE").is_ok();
                let mut kc_elig: u32 = 0;
                let mut kc_pot: u32 = 0;
                for o in tgt_lo..tgt_hi {
                    let e = elig[o];
                    if mb_probe && e > 0.0 {
                        kc_elig += 1;
                    }
                    if e <= 0.0 {
                        continue;
                    }
                    let ids: Vec<SynapseId> = net.incoming[o].clone();
                    let inc = ELIG_BOOST * e * REPLAY_N as f32;
                    for sid in ids {
                        let s = &mut net.synapses[sid.idx()];
                        if s.silent_ticks != u64::MAX {
                            s.w = (s.w + inc).min(1.0);
                            if mb_probe {
                                kc_pot += 1;
                            }
                        }
                    }
                }
                if mb_probe {
                    eprintln!("MB_PROBE touch beat={beat} kc_elig={kc_elig} kc_pot={kc_pot} tgt=({tgt_lo}..{tgt_hi})");
                }
                gate_left = REWARD_WIN; // freeze STDP so this credit survives
            }
            if stasis {
                // B: food STAYS in place on touch — a forager that found
                // it once feeds repeatedly (within-life stability makes
                // learning/memory pay). Anti-farm handled by the energy
                // tank cap: a parked full body can't gain more (see the
                // energy region), so touches fire only on genuine
                // re-approach from below full.
            } else if prog {
                // Each WIN pushes food farther (no contract on failure).
                food_distance = (food_distance + PROG_STEP).min(45.0);
                respawn_near(&mut world, food_idx, &mut rng, food_distance);
            } else if curriculum {
                // Curriculum: reward success by moving food further out.
                food_distance = (food_distance + FOOD_STEP).min(FOOD_MAX_DIST);
                respawn_food_at(&mut world, food_idx, &mut rng, food_distance);
            } else if food_radius < f32::MAX {
                respawn_near(&mut world, food_idx, &mut rng, food_radius);
            } else {
                respawn_food(&mut world, food_idx, &mut rng);
            }
            if let Some(v) = viewer {
                v.push_scene(render_scene_live(&world, None));
            }
        } else if curriculum && !prog {
            // Curriculum failure streak: pull food back closer so the
            // organism is nudged toward it again (self-correcting).
            failures_since_touch += 1;
            if failures_since_touch >= FAILURES_BEFORE_CONTRACT {
                failures_since_touch = 0;
                food_distance = (food_distance - FOOD_CONTRACT).max(FOOD_MIN_DIST);
                respawn_food_at(&mut world, food_idx, &mut rng, food_distance);
                if let Some(v) = viewer {
                    v.push_scene(render_scene_live(&world, None));
                }
            }
        }
        // Food is the reward (D): per-step TD reward (30 on touch, else 0),
        // credited when the next beat's place is known.
        if habit {
            last_reward = Some(if touched { 30.0 } else { 0.0 });
        }
        // Option 3: food sharpens the gradient-response sensitivity.
        if grad && touched {
            gb.as_mut().unwrap().reward(grad_last_sm);
        }
        let b = &world.body;
        // Coverage: mark the arena cell the body is in (novelty metric).
        {
            let c = COV_N as f32;
            let ix = (((b.pos.x + 50.0) / 100.0 * c).floor() as isize).clamp(0, COV_N as isize - 1) as usize;
            let iz = (((b.pos.z + 50.0) / 100.0 * c).floor() as isize).clamp(0, COV_N as isize - 1) as usize;
            let cell = iz * COV_N + ix;
            if !visited[cell] {
                visited[cell] = true;
                coverage += 1;
            }
        }
        let fp = world.primitives[food_idx].pos;
        let food_dist = ((b.pos.x - fp.x).powi(2)
            + (b.pos.y - fp.y).powi(2)
            + (b.pos.z - fp.z).powi(2))
        .sqrt();
        let out_sum: f32 = out.iter().sum();
        println!(
            "WL2 beat={} energy={:.1} alive={} food_dist={:.2} out_sum={:.0} minL2={:.1} flag={}",
            beat, b.energy, if b.alive { 1 } else { 0 }, food_dist, out_sum, min_l,
            flagged as u8
        );
        if let Some(v) = viewer {
            let prop = enc.proprio_rates();
            let fp = world.primitives[food_idx].pos;
            v.push(frame_json(beat, &world, gen, org, lum_now.as_deref().unwrap_or(&[]), &prop, &rates, min_l, flagged, fault_fired, food_dist, &fp));
        }
        // Window update AFTER the verdict (recent experience).
        window.push(rv);
        if window.len() > W {
            window.remove(0);
        }
        if !world.body.alive {
            died_at = Some(beat + 1);
            break;
        }
// Decay the eligibility trace for the next beat's credit window.
        for e in elig.iter_mut() {
            *e *= ELIG_DECAY;
        }
        if sw_probe {
            let mut d = 0.0f64;
            for (s, &w0) in net.synapses.iter().zip(&w_snap) {
                d += (s.w - w0).abs() as f64;
            }
            if beat_boosted {
                sw_boost += d;
                sw_boost_beats += 1;
            } else {
                sw_base += d;
            }
        }
        // Efference copy uses THIS beat's commanded rates next beat.
        out_last.copy_from_slice(&rates);
        // Reward gate winds down: STDP remains paused for the window.
        if gate_left > 0 {
            gate_left -= 1;
        }
        // Band start-state parity (D-62 stack) at beat end.
        let v_rest = net.cfg.lif.v_rest;
        for n in net.neurons.iter_mut().skip(band_lo).take(K) {
            n.u_slow = 0.0;
            n.v = v_rest;
            n.z_latch = 0;
            n.i_syn = 0.0;
        }
    }
    if sw_probe {
        eprintln!(
            "SW_PROBE life: touches={food_touches} beats={beats} gross_dw_boost={sw_boost:.6} ({} beats) gross_dw_base={sw_base:.6} ({} beats) ratio={:.3}",
            sw_boost_beats,
            0.max(beats as i64 - sw_boost_beats as i64) as u32,
            if sw_base > 0.0 { sw_boost / sw_base } else { f64::INFINITY }
        );
    }
    (
        LifeResult {
            beats: died_at.unwrap_or(beats),
            died_at,
            final_energy: world.body.energy,
            food_touches,
            novel_flags,
            coverage,
            ext_news,
            final_pos: [world.body.pos.x, world.body.pos.z],
            persistence,
            nest_returns,
        },
        hb,
    )
}

/// Same L2 as the shared reflex module (anima-exp::reflex); local copy
/// keeps this bin free of probe-presentation machinery.
fn reflex_l2(a: &[f32], b: &[f32]) -> f32 {
    let mut s = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        let d = x - y;
        s += d * d;
    }
    s.sqrt()
}

// ---------------------------------------------------------------- evolve

/// Evolution (goal 4): N=4 organisms per generation; the top-2 by
/// fitness breed (best -> 2 offspring, 2nd -> 1), the best is carried
/// UNCHANGED; offspring inherit weights by (pre, post) key with +/-10%
/// mutation (p=0.1); 6 generations on each of EVOLVE_SEEDS (or until
/// the first generation with a food touch, capped at `max_gens` —
/// --until mode; a touch is the first real fitness gradient).
fn evolve_run(
    max_gens: u64, beats: u64, fault: bool, motor_zero: bool, viewer: Option<&Server>,
    explore: bool, generalize: bool, smell: bool,
) {
    for &esec in &EVOLVE_SEEDS {
        println!("=== seed {esec} ===");
        let mut pop: Vec<(Network, u64)> = (0..N_POP)
            .map(|i| {
                let s = io::derive_seed64(esec, 0, i as u64);
                (build_net(net_inputs(smell), s), s)
            })
            .collect();
        let mut gen_best: Vec<f32> = Vec::new();
        let mut last_win_ret: u32 = 0;
        let mut last_elite: u64 = u64::MAX;
        let mut elite_streak: u32 = 0;
        // Novelty-search selection (SL2_NOVELTY=1): survivors rank by
        // behavioral novelty against an archive of past winners' final
        // positions (minimal-criterion coevolution, Lehman/Stanley).
        let novelty_sel = std::env::var("SL2_NOVELTY").is_ok();
        let mut archive: Vec<[u32; 3]> = Vec::new();
        // A — SL2_DENSITY=1: grade fitness on sustained foraging density
        // (touches^2 / beats) in addition to terminal energy + touches.
        let density = std::env::var("SL2_DENSITY").is_ok();
        // ARS — SL2_ARS=1: grade fitness on quick re-approach (persistence:
        // exp(-inter-touch-gap/TAU)) in addition to energy + density.
        let ars = std::env::var("SL2_ARS").is_ok();
        // SL2_NESTFIT — reward nest returns (positive homing pressure).
        let nest_fit = std::env::var("SL2_NESTFIT").is_ok();
        for g in 0..max_gens {
            let mut fits: Vec<(f32, usize, u64)> = Vec::new();
            let mut bcs: Vec<[u32; 3]> = Vec::new();
            let mut nest_ret: Vec<u32> = Vec::new();
            let mut deaths = 0u32;
            let mut ftot = 0u32;
            let mut esum = 0.0f32;
            // Per-generation curriculum ramp: food starts FARTHER each
            // generation (8 + 3·g, capped 40) so selection is forced to
            // build approach from longer range — the shaping fix for the
            // generalization failure.
            let gen_start = (FOOD_START_DIST + 3.0 * g as f32).min(FOOD_MAX_DIST);
            for (i, (net, s)) in pop.iter_mut().enumerate() {
                let (r, _) = life_with(net, esec, beats, fault, motor_zero, viewer, Some(g), Some(i as u64), explore, explore, smell, gen_start, None);
                if r.died_at.is_some() {
                    deaths += 1;
                }
                ftot += r.food_touches;
                esum += r.final_energy;
                nest_ret.push(r.nest_returns);
                let mut fit = if r.died_at.is_some() {
                    0.0 // D-38 death gate: dead = 0, unconditionally — a
                    // quick-die-toucher must NOT out-rank a live survivor.
                } else if ars {
                    // ARS: add persistence (quick re-approach/staying).
                    r.final_energy + TOUCH_BONUS * r.food_touches as f32
                        + DENSITY_BONUS * (r.food_touches as f32).powi(2) / beats.max(1) as f32
                        + PERSIST_BONUS * r.persistence
                } else if density {
                    r.final_energy + TOUCH_BONUS * r.food_touches as f32
                        + DENSITY_BONUS * (r.food_touches as f32).powi(2) / beats.max(1) as f32
                } else {
                    r.final_energy + TOUCH_BONUS * r.food_touches as f32
                };
                if nest_fit && r.died_at.is_none() {
                    fit += NEST_RETURN_BONUS * r.nest_returns as f32;
                }
                fits.push((fit, i, *s));
                // Behavior characterization: quantized final (x, z) +
                // alive flag -> 10x10 arena bins + survival bit.
                let xb = (((r.final_pos[0] + 50.0) / 10.0) as i32).clamp(0, 9) as u32;
                let zb = (((r.final_pos[1] + 50.0) / 10.0) as i32).clamp(0, 9) as u32;
                bcs.push([xb, zb, if r.died_at.is_none() { 1 } else { 0 }]);
            }
            let mean: f32 = fits.iter().map(|(f, _, _)| f).sum::<f32>() / fits.len() as f32;
            let best = fits.iter().map(|(f, _, _)| *f).fold(f32::MIN, f32::max);
            gen_best.push(best);
            // Novelty of each organism vs the archive (0 when archive
            // empty or novelty search off).
            let mut nov: Vec<f32> = vec![0.0; N_POP];
            if novelty_sel {
                for (n, bc) in bcs.iter().enumerate() {
                    if !archive.is_empty() {
                        nov[n] = archive
                            .iter()
                            .map(|a| {
                                ((a[0] != bc[0]) as u32 + (a[1] != bc[1]) as u32 + (a[2] != bc[2]) as u32) as f32
                            })
                            .sum::<f32>()
                            / (3.0 * archive.len() as f32);
                    }
                }
            }
            let mut order: Vec<usize> = (0..N_POP).collect();
            if novelty_sel {
                // Minimal criterion + behavioral novelty: survivors
                // (fit > 0) rank by (alive, novelty, fitness), so the
                // death gate is preserved while novelty breaks ties
                // among survivors — escapes the deceptive objective.
                order.sort_by(|&a, &b| {
                    let ka = (fits[a].0 > 0.0, nov[a], fits[a].0);
                    let kb = (fits[b].0 > 0.0, nov[b], fits[b].0);
                    kb.partial_cmp(&ka).unwrap()
                });
            } else {
                order.sort_by(|&a, &b| fits[b].0.partial_cmp(&fits[a].0).unwrap());
            }
            let win_nov = if novelty_sel && !archive.is_empty() { nov[order[0]] } else { 0.0 };
            let win_ret = nest_ret[order[0]];
            last_win_ret = win_ret;
            println!(
                "GEN {g}: fit=[{}] mean={mean:.1} best={best:.1} deaths={deaths} food_touches={ftot} sum={esum:.1}{} ret_win={win_ret}",
                fits.iter().map(|(f, _, _)| format!("{f:.1}")).collect::<Vec<_>>().join(" "),
                if novelty_sel { format!(" nov_win={win_nov:.2}") } else { String::new() }
            );
            // TARGET (only in normal mode): a generation where the BEST
            // organism survives with a surplus (best > 40) is the first
            // sign of real food pursuit. In champion mode (--generalize)
            // we do NOT stop — keep mating toward the ultimate organism.
            if !generalize && std::env::var("SL2_NOTARGET").is_err() && best > 40.0 {
                println!("TARGET: seed {esec} surplus survivor at gen {g} (best={best:.1} food_touches={ftot})");
                break;
            }
            // Selection order was computed above (fitness or novelty).
            let mut next: Vec<(Network, u64)> = Vec::new();
            // ELITE: the winner carried into the next generation
            // UNCHANGED (rebuild from its own seed, weights copied
            // without mutation).
            let elite_idx = order[0];
            let elite_seed = pop[elite_idx].1;
            // Fix B: track how many consecutive gens THIS seed has been
            // best; after ELITE_STALL_GENS, carry it WITH mutation so a
            // stagnated winner keeps searching instead of freezing.
            if elite_seed == last_elite {
                elite_streak += 1;
            } else {
                last_elite = elite_seed;
                elite_streak = 0;
            }
            let mutate_elite = elite_streak >= ELITE_STALL_GENS;
            next.push((breed_inner(&pop[elite_idx].0, elite_seed, mutate_elite), elite_seed));
            // Offspring: best -> 2, second -> 1 (mirror evolve.rs).
            for (pi, &idx) in order.iter().take(2).enumerate() {
                let n_off: u64 = if pi == 0 { 2 } else { 1 };
                for _ in 0..n_off {
                    let slot = next.len();
                    let cs = io::derive_seed64(esec, g + 1, slot as u64);
                    let child = breed_inner(&pop[idx].0, cs, true);
                    next.push((child, cs));
                }
            }
            pop = next;
            // Archive the winner's BC (cap 50, evict oldest) so novelty
            // pressures diverge from what has already been selected.
            if novelty_sel {
                archive.push(bcs[order[0]]);
                if archive.len() > 50 {
                    archive.remove(0);
                }
            }
            // Champion mode (--generalize): after mating, probe THIS
            // generation's elite (pop[0] — best of the last gen, carried
            // unchanged) on NATURAL food. A FRESH copy is probed so the
            // probe's own plasticity never contaminates the lineage. The
            // only way to eat is to genuinely navigate to food — reaching
            // ULTIMATE_BAR means the ultimate forager emerged: stop.
            // (Fix B: the elite is now mutated on stall, so same seed no
            // longer implies an unchanged genome — always probe.)
            if generalize {
                let elite_seed = pop[0].1;
                let mut probe = breed_inner(&pop[0].0, elite_seed, false);
                let (pr, _) = life_with(&mut probe, esec, BEATS_LONG, fault, motor_zero, viewer, None, None, true, false, smell, FOOD_START_DIST, None);
                if pr.food_touches >= ULTIMATE_BAR {
                    println!("ULTIMATE seed {esec} at gen {g} net_seed={elite_seed} natural-food touches={} final_energy={:.1}",
                        pr.food_touches, pr.final_energy);
                    break;
                }
            }
        }
        println!(
            " seed {esec} RESULT: best g0={:.1} -> g{}={:.1} elite_ret={}",
            gen_best[0],
            gen_best.len() - 1,
            *gen_best.last().unwrap(),
            last_win_ret
        );
    }
}

/// Inherit the parent's weights by (pre, post) key into a fresh child of
/// the SAME anatomy (no size mutation), mutate each inherited weight
/// +/-10% with p=0.1 when `mutate`; parent-born synapses (V2 growth
/// during life) are re-added when both endpoints exist in the child —
/// mirror of evolve.rs breed_inner, simplified (no size mutation).
fn breed_inner(parent: &Network, seed: u64, mutate: bool) -> Network {
    // BTreeMap: ORDERED iteration — residual-add and mutation draws run
    // against ordered keys (HashMap order is per-process nondeterministic).
    let mut wmap: BTreeMap<(u32, u32), f32> = BTreeMap::new();
    for s in &parent.synapses {
        if s.silent_ticks == u64::MAX {
            continue;
        }
        wmap.insert((s.pre.0, s.post.0), s.w);
    }
    let n_in = parent.neurons.len() - pool_size() - 12 - K; // same anatomy as parent
    let mut child = build_net(n_in, seed);
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
    let mut consumed: Vec<(u32, u32)> = Vec::new();
    for s in child.synapses.iter_mut() {
        let key = (s.pre.0, s.post.0);
        if s.silent_ticks != u64::MAX {
            if let Some(w) = wmap.get(&key).copied() {
                let mut w2 = w;
                if mutate && rng.gen::<f32>() < W_MUT_P {
                    w2 += (rng.gen::<f32>() * 2.0 - 1.0) * W_MUT_AMP * w2;
                }
                s.w = w2.clamp(0.0, 1.0);
                consumed.push(key);
            }
        }
    }
    for k in &consumed {
        wmap.remove(k);
    }
    // Scalability guard: heritable V2-consolidated synapses must not
    // compound without bound across generations (the observed ~7x crawl).
    // Only re-add parent-born synapses while the child is under the cap.
    let cap = max_synapses();
    let n_child = child.neurons.len() as u32;
    let have: HashSet<(u32, u32)> = child
        .synapses
        .iter()
        .filter(|s| s.silent_ticks != u64::MAX)
        .map(|s| (s.pre.0, s.post.0))
        .collect();
    for ((pre, post), w) in &wmap {
        if *pre < n_child && *post < n_child && !have.contains(&(*pre, *post)) {
            if child.synapses.len() >= cap {
                break; // growth cap hit — stop inheriting new synapses
            }
            child.add_synapse_full(NeuronId(*pre), NeuronId(*post), *w, true, false, Tick(0));
        }
    }
    child
}

pub struct GradientBrain {
    sens: [f32; 4], // per-bearing-channel learned sensitivity gains
}

impl GradientBrain {
    fn new() -> Self {
        Self { sens: [1.0; 4] }
    }
    /// Continuous odor-gradient pursuit: yaw ∝ (right−left), pitch ∝
    /// (up−down), forward thrust. Proximity-brake via the NEAR smell
    /// channel (a real range signal). NOTE: the bearing channels are
    /// distance-INDEPENDENT and ZERO when food is directly ahead, so they
    /// must NOT be treated as lost/close — on-target geometry is quiet,
    /// keep going straight. (The earlier lost-scent spin was a bug: it
    /// veered off exactly when homing straight at food.)
    fn act(&mut self, sm: [f32; 4], near: f32) -> Vec<f32> {
        let [r, l, u, d] = sm;
        let yaw = GRAD_STEER * (r - l) * self.sens[0];
        let pitch = GRAD_STEER * (u - d) * self.sens[2] * 0.5;
        let closing = near >= GRAD_NEAR_THRESH;
        let mut rates = vec![0.0f32; 12];
        rates[0] = if closing { 40.0 } else { GRAD_THRUST_BASE }; // keep closing, gently
        rates[2] = 50.0; // gentle lift
        if closing {
            rates[7] = 60.0; // light brake: slow to land on food, not stall outside it
        }
        if yaw > 0.0 {
            rates[3] = yaw.clamp(0.0, 150.0); // +yaw_rate = toward +X = RIGHT (channel 3)
        } else {
            rates[4] = (-yaw).clamp(0.0, 150.0); // channel 4 = negative yaw_rate = LEFT
        }
        if pitch > 0.0 {
            rates[5] = pitch.clamp(0.0, 150.0);
        } else {
            rates[6] = (-pitch).clamp(0.0, 150.0);
        }
        rates
    }
    /// Food reward sharpens the sensitivity of the channels that were
    /// on: smell→steer association is rewarded (learned chemotaxis).
    fn reward(&mut self, sm: [f32; 4]) {
        for (i, s) in sm.iter().enumerate() {
            if *s > 1.0 {
                self.sens[i] = (self.sens[i] + GRAD_LR * (*s / 40.0)).clamp(0.5, 3.0);
            }
        }
    }
}

// ---------------------------------------------------------------- habit
// D — Place-cell + habit-value brain (hippocampus + striatum).
// A fixed set of PLACE_N prototype vectors (tuned by competitive
// learning toward visited sensory states) forms a self-organized "where
// am I" code; a state→action value table is trained by food-energy TD(λ)
// with eligibility — rewarding STATES and ACTIONS (the correct credit
// local synapses never had), ε-greedy steering.

struct Habit {
    protos: Vec<Vec<f32>>,
    sens_dim: usize,
    v: Vec<f32>,        // [place*H_A + action]
    e: Vec<f32>,        // eligibility trace (same shape)
    eps: f32,
    rng: Xoshiro256PlusPlus,
    prev: Option<(usize, usize)>, // (s, a) taken last beat
}

impl Habit {
    fn new(sens_dim: usize, seed: u64) -> Self {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
        let protos: Vec<Vec<f32>> = (0..PLACE_N)
            .map(|_| (0..sens_dim).map(|_| rng.gen::<f32>()).collect())
            .collect();
        Self {
            protos,
            sens_dim,
            v: vec![0.0; PLACE_N * H_A],
            e: vec![0.0; PLACE_N * H_A],
            eps: H_EPS0,
            rng,
            prev: None,
        }
    }

    /// Locate: nearest place prototype (updated toward the sense).
    fn loc(&mut self, sense: &[f32]) -> usize {
        let mut best = 0;
        let mut bd = f32::MAX;
        for (i, p) in self.protos.iter().enumerate() {
            let d = p.iter().zip(sense.iter()).map(|(a, b)| (a - b) * (a - b)).sum::<f32>();
            if d < bd {
                bd = d;
                best = i;
            }
        }
        // Competitive update: the winning prototype moves toward the sense.
        let p = &mut self.protos[best];
        for (pv, sv) in p.iter_mut().zip(sense.iter()) {
            *pv += H_PHI * (sv - *pv);
        }
        best
    }

    /// Action selection: ε-greedy over the value of the just-located place.
    fn pick(&mut self, s: usize) -> usize {
        if self.rng.gen::<f32>() < self.eps {
            (self.rng.gen::<f32>() * H_A as f32) as usize % H_A
        } else {
            let base = s * H_A;
            (0..H_A).max_by(|&a, &b| self.v[base + a].partial_cmp(&self.v[base + b]).unwrap()).unwrap()
        }
    }

    /// Record the (s, a) just taken (credits next beat's reward).
    fn set_prev(&mut self, s: usize, a: usize) {
        self.prev = Some((s, a));
    }

    /// Food reward → TD(λ): credit the previous (s,a) toward state s_next.
    fn reward(&mut self, r: f32, s_next: usize) {
        if let Some((s, a)) = self.prev {
            let base = s * H_A;
            let nb = s_next * H_A;
            let vmax = (0..H_A).map(|aa| self.v[nb + aa]).fold(f32::MIN, f32::max);
            let td = r + H_GAMMA * vmax - self.v[base + a];
            self.e[base + a] += 1.0;
            for i in 0..self.v.len() {
                self.v[i] += H_ALPHA * td * self.e[i];
                self.e[i] *= H_GAMMA * H_LAMBDA;
            }
            // Bound values (food refill ~ +30).
            for vv in self.v.iter_mut() {
                *vv = vv.clamp(0.0, 120.0);
            }
        }
        // Exploration anneals to a floor (env), so it never fully collapses.
        self.eps = (self.eps - 0.0005).max(eps_floor());
    }
}

/// One observer frame (field names exact, plan step 5b). Built from
/// pure reads AFTER the beat's telemetry; the sim is bit-identical
/// with or without a viewer.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
fn frame_json(
    beat: u64, world: &World, gen: Option<u64>, org: Option<u64>, retina: &[f32],
    prop: &[f32], out: &[f32], min_l: f32, flagged: bool, fault_fired: bool,
    food_dist: f32, food_pos: &Vec3,
) -> String {
    let b = &world.body;
    serde_json::json!({
        "msg": "world2",
        "beat": beat,
        "gen": gen,
        "org": org,
        "alive": b.alive,
        "energy": b.energy,
        "died_at": if b.alive { None::<u64> } else { Some(beat + 1) },
        "pos": [b.pos.x, b.pos.y, b.pos.z],
        "yaw": b.yaw,
        "pitch": b.pitch,
        "vel": [b.vel.x, b.vel.y, b.vel.z],
        "food_dist": food_dist,
        "food": [food_pos.x, food_pos.y, food_pos.z],
        "out": out, // the ACTUAL commanded rates (clamped + noise), not the raw demand
        "retina": retina,
        "prop": prop,
        "minl2": min_l,
        "flag": flagged,
        "fault": fault_fired,
    })
    .to_string()
}

/// Static arena primitives for the viewer (one-shot, stashed and
/// re-sent to every client that joins).
/// The organism's spawn XZ (World::new is deterministic: origin facing +Z).
fn spawn_xz(world_seed: u64) -> (f32, f32) {
    let w = World::new(world_seed);
    (w.body.pos.x, w.body.pos.z)
}

/// Exploration mode (--explore): ONE long live life with the exploration
/// curriculum (food starts close and walks in/out, motor noise, and the
/// intrinsic curiosity reward) — the mode the user watches to see the
/// organism learn to pursue food.
fn run_explore(
    world_seed: u64, net_seed: u64, fault: bool, motor_zero: bool, viewer: Option<&Server>,
    smell: bool,
) -> LifeResult {
    let mut net = build_net(net_inputs(smell), net_seed);
    // Learning-horizon watch length (SL2_BEATS; default 600). Long lives
    // give within-life plasticity time to compound before death.
    let beats = std::env::var("SL2_BEATS").ok().and_then(|v| v.parse().ok()).unwrap_or(BEATS_LONG);
    // SL2_NOCURR=1: run with the curriculum OFF (food at natural seed
    // distance ~28, no close start, no walk) — the generalization test
    // for habit-driven foraging.
    let curriculum = !std::env::var("SL2_NOCURR").is_ok();
    let (r, _) = life_with(&mut net, world_seed, beats, fault, motor_zero, viewer, None, None, true, curriculum, smell, FOOD_START_DIST, None);
    println!(
        "EXPLORE RESULT beats={} died_at={} final_energy={:.1} food_touches={} novel_flags={} ext_news={} coverage={}/{}",
        r.beats,
        r.died_at.map(|d| d.to_string()).unwrap_or_else(|| "none".to_string()),
        r.final_energy,
        r.food_touches,
        r.novel_flags,
        r.ext_news,
        r.coverage,
        COV_N * COV_N
    );
    r
}

/// The ultimate-organism falsifier: train a place+habit brain up under
/// the curriculum (food walks 8->40 within the life), then run the SAME
/// habit on NATURAL food (no close start, no walk). If the trained habit
/// forages naturally, the organism has generalized — the record no local
/// credit mechanism ever reached.
fn run_transfer(
    world_seed: u64, net_seed: u64, fault: bool, motor_zero: bool, viewer: Option<&Server>,
    smell: bool,
) {
    let mut net = build_net(net_inputs(smell), net_seed);
    let train_beats = std::env::var("SL2_TRAIN_BEATS").ok().and_then(|v| v.parse().ok()).unwrap_or(900);
    let nat_beats = std::env::var("SL2_BEATS").ok().and_then(|v| v.parse().ok()).unwrap_or(2000);
    let hb0 = Some(Habit::new(12, io::derive_seed64(world_seed, io::hash_str("habit"), net_seed)));
    // Phase 1: curriculum train (shared habit).
    let (tr, hb) = life_with(&mut net, world_seed, train_beats, fault, motor_zero, viewer, None, None, true, true, smell, FOOD_START_DIST, hb0);
    println!(
        "TRANSFER phase1 (curriculum): beats={} touches={} final_energy={:.1} coverage={}/{}",
        tr.beats, tr.food_touches, tr.final_energy, tr.coverage, COV_N * COV_N
    );
    // Phase 2: natural food, same trained habit.
    let (nr, _) = life_with(&mut net, world_seed, nat_beats, fault, motor_zero, viewer, None, None, true, false, smell, FOOD_START_DIST, hb);
    println!(
        "TRANSFER phase2 (natural): beats={} died_at={} touches={} final_energy={:.1} coverage={}/{}",
        nr.beats,
        nr.died_at.map(|d| d.to_string()).unwrap_or_else(|| "none".to_string()),
        nr.food_touches,
        nr.final_energy,
        nr.coverage,
        COV_N * COV_N
    );
}
/// The net's construction seed, recovered from its own rng state marker:
/// networks are built by `build_net(seed)` and do not store it, so we
/// thread the value through `net.seed`-equivalent — the Network struct
/// exposes `seed` (u64) directly.
fn net_seed_at_start(net: &Network) -> u64 {
    net.seed
}

/// Snake mode: move the food to a fresh random position after a touch.
/// Ground band y in [1.0, 3.0], inside the arena, at least 12 units
/// (2x the 6.0 refill radius) from the organism — one respawn = one
/// meal, no free chain-eating. Deterministic via the caller's rng.
fn respawn_food(world: &mut World, food_idx: usize, rng: &mut Xoshiro256PlusPlus) {
    use anima_world::world::{ARENA_XZ, Vec3};
    let body = world.body.pos;
    let mut x;
    let mut z;
    let mut guard = 0;
    loop {
        x = (rng.gen::<f32>() - 0.5) * 2.0 * (ARENA_XZ - 6.0);
        z = (rng.gen::<f32>() - 0.5) * 2.0 * (ARENA_XZ - 6.0);
        let y = 1.0 + rng.gen::<f32>() * 2.0;
        let dx = x - body.x;
        let dz = z - body.z;
        if dx * dx + dz * dz >= 12.0 * 12.0 || guard > 64 {
            let p = world.primitives.get_mut(food_idx).expect("food exists");
            p.pos = Vec3::new(x, y, z);
            return;
        }
        guard += 1;
    }
}

/// Env rescale (SL2_FOODR): food spawns/respawns within `maxr` of the
/// organism (a "small cage") so distance-to-cover and teleport distance
/// are solvable at the organism's own scale. Used instead of the whole
/// arena whenever the env sets a bounded radius. Default (unset) leaves
/// the world untouched.
fn respawn_near(world: &mut World, food_idx: usize, rng: &mut Xoshiro256PlusPlus, maxr: f32) {
    use anima_world::world::Vec3;
    let body = world.body.pos;
    let ang = rng.gen::<f32>() * std::f32::consts::TAU;
    let dist = 9.0 + rng.gen::<f32>() * (maxr - 9.0); // > eat radius 6
    let x = (body.x + ang.cos() * dist).clamp(-47.0, 47.0);
    let z = (body.z + ang.sin() * dist).clamp(-47.0, 47.0);
    let p = world.primitives.get_mut(food_idx).expect("food exists");
    p.pos = Vec3::new(x, 2.0, z);
}

/// Exploration curriculum respawn: place food a chosen `dist` from the
/// organism along a random heading, clamped inside the arena — the
/// success/failure distance walk (snake mode with a distance knob).
fn respawn_food_at(world: &mut World, food_idx: usize, rng: &mut Xoshiro256PlusPlus, dist: f32) {
    use anima_world::world::ARENA_XZ;
    let body = world.body.pos;
    let ang = rng.gen::<f32>() * std::f32::consts::TAU;
    let x = (body.x + ang.cos() * dist).clamp(-(ARENA_XZ - 3.0), ARENA_XZ - 3.0);
    let z = (body.z + ang.sin() * dist).clamp(-(ARENA_XZ - 3.0), ARENA_XZ - 3.0);
    let p = world.primitives.get_mut(food_idx).expect("food exists");
    p.pos = Vec3::new(x, 2.0, z);
}

/// Food-smell spike trains (--smell). Body-relative odour gradient: 6
/// channels at input ids 56..62 encoding [right, left, up, down, near,
/// far] rates (Hz) from the food's azimuth/elevation/range in the
/// body's heading frame — so "food to the LEFT" lights the left channel
/// and STDP can wire smell->yaw. Same emission convention as the retina
/// (hash-seeded Xoshiro, Poisson gaps, sorted) ⇒ deterministic.
/// Food-smell RATES (Hz), body-relative: [right, left, up, down, near,
/// far] — the direction/range odour gradient. Distance-INVARIANT for the
/// first four (bearing); used by the habit place code so "food ahead"
/// is the same place at 8 or 28 units (value transfer).
fn smell_rates(world: &World, food: Vec3) -> [f32; 6] {
    use std::f32::consts::PI;
    let b = &world.body;
    let d = Vec3::new(food.x - b.pos.x, food.y - b.pos.y, food.z - b.pos.z);
    let dist = (d.x * d.x + d.y * d.y + d.z * d.z).sqrt();
    let dn = if dist > 1e-6 { Vec3::new(d.x / dist, d.y / dist, d.z / dist) } else { Vec3::ZERO };
    let (sy, cy) = b.yaw.sin_cos();
    let (sp, cp) = b.pitch.sin_cos();
    let fwd = Vec3::new(sy * cp, sp, cy * cp);
    let right = Vec3::new(cy, 0.0, -sy);
    let az = (dn.x * right.x + dn.z * right.z).atan2(dn.x * fwd.x + dn.y * fwd.y + dn.z * fwd.z);
    let el = dn.y.asin();
    let half = PI / 2.0;
    let max_r = 40.0f32;
    let right_r = (az / half).clamp(0.0, 1.0) * max_r;
    let left_r = (-az / half).clamp(0.0, 1.0) * max_r;
    let up_r = (el / half).clamp(0.0, 1.0) * max_r;
    let down_r = (-el / half).clamp(0.0, 1.0) * max_r;
    let closeness = (dist / 45.0).clamp(0.0, 1.0);
    let near_r = (1.0 - closeness) * max_r + 2.0;
    let far_r = closeness * max_r + 2.0;
    [right_r, left_r, up_r, down_r, near_r, far_r]
}

fn smell_trains(world: &World, food: Vec3, seed: u64, gain: f32) -> Vec<(u64, InputChannelId)> {
    use std::f32::consts::PI;
    let max_r = 40.0f32;
    let rates = smell_rates(world, food).map(|r| (r * gain).clamp(2.0, max_r)); // hunger boosts food salience
    let base = 56usize; // 48 vision + 8 proprio
    let mut out = Vec::new();
    for (k, r) in rates.iter().enumerate() {
        let ch = (base + k) as u32;
        let ch_seed = io::derive_seed64(seed, io::hash_str("smell"), io::hash_str(&format!("s{k}")) ^ ch as u64);
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(ch_seed);
        let lambda = r.clamp(2.0, max_r) / 1000.0;
        let mut t: u64 = 0;
        loop {
            let u: f32 = rng.gen::<f32>().max(1e-6);
            let gap = ((-(1.0 - u as f64).ln()) / lambda as f64).ceil() as u64;
            t += gap.max(1);
            if t >= io::BEAT_MS {
                break;
            }
            out.push((t, InputChannelId(ch)));
        }
    }
    out.sort_unstable_by_key(|(t, _)| *t);
    out
}

/// Live scene frame for the viewer: current primitives (the food MOVES
/// in snake mode) + optionally the organism spawn XZ. Stashed and
/// re-sent to every client that joins.
fn render_scene_live(world: &World, you: Option<(f32, f32)>) -> String {
    let prims: Vec<serde_json::Value> = world
        .primitives
        .iter()
        .map(|p| {
            serde_json::json!({ "x": p.pos.x, "y": p.pos.y, "z": p.pos.z, "color": p.color, "food": p.food })
        })
        .collect();
    serde_json::json!({ "msg": "scene", "prims": prims, "you": you }).to_string()
}
fn render_scene(world_seed: u64) -> String {
    let world = World::new(world_seed);
    let prims: Vec<serde_json::Value> = world
        .primitives
        .iter()
        .map(|p| {
            serde_json::json!({ "x": p.pos.x, "y": p.pos.y, "z": p.pos.z, "color": p.color, "food": p.food })
        })
        .collect();
    serde_json::json!({ "msg": "scene", "prims": prims }).to_string()
}

/// Watch-only viewer: frames are pushed into a broadcast channel; a
/// server thread pumps them to WS clients in 50ms batches. `send`
/// never blocks the sim (broadcast drops the oldest value / lagging
/// receivers; failures are ignored).
struct Server {
    tx: tokio::sync::broadcast::Sender<String>,
    scene: std::sync::Arc<std::sync::Mutex<Option<String>>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Server {
    fn start(port: u16) -> Self {
        let (tx, _rx) = tokio::sync::broadcast::channel::<String>(256);
        let scene = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));
        let tx2 = tx.clone();
        let scene2 = scene.clone();
        let thread = std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("viewer tokio runtime");
            rt.block_on(serve(port, tx2, scene2));
        });
        Server { tx, scene, thread: Some(thread) }
    }

    /// Block forever on the server thread (serve never returns): keeps
    /// the process alive after the sim ends so viewers can still view
    /// and 4i can kill it. Idempotent.
    fn join(mut self) {
        if let Some(h) = self.thread.take() {
            let _ = h.join();
        }
    }

    /// One world2 frame. Pure side channel; send failures (no viewers)
    /// are ignored.
    fn push(&self, frame: String) {
        let _ = self.tx.send(frame);
    }

    /// One-shot scene frame: stashed for late joiners, then broadcast.
    fn push_scene(&self, scene: String) {
        *self.scene.lock().unwrap() = Some(scene.clone());
        let _ = self.tx.send(scene);
    }
}

async fn serve(port: u16, tx: tokio::sync::broadcast::Sender<String>, scene: std::sync::Arc<std::sync::Mutex<Option<String>>>) {
    use axum::extract::ws::WebSocketUpgrade;
    use axum::extract::State;
    use axum::response::Html;
    use axum::routing::get;
    use axum::Router;

    async fn index() -> Html<&'static str> {
        Html(INDEX_HTML)
    }

    async fn ws_handler(
        ws: WebSocketUpgrade,
        State((tx, scene)): State<(tokio::sync::broadcast::Sender<String>, std::sync::Arc<std::sync::Mutex<Option<String>>>)>,
    ) -> impl axum::response::IntoResponse {
        ws.on_upgrade(move |socket| client_loop(socket, tx, scene))
    }

    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(ws_handler))
        .with_state((tx, scene));

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("world_survival bind {addr}: {e}"));
    axum::serve(listener, app).await.unwrap();
}

async fn client_loop(
    socket: axum::extract::ws::WebSocket,
    tx: tokio::sync::broadcast::Sender<String>,
    scene: std::sync::Arc<std::sync::Mutex<Option<String>>>,
) {
    use axum::extract::ws::Message;
    use futures_util::{SinkExt, StreamExt};
    let (mut sender, mut receiver) = socket.split();
    // Late joiners get the one-shot scene frame first (clone the guard
    // value before the await — the lock must not cross it).
    let scene_frame = scene.lock().unwrap().clone();
    if let Some(s) = scene_frame {
        if sender.send(Message::Text(s)).await.is_err() {
            return;
        }
    }
    // Batched pump: every 50 ms drain ALL pending frames into one send.
    // A blocked TCP sink can stall a per-frame pump forever; the
    // interval guarantees the loop keeps waking and drops stale frames
    // instead of queueing unbounded (anima-viz lesson, measured fix).
    let mut rx = tx.subscribe();
    let mut tick = tokio::time::interval(std::time::Duration::from_millis(50));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = tick.tick() => {
                let mut batch = String::new();
                let mut n = 0;
                while let Ok(text) = rx.try_recv() {
                    batch.push_str(&text);
                    batch.push('\n');
                    n += 1;
                    if n >= 256 { break; } // bound per-flush work
                }
                if !batch.is_empty() {
                    let frame = serde_json::json!({ "msg": "batch", "lines": batch }).to_string();
                    if sender.send(Message::Text(frame)).await.is_err() {
                        break;
                    }
                }
            }
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(_)) => {} // watch-only: client input ignored
                    Some(Err(_)) | None => break,
                }
            }
        }
    }
}

const INDEX_HTML: &str = r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>world2 - live organism</title>
<style>
  :root { color-scheme: dark; }
  * { box-sizing: border-box; }
  body { margin:0; background:#0c1116; color:#dfe7ee; font:13px/1.4 ui-monospace,Menlo,Consolas,monospace; overflow:hidden; }
  #app { display:flex; height:100vh; }
  #stage { position:relative; flex:1 1 auto; min-width:0; }
  #arena { position:absolute; inset:0; width:100%; height:100%; display:block; cursor:grab; background:#080c10; touch-action:none; }
  #arena.drag { cursor:grabbing; }
  #hud { position:absolute; top:10px; left:10px; right:10px; display:flex; gap:8px; pointer-events:none; flex-wrap:wrap; }
  .chip { background:rgba(13,20,27,.82); border:1px solid #26333f; border-radius:6px; padding:5px 10px; font-size:12px; white-space:nowrap; }
  .chip b { color:#7fd1ff; font-weight:600; }
  .lamp { display:inline-block; width:10px; height:10px; border-radius:50%; background:#333; margin-right:5px; vertical-align:-1px; }
  .lamp.on { background:#3ddc84; box-shadow:0 0 8px #3ddc84; }
  .lamp.warn { background:#ffb020; box-shadow:0 0 8px #ffb020; }
  #energyWrap { display:inline-block; width:110px; height:12px; background:#0b0f13; border:1px solid #2a3542; border-radius:3px; overflow:hidden; vertical-align:-1px; }
  #energyBar { height:100%; width:0; background:linear-gradient(90deg,#2fae6a,#3ddc84); }
  #right { flex:0 0 320px; display:flex; flex-direction:column; gap:10px; padding:10px; background:#10161c; border-left:1px solid #1d2833; min-height:0; }
  .panel { background:#161e26; border:1px solid #26333f; border-radius:8px; padding:10px; }
  h2 { margin:0 0 8px; font-size:11px; letter-spacing:1px; color:#8fa3b8; text-transform:uppercase; }
  canvas.cell { display:block; background:#080c10; border:1px solid #22303b; border-radius:6px; }
  #retina { width:100%; height:auto; }
  #log { color:#9fb3c8; height:6.3em; overflow:hidden; font-size:12px; }
  #log div { white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
  #hint { color:#5c7186; font-size:11px; }
</style>
</head>
<body>
<div id="app">
  <div id="stage">
    <canvas id="arena"></canvas>
    <div id="hud">
      <span class="chip"><span class="lamp" id="aliveLamp"></span><b id="beat">0</b><span id="genTag"></span></span>
      <span class="chip">energy <span id="energyWrap"><span id="energyBar"></span></span> <b id="energyVal">0</b></span>
      <span class="chip"><span class="lamp" id="novLamp"></span>new <span id="minl2"></span></span>
      <span class="chip"><span class="lamp" id="faultLamp"></span>fault</span>
      <span class="chip">food <b id="foodDist">-</b></span>
      <span class="chip">zoom <b id="zoomVal">x1</b></span>
    </div>
  </div>
  <div id="right">
    <div class="panel">
      <h2>Retina (8x6) — what it sees</h2>
      <canvas id="retina" class="cell" width="560" height="420"></canvas>
    </div>
    <div class="panel">
      <h2>Body sense (8) + motors (12)</h2>
      <canvas id="bars" class="cell" width="560" height="140"></canvas>
    </div>
    <div class="panel" style="flex:1 1 auto; min-height:0; overflow:hidden;">
      <h2>Log</h2>
      <div id="log"></div>
    </div>
    <div class="panel"><div id="hint">drag = pan &middot; wheel = zoom &middot; double-click = follow organism</div></div>
  </div>
</div>
<script>
"use strict";
var S = { beat:0, alive:false, energy:0, pos:[0,1.5,0], yaw:0, pitch:0, vel:[0,0,0],
          foodDist:-1, out:[], retina:[], prop:[], minl2:0, flag:false, fault:false,
          prims:[], gen:null, org:null };
function $(id){ return document.getElementById(id); }
var arena = $("arena"), actx = arena.getContext("2d");
var retina = $("retina"), rctx = retina.getContext("2d");
var bars = $("bars"), bctx = bars.getContext("2d");

// --- view state: pan (world units), zoom (px per world unit), follow ---
var view = { cx:0, cz:10, pxPerUnit:10, follow:true, minPx:1.4, maxPx:26 };
function fitArena(){
  var r = arena.getBoundingClientRect();
  var dpr = window.devicePixelRatio || 1;
  if (arena.width !== Math.round(r.width*dpr) || arena.height !== Math.round(r.height*dpr)){
    arena.width = Math.round(r.width*dpr); arena.height = Math.round(r.height*dpr);
  }
  view.minPx = Math.max(1.4, Math.min(arena.width, arena.height) / 160); // whole 100x100 arena fits
  if (view.pxPerUnit < view.minPx) view.pxPerUnit = view.minPx;
  if (view.pxPerUnit > view.maxPx) view.pxPerUnit = view.maxPx;
  view.cx = Math.max(-60, Math.min(60, view.cx));
  view.cz = Math.max(-60, Math.min(60, view.cz));
}

function drawArena(){
  fitArena();
  var W = arena.width, H = arena.height;
  actx.clearRect(0,0,W,H);
  var px = view.pxPerUnit;
  var X = function(wx){ return W/2 + (wx - view.cx)*px; };
  var Y = function(wz){ return H/2 + (wz - view.cz)*px; };
  // ground grid every 10 units + fine grid every 2 when zoomed in
  var steps = px > 7 ? [2,10] : [10];
  for (var si = 0; si < steps.length; si++){
    var step = steps[si];
    actx.strokeStyle = step === 10 ? "#1a2530" : "#121a22";
    actx.lineWidth = Math.max(1, window.devicePixelRatio||1);
    var startW = Math.floor((view.cx - (W/2)/px)/step)*step;
    var endW = view.cx + (W/2)/px;
    for (var gx = startW; gx <= endW; gx += step){
      actx.beginPath(); actx.moveTo(X(gx), 0); actx.lineTo(X(gx), H); actx.stroke();
    }
    var startH = Math.floor((view.cz - (H/2)/px)/step)*step;
    var endH = view.cz + (H/2)/px;
    for (var gz = startH; gz <= endH; gz += step){
      actx.beginPath(); actx.moveTo(0, Y(gz)); actx.lineTo(W, Y(gz)); actx.stroke();
    }
  }
  // arena bounds
  actx.strokeStyle = "#31465c"; actx.lineWidth = 2;
  actx.strokeRect(X(-50), Y(-50), 100*px, 100*px);
  // food-eat radius hint around the organism
  var bx = X(S.pos[0]), bz = Y(S.pos[2]);
  if (S.alive && S.foodDist >= 0 && S.foodDist < 6){
    actx.beginPath(); actx.arc(bx, bz, 6*px, 0, Math.PI*2);
    actx.strokeStyle = "rgba(61,220,132,0.55)"; actx.setLineDash([4,4]); actx.stroke(); actx.setLineDash([]);
  }
  // primitives
  S.prims.forEach(function(p){
    var x = X(p.x), z = Y(p.z);
    if (x < -40 || x > W+40 || z < -40 || z > H+40) return;
    actx.beginPath();
    var rad = (p.food ? 7 : 4.5) * Math.max(1, px/10);
    actx.arc(x, z, rad, 0, Math.PI*2);
    if (p.food){
      actx.strokeStyle = "rgb(" + p.color.map(function(c){ return Math.round(c*255); }).join(",") + ")";
      actx.lineWidth = 3; actx.stroke();
      // pulsing glow so food is easy to find
      var pulse = 0.5 + 0.5*Math.sin(performance.now()/300);
      actx.beginPath(); actx.arc(x, z, rad + 4 + pulse*3, 0, Math.PI*2);
      actx.strokeStyle = "rgba(61,220,132," + (0.25 + 0.2*pulse).toFixed(3) + ")";
      actx.lineWidth = 2; actx.stroke();
    } else {
      actx.fillStyle = "rgb(" + p.color.map(function(c){ return Math.round(c*255); }).join(",") + ")";
      actx.fill();
    }
  });
  // body trail (last 600 beats)
  if (trail.length > 1){
    actx.beginPath();
    actx.moveTo(X(trail[0][0]), Y(trail[0][1]));
    for (var ti = 1; ti < trail.length; ti++) actx.lineTo(X(trail[ti][0]), Y(trail[ti][1]));
    actx.strokeStyle = "rgba(127,209,255,0.35)"; actx.lineWidth = 2; actx.stroke();
  }
  // camera cone (azimuth half-FOV 0.8 rad), length capped at 30 units
  var heading = Math.atan2(Math.cos(S.yaw), Math.sin(S.yaw)); // screen: x right, z down
  var L = 30*px;
  actx.beginPath();
  actx.moveTo(bx, bz);
  actx.lineTo(bx + Math.cos(heading-0.8)*L, bz + Math.sin(heading-0.8)*L);
  actx.lineTo(bx + Math.cos(heading+0.8)*L, bz + Math.sin(heading+0.8)*L);
  actx.closePath();
  actx.fillStyle = "rgba(120,180,255,0.08)"; actx.fill();
  actx.strokeStyle = "rgba(120,180,255,0.3)"; actx.lineWidth = 1.5; actx.stroke();
  // organism: glow + arrow
  actx.beginPath(); actx.arc(bx, bz, 14 + 4*Math.sin(performance.now()/250), 0, Math.PI*2);
  actx.fillStyle = "rgba(127,209,255,0.15)"; actx.fill();
  var a = Math.max(14, 1.6*px);
  actx.beginPath();
  actx.moveTo(bx + Math.cos(heading)*a, bz + Math.sin(heading)*a);
  actx.lineTo(bx - Math.cos(heading)*a*0.7, bz - Math.sin(heading)*a*0.7);
  actx.strokeStyle = S.alive ? "#7fd1ff" : "#5c7186"; actx.lineWidth = Math.max(2.5, px/4); actx.stroke();
  actx.beginPath();
  actx.moveTo(bx + Math.cos(heading)*a, bz + Math.sin(heading)*a);
  actx.lineTo(bx + Math.cos(heading+2.5)*a*0.7, bz + Math.sin(heading+2.5)*a*0.7);
  actx.moveTo(bx + Math.cos(heading)*a, bz + Math.sin(heading)*a);
  actx.lineTo(bx + Math.cos(heading-2.5)*a*0.7, bz + Math.sin(heading-2.5)*a*0.7);
  actx.stroke();
}

var trail = [];

function drawRetina(){
  var W = retina.width, H = retina.height; // 560x420 buffer, 8x6 cells
  rctx.clearRect(0,0,W,H);
  if (S.retina.length === 48){
    var cw = W/8, ch = H/6;
    for (var i = 0; i < 48; i++){
      var col = i % 8, row = (i/8)|0;
      var l = Math.max(0, Math.min(1, S.retina[i]));
      var v = Math.round(l*255);
      // slight blue tint so "what it sees" looks organic
      rctx.fillStyle = "rgb(" + Math.round(v*0.92) + "," + v + "," + Math.round(v*0.95 + 10) + ")";
      rctx.fillRect(col*cw, row*ch, cw-3, ch-3);
    }
  } else {
    rctx.fillStyle = "#2a3542"; rctx.font = "20px monospace"; rctx.fillText("no frame", 16, 32);
  }
}

function drawBars(){
  var W = bars.width, H = bars.height; // 560x140
  bctx.clearRect(0,0,W,H);
  bctx.font = "12px monospace";
  // 8 proprio bars then 12 motor bars, one row of 20 slots
  var n = 20, cw = W/n;
  for (var i = 0; i < n; i++){
    var isProp = i < 8;
    var v = isProp ? (S.prop[i]||0)/40 : (S.out[i-8]||0)/150;
    v = Math.max(0, Math.min(1, v));
    var bh = 90*v;
    bctx.fillStyle = "#16202a"; bctx.fillRect(i*cw+3, 110-90, cw-6, 90);
    bctx.fillStyle = isProp ? "#4aa8ff" : "#c98bff";
    bctx.fillRect(i*cw+3, 110-bh, cw-6, bh);
    bctx.fillStyle = "#5c7186";
    bctx.fillText(isProp ? ("p"+i) : ("m"+(i-8)), i*cw+5, 128);
  }
}

function drawMeters(){
  $("energyBar").style.width = Math.max(0, Math.min(1, (S.energy||0)/150))*100 + "%";
  $("energyVal").textContent = Math.round(S.energy||0);
  $("aliveLamp").className = "lamp" + (S.alive ? " on" : "");
  $("novLamp").className = "lamp" + (S.flag ? " warn" : "");
  $("faultLamp").className = "lamp" + (S.fault ? " warn" : "");
  $("minl2").textContent = S.minl2 > 1e30 ? "inf" : Number(S.minl2).toFixed(0);
  $("foodDist").textContent = S.foodDist < 0 ? "-" : S.foodDist.toFixed(1);
  $("beat").textContent = String(S.beat) + (S.alive ? "" : " RIP");
  $("genTag").textContent = (S.gen !== null && S.gen !== undefined && S.org !== null && S.org !== undefined) ? (" g" + S.gen + "o" + S.org) : "";
  $("zoomVal").textContent = "x" + (view.pxPerUnit/10).toFixed(1);
}

var logEl = $("log");
function logLine(msg){
  var d = document.createElement("div");
  d.textContent = msg;
  logEl.appendChild(d);
  while (logEl.childNodes.length > 6) logEl.removeChild(logEl.firstChild);
}

function apply(f){
  S.beat = f.beat; S.alive = !!f.alive; S.energy = f.energy;
  S.pos = f.pos; S.yaw = f.yaw; S.pitch = f.pitch; S.vel = f.vel;
  S.foodDist = f.food_dist; S.out = f.out; S.retina = f.retina; S.prop = f.prop;
  S.minl2 = f.minl2; S.flag = !!f.flag; S.fault = !!f.fault;
  S.gen = f.gen; S.org = f.org;
  if (f.msg && f.msg === "scene") { S.prims = f.prims || []; }
  trail.push([f.pos[0], f.pos[2]]);
  if (trail.length > 600) trail.shift();
  var tag = (f.gen !== null && f.gen !== undefined && f.org !== null && f.org !== undefined)
    ? " gen " + f.gen + " org " + f.org : "";
  logLine("beat " + f.beat + tag + " energy " + Math.round(f.energy*10)/10 +
    " d(food) " + (f.food_dist >= 0 ? f.food_dist.toFixed(1) : "-") +
    (f.flag ? " NEW!" : "") + (f.fault ? " FAULT!" : ""));
}

function handle(line){
  var f = null;
  try { f = JSON.parse(line); } catch (e) { return; }
  if (f && f.msg === "world2") apply(f);
}

var ws = null;
function connect(){
  ws = new WebSocket((location.protocol === "https:" ? "wss://" : "ws://") + location.host + "/ws");
  ws.onmessage = function(ev){
    var f = null;
    try { f = JSON.parse(ev.data); } catch (e) { return; }
    if (!f) return;
    if (f.msg === "scene") { S.prims = f.prims || []; if (f.you) { view.cx = f.you[0]; view.cz = f.you[1]; } }
    else if (f.msg === "batch") {
      var lines = Array.isArray(f.lines) ? f.lines : String(f.lines).split("\n");
      for (var i = 0; i < lines.length; i++){ if (lines[i]) handle(lines[i]); }
    } else if (f.msg === "world2") { apply(f); }
  };
  ws.onclose = function(){ setTimeout(connect, 1000); };
}

// --- pan / zoom / follow (pointer events; CSS px, NOT device px) ---
var drag = null;
arena.addEventListener("pointerdown", function(e){
  drag = { x: e.clientX, y: e.clientY, cx: view.cx, cz: view.cz };
  view.follow = false;
  arena.setPointerCapture(e.pointerId);
  arena.classList.add("drag");
});
arena.addEventListener("pointermove", function(e){
  if (!drag) return;
  var rect = arena.getBoundingClientRect();
  view.cx = drag.cx - (e.clientX - drag.x)/view.pxPerUnit;
  view.cz = drag.cz - (e.clientY - drag.y)/view.pxPerUnit;
});
function endDrag(e){
  if (!drag) return;
  drag = null;
  arena.classList.remove("drag");
  if (e && e.pointerId !== undefined && arena.hasPointerCapture && arena.hasPointerCapture(e.pointerId)) arena.releasePointerCapture(e.pointerId);
}
arena.addEventListener("pointerup", endDrag);
arena.addEventListener("pointercancel", endDrag);
arena.addEventListener("dblclick", function(){ view.follow = true; });
arena.addEventListener("wheel", function(e){
  e.preventDefault();
  var rect = arena.getBoundingClientRect();
  var mx = e.clientX - rect.left, my = e.clientY - rect.top;
  // zoom around the mouse point (CSS px math; clamp inside fitArena)
  var old = view.pxPerUnit;
  var f2 = Math.exp(-e.deltaY * 0.0012);
  view.pxPerUnit = Math.max(view.minPx, Math.min(view.maxPx, old * f2));
  var k = view.pxPerUnit/old;
  view.cx += (mx - arena.getBoundingClientRect().width/2)*(1 - 1/k)/view.pxPerUnit;
  view.cz += (my - arena.getBoundingClientRect().height/2)*(1 - 1/k)/view.pxPerUnit;
}, { passive:false });

// follow-cam: keep the organism centered (default mode)
function followCam(){
  if (view.follow){ view.cx = S.pos[0]; view.cz = S.pos[2]; }
}

// 60fps paint loop; a swallowed exception must not kill rendering
// (re-armed by a 250ms watchdog when stale > 1s).
var raf = 0, lastPaint = 0, running = false;
function frame(){
  running = true;
  try { followCam(); drawArena(); drawRetina(); drawBars(); drawMeters(); lastPaint = performance.now(); }
  catch (e) { /* keep the loop alive */ }
  raf = requestAnimationFrame(frame);
}
function arm(){ if (!running) raf = requestAnimationFrame(frame); }
setInterval(function(){
  if (!running) arm();
  else if (performance.now() - lastPaint > 1000){
    cancelAnimationFrame(raf); running = false; arm();
  }
}, 250);
window.addEventListener("resize", fitArena);

connect();
arm();
</script>
</body>
</html>
"##;