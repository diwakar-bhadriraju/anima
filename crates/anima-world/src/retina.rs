//! Retina encoder: deterministic filterbank (luminance + local contrast)
//! -> per-cell Poisson spike trains over BEAT_MS, plus 8 proprioception
//! channels. Implements `anima_exp::encoders::SensoryEncoder` so it is a
//! drop-in at the survival-loop train site (D-59c interface).
//!
//! Emission conventions are IDENTICAL to io::symbol_trains_mode: FNV-1a
//! name hash, splitmix64-style derive of the per-channel seed, a
//! Xoshiro256PlusPlus per channel, Poisson gaps, sorted by tick. Same
//! seed + same world state -> identical trains (D-70.1 determinism).

use anima_core::network::InputChannelId;
use anima_exp::encoders::SensoryEncoder;
use anima_exp::io;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::camera::{Frame, RETINA_COLS, RETINA_ROWS};
use crate::world::World;

pub const VISION_CHANNELS: usize = RETINA_COLS * RETINA_ROWS; // 48
pub const PROPRIO_CHANNELS: usize = 8;
/// Rate ceiling for a fully lit high-contrast cell (Hz), frozen D-70.1.
pub const RATE_MAX_HZ: f32 = 40.0;
/// Base rate floor so silent cells still carry a token of input (Hz).
pub const RATE_FLOOR_HZ: f32 = 2.0;

/// Poisson emission over one BEAT_MS window for one channel at a rate.
fn poisson_trains(seed: u64, cell_name: &str, cell: u64, rate_hz: f32) -> Vec<(u64, InputChannelId)> {
    let master = seed;
    let ch_seed = io::derive_seed64(master, io::hash_str("retina"), io::hash_str(cell_name) ^ cell);
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(ch_seed);
    let lambda = rate_hz / 1000.0;
    let mut out = Vec::new();
    let mut t_off: u64 = 0;
    loop {
        let u: f32 = rng.gen::<f32>().max(1e-6);
        let gap = ((-(1.0 - u as f64).ln()) / lambda as f64).ceil() as u64;
        t_off += gap.max(1);
        if t_off >= io::BEAT_MS {
            break;
        }
        out.push((t_off, InputChannelId(cell as u32)));
    }
    out
}

/// The retina encoder: snapshot of one beat's frame + proprioception.
/// Constructed per beat from the world (the world is static within a
/// beat; the encoder is a pure function of (world state, seed)).
#[derive(Debug, Clone)]
pub struct RetinaEncoder {
    /// Per-cell rate_hz: vision cells 0..48 then proprio cells 48..56.
    rates_hz: Vec<f32>,
}

impl RetinaEncoder {
    /// Encode the world's current state into per-cell rates.
    /// rate = floor + (RATE_MAX - floor) * lum * (1 + 0.5 * contrast),
    /// clipped to [RATE_FLOOR_HZ, RATE_MAX_HZ]. Proprio cells: constant
    /// rates encoding [yaw bins(4), pitch(1), speed(2), altitude(1)].
    pub fn from_world(world: &World) -> Self {
        let frame: Frame = crate::camera::rasterize(world);
        let mut rates = Vec::with_capacity(VISION_CHANNELS + PROPRIO_CHANNELS);
        for i in 0..frame.cells() {
            let lum = frame.luminance[i];
            let con = frame.contrast[i];
            let r = RATE_FLOOR_HZ + (RATE_MAX_HZ - RATE_FLOOR_HZ) * lum * (1.0 + 0.5 * con);
            rates.push(r.clamp(RATE_FLOOR_HZ, RATE_MAX_HZ));
        }
        // Proprioception: yaw quantized to 4 bins (constant rate 25 Hz on
        // the active bin cell), pitch sign (active cell 20 Hz), speed two
        // bins (30 Hz on the active one), altitude bin (18 Hz when above
        // ARENA_Z_MAX/2).
        let mut p = [0.0f32; PROPRIO_CHANNELS];
        let mut yaw_norm = world.body.yaw / std::f32::consts::PI * 0.5 + 0.5; // [0,1)
        if yaw_norm < 0.0 { yaw_norm += 1.0; }
        if yaw_norm >= 1.0 { yaw_norm -= 1.0; }
        p[(yaw_norm * 4.0) as usize % 4] = 25.0;
        p[4] = if world.body.pitch > 0.01 { 20.0 } else if world.body.pitch < -0.01 { 10.0 } else { 5.0 };
        let speed = (world.body.vel.x * world.body.vel.x
            + world.body.vel.y * world.body.vel.y
            + world.body.vel.z * world.body.vel.z).sqrt();
        p[5] = if speed > 10.0 { 30.0 } else if speed > 1.0 { 15.0 } else { 5.0 };
        p[6] = 30.0; // (reserved, fixed)
        p[7] = if world.body.pos.y > crate::world::ARENA_Z_MAX * 0.5 { 18.0 } else { 6.0 };
        for (i, r) in p.iter().enumerate() {
            let r = r.clamp(RATE_FLOOR_HZ, RATE_MAX_HZ);
            rates.push(r);
        }
        Self { rates_hz: rates }
    }
}

impl SensoryEncoder for RetinaEncoder {
    fn channel_count(&self) -> usize {
        VISION_CHANNELS + PROPRIO_CHANNELS
    }

    /// One BEAT_MS window of spike trains, deterministic in
    /// (world state, seed). Vision cells use "v:{col}:{row}", proprio
    /// cells "p:{i}" as names in the seed derivation.
    fn frames(&self, seed: u64) -> Vec<(u64, InputChannelId)> {
        let mut out = Vec::new();
        for c in 0..VISION_CHANNELS {
            let col = c % RETINA_COLS;
            let row = c / RETINA_COLS;
            out.extend(poisson_trains(seed, &format!("v:{col}:{row}"), c as u64, self.rates_hz[c]));
        }
        for i in 0..PROPRIO_CHANNELS {
            out.extend(poisson_trains(seed, &format!("p:{i}"), (VISION_CHANNELS + i) as u64, self.rates_hz[VISION_CHANNELS + i]));
        }
        out.sort_unstable_by_key(|(t, _)| *t);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::World;

    #[test]
    fn determinism_trains_same_seed() {
        let w = World::new(21);
        let enc = RetinaEncoder::from_world(&w);
        assert_eq!(enc.frames(1), enc.frames(1), "same seed+state -> identical trains");
    }

    #[test]
    fn seed_sensitivity_trains() {
        let w = World::new(21);
        let enc = RetinaEncoder::from_world(&w);
        assert_ne!(enc.frames(1), enc.frames(2), "different seeds -> different trains");
    }

    #[test]
    fn channel_count_and_bounds() {
        let w = World::new(21);
        let enc = RetinaEncoder::from_world(&w);
        assert_eq!(enc.channel_count(), 56);
        assert!(enc.rates_hz.iter().all(|&r| (RATE_FLOOR_HZ..=RATE_MAX_HZ).contains(&r)));
        let tr = enc.frames(5);
        for (t, c) in &tr {
            assert!(*t < io::BEAT_MS, "tick must be inside the beat");
            assert!(c.0 < 56, "channel must be in range");
        }
        let mut sorted = tr.clone();
        sorted.sort_unstable_by_key(|(t, _)| *t);
        assert_eq!(tr, sorted, "trains must be sorted by tick");
    }
}