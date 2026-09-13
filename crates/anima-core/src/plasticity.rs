//! Synaptic plasticity engine.
//!
//! `PlasticityRule` is the fixed interface (D2/E2/E3 variants live here).
//! E1 default: pairwise trace-based STDP with additive bounds (D7) — an
//! experimental default, not settled truth.

use crate::network::{exp_approx, Network, NeuronId, SynapseId};

/// Per-synapse eligibility traces, updated every tick from the spike set.
#[derive(Debug, Clone)]
pub struct Traces {
    /// Pre-synaptic trace per synapse (index = SynapseId).
    pre: Vec<f32>,
    /// Post-synaptic trace per synapse (index = SynapseId).
    post: Vec<f32>,
    decay: f32,
    tau_ms: f32,
}

impl Traces {
    pub fn new(net: &Network, tau_ms: f32) -> Self {
        let n = net.synapses.len();
        let decay = exp_approx(-1.0 / tau_ms);
        Self { pre: vec![0.0; n], post: vec![0.0; n], decay, tau_ms: tau_ms.max(1.0) }
    }

    /// Grow trace storage when synapses are added.
    pub fn sync_len(&mut self, net: &Network) {
        let n = net.synapses.len();
        self.pre.resize(n, 0.0);
        self.post.resize(n, 0.0);
    }

    pub fn pre(&self, s: SynapseId) -> f32 {
        self.pre.get(s.idx()).copied().unwrap_or(0.0)
    }
    pub fn post(&self, s: SynapseId) -> f32 {
        self.post.get(s.idx()).copied().unwrap_or(0.0)
    }
    pub fn tau_ms(&self) -> f32 {
        self.tau_ms
    }

    /// One tick of trace dynamics: decay all, then bump on spikes.
    /// Called with the tick's spike set AFTER the network step.
    pub fn step(&mut self, net: &Network, spikes: &[NeuronId]) {
        // Births may have added synapses since the last resize; the trace
        // arrays must cover every SynapseId before bumping (E4 exposes
        // this: prior to this fix a newborn's synapse id overflowed).
        self.sync_len(net);
        for t in self.pre.iter_mut() {
            *t *= self.decay;
        }
        for t in self.post.iter_mut() {
            *t *= self.decay;
        }
        // Synapses are ordered by id; bump pre-trace of every live synapse
        // whose pre neuron spiked, post-trace whose post spiked.
        for &n in spikes {
            for &sid in &net.outgoing[n.idx()] {
                if net.synapse_alive(sid) {
                    self.pre[sid.idx()] += 1.0;
                }
            }
            for &sid in &net.incoming[n.idx()] {
                if net.synapse_alive(sid) {
                    self.post[sid.idx()] += 1.0;
                }
            }
        }
    }
}

/// A plasticity update outcome for one synapse, for event emission.
#[derive(Debug, Clone)]
pub struct WeightChange {
    pub synapse: SynapseId,
    pub before: f32,
    pub after: f32,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct StdpParams {
    pub tau_plus: f32,  // ms
    pub tau_minus: f32, // ms
    pub a_plus: f32,
    pub a_minus: f32,
    /// Passive decay per tick.
    pub decay: f32,
    /// Below this |Δw| over the event threshold window we stay silent —
    /// event emission threshold handled by caller.
    pub w_min: f32,
    pub w_max: f32,
}

impl Default for StdpParams {
    fn default() -> Self {
        Self {
            tau_plus: 20.0,
            tau_minus: 20.0,
            a_plus: 0.005,
            a_minus: 0.0053,
            decay: 1e-6,
            w_min: 0.0,
            w_max: 1.0,
        }
    }
}

/// Fixed interface every plasticity rule implements. `gate` in [0,1] scales
/// updates (E1 passes 1.0 = always-on, U4a; E5 selects gates).
pub trait PlasticityRule {
    /// Apply one tick of plasticity. Returns weight changes worth emitting
    /// as events (|Δw| above caller threshold); full precision lives in
    /// periodic snapshots.
    fn update(&mut self, net: &mut Network, traces: &mut Traces, gate: f32) -> Vec<WeightChange>;

    /// Which synapses have crossed the silence/prune boundary this tick.
    /// Returns (synapse, reason) pairs for the structural module to emit.
    fn silent_synapses(&self, net: &Network) -> Vec<(SynapseId, &'static str)> {
        let _ = net;
        Vec::new()
    }
}

/// Pairwise trace-based STDP, additive bounds (D7).
/// pre-before-post ⇒ LTP (a_plus × pre-trace at post spike);
/// post-before-pre ⇒ LTD (a_minus × post-trace at pre spike).
/// The tick loop lives in [`stdp_tick`]; this type bundles parameters with
/// the silence/prune thresholds the harness passes to
/// [`silent_synapse_pass`].
pub struct PairwiseStdp {
    pub params: StdpParams,
    /// Weight below which a synapse counts as silent.
    pub silence_w: f32,
    /// Sustained silence (ticks) required before prune eligibility.
    pub silence_ticks: u64,
    /// Minimum synapse age (ticks) before prune eligibility.
    pub min_age_ticks: u64,
}

impl PairwiseStdp {
    pub fn new(params: StdpParams) -> Self {
        Self { params, silence_w: 0.02, silence_ticks: 60_000, min_age_ticks: 30_000 }
    }
}

impl PlasticityRule for PairwiseStdp {
    fn update(&mut self, net: &mut Network, traces: &mut Traces, gate: f32) -> Vec<WeightChange> {
        let n = net.synapses.len();
        traces.sync_len(net);
        let mut changes = Vec::new();
        for i in 0..n {
            let (alive, plastic) = {
                let s = &net.synapses[i];
                (s.silent_ticks != u64::MAX, s.plastic)
            };
            if !alive || !plastic {
                continue;
            }
            let sid = SynapseId(i as u32);
            let pre_t = traces.pre(sid);
            let post_t = traces.post(sid);
            // Δw: LTP from pre-trace (evidence of recent pre spike) when post
            // spikes is captured by post trace containing +1 at post spike;
            // standard trace formulation applied per tick:
            //   dw = a_plus * pre_trace * post_fired - a_minus * post_trace * pre_fired
            // Here we use the continuous form: per-tick update proportional to
            // trace products with spike indicators this tick.
            let post_fired = net.neurons[net.synapses[i].post.idx()].rate_hz > 0.0 && {
                // approximate spike indicator via trace bump this tick is not
                // accessible here; instead update on trace values directly
                false
            };
            let _ = post_fired;
            // Continuous trace-form STDP per tick:
            // pre trace decays after pre spike; when post spikes, LTP ∝ pre trace.
            // post trace decays after post spike; when pre spikes, LTD ∝ post trace.
            // We detect "spiked this tick" from the trace bump (trace jumped by
            // ~1 this tick). To keep this exact and cheap, the harness passes
            // spikes to Traces::step BEFORE plasticity, so a fresh bump is
            // trace > (decayed old). Simplification: use product form
            // (Morrison et al.): dw = a_plus * x_pre * y_post_rate — for E1 we
            // use the per-spike form driven by trace values:
            let s = &mut net.synapses[i];
            let before = s.w;
            let dw = self.params.a_plus * pre_t * post_t * 0.0; // placeholder, replaced below
            let _ = dw;
            // Per-spike form: LTP when post fires (post trace just bumped to
            // include this tick's spike): use post_t as carrier of timing.
            let ltp = self.params.a_plus * pre_t; // applied when post fired this tick
            let ltd = self.params.a_minus * post_t; // applied when pre fired this tick
            // Detect firing this tick via instantaneous rate target: the
            // network sets rate target to 1000 only on spike ticks, but rate
            // is an EMA. The harness therefore calls update with spike set —
            // see update_with_spikes. Plain update() applies the passive decay
            // only; the harness uses update_with_spikes for STDP terms.
            s.w -= self.params.decay * gate;
            if s.w < self.params.w_min {
                s.w = self.params.w_min;
            }
            let after = s.w;
            let _ = (ltp, ltd);
            if after != before {
                changes.push(WeightChange { synapse: sid, before, after });
            }
        }
        changes
    }
}

/// Pairwise trace-based STDP with **multiplicative** (soft) bounds — E2
/// arm B. Identical timing structure to [`PairwiseStdp`]/[`stdp_tick`];
/// the weight update is scaled by the synapse's own state:
///   LTP: Δw = +a_plus  · pre_trace · (1 − w)   (far from ceiling learns fast)
///   LTD: Δw = −a_minus · post_trace · w        (near floor barely depresses)
/// Saturated synapses stay plastic, sustaining competition between patterns.
pub struct MultiplicativeStdp;

impl MultiplicativeStdp {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MultiplicativeStdp {
    fn default() -> Self {
        Self::new()
    }
}

/// Multiplicative per-spike STDP: same walk as [`stdp_tick`] with the
/// weight-dependent scaling applied to both signs.
pub fn stdp_tick_multiplicative(
    params: &StdpParams,
    net: &mut Network,
    traces: &Traces,
    spikes: &[NeuronId],
    gate: f32,
) -> Vec<WeightChange> {
    let mut changes = Vec::new();
    // LTP pass: for each post neuron that fired, potentiate its incoming.
    for &post in spikes {
        let incoming: Vec<SynapseId> = net.incoming[post.idx()].clone();
        for sid in incoming {
            if !net.synapse_alive(sid) {
                continue;
            }
            let s = &net.synapses[sid.idx()];
            if !s.plastic {
                continue;
            }
            if spikes.contains(&s.pre) {
                continue;
            }
            let pre_t = traces.pre(sid);
            if pre_t <= 0.0 {
                continue;
            }
            let s = &mut net.synapses[sid.idx()];
            let before = s.w;
            let room = (params.w_max - s.w).max(0.0);
            s.w = (s.w + params.a_plus * pre_t * gate * room).min(params.w_max);
            if s.w != before {
                changes.push(WeightChange { synapse: sid, before, after: s.w });
            }
        }
    }
    // LTD pass: for each pre neuron that fired, depress its outgoing.
    for &pre in spikes {
        let outgoing: Vec<SynapseId> = net.outgoing[pre.idx()].clone();
        for sid in outgoing {
            if !net.synapse_alive(sid) {
                continue;
            }
            let s = &net.synapses[sid.idx()];
            if !s.plastic {
                continue;
            }
            if spikes.contains(&s.post) {
                continue;
            }
            let post_t = traces.post(sid);
            if post_t <= 0.0 {
                continue;
            }
            let s = &mut net.synapses[sid.idx()];
            let before = s.w;
            let mass = (s.w - params.w_min).max(0.0);
            s.w = (s.w - params.a_minus * post_t * gate * mass).max(params.w_min);
            if s.w != before {
                changes.push(WeightChange { synapse: sid, before, after: s.w });
            }
        }
    }
    changes
}

/// Extension used by the harness: per-spike STDP given the tick's spike set.
/// pre-before-post ⇒ LTP: when post fires, potentiate by a_plus × pre-trace.
/// post-before-pre ⇒ LTD: when pre fires, depress by a_minus × post-trace.
pub fn stdp_tick(
    params: &StdpParams,
    net: &mut Network,
    traces: &Traces,
    spikes: &[NeuronId],
    gate: f32,
) -> Vec<WeightChange> {
    let mut changes = Vec::new();
    // LTP pass: for each post neuron that fired, potentiate its incoming.
    for &post in spikes {
        let incoming: Vec<SynapseId> = net.incoming[post.idx()].clone();
        for sid in incoming {
            if !net.synapse_alive(sid) {
                continue;
            }
            let s = &net.synapses[sid.idx()];
            if !s.plastic {
                continue;
            }
            // Skip synapses whose pre ALSO fired this tick (coincident — net
            // zero, assigned neither sign).
            if spikes.contains(&s.pre) {
                continue;
            }
            let pre_t = traces.pre(sid);
            if pre_t <= 0.0 {
                continue;
            }
            let s = &mut net.synapses[sid.idx()];
            let before = s.w;
            s.w = (s.w + params.a_plus * pre_t * gate).min(params.w_max);
            if s.w != before {
                changes.push(WeightChange { synapse: sid, before, after: s.w });
            }
        }
    }
    // LTD pass: for each pre neuron that fired, depress its outgoing.
    for &pre in spikes {
        let outgoing: Vec<SynapseId> = net.outgoing[pre.idx()].clone();
        for sid in outgoing {
            if !net.synapse_alive(sid) {
                continue;
            }
            let s = &net.synapses[sid.idx()];
            if !s.plastic {
                continue;
            }
            if spikes.contains(&s.post) {
                continue;
            }
            let post_t = traces.post(sid);
            if post_t <= 0.0 {
                continue;
            }
            let s = &mut net.synapses[sid.idx()];
            let before = s.w;
            s.w = (s.w - params.a_minus * post_t * gate).max(params.w_min);
            if s.w != before {
                changes.push(WeightChange { synapse: sid, before, after: s.w });
            }
        }
    }
    changes
}

/// Advance silence accounting and return prune candidates: synapses with
/// w < silence_w for > silence_ticks sim-time and age > min_age.
pub fn silent_synapse_pass(
    net: &mut Network,
    silence_w: f32,
    silence_ticks: u64,
    min_age: u64,
) -> Vec<(SynapseId, &'static str)> {
    let now = net.tick.0;
    let mut prunes = Vec::new();
    for i in 0..net.synapses.len() {
        let s = &mut net.synapses[i];
        if s.silent_ticks == u64::MAX {
            continue;
        }
        if s.w < silence_w {
            s.silent_ticks += 1;
        } else {
            s.silent_ticks = 0;
        }
        if s.silent_ticks > silence_ticks && now - s.created.0 > min_age {
            prunes.push((SynapseId(i as u32), "silent-synapse"));
        }
    }
    prunes
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::{InputFrame, NetworkConfig, Tick};

    fn small_net() -> Network {
        let mut net = Network::new(NetworkConfig::default(), 2, 3, 1, 7);
        // Deterministic chain: ch0 -> internal, internal -> output.
        // We just use the existing seeded wiring; tests locate synapses.
        net
    }

    /// Crafted pair: pre fires 5 ticks before post ⇒ net potentiation.
    #[test]
    fn pre_before_post_potentiates() {
        let mut net = small_net();
        let mut traces = Traces::new(&net, 20.0);
        // Find a synapse pre=A post=B where we can drive both.
        let sid = net
            .synapses
            .iter()
            .find(|s| {
                net.neurons[s.pre.idx()].class != crate::network::NeuronClass::Input
                    && net.neurons[s.post.idx()].class != crate::network::NeuronClass::Input
            })
            .map(|s| s.id)
            .expect("seeded net has an internal→internal synapse");
        let pre = net.synapses[sid.idx()].pre;
        let post = net.synapses[sid.idx()].post;
        let w0 = net.synapses[sid.idx()].w;

        // Pre fires at t=0 (force spike via threshold), post fires at t=5.
        let params = StdpParams::default();
        force_spike(&mut net, pre);
        let ev = net.step(&InputFrame { tick: Tick(0), spikes: vec![] });
        traces.step(&net, &ev.spikes);
        for t in 1..5 {
            let ev = net.step(&InputFrame { tick: Tick(t), spikes: vec![] });
            traces.step(&net, &ev.spikes);
        }
        force_spike(&mut net, post);
        let ev = net.step(&InputFrame { tick: Tick(5), spikes: vec![] });
        traces.step(&net, &ev.spikes);
        let changes = stdp_tick(&params, &mut net, &traces, &ev.spikes, 1.0);
        let w1 = net.synapses[sid.idx()].w;
        let delta = changes.iter().find(|c| c.synapse == sid).map(|c| c.after - c.before);
        // Either picked up in changes (bounded at w_max) or w rose.
        let rose = w1 > w0 + 1e-6 || delta.map(|d| d > 0.0).unwrap_or(false);
        assert!(rose, "pre-before-post must potentiate: w0={w0} w1={w1}");
    }

    /// Crafted pair: post fires 5 ticks before pre ⇒ net depression.
    #[test]
    fn post_before_pre_depresses() {
        let mut net = small_net();
        let mut traces = Traces::new(&net, 20.0);
        let sid = net
            .synapses
            .iter()
            .find(|s| {
                net.neurons[s.pre.idx()].class != crate::network::NeuronClass::Input
                    && net.neurons[s.post.idx()].class != crate::network::NeuronClass::Input
            })
            .map(|s| s.id)
            .expect("internal→internal synapse");
        let pre = net.synapses[sid.idx()].pre;
        let post = net.synapses[sid.idx()].post;
        let w0 = net.synapses[sid.idx()].w;

        let params = StdpParams::default();
        force_spike(&mut net, post);
        let ev = net.step(&InputFrame { tick: Tick(0), spikes: vec![] });
        traces.step(&net, &ev.spikes);
        for t in 1..5 {
            let ev = net.step(&InputFrame { tick: Tick(t), spikes: vec![] });
            traces.step(&net, &ev.spikes);
        }
        force_spike(&mut net, pre);
        let ev = net.step(&InputFrame { tick: Tick(5), spikes: vec![] });
        traces.step(&net, &ev.spikes);
        let changes = stdp_tick(&params, &mut net, &traces, &ev.spikes, 1.0);
        let w1 = net.synapses[sid.idx()].w;
        let fell = w1 < w0 - 1e-6
            || changes
                .iter()
                .find(|c| c.synapse == sid)
                .map(|c| c.after - c.before < 0.0)
                .unwrap_or(false);
        assert!(fell, "post-before-pre must depress: w0={w0} w1={w1}");
    }

    #[test]
    fn weights_clamped_at_bounds() {
        let mut net = small_net();
        let mut traces = Traces::new(&net, 20.0);
        let params = StdpParams { a_plus: 10.0, ..StdpParams::default() };
        let pre = net.channels[0].target;
        // Drive input channel hard; every internal target potentiation is clamped.
        for t in 0..100 {
            let ev = net.step(&InputFrame {
                tick: Tick(t),
                spikes: vec![crate::network::InputChannelId(0)],
            });
            traces.step(&net, &ev.spikes);
            stdp_tick(&params, &mut net, &traces, &ev.spikes, 1.0);
        }
        for s in net.live_synapses() {
            assert!(s.w <= params.w_max + 1e-6, "w={} > w_max", s.w);
            assert!(s.w >= params.w_min - 1e-6);
        }
    }

    #[test]
    fn silent_synapse_pass_flags_long_quiet() {
        let mut net = small_net();
        // Force all weights below silence threshold and age them artificially.
        for i in 0..net.synapses.len() {
            net.synapses[i].w = 0.001;
            net.synapses[i].created = Tick(0);
        }
        let mut flagged = Vec::new();
        for _ in 0..(60_001) {
            flagged = silent_synapse_pass(&mut net, 0.02, 60_000, 30_000);
            net.tick = Tick(net.tick.0 + 1);
            if !flagged.is_empty() {
                break;
            }
        }
        assert!(!flagged.is_empty());
        assert_eq!(flagged[0].1, "silent-synapse");
    }

    /// Multiplicative LTP is scaled by (1 − w): at w near ceiling Δw → 0.
    #[test]
    fn multiplicative_ltp_scales_with_room() {
        let params = StdpParams::default();
        let mut cases = vec![(0.2_f32, 0.0_f32), (0.8, 0.0)];
        for (w_set, expected) in cases.iter_mut() {
            let mut net = small_net();
            let sid = sid_of(&net); net.synapses[sid.idx()].w = *w_set;
            let sid = sid_of(&net);
            let pre = net.synapses[sid.idx()].pre;
            let post = net.synapses[sid.idx()].post;
            let mut traces = Traces::new(&net, 20.0);
            force_spike(&mut net, pre);
            let ev = net.step(&InputFrame { tick: Tick(0), spikes: vec![] });
            traces.step(&net, &ev.spikes);
            force_spike(&mut net, post);
            let ev = net.step(&InputFrame { tick: Tick(1), spikes: vec![] });
            traces.step(&net, &ev.spikes);
            let changes = stdp_tick_multiplicative(&params, &mut net, &traces, &ev.spikes, 1.0);
            *expected = changes
                .iter()
                .find(|c| c.synapse == sid)
                .map(|c| c.after - c.before)
                .unwrap_or(0.0);
        }
        let (dw_low_w, dw_high_w) = (cases[0].1, cases[1].1);
        assert!(
            dw_low_w > dw_high_w + 1e-9,
            "multiplicative LTP must shrink as w grows: dw(w=0.2)={dw_low_w} vs dw(w=0.8)={dw_high_w}"
        );
    }

    /// Multiplicative LTD is scaled by w: at w near floor Δw → 0.
    #[test]
    fn multiplicative_ltd_scales_with_weight() {
        let params = StdpParams::default();
        let mut cases = vec![(0.9_f32, 0.0_f32), (0.1, 0.0)];
        for (w_set, expected) in cases.iter_mut() {
            let mut net = small_net();
            let sid = sid_of(&net); net.synapses[sid.idx()].w = *w_set;
            let sid = sid_of(&net);
            let pre = net.synapses[sid.idx()].pre;
            let post = net.synapses[sid.idx()].post;
            let mut traces = Traces::new(&net, 20.0);
            // post fires first (builds post trace), then pre ⇒ LTD
            force_spike(&mut net, post);
            let ev = net.step(&InputFrame { tick: Tick(0), spikes: vec![] });
            traces.step(&net, &ev.spikes);
            force_spike(&mut net, pre);
            let ev = net.step(&InputFrame { tick: Tick(1), spikes: vec![] });
            traces.step(&net, &ev.spikes);
            let changes = stdp_tick_multiplicative(&params, &mut net, &traces, &ev.spikes, 1.0);
            *expected = changes
                .iter()
                .find(|c| c.synapse == sid)
                .map(|c| c.after - c.before)
                .unwrap_or(0.0);
        }
        let (dw_high_w, dw_low_w) = (cases[0].1, cases[1].1);
        assert!(
            dw_high_w < dw_low_w - 1e-9,
            "multiplicative LTD must shrink as w → floor: dw(w=0.9)={dw_high_w} vs dw(w=0.1)={dw_low_w}"
        );
    }

    /// Multiplicative LTP at w = w_max must be exactly zero (soft ceiling).
    #[test]
    fn multiplicative_ltp_vanishes_at_ceiling() {
        let params = StdpParams { a_plus: 0.05, ..StdpParams::default() };
        let mut net = small_net();
        let sid = sid_of(&net); net.synapses[sid.idx()].w = params.w_max;
        let sid = sid_of(&net);
        let pre = net.synapses[sid.idx()].pre;
        let post = net.synapses[sid.idx()].post;
        let mut traces = Traces::new(&net, 20.0);
        force_spike(&mut net, pre);
        let ev = net.step(&InputFrame { tick: Tick(0), spikes: vec![] });
        traces.step(&net, &ev.spikes);
        force_spike(&mut net, post);
        let ev = net.step(&InputFrame { tick: Tick(1), spikes: vec![] });
        traces.step(&net, &ev.spikes);
        let changes = stdp_tick_multiplicative(&params, &mut net, &traces, &ev.spikes, 1.0);
        let d = changes
            .iter()
            .find(|c| c.synapse == sid)
            .map(|c| c.after - c.before)
            .unwrap_or(0.0);
        assert!(d <= 1e-9, "no LTP at ceiling: Δw = {d}");
    }

    fn sid_of(net: &Network) -> SynapseId {
        net.synapses
            .iter()
            .find(|s| {
                net.neurons[s.pre.idx()].class != crate::network::NeuronClass::Input
                    && net.neurons[s.post.idx()].class != crate::network::NeuronClass::Input
            })
            .map(|s| s.id)
            .expect("internal→internal synapse")
    }

    fn force_spike(net: &mut Network, id: NeuronId) {
        let n = &mut net.neurons[id.idx()];
        n.v = 100.0; // far over threshold; refractory check passes at t=0
        n.refractory_until = Tick(0);
    }

    /// E4 regression: a birth adds synapses with ids beyond the traces'
    /// initial allocation; step() must not index out of bounds.
    #[test]
    fn traces_step_survives_synapse_growth() {
        let mut net = small_net();
        let mut traces = Traces::new(&net, 20.0);
        // Simulate growth: append a live synapse (as structural birth does).
        let before = net.synapses.len();
        let pre = net.neurons.iter().find(|n| n.class != crate::network::NeuronClass::Input).unwrap().id;
        let post = pre;
        net.add_synapse(pre, post, 0.5, true, Tick(100));
        assert_eq!(net.synapses.len(), before + 1);
        // Spike the pre neuron; bump path touches the new synapse id.
        let spike_ev = {
            let mut n = &mut net.neurons[pre.idx()];
            n.v = 100.0;
            n.refractory_until = Tick(0);
            net.step(&InputFrame { tick: Tick(150), spikes: vec![] })
        };
        traces.step(&net, &spike_ev.spikes); // must not panic
        assert_eq!(traces.pre.len(), net.synapses.len());
    }
}
