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
            let v = capture_ref(&mut net, seed, mode, s, &p, &mut tr);
            (s.to_string(), v[..12].to_vec()) // identity = first 12 dims only (flag-independent)
        }).collect();
        let beat_n = 3u64;
        let mut rec: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
        let mut pres: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
        let mut novel_det = 0u32;
        let mut reflex_det = 0u32;
        let mut known_fp = 0u32;
        for _ in 0..beat_n {
            for s in stage {
                let act = decode_beat(&mut net, seed, mode, s, &p, &mut tr, &refs);
                if act == *s { *rec.entry(s.to_string()).or_insert(0) += 1; }
                *pres.entry(s.to_string()).or_insert(0) += 1;
                gap(&mut net, &p, &mut tr, 1500);
            }
            let act_d = decode_beat(&mut net, seed, mode, "D", &p, &mut tr, &refs);
            if act_d == "NOVEL" || act_d == "UNSURE" { novel_det += 1; }
            // D-58 CONTROL (registered falsifier's knowns-arm): present
            // each KNOWN symbol; the reflex must NOT flag it novel vs
            // its conventionuted refs (false positive -> rule measures
            // difference, not novelty). Count false_known_novel per beat.
            if std::env::var("D58_REFLEX").is_ok() {
                let th_fam_ctl: f32 = std::env::var("D58_TH").ok().and_then(|v| v.parse().ok()).unwrap_or(60.0);
                // knowns-control: each known vs PER-BEAT templates (incl.
                // its own, captured fresh NOW - state-matched, D-55 rule;
                // stage-frozen templates drift as the net keeps running).
                // ONE per-beat template set, shared by both falsifier arms
                // (captured before either arm's presentations -> both arms
                // scored against the SAME net state).
                let tpl: Vec<(String, Vec<f32>)> = stage.iter().map(|s| {
                    let v = capture_ref(&mut net, seed, mode, s, &p, &mut tr);
                    (s.to_string(), v[12..].to_vec())
                }).collect();
                for ks in stage {
                    let kv = capture_ref(&mut net, seed, mode, ks, &p, &mut tr);
                    let kn2 = kv[12..].to_vec();
                    let min_l = tpl.iter().map(|(_, t)| l2(&kn2, t)).fold(f32::MAX, f32::min);
                    if std::env::var("D58_DEBUG").is_ok() {
                        eprintln!("  D58 knowns-fp({}): min-L2={:.1} flagged={}", ks, min_l, min_l > th_fam_ctl);
                    }
                    if min_l > th_fam_ctl { known_fp += 1; }
                }
            }
            // D-58 verified rule (e918550): NOVEL iff min-L2 distance to
            // the nearest known template EXCEEDS th_fam. Measured:
            // known self-distance 2-21, D distance 115-179 -> th 60
            // separates. Cosine FAILED (D is same shape, lower magnitude
            // - angle preserves direction, discards magnitude); L2 is
            // magnitude-aware and is THE registered rule.
            if std::env::var("D58_REFLEX").is_ok() {
                let th_fam: f32 = std::env::var("D58_TH").ok().and_then(|v| v.parse().ok()).unwrap_or(60.0);
                // D arm: novel iff min-L2(D, PER-BEAT templates) > th_fam
                let tpl2: Vec<(String, Vec<f32>)> = stage.iter().map(|s| {
                    let v = capture_ref(&mut net, seed, mode, s, &p, &mut tr);
                    (s.to_string(), v[12..].to_vec())
                }).collect();
                let dv = capture_ref(&mut net, seed, mode, "D", &p, &mut tr);
                let dn = dv[12..].to_vec();
                let min_dl2 = tpl2.iter().map(|(_, t)| l2(&dn, t)).fold(f32::MAX, f32::min);
                let novel_by_reflex = min_dl2 > th_fam;
                if novel_by_reflex { reflex_det += 1; }
                if std::env::var("D58_DEBUG").is_ok() {
                    eprintln!("  D58 reflex-novelty(D): min-L2={:.1} th={} novel={}", min_dl2, th_fam, novel_by_reflex);
                }
            }
            gap(&mut net, &p, &mut tr, 1500);
        }
        if std::env::var("D58_DEBUG").is_ok() && std::env::var("D58_REFLEX").is_ok() {
            for s in stage {
                let v = capture_ref(&mut net, seed, mode, s, &p, &mut tr);
                let nv = &v[12..];
                eprintln!("  D58[{}] novelty-nodes: {:?}", s, nv);
            }
            let dv = capture_ref(&mut net, seed, mode, "D", &p, &mut tr);
            eprintln!("  D58[D] novelty-nodes: {:?}", &dv[12..]);
        }
        let mut line = format!("  stage{} vocab={:?} ", si + 1, stage);
        for s in known_all {
            if stage.contains(&s) {
                let ok = *rec.get(s).unwrap_or(&0);
                let tot = *pres.get(s).unwrap_or(&1);
                line.push_str(&format!("{s}={ok}/{tot} "));
            } else { line.push_str(&format!("{s}=UNTRAINED ")); }
        }
        line.push_str(&format!("novel_det={}/{} reflex_det={}/{} known_fp={}/{}", novel_det, beat_n, reflex_det, beat_n, known_fp, beat_n * stage.len() as u64));
        println!("{line}");
    }
    println!("done");
}

fn l2(a: &[f32], b: &[f32]) -> f32 {
    let mut s = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) { let d = x - y; s += d * d; }
    s.sqrt()
}
fn cos_centered(a: &[f32], b: &[f32]) -> f32 {
    // Centered cosine: subtract the common per-node mean across the two
    // vectors so saturated-common components (shared ~500 nodes) cancel
    // and discriminative nodes drive the angle. D-58 correction: raw
    // cosine was dominated by common saturation -> D looked familiar.
    let mut ma = 0.0f32; let mut mb = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) { ma += *x; mb += *y; }
    let na = a.len() as f32; let nb = b.len() as f32;
    if na == 0.0 || nb == 0.0 { return 0.0; }
    ma /= na; mb /= nb;
    let mut num = 0.0f32; let mut da = 0.0f32; let mut db = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        let ax = x - ma; let by = y - mb;
        num += ax * by; da += ax * ax; db += by * by;
    }
    if da == 0.0 || db == 0.0 { return 0.0; }
    num / (da.sqrt() * db.sqrt())
}
fn kn() -> usize {
    std::env::var("D58_K").ok().and_then(|v| v.parse().ok()).unwrap_or(2)
}
fn capture_ref(net: &mut Network, seed: u64, mode: &str, sym: &str, _p: &StdpParams, _tr: &mut Traces) -> Vec<f32> {
    let st = io::symbol_trains_mode(sym, mode, seed);
    let mut out = vec![0.0f32; 12];
    let rfx = std::env::var("D58_REFLEX").is_ok();
    let mut nov = vec![0.0f32; if rfx { kn() } else { 0 }];
    for t in 0..io::BEAT_MS {
        let f = InputFrame { tick: net.tick, spikes: st.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect() };
        let e = net.step(&f);
        for c in &e.spikes {
            let ci = c.0 as usize;
            if (io::OUTPUT_LO..io::OUTPUT_HI).contains(&c.0) { out[(ci - io::OUTPUT_LO as usize) as usize] += 1.0; }
            else if rfx && ci >= 76 && ci < 76 + kn() { nov[ci - 76] += 1.0; }
        }
        net.tick = Tick(net.tick.0 + 1);
    }
    if rfx { { let mut v = out; v.extend(nov); v } } else { out }
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
        tau_het_rel_sd: 0.0, phi_rel: 0.5, eta_rel: 0.0, v2: Some(v2p()),
        d58_reflex: if std::env::var("D58_REFLEX").is_ok() {
            std::env::var("D58_K").ok().and_then(|v| v.parse().ok()).unwrap_or(2)
        } else { 0 },
        ..Default::default()
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