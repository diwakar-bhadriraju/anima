//! Event envelope + kinds (D6). Every consumer — recorder, viz, analyzer —
//! depends on this schema.
//!
//! Serialization contract: `t` = u64 sim-ms, `seq` = u64 monotonic, ids =
//! u64, floats = f32-valued JSON numbers with NaN/∞ → `null` (analyzer
//! treats `null` as missing).

use serde::{Deserialize, Serialize};

use anima_core::network::{NeuronId, SimMs, SynapseId};

/// Wrap an f32 for the wire: NaN/±∞ serialize to `null` instead of
/// invalid JSON.
pub fn f32_json(v: f32) -> Option<f32> {
    if v.is_finite() { Some(v) } else { None }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope {
    pub seq: u64,
    /// Simulation time in ms (u64).
    pub t: SimMs,
    pub exp_id: String,
    pub kind: EventKind,
    pub payload: Payload,
}

/// Payload variants — one per kind, tagged to match `EventKind`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "k", rename_all = "snake_case")]
pub enum Payload {
    RunStarted {
        config_hash: String,
        seed: u64,
        params: serde_json::Value,
    },
    RunEnded {
        reason: String,
    },
    TickStats {
        mean_rate_hz: Option<f32>,
        active_neurons: u64,
        spikes: u64,
    },
    Spike {
        n: NeuronId,
    },
    OutputActivity {
        n: NeuronId,
    },
    StimulusPresented {
        pattern_id: String,
        stage: String,
    },
    SynapseCreated {
        syn: SynapseId,
        pre: NeuronId,
        post: NeuronId,
        w: Option<f32>,
        reason: ReasonPayload,
    },
    SynapsePruned {
        syn: SynapseId,
        reason: ReasonPayload,
    },
    SynapseStrengthened {
        syn: SynapseId,
        delta: Option<f32>,
    },
    SynapseWeakened {
        syn: SynapseId,
        delta: Option<f32>,
    },
    NeuronCreated {
        n: NeuronId,
        reason: ReasonPayload,
    },
    NeuronDormant {
        n: NeuronId,
        reason: ReasonPayload,
    },
    NeuronReactivated {
        n: NeuronId,
        reason: ReasonPayload,
    },
    NeuronRetired {
        n: NeuronId,
        reason: ReasonPayload,
    },
    PredictionError {
        value: Option<f32>,
    },
    NoveltySignal {
        value: Option<f32>,
    },
    ResourceUsage {
        neurons: u64,
        synapses: u64,
        spikes_window: u64,
        metabolic_cost: Option<f32>,
    },
    Failure {
        kind: String,
        detail: String,
    },
}

/// Causal metadata carried by every structural event (§15).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasonPayload {
    pub trigger: String,
    pub contributing: Vec<(String, Option<f32>)>,
    pub thresholds: Vec<(String, Option<f32>)>,
}

impl ReasonPayload {
    pub fn simple(trigger: &str) -> Self {
        Self {
            trigger: trigger.to_string(),
            contributing: Vec::new(),
            thresholds: Vec::new(),
        }
    }
    pub fn from_core(r: &anima_core::structural::Reason) -> Self {
        let cnv = |v: f32| f32_json(v);
        Self {
            trigger: r.trigger.clone(),
            contributing: r.contributing.iter().map(|(k, v)| (k.clone(), cnv(*v))).collect(),
            thresholds: r.thresholds.iter().map(|(k, v)| (k.clone(), cnv(*v))).collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EventKind {
    RunStarted,
    RunEnded,
    TickStats,
    Spike,
    OutputActivity,
    StimulusPresented,
    SynapseCreated,
    SynapsePruned,
    SynapseStrengthened,
    SynapseWeakened,
    NeuronCreated,
    NeuronDormant,
    NeuronReactivated,
    NeuronRetired,
    PredictionError,
    NoveltySignal,
    ResourceUsage,
    Failure,
}

impl EventKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EventKind::RunStarted => "run-started",
            EventKind::RunEnded => "run-ended",
            EventKind::TickStats => "tick-stats",
            EventKind::Spike => "spike",
            EventKind::OutputActivity => "output-activity",
            EventKind::StimulusPresented => "stimulus-presented",
            EventKind::SynapseCreated => "synapse-created",
            EventKind::SynapsePruned => "synapse-pruned",
            EventKind::SynapseStrengthened => "synapse-strengthened",
            EventKind::SynapseWeakened => "synapse-weakened",
            EventKind::NeuronCreated => "neuron-created",
            EventKind::NeuronDormant => "neuron-dormant",
            EventKind::NeuronReactivated => "neuron-reactivated",
            EventKind::NeuronRetired => "neuron-retired",
            EventKind::PredictionError => "prediction-error",
            EventKind::NoveltySignal => "novelty-signal",
            EventKind::ResourceUsage => "resource-usage",
            EventKind::Failure => "failure",
        }
    }
}

impl EventKind {
    pub fn kind_of(payload: &Payload) -> EventKind {
        match payload {
            Payload::RunStarted { .. } => EventKind::RunStarted,
            Payload::RunEnded { .. } => EventKind::RunEnded,
            Payload::TickStats { .. } => EventKind::TickStats,
            Payload::Spike { .. } => EventKind::Spike,
            Payload::OutputActivity { .. } => EventKind::OutputActivity,
            Payload::StimulusPresented { .. } => EventKind::StimulusPresented,
            Payload::SynapseCreated { .. } => EventKind::SynapseCreated,
            Payload::SynapsePruned { .. } => EventKind::SynapsePruned,
            Payload::SynapseStrengthened { .. } => EventKind::SynapseStrengthened,
            Payload::SynapseWeakened { .. } => EventKind::SynapseWeakened,
            Payload::NeuronCreated { .. } => EventKind::NeuronCreated,
            Payload::NeuronDormant { .. } => EventKind::NeuronDormant,
            Payload::NeuronReactivated { .. } => EventKind::NeuronReactivated,
            Payload::NeuronRetired { .. } => EventKind::NeuronRetired,
            Payload::PredictionError { .. } => EventKind::PredictionError,
            Payload::NoveltySignal { .. } => EventKind::NoveltySignal,
            Payload::ResourceUsage { .. } => EventKind::ResourceUsage,
            Payload::Failure { .. } => EventKind::Failure,
        }
    }
}

/// Builder: assigns monotonic `seq` and the event kind from the payload.
pub struct EventBuilder {
    pub seq: u64,
    pub exp_id: String,
}

impl EventBuilder {
    pub fn new(exp_id: &str) -> Self {
        Self { seq: 0, exp_id: exp_id.to_string() }
    }

    pub fn build(&mut self, t: SimMs, payload: Payload) -> Envelope {
        let kind = EventKind::kind_of(&payload);
        let env = Envelope { seq: self.seq, t, exp_id: self.exp_id.clone(), kind, payload };
        self.seq += 1;
        env
    }
}
