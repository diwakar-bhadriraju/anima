//! ANIMA E6 rate balancing (docs/anima-e6-protocol.md §3, frozen).
//!
//! Per-channel event-rate estimate φ (EMA over structural windows) and the
//! local balance factor β = φ̄_afferents / φ_pre, applied at EXACTLY two
//! plasticity sites: the STDP a⁺ branch and the M3 co-active candidate
//! accumulation (both × β_pre). Everything else — LTD, M2, M4, M5, M6 —
//! is untouched.
//!
//! Frozen properties: φ is per-channel, history-derived (no labels, no
//! RNG), curriculum-blind, deterministic, one-window lagged; β is clamped
//! to [β_min, β_max]; when φ is constant β ≡ 1 (mechanism exactly inert).

use crate::network::{NeuronId, Network};

/// Frozen E6 parameters (docs/anima-e6-protocol.md §3).
#[derive(Debug, Clone)]
pub struct E6Params {
    /// EMA smoothing: φ ← (1−α)·φ + α·(c/W). Frozen α = 1/25.
    pub alpha: f32,
    /// Initial φ per channel. Frozen 0.02 events/tick (≈ 20 Hz).
    pub phi_init: f32,
    /// Floor for φ. Frozen 0.001 events/tick.
    pub phi_min: f32,
    /// β clamp bounds. Frozen [0.1, 10] (inert in this curriculum;
    /// natural range [0.5, 2]).
    pub beta_min: f32,
    pub beta_max: f32,
    /// Structural window length in ticks (frozen W = 100).
    pub window_ticks: u32,
    /// Number of input channels (ids 0..n_input are input neurons).
    pub n_input: usize,
}

impl E6Params {
    pub fn new(alpha: f32, phi_init: f32, phi_min: f32, beta_min: f32, beta_max: f32, window_ticks: u32, n_input: usize) -> Self {
        Self { alpha, phi_init, phi_min, beta_min, beta_max, window_ticks, n_input }
    }

    pub fn disabled(n_input: usize) -> Self {
        // Identity construction: α = 0 keeps φ at init forever ⇒ β ≡ 1.
        Self::new(0.0, 0.02, 0.001, 0.1, 10.0, 100, n_input)
    }
}

/// E6 per-channel rate estimate + β computation (mechanism state).
#[derive(Debug, Clone)]
pub struct RateBalance {
    params: E6Params,
    /// φ per input channel.
    pub(crate) phi: Vec<f32>,
    /// Spike-event counts in the current window, per channel.
    counts: Vec<u32>,
}

impl RateBalance {
    pub fn new(params: E6Params) -> Self {
        let n = params.n_input;
        let phi_init = params.phi_init;
        Self {
            params,
            phi: vec![phi_init; n],
            counts: vec![0; n],
        }
    }

    /// Read-only measurement access to the current φ estimates
    /// (analysis/tests; the mechanism never reads this path).
    pub fn phi(&self) -> &[f32] {
        &self.phi
    }

    /// Fast path: record one input-channel event per tick (channel id ==
    /// input neuron id; environment de-duplicates per tick, so this is a
    /// 0/1 count per channel per tick — the frozen counting semantics).
    pub fn tick(&mut self, spike: NeuronId) {
        let i = spike.idx();
        if i < self.params.n_input {
            self.counts[i] = self.counts[i].saturating_add(1);
        }
    }

    /// Window-start update (frozen): φ ← (1−α)·φ + α·(c/W), floored, then
    /// counts reset. Called once per structural window from the PREVIOUS
    /// window's counts (one-window lag: counts cover [tick−W, tick)).
    pub fn window_start(&mut self) {
        let w = self.params.window_ticks.max(1) as f32;
        for (i, phi_i) in self.phi.iter_mut().enumerate() {
            let c = self.counts[i] as f32 / w;
            let next = (1.0 - self.params.alpha) * *phi_i + self.params.alpha * c;
            *phi_i = next.max(self.params.phi_min);
            self.counts[i] = 0;
        }
    }

    /// β for one (pre → post) contact: β = φ̄_afferents(post) / φ_pre for
    /// input-channel pre; 1.0 otherwise. φ̄_afferents = mean φ over the
    /// post neuron's LIVE input-channel afferents (the same set definition
    /// for the M3 candidate site, per protocol §3.2). Degenerate guards
    /// (φ_pre or φ̄ ≤ 0) return 1.0 — the floor makes these unreachable in
    /// practice. Clamped to [β_min, β_max].
    pub fn beta(&self, net: &Network, post: NeuronId, pre: NeuronId) -> f32 {
        let pi = pre.idx();
        if pi >= self.params.n_input {
            return 1.0; // recurrent / inhibitory / output — not rate-balanced
        }
        let phi_pre = self.phi[pi];
        if phi_pre <= 0.0 {
            return 1.0;
        }
        let mut sum = 0.0f32;
        let mut n = 0usize;
        for &sid in &net.incoming[post.idx()] {
            let s = &net.synapses[sid.idx()];
            if s.silent_ticks == u64::MAX {
                continue;
            }
            let pi = s.pre.idx();
            if pi < self.params.n_input {
                sum += self.phi[pi];
                n += 1;
            }
        }
        if n == 0 || sum <= 0.0 {
            return 1.0;
        }
        let bar = sum / n as f32;
        (bar / phi_pre).clamp(self.params.beta_min, self.params.beta_max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::{Network, NetworkConfig, NeuronId, SynapseId, Tick};

    fn rb() -> RateBalance {
        RateBalance::new(E6Params::new(0.04, 0.02, 0.001, 0.1, 10.0, 100, 6))
    }

    fn net6() -> Network {
        Network::new(NetworkConfig::default(), 6, 4, 2, 7)
    }

    /// §3.1: φ initialized to phi_init; EMA update exact formula.
    #[test]
    fn e6_phi_init_and_ema_update() {
        let mut r = rb();
        assert!(r.phi.iter().all(|&p| p == 0.02), "init = phi_init");
        // Channel 0: 200 events / 100 ticks => c/W = 2.0; α = 0.04.
        for _ in 0..200 {
            r.tick(NeuronId(0));
        }
        r.window_start();
        let want = 0.96 * 0.02 + 0.04 * 2.0; // 0.0992
        assert!((r.phi[0] - want).abs() < 1e-6, "EMA update: {} vs {want}", r.phi[0]);
        assert!(
            (r.phi[1] - 0.96 * 0.02).abs() < 1e-6,
            "every channel EMA-updates with its own count (zero here)"
        );
        assert_eq!(r.counts[0], 0, "counts reset after window_start");
    }

    /// §3.1: floor — silent windows decay φ toward phi_min, never below.
    #[test]
    fn e6_phi_floor_never_below_min() {
        let mut r = rb();
        for _ in 0..300 {
            r.window_start();
            assert!(r.phi.iter().all(|&p| p >= 0.001 - 1e-9), "floor respected");
        }
        assert!(r.phi.iter().all(|&p| p == 0.001), "converged to floor");
    }

    /// §3.2/§3.4: β ≡ 1 when φ is constant (identity requirement).
    #[test]
    fn e6_beta_identity_when_phi_constant() {
        let net = net6();
        let r = rb(); // all channels at init 0.02
        for post in 2..net.neurons.len() {
            for pre in 0..6 {
                assert_eq!(r.beta(&net, NeuronId(post as u32), NeuronId(pre as u32)), 1.0);
            }
        }
    }

    /// §3.2: β = φ̄_afferents(pre) / φ_pre over LIVE input afferents only;
    /// recurrent pre (≥ n_input) always 1.0.
    #[test]
    fn e6_beta_local_afferent_mean_and_recurrent() {
        let mut net = net6();
        // post = id 6 (first internal). Wire input afferents from ch 1, 2, 3
        // (φ set below) plus one recurrent afferent from internal id 7.
        net.add_synapse(NeuronId(1), NeuronId(6), 0.05, true, Tick(0));
        net.add_synapse(NeuronId(2), NeuronId(6), 0.05, true, Tick(0));
        net.add_synapse(NeuronId(3), NeuronId(6), 0.05, true, Tick(0));
        net.add_synapse(NeuronId(7), NeuronId(6), 0.02, true, Tick(0));
        let mut r = rb();
        r.phi[1] = 0.02;
        r.phi[2] = 0.03;
        r.phi[3] = 0.01;
        // φ̄ over live INPUT afferents = (0.02 + 0.03 + 0.01)/3 = 0.02.
        let bar = 0.02f32;
        assert!((r.beta(&net, NeuronId(6), NeuronId(1)) - bar / 0.02).abs() < 1e-6, "ch1");
        assert!((r.beta(&net, NeuronId(6), NeuronId(2)) - bar / 0.03).abs() < 1e-6, "ch2");
        assert!((r.beta(&net, NeuronId(6), NeuronId(3)) - bar / 0.01).abs() < 1e-6, "ch3");
        assert_eq!(r.beta(&net, NeuronId(6), NeuronId(7)), 1.0, "recurrent unscaled");
        assert_eq!(r.beta(&net, NeuronId(6), NeuronId(9)), 1.0, "non-input pre unscaled");
    }

    /// §3.2: β clamped to [beta_min, beta_max].
    #[test]
    fn e6_beta_clamp() {
        let mut net = net6();
        // Clear any seeded wiring onto post 6 so the afferent set is exact.
        let seeded: Vec<SynapseId> = net.incoming[6].iter().copied().collect();
        for sid in seeded {
            net.prune_synapse(sid);
        }
        // Post 6 gets 11 live input afferents: ch0 (φ 10.0) + ch1..ch5
        // duplicated (10 synapses at φ 0.001). bar = (10 + 0.01)/11 ≈ 0.91.
        // β(near-floor ch) ≈ 910 → max clamp; β(ch0) = 0.091 → min clamp.
        for _ in 0..2 {
            for ch in 1..6u32 {
                net.add_synapse(NeuronId(ch), NeuronId(6), 0.05, true, Tick(0));
            }
        }
        net.add_synapse(NeuronId(0), NeuronId(6), 0.05, true, Tick(0));
        let mut r = rb();
        for i in 0..6 {
            r.phi[i] = 0.001;
        }
        r.phi[0] = 10.0;
        assert_eq!(r.beta(&net, NeuronId(6), NeuronId(3)), 10.0, "clamped at beta_max");
        assert_eq!(r.beta(&net, NeuronId(6), NeuronId(0)), 0.1, "clamped at beta_min");
    }

    /// §3.1: deterministic — identical spike history ⇒ identical φ.
    #[test]
    fn e6_phi_deterministic() {
        let mut a = rb();
        let mut b = rb();
        let hist: Vec<NeuronId> = (0..250).map(|t| NeuronId((t * 7) % 6)).collect();
        for (i, n) in hist.iter().enumerate() {
            a.tick(*n);
            b.tick(*n);
            if i % 100 == 99 {
                a.window_start();
                b.window_start();
            }
        }
        assert_eq!(a.phi, b.phi, "same history -> same φ");
    }
}
