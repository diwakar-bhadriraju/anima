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
pub struct HomeostaticSaturation {
    pub rate_threshold_hz: f32,
    pub sustained_ms: u64,
    over_since: Option<Tick>,
}

impl HomeostaticSaturation {
    pub fn new() -> Self {
        Self { rate_threshold_hz: 25.0, sustained_ms: 2_000, over_since: None }
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
pub struct PersistentError {
    pub sigma: f32,
    pub sustained_ms: u64,
    over_since: Option<Tick>,
}

impl PersistentError {
    pub fn new() -> Self {
        Self { sigma: 2.0, sustained_ms: 5_000, over_since: None }
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
            let since = self.over_since?;
            if net.tick.0.saturating_sub(since.0) >= self.sustained_ms {
                return Some(
                    Reason::new("persistent-error")
                        .factor("prediction-error", signals.prediction_error)
                        .factor("bound", bound)
                        .threshold("sigma", self.sigma)
                        .threshold("sustained-ms", self.sustained_ms as f32),
                );
            }
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
            Box::new(t)
        }
        "persistent-error" => Box::new(PersistentError::new()),
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
}

impl Default for StructuralMonitor {
    fn default() -> Self {
        Self {
            dormancy_rate_hz: 0.1,
            dormancy_ms: 30_000,
            recovery_rate_hz: 1.0,
            retirement_ms: 300_000,
            wiring_synapses: 20,
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
            dormant_since: None,
            retired: false,
        });
        net.incoming.push(Vec::new());
        net.outgoing.push(Vec::new());
        // Wire to most-recently-coactive partners: here, the highest-rate
        // non-input neurons (rate EMA is our coactivity proxy), seeded.
        let mut partners: Vec<(NeuronId, f32)> = net
            .neurons
            .iter()
            .filter(|n| n.class != NeuronClass::Input && !n.retired && n.id != id)
            .map(|n| (n.id, n.rate_hz))
            .collect();
        partners.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let take = self.wiring_synapses.min(partners.len());
        for &(partner, _) in &partners[..take] {
            let w = net.rng.gen::<f32>() * net.cfg.w_init + 0.05;
            net.add_synapse(partner, id, w, true, net.tick);
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
    fn persistent_error_trigger_fires() {
        let mut net = Network::new(NetworkConfig::default(), 2, 4, 1, 3);
        let mut trig = PersistentError::new();
        trig.over_since = Some(Tick(0));
        net.tick = Tick(6_000);
        let sig = Signals { prediction_error: 1.0, pe_mean: 0.0, pe_std: 0.1, novelty: 0.0 };
        let r = trig.should_birth(&net, &sig).expect("err=1.0 >> mu+2sigma=0.2 for 6s");
        assert_eq!(r.trigger, "persistent-error");
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
}
