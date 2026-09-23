//! D-37 gate probe: does homeostatic-saturation birth trigger create a
//! VIABLE new neuron in the survival regime (born + survives, no runaway)?
//! If yes, the GA build (D-38) has something to select for.
use anima_core::network::{InputFrame, Network, NetworkConfig, NeuronId, Tick, V2Params};
use anima_core::plasticity::{stdp_tick, StdpParams, Traces};
use anima_core::structural_v2::V2Plasticity;
use anima_exp::config::SurvivalSpec;
use anima_exp::io;
use anima_exp::survival;

fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(20260912);
    let beats: u64 = std::env::args().nth(2).and_then(|a| a.parse().ok()).unwrap_or(100);
    let mut net = build(seed, 40);
    let p = params();
    form(&mut net, seed);
    let refs = refs_of(&mut net, seed);
    let spec = SurvivalSpec { beats, r_window: 10, th_known: 0.20, q_floor: 1.0,
        a_bounds: [5.0, 250.0], horizon: 30, off_ms: 1500, p_novel: 0.2 };
    // homeostatic-saturation birth trigger
    let avoid_coactive: bool = std::env::args().nth(4).map(|a| a == "1" || a == "true").unwrap_or(false);
    let mut mon = anima_core::structural::StructuralMonitor::default();
    mon.wiring_avoid_coactive = avoid_coactive;
    let bidirectional: bool = std::env::args().nth(5).map(|a| a == "1" || a == "true").unwrap_or(false);
    mon.wiring_bidirectional = bidirectional;
    let wscale: f32 = std::env::args().nth(6).and_then(|a| a.parse().ok()).unwrap_or(1.0);
    mon.wiring_w_scale = wscale;
    let rate: f32 = std::env::args().nth(3).and_then(|a| a.parse().ok()).unwrap_or(30.0);
    let mut trigger = anima_core::structural::make_trigger(
        "homeostatic-saturation", Some(rate), Some(2000), Some(5000));
    let n_before = net.neurons.len();
    let syn_before = net.live_synapses().count();
    let mut traces = Traces::new(&net, 20.0);
    let mut v2p = v2_params();
    let be: usize = std::env::args().nth(7).and_then(|a| a.parse().ok()).unwrap_or(120);
    // size-scaled budget: k * n_neurons (survival probe passes k in arg 7)
    let k = be as f32;
    v2p.b_e = (k * net.neurons.len() as f32).max(40.0) as usize;
    let mut v2 = V2Plasticity::new(&mut net, v2p, None);
    let out = survival::run_world_full(
        &mut net, seed, seed, &refs, &spec, &p, &mut traces,
        Some(&mut v2), 100,
        Some((mon, trigger)), None, true,
    );
    let born = net.neurons.len() - n_before;
    let syn_growth = net.live_synapses().count() as isize - syn_before as isize;
    println!("birthprobe seed={seed} beats={beats}: neurons {n_before}->{} (born={born}) syn_growth={syn_growth:+} died_at={:?} fit={:.3}",
        net.neurons.len(), out.died_at,
        out.mean_viability * (0.5 + 0.5 * out.known_recognized_frac));
}

fn v2_params() -> V2Params {
    V2Params {
        p_in: 0.5, w_in_lo: 0.02, w_in_hi: 0.06, p_rec: 0.2, w_rec_lo: 0.005, w_rec_hi: 0.02,
        t_e: 0.8, assembly_protect: true, p_max_frac: 0.75, w_consolidate_min: 0.05,
        alloc_residual: true, dormant_reserve: false, recruit_gain: false,
        d_core: true, d_claim: true, d_sparse: false, d_elig: false, d_elig_ro: false, d_ing: false,
        c_slots: 6, w_c_init: 0.01, delta_perm: 0.01, decay_c: 0.99, theta_permanent: 0.05,
        w_c_permanent: 0.02, theta_die: 0.005, p_cand_in: 0.5, p_cand_rec: 0.5,
        theta_prune: 0.005, prune_windows: 10, b_e: 120, b_i: 30,
        p_inh: 0.3, w_inh_lo: 0.01, w_inh_hi: 0.03, a_inh: 0.005, decay_inh: 0.98, w_inh_max: 0.10,
        window_ticks: 100, m2_buckets: 1, m2_epoch_windows: 1, disable_m2: false, disable_m3_m4: false,
        disable_m5: false, disable_m6: false,
    }
}
fn build(seed: u64, n_internal: usize) -> Network {
    let cfg = NetworkConfig {
        connectivity: 0.038, w_init: 0.2, amplitude: 52.0, adaptation_tau_ms: 200.0,
        adaptation_gain: 0.05, inhibition_gain: 0.0, slow_state_beta: 0.0046875,
        slow_state_tau_ms: 5000.0, slow_state_beta_drive: false, latch_enable: true,
        theta_rel_mean: 1.0, theta_rel_sd: 0.0, u_plateau_rel_mean: 1.0, u_plateau_rel_sd: 0.0,
        tau_het_rel_sd: 0.0, phi_rel: 0.5, eta_rel: 0.0, v2: Some(v2_params()),
        ..NetworkConfig::default()
    };
    Network::new(cfg, 24, n_internal, 12, seed)
}
fn params() -> StdpParams {
    StdpParams { tau_plus: 20.0, tau_minus: 20.0, a_plus: 0.005, a_minus: 0.0053,
        decay: 1e-6, w_min: 0.0, w_max: 1.0 }
}
fn form(net: &mut Network, seed: u64) {
    let p = params(); let mut traces = Traces::new(net, 20.0);
    for _rep in 0..20usize { for sym in ["A", "C"] {
        let tr = io::symbol_trains(sym, seed);
        for t in 0..io::BEAT_MS {
            let f = InputFrame { tick: net.tick, spikes: tr.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect() };
            let ev = net.step(&f); traces.step(net, &ev.spikes);
            let _ = stdp_tick(&p, net, &traces, &ev.spikes, 1.0, None);
            net.tick = Tick(net.tick.0 + 1);
        }
        for _ in 0..1500 {
            let f = InputFrame { tick: net.tick, spikes: vec![] };
            let ev = net.step(&f); traces.step(net, &ev.spikes);
            let _ = stdp_tick(&p, net, &traces, &ev.spikes, 1.0, None);
            net.tick = Tick(net.tick.0 + 1);
        }
    }}
}
fn refs_of(net: &mut Network, seed: u64) -> Vec<(String, Vec<f32>)> {
    let mut acc: std::collections::BTreeMap<String, Vec<f32>> = Default::default();
    let mut cnt: std::collections::BTreeMap<String, u32> = Default::default();
    for sym in ["A", "C"] { for _ in 0..2u64 {
        let tr = io::symbol_trains(sym, seed); let mut out = vec![0f32; 12];
        for t in 0..io::BEAT_MS {
            let f = InputFrame { tick: net.tick, spikes: tr.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect() };
            let ev = net.step(&f);
            for c in &ev.spikes { if (64..76).contains(&c.0) { out[(c.0 - 64) as usize] += 1.0; } }
            net.tick = Tick(net.tick.0 + 1);
        }
        let e = acc.entry(sym.to_string()).or_insert_with(|| vec![0f32; 12]);
        for i in 0..12 { e[i] += out[i]; }
        *cnt.entry(sym.to_string()).or_insert(0) += 1;
    }}
    acc.into_iter().map(|(p, v)| { let c = *cnt.get(&p).unwrap_or(&1).max(&1) as f32; (p, v.iter().map(|x| x / c).collect()) }).collect()
}