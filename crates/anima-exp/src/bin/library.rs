//! D-57 Phase-1 LIBRARY world. Honest metrics only (D-53 rules):
//! per-symbol held-out recognition against a fresh cross-symbol ref
//! set; novelty = D decodes NOVEL/UNSURE vs same refs (no self-ref,
//! no tautology). refs refreshed per stage (D-55 state-matching).
use anima_core::network::{Network, NetworkConfig, Tick, InputFrame};
use anima_core::plasticity::{stdp_tick, StdpParams, Traces};
use anima_exp::io;

fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(20260912);
    let mode = "d50";
    let mut net = Network::new(v2cfg(), 24, 40, 12, seed);
    let p = StdpParams { tau_plus: 20.0, tau_minus: 20.0, a_plus: 0.005, a_minus: 0.0053, decay: 1e-6, w_min: 0.0, w_max: 1.0 };
    let stages: Vec<Vec<&str>> = vec![vec!["A"], vec!["A", "C"], vec!["A", "C", "E"]];
    let known_all = ["A", "C", "E"];
    let mut tr = Traces::new(&net, 20.0);
    let mut prev: Vec<&str> = Vec::new();
    println!("== library seed={} mode={} ==", seed, mode);
    for (si, stage) in stages.iter().enumerate() {
        for s in stage { if !prev.contains(s) { form_one(&mut net, seed, mode, s, &p, &mut tr); } }
        prev = stage.clone();
        let refs: Vec<(String, Vec<f32>)> = stage.iter().map(|s| {
            (s.to_string(), capture_ref(&mut net, seed, mode, s, &p, &mut tr))
        }).collect();
        let beat_n = 3u64;
        let mut rec: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
        let mut pres: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
        let mut novel_det = 0u32;
        for _ in 0..beat_n {
            for s in stage {
                let act = decode_beat(&mut net, seed, mode, s, &p, &mut tr, &refs);
                if act == *s { *rec.entry(s.to_string()).or_insert(0) += 1; }
                *pres.entry(s.to_string()).or_insert(0) += 1;
                gap(&mut net, &p, &mut tr, 1500);
            }
            let act_d = decode_beat(&mut net, seed, mode, "D", &p, &mut tr, &refs);
            if act_d == "NOVEL" || act_d == "UNSURE" { novel_det += 1; }
            gap(&mut net, &p, &mut tr, 1500);
        }
        let mut line = format!("  stage{} vocab={:?} ", si + 1, stage);
        for s in known_all {
            if stage.contains(&s) {
                let ok = *rec.get(s).unwrap_or(&0);
                let tot = *pres.get(s).unwrap_or(&1);
                line.push_str(&format!("{s}={ok}/{tot} "));
            } else { line.push_str(&format!("{s}=UNTRAINED ")); }
        }
        line.push_str(&format!("novel_det={}/{}", novel_det, beat_n));
        println!("{line}");
    }
    println!("done");
}

fn capture_ref(net: &mut Network, seed: u64, mode: &str, sym: &str, _p: &StdpParams, _tr: &mut Traces) -> Vec<f32> {
    let st = io::symbol_trains_mode(sym, mode, seed);
    let mut out = vec![0.0f32; 12];
    for t in 0..io::BEAT_MS {
        let f = InputFrame { tick: net.tick, spikes: st.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect() };
        let e = net.step(&f);
        for c in &e.spikes { if (io::OUTPUT_LO..io::OUTPUT_HI).contains(&c.0) { out[(c.0 - io::OUTPUT_LO) as usize] += 1.0; } }
        net.tick = Tick(net.tick.0 + 1);
    }
    out
}
fn decode_beat(net: &mut Network, seed: u64, mode: &str, sym: &str, p: &StdpParams, tr: &mut Traces, refs: &[(String, Vec<f32>)]) -> String {
    let v = capture_ref(net, seed, mode, sym, p, tr);
    io::decode(&v, refs, 1.0, 0.20)
}
fn form_one(net: &mut Network, seed: u64, mode: &str, sym: &str, p: &StdpParams, tr: &mut Traces) {
    for _ in 0..20 {
        let st = io::symbol_trains_mode(sym, mode, seed);
        for t in 0..io::BEAT_MS {
            let f = InputFrame { tick: net.tick, spikes: st.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect() };
            let e = net.step(&f); tr.step(net, &e.spikes);
            let _ = stdp_tick(p, net, tr, &e.spikes, 1.0, None);
            net.tick = Tick(net.tick.0 + 1);
        }
        gap(net, p, tr, 1500);
    }
}
fn gap(net: &mut Network, p: &StdpParams, tr: &mut Traces, ms: u64) {
    for _ in 0..ms {
        let f = InputFrame { tick: net.tick, spikes: vec![] };
        let e = net.step(&f); tr.step(net, &e.spikes);
        let _ = stdp_tick(p, net, tr, &e.spikes, 1.0, None);
        net.tick = Tick(net.tick.0 + 1);
    }
}
fn v2cfg() -> NetworkConfig {
    NetworkConfig {
        connectivity: 0.038, w_init: 0.2, amplitude: 52.0, adaptation_tau_ms: 200.0,
        adaptation_gain: 0.05, inhibition_gain: 0.0, slow_state_beta: 0.0046875,
        slow_state_tau_ms: 5000.0, slow_state_beta_drive: false, latch_enable: true,
        theta_rel_mean: 1.0, theta_rel_sd: 0.0, u_plateau_rel_mean: 1.0, u_plateau_rel_sd: 0.0,
        tau_het_rel_sd: 0.0, phi_rel: 0.5, eta_rel: 0.0, v2: Some(v2p()), ..Default::default()
    }
}
fn v2p() -> anima_core::network::V2Params {
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
        disable_m5: false, disable_m6: false
    }
}