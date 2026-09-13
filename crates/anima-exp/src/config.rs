//! Experiment config: single TOML per experiment (step 9).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpConfig {
    pub run: RunSection,
    pub organism: OrganismSection,
    pub plasticity: PlasticitySection,
    pub structural: StructuralSection,
    pub resources: ResourceSection,
    /// Named patterns (D9: synthetic, config-defined).
    #[serde(default)]
    pub pattern: Vec<PatternSpec>,
    /// Ordered curriculum stages.
    #[serde(default)]
    pub stage: Vec<StageSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSection {
    pub exp_id: String,
    pub seed: u64,
    pub viz_port: u16,
    /// Default speed: target ticks/s (1–10000).
    pub ticks_per_sec: f32,
    /// Broadcast/WS decimation.
    pub stats_decimate: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganismSection {
    pub n_input_channels: usize,
    /// Channels per input group (A/B/C × 8 for E1).
    pub group_size: usize,
    pub n_internal: usize,
    pub n_output: usize,
    pub connectivity: f32,
    pub w_init: f32,
    pub amplitude: f32,
    /// U1 (E3): spike-frequency adaptation current decay tau (ms).
    #[serde(default = "default_adaptation_tau")]
    pub adaptation_tau_ms: f32,
    /// U1 (E3): adaptation current per spike; 0 = bare LIF (E1 semantics).
    #[serde(default)]
    pub adaptation_gain: f32,
    /// U1-inhibition (E3b): inhibitory current deposited per co-active
    /// spiker onto other same-tick spikers; 0 = E3 semantics.
    #[serde(default)]
    pub inhibition_gain: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlasticitySection {
    /// "stdp-pairwise" (E1 default, D7).
    pub rule: String,
    pub tau_plus_ms: f32,
    pub tau_minus_ms: f32,
    pub a_plus: f32,
    pub a_minus: f32,
    pub w_min: f32,
    pub w_max: f32,
    pub decay: f32,
    /// Prune thresholds (silent-synapse).
    pub silence_w: f32,
    pub silence_ticks: u64,
    pub min_age_ticks: u64,
    /// Learning gate: "always" in E1 (U4a).
    pub gate: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralSection {
    /// "none" | "homeostatic-saturation" | "persistent-error".
    pub birth_trigger: String,
    /// HomeostaticSaturation trigger params (crafted configs/tests; E4).
    #[serde(default)]
    pub trigger_rate_hz: Option<f32>,
    #[serde(default)]
    pub trigger_sustained_ms: Option<u64>,
    /// A8/E4b: minimum sim-ms between homeostatic-saturation births
    /// (0 = no limit, E4/E3 behavior).
    #[serde(default)]
    pub trigger_cooldown_ms: Option<u64>,
    pub dormancy_rate_hz: f32,
    pub dormancy_ms: u64,
    pub recovery_rate_hz: f32,
    pub retirement_ms: u64,
    pub wiring_synapses: usize,
    /// U3 (E4): newborn afferents target the LOWEST rate-EMA neurons
    /// (away from the shared co-active pool) when true; Phase 0 keeps
    /// highest-rate partners when false/absent.
    #[serde(default)]
    pub wiring_avoid_coactive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSection {
    pub max_neurons: usize,
    pub max_synapses: usize,
    pub births_per_window: usize,
    pub runaway_rate_hz: f32,
    pub runaway_sustained_ms: u64,
    pub fragmentation_min_component: f32,
}

/// D9 pattern spec: synthetic config-defined stimulus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSpec {
    pub id: String,
    /// Input groups this pattern drives (e.g. ["A"], ["A","C"]).
    pub channels: Vec<String>,
    pub rate_hz: f32,
    pub duration_ms: u64,
    pub jitter_ms: f32,
}

/// Curriculum stage: ordered presentations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageSpec {
    pub id: String,
    /// Pattern ids presented in this stage.
    pub present: Vec<String>,
    /// Repetitions per pattern.
    pub reps: usize,
    /// "interleaved" | "blocked".
    pub order: String,
    /// Off-time between presentations (ms).
    pub off_ms: u64,
    /// Silence-probe stage: no pattern, just wait (e.g. S0).
    #[serde(default)]
    pub silence_ms: Option<u64>,
}

/// U1 default adaptation tau (ms): slow enough to integrate bursts,
/// fast enough to recover between 500 ms presentations.
fn default_adaptation_tau() -> f32 {
    200.0
}

impl ExpConfig {
    pub fn parse(path: &std::path::Path) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        toml::from_str(&raw).map_err(|e| e.to_string())
    }
}
