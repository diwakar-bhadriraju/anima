//! Phase III sZ evolution/death-selection (docs/phase3/s-evolution-protocol.md
//! D-32, frozen). Population N=4, elitist top-2 keep + 2 offspring each;
//! variation = brain-size mutation (n_internal +/-4, band [32,56]) + weight
//! mutation (+/-10% p=0.1 via (pre,post) key inheritance); life = S1
//! formation (live STDP) + closed survival loop; fitness = mean viability
//! (+ small recognition bonus); 8 gens x 3 seeds. PRIMARY: does population
//! mean size rise/fall/flat (the user's "network too small" question).
use std::collections::HashMap;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use anima_core::network::{Network, NetworkConfig, Tick, V2Params};
use anima_core::plasticity::{stdp_tick, StdpParams, Traces};

use anima_exp::config::SurvivalSpec;
use anima_exp::io;
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
fn build_net(seed: u64, n_internal: usize) -> Network {
    let cfg = NetworkConfig {
        connectivity: 0.038, w_init: 0.2, amplitude: 52.0,
        adaptation_tau_ms: 200.0, adaptation_gain: 0.05, inhibition_gain: 0.0,
        slow_state_beta: 0.0046875, slow_state_tau_ms: 5000.0, slow_state_beta_drive: false,
        latch_enable: true, theta_rel_mean: 1.0, theta_rel_sd: 0.0, u_plateau_rel_mean: 1.0,
        u_plateau_rel_sd: 0.0, tau_het_rel_sd: 0.0, phi_rel: 0.5, eta_rel: 0.0,
        v2: Some(v2_params()), ..NetworkConfig::default()
    };
    Network::new(cfg, 24, n_internal, 12, seed)
}
fn params() -> StdpParams {
    StdpParams { tau_plus: 20.0, tau_minus: 20.0, a_plus: 0.005, a_minus: 0.0053,
        decay: 1e-6, w_min: 0.0, w_max: 1.0 }
}
fn survival_spec() -> SurvivalSpec {
    SurvivalSpec { beats: 30, r_window: 10, th_known: 0.20, q_floor: 1.0,
        a_bounds: [5.0, 250.0], horizon: 30, off_ms: 1500, p_novel: 0.2 }
}

fn form_s1(net: &mut Network, seed: u64) {
    let p = params();
    let mut traces = Traces::new(net, 20.0);
    for _rep in 0..20usize {
        for sym in ["A", "C"] {
            let tr = io::symbol_trains(sym, seed);
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
    for sym in ["A", "C"] {
        for _ in 0..3u64 {
            let tr = io::symbol_trains(sym, seed);
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

/// Inherit parent's weights by (pre,post) key into a fresh child; apply
/// weight mutation. n_internal may differ (size mutation): shared neurons
/// inherit, new neurons' synapses stay fresh.
/// Pure copy of the parent brain (no mutation) - elitism.
fn breed_no_mut(parent: &Network, seed: u64, n_internal: usize) -> Network {
    breed_inner(parent, seed, n_internal, false)
}
fn breed(parent: &Network, seed: u64, n_internal: usize) -> Network {
    breed_inner(parent, seed, n_internal, true)
}
fn breed_inner(parent: &Network, seed: u64, n_internal: usize, mutate: bool) -> Network {
    let mut wmap: HashMap<(u32, u32), f32> = HashMap::new();
    for s in &parent.synapses { wmap.insert((s.pre.0, s.post.0), s.w); }
    let mut child = build_net(seed, n_internal);
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
    let n_shared = parent.neurons.len().min(child.neurons.len());
    for s in child.synapses.iter_mut() {
        if s.pre.0 < n_shared as u32 && s.post.0 < n_shared as u32 {
            if let Some(w) = wmap.get(&(s.pre.0, s.post.0)).copied() {
                let mut w = w;
                if mutate && rng.gen::<f32>() < W_MUT_P { w += (rng.gen::<f32>() * 2.0 - 1.0) * W_MUT_AMP * w; }
                s.w = w.clamp(0.0, 1.0);
            }
        }
    }
    child
}

struct Org { net: Network, size: usize, seed: u64 }
fn main() {
    let spec = survival_spec();
    let p = params();
    for &esec in &SEEDS {
        println!("=== seed {esec} ===");
        let mut pop: Vec<Org> = (0..N_POP).map(|i| { let s = esec ^ (i as u64 * 7919);
            Org { net: build_net(s, 40), size: 40, seed: s } }).collect();
        let mut formed_flags: Vec<bool> = vec![false; N_POP]; // gen-0 organisms form; offspring inherit
        let mut gen_sizes: Vec<f32> = Vec::new();
        let mut gen_fits: Vec<f32> = Vec::new();
        let mut gen_best: Vec<f32> = Vec::new();
        for g in 0..GENS {
            let mut scored: Vec<(f32, usize, usize, u64)> = Vec::new(); // (fitness, popidx, size, orgseed)
            for (i, (org, formed)) in pop.iter_mut().zip(formed_flags.iter_mut()).enumerate() {
                let mut traces = Traces::new(&org.net, 20.0);
                if !*formed { form_s1(&mut org.net, org.seed); *formed = true; }
                let refs = capture_refs(&mut org.net, org.seed);
                let world_seed = esec; // frozen across generations (comparable fitness)
                let out = survival::run_world(&mut org.net, org.seed, world_seed, &refs, &spec, &p, &mut traces);
                org.size = org.net.neurons.len() - 24 - 12;
                // fitness in [0,1]: mean viability scaled by the recognition
                // fraction (multiplicative, not additive - additive let the
                // composite exceed 1.0 and break selection ranking)
                let f = out.mean_viability * (0.5 + 0.5 * out.known_recognized_frac);
                scored.push((f, i, org.size, org.seed));
            }
            scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
            let mean_sz = scored.iter().map(|(_, _, s, _)| *s as f32).sum::<f32>() / scored.len() as f32;
            let mean_f = scored.iter().map(|(f, _, _, _)| *f).sum::<f32>() / scored.len() as f32;
            gen_sizes.push(mean_sz); gen_fits.push(mean_f); gen_best.push(scored[0].0);
            println!(" gen {g}: fit=[{}] mean_sz={mean_sz:.1} mean_fit={mean_f:.3} best={best:.3}",
                scored.iter().map(|(f, _, s, _)| format!("{s}:{f:.2}")).collect::<Vec<_>>().join(" "),
                best = scored[0].0);
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
                let mut net2 = build_net(best_seed, best_size);
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
            next.push(Org { net: breed_no_mut(&elite.net, elite.seed, elite.size),
                size: elite.size, seed: elite.seed });
            for (_, idx, sz, _) in scored.iter().take(2) {
                let parent = &pop[*idx];
                for off in 0..1 {
                    let mut sz2 = *sz as i32;
                    let r = rng.gen::<f32>();
                    if r < 0.35 { sz2 += SIZE_STEP; } else if r < 0.70 { sz2 -= SIZE_STEP; }
                    let sz2 = sz2.clamp(BAND[0], BAND[1]) as usize;
                    let cs = esec ^ (g as u64) << 8 ^ (off as u64 * 104729) ^ (sz2 as u64);
                    let child = breed(&parent.net, cs, sz2);
                    next.push(Org { net: child, size: sz2, seed: cs });
                }
            }
            formed_flags = vec![true; next.len()]; // offspring inherit trained weights
            pop = next;
        }
        println!(" seed {esec} RESULT: size g0={:.1} -> g7={:.1}  fit g0={:.3} -> g7={:.3}  best_g7={:.3}",
            gen_sizes[0], *gen_sizes.last().unwrap(), gen_fits[0], *gen_fits.last().unwrap(), *gen_best.last().unwrap());
    }
}