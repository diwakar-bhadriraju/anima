//! D-59 shared reflex-novelty rule (docs/phase3/d58-novelty-reflex-protocol.md).
//! Promoted from the library.rs prototype into a shared module so the
//! survival loop can run the SAME verified rule in-life (D-59: new-vs-known
//! during real closed-loop life, not only in the prototype harness).
//!
//! Verified rule (D-58 final tables): NOVEL iff
//! min-L2(D-signature, per-beat known templates) > th_fam. REFLEX_K = 8
//! (k=2 collides, k=16 re-collides - non-monotone), th_fam = 60.0.
//! Templates must be captured PER-BEAT state-matched (stage-frozen
//! regresses); L2 not cosine (magnitude-aware).
//!
//! The reflex band: K output-class neurons appended AFTER the identity band
//! at construction (NetworkConfig.d58_reflex), each with FIXED seeded
//! non-plastic synapses from every input channel (content-neutral - the
//! nodes respond to input-drive SIGNATURE, not symbol identity). Band
//! position is derived from the LIVE network (the last K neurons at first
//! use, before any mid-life births), so any n_internal or size drift cannot
//! shift it; births append after the band.
use anima_core::network::{InputFrame, Network, Tick};

use crate::io;

/// Verified optimum reflex node count (d58 sweep: k=2 collides, k=16
/// re-collides; K=8 separates with the per-beat template rule). Replaces
/// the prototype's kn() default of 2.
pub const REFLEX_K: usize = 8;
/// Verified familiarity threshold: min-L2 > REFLEX_TH_FAM => NOVEL.
pub const REFLEX_TH_FAM: f32 = 60.0;

/// L2 distance (magnitude-aware; NOT cosine - the D-58 measured failure
/// mode was cosine's magnitude blindness).
pub fn l2(a: &[f32], b: &[f32]) -> f32 {
    let mut s = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        let d = x - y;
        s += d * d;
    }
    s.sqrt()
}

/// Present one 500ms beat of `sym` (no STDP - probing must not learn) and
/// return the K-dim reflex-band spike-count vector.
///
/// Band = the last `k` neurons at first use (the reflex nodes appended at
/// construction). `k` MUST equal the network's `cfg.d58_reflex`; caller
/// guarantees it (prototype harness only; run_world_full does NOT use this
/// path - it rides the real beats).
pub fn signature(net: &mut Network, seed: u64, mode: &str, sym: &str, k: usize) -> Vec<f32> {
    debug_assert!(k > 0 && k == net.cfg.d58_reflex);
    if k == 0 {
        return Vec::new();
    }
    let base = net.neurons.len() - k;
    let st = io::symbol_trains_mode(sym, mode, seed);
    let mut nov = vec![0.0f32; k];
    for t in 0..io::BEAT_MS {
        let f = InputFrame {
            tick: net.tick,
            spikes: st.iter().filter(|(tt, _)| *tt == t).map(|(_, c)| *c).collect(),
        };
        let e = net.step(&f);
        for c in &e.spikes {
            let ci = c.0 as usize;
            if ci >= base && ci < base + k {
                nov[ci - base] += 1.0;
            }
        }
        net.tick = Tick(net.tick.0 + 1);
    }
    nov
}

/// The verified novelty rule packaged: capture per-beat templates from
/// `knowns` (state-matched, in order), then the probe's signature; NOVEL
/// iff min-L2(probe, templates) > th.
pub fn novelty_verdict(
    net: &mut Network,
    seed: u64,
    mode: &str,
    knowns: &[&str],
    probe_sym: &str,
    k: usize,
    th: f32,
) -> bool {
    let tpl: Vec<Vec<f32>> = knowns.iter().map(|s| signature(net, seed, mode, s, k)).collect();
    let pv = signature(net, seed, mode, probe_sym, k);
    let min_l = tpl.iter().map(|t| l2(&pv, t)).fold(f32::MAX, f32::min);
    min_l > th
}