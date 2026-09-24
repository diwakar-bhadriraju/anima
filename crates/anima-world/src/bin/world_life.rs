//! D-70.2 motion closed loop (docs/phase3/d70-2-motion-closed-loop-protocol.md):
//! retina -> network (STDP on, V2 off) -> motor -> body -> next beat's scene.
//! Deterministic: same (world_seed, net_seed) -> bit-identical telemetry.
//!
//! Usage: world_life [world_seed] [net_seed] [beats]
//! frozen diagnostics: WL_VERBOSE=1 (per-beat telemetry),
//! WL_MOTOR_OFF=1 (zero motor rates -> body must stay put, A3).

use anima_core::network::{InputFrame, Network, NetworkConfig, Tick};
use anima_core::plasticity::{stdp_tick, StdpParams, Traces};
use anima_exp::encoders::SensoryEncoder;
use anima_exp::io;
use anima_world::camera::rasterize;
use anima_world::{MotorDrive, RetinaEncoder, World};

fn main() {
    let world_seed: u64 = std::env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(20260924);
    let net_seed: u64 = std::env::args().nth(2).and_then(|a| a.parse().ok()).unwrap_or(77);
    let beats: u64 = std::env::args().nth(3).and_then(|a| a.parse().ok()).unwrap_or(60);
    let verbose = std::env::var("WL_VERBOSE").is_ok();
    let motor_off = std::env::var("WL_MOTOR_OFF").is_ok();

    let mut world = World::new(world_seed);
    let start_pos = world.body.pos;
    let n_in = 56usize;
    let n_int = 40usize;
    let n_out = 12usize;
    // Output band = inputs + internals .. +outputs (NOT 64..76: that is
    // the 24-input survival anatomy; with 56 inputs the outputs sit at
    // 96..108).
    let out_lo = n_in + n_int;
    let out_hi = out_lo + n_out;
    let cfg = NetworkConfig {
        connectivity: 0.038, w_init: 0.2, amplitude: 52.0, adaptation_tau_ms: 200.0,
        adaptation_gain: 0.05, inhibition_gain: 0.0, slow_state_beta: 0.0046875,
        slow_state_tau_ms: 5000.0, slow_state_beta_drive: false, latch_enable: true,
        theta_rel_mean: 1.0, theta_rel_sd: 0.0, u_plateau_rel_mean: 1.0, u_plateau_rel_sd: 0.0,
        tau_het_rel_sd: 0.0, phi_rel: 0.5, eta_rel: 0.0,
        ..NetworkConfig::default()
    };
    let mut net = Network::new(cfg, n_in, n_int, n_out, net_seed);
    let p = StdpParams {
        tau_plus: 20.0, tau_minus: 20.0, a_plus: 0.005, a_minus: 0.0053,
        decay: 1e-6, w_min: 0.0, w_max: 1.0,
    };
    let mut traces = Traces::new(&net, 20.0);

    let mut prev_lum: Option<Vec<f32>> = None;
    let mut frames_changed = 0u32;
    for beat in 0..beats {
        // 1-2. Scene -> encoder -> spike trains (scene static in-beat).
        let enc = RetinaEncoder::from_world(&world);
        let train = enc.frames(world_seed);
        // 3. Network over the beat, STDP on.
        let mut out = vec![0.0f32; 12];
        for t in 0..io::BEAT_MS {
            let frame = InputFrame {
                tick: net.tick,
                spikes: train.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect(),
            };
            let ev = net.step(&frame);
            for c in &ev.spikes {
                if c.0 >= out_lo as u32 && c.0 < out_hi as u32 {
                    out[(c.0 - out_lo as u32) as usize] += 1.0;
                }
            }
            traces.step(&net, &ev.spikes);
            let _ = stdp_tick(&p, &mut net, &traces, &ev.spikes, 1.0, None);
            net.tick = Tick(net.tick.0 + 1);
        }
        // 4-5. Motor -> body -> world (the NEXT beat's scene moves).
        if motor_off {
            world.step(&[0.0f32; 12]);
        } else {
            let mc = MotorDrive::from_counts(&out);
            world.step(&mc.rates_hz);
        }
        // frame_changed: the organism's motion changed the camera view vs
        // the previous beat (A2; A3 with WL_MOTOR_OFF=1 asserts it stays
        // 0). Computed unconditionally (summary needs it).
        let lum = rasterize(&world).luminance;
        let changed = prev_lum.as_ref().is_some_and(|pf| {
            pf.iter().zip(lum.iter()).any(|(a, b2)| (a - b2).abs() > 1e-3)
        });
        if changed {
            frames_changed += 1;
        }
        prev_lum = Some(lum);
        if verbose {
            let b = &world.body;
            let speed = (b.vel.x * b.vel.x + b.vel.y * b.vel.y + b.vel.z * b.vel.z).sqrt();
            let out_sum: f32 = out.iter().sum();
            eprintln!("WL beat={} pos=({:.2},{:.2},{:.2}) yaw={:.2} pitch={:.2} speed={:.2} out_sum={:.0} frame_changed={}",
                beat, b.pos.x, b.pos.y, b.pos.z, b.yaw, b.pitch, speed, out_sum,
                if changed { 1 } else { 0 });
        }
    }
    // Unconditional machine-checkable summary (A2/A3).
    let b = &world.body;
    let moved = (b.pos.x - start_pos.x).abs()
        + (b.pos.y - start_pos.y).abs()
        + (b.pos.z - start_pos.z).abs()
        > 1e-3;
    println!("WL RESULT beats={} moved={} frames_changed={} pos=({:.3},{:.3},{:.3})",
        beats, if moved { 1 } else { 0 }, frames_changed, b.pos.x, b.pos.y, b.pos.z);
}