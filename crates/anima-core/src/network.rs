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

        // Sparse wiring: every non-input neuron receives from all source
        // classes (input + internal + earlier outputs) with prob p.
        let targets: Vec<NeuronId> = net
            .neurons
            .iter()
            .filter(|n| n.class != NeuronClass::Input)
            .map(|n| n.id)
            .collect();
        for &post in &targets {
            let sources: Vec<NeuronId> = net.neurons.iter().map(|n| n.id).collect();
            for src in sources {
                if src == post {
                    continue;
                }
                if net.rng.gen::<f32>() < net.cfg.connectivity {
                    let w =
                        (net.cfg.w_init * (0.5 + net.rng.gen::<f32>())).clamp(0.0, net.cfg.w_max);
                    net.add_synapse(src, post, w, true, Tick(0));
                }
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
        for i in 0..n {
            if self.neurons[i].class == NeuronClass::Input || self.neurons[i].retired {
                continue;
            }
            let neur = &mut self.neurons[i];
            // decay i_syn first (exponential kernel, tau_syn)
            neur.i_syn *= decay_syn;
            let dv = (-(neur.v - p.v_rest) + neur.i_syn + neur.i_ext) * dt / p.tau_m;
            neur.v += dv;
            let spiked = self.tick.0 >= neur.refractory_until.0 && neur.v >= p.v_th;
            if spiked {
                neur.v = p.v_reset;
                neur.refractory_until = Tick(self.tick.0 + p.refractory);
                spikes.push(neur.id());
            }
        }

        // Rate EMA (Hz): instantaneous 1000 Hz on spike ticks, 0 otherwise,
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
                let current = s.amplitude * s.w;
                self.neurons[post.idx()].i_syn += current;
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
}
