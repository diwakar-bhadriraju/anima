//! Phase III I/O codebook (docs/phase3/io-encoder-decoder-plan.md D-25/D-26).
//! VERIFIED pieces extracted verbatim from examples/talk.rs (which took
//! ~6 failed attempts): these are the tested implementations, do NOT
//! rewrite. Used by the survival loop (recognition-rate) and the
//! closed-loop env.
use anima_core::network::{InputChannelId, InputFrame, Tick};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

pub const BEAT_MS: u64 = 500;
pub const OUTPUT_LO: u32 = 64;
pub const OUTPUT_HI: u32 = 76;
/// Input alphabet (symbol -> exclusive channel subset), MODE-DRIVEN.
/// Legacy mode (D-50 default): A/C 8-channel trained, D 8-channel NOVEL.
/// D50 mode: 3 trained (A/C/E) + D NOVEL, 6 channels each (24 total),
/// the ONLY change being symbol count at matched size.
pub fn alphabet(mode: &str) -> Vec<(&'static str, &'static [u32])> {
    match mode {
        "d50-2" => vec![
            ("A", &[0, 1, 2, 3, 4, 5]),
            ("C", &[6, 7, 8, 9, 10, 11]),
            ("D", &[12, 13, 14, 15, 16, 17]), // never-trained NOVEL
        ],
        "d50" => vec![
            ("A", &[0, 1, 2, 3, 4, 5]),
            ("C", &[6, 7, 8, 9, 10, 11]),
            ("E", &[12, 13, 14, 15, 16, 17]),
            ("D", &[18, 19, 20, 21, 22, 23]), // never-trained NOVEL probe
        ],
        _ => vec![
            ("A", &[0, 1, 2, 3, 4, 5, 6, 7]),
            ("C", &[8, 9, 10, 11, 12, 13, 14, 15]),
            ("D", &[16, 17, 18, 19, 20, 21, 22, 23]), // never-trained NOVEL
        ],
    }
}
/// Trained (non-novel) symbols for the active mode.
pub fn known_syms(mode: &str) -> Vec<&'static str> {
    alphabet(mode).iter().filter(|(s, _)| *s != "D").map(|(s, _)| *s).collect()
}
pub const RATE_HZ: f32 = 20.0;

/// Verified name-hash (FNV-1a) used by the train derivers; pub so the
/// world crate's retina encoder uses the SAME convention (determinism
/// across crates).
pub fn hash_str(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() { h ^= *b as u64; h = h.wrapping_mul(0x100000001b3); }
    h
}
/// Verified seed-derivation (splitmix64-style) used by the train
/// derivers; pub for the same convention-sharing reason.
pub fn derive_seed64(master: u64, a: u64, b: u64) -> u64 {
    let mut z = master.wrapping_add(a.wrapping_mul(0x9E3779B97F4A7C15)).wrapping_add(b);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB) ^ (z >> 31)
}

/// Deterministic Poisson trains for one symbol beat (same convention as
/// env.rs; verified in talk.rs). Empty for unknown/QUIET.
pub fn symbol_trains(sym: &str, seed: u64) -> Vec<(u64, InputChannelId)> {
    symbol_trains_mode(sym, "", seed)
}
/// mode-aware trains (D-50: mode selects channel layout). Empty mode =
/// legacy. eff=1.0 -> byte-identical to symbol_trains_mode (multiplying
/// RATE_HZ by 1.0 is exact).
pub fn symbol_trains_mode(sym: &str, mode: &str, seed: u64) -> Vec<(u64, InputChannelId)> {
    symbol_trains_mode_eff(sym, mode, seed, 1.0)
}
/// D-59 closed-loop motor world: `eff` scales the per-channel Poisson rate
/// of the beat's train (the world's sensed response to the previous
/// action). eff=1.0 = identity; consumers clamp.
pub fn symbol_trains_mode_eff(sym: &str, mode: &str, seed: u64, eff: f32) -> Vec<(u64, InputChannelId)> {
    let Some(chans) = alphabet(mode).iter().find(|(s, _)| *s == sym).map(|(_, c)| c.to_vec()) else { return vec![] };
    let lambda = RATE_HZ * eff / 1000.0;
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

/// D-59 (docs/phase3/d59-reflex-integration-protocol.md): motor command -
/// one rate per output neuron, LINEAR PROPORTIONAL mapping from a beat's
/// output spike counts ("how much that neuron activates, that much the
/// motor moves"). No thresholding; consumers clamp.
pub struct MotorCommand {
    pub rates_hz: Vec<f32>,
}

/// Rate-proportional motor readout: spike count over the 500ms beat -> Hz.
pub fn motor(out: &[f32]) -> MotorCommand {
    MotorCommand {
        rates_hz: out.iter().map(|x| x * 1000.0 / BEAT_MS as f32).collect(),
    }
}

/// Plain cosine (the outselect-validated 100% A/C convention), NOT
/// centered-cos - the validated decode used plain-cos + argmax.
pub fn cos(a: &[f32], b: &[f32]) -> f32 {
    let (mut n, mut na, mut nb) = (0.0f32, 0.0f32, 0.0f32);
    for (x, y) in a.iter().zip(b) {
        n += x * y; na += x * x; nb += y * y;
    }
    if na <= 0.0 || nb <= 0.0 { 0.0 } else { n / (na.sqrt() * nb.sqrt()) }
}

/// Decode a 12-dim output vector via the frozen codebook (D-25):
/// amp<q_floor=QUIET; plain-cos argmax to A/C if above th_known; else
/// NOVEL/UNSURE. q_floor/th_known frozen registration values.
pub fn decode(out: &[f32], refs: &[(String, Vec<f32>)], q_floor: f32, th_known: f32) -> String {
    if out.len() < (OUTPUT_HI - OUTPUT_LO) as usize { return "UNSURE".into(); }
    let amp = out.iter().sum::<f32>() / out.len() as f32;
    if amp < q_floor { return "QUIET".into(); }
    let mut best: Option<(f32, &str)> = None;
    for (p, r) in refs {
        let c = cos(out, r);
        if best.as_ref().map(|(bc, _)| c > *bc).unwrap_or(true) { best = Some((c, p)); }
    }
    if let Some((c, p)) = best {
        if c > th_known { return p.to_string(); }
        return if refs.is_empty() { "UNSURE".into() } else { "NOVEL".into() };
    }
    "UNSURE".into()
}
