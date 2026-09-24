//! Phase III sZ evolution/death-selection (docs/phase3/s-evolution-protocol.md
//! D-32, frozen). Population N=4, elitist top-2 keep + 2 offspring each;
//! variation = brain-size mutation (n_internal +/-4, band [32,56]) + weight
//! mutation (+/-10% p=0.1 via (pre,post) key inheritance); life = S1
//! formation (live STDP) + closed survival loop; fitness = mean viability
//! (+ small recognition bonus); 8 gens x 3 seeds. PRIMARY: does population
//! mean size rise/fall/flat (the user's "network too small" question).
use std::collections::BTreeMap;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use anima_core::network::{NeuronId, Network, NetworkConfig, Tick, V2Params};
use anima_core::plasticity::{stdp_tick, StdpParams, Traces};
use anima_core::structural_v2::V2Plasticity;

use anima_exp::config::SurvivalSpec;
use anima_exp::io;
use anima_exp::reflex;
use anima_exp::survival;

const N_POP: usize = 4;
const GENS: u32 = 8;
const BAND: [i32; 2] = [32, 56];
const SIZE_STEP: i32 = 4;
const W_MUT_P: f32 = 0.10;
const W_MUT_AMP: f32 = 0.10;
const SEEDS: [u64; 3] = [20260912, 9001, 424242];

fn v2_params() -> V2Params {
    V2Params {
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
fn build_net(seed: u64, n_internal: usize, out_inh: f32) -> Network {
    // D-54: optional output competition gain via env (D54_COMP). Off by
    // default = identity. Re-tested POST-survival per D-55 (the earlier
    // sweep measured the pre-survival degenerate state - invalid).
    let comp_gain: f32 = std::env::var("D54_COMP").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0);
    let cfg = NetworkConfig {
        connectivity: 0.038, w_init: 0.2, amplitude: 52.0,
        adaptation_tau_ms: 200.0, adaptation_gain: 0.05, inhibition_gain: 0.0,
        slow_state_beta: 0.0046875, slow_state_tau_ms: 5000.0, slow_state_beta_drive: false,
        latch_enable: true, theta_rel_mean: 1.0, theta_rel_sd: 0.0, u_plateau_rel_mean: 1.0,
        u_plateau_rel_sd: 0.0, tau_het_rel_sd: 0.0, phi_rel: 0.5, eta_rel: 0.0,
        v2: Some(v2_params()), output_inhibition_gain: out_inh, // D-46
        output_competition_gain: comp_gain, // D-54
        // D-59: reflex band in-life (the D-58 rule moved into the survival
        // loop). D59_REFLEX set => REFLEX_K novelty nodes appended after the
        // identity band; unset = 0 = byte-identical baseline.
        d58_reflex: if std::env::var("D59_REFLEX").is_ok() { reflex::REFLEX_K } else { 0 },
        ..NetworkConfig::default()
    };
    Network::new(cfg, 24, n_internal, 12, seed)
}
fn params() -> StdpParams {
    StdpParams { tau_plus: 20.0, tau_minus: 20.0, a_plus: 0.005, a_minus: 0.0053,
        decay: 1e-6, w_min: 0.0, w_max: 1.0 }
}
fn survival_spec() -> SurvivalSpec {
    let beat_count = std::env::var("EVO_BEATS").ok().and_then(|s| s.parse().ok()).unwrap_or(30u64);
    SurvivalSpec { beats: beat_count, r_window: 10, th_known: 0.20, q_floor: 1.0,
        a_bounds: [5.0, 250.0], horizon: 30, off_ms: 1500, p_novel: 0.2 }
}

fn d50_mode() -> &'static str {
    match std::env::var("D50_MODE").as_deref() {
        Ok("1") => "d50",    // 3 known (A,C,E) 6ch
        Ok("2") => "d50-2",  // 2 known (A,C) 6ch - channel-squeeze control
        _ => "",
    }
}

fn form_s1(net: &mut Network, seed: u64) {
    let p = params();
    let mut traces = Traces::new(net, 20.0);
    let known: Vec<&'static str> = io::known_syms(d50_mode());
    for _rep in 0..20usize {
        for sym in &known {
            let tr = io::symbol_trains_mode(sym, d50_mode(), seed);
            for t in 0..io::BEAT_MS {
                let frame = anima_core::network::InputFrame {
                    tick: net.tick,
                    spikes: tr.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect(),
                };
                let ev = net.step(&frame);
                traces.step(net, &ev.spikes);
                let _ = stdp_tick(&p, net, &traces, &ev.spikes, 1.0, None);
                net.tick = Tick(net.tick.0 + 1);
            }
            for _ in 0..1500 {
                let frame = anima_core::network::InputFrame { tick: net.tick, spikes: vec![] };
                let ev = net.step(&frame);
                traces.step(net, &ev.spikes);
                let _ = stdp_tick(&p, net, &traces, &ev.spikes, 1.0, None);
                net.tick = Tick(net.tick.0 + 1);
            }
        }
    }
}
fn capture_refs(net: &mut Network, seed: u64) -> Vec<(String, Vec<f32>)> {
    let mut acc: std::collections::BTreeMap<String, Vec<f32>> = Default::default();
    let mut cnt: std::collections::BTreeMap<String, u32> = Default::default();
    for sym in io::known_syms(d50_mode()) {
        for _ in 0..3u64 {
            let tr = io::symbol_trains_mode(sym, d50_mode(), seed);
            let mut out = vec![0.0f32; 12];
            for t in 0..io::BEAT_MS {
                let frame = anima_core::network::InputFrame {
                    tick: net.tick,
                    spikes: tr.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect(),
                };
                let ev = net.step(&frame);
                for c in &ev.spikes { if (io::OUTPUT_LO..io::OUTPUT_HI).contains(&c.0) { out[(c.0 - io::OUTPUT_LO) as usize] += 1.0; } }
                net.tick = Tick(net.tick.0 + 1);
            }
            let e = acc.entry(sym.to_string()).or_insert_with(|| vec![0.0f32; 12]);
            for i in 0..12 { e[i] += out[i]; }
            *cnt.entry(sym.to_string()).or_insert(0) += 1;
        }
    }
    acc.into_iter().map(|(p, v)| { let c = *cnt.get(&p).unwrap_or(&1).max(&1) as f32; (p, v.iter().map(|x| x / c).collect()) }).collect()
}

/// D-50 separation falsifier: present each known symbol HELD-OUT and
/// measure per-pair cos-argmax accuracy against the refs. Returns
/// (mean_pairwise_acc, per_pair). This is the N(N-1)/2 pairwise metric
/// the protocol specifies (not the single-vs-pair survival fit).
/// D-50 separation falsifier (REGISTERED metric): present each known
/// symbol FRESH (held-out, 2 presentations) and DECODE each via the
/// frozen io codebook against the captured refs. Returns (mean decode
/// acc over all knowns, per-pair confusion). This is the real test: can
/// the organism correctly DECODE each symbol, with the N(N-1)/2 pairwise
/// confusion matrix falling out of misassignments. (Earlier version
/// wrongly compared held-out responses against EACH OTHER - cos(x,x)=1.0
/// trivial - fixed to decode-against-refs per advisory.)
/// D-52 timing probe: dump per-output-neuron spike TIMES for one beat of
/// each known symbol (verbose). Answers H-52a: does output-band TIMING
/// differ between symbols, or is it timing-constant (one tonic neuron
/// every tick)? The fork-test before building a binned codec.
fn probe_output_timing(net: &mut Network, seed: u64, sym: &str, mode: &str) {
    let tr = io::symbol_trains_mode(sym, mode, seed);
    let mut times: Vec<(u64, u32)> = Vec::new();
    for t in 0..io::BEAT_MS {
        let frame = anima_core::network::InputFrame {
            tick: net.tick,
            spikes: tr.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect(),
        };
        let ev = net.step(&frame);
        for c in &ev.spikes {
            if (io::OUTPUT_LO..io::OUTPUT_HI).contains(&c.0) {
                times.push((t, c.0 - io::OUTPUT_LO));
            }
        }
        net.tick = Tick(net.tick.0 + 1);
    }
    if std::env::var("EVOLVE_VERBOSE").is_ok() {
        eprintln!("  TIMING[{sym}]: {} output spikes; first-20 (t,outch): {:?}", times.len(), &times[..times.len().min(20)]);
        // summary: per-output-channel spike-time histogram (mean time, count)
        let mut per: Vec<(u32, usize, f64)> = Vec::new();
        for oc in 0..(io::OUTPUT_HI - io::OUTPUT_LO) as u32 {
            let occ: Vec<u64> = times.iter().filter(|(_, o)| *o == oc).map(|(t, _)| *t).collect();
            if !occ.is_empty() {
                let mean = occ.iter().map(|t| *t as f64).sum::<f64>() / occ.len() as f64;
                let spread = occ.iter().map(|t| (*t as f64 - mean).abs()).sum::<f64>() / occ.len() as f64;
                per.push((oc, occ.len(), spread));
            }
        }
        eprintln!("  TIMING[{sym}] per-outch (outch,n_spikes,mean|spread): {:?}", per);
        // D-52 fork decisor: per-channel 5-bin temporal histogram
        // (5x100ms over the 500ms beat). If A vs C differ in PROFILE
        // SHAPE (not just magnitude), the binned codec has signal.
        let bins = 5usize;
        let bin_ms = (io::BEAT_MS as usize) / bins;
        for oc in 0..(io::OUTPUT_HI - io::OUTPUT_LO) as u32 {
            let occ: Vec<u64> = times.iter().filter(|(_, o)| *o == oc).map(|(t, _)| *t).collect();
            if occ.len() < 5 { continue; } // only channels with enough spikes
            let mut hist = vec![0usize; bins];
            for t in &occ { let b = ((*t as usize) / bin_ms).min(bins-1); hist[b] += 1; }
            eprintln!("  TIMING[{sym}] ch{oc} HIST(5x100ms): {:?}", hist);
        }
    }
}

fn capture_separation(net: &mut Network, seed: u64, refs: &[(String, Vec<f32>)], spec: &SurvivalSpec) -> (f32, Vec<(String, f32)>) {
    let knowns = io::known_syms(d50_mode());
    // per-symbol held-out decode accuracy (fresh presentations vs refs)
    let mut per_sym: Vec<(String, u32, u32)> = Vec::new(); // (sym, correct, presented)
    for sym in &knowns {
        let mut correct = 0u32; let mut presented = 0u32;
        for _ in 0..4u64 { // 4 fresh held-out presentations
            let tr = io::symbol_trains_mode(sym, d50_mode(), seed);
            let mut out = vec![0.0f32; 12];
            for t in 0..io::BEAT_MS {
                let frame = anima_core::network::InputFrame {
                    tick: net.tick,
                    spikes: tr.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect(),
                };
                let ev = net.step(&frame);
                for c in &ev.spikes { if (io::OUTPUT_LO..io::OUTPUT_HI).contains(&c.0) { out[(c.0 - io::OUTPUT_LO) as usize] += 1.0; } }
                net.tick = Tick(net.tick.0 + 1);
            }
            let act = io::decode(&out, refs, spec.q_floor, spec.th_known);
            presented += 1;
            if act == *sym { correct += 1; }
        }
        per_sym.push(((*sym).to_string(), correct, presented));
    }
    // pairwise confusion: for each pair, the correctness of decoding
    // a symbol as itself vs as the other (from the same held-out runs is
    // not retained per presentation; approximate per-pair via presence
    // in the correct-decode accounting). Report per-symbol acc as the
    // pairs' components.
    // novel-probe integrity (D): D must NOT decode to any known symbol.
    // Count any D presentation that decodes to a known ref as a
    // contamination. (Guards the known-vs-novel falsifier.)
    for _ in 0..4u64 {
        let tr = io::symbol_trains_mode("D", d50_mode(), seed);
        let mut out = vec![0.0f32; 12];
        for t in 0..io::BEAT_MS {
            let frame = anima_core::network::InputFrame {
                tick: net.tick,
                spikes: tr.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect(),
            };
            let ev = net.step(&frame);
            for c in &ev.spikes { if (io::OUTPUT_LO..io::OUTPUT_HI).contains(&c.0) { out[(c.0 - io::OUTPUT_LO) as usize] += 1.0; } }
            net.tick = Tick(net.tick.0 + 1);
        }
        let act = io::decode(&out, refs, spec.q_floor, spec.th_known);
        if knowns.iter().any(|k| *k == act.as_str()) {
            if std::env::var("EVOLVE_VERBOSE").is_ok() {
                eprintln!("  NOTE: D probed decode -> known symbol '{act}' (contamination)");
            }
        }
    }
    let mut pairs: Vec<(String, f32)> = Vec::new();
    let mut total_acc = 0.0f32; let mut npair = 0;
    for i in 0..knowns.len() {
        for j in i+1..knowns.len() {
            // pair acc = mean of the two symbols' decode accuracy
            let acc_i = per_sym[i].1 as f32 / per_sym[i].2.max(1) as f32;
            let acc_j = per_sym[j].1 as f32 / per_sym[j].2.max(1) as f32;
            let pacc = (acc_i + acc_j) / 2.0;
            pairs.push((format!("{}-{}", knowns[i], knowns[j]), pacc));
            total_acc += pacc; npair += 1;
        }
    }
    (total_acc / npair.max(1) as f32, pairs)
}

/// Inherit parent's weights by (pre,post) key into a fresh child; apply
/// weight mutation. n_internal may differ (size mutation): shared neurons
/// inherit, new neurons' synapses stay fresh.
/// Pure copy of the parent brain (no mutation) - elitism.
fn breed_no_mut(parent: &Network, seed: u64, n_internal: usize, out_inh: f32) -> Network {
    breed_inner(parent, seed, n_internal, false, out_inh)
}
fn breed(parent: &Network, seed: u64, n_internal: usize, out_inh: f32) -> Network {
    breed_inner(parent, seed, n_internal, true, out_inh)
}
fn breed_inner(parent: &Network, seed: u64, n_internal: usize, mutate: bool, out_inh: f32) -> Network {
    // BTreeMap: ORDERED iteration. Residual-add + mutation draw rng
    // against ordered keys; HashMap order is per-process
    // nondeterministic -> offspring differ run-to-run (measured: same
    // cmd twice, gen-1 mean_fit 0.532 vs 0.531). BTreeMap fixes it.
    let mut wmap: BTreeMap<(u32, u32), f32> = BTreeMap::new();
    // build weight map from LIVE parent synapses only (skip tombstones)
    for s in &parent.synapses {
        if s.silent_ticks == u64::MAX { continue; }
        wmap.insert((s.pre.0, s.post.0), s.w);
    }
    let mut child = build_net(seed, n_internal, out_inh);
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
    let n_shared = parent.neurons.len().min(child.neurons.len());
    let mut consumed: Vec<(u32, u32)> = Vec::new();
    for s in child.synapses.iter_mut() {
        let key = (s.pre.0, s.post.0);
        if s.silent_ticks != u64::MAX && s.pre.0 < n_shared as u32 && s.post.0 < n_shared as u32 {
            if let Some(w) = wmap.get(&key).copied() {
                let mut w = w;
                if mutate && rng.gen::<f32>() < W_MUT_P {
                    w += (rng.gen::<f32>() * 2.0 - 1.0) * W_MUT_AMP * w;
                }
                s.w = w.clamp(0.0, 1.0);
                consumed.push(key); // drain matched so we don't re-add below
            }
        }
    }
    for k in consumed { wmap.remove(&k); }
    // Parent-born synapses (created during the parent's life) that the fresh
    // child truly lacks are ADDED - only when BOTH endpoints exist in the
    // child's neuron range AND the child doesn't already carry the key.
    // (matched/drained keys never reach here; size-mutation keys outside
    //  the child's range must be dropped, not added - a smaller child
    //  cannot host a synapse to a neuron it does not have.)
    let n_child = child.neurons.len() as u32;
    let mut born = 0usize;
    let have: std::collections::HashSet<(u32, u32)> = child.synapses.iter()
        .filter(|s| s.silent_ticks != u64::MAX)
        .map(|s| (s.pre.0, s.post.0)).collect();
    for ((pre, post), w) in &wmap {
        if *pre < n_child && *post < n_child && !have.contains(&(*pre, *post)) {
            child.add_synapse_full(NeuronId(*pre), NeuronId(*post), *w, true, false, Tick(0));
            born += 1;
        }
    }
    if std::env::var("EVOLVE_VERBOSE").is_ok() {
        eprintln!("breed: leftover(would-add)={born}; child live={} parent live={} (expect live<=parent-live if no unintended add)",
            child.live_synapses().count(), parent.live_synapses().count());
    }
    child
}

struct Org { net: Network, size: usize, seed: u64, growth_params: [f32; 9] }
fn main() {
    let spec = survival_spec();
    let p = params();
    // optional argv[1] seed filter: run only the requested seed (e.g.
    // `evolve 424242`). Empty/absent = all SEEDS. (D-46 fix: evolution
    // previously IGNORED argv and always ran all 3 seeds, which
    // invalidated "single-seed" verbose parses - they were another seed.)
    let want: Option<u64> = std::env::args().nth(1).and_then(|a| a.parse().ok());
    // optional argv[2] = gens override (size-push: run past the old
    // ~229 wall, e.g. `evolve 424242 16`). Default GENS.
    let gencap: u32 = std::env::args().nth(2).and_then(|a| a.parse().ok()).unwrap_or(GENS);
    // D-59 (docs/phase3/d59-reflex-integration-protocol.md): in-life gates.
    // D59_REFLEX=1 -> reflex band + reflex novelty detection; D59_MOTOR=1
    // -> closed-loop motor world. Both off = byte-identical baseline.
    let d59_reflex_k: usize = if std::env::var("D59_REFLEX").is_ok() { reflex::REFLEX_K } else { 0 };
    let d59_motor: bool = std::env::var("D59_MOTOR").is_ok();
    // D-59 per-gen summary columns only when the run is D-59-active.
    let d59_show = d59_reflex_k > 0 || d59_motor;
    for &esec in &SEEDS {
        if let Some(w) = want { if w != esec { continue; } }
        println!("=== seed {esec} gens={gencap} ===");
        let mut pop: Vec<Org> = (0..N_POP).map(|i| { let s = esec ^ (i as u64 * 7919);
            Org { net: build_net(s, 40, 0.0), size: 40, seed: s, growth_params: [0.05, 0.01, 0.02, 120.0, 0.1, 30.0, 1500.0, 3000.0, 0.0] } }).collect();
        let mut formed_flags: Vec<bool> = vec![false; N_POP]; // gen-0 organisms form; offspring inherit
        let mut gen_sizes: Vec<f32> = Vec::new();
        let mut gen_fits: Vec<f32> = Vec::new();
        let mut gen_best: Vec<f32> = Vec::new();
        for g in 0..gencap {
            let mut scored: Vec<(f32, usize, usize, u64)> = Vec::new(); // (fitness, popidx, size, orgseed)
            // D-59 per-gen columns: means of per-org fractions, printed
            // only when Some across ALL organisms of the gen.
            let mut g_reflex: Vec<f32> = Vec::new();
            let mut g_cons: Vec<f32> = Vec::new();
            let mut g_wv: Vec<f32> = Vec::new();
            for (i, (org, formed)) in pop.iter_mut().zip(formed_flags.iter_mut()).enumerate() {
                let mut traces = Traces::new(&org.net, 20.0);
                if !*formed { form_s1(&mut org.net, org.seed); *formed = true; }
                let refs = capture_refs(&mut org.net, org.seed);
                let world_seed = esec; // frozen across generations (comparable fitness)
                // V2 self-construction for this life: growth-law genes are
                // heritable AND selected (the real GA target - the rule the
                // organism uses to build its own connections).
                let mut v2params = v2_params();
                v2params.theta_permanent = org.growth_params[0];
                v2params.delta_perm = org.growth_params[1];
                v2params.w_c_permanent = org.growth_params[2];
                // b_e scales with organism size (a neuron's spare capacity =
                // a fraction k of the pool it may connect to; the gene is k,
                // not an absolute cap - absolute caps break as the organism
                // legitimately grows past them, tripping the M5 assert).
                let k = org.growth_params[3].clamp(0.5, 6.0);
                v2params.b_e = (k * org.net.neurons.len() as f32).max(40.0) as usize;
                // E4 viable birth (D-39): bidirectional participatory
                // newborn + attenuated wiring + scaled budget
                let mut mon = anima_core::structural::StructuralMonitor::default();
                mon.wiring_bidirectional = true;
                mon.wiring_w_scale = org.growth_params[4].clamp(0.02, 0.5);
                let trig_rate = org.growth_params[5].clamp(5.0, 60.0);
                let trig_sust = (org.growth_params[6].clamp(500.0, 5000.0)) as u64;
                let trig_cool = (org.growth_params[7].clamp(1000.0, 8000.0)) as u64;
                let mut trigger = anima_core::structural::make_trigger(
                    "homeostatic-saturation", Some(trig_rate), Some(trig_sust), Some(trig_cool));
                let _ = (&trig_rate, trig_sust, trig_cool); // keep alive if unused
                let pe_state0 = Some((0.0f32, 0.0f32));
                let syn_before = org.net.live_synapses().count();
                let mut v2 = V2Plasticity::new(&mut org.net, v2params, None);
                let structural_opt = Some((mon, trigger));
                // in-loop resource monitor (D-38 advisory): runaway /
                // exhaustion = death, same standard as harness. Capacity
                // sized to allow legit self-construction growth.
                let mut rcfg = anima_core::resources::ResourceConfig::default();
                rcfg.max_neurons = 600;
                // argv[3] = synapse cap override (D-48 registered param,
                // default 20000). NOTE: the 20000 default is an
                // INCIDENTAL monitor-wiring constant (e06bbb3), never
                // registered - the ~229 'wall' may be this budget, not a
                // structural ceiling; cap-raised runs test that.
                // argv[3]: 0 = D-49 size-scaled mode (k=200 synapses per
                // neuron, budget recomputed per check - removes the hard
                // cap to measure the representational slope above 320);
                // default 20000 = fixed cap (D-48 behavior, identity).
                let syn_cap: usize = std::env::args().nth(3).and_then(|a| a.parse().ok()).unwrap_or(20_000);
                if syn_cap == 0 {
                    rcfg.size_scaled_synapses_k = Some(200.0); // measured k (D-49)
                    rcfg.max_synapses = usize::MAX; // fixed path disabled
                } else {
                    rcfg.max_synapses = syn_cap;
                    rcfg.size_scaled_synapses_k = None;
                }
                // ALIGN runaway threshold with the survival loop's own
                // activity ceiling (a_bounds[1]=250 Hz). The harness's
                // 50 Hz fires on HEALTHY pool operation (measured 58.9 Hz
                // mean in a surviving organism) - false death for every
                // organism. Only genuine runaway PAST the operational
                // envelope should trip the monitor.
                // margin above ceiling: near-ceiling growth (253.8 Hz at
                // a_bounds[1]=250) is caught by survival's own a-bounds gate;
                // the monitor is the BACKUP for genuine runaway beyond it.
                rcfg.runaway_rate_hz = spec.a_bounds[1] + 30.0;
                let rmon = Some(anima_core::resources::ResourceMonitor::new(rcfg));
                let out = survival::run_world_full(&mut org.net, org.seed, world_seed, &refs, &spec, &p, &mut traces, Some(&mut v2), 100, structural_opt, pe_state0, true, rmon, d50_mode(), d59_reflex_k, d59_motor);
                let syn_growth = org.net.live_synapses().count() as isize - syn_before as isize;
                org.size = org.net.neurons.len() - 24 - 12;
                // fitness in [0,1]: mean viability scaled by the recognition
                // fraction (multiplicative, not additive - additive let the
                // composite exceed 1.0 and break selection ranking)
                // resource failure (runaway-activity / exhaustion) is the
                // hard failure mode the monitor exists to catch - flat zero
                // fitness so selection aggressively prunes it (D-38).
                // D-38 death-gate (RESTORED after D-44 falsified): death
                // or resource failure -> fitness 0.0, unconditionally.
                // D-44's survival-duration floor was FALSIFIED (174477e):
                // (a) it did NOT rescue recognition-collapse deaths (base
                // = a*r*s = 0 at r=0, floor multiplies 0 -> stays 0); (b)
                // it let mid-fitness dying organisms breed into the pool,
                // REGRESSING seed 9001 by 30% (230->160). Flat 0 is the
                // registered baseline; keep it.
                let f = if out.failed.is_some() || out.died_at.is_some() {
                    let _ = out.failed; // keep field referenced
                    0.0
                } else {
                    // D-50: in d50 mode, multiply by the pairwise separation
                    // falsifier so selection must HOLD separation across all
                    // N(N-1)/2 pairs, not just survive. Legacy mode: no change.
                    let sep = if d50_mode() == "d50" || d50_mode() == "d50-2" {
                        let (acc, pairs) = capture_separation(&mut org.net, org.seed, &refs, &spec);
                        // D-53/54 reconciliation: the refs above were
                        // captured PRE-survival (line 338); capture_separation
                        // runs POST-survival. Re-capture on the POST-survival
                        // net and re-measure - if sep jumps, the low sep was
                        // a stale-refs artifact (state mismatch), not a
                        // representation limit.
                        let postrefs = capture_refs(&mut org.net, org.seed);
                        let (acc2, pairs2) = capture_separation(&mut org.net, org.seed, &postrefs, &spec);
                        if std::env::var("EVOLVE_VERBOSE").is_ok() {
                            eprintln!("  org {i} SEPSPLIT pre-survival-refs={acc:.3} post-survival-refs={acc2:.3}");
                            eprintln!("  org {i} pair2-post: {:?}", pairs2);
                        }
                        if std::env::var("EVOLVE_VERBOSE").is_ok() {
                            let mut coses = String::new();
                            for ri in 0..refs.len(){ for rj in ri+1..refs.len(){
                                coses.push_str(&format!("cos({}-{})={:.3} ", refs[ri].0, refs[rj].0, io::cos(&refs[ri].1,&refs[rj].1)));
                            }}
                            eprintln!("  org {i} refs: {}", coses);
                            eprintln!("  org {i} D50-sep: {:?} mean={acc:.3}", pairs);
                        }
                        acc
                    } else { 1.0 };
                    out.mean_viability * (0.5 + 0.5 * out.known_recognized_frac) * sep
                };
                // death-cause instrumentation (D-47): thread died_at +
                // fail kind into the scoring line so the wall mechanism
                // (recognition-collapse-while-alive vs runaway) is
                // recorded - it determines the next intervention.
                if std::env::var("D52_TIMING").is_ok() {
                    let mt = d50_mode();
                    for s2 in &io::known_syms(mt) {
                        probe_output_timing(&mut org.net, org.seed, s2, mt);
                    }
                }
                if std::env::var("EVOLVE_VERBOSE").is_ok() {
                    eprintln!("  org {i} NOVEL: detected={} contaminated={} (loop-side, gap-separated)",
                        out.novel_detected_frac, out.novel_contaminated);
                    if d59_show {
                        eprintln!("  org {i} reflex: det_frac={:?} cons_frac={:?} wv_frac={:?}",
                            out.reflex_novel_detected_frac, out.consequence_novel_frac, out.world_consequence_violation_frac);
                    }
                    let mut coses = String::new();
                    for ri in 0..refs.len() {
                        for rj in (ri+1)..refs.len() {
                            coses.push_str(&format!("cos({}-{})={:.3} ", refs[ri].0, refs[rj].0, io::cos(&refs[ri].1, &refs[rj].1)));
                        }
                    }
                    let vecs: Vec<String> = refs.iter().map(|(p,v)| {
                        let elems: Vec<String> = v.iter().map(|x| format!("{:.2}", x)).collect();
                        format!("{p}=[{}]", elems.join(","))
                    }).collect();
                    eprintln!("  org {i} mode={} REFS: {} | {}", d50_mode(), vecs.join("  "), coses);
                    eprintln!("  org {i}: dead={:?} fail={:?} n={} syn={} fit={f:.3}",
                        out.died_at, out.failed, org.net.neurons.len(),
                        org.net.live_synapses().count());
                }
                if std::env::var("EVOLVE_VERBOSE").is_ok() {
                    eprintln!("  org {i}: gp={:?} syn_growth={syn_growth:+} fit={f:.3}",
                        org.growth_params, );
                }
                scored.push((f, i, org.size, org.seed));
                if let Some(v) = out.reflex_novel_detected_frac { g_reflex.push(v); }
                if let Some(v) = out.consequence_novel_frac { g_cons.push(v); }
                if let Some(v) = out.world_consequence_violation_frac { g_wv.push(v); }
            }
            scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
            let mean_sz = scored.iter().map(|(_, _, s, _)| *s as f32).sum::<f32>() / scored.len() as f32;
            let mean_f = scored.iter().map(|(f, _, _, _)| *f).sum::<f32>() / scored.len() as f32;
            let best = scored[0].0;
            gen_sizes.push(mean_sz); gen_fits.push(mean_f); gen_best.push(best);
            // D-59: append reflex/cons/wv columns ONLY on active runs
            // (fields all Some); flag-off prints the legacy line
            // byte-identically. '-' = not Some across all organisms.
            if d59_show {
                let mean = |v: &Vec<f32>| if v.len() == N_POP {
                    format!("{:.2}", v.iter().sum::<f32>() / N_POP as f32)
                } else { "-".to_string() };
                println!(" gen {g}: fit=[{}] mean_sz={mean_sz:.1} mean_fit={mean_f:.3} best={best:.3} reflex={} cons={} wv={}",
                    scored.iter().map(|(f, _, s, _)| format!("{s}:{f:.2}")).collect::<Vec<_>>().join(" "),
                    mean(&g_reflex), mean(&g_cons), mean(&g_wv));
            } else {
                println!(" gen {g}: fit=[{}] mean_sz={mean_sz:.1} mean_fit={mean_f:.3} best={best:.3}",
                    scored.iter().map(|(f, _, s, _)| format!("{s}:{f:.2}")).collect::<Vec<_>>().join(" "));
            }
            // selection: ELITISM (best organism carried verbatim - prevents
            // destructive mutation from erasing the winner) + breed 3
            // mutated offspring from the top 2
            let mut rng = Xoshiro256PlusPlus::seed_from_u64(esec ^ (g as u64) ^ 0xDEAD);
            let mut next: Vec<Org> = Vec::new();
            // determinism check (gen 0 only): re-score organism 0 twice;
            // identical fitness required (same world + same org seed)
            if g == 0 && std::env::var("EVOLVE_DETERMINISM").is_ok() {
                // rebuild THE SAME organism that scored[0] belongs to (its
                // own seed + size), rescore under the same world, compare.
                let (_, _, best_size, best_seed) = scored[0];
                let mut net2 = build_net(best_seed, best_size, pop[scored[0].1].growth_params[8]);
                let mut traces2 = Traces::new(&net2, 20.0);
                form_s1(&mut net2, best_seed);
                let refs2 = capture_refs(&mut net2, best_seed);
                let out2 = survival::run_world(&mut net2, best_seed, esec ^ (g as u64).wrapping_mul(0xABCDEF), &refs2, &spec, &p, &mut traces2);
                let f2 = out2.mean_viability * (0.5 + 0.5 * out2.known_recognized_frac);
                eprintln!("DETERMINISM: best-org rebuild fit={:.6} vs scored {:.6} -> {}",
                    f2, scored[0].0, if (f2 - scored[0].0).abs() < 1e-4 {"IDENTICAL"} else {"DIVERGES"});            }
            // ELITISM (D-32 amendment): the best organism is carried to the
            // next generation UNMUTATED (pure copy by (pre,post) inheritance,
            // seed = parent's) so destructive mutation cannot erase the
            // winner. Then 3 mutated offspring from the top 2.
            let elite_idx = scored[0].1;
            let elite = &pop[elite_idx];
            // growth-law genes: elite gets a light mutation (0.2), offspring
            // heavier (0.3) - selection acts on the SELF-CONSTRUCTION RULE.
            let mut egp = elite.growth_params;
            if rng.gen::<f32>() < 0.2 { egp[0] = (egp[0] * (0.5 + rng.gen::<f32>())).clamp(0.01, 0.2); }
            next.push(Org { net: breed_no_mut(&elite.net, elite.seed, elite.size, egp[8]),
                size: elite.size, seed: elite.seed, growth_params: egp });
            for (pi, (_, idx, sz, _)) in scored.iter().take(2).enumerate() {
                let parent = &pop[*idx];
                let n_off: u32 = if pi == 0 { 2 } else { 1 };
                for off in 0..n_off {
                    let sz2 = *sz; // NO size mutation: growth is self-emergent
                    // via M3 during life, never hand-resized (D-36).
                    let cs = esec ^ (g as u64) << 8 ^ (off as u64 * 104729);
                    let mut cgp = parent.growth_params;
                    for pi in 0..9 {
                        if rng.gen::<f32>() < 0.3 {
                            if pi == 8 {
                                // D-46: output_inhibition_gain. ADDITIVE
                                // mutation so selection can reach the
                                // known-good range (~0.1-0.2). Multiplicative
                                // stuck it at the 0.005 floor (inert) - the
                                // calibrated probes showed 0.1 works, 0.05
                                // hurts, 0 is the healthy-seed baseline.
                                cgp[8] = (cgp[8] + (rng.gen::<f32>() * 0.2 - 0.05)).clamp(0.0, 0.5);
                            } else {
                                cgp[pi] = (cgp[pi] * (0.5 + rng.gen::<f32>())).clamp(0.005, 300.0);
                            }
                        }
                    }
                    let child = breed(&parent.net, cs, sz2, cgp[8]);
                    next.push(Org { net: child, size: sz2, seed: cs, growth_params: cgp });
                }
            }
            formed_flags = vec![true; next.len()]; // offspring inherit trained weights
            pop = next;
        }
        println!(" seed {esec} RESULT: size g0={:.1} -> g7={:.1}  fit g0={:.3} -> g7={:.3}  best_g7={:.3}",
            gen_sizes[0], *gen_sizes.last().unwrap(), gen_fits[0], *gen_fits.last().unwrap(), *gen_best.last().unwrap());
    }
}