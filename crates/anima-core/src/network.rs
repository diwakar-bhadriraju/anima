//! Organism state and deterministic stepping.
//!
//! The `Network` owns neurons, synapses, and a seeded RNG. `step` is pure
//! with respect to RNG state given the same stimulus script: same seed +
//! same input ⇒ identical event stream. Wall-clock never enters here.

use serde::{Deserialize, Serialize};

use rand::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;

/// Simulation tick. 1 tick = 1 ms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tick(pub u64);

/// Simulation time in milliseconds (reported as `t` in telemetry).
pub type SimMs = u64;

macro_rules! id_newtype {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        pub struct $name(pub u32);

        impl $name {
            pub fn idx(self) -> usize {
                self.0 as usize
            }
        }
        impl From<u32> for $name {
            fn from(v: u32) -> Self {
                Self(v)
            }
        }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

id_newtype!(NeuronId);
id_newtype!(SynapseId);
id_newtype!(InputChannelId);

/// Role tag, not a cell type: output neurons are ordinary LIF internal-class
/// neurons whose spikes are additionally emitted as `OutputActivity`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NeuronClass {
    Input,
    Internal,
    Output,
}

/// LIF parameters (fixed tick dt = 1 ms).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LIFParams {
    pub v_rest: f32,
    pub tau_m: f32,      // ms
    pub v_th: f32,
    pub v_reset: f32,
    pub refractory: u64, // ticks
    pub tau_syn: f32,    // ms, exponential current decay
}

impl Default for LIFParams {
    fn default() -> Self {
        Self {
            v_rest: 0.0,
            tau_m: 20.0,
            v_th: 1.0,
            v_reset: 0.0,
            refractory: 2,
            tau_syn: 5.0,
        }
    }
}

fn default_slow_tau() -> f32 {
    2500.0
}

fn one_f32() -> f32 {
    1.0
}

fn v22_theta_mean() -> f32 { 2.0 }
fn v22_plateau_mean() -> f32 { 0.9 }
fn v22_phi_rel() -> f32 { 0.5 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neuron {
    pub id: NeuronId,
    pub class: NeuronClass,
    /// Creation tick (for age computations; grown neurons continue the id space).
    pub born: Tick,
    /// Input channel this neuron feeds (set when class == Input).
    pub channel: Option<InputChannelId>,
    /// Membrane potential.
    pub v: f32,
    /// Remaining refractory ticks (0 = excitable).
    pub refractory_until: Tick,
    /// Exponential synaptic current accumulator.
    pub i_syn: f32,
    /// External current injection (A), used by probes/tests.
    pub i_ext: f32,
    /// EMA firing rate in Hz (tau = 1 s) — cheap per-tick estimate for
    /// telemetry and homeostatic machinery. Updated every tick.
    pub rate_hz: f32,
    /// U1: spike-frequency adaptation current (hyperpolarizing). Decays
    /// with adaptation_tau_ms; injected per spike by adaptation_gain.
    pub i_adapt: f32,
    /// V2.1: slow depolarizing intrinsic state (docs/v2_1-spec.md).
    /// u += beta per spike; decays with slow_state_tau_ms; injected
    /// into dv as +u. beta = 0 (default) => bit-identical V2.
    #[serde(default)]
    pub u_slow: f32,
    /// V2.2 (docs/v2_2-spec.md): bistable latch state.
    /// 0 = unlatched, 1 = latched. SET when decayed u >= theta_i;
    /// RESET when decayed u < phi_i. Plateau U_i adds to u in dv.
    #[serde(default)]
    pub z_latch: u8,
    /// V2.2 G2: per-neuron heterogeneity draws (LogNormal, frozen
    /// spec §2; identity 1.0 when heterogeneity sd = 0 — NO RNG
    /// consumed in that case).
    #[serde(default = "one_f32")]
    pub theta_rel: f32,
    #[serde(default = "one_f32")]
    pub u_plateau_rel: f32,
    #[serde(default = "one_f32")]
    pub tau_het_rel: f32,
    /// Dormancy state (structural machinery).
    pub dormant_since: Option<Tick>,
    pub retired: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synapse {
    pub id: SynapseId,
    pub pre: NeuronId,
    pub post: NeuronId,
    /// Weight in [w_min, w_max] (default [0, 1]).
    pub w: f32,
    /// Fixed per-synapse amplitude scale (current per spike = amplitude * w).
    pub amplitude: f32,
    /// Creation tick.
    pub created: Tick,
    /// Ticks this synapse has been continuously below the silence threshold.
    /// u64::MAX tombstones pruned synapses (ids are permanent indices).
    pub silent_ticks: u64,
    /// Plastic (true) or fixed (false; structural rewiring creates these).
    pub plastic: bool,
    /// V2 (M6): true = anti-Hebbian inhibitory synapse. Deliveries are
    /// negative (depress post); STDP never touches these (M6 owns them).
    #[serde(default)]
    pub inhibitory: bool,
}

/// Pure spike source — the last deterministic stage of the organism's "body"
/// (D8). No membrane, no dynamics, no plasticity on the channel itself: it
/// emits exactly the spikes the environment delivered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputChannel {
    pub id: InputChannelId,
    /// Group label (e.g. "A", "B", "C"); pure metadata for viz/analysis.
    pub group: String,
    /// The internal neuron this channel drives (1:1 afferent).
    pub target: NeuronId,
}

/// The per-tick delivered spike set on input channels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputFrame {
    pub tick: Tick,
    pub spikes: Vec<InputChannelId>,
}

/// Per-neuron dynamic state that a step produces for consumers.
#[derive(Debug, Clone)]
pub struct NeuronOutput {
    pub id: NeuronId,
    pub spiked: bool,
    pub v: f32,
    pub rate_hz: f32,
}

/// Events produced by one network step. Structural/plasticity events are
/// produced by their own modules; the network emits spikes and state.
#[derive(Debug, Clone)]
pub struct StepEvents {
    /// All neurons that spiked this tick, in id order.
    pub spikes: Vec<NeuronId>,
    /// Subset of spikes from output-class neurons.
    pub output_spikes: Vec<NeuronId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub lif: LIFParams,
    /// Initial sparse connectivity p between internal/input→internal neurons.
    pub connectivity: f32,
    /// Initial weight mean for wiring.
    pub w_init: f32,
    /// Weight bounds.
    pub w_min: f32,
    pub w_max: f32,
    /// Synaptic amplitude scale (fixed).
    pub amplitude: f32,
    /// EMA rate constant (Hz per spike-tick).
    pub rate_tau_ms: f32,
    /// U1 (E3): adaptation current decay tau (ms).
    pub adaptation_tau_ms: f32,
    /// U1 (E3): adaptation current injected per spike (gain 0 = E1).
    pub adaptation_gain: f32,
    /// U1-inhibition (E3b): inhibitory current each non-input spiker
    /// deposits onto every OTHER same-tick non-input spiker (0 = E3).
    pub inhibition_gain: f32,
    /// V2.1 (docs/v2_1-spec.md): slow depolarizing intrinsic state.
    /// slow_state_beta = per-spike increment; slow_state_tau_ms = decay.
    /// beta = 0 (default) => bit-identical V2.
    #[serde(default)]
    pub slow_state_beta: f32,
    #[serde(default = "default_slow_tau")]
    pub slow_state_tau_ms: f32,
    // ---- V2.2 (docs/v2_2-spec.md §1.3): all defaults = identity. ----
    /// Master switch: false => code path is V2.1 exactly.
    #[serde(default)]
    pub latch_enable: bool,
    /// SET threshold theta_i = u_reg * theta_rel_i; u_reg is the
    /// per-neuron regeneration equilibrium beta / (1 - exp(-1000/tau_s)).
    #[serde(default = "v22_theta_mean")]
    pub theta_rel_mean: f32,
    #[serde(default)]
    pub theta_rel_sd: f32,
    /// Plateau amplitude U_i = u_reg * u_plateau_rel_i (adds to u).
    #[serde(default = "v22_plateau_mean")]
    pub u_plateau_rel_mean: f32,
    #[serde(default)]
    pub u_plateau_rel_sd: f32,
    /// tau_s_i = tau_s * tau_het_rel_i (default 0 = off => rel 1.0).
    #[serde(default)]
    pub tau_het_rel_sd: f32,
    /// RESET release threshold phi_i = theta_i * phi_rel.
    #[serde(default = "v22_phi_rel")]
    pub phi_rel: f32,
    /// Y1 per-spike subtraction eta = beta * eta_rel (0 = off).
    #[serde(default)]
    pub eta_rel: f32,
    /// ANIMA v2 (docs/anima-v2-protocol.md): when Some, M1 dense-weak
    /// initialization + the V2Plasticity mechanisms (M2–M6) are active.
    #[serde(default)]
    pub v2: Option<V2Params>,
}

/// Frozen ANIMA v2 parameters (docs/anima-v2-protocol.md §2–§8).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2Params {
    // Ablation/control switches (protocol §13/14): false = mechanism ON.
    pub disable_m2: bool,
    pub disable_m3_m4: bool,
    pub disable_m5: bool,
    pub disable_m6: bool,
    // M1 — dense-weak initialization
    pub p_in: f32,
    pub w_in_lo: f32,
    pub w_in_hi: f32,
    pub p_rec: f32,
    pub w_rec_lo: f32,
    pub w_rec_hi: f32,
    // M2 — normalization
    pub t_e: f32,
    // M3 — candidates
    pub c_slots: usize,
    pub w_c_init: f32,
    pub delta_perm: f32,
    pub decay_c: f32,
    pub theta_permanent: f32,
    pub w_c_permanent: f32,
    pub theta_die: f32,
    pub p_cand_in: f32,
    pub p_cand_rec: f32,
    // M4 — pruning
    pub theta_prune: f32,
    pub prune_windows: u64,
    // M5 — budgets
    pub b_e: usize,
    pub b_i: usize,
    // M6 — anti-Hebbian inhibition
    pub p_inh: f32,
    pub w_inh_lo: f32,
    pub w_inh_hi: f32,
    pub a_inh: f32,
    pub decay_inh: f32,
    pub w_inh_max: f32,
    // Structural window
    pub window_ticks: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            lif: LIFParams::default(),
            connectivity: 0.15,
            w_init: 0.2,
            w_min: 0.0,
            w_max: 1.0,
            amplitude: 0.5,
            rate_tau_ms: 1000.0,
            // U1 defaults: adaptation OFF — gain 0 must reproduce E1 exactly.
            adaptation_tau_ms: 200.0,
            adaptation_gain: 0.0,
            // U1-inhibition default OFF: gain 0 must reproduce E3 exactly.
            inhibition_gain: 0.0,
            // V2.1 default OFF: beta 0 => u stays exactly 0.0 => V2 identity.
            slow_state_beta: 0.0,
            slow_state_tau_ms: default_slow_tau(),
            latch_enable: false,
            theta_rel_mean: v22_theta_mean(),
            theta_rel_sd: 0.0,
            u_plateau_rel_mean: v22_plateau_mean(),
            u_plateau_rel_sd: 0.0,
            tau_het_rel_sd: 0.0,
            phi_rel: v22_phi_rel(),
            eta_rel: 0.0,
            // V2 default OFF: None must reproduce E1–E4f exactly.
            v2: None,
        }
    }
}

pub struct Network {
    pub cfg: NetworkConfig,
    pub neurons: Vec<Neuron>,
    pub synapses: Vec<Synapse>,
    /// Adjacency: incoming synapse ids per neuron.
    pub incoming: Vec<Vec<SynapseId>>,
    /// Outgoing synapse ids per neuron.
    pub outgoing: Vec<Vec<SynapseId>>,
    pub channels: Vec<InputChannel>,
    pub tick: Tick,
    pub rng: Xoshiro256PlusPlus,
    pub seed: u64,
}

impl Network {
    /// Build the E1-style organism: input channels drive 1:1 input neurons;
    /// internal + output neurons are LIF; sparse random wiring, seeded.
    pub fn new(
        cfg: NetworkConfig,
        n_input_channels: usize,
        n_internal: usize,
        n_output: usize,
        seed: u64,
    ) -> Self {
        use rand::Rng;


        let rng = Xoshiro256PlusPlus::seed_from_u64(seed);
        let mut neurons: Vec<Neuron> = Vec::with_capacity(n_input_channels + n_internal + n_output);
        let mut channels: Vec<InputChannel> = Vec::with_capacity(n_input_channels);

        for ch in 0..n_input_channels {
            let id = NeuronId(neurons.len() as u32);
            neurons.push(Neuron {
                id,
                class: NeuronClass::Input,
                born: Tick(0),
                channel: Some(InputChannelId(ch as u32)),
                v: cfg.lif.v_rest,
                refractory_until: Tick(0),
                i_syn: 0.0,
                i_ext: 0.0,
                rate_hz: 0.0,
                i_adapt: 0.0,
                u_slow: 0.0,
                z_latch: 0,
                theta_rel: 1.0,
                u_plateau_rel: 1.0,
                tau_het_rel: 1.0,
                dormant_since: None,
                retired: false,
            });
            channels.push(InputChannel {
                id: InputChannelId(ch as u32),
                group: String::new(),
                target: id,
            });
        }
        for _ in 0..n_internal {
            let id = NeuronId(neurons.len() as u32);
            neurons.push(Neuron {
                id,
                class: NeuronClass::Internal,
                born: Tick(0),
                channel: None,
                v: cfg.lif.v_rest,
                refractory_until: Tick(0),
                i_syn: 0.0,
                i_ext: 0.0,
                rate_hz: 0.0,
                i_adapt: 0.0,
                u_slow: 0.0,
                z_latch: 0,
                theta_rel: 1.0,
                u_plateau_rel: 1.0,
                tau_het_rel: 1.0,
                dormant_since: None,
                retired: false,
            });
        }
        for _ in 0..n_output {
            let id = NeuronId(neurons.len() as u32);
            neurons.push(Neuron {
                id,
                class: NeuronClass::Output,
                born: Tick(0),
                channel: None,
                v: cfg.lif.v_rest,
                refractory_until: Tick(0),
                i_syn: 0.0,
                i_ext: 0.0,
                rate_hz: 0.0,
                i_adapt: 0.0,
                u_slow: 0.0,
                z_latch: 0,
                theta_rel: 1.0,
                u_plateau_rel: 1.0,
                tau_het_rel: 1.0,
                dormant_since: None,
                retired: false,
            });
        }

        let mut net = Self {
            cfg,
            neurons,
            synapses: Vec::new(),
            incoming: vec![Vec::new(); n_input_channels + n_internal + n_output],
            outgoing: vec![Vec::new(); n_input_channels + n_internal + n_output],
            channels,
            tick: Tick(0),
            rng,
            seed,
        };

        // Wiring (frozen order, docs/anima-v2-protocol.md §2):
        // when cfg.v2 is set (ANIMA v2): M1 dense-weak initialization —
        // for each non-input neuron in id order: (1) input afferents in
        // channel-id order (Bernoulli p_in, w ~ U(w_in_lo, w_in_hi));
        // (2) recurrent afferents in source-id order (Bernoulli p_rec,
        // w ~ U(w_rec_lo, w_rec_hi)); (3) M6 inhibitory afferents
        // (Bernoulli p_inh, w ~ U(w_inh_lo, w_inh_hi)); then (4)
        // candidate pools in neuron-id order.
        // Legacy: sparse wiring — every non-input neuron receives from all
        // source classes with prob p (E1–E4f behavior, bit-identical).
        let targets: Vec<NeuronId> = net
            .neurons
            .iter()
            .filter(|n| n.class != NeuronClass::Input)
            .map(|n| n.id)
            .collect();
        if let Some(v2) = net.cfg.v2.clone() {
            let all_ids: Vec<NeuronId> = net.neurons.iter().map(|n| n.id).collect();
            for &post in &targets {
                // (1) input afferents, channel-id order.
                for ch in 0..n_input_channels {
                    let src = net.channels[ch].target;
                    if net.rng.gen::<f32>() < v2.p_in {
                        let w = v2.w_in_lo + net.rng.gen::<f32>() * (v2.w_in_hi - v2.w_in_lo);
                        net.add_synapse(src, post, w, true, Tick(0));
                    }
                }
                // (2) recurrent afferents, source-id order (skip self).
                for src in all_ids.iter().copied() {
                    if src == post || net.neurons[src.idx()].class == NeuronClass::Input {
                        continue;
                    }
                    if net.rng.gen::<f32>() < v2.p_rec {
                        let w = v2.w_rec_lo + net.rng.gen::<f32>() * (v2.w_rec_hi - v2.w_rec_lo);
                        net.add_synapse(src, post, w, true, Tick(0));
                    }
                }
                // (3) M6 inhibitory afferents, source-id order (D8: never
                // from input neurons, never to input neurons). M5 budget
                // B_i is the binding constraint: at most B_i inhibitory
                // afferents per neuron (frozen values unchanged; see
                // protocol audit note A-1).
                let mut inh_count = 0usize;
                for src in all_ids.iter().copied() {
                    if src == post || net.neurons[src.idx()].class == NeuronClass::Input {
                        continue;
                    }
                    if net.rng.gen::<f32>() < v2.p_inh && inh_count < v2.b_i {
                        let w = v2.w_inh_lo + net.rng.gen::<f32>() * (v2.w_inh_hi - v2.w_inh_lo);
                        net.add_synapse_full(src, post, w, false, true, Tick(0));
                        inh_count += 1;
                    }
                }
            }
        } else {
            for &post in &targets {
                let sources: Vec<NeuronId> = net.neurons.iter().map(|n| n.id).collect();
                for src in sources {
                    if src == post {
                        continue;
                    }
                    if net.rng.gen::<f32>() < net.cfg.connectivity {
                        let w = (net.cfg.w_init * (0.5 + net.rng.gen::<f32>()))
                            .clamp(0.0, net.cfg.w_max);
                        net.add_synapse(src, post, w, true, Tick(0));
                    }
                }
            }
        }

        // V2.2 G2 (docs/v2_2-spec.md §2/§5): per-neuron LogNormal draws,
        // appended strictly AFTER all V2/V2.1 construction draws, in neuron-id
        // order, theta -> U -> T per neuron. SKIPPED ENTIRELY when both
        // heterogeneity sds are 0 (identity consumes zero RNG draws =>
        // bit-exact V2.1/V2 baselines).
        let het_needed = net.cfg.latch_enable
            && (net.cfg.theta_rel_sd > 0.0
                || net.cfg.u_plateau_rel_sd > 0.0
                || net.cfg.tau_het_rel_sd > 0.0);
        if het_needed {
            // median-preserving LogNormal: given multiplicative mean m and sd s,
            // mu = ln(m) - s^2/2, sigma = s (sd of log-space multiplier).
            let draw_lognormal = |rng: &mut Xoshiro256PlusPlus, m: f32, s: f32| -> f32 {
                use rand::Rng;
                let mu = (m.max(1e-9)).ln() - s * s / 2.0;
                // Box-Muller from two uniforms (deterministic order)
                let u1: f32 = rng.gen::<f32>().max(1e-9);
                let u2: f32 = rng.gen::<f32>();
                let r = (-2.0 * u1.ln()).sqrt();
                let z = r * (2.0 * core::f32::consts::PI * u2).cos();
                (mu + s * z).exp()
            };
            let ids: Vec<usize> = (0..net.neurons.len()).collect();
            for &i in &ids {
                let th = draw_lognormal(&mut net.rng, net.cfg.theta_rel_mean, net.cfg.theta_rel_sd);
                let up = draw_lognormal(&mut net.rng, net.cfg.u_plateau_rel_mean, net.cfg.u_plateau_rel_sd);
                let t = if net.cfg.tau_het_rel_sd > 0.0 {
                    draw_lognormal(&mut net.rng, 1.0, net.cfg.tau_het_rel_sd)
                } else {
                    1.0
                };
                let n = &mut net.neurons[i];
                n.theta_rel = th;
                n.u_plateau_rel = up;
                n.tau_het_rel = t;
            }
        }
        net
    }

    pub fn add_synapse(
        &mut self,
        pre: NeuronId,
        post: NeuronId,
        w: f32,
        plastic: bool,
        tick: Tick,
    ) -> SynapseId {
        self.add_synapse_full(pre, post, w, plastic, false, tick)
    }

    pub fn add_synapse_full(
        &mut self,
        pre: NeuronId,
        post: NeuronId,
        w: f32,
        plastic: bool,
        inhibitory: bool,
        tick: Tick,
    ) -> SynapseId {
        let id = SynapseId(self.synapses.len() as u32);
        self.synapses.push(Synapse {
            id,
            pre,
            post,
            w: w.clamp(self.cfg.w_min, self.cfg.w_max),
            amplitude: self.cfg.amplitude,
            created: tick,
            silent_ticks: 0,
            plastic,
            inhibitory,
        });
        self.outgoing[pre.idx()].push(id);
        self.incoming[post.idx()].push(id);
        id
    }

    /// Prune a synapse: ids are permanent indices into a grow-and-keep vec,
    /// so pruned synapses are tombstoned (`silent_ticks == u64::MAX`) and
    /// removed from both adjacency lists. Idempotent.
    pub fn prune_synapse(&mut self, id: SynapseId) {
        if let Some(s) = self.synapses.get_mut(id.idx()) {
            if s.silent_ticks != u64::MAX {
                s.silent_ticks = u64::MAX;
                if let Some(list) = self.outgoing.get_mut(s.pre.idx()) {
                    list.retain(|&x| x != id);
                }
                if let Some(list) = self.incoming.get_mut(s.post.idx()) {
                    list.retain(|&x| x != id);
                }
            }
        }
    }

    /// Mutable live-synapse iterator (tombstones skipped).
    pub fn live_synapses_mut(&mut self) -> impl Iterator<Item = &mut Synapse> {
        self.synapses.iter_mut().filter(|s| s.silent_ticks != u64::MAX)
    }

    pub fn synapse_alive(&self, id: SynapseId) -> bool {
        self.synapses.get(id.idx()).map(|s| s.silent_ticks != u64::MAX).unwrap_or(false)
    }

    pub fn live_synapses(&self) -> impl Iterator<Item = &Synapse> {
        self.synapses.iter().filter(|s| s.silent_ticks != u64::MAX)
    }

    pub fn live_synapse_count(&self) -> usize {
        self.synapses.iter().filter(|s| s.silent_ticks != u64::MAX).count()
    }

    pub fn neuron(&self, id: NeuronId) -> &Neuron {
        &self.neurons[id.idx()]
    }

    pub fn set_channel_groups(&mut self, group_of: impl Fn(usize) -> String) {
        for (i, ch) in self.channels.iter_mut().enumerate() {
            ch.group = group_of(i);
        }
    }

    /// Input-class neurons driven by the frame's spikes this tick. Pure spike
    /// source semantics (D8): the input neurons spike exactly when the frame
    /// says so — no state, no drift.
    fn deliver_input(&mut self, frame: &InputFrame) -> Vec<NeuronId> {
        let mut spiked = Vec::with_capacity(frame.spikes.len());
        for &ch in &frame.spikes {
            let target = self.channels[ch.idx()].target;
            spiked.push(target);
        }
        spiked
    }

    /// One deterministic tick. Order of operations:
    /// 1. deliver input spikes (input neurons spike iff channel fired)
    /// 2. all non-input neurons: integrate decayed currents, evaluate
    ///    threshold (refractory respected)
    /// 3. spiking neurons reset v, deposit current on postsynaptic targets
    /// 4. advance tick
    ///
    /// Synaptic transmission delay: 1 tick (spikes affect postsynaptic
    /// membrane on the next step via i_syn deposit evaluated after
    /// integration — we deposit into i_syn of targets, which integrates next
    /// tick because decay+integration reads i_syn at step start).
    pub fn step(&mut self, frame: &InputFrame) -> StepEvents {
        let dt = 1.0f32;
        let p = self.cfg.lif;
        let decay_syn = exp_approx(-dt / p.tau_syn);
        let mut spikes: Vec<NeuronId> = self.deliver_input(frame);

        // Integration pass over non-input, non-retired neurons.
        let n = self.neurons.len();
        let decay_adapt = exp_approx(-dt / self.cfg.adaptation_tau_ms);
        // V2.1: slow-state decay (exact exponential, mirror of i_adapt).
        // beta = 0 => u stays exactly 0.0 and dv adds +0.0 (V2 identity).
        let decay_slow = exp_approx(-dt / self.cfg.slow_state_tau_ms);
        for i in 0..n {
            if self.neurons[i].class == NeuronClass::Input || self.neurons[i].retired {
                continue;
            }
            let neur = &mut self.neurons[i];
            // decay i_syn first (exponential kernel, tau_syn)
            neur.i_syn *= decay_syn;
            // U1: adaptation current decays on the same exponential form.
            neur.i_adapt *= decay_adapt;
            // V2.1: decay-then-read (same convention as i_adapt/i_syn).
            // V2.2 G2: per-neuron tau (identity 1.0 when tau_het off).
            let decay_slow_i = if self.cfg.latch_enable && neur.tau_het_rel != 1.0 {
                exp_approx(-dt / (self.cfg.slow_state_tau_ms * neur.tau_het_rel))
            } else {
                decay_slow
            };
            neur.u_slow *= decay_slow_i;
            // V2.2 G1: latch gate on the DECAYED u (spec §1.2 step 2).
            let u_eff = if self.cfg.latch_enable {
                // u_reg = per-neuron regeneration equilibrium (spec §1.3):
                // beta / (1 - exp(-1000/tau_s_i)) — the equilibrium of the
                // integrate-and-decay loop at 1000 spikes/s.
                let tau_i = self.cfg.slow_state_tau_ms * neur.tau_het_rel;
                // A-1 (approved 2026-09-20, pre-execution): u_reg is the
                // firing-rate equilibrium beta * f_ref * tau_s / 1000 with
                // f_ref = 100 Hz (E3 pre-registered rate-calibration band
                // [100,200] Hz midpoint; docs/anima-e3-protocol.md). Units:
                // [u] = [u/spike] * [spikes/s] * [s]. Supersedes the frozen
                // spec's 1 Hz-per-tick formula (scale defect, see spec doc).
                const F_REF_HZ: f32 = 100.0;
                let u_reg = self.cfg.slow_state_beta * F_REF_HZ * tau_i / 1000.0;
                let theta = u_reg * neur.theta_rel;
                if neur.z_latch == 0 && neur.u_slow >= theta {
                    neur.z_latch = 1;
                } else if neur.z_latch == 1 && neur.u_slow < theta * self.cfg.phi_rel {
                    neur.z_latch = 0;
                }
                // Plateau ADDS to u (spec §1): u_eff = u + z * U_i.
                neur.u_slow + (neur.z_latch as f32) * u_reg * neur.u_plateau_rel
            } else {
                neur.u_slow
            };
            let dv = (-(neur.v - p.v_rest) + neur.i_syn + neur.i_ext - neur.i_adapt + u_eff) * dt / p.tau_m;
            neur.v += dv;
            let spiked = self.tick.0 >= neur.refractory_until.0 && neur.v >= p.v_th;
            if spiked {
                neur.v = p.v_reset;
                neur.refractory_until = Tick(self.tick.0 + p.refractory);
                // U1: hyperpolarizing kick per spike.
                neur.i_adapt += self.cfg.adaptation_gain;
                // V2.1: depolarizing slow kick per spike (spec §1.2-1.3).
                neur.u_slow += self.cfg.slow_state_beta;
                // V2.2 Y1: local per-spike subtraction eta = beta*eta_rel,
                // floored at 0 (spec §3; 0 = off when eta_rel = 0).
                if self.cfg.latch_enable && self.cfg.eta_rel > 0.0 {
                    let eta = self.cfg.slow_state_beta * self.cfg.eta_rel;
                    neur.u_slow = (neur.u_slow - eta).max(0.0);
                }
                spikes.push(neur.id());
            }
        }
        // low-passed with tau = rate_tau_ms.
        let alpha = dt / self.cfg.rate_tau_ms;
        for neur in &mut self.neurons {
            let target = if spikes.contains(&neur.id) { 1000.0 } else { 0.0 };
            neur.rate_hz += (target - neur.rate_hz) * alpha;
        }

        // Deposit current onto postsynaptic targets of spikers.
        let spikers: Vec<NeuronId> = spikes.clone();
        for &src in &spikers {
            for &sid in self.outgoing[src.idx()].clone().iter() {
                let s = &self.synapses[sid.idx()];
                if s.silent_ticks == u64::MAX {
                    continue;
                }
                let post = s.post;
                // V2 M6: inhibitory synapses deliver negative current.
                let current = if s.inhibitory { -(s.amplitude * s.w) } else { s.amplitude * s.w };
                self.neurons[post.idx()].i_syn += current;
            }
        }

        // U1-inhibition (E3b): every non-input spiker deposits an extra
        // inhibitory current onto every OTHER same-tick non-input spiker.
        // Competition between co-active neurons; 1-tick delay, same
        // tau_syn kernel. Gain 0 is a no-op (E3 identity).
        if self.cfg.inhibition_gain != 0.0 {
            let non_input: Vec<NeuronId> = spikers
                .iter()
                .copied()
                .filter(|&id| self.neurons[id.idx()].class != NeuronClass::Input)
                .collect();
            let g = self.cfg.inhibition_gain;
            for &a in &non_input {
                for &b in &non_input {
                    if a != b {
                        self.neurons[b.idx()].i_syn -= g;
                    }
                }
            }
        }

        self.tick = Tick(self.tick.0 + 1);

        let output_spikes: Vec<NeuronId> = spikes
            .iter()
            .copied()
            .filter(|&id| self.neurons[id.idx()].class == NeuronClass::Output)
            .collect();

        StepEvents { spikes, output_spikes }
    }
}

impl Neuron {
    pub fn id(&self) -> NeuronId {
        self.id
    }
}

/// Fast exp approximation adequate for the decay factors used here
/// (|x| small, x negative). Relative error < 1e-4 — deterministic and
/// consistent across platforms (no libm variance).
#[inline]
pub fn exp_approx(x: f32) -> f32 {
    // 6th-order Taylor around 0 with argument squaring:
    // e^x = (1 + x/2 + x^2/8 + ...)^2 style — we use the standard
    // polynomial on x/2^k then square k times.
    let mut y = 1.0f32 + x / 64.0;
    y = y * y; // x/32
    y = y * y; // x/16
    y = y * y; // x/8
    y = y * y; // x/4
    y = y * y; // x/2
    y = y * y; // x
    y
}

#[cfg(test)]
mod tests {
    use super::*;

    fn net1() -> Network {
        Network::new(NetworkConfig::default(), 4, 6, 2, 42)
    }

    #[test]
    fn lif_spike_at_threshold() {
        // Drive a neuron with constant external current past threshold.
        let mut net = net1();
        let target = net.neurons.iter().find(|n| n.class == NeuronClass::Internal).unwrap().id;
        net.neurons[target.idx()].i_ext = 2.0; // v_ss = 2.0 > v_th ⇒ periodic spiking
        let mut spiked_at = None;
        for t in 0..200 {
            let ev = net.step(&InputFrame { tick: Tick(t), spikes: vec![] });
            if ev.spikes.contains(&target) {
                spiked_at = Some(t);
                break;
            }
        }
        assert!(spiked_at.is_some(), "constant current must eventually spike");
        // Refractory: no spike in the immediately following tick even at threshold-crossing drive.
        net.neurons[target.idx()].v = 5.0 * net.cfg.lif.v_th; // force above threshold
        let before = net.tick.0;
        let _ = net.step(&InputFrame { tick: Tick(before), spikes: vec![] }); // consumes forced spike
        assert!(net.neuron(target).refractory_until.0 > net.tick.0.saturating_sub(1));
    }

    #[test]
    fn passive_decay_to_rest() {
        let mut net = net1();
        let target = net.neurons.iter().find(|n| n.class == NeuronClass::Internal).unwrap().id;
        net.neurons[target.idx()].v = 0.9;
        for t in 0..200 {
            let _ = net.step(&InputFrame { tick: Tick(t), spikes: vec![] });
        }
        assert!((net.neuron(target).v - net.cfg.lif.v_rest).abs() < 1e-2);
    }

    #[test]
    fn input_channel_emits_exactly_delivered_spikes() {
        // D8: pure spike source — same frame repeated, no drift, no state.
        let mut net = net1();
        let ch0 = InputChannelId(0);
        for _ in 0..50 {
            let ev = net.step(&InputFrame { tick: net.tick, spikes: vec![ch0] });
            assert!(ev.spikes.contains(&net.channels[0].target));
            assert_eq!(ev.spikes.iter().filter(|&&n| n == net.channels[0].target).count(), 1);
        }
        // Silence: no input-neuron spikes.
        for _ in 0..50 {
            let ev = net.step(&InputFrame { tick: net.tick, spikes: vec![] });
            assert!(!ev.spikes.contains(&net.channels[0].target));
        }
    }

    #[test]
    fn determinism_same_seed_same_events() {
        // Compare spikes AND initial wiring: with quiescent internal neurons
        // the spike stream alone is frame-driven, so wiring is where the
        // seed must show. Also drive hard enough to recruit internal spikes.
        let mk = |seed| {
            let mut net = Network::new(
                NetworkConfig { amplitude: 8.0, ..NetworkConfig::default() },
                8,
                12,
                4,
                seed,
            );
            let mut log = String::new();
            for s in net.live_synapses() {
                log.push_str(&format!("s{}:{}->{}:{:.4};", s.id.0, s.pre.0, s.post.0, s.w));
            }
            for t in 0..500u64 {
                let frame = InputFrame {
                    tick: Tick(t),
                    spikes: if t % 7 == 0 {
                        vec![InputChannelId(0), InputChannelId(3)]
                    } else {
                        vec![]
                    },
                };
                let ev = net.step(&frame);
                log.push_str(&format!("{:?};", ev.spikes));
            }
            log
        };
        assert_eq!(mk(7), mk(7), "same seed must reproduce exactly");
        assert_ne!(mk(7), mk(8), "different seed must change the stream");
    }

    #[test]
    fn prune_removes_from_adjacency() {
        let mut net = net1();
        let sid = net.synapses[0].id;
        assert!(net.synapse_alive(sid));
        net.prune_synapse(sid);
        assert!(!net.synapse_alive(sid));
        assert!(!net.outgoing[net.synapses[sid.idx()].pre.idx()].contains(&sid));
        assert!(!net.incoming[net.synapses[sid.idx()].post.idx()].contains(&sid));
    }

    /// U1: adaptation current decays exponentially with adaptation_tau_ms.
    #[test]
    fn adaptation_current_decays_with_tau() {
        let mut net = Network::new(
            NetworkConfig { adaptation_tau_ms: 200.0, ..NetworkConfig::default() },
            2,
            1,
            0,
            1,
        );
        let id = net.neurons.iter().find(|n| n.class == NeuronClass::Internal).unwrap().id;
        net.neurons[id.idx()].i_adapt = 1.0;
        // No drive: pure decay.
        for _ in 0..200 {
            let _ = net.step(&InputFrame { tick: Tick(0), spikes: vec![] });
        }
        let after = net.neuron(id).i_adapt;
        let expected = 1.0f32 * exp_approx(-200.0 / 200.0);
        assert!(
            (after - expected).abs() < 0.02,
            "i_adapt {after} must track exp(-t/tau) = {expected}"
        );
    }

    /// U1: each spike injects adaptation_gain of hyperpolarizing current,
    /// which raises the effective threshold crossable drive.
    #[test]
    fn adaptation_injected_per_spike_raises_spike_threshold() {
        // Current strong enough for periodic spiking without adaptation.
        let mk = |gain| {
            let mut net = Network::new(
                NetworkConfig {
                    adaptation_tau_ms: 200.0,
                    adaptation_gain: gain,
                    ..NetworkConfig::default()
                },
                2,
                1,
                0,
                1,
            );
            let id = net.neurons.iter().find(|n| n.class == NeuronClass::Internal).unwrap().id;
            net.neurons[id.idx()].i_ext = 2.0;
            let mut spikes = 0;
            for t in 0..500u64 {
                let ev = net.step(&InputFrame { tick: Tick(t), spikes: vec![] });
                if ev.spikes.contains(&id) {
                    spikes += 1;
                }
            }
            let adapt = net.neuron(id).i_adapt;
            (spikes, adapt)
        };
        let (spikes0, adapt0) = mk(0.0);
        let (spikes_pos, adapt_pos) = mk(0.5);
        assert!(spikes0 > 5, "control must spike often (got {spikes0})");
        assert!(spikes_pos < spikes0, "adaptation must reduce spike count");
        assert!(adapt_pos > adapt0, "spiking must accumulate i_adapt");
    }

    /// U1 guard: gain 0 leaves the E1 trajectory bit-identical.
    #[test]
    fn adaptation_gain_zero_reproduces_bare_lif() {
        let drive = |mut net: Network| {
            let id = net.neurons.iter().find(|n| n.class == NeuronClass::Internal).unwrap().id;
            net.neurons[id.idx()].i_ext = 1.5;
            let mut log = String::new();
            for t in 0..300u64 {
                let ev = net.step(&InputFrame { tick: Tick(t), spikes: vec![] });
                log.push_str(&format!("{:?};", ev.spikes));
            }
            log
        };
        let plain = drive(Network::new(NetworkConfig::default(), 4, 6, 2, 42));
        let with_adapt = drive(Network::new(
            NetworkConfig {
                adaptation_tau_ms: 200.0,
                adaptation_gain: 0.0,
                ..NetworkConfig::default()
            },
            4,
            6,
            2,
            42,
        ));
        assert_eq!(plain, with_adapt, "gain 0 must not change dynamics");
    }

    /// U1-inhibition: inhibitory current must be negative-going (deposits
    /// reduce i_syn) and must reduce co-activation. Two internally driven
    /// neurons spike together when inhibition is off; with gain on, the
    /// mutual suppression desynchronizes them.
    #[test]
    fn inhibition_suppresses_coincident_spiking() {
        let count_coincident = |gain: f32| {
            let mut net = Network::new(
                NetworkConfig {
                    inhibition_gain: gain,
                    ..NetworkConfig::default()
                },
                2,
                2,
                0,
                9,
            );
            let ids: Vec<NeuronId> = net
                .neurons
                .iter()
                .filter(|n| n.class == NeuronClass::Internal)
                .map(|n| n.id)
                .collect();
            for id in &ids {
                net.neurons[id.idx()].i_ext = 2.0;
            }
            let mut coincident = 0;
            let mut total = [0usize; 2];
            for t in 0..400u64 {
                let ev = net.step(&InputFrame { tick: Tick(t), spikes: vec![] });
                let hit: Vec<bool> = ids.iter().map(|id| ev.spikes.contains(id)).collect();
                for k in 0..2 {
                    total[k] += hit[k] as usize;
                }
                if hit[0] && hit[1] {
                    coincident += 1;
                }
            }
            (coincident, total)
        };
        let (coinc0, tot0) = count_coincident(0.0);
        let (coinc_pos, tot_pos) = count_coincident(2.0);
        assert!(tot0[0] > 20 && tot0[1] > 20, "control must spike often");
        assert!(coinc_pos < coinc0, "inhibition must reduce coincident spikes");
        assert!(
            tot_pos[0] + tot_pos[1] > 0,
            "inhibition must not silence the neurons entirely"
        );
    }

    /// V2.1 Stage A (docs/v2_1-spec.md §14): beta = 0 => u stays
    /// exactly 0.0 and trajectories are byte-identical to V2.
    #[test]
    fn v21_identity_gate_beta_zero() {
        use crate::network::{InputChannelId, InputFrame, NetworkConfig, Tick};
        let drive = |mut net: crate::network::Network, n: u64| -> (Vec<u32>, Vec<f32>) {
            use rand::Rng;
            let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(7);
            let mut spikes = Vec::new();
            for t in 0..n {
                let mut chs = Vec::new();
                for c in 0..24u32 {
                    if rng.gen::<f32>() < 0.02 {
                        chs.push(InputChannelId(c));
                    }
                }
                let ev = net.step(&InputFrame { tick: Tick(t), spikes: chs });
                for sp in &ev.spikes { spikes.push(sp.0); }
            }
            let us: Vec<f32> = net.neurons.iter().map(|n| n.u_slow).collect();
            (spikes, us)
        };
        let mk = |beta: f32| crate::network::Network::new(
            NetworkConfig {
                adaptation_gain: 0.05,
                slow_state_beta: beta,
                slow_state_tau_ms: 2500.0,
                ..NetworkConfig::default()
            },
            24, 40, 12, 42,
        );
        let (s0, u0) = drive(mk(0.0), 100_000);
        let (s2, u2) = drive(mk(0.0), 100_000);
        assert_eq!(s0, s2, "same-beta determinism");
        assert!(u0.iter().all(|&u| u == 0.0), "beta=0 => u exactly 0.0");
        // V2 network = config with the fields defaulted (absent semantics)
        let mut v2cfg = NetworkConfig::default();
        v2cfg.adaptation_gain = 0.05;
        let (sv, uv) = drive(crate::network::Network::new(v2cfg, 24, 40, 12, 42), 100_000);
        assert_eq!(s0, sv, "beta=0 == V2 byte-identical spike sequence");
        assert!(uv.iter().all(|&u| u == 0.0));
        // beta > 0 => u becomes nonzero (dense-drive sanity arm)
        {
            use rand::Rng;
            let mut net = mk(0.05);
            let mut rng = rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(9);
            for t in 0..5000u64 {
                let mut chs = Vec::new();
                for c in 0..24u32 { if rng.gen::<f32>() < 0.5 { chs.push(InputChannelId(c)); } }
                net.step(&InputFrame { tick: Tick(t), spikes: chs });
            }
            let max_u = net.neurons.iter().map(|n| n.u_slow).fold(0.0f32, f32::max);
            assert!(max_u > 0.0, "beta>0 dense drive => u accumulates (max {max_u})");
        }
    }

    /// V2.1 Stage B sanity: u decays with tau_s after activity stops.
    #[test]
    fn v21_u_decays_with_tau() {
        use crate::network::{InputChannelId, InputFrame, NetworkConfig, Tick};
        let mut net = crate::network::Network::new(
            NetworkConfig {
                adaptation_gain: 0.05,
                slow_state_beta: 0.05,
                slow_state_tau_ms: 2500.0,
                ..NetworkConfig::default()
            },
            24, 40, 12, 42,
        );
        // drive 500 ms
        for t in 0..500u64 {
            net.step(&InputFrame { tick: Tick(t), spikes: vec![InputChannelId(0), InputChannelId(1)] });
        }
        let max_u = net.neurons.iter().map(|n| n.u_slow).fold(0.0f32, f32::max);
        assert!(max_u > 0.0, "u accumulated during drive");
        // silence 2500 ms (one tau): sample u at snapshot cadence
        let u_at = |net: &crate::network::Network| net.neurons.iter().map(|n| n.u_slow).sum::<f32>();
        let u_t0 = u_at(&net);
        for t in 500..3000u64 {
            net.step(&InputFrame { tick: Tick(t), spikes: vec![] });
        }
        let u_tau = u_at(&net);
        // total u after ~1 tau should be < ~45% of start (exp(-1)~0.37 plus
        // any endogenous replenishment; loose bound, no tuning)
        assert!(u_tau < 0.45 * u_t0, "u decays: {u_tau} vs {u_t0}");
    }

    /// U1-inhibition guard: gain 0 must reproduce E3 trajectories
    /// bit-identically (both adaptation and inhibition inactive path).
    #[test]
    fn inhibition_gain_zero_reproduces_e3() {
        let drive = |mut net: Network| {
            let id = net.neurons.iter().find(|n| n.class == NeuronClass::Internal).unwrap().id;
            net.neurons[id.idx()].i_ext = 2.0;
            let mut log = String::new();
            for t in 0..300u64 {
                let ev = net.step(&InputFrame { tick: Tick(t), spikes: vec![] });
                log.push_str(&format!("{:?};", ev.spikes));
            }
            log
        };
        let e3 = drive(Network::new(
            NetworkConfig {
                adaptation_tau_ms: 200.0,
                adaptation_gain: 0.05,
                inhibition_gain: 0.0,
                ..NetworkConfig::default()
            },
            4,
            6,
            2,
            42,
        ));
        let e3b_inhibit_off = drive(Network::new(
            NetworkConfig {
                adaptation_tau_ms: 200.0,
                adaptation_gain: 0.05,
                inhibition_gain: 0.0,
                ..NetworkConfig::default()
            },
            4,
            6,
            2,
            42,
        ));
        assert_eq!(e3, e3b_inhibit_off, "inhibition gain 0 must not change dynamics");
    }
}
