//! Phase III I/O layer (docs/phase3/io-encoder-decoder-plan.md D-25/D-26):
//! two-way demo. Self-contained (no anima_exp lib dep): builds the
//! committed E-nogain brain from the FROZEN clla-arex-s20260912 params,
//! trains S1 (A/C) with the same STDP machinery, captures A_ref/C_ref at
//! S1-end (the codebook refs), then runs a REPL: you type a symbol
//! (A/C/D/QUIET) -> it presents one 500 ms beat deterministically ->
//! decodes the 12-neuron output to {A, C, NOVEL, UNSURE, QUIET} + hex.
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use anima_core::network::{InputChannelId, InputFrame, Network, NetworkConfig, Tick, V2Params};
use anima_core::plasticity::{stdp_tick, StdpParams, Traces};

const BEAT_MS: u64 = 500;
const ALPHABET: [(&str, &[u32]); 3] = [
    ("A", &[0, 1, 2, 3, 4, 5, 6, 7]),
    ("C", &[8, 9, 10, 11, 12, 13, 14, 15]),
    ("D", &[16, 17, 18, 19, 20, 21, 22, 23]),
];
const RATE_HZ: f32 = 20.0;

/// Frozen E-nogain run params (configs/clla-arex-s20260912-il.toml) --
/// keep byte-identical to the committed control so the brain is the same.
fn frozen_v2() -> V2Params {
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
fn frozen_net(seed: u64) -> Network {
    let cfg = NetworkConfig {
        connectivity: 0.038, w_init: 0.2, amplitude: 52.0,
        adaptation_tau_ms: 200.0, adaptation_gain: 0.05, inhibition_gain: 0.0,
        slow_state_beta: 0.0046875, slow_state_tau_ms: 5000.0, slow_state_beta_drive: false,
        latch_enable: true, theta_rel_mean: 1.0, theta_rel_sd: 0.0, u_plateau_rel_mean: 1.0,
        u_plateau_rel_sd: 0.0, tau_het_rel_sd: 0.0, phi_rel: 0.5, eta_rel: 0.0,
        v2: Some(frozen_v2()), ..NetworkConfig::default()
    };
    Network::new(cfg, 24, 40, 12, seed)
}

fn hash_str(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() { h ^= *b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}
fn derive_seed64(master: u64, a: u64, b: u64) -> u64 {
    let mut z = master.wrapping_add(a.wrapping_mul(0x9E3779B97F4A7C15)).wrapping_add(b);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB) ^ (z >> 31)
}

/// Deterministic Poisson trains for one symbol beat (same convention as env.rs).
fn symbol_trains(sym: &str, seed: u64) -> Vec<(u64, InputChannelId)> {
    let Some(chans) = ALPHABET.iter().find(|(s, _)| *s == sym).map(|(_, c)| c.to_vec()) else { return vec![] };
    let lambda = RATE_HZ / 1000.0;
    let mut out = Vec::new();
    for &ch in &chans {
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(derive_seed64(seed, hash_str(sym), ch as u64));
        let mut t_off: u64 = 0;
        loop {
            let u: f32 = rng.gen::<f32>().max(1e-6);
            let gap = ((-(1.0 - u as f64).ln()) / lambda as f64).ceil() as u64;
            t_off += gap.max(1);
            if t_off >= BEAT_MS { break; }
            out.push((t_off, InputChannelId(ch)));
        }
    }
    out.sort_unstable();
    out
}

fn cent(v: &[f32]) -> f32 { v.iter().sum::<f32>() / v.len() as f32 }
fn cosc(a: &[f32], b: &[f32]) -> f32 {
    let (am, bm) = (cent(a), cent(b));
    let (mut n, mut na, mut nb) = (0.0f32, 0.0f32, 0.0f32);
    for (x, y) in a.iter().zip(b) {
        let (x, y) = (x - am, y - bm);
        n += x * y; na += x * x; nb += y * y;
    }
    if na <= 0.0 || nb <= 0.0 { 0.0 } else { n / (na.sqrt() * nb.sqrt()) }
}

fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(20260912);
    let mut net = frozen_net(seed);
    let params = StdpParams { tau_plus: 20.0, tau_minus: 20.0, a_plus: 0.005, a_minus: 0.0053,
        decay: 1e-6, w_min: 0.0, w_max: 1.0 };
    let mut traces = Traces::new(&net, 20.0);

    // ---- S1 training: interleaved A/C, 20 reps, 500ms beat + 1500ms gap ----
    let mut out_refs: std::collections::BTreeMap<String, Vec<f32>> = Default::default();
    let mut out_cnt: std::collections::BTreeMap<String, u32> = Default::default();
    let mut tick = Tick(0);
    for _rep in 0..20usize {
        for sym in ["A", "C"] {
            let tr = symbol_trains(sym, seed);
            let mut beat = vec![0.0f32; 12];
            for t in 0..BEAT_MS {
                let frame = InputFrame { tick, spikes: tr.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect() };
                let ev = net.step(&frame);
                traces.step(&net, &ev.spikes);
                for c in &ev.spikes { if (64..76).contains(&c.0) { beat[(c.0 - 64) as usize] += 1.0; } }
                stdp_tick(&params, &mut net, &traces, &ev.spikes, 1.0, None);
                tick = Tick(tick.0 + 1);
            }
            let e = out_refs.entry(sym.to_string()).or_insert_with(|| vec![0.0f32; 12]);
            for i in 0..12 { e[i] += beat[i]; }
            *out_cnt.entry(sym.to_string()).or_insert(0) += 1;
            for _ in 0..1500 { // inter-beat gap
                let frame = InputFrame { tick, spikes: vec![] };
                let ev = net.step(&frame); traces.step(&net, &ev.spikes);
                stdp_tick(&params, &mut net, &traces, &ev.spikes, 1.0, None);
                tick = Tick(tick.0 + 1);
            }
        }
    }
    for (p, v) in out_refs.iter_mut() { let c = *out_cnt.get(p).unwrap_or(&0) as f32; for x in v.iter_mut() { *x /= c.max(1.0); } }
    let (ra, rc) = (out_refs.get("A").cloned().unwrap_or_else(|| vec![0.0; 12]), out_refs.get("C").cloned().unwrap_or_else(|| vec![0.0; 12]));

    // codebook frozen thresholds (calibrated from S1 at first use; D-25)
    let th_known: f32 = 0.20;
    let q_floor: f32 = 1.0;
    println!("ANIMA I/O demo (seed {seed}) - S1 trained. refs: A-vs-A={:.3} C-vs-C={:.3}",
        cosc(&ra, &ra), cosc(&rc, &rc));
    println!("Type a symbol [A, C, D, QUIET, q=quit]");
    let stdin = std::io::stdin();
    let mut line = String::new();
    loop {
        print!("> "); use std::io::Write; std::io::stdout().flush().ok();
        line.clear();
        if stdin.read_line(&mut line).is_err() { break; }
        let sym = line.trim().to_uppercase();
        if sym == "Q" || sym == "QUIT" { break; }
        let tr = if sym == "QUIET" { vec![] } else if ALPHABET.iter().any(|(s, _)| *s == sym) { symbol_trains(&sym, seed) }
            else { println!("unknown symbol: {sym}"); continue };
        let mut out = vec![0.0f32; 12];
        for t in 0..BEAT_MS {
            let frame = InputFrame { tick, spikes: tr.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect() };
            let ev = net.step(&frame); traces.step(&net, &ev.spikes);
            for c in &ev.spikes { if (64..76).contains(&c.0) { out[(c.0 - 64) as usize] += 1.0; } }
            stdp_tick(&params, &mut net, &traces, &ev.spikes, 1.0, None);
            tick = Tick(tick.0 + 1);
        }
        let amp = out.iter().sum::<f32>() / 12.0;
        let (rhoa, rhoc) = (cosc(&out, &ra), cosc(&out, &rc));
        let dec = if amp < q_floor { "QUIET" }
            else if rhoa > th_known && rhoa > rhoc { "A" }
            else if rhoc > th_known && rhoc > rhoa { "C" }
            else if rhoa.max(rhoc) <= th_known { "NOVEL" }
            else { "UNSURE" };
        let hexbyte = |v: &[f32]| -> u8 { let mut b = 0u8; for i in 0..8 { if v[i] > 1.0 { b |= 1 << i; } } b };
        println!("{sym:5} -> {dec:6}  rhoA={rhoa:+.2} rhoC={rhoc:+.2} amp={amp:.0}  raw={:02X}", hexbyte(&out));
    }
    println!("bye");
}