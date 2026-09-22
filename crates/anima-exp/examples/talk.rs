//! Phase III I/O layer (docs/phase3/io-encoder-decoder-plan.md D-25/D-26):
//! FAITHFUL two-way demo. Loads a COMMITTED trained brain (final snapshot)
//! and the A/C output-reference vectors from the run's own S1 telemetry
//! (D-26 ref provenance; mirrors the verified outselect.rs counting loop),
//! then REPL: type a symbol (A/C/D/QUIET) -> present one deterministic
//! beat (NO plasticity in-session) -> decode 12-neuron output to
//! {A, C, NOVEL, UNSURE, QUIET} + raw hex.
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use anima_core::network::{InputChannelId, InputFrame, Network, NetworkConfig, Tick, V2Params};
use anima_core::plasticity::{stdp_tick, StdpParams, Traces};
use anima_telemetry::recorder::read_snapshots;
use anima_telemetry::TelemetryReader;
use anima_telemetry::events::Payload;

const BEAT_MS: u64 = 500;
const ALPHABET: [(&str, &[u32]); 3] = [
    ("A", &[0, 1, 2, 3, 4, 5, 6, 7]),
    ("C", &[8, 9, 10, 11, 12, 13, 14, 15]),
    ("D", &[16, 17, 18, 19, 20, 21, 22, 23]),
];
const RATE_HZ: f32 = 20.0;

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

/// Load trained weights (final snapshot) + S1 output refs (verified
/// two-pass outselect.rs counting, S1 presentations only).
fn load(run_dir: &str, net: &mut Network) -> (Vec<f32>, Vec<f32>, u64) {
    let snaps = read_snapshots(&std::path::Path::new(run_dir).join("snapshots.bin.zst")).expect("snapshots");
    let last = snaps.last().expect(">=1 snapshot");
    // FAITHFUL restore by (pre,post) KEY matching, NOT by index (the run's
    // synapse set diverged from construction order via M3-M4 churn/prune).
    use std::collections::HashMap;
    let mut wmap: HashMap<(u32,u32), f32> = HashMap::new();
    let mut cons_map: HashMap<(u32,u32), bool> = HashMap::new();
    let mut track_map: HashMap<(u32,u32), u8> = HashMap::new();
    for ss in &last.synapses {
        wmap.insert((ss.pre, ss.post), ss.w.unwrap_or(0.0));
        cons_map.insert((ss.pre, ss.post), ss.consolidated);
        track_map.insert((ss.pre, ss.post), ss.track);
    }
    let (mut matched, mut pruned) = (0usize, 0usize);
    for s in net.synapses.iter_mut() {
        let key = (s.pre.0, s.post.0);
        if let Some(w) = wmap.get(&key).copied() {
            s.w = w; s.consolidated = cons_map.get(&key).copied().unwrap_or(false);
            let t = track_map.get(&key).copied().unwrap_or(0); if t != 0 { s.track = t; }
            matched += 1;
        } else {
            // trained brain pruned this synapse - tombstone it
            s.silent_ticks = u64::MAX; pruned += 1;
        }
    }
    println!("  key-restore: snapshotSyn={} matched={matched} pruned(tombstoned)={pruned} netSyn={}", last.synapses.len(), net.synapses.len());
    // neuron dynamic state (v, slow depolarization, latch) - without this
    // the loaded brain is cold/at-rest and won't sustain its trained firing
    for (i, n) in net.neurons.iter_mut().enumerate() {
        if let Some(ns) = last.neurons.get(i) {
            if let Some(v) = ns.v { n.v = v; }
            if let Some(u) = ns.u_slow { n.u_slow = u; }
            if let Some(z) = ns.z_latch { n.z_latch = z; }
            n.retired = ns.retired;
        }
    }
    // refs: pres (S1 A/C) + output spike list, per-presentation window
    let reader = TelemetryReader::open(&std::path::Path::new(run_dir).join("telemetry")).unwrap();
    let idx = reader.chunk_index();
    let mut pres: Vec<(u64, String)> = vec![];
    let mut spk: Vec<(u64, u32)> = vec![];
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            if let Ok(e) = row.envelope("e") {
                match &e.payload {
                    Payload::StimulusPresented { pattern_id, stage } => {
                        if stage == "S1" && (pattern_id == "A" || pattern_id == "C") {
                            pres.push((row.t, pattern_id.clone()));
                        }
                    }
                    Payload::Spike { n } => if (64..76).contains(&n.0) { spk.push((row.t, n.0)); },
                    _ => {}
                }
            }
        }
    }
    pres.sort_by_key(|p| p.0);
    let mut refs: std::collections::BTreeMap<String, Vec<f32>> = Default::default();
    let mut cnt: std::collections::BTreeMap<String, u32> = Default::default();
    for (t, p) in &pres {
        let mut v = vec![0.0f32; 12];
        for (st, s) in &spk { if *st >= *t && *st < t + BEAT_MS { v[(s - 64) as usize] += 1.0; } }
        let e = refs.entry(p.clone()).or_insert_with(|| vec![0.0f32; 12]);
        for i in 0..12 { e[i] += v[i]; }
        *cnt.entry(p.clone()).or_insert(0) += 1;
    }
    for (p, v) in refs.iter_mut() { let c = cnt[p] as f32; for x in v.iter_mut() { *x /= c; } }
    let (ra, rc) = (refs.get("A").cloned().unwrap_or(vec![0.0; 12]), refs.get("C").cloned().unwrap_or(vec![0.0; 12]));
    (ra, rc, last.tick)
}

fn main() {
    let mut run_dir: Option<String> = None;
    let mut seed: u64 = 20260912;
    let mut rest = std::env::args().skip(1);
    while let Some(a) = rest.next() {
        match a.as_str() {
            "--load" => run_dir = rest.next(),
            _ => if let Ok(n) = a.parse::<u64>() { seed = n; },
        }
    }
    let run_dir = run_dir.expect("usage: talk --load <run_dir> [seed]");
    let mut net = frozen_net(seed);
    let params = StdpParams { tau_plus: 20.0, tau_minus: 20.0, a_plus: 0.005, a_minus: 0.0053,
        decay: 1e-6, w_min: 0.0, w_max: 1.0 };
    let mut traces = Traces::new(&net, 20.0);
    let (ra, rc, tick0) = load(&run_dir, &mut net);
    println!("loaded {run_dir} at t={tick0}");
    println!("refA=[{}]", ra.iter().map(|x| format!("{x:.0}")).collect::<Vec<_>>().join(","));
    println!("refC=[{}]", rc.iter().map(|x| format!("{x:.0}")).collect::<Vec<_>>().join(","));
    println!("ref cross-cos(A,C)={:.3}  |A|={:.1} |C|={:.1}", cosc(&ra, &rc), cent(&ra), cent(&rc));
    let th_known = 0.20; let q_floor = 1.0;
    let stdin = std::io::stdin();
    let mut tick = Tick(tick0);
    loop {
        print!("> "); use std::io::Write; std::io::stdout().flush().ok();
        let mut line = String::new();
        if stdin.read_line(&mut line).is_err() { break; }
        let sym = line.trim().to_uppercase();
        if sym == "Q" || sym == "QUIT" { break; }
        let tr = if sym == "QUIET" { vec![] } else if ALPHABET.iter().any(|(s, _)| *s == sym) { symbol_trains(&sym, seed) }
            else { println!("unknown: {sym}"); continue };
        println!("input spikes this beat: {} (A/C/D=8ch x 20Hz; QUIET=0)", tr.len());
        // two-beat probe: beat 1 = settle (builds on slow tau~5s dynamics),
        // beat 2 = the decoded response. Restart tick from load marker.
        tick = Tick(tick0);
        let mut out = vec![0.0f32; 12];
        for beat in 0..2u32 {
            for t in 0..BEAT_MS {
                let frame = InputFrame { tick, spikes: tr.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect() };
                let ev = net.step(&frame);
                if beat == 1 { for c in &ev.spikes { if (64..76).contains(&c.0) { out[(c.0 - 64) as usize] += 1.0; } } }
                tick = Tick(tick.0 + 1);
            }
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