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
    /// ANIMA v2 (docs/anima-v2-protocol.md): structural plasticity
    /// mechanisms M1–M6. Absent/false = E1–E4f behavior exactly.
    #[serde(default)]
    pub v2: Option<V2Section>,
    /// ANIMA E6 (docs/anima-e6-protocol.md §3): rate balancing.
    /// Absent or enable=false ⇒ exact v2/v3 behavior.
    #[serde(default)]
    pub e6: Option<E6Section>,
}

/// Frozen ANIMA E6 parameters (docs/anima-e6-protocol.md §3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E6Section {
    #[serde(default)]
    pub enable: bool,
    /// EMA smoothing α (frozen 1/25).
    #[serde(default = "default_e6_alpha")]
    pub alpha: f32,
    /// Initial φ per channel (frozen 0.02 events/tick).
    #[serde(default = "default_e6_phi_init")]
    pub phi_init: f32,
    /// Floor for φ (frozen 0.001 events/tick).
    #[serde(default = "default_e6_phi_min")]
    pub phi_min: f32,
    /// β clamp bounds (frozen [0.1, 10]).
    #[serde(default = "default_e6_beta_min")]
    pub beta_min: f32,
    #[serde(default = "default_e6_beta_max")]
    pub beta_max: f32,
}

fn default_variant_block() -> u64 {
    1
}

fn cfg_one_u32() -> u32 { 1 }
fn cfg_v23_epoch_windows() -> u32 { 40 }
fn cfg_bool_false(b: &bool) -> bool { !*b }
fn cfg_clla_p_max_frac() -> f32 { 0.75 }
fn cfg_clla_w_consolidate_min() -> f32 { 0.05 }

fn default_e6_alpha() -> f32 {
    1.0 / 25.0
}
fn default_e6_phi_init() -> f32 {
    0.02
}
fn default_e6_phi_min() -> f32 {
    0.001
}
fn default_e6_beta_min() -> f32 {
    0.1
}
fn default_e6_beta_max() -> f32 {
    10.0
}

/// Frozen ANIMA v2 protocol parameters (§2–§8 of the protocol).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2Section {
    /// M1: dense-weak initialization + all v2 mechanisms.
    pub enabled: bool,
    /// Protocol §13/14 control + ablation arms. All default false
    /// (= mechanism ON); each arm config sets exactly one of these.
    #[serde(default)]
    pub disable_m2: bool,
    #[serde(default)]
    pub disable_m3_m4: bool,
    #[serde(default)]
    pub disable_m5: bool,
    #[serde(default)]
    pub disable_m6: bool,
    // M1
    pub p_in: f32,
    pub w_in_lo: f32,
    pub w_in_hi: f32,
    pub p_rec: f32,
    pub w_rec_lo: f32,
    pub w_rec_hi: f32,
    // M2
    pub t_e: f32,
    /// V2.3: trace-partitioned M2 (docs/v2_3-design.md). 1 (default) =
    /// shared budget, identity. >1 = capacity-matched write-epoch buckets.
    #[serde(default = "cfg_one_u32")]
    pub m2_buckets: u32,
    /// V2.3: epoch length in structural windows (default 40 = 4 s).
    #[serde(default = "cfg_v23_epoch_windows")]
    pub m2_epoch_windows: u32,
    // CLLA (docs/anima-clla-protocol.md): consolidation-locked allocation.
    /// Master flag: false = byte-identical pre-CLLA behavior (identity).
    #[serde(default, skip_serializing_if = "cfg_bool_false")]
    pub assembly_protect: bool,
    /// Protected-mass cap as fraction of t_e (protocol frozen: 0.75).
    #[serde(default = "cfg_clla_p_max_frac")]
    pub p_max_frac: f32,
    /// Minimum candidate weight at permanence to consolidate (protocol
    /// frozen: 0.05 = theta_permanent).
    #[serde(default = "cfg_clla_w_consolidate_min")]
    pub w_consolidate_min: f32,
    /// CLLA allocation rule (docs/x-clla-allocation-rule.md): gates M3
    /// candidate accumulation by protected-input-current fraction.
    #[serde(default, skip_serializing_if = "cfg_bool_false")]
    pub alloc_residual: bool,
    /// Dormant-candidate reserve (docs/x-clla-dormant-reserve.md): retain
    /// ever-coactive candidate pre-associations within c_slots.
    #[serde(default, skip_serializing_if = "cfg_bool_false")]
    pub dormant_reserve: bool,
    /// Local recruitment gain (frozen k_g = 8.0 mechanism constant, code-
    /// side; docs/x-clla-recruitment-design-review.md §10). false = identity.
    #[serde(default, skip_serializing_if = "cfg_bool_false")]
    pub recruit_gain: bool,
    /// Phase II-A D-core context tracks (docs/x-phase2-a-protocol.md).
    /// false = identity.
    #[serde(default, skip_serializing_if = "cfg_bool_false")]
    pub d_core: bool,
    /// Phase II-AR candidate E (docs/x-phase2-ar-protocol.md). Requires d_core.
    #[serde(default, skip_serializing_if = "cfg_bool_false")]
    pub d_claim: bool,
    /// Phase III sparse-commit (docs/phase3/sparse-commit-protocol.md).
    #[serde(default, skip_serializing_if = "cfg_bool_false")]
    pub d_sparse: bool,
    // M3
    pub c_slots: usize,
    pub w_c_init: f32,
    pub delta_perm: f32,
    pub decay_c: f32,
    pub theta_permanent: f32,
    pub w_c_permanent: f32,
    pub theta_die: f32,
    pub p_cand_in: f32,
    pub p_cand_rec: f32,
    // M4
    pub theta_prune: f32,
    pub prune_windows: u64,
    // M5
    pub b_e: usize,
    pub b_i: usize,
    // M6
    pub p_inh: f32,
    pub w_inh_lo: f32,
    pub w_inh_hi: f32,
    pub a_inh: f32,
    pub decay_inh: f32,
    pub w_inh_max: f32,
    // window
    pub window_ticks: u64,
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
    /// V2.1 (docs/v2_1-spec.md): slow depolarizing intrinsic state.
    /// slow_state_beta per spike; slow_state_tau_ms decay. Defaults
    /// absent => beta 0 => V2 identity.
    #[serde(default)]
    pub slow_state_beta: f32,
    #[serde(default = "default_slow_tau_ms")]
    pub slow_state_tau_ms: f32,
    /// X-series: drive-gated slow-state write. false (default) = exact
    /// ungated write (byte-identical path).
    #[serde(default)]
    pub slow_state_beta_drive: bool,
    /// V2.2 (docs/v2_2-spec.md §1.3): bistable latch. All defaults =
    /// identity (latch off => V2.1 exactly).
    #[serde(default)]
    pub latch_enable: bool,
    #[serde(default = "default_v22_theta_mean")]
    pub theta_rel_mean: f32,
    #[serde(default)]
    pub theta_rel_sd: f32,
    #[serde(default = "default_v22_plateau_mean")]
    pub u_plateau_rel_mean: f32,
    #[serde(default)]
    pub u_plateau_rel_sd: f32,
    #[serde(default)]
    pub tau_het_rel_sd: f32,
    #[serde(default = "default_v22_phi_rel")]
    pub phi_rel: f32,
    #[serde(default)]
    pub eta_rel: f32,
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
    /// E4d: newborn also gets 20 outgoing efferents onto the SAME
    /// allocated partners (fan-out-matched); false = E4c sink shape.
    #[serde(default)]
    pub wiring_bidirectional: bool,
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

/// One counterbalanced phase variant (docs/anima-e11-protocol.md §6):
/// a full tiling of the pattern duration; the variant is selected by
/// rep parity (registered semantics).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseVariantSpec {
    pub phases: Vec<PhaseSpec>,
}

/// One within-presentation phase (docs/anima-e9-protocol.md §9): the
/// pattern drives `channel_ids` at `rate_hz` during [from_ms, to_ms).
/// Environment-only construction — the organism sees only the spike
/// trains. Phases MUST tile the pattern's [0, duration_ms) exactly
/// (contiguous, gap-free, first from = 0, last to = duration_ms).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseSpec {
    pub from_ms: u64,
    pub to_ms: u64,
    pub channel_ids: Vec<u32>,
    pub rate_hz: f32,
}

/// D9 pattern spec: synthetic config-defined stimulus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternSpec {
    pub id: String,
    /// Input groups this pattern drives (e.g. ["A"], ["A","C"]).
    pub channels: Vec<String>,
    /// v3 (anima-v3-protocol.md §2): explicit channel ids for patterns
    /// whose active set is not expressible as group-label blocks
    /// (overlapping categories). When Some, it wins over `channels`
    /// (configs MUST NOT populate both). Environment-only: no
    /// organism/RNG/seed behavior changes.
    #[serde(default)]
    pub channel_ids: Option<Vec<u32>>,
    /// E9 (docs/anima-e9-protocol.md §9): optional within-presentation
    /// phases. Mutually exclusive with `channels`/`channel_ids`; when
    /// Some, the pattern's active channels = union over phases and each
    /// phase generates its own deterministic Poisson streams (seed tuple
    /// extended by the phase index — only for phase configs; phase-less
    /// configs keep the exact existing derivation). Absent ⇒ byte-
    /// identical behavior for every existing config.
    #[serde(default)]
    pub phases: Option<Vec<PhaseSpec>>,
    /// E11/E12 (docs/anima-e11-protocol.md §2, anima-e12-protocol.md
    /// §2): optional counterbalanced phase variants. Mutually exclusive
    /// with phases/channels/channel_ids. When Some, the pattern's
    /// phases for presentation with rep index r are
    /// `variants[(r / variant_block) % variants.len()]` (registered
    /// semantics; E11 uses two variants with variant_block 1 — the
    /// E11 60/60 interleave; E12 uses variant_block 60 — the blocked
    /// 60 SEQ then 60 REV history). Environment-only; variant selection
    /// consumes NO RNG. Absent => byte-identical behavior for every
    /// existing config.
    #[serde(default)]
    pub phase_variants: Option<Vec<PhaseVariantSpec>>,
    /// E12 (docs/anima-e12-protocol.md §2): variant-block size for the
    /// registered selection rule. Default 1 => the E11 rule
    /// `rep % variants.len()` byte-identically.
    #[serde(default = "default_variant_block")]
    pub variant_block: u64,
    pub rate_hz: f32,
    pub duration_ms: u64,
    pub jitter_ms: f32,
}

/// Curriculum stage: ordered presentations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageSpec {
    pub id: String,
    /// Pattern ids presented in this stage.
    #[serde(default)]
    pub present: Vec<String>,
    /// Repetitions per pattern.
    #[serde(default)]
    pub reps: usize,
    /// "interleaved" | "blocked".
    #[serde(default = "default_stage_order")]
    pub order: String,
    /// Off-time between presentations (ms).
    #[serde(default)]
    pub off_ms: u64,
    /// Silence-probe stage: no pattern, just wait (e.g. S0).
    #[serde(default)]
    pub silence_ms: Option<u64>,
    /// E18 (docs/anima-e18-protocol.md): trial-block stage.
    /// mode = "trials": each trial = one antecedent (drawn from
    /// `antecedents`, balanced seeded-random per `balance_window`
    /// trials) -> gap_ms silence -> the `probe` pattern -> iti_ms
    /// silence. Deterministic; consumes the environment's seeded RNG;
    /// carries NO trial-type labels (the schedule representation has
    /// no field saying which antecedent a probe follows).
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub trials: Option<u64>,
    #[serde(default)]
    pub antecedents: Option<Vec<String>>,
    #[serde(default)]
    pub probe: Option<String>,
    #[serde(default)]
    pub gap_ms: Option<u64>,
    #[serde(default)]
    pub iti_ms: Option<u64>,
    #[serde(default)]
    pub balance_window: Option<u64>,
}

/// U1 default adaptation tau (ms): slow enough to integrate bursts,
/// fast enough to recover between 500 ms presentations.
fn default_adaptation_tau() -> f32 {
    200.0
}

impl ExpConfig {
    pub fn parse(path: &std::path::Path) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let cfg: ExpConfig = toml::from_str(&raw).map_err(|e| e.to_string())?;
        // v3 (anima-v3-protocol.md §2): channel_ids must be in-bounds and
        // mutually exclusive with group labels.
        for p in &cfg.pattern {
            if let Some(ids) = &p.channel_ids {
                if !p.channels.is_empty() {
                    return Err(format!(
                        "pattern '{}': channels and channel_ids are mutually exclusive",
                        p.id
                    ));
                }
                for &c in ids {
                    if c as usize >= cfg.organism.n_input_channels {
                        return Err(format!(
                            "pattern '{}': channel {} out of range (n_input_channels = {})",
                            p.id, c, cfg.organism.n_input_channels
                        ));
                    }
                }
            }
        }
        // E9 (docs/anima-e9-protocol.md §9): phase validation.
        for p in &cfg.pattern {
            if let Some(phases) = &p.phases {
                if !p.channels.is_empty() || p.channel_ids.is_some() || p.phase_variants.is_some() {
                    return Err(format!(
                        "pattern '{}': phases are mutually exclusive with channels/channel_ids/phase_variants",
                        p.id
                    ));
                }
                let mut prev_to: Option<u64> = None;
                for (i, ph) in phases.iter().enumerate() {
                    if ph.to_ms <= ph.from_ms {
                        return Err(format!("pattern '{}': phase {i} has to_ms <= from_ms", p.id));
                    }
                    if ph.to_ms > p.duration_ms {
                        return Err(format!("pattern '{}': phase {i} exceeds duration", p.id));
                    }
                    if let Some(pt) = prev_to {
                        if ph.from_ms != pt {
                            return Err(format!(
                                "pattern '{}': phases must tile [0, {}] contiguously (gap at phase {i})",
                                p.id, p.duration_ms
                            ));
                        }
                    } else if ph.from_ms != 0 {
                        return Err(format!("pattern '{}': first phase must start at 0", p.id));
                    }
                    for &c in &ph.channel_ids {
                        if c as usize >= cfg.organism.n_input_channels {
                            return Err(format!(
                                "pattern '{}': phase {i} channel out of range",
                                p.id
                            ));
                        }
                    }
                    prev_to = Some(ph.to_ms);
                }
                if let Some(pt) = prev_to {
                    if pt != p.duration_ms {
                        return Err(format!(
                            "pattern '{}': phases must tile [0, {}] exactly (ends at {pt})",
                            p.id, p.duration_ms
                        ));
                    }
                }
            }
            if let Some(variants) = &p.phase_variants {
                if !p.channels.is_empty() || p.channel_ids.is_some() || p.phases.is_some() {
                    return Err(format!(
                        "pattern '{}': phase_variants are mutually exclusive with phases/channels/channel_ids",
                        p.id
                    ));
                }
                if variants.len() < 2 {
                    return Err(format!(
                        "pattern '{}': phase_variants needs >= 2 variants (E11 registered semantics)",
                        p.id
                    ));
                }
                if p.variant_block == 0 {
                    return Err(format!("pattern '{}': variant_block must be >= 1", p.id));
                }
                for (vi, variant) in variants.iter().enumerate() {
                    let mut prev_to: Option<u64> = None;
                    for (pi, ph) in variant.phases.iter().enumerate() {
                        if ph.to_ms <= ph.from_ms {
                            return Err(format!(
                                "pattern '{}': variant {vi} phase {pi} has to_ms <= from_ms",
                                p.id
                            ));
                        }
                        if ph.to_ms > p.duration_ms {
                            return Err(format!(
                                "pattern '{}': variant {vi} phase {pi} exceeds duration",
                                p.id
                            ));
                        }
                        if let Some(pt) = prev_to {
                            if ph.from_ms != pt {
                                return Err(format!(
                                    "pattern '{}': variant {vi} must tile [0, {}] contiguously",
                                    p.id, p.duration_ms
                                ));
                            }
                        } else if ph.from_ms != 0 {
                            return Err(format!("pattern '{}': variant {vi} first phase must start at 0", p.id));
                        }
                        for &c in &ph.channel_ids {
                            if c as usize >= cfg.organism.n_input_channels {
                                return Err(format!(
                                    "pattern '{}': variant {vi} phase {pi} channel out of range",
                                    p.id
                                ));
                            }
                        }
                        prev_to = Some(ph.to_ms);
                    }
                    if let Some(pt) = prev_to {
                        if pt != p.duration_ms {
                            return Err(format!(
                                "pattern '{}': variant {vi} must tile [0, {}] exactly (ends at {pt})",
                                p.id, p.duration_ms
                            ));
                        }
                    } else {
                        return Err(format!("pattern '{}': variant {vi} has no phases", p.id));
                    }
                }
            }
        }
        // E6 (docs/anima-e6-protocol.md §3): requires the v2 layer; frozen
        // param sanity — values must be inside the registered regimes.
        if let Some(e) = &cfg.e6 {
            if e.enable {
                if !cfg.v2.as_ref().is_some_and(|v| v.enabled) {
                    return Err("E6 requires [v2] enabled".into());
                }
                if !(e.alpha > 0.0 && e.alpha <= 1.0) {
                    return Err("E6 alpha must be in (0, 1]".into());
                }
                if e.phi_min >= e.phi_init || e.phi_min < 0.0 {
                    return Err("E6 phi_min must be < phi_init".into());
                }
                if e.beta_min > 1.0 || e.beta_max < 1.0 || e.beta_min >= e.beta_max {
                    return Err("E6 beta clamp must bracket 1.0".into());
                }
            }
        }
        Ok(cfg)
    }
}

fn default_stage_order() -> String {
    "interleaved".into()
}

fn default_slow_tau_ms() -> f32 {
    2500.0
}
fn default_v22_theta_mean() -> f32 { 2.0 }
fn default_v22_plateau_mean() -> f32 { 0.9 }
fn default_v22_phi_rel() -> f32 { 0.5 }
