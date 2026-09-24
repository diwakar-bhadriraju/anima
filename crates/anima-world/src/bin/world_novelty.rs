//! D-70.3 world-novelty falsifier (docs/phase3/d70-3-world-novelty-protocol.md):
//! the D-70.2 loop + the reflex band on the 56 retina channels with the
//! D-61/D-63 readout exclusivity + D-62 band start-state parity stack
//! ACTIVE (V2Plasticity runs on the pool; the band is excluded; births
//! are OFF for stage 1), a rolling W=5 recent-beat template window,
//! novel-object first-sight at beat T1 (camera-relative placement), and
//! physical motor fault at T2.
//!
//! Usage: world_novelty [world_seed] [net_seed] [k]
//! constants frozen: TH=60, W=5, T1=40, T2=50, beats=60.
//! env: WN_VERBOSE=1 (per-beat), WN_PARITY=0 (disable band parity;
//! control), WN_FAULT=0 (disable fault; control), D61_EXCL=0 (disable
//! the V2 readout exclusivity; control).

use anima_core::network::{InputFrame, Network, NetworkConfig, Tick};
use anima_core::plasticity::{stdp_tick, StdpParams, Traces};
use anima_core::structural_v2::V2Plasticity;
use anima_exp::encoders::SensoryEncoder;
use anima_exp::io;
use anima_world::world::Vec3;
use anima_world::{MotorDrive, RetinaEncoder, World};

const TH: f32 = 60.0;
const W: usize = 5;
const T1: u64 = 40; // novel first-sight
const T2: u64 = 50; // motor fault
const BEATS: u64 = 60;
const FIRST_SIGHT_WINDOW: u64 = 3; // beats T1..T1+2 assessed in S1b

fn main() {
    let world_seed: u64 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(20260924);
    let net_seed: u64 = std::env::args().nth(2).and_then(|a| a.parse().ok()).unwrap_or(77);
    let k: usize = std::env::args().nth(3).and_then(|a| a.parse().ok()).unwrap_or(12);
    let verbose = std::env::var("WN_VERBOSE").is_ok();
    let parity = !std::env::var("WN_PARITY").map(|v| v == "0").unwrap_or(false);
    let fault = !std::env::var("WN_FAULT").map(|v| v == "0").unwrap_or(false);

    let mut world = World::new(world_seed);
    // The D-70.1 scene has 5 primitives: the 4 KNOWN ones stay; the
    // 5th (yellow pyramid) becomes the NOVEL object placed at T1.
    let novel = world.primitives.pop().expect("scene has 5 primitives");
    assert_eq!(novel.color, [0.9, 0.85, 0.2], "5th primitive is the yellow pyramid");

    let n_in = 56usize;
    let n_int = 40usize;
    let n_out = 12usize;
    let out_lo = n_in + n_int;
    let out_hi = out_lo + n_out;
    let band_lo = out_hi;
    let cfg = NetworkConfig {
        connectivity: 0.038, w_init: 0.2, amplitude: 52.0, adaptation_tau_ms: 200.0,
        adaptation_gain: 0.05, inhibition_gain: 0.0, slow_state_beta: 0.0046875,
        slow_state_tau_ms: 5000.0, slow_state_beta_drive: false, latch_enable: true,
        theta_rel_mean: 1.0, theta_rel_sd: 0.0, u_plateau_rel_mean: 1.0, u_plateau_rel_sd: 0.0,
        tau_het_rel_sd: 0.0, phi_rel: 0.5, eta_rel: 0.0,
        d58_reflex: k,
        ..NetworkConfig::default()
    };
    let mut net = Network::new(cfg, n_in, n_int, n_out, net_seed);
    let p = StdpParams {
        tau_plus: 20.0, tau_minus: 20.0, a_plus: 0.005, a_minus: 0.0053,
        decay: 1e-6, w_min: 0.0, w_max: 1.0,
    };
    let mut traces = Traces::new(&net, 20.0);
    // V2Plasticity ON (pool learning); the band is read-only via the
    // D-61/D-63 exclusivity (env D61_EXCL default active). Births OFF.
    let mut v2 = V2Plasticity::new(&mut net, evolve_v2_params(), None);

    // Rolling template window: band responses of the last W beats.
    // ORDERING RULE (D-59): judge the beat BEFORE pushing it.
    let mut window: Vec<Vec<f32>> = Vec::new();
    let mut novel_pos: Option<Vec3> = None;
    let mut known_fp = 0u32;
    let mut known_n = 0u32;
    let mut novel_seen = 0u32;
    let mut novel_n = 0u32;
    let mut cons_novel = 0u32;
    let mut cons_n = 0u32;

    for beat in 0..BEATS {
        // T1: place the novel object at the CURRENT CAMERA's forward
        // offset (dist 20, on-axis) — first-sight guaranteed, and a
        // deterministic function of the run.
        if beat == T1 {
            let (sy, cy) = world.body.yaw.sin_cos();
            let (sp, cp) = world.body.pitch.sin_cos();
            let fwd = Vec3::new(sy * cp, sp, cy * cp);
            let mut nv = novel;
            nv.pos = Vec3::new(
                world.body.pos.x + fwd.x * 20.0,
                world.body.pos.y + fwd.y * 20.0,
                world.body.pos.z + fwd.z * 20.0,
            );
            novel_pos = Some(nv.pos);
            world.primitives.push(nv);
        }
        let enc = RetinaEncoder::from_world(&world);
        let train = enc.frames(world_seed);
        let mut out = vec![0.0f32; 12];
        let mut rv = vec![0.0f32; k];
        for t in 0..io::BEAT_MS {
            let frame = InputFrame {
                tick: net.tick,
                spikes: train.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect(),
            };
            let ev = net.step(&frame);
            for c in &ev.spikes {
                let ci = c.0 as usize;
                if ci >= out_lo && ci < out_hi {
                    out[ci - out_lo] += 1.0;
                } else if ci >= band_lo && ci < band_lo + k {
                    rv[ci - band_lo] += 1.0;
                }
            }
            traces.step(&net, &ev.spikes);
            let _ = stdp_tick(&p, &mut net, &traces, &ev.spikes, 1.0, None);
            // V2 pool machinery (window cadence like the survival loop).
            v2.tick(&ev.spikes);
            if net.tick.0 % 100 == 0 && net.tick.0 > 0 {
                let tk = net.tick;
                let _ = v2.window(&mut net, tk);
            }
            net.tick = Tick(net.tick.0 + 1);
        }
        // Verdict BEFORE window update (ordering rule, D-59).
        let min_l = if window.is_empty() {
            f32::MAX
        } else {
            window.iter().map(|w| reflex_l2(&rv, w)).fold(f32::MAX, f32::min)
        };
        let flagged = min_l > TH;
        if beat >= T1 && beat < T1 + FIRST_SIGHT_WINDOW {
            novel_n += 1;
            if flagged {
                novel_seen += 1;
            }
        } else if beat > 0 && beat < T1 {
            known_n += 1;
            if flagged {
                known_fp += 1;
            }
        }
        // Motor -> body; fault at T2 reverses the DEMAND (physical:
        // the body moves opposite -> the next beat's scene reverses).
        let mc = MotorDrive::from_counts(&out);
        let negate = fault && beat >= T2;
        let rates: Vec<f32> = mc.rates_hz.iter().map(|r| if negate { -r } else { *r }).collect();
        world.step(&rates);
        // Consequence check (Stage 2): post-fault beats whose scene
        // (under reversed motion) violates recent experience.
        if fault && beat >= T2 && !window.is_empty() {
            cons_n += 1;
            if flagged {
                cons_novel += 1;
            }
        }
        if verbose {
            let b = &world.body;
            let drive = mc.rates_hz.iter().sum::<f32>() / mc.rates_hz.len().max(1) as f32 / 1000.0;
            // Novel-object visibility: direction from the camera to the
            // placed object (az/el relative to camera forward, distance).
            let (s_az, s_el, s_d) = if let Some(np) = novel_pos {
                let (sy, cy) = b.yaw.sin_cos();
                let (sp, cp) = b.pitch.sin_cos();
                let fwd = Vec3::new(sy * cp, sp, cy * cp);
                let right = Vec3::new(cy, 0.0, -sy);
                let up = Vec3::new(0.0, 1.0, 0.0);
                let d = Vec3::new(np.x - b.pos.x, np.y - b.pos.y, np.z - b.pos.z);
                let dist = (d.x * d.x + d.y * d.y + d.z * d.z).sqrt();
                let dn = if dist > 1e-6 { Vec3::new(d.x / dist, d.y / dist, d.z / dist) } else { Vec3::ZERO };
                let az = (dn.x * right.x + dn.z * right.z).atan2(dn.x * fwd.x + dn.y * fwd.y + dn.z * fwd.z);
                let el = (dn.y).asin();
                (az, el, dist)
            } else { (0.0f32, 0.0f32, -1.0f32) };
            eprintln!("WN beat={} minL2={:.1} flag={} pos=({:.2},{:.2},{:.2}) drive={:.3} rv={:?} obj_az={:.2} obj_el={:.2} obj_d={:.1}",
                beat, min_l, flagged as u8, b.pos.x, b.pos.y, b.pos.z, drive, rv, s_az, s_el, s_d);
        }
        // Window update AFTER the verdict (recent experience).
        window.push(rv);
        if window.len() > W {
            window.remove(0);
        }
        // Band start-state parity (D-62 stack; WN_PARITY=0 = control).
        if parity {
            let v_rest = net.cfg.lif.v_rest;
            for n in net.neurons.iter_mut().skip(band_lo).take(k) {
                n.u_slow = 0.0;
                n.v = v_rest;
                n.z_latch = 0;
                n.i_syn = 0.0;
            }
        }
    }
    println!("WN RESULT k={} known_fp={}/{} novel_seen={}/{} cons_novel={}/{} pos=({:.3},{:.3},{:.3})",
        k, known_fp, known_n, novel_seen, novel_n, cons_novel, cons_n,
        world.body.pos.x, world.body.pos.y, world.body.pos.z);
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

/// V2 params mirroring evolve's v2_params() (frozen D-70.3).
fn evolve_v2_params() -> anima_core::network::V2Params {
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