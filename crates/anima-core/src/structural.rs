//! Structural plasticity: neuron birth triggers, wiring, dormancy.
//! Machinery exists now; the trigger is config-selected (E1: none, E4: A/B/C/D).

use serde::{Deserialize, Serialize};

use crate::network::{Neuron, NeuronClass, NeuronId, Network, Tick};

/// Causal metadata is mandatory (§15): every structural event carries why.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reason {
    pub trigger: String,
    /// (factor, value) pairs — the measured signals that fired the trigger.
    pub contributing: Vec<(String, f32)>,
    /// Thresholds that were applied.
    pub thresholds: Vec<(String, f32)>,
}

impl Reason {
    pub fn new(trigger: &str) -> Self {
        Self {
            trigger: trigger.to_string(),
            contributing: Vec::new(),
            thresholds: Vec::new(),
        }
    }
    pub fn factor(mut self, name: &str, value: f32) -> Self {
        self.contributing.push((name.to_string(), value));
        self
    }
    pub fn threshold(mut self, name: &str, value: f32) -> Self {
        self.thresholds.push((name.to_string(), value));
        self
    }
}

/// Cross-module signals available to triggers.
#[derive(Debug, Clone, Default)]
pub struct Signals {
    /// Running mean prediction error (instrumentation, E1: input mismatch).
    pub prediction_error: f32,
    /// Mean + std of recent prediction error (for PersistentError).
    pub pe_mean: f32,
    pub pe_std: f32,
    /// Novelty signal (E1: instrumentation-computed).
    pub novelty: f32,
}

pub trait BirthTrigger: Send {
    /// Called every tick (or every window); returns a birth reason when a
    /// new neuron should be born.
    fn should_birth(&mut self, net: &Network, signals: &Signals) -> Option<Reason>;
    fn name(&self) -> &'static str;
}

/// E1: no internal growth.
pub struct NoBirth;
impl BirthTrigger for NoBirth {
    fn should_birth(&mut self, _net: &Network, _signals: &Signals) -> Option<Reason> {
        None
    }
    fn name(&self) -> &'static str {
        "none"
    }
}

/// Region mean rate > 25 Hz sustained 2 s.
/// A8/E4b: optional `cooldown_ms` — after a birth, the armed trigger is
/// held (over_since NOT reset) until the cooldown elapses, so an ongoing
/// overload admits at most one birth per cooldown window. 0 = no limit
/// (E4/E3 behavior).
pub struct HomeostaticSaturation {
    pub rate_threshold_hz: f32,
    pub sustained_ms: u64,
    pub cooldown_ms: u64,
    over_since: Option<Tick>,
    last_birth: Option<Tick>,
}

impl HomeostaticSaturation {
    pub fn new() -> Self {
        Self {
            rate_threshold_hz: 25.0,
            sustained_ms: 2_000,
            cooldown_ms: 0,
            over_since: None,
            last_birth: None,
        }
    }
}

impl Default for HomeostaticSaturation {
    fn default() -> Self {
        Self::new()
    }
}

impl BirthTrigger for HomeostaticSaturation {
    fn should_birth(&mut self, net: &Network, _signals: &Signals) -> Option<Reason> {
        let internal: Vec<&Neuron> =
            net.neurons.iter().filter(|n| n.class == NeuronClass::Internal).collect();
        if internal.is_empty() {
            return None;
        }
        let mean = internal.iter().map(|n| n.rate_hz).sum::<f32>() / internal.len() as f32;
        if mean > self.rate_threshold_hz {
            let since = *self.over_since.get_or_insert(net.tick);
            if net.tick.0.saturating_sub(since.0) >= self.sustained_ms {
                // A8/E4b: if a birth happened recently, hold the armed
                // state (keep over_since) and fire once the cooldown
                // elapsed — at most one birth per cooldown window under
                // continuous overload.
                if let Some(lb) = self.last_birth {
                    if net.tick.0.saturating_sub(lb.0) < self.cooldown_ms {
                        return None;
                    }
                }
                // Re-arm: fire once per sustained episode, not every tick
                // while overloaded (otherwise admission windows exhaust
                // and the run aborts with birth-rate-limit).
                self.over_since = None;
                self.last_birth = Some(net.tick);
                return Some(
                    Reason::new("homeostatic-saturation")
                        .factor("region-mean-rate-hz", mean)
                        .threshold("rate-threshold-hz", self.rate_threshold_hz)
                        .threshold("sustained-ms", self.sustained_ms as f32),
                );
            }
        } else {
            self.over_since = None;
        }
        None
    }
    fn name(&self) -> &'static str {
        "homeostatic-saturation"
    }
}

/// Running prediction error > μ+2σ sustained 5 s.
/// A11: previously dead code — `over_since?` could never arm. Same
/// latch/cooldown semantics as HomeostaticSaturation.
pub struct PersistentError {
    pub sigma: f32,
    pub sustained_ms: u64,
    pub cooldown_ms: u64,
    over_since: Option<Tick>,
    last_birth: Option<Tick>,
}

impl PersistentError {
    pub fn new() -> Self {
        Self { sigma: 2.0, sustained_ms: 5_000, cooldown_ms: 0, over_since: None, last_birth: None }
    }
}

impl Default for PersistentError {
    fn default() -> Self {
        Self::new()
    }
}

impl BirthTrigger for PersistentError {
    fn should_birth(&mut self, net: &Network, signals: &Signals) -> Option<Reason> {
        let bound = signals.pe_mean + self.sigma * signals.pe_std;
        if signals.prediction_error > bound {
            let since = *self.over_since.get_or_insert(net.tick);
            if net.tick.0.saturating_sub(since.0) >= self.sustained_ms {
                // A11: cooldown floor + re-arm (same as homeostatic-saturation).
                if let Some(lb) = self.last_birth {
                    if net.tick.0.saturating_sub(lb.0) < self.cooldown_ms {
                        return None;
                    }
                }
                self.over_since = None;
                self.last_birth = Some(net.tick);
                return Some(
                    Reason::new("persistent-error")
                        .factor("prediction-error", signals.prediction_error)
                        .factor("bound", bound)
                        .threshold("sigma", self.sigma)
                        .threshold("sustained-ms", self.sustained_ms as f32),
                );
            }
        } else {
            self.over_since = None;
        }
        None
    }
    fn name(&self) -> &'static str {
        "persistent-error"
    }
}

/// Config selector (E1 "none", E4 selects; crafted configs/tests override
/// threshold params).
pub fn make_trigger(
    kind: &str,
    rate_hz: Option<f32>,
    sustained_ms: Option<u64>,
    cooldown_ms: Option<u64>,
) -> Box<dyn BirthTrigger> {
    match kind {
        "none" => Box::new(NoBirth),
        "homeostatic-saturation" => {
            let mut t = HomeostaticSaturation::new();
            if let Some(r) = rate_hz {
                t.rate_threshold_hz = r;
            }
            if let Some(s) = sustained_ms {
                t.sustained_ms = s;
            }
            if let Some(c) = cooldown_ms {
                t.cooldown_ms = c;
            }
            Box::new(t)
        }
        "persistent-error" => {
            let mut t = PersistentError::new();
            if let Some(c) = cooldown_ms {
                t.cooldown_ms = c;
            }
            Box::new(t)
        }
        other => panic!("unknown birth trigger: {other}"),
    }
}

/// Structural outcomes for one tick.
#[derive(Debug, Clone)]
pub struct StructuralEvents {
    pub births: Vec<(NeuronId, Reason)>,
    pub dormant: Vec<NeuronId>,
    pub reactivated: Vec<NeuronId>,
    pub retired: Vec<NeuronId>,
}

#[derive(Clone)]
pub struct StructuralMonitor {
    pub dormancy_rate_hz: f32,      // below ⇒ dormant
    pub dormancy_ms: u64,           // sustained
    pub recovery_rate_hz: f32,      // above ⇒ reactivated
    pub retirement_ms: u64,         // dormant longer than ⇒ retired
    pub wiring_synapses: usize,     // per birth
    /// U3 (E4): when true, newborn afferents target the LOWEST rate-EMA
    /// neurons (away from the co-active pool); false = Phase 0 behavior.
    pub wiring_avoid_coactive: bool,
    /// E4d: when true, the newborn ALSO gets outgoing efferents back
    /// onto the SAME allocated partners (fan-out-matched, 20 in + 20
    /// out); false = E4c/E4 sink shape (bit-identical).
    pub wiring_bidirectional: bool,
}

impl Default for StructuralMonitor {
    fn default() -> Self {
        Self {
            dormancy_rate_hz: 0.1,
            dormancy_ms: 30_000,
            recovery_rate_hz: 1.0,
            retirement_ms: 300_000,
            wiring_synapses: 20,
            wiring_avoid_coactive: false,
            wiring_bidirectional: false,
        }
    }
}

impl StructuralMonitor {
    /// Dormancy lifecycle check (call each tick).
    fn lifecycle(&self, net: &mut Network) -> (Vec<NeuronId>, Vec<NeuronId>, Vec<NeuronId>) {
        let dormant = Vec::new();
        let mut reactivated = Vec::new();
        let mut retired = Vec::new();
        let now = net.tick.0;
        for i in 0..net.neurons.len() {
            if net.neurons[i].class == NeuronClass::Input || net.neurons[i].retired {
                continue;
            }
            let n = &mut net.neurons[i];
            match n.dormant_since {
                None => {
                    if n.rate_hz < self.dormancy_rate_hz {
                        // entering dormancy requires sustained quiet —
                        // track via dormant_since as candidate start.
                        // For sustained tracking we store the candidate
                        // start in dormant_since and confirm at duration.
                        n.dormant_since = Some(Tick(now));
                    }
                }
                Some(since) => {
                    if n.rate_hz >= self.recovery_rate_hz {
                        if now - since.0 >= self.dormancy_ms {
                            reactivated.push(n.id);
                        }
                        n.dormant_since = None;
                    } else if now - since.0 >= self.retirement_ms {
                        n.retired = true;
                        retired.push(n.id);
                    }
                }
            }
        }
        (dormant, reactivated, retired)
    }

    /// Full structural pass: dormancy lifecycle + optional birth wiring.
    /// Birth appends the neuron, wires sparse synapses to most-recently-
    /// coactive partners (seeded), and returns the new id with its reason.
    pub fn step(
        &mut self,
        net: &mut Network,
        trigger: &mut dyn BirthTrigger,
        signals: &Signals,
    ) -> StructuralEvents {
        let (mut dormant, reactivated, mut retired) = self.lifecycle(net);
        let _ = &mut dormant;
        let _ = &mut retired;

        let mut births = Vec::new();
        if let Some(reason) = trigger.should_birth(net, signals) {
            let id = self.birth(net);
            births.push((id, reason));
        }
        StructuralEvents { births, dormant, reactivated, retired }
    }

    fn birth(&self, net: &mut Network) -> NeuronId {
        use rand::Rng;
        let id = NeuronId(net.neurons.len() as u32);
        net.neurons.push(Neuron {
            id,
            class: NeuronClass::Internal,
            born: net.tick,
            channel: None,
            v: net.cfg.lif.v_reset,
            refractory_until: Tick(0),
            i_syn: 0.0,
            i_ext: 0.0,
            rate_hz: 0.0,
            i_adapt: 0.0,
            u_slow: 0.0,
            z_latch: 0,
            g_drive: 0.0,
            theta_rel: 1.0,
            u_plateau_rel: 1.0,
            tau_het_rel: 1.0,
            rg_w: 0.0,
            rg_p: 0.0,
            dormant_since: None,
            retired: false,
        });
        net.incoming.push(Vec::new());
        net.outgoing.push(Vec::new());
        net.rg_gate.push(0.0);
        net.rg_tick.push(0.0);
        // V2.2 G2 (spec §1.2 step 6): born neurons draw heterogeneity from
        // the same seeded distributions, appended to the network RNG stream.
        if net.cfg.latch_enable
            && (net.cfg.theta_rel_sd > 0.0
                || net.cfg.u_plateau_rel_sd > 0.0
                || net.cfg.tau_het_rel_sd > 0.0)
        {
            let draw_lognormal = |rng: &mut rand_xoshiro::Xoshiro256PlusPlus, m: f32, s: f32| -> f32 {
                use rand::Rng;
                let mu = (m.max(1e-9)).ln() - s * s / 2.0;
                let u1: f32 = rng.gen::<f32>().max(1e-9);
                let u2: f32 = rng.gen::<f32>();
                let r = (-2.0 * u1.ln()).sqrt();
                let z = r * (2.0 * core::f32::consts::PI * u2).cos();
                (mu + s * z).exp()
            };
            let th = draw_lognormal(&mut net.rng, net.cfg.theta_rel_mean, net.cfg.theta_rel_sd);
            let up = draw_lognormal(&mut net.rng, net.cfg.u_plateau_rel_mean, net.cfg.u_plateau_rel_sd);
            let t = if net.cfg.tau_het_rel_sd > 0.0 {
                draw_lognormal(&mut net.rng, 1.0, net.cfg.tau_het_rel_sd)
            } else { 1.0 };
            let n = &mut net.neurons[id.0 as usize];
            n.theta_rel = th;
            n.u_plateau_rel = up;
            n.tau_het_rel = t;
        }
        // Wire to partners by coactivity preference. Default (Phase 0):
        // highest-rate (most-recently-coactive) partners. U3/E4
        // (`wiring_avoid_coactive`): lowest-rate — capacity allocated
        // away from the shared co-active pool. Seeded weight draw.
        let mut partners: Vec<(NeuronId, f32)> = net
            .neurons
            .iter()
            .filter(|n| n.class != NeuronClass::Input && !n.retired && n.id != id)
            .map(|n| (n.id, n.rate_hz))
            .collect();
        if self.wiring_avoid_coactive {
            partners.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        } else {
            partners.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        }
        let take = self.wiring_synapses.min(partners.len());
        for &(partner, _) in &partners[..take] {
            let w = net.rng.gen::<f32>() * net.cfg.w_init + 0.05;
            net.add_synapse(partner, id, w, true, net.tick);
            // E4d: fan-out-matched — same partner gets an efferent back
            // from the newborn (newborn → partner), same weight family.
            // The newborn's firing now flows into the allocated pool
            // instead of accumulating as a high-gain sink.
            if self.wiring_bidirectional {
                let w_out = net.rng.gen::<f32>() * net.cfg.w_init + 0.05;
                net.add_synapse(id, partner, w_out, true, net.tick);
            }
        }
        id
    }

    /// Retire a neuron: prune its synapses (reason `neuron-retired` handled
    /// by caller event emission).
    pub fn retire_neuron(&self, net: &mut Network, id: NeuronId) -> Vec<crate::network::SynapseId> {
        let mut pruned = Vec::new();
        let out: Vec<_> = net.outgoing[id.idx()].clone();
        let inc: Vec<_> = net.incoming[id.idx()].clone();
        for sid in out.into_iter().chain(inc.into_iter()) {
            if net.synapse_alive(sid) {
                net.prune_synapse(sid);
                pruned.push(sid);
            }
        }
        pruned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::{InputFrame, NetworkConfig};

    #[test]
    fn no_birth_never_fires() {
        let mut net = Network::new(NetworkConfig::default(), 2, 4, 1, 3);
        let mut mon = StructuralMonitor::default();
        for t in 0..1000 {
            let mut no_birth = NoBirth; let ev = mon.step(&mut net, &mut no_birth, &Signals::default());
            assert!(ev.births.is_empty());
            let _ = net.step(&InputFrame { tick: Tick(t), spikes: vec![] });
        }
    }

    #[test]
    fn saturation_trigger_births_with_reason() {
        let mut net = Network::new(NetworkConfig::default(), 2, 4, 1, 3);
        // Force saturation: set rates high and hold the trigger's clock.
        let mut trig = HomeostaticSaturation::new();
        trig.over_since = Some(Tick(0));
        for n in net.neurons.iter_mut() {
            n.rate_hz = 40.0;
        }
        net.tick = Tick(3_000);
        let r = trig.should_birth(&net, &Signals::default()).expect("must fire at 40Hz>25Hz for 3s");
        assert_eq!(r.trigger, "homeostatic-saturation");
        assert!(!r.contributing.is_empty());
        assert!(!r.thresholds.is_empty());
    }

    #[test]
    fn saturation_trigger_latches_rearm_per_episode() {
        let mut net = Network::new(NetworkConfig::default(), 2, 4, 1, 3);
        for n in net.neurons.iter_mut() {
            n.rate_hz = 40.0;
        }
        let mut trig = HomeostaticSaturation { rate_threshold_hz: 25.0, sustained_ms: 25, cooldown_ms: 0, over_since: None, last_birth: None };
        // Continuous overload over 100 ticks with sustained 25 ms: must
        // fire exactly 3× (t=+25, +50, +75), NOT once per tick.
        let mut fire_ticks = Vec::new();
        for d in 0..100u64 {
            net.tick = Tick(1_000 + d);
            if trig.should_birth(&net, &Signals::default()).is_some() {
                fire_ticks.push(1_000 + d);
            }
        }
        assert_eq!(fire_ticks, vec![1_025, 1_051, 1_077], "one fire per sustained window (re-arm next tick)");
        // Episode ends: mean drops below threshold => re-arm allowed.
        for n in net.neurons.iter_mut() {
            n.rate_hz = 5.0;
        }
        net.tick = Tick(3_000);
        assert!(trig.should_birth(&net, &Signals::default()).is_none());
        // New episode fires again.
        for n in net.neurons.iter_mut() {
            n.rate_hz = 40.0;
        }
        net.tick = Tick(5_090);
        let _ = trig.should_birth(&net, &Signals::default()); // arms the window
        net.tick = Tick(5_115);
        let r = trig.should_birth(&net, &Signals::default()).expect("second episode fires");
        assert_eq!(r.trigger, "homeostatic-saturation");
    }

    /// A8/E4b: with cooldown_ms set, an ongoing overload admits at most
    /// one birth per cooldown window; the armed state is held, so the
    /// next fire lands exactly at cooldown expiry.
    #[test]
    fn saturation_trigger_cooldown_limits_cadence() {
        let mut net = Network::new(NetworkConfig::default(), 2, 4, 1, 3);
        for n in net.neurons.iter_mut() {
            n.rate_hz = 40.0;
        }
        let mut trig = HomeostaticSaturation {
            rate_threshold_hz: 25.0,
            sustained_ms: 25,
            cooldown_ms: 100,
            over_since: None,
            last_birth: None,
        };
        let mut fire_ticks = Vec::new();
        // Continuous overload over 300 ticks with 100 ms cooldown:
        // fires at 1025, then holds until 1126-ish, then 1227-ish…
        for d in 0..300u64 {
            net.tick = Tick(1_000 + d);
            if trig.should_birth(&net, &Signals::default()).is_some() {
                fire_ticks.push(1_000 + d);
            }
        }
        // Latch spacing is 26 (25 sustained + 1 re-arm tick); cooldown
        // keeps the spacing at <= 26 + 100 between fires.
        let gaps: Vec<u64> = fire_ticks.windows(2).map(|w| w[1] - w[0]).collect();
        assert!(fire_ticks.len() >= 3, "cooldown must not silence growth: {fire_ticks:?}");
        for g in &gaps {
            assert!(*g >= 100, "fires must be >= cooldown apart (gap {g}, ticks {fire_ticks:?})");
        }
    }

    /// A8 default: cooldown_ms = 0 reproduces the E4 latch cadence
    /// exactly (no additional spacing).
    #[test]
    fn saturation_trigger_cooldown_zero_is_noop() {
        let mut net = Network::new(NetworkConfig::default(), 2, 4, 1, 3);
        for n in net.neurons.iter_mut() {
            n.rate_hz = 40.0;
        }
        let mut trig = HomeostaticSaturation { rate_threshold_hz: 25.0, sustained_ms: 25, cooldown_ms: 0, over_since: None, last_birth: None };
        let mut ticks = Vec::new();
        for d in 0..100u64 {
            net.tick = Tick(1_000 + d);
            if trig.should_birth(&net, &Signals::default()).is_some() {
                ticks.push(1_000 + d);
            }
        }
        assert_eq!(ticks, vec![1_025, 1_051, 1_077], "cooldown 0 = E4 cadence");
    }

    #[test]
    fn persistent_error_trigger_fires() {
        let mut net = Network::new(NetworkConfig::default(), 2, 4, 1, 3);
        let mut trig = PersistentError::new();
        net.tick = Tick(1_000);
        let sig = Signals { prediction_error: 1.0, pe_mean: 0.0, pe_std: 0.1, novelty: 0.0 };
        // Sustained: arms at t=1000, sustained 5000 ⇒ fires at t=6000.
        assert!(trig.should_birth(&net, &sig).is_none(), "arms first");
        net.tick = Tick(6_000);
        let r = trig.should_birth(&net, &sig).expect("err=1.0 >> mu+2sigma=0.2 for 5s");
        assert_eq!(r.trigger, "persistent-error");
        assert!(!r.contributing.is_empty());
        assert!(!r.thresholds.is_empty());
    }

    #[test]
    fn persistent_error_clears_below_bound_and_reacts() {
        let mut net = Network::new(NetworkConfig::default(), 2, 4, 1, 3);
        let mut trig = PersistentError { sigma: 2.0, sustained_ms: 25, cooldown_ms: 0, over_since: None, last_birth: None };
        let hi = Signals { prediction_error: 1.0, pe_mean: 0.0, pe_std: 0.1, novelty: 0.0 };
        let lo = Signals { prediction_error: 0.0, pe_mean: 0.0, pe_std: 0.1, novelty: 0.0 };
        // Below bound: no arming.
        net.tick = Tick(100);
        assert!(trig.should_birth(&net, &lo).is_none());
        // Above bound: arms and fires after sustained_ms.
        net.tick = Tick(200);
        assert!(trig.should_birth(&net, &hi).is_none(), "armed at 200");
        net.tick = Tick(225);
        assert!(trig.should_birth(&net, &hi).is_some(), "fire at 200+25");
        // Drops below bound: re-arm allowed, fires again on next episode.
        net.tick = Tick(300);
        assert!(trig.should_birth(&net, &lo).is_none(), "cleared");
        net.tick = Tick(400);
        assert!(trig.should_birth(&net, &hi).is_none(), "re-armed at 400");
        net.tick = Tick(425);
        assert!(trig.should_birth(&net, &hi).is_some(), "second episode fires");
    }

    #[test]
    fn persistent_error_cooldown_limits_cadence() {
        let mut net = Network::new(NetworkConfig::default(), 2, 4, 1, 3);
        let mut trig = PersistentError { sigma: 2.0, sustained_ms: 25, cooldown_ms: 100, over_since: None, last_birth: None };
        let sig = Signals { prediction_error: 1.0, pe_mean: 0.0, pe_std: 0.1, novelty: 0.0 };
        let mut fire_ticks = Vec::new();
        for d in 0..300u64 {
            net.tick = Tick(1_000 + d);
            if trig.should_birth(&net, &sig).is_some() {
                fire_ticks.push(1_000 + d);
            }
        }
        assert!(fire_ticks.len() >= 3, "cooldown must not silence: {fire_ticks:?}");
        for w in fire_ticks.windows(2) {
            assert!(w[1] - w[0] >= 100, "fires must be >= cooldown apart");
        }
    }

    #[test]
    fn birth_wires_and_appends() {
        let mut net = Network::new(NetworkConfig::default(), 2, 6, 2, 5);
        let before_n = net.neurons.len();
        let before_s = net.live_synapse_count();
        let mut mon = StructuralMonitor::default();
        let id = mon.birth(&mut net);
        assert_eq!(id.0 as usize, before_n, "birth = next index in NeuronId space");
        assert_eq!(net.neurons.len(), before_n + 1);
        assert!(net.live_synapse_count() >= before_s);
        assert!(net.incoming[id.idx()].len() > 0);
        // Parallel adjacency invariant: every neuron has incoming+outgoing
        // slots (regression: outgoing.push was once missing, causing
        // index-out-of-bounds when the newborn spiked).
        assert_eq!(net.outgoing.len(), net.neurons.len());
        assert_eq!(net.incoming.len(), net.neurons.len());
    }

    #[test]
    fn dormancy_lifecycle_marks_and_recovers() {
        let mut net = Network::new(NetworkConfig::default(), 2, 4, 1, 3);
        let mut mon = StructuralMonitor::default();
        // Run quiet for 35 s sim-time ⇒ dormant candidates appear.
        for t in 0..35_000 {
            let _ = net.step(&InputFrame { tick: Tick(t), spikes: vec![] });
            let mut no_birth = NoBirth; let _ = mon.step(&mut net, &mut no_birth, &Signals::default());
        }
        // With zero activity, dormant_since set; nothing retired (<300 s).
        assert!(net.neurons.iter().any(|n| n.dormant_since.is_some()));
        // Force activity: rates spike ⇒ reactivated cleared to None.
        for n in net.neurons.iter_mut() {
            n.rate_hz = 5.0;
        }
        let mut no_birth = NoBirth; let _ = mon.step(&mut net, &mut no_birth, &Signals::default());
        assert!(net.neurons.iter().all(|n| n.dormant_since.is_none() || n.class == NeuronClass::Input));
    }

    #[test]
    fn retirement_prunes_synapses() {
        let mut net = Network::new(NetworkConfig::default(), 2, 4, 1, 3);
        let mon = StructuralMonitor::default();
        let victim = net.neurons.iter().find(|n| n.class == NeuronClass::Internal).unwrap().id;
        let had_out = !net.outgoing[victim.idx()].is_empty();
        mon.retire_neuron(&mut net, victim);
        assert!(net.outgoing[victim.idx()].is_empty());
        assert!(net.incoming[victim.idx()].is_empty());
        assert!(had_out || net.incoming[victim.idx()].is_empty());
    }

    /// U3/E4: `wiring_avoid_coactive` must route the newborn's afferents
    /// to the LOWEST-rate partners (away from the co-active pool).
    #[test]
    fn wiring_avoid_coactive_targets_lowest_rate() {
        let mut net = Network::new(NetworkConfig::default(), 2, 6, 2, 5);
        // Paint rate gradient across internal neurons (co-active proxy).
        let internals: Vec<NeuronId> = net
            .neurons
            .iter()
            .filter(|n| n.class == NeuronClass::Internal)
            .map(|n| n.id)
            .collect();
        for (k, id) in internals.iter().enumerate() {
            net.neurons[id.idx()].rate_hz = k as f32; // 0, 1, ..., 5
        }
        let mut mon = StructuralMonitor {
            wiring_synapses: 3,
            wiring_avoid_coactive: true,
            ..StructuralMonitor::default()
        };
        let id = mon.birth(&mut net);
        let partners: Vec<u32> = net
            .incoming[id.idx()]
            .iter()
            .map(|sid| net.synapses[sid.idx()].pre.0)
            .collect();
        assert_eq!(partners.len(), 3, "wiring_synapses afferents");
        // Invariant of the ascending-prefix rule: every chosen partner
        // must have rate <= every non-input neuron that was NOT chosen.
        let rate_of = |n: u32| {
            net.neurons
                .iter()
                .find(|x| x.id.0 == n)
                .map(|x| x.rate_hz)
                .unwrap_or(f32::INFINITY)
        };
        let unchosen: Vec<u32> = net
            .neurons
            .iter()
            .filter(|n| {
                n.class != NeuronClass::Input
                    && !n.retired
                    && n.id != id
                    && !partners.contains(&n.id.0)
            })
            .map(|n| n.id.0)
            .collect();
        for p in &partners {
            for q in &unchosen {
                assert!(
                    rate_of(*p) <= rate_of(*q),
                    "partner {p} (r={}) must not outrank unchosen {q} (r={})",
                    rate_of(*p),
                    rate_of(*q)
                );
            }
        }
    }

    /// U3/E4 identity guard: with wiring_avoid_coactive = false the wiring
    /// must reproduce the Phase 0 behavior exactly (highest-rate partners).
    #[test]
    fn wiring_phase0_identity_when_not_avoiding() {
        let mut net = Network::new(NetworkConfig::default(), 2, 6, 2, 5);
        let internals: Vec<NeuronId> = net
            .neurons
            .iter()
            .filter(|n| n.class == NeuronClass::Internal)
            .map(|n| n.id)
            .collect();
        for (k, id) in internals.iter().enumerate() {
            net.neurons[id.idx()].rate_hz = k as f32;
        }
        let mut mon = StructuralMonitor {
            wiring_synapses: 3,
            wiring_avoid_coactive: false,
            ..StructuralMonitor::default()
        };
        let id = mon.birth(&mut net);
        let partners: Vec<u32> = net
            .incoming[id.idx()]
            .iter()
            .map(|sid| net.synapses[sid.idx()].pre.0)
            .collect();
        assert_eq!(partners.len(), 3);
        let highest: Vec<u32> = internals.iter().rev().take(3).map(|n| n.0).collect();
        for p in &partners {
            assert!(
                highest.contains(p),
                "Phase 0 must pick highest-rate partners, got {p} in {partners:?}"
            );
        }
    }

    /// E4d: bidirectional birth creates 20 in + 20 out onto the SAME
    /// partners (fan-out-matched).
    #[test]
    fn bidirectional_birth_creates_matched_fanout() {
        let mut net = Network::new(NetworkConfig::default(), 2, 6, 2, 5);
        let internals: Vec<NeuronId> = net
            .neurons
            .iter()
            .filter(|n| n.class == NeuronClass::Internal)
            .map(|n| n.id)
            .collect();
        for (k, id) in internals.iter().enumerate() {
            net.neurons[id.idx()].rate_hz = k as f32;
        }
        let mut mon = StructuralMonitor {
            wiring_synapses: 3,
            wiring_avoid_coactive: true,
            wiring_bidirectional: true,
            ..StructuralMonitor::default()
        };
        let id = mon.birth(&mut net);
        let in_partners: Vec<u32> = net
            .incoming[id.idx()]
            .iter()
            .map(|sid| net.synapses[sid.idx()].pre.0)
            .collect();
        let out_partners: Vec<u32> = net
            .outgoing[id.idx()]
            .iter()
            .map(|sid| net.synapses[sid.idx()].post.0)
            .collect();
        assert_eq!(in_partners.len(), 3);
        assert_eq!(out_partners.len(), 3, "fan-out must match fan-in count");
        // Same partner SET both directions.
        let mut a = in_partners.clone();
        let mut b = out_partners.clone();
        a.sort_unstable();
        b.sort_unstable();
        assert_eq!(a, b, "bidirectional wiring targets the same partners");
        // All outgoing from the newborn, plastic.
        for sid in &net.outgoing[id.idx()] {
            assert_eq!(net.synapses[sid.idx()].pre, id);
            assert!(net.synapses[sid.idx()].plastic);
        }
    }

    /// E4d identity guard: wiring_bidirectional = false reproduces the
    /// exact E4c sink shape (only incoming synapses, same partner sets).
    #[test]
    fn bidirectional_off_reproduces_ec_cadence_wiring() {
        let mk = |bidirectional: bool| {
            let mut net = Network::new(NetworkConfig::default(), 2, 6, 2, 5);
            let internals: Vec<NeuronId> = net
                .neurons
                .iter()
                .filter(|n| n.class == NeuronClass::Internal)
                .map(|n| n.id)
                .collect();
            for (k, id) in internals.iter().enumerate() {
                net.neurons[id.idx()].rate_hz = k as f32;
            }
            let mon = StructuralMonitor {
                wiring_synapses: 3,
                wiring_avoid_coactive: true,
                wiring_bidirectional: bidirectional,
                ..StructuralMonitor::default()
            };
            // Deterministic: drive the same RNG draw by using the same
            // seed (id space identical), then record synapse topology.
            let id = mon.birth(&mut net);
            let inc: Vec<(u32, u32)> = net
                .incoming[id.idx()]
                .iter()
                .map(|sid| {
                    let s = &net.synapses[sid.idx()];
                    (s.pre.0, s.post.0)
                })
                .collect();
            (inc, net.outgoing[id.idx()].len())
        };
        let (inc_off, out_off) = mk(false);
        let (inc_on, out_on) = mk(true);
        // Fan-in identical; fan-out present iff bidirectional.
        assert_eq!(inc_off, inc_on, "fan-in must be identical");
        assert_eq!(out_off, 0, "E4c shape has no fan-out");
        assert_eq!(out_on, 3, "bidirectional adds matched fan-out");
    }

    /// E4e: low fan-in birth (wiring_synapses = 4) creates exactly 4
    /// afferents from the lowest-rate set and ZERO outgoing (no feedback).
    #[test]
    fn low_fanin_birth_creates_four_afferents_only() {
        let mut net = Network::new(NetworkConfig::default(), 2, 6, 2, 5);
        let internals: Vec<NeuronId> = net
            .neurons
            .iter()
            .filter(|n| n.class == NeuronClass::Internal)
            .map(|n| n.id)
            .collect();
        for (k, id) in internals.iter().enumerate() {
            net.neurons[id.idx()].rate_hz = k as f32;
        }
        let mon = StructuralMonitor {
            wiring_synapses: 4,
            wiring_avoid_coactive: true,
            wiring_bidirectional: false,
            ..StructuralMonitor::default()
        };
        let id = mon.birth(&mut net);
        assert_eq!(net.incoming[id.idx()].len(), 4, "exactly 4 afferents");
        assert!(net.outgoing[id.idx()].is_empty(), "no outgoing feedback");
        // Prefix invariant of the ascending rule: every chosen partner
        // must have rate <= every non-input neuron that was NOT chosen.
        let partners: Vec<u32> = net
            .incoming[id.idx()]
            .iter()
            .map(|sid| net.synapses[sid.idx()].pre.0)
            .collect();
        let rate_of = |n: u32| {
            net.neurons
                .iter()
                .find(|x| x.id.0 == n)
                .map(|x| x.rate_hz)
                .unwrap_or(f32::INFINITY)
        };
        let unchosen: Vec<u32> = net
            .neurons
            .iter()
            .filter(|n| {
                n.class != NeuronClass::Input
                    && !n.retired
                    && n.id != id
                    && !partners.contains(&n.id.0)
            })
            .map(|n| n.id.0)
            .collect();
        for p in &partners {
            for q in &unchosen {
                assert!(
                    rate_of(*p) <= rate_of(*q),
                    "partner {p} (r={}) must not outrank unchosen {q} (r={})",
                    rate_of(*p),
                    rate_of(*q)
                );
            }
        }
        // All plastic (same allocation semantics as E4).
        for sid in &net.incoming[id.idx()] {
            assert!(net.synapses[sid.idx()].plastic);
        }
    }

    /// E4f: any pre-registered fan-in value {8, 12, 16} yields exactly
    /// that many afferents and zero outgoing.
    #[test]
    fn mid_fanin_values_create_exact_afferent_counts() {
        for fan_in in [8usize, 12, 16] {
            let mut net = Network::new(NetworkConfig::default(), 2, 20, 2, 11);
            let n_candidates = net
                .neurons
                .iter()
                .filter(|n| n.class != NeuronClass::Input && !n.retired)
                .count();
            assert!(
                n_candidates >= fan_in,
                "test net must have enough candidates for fan-in {fan_in}"
            );
            let mon = StructuralMonitor {
                wiring_synapses: fan_in,
                wiring_avoid_coactive: true,
                wiring_bidirectional: false,
                ..StructuralMonitor::default()
            };
            let id = mon.birth(&mut net);
            assert_eq!(net.incoming[id.idx()].len(), fan_in, "fan-in {fan_in}");
            assert!(net.outgoing[id.idx()].is_empty(), "no fan-out for {fan_in}");
        }
    }
}
