//! `anima-core`: the organism. LIF network, deterministic stepping,
//! plasticity engine, structural machinery, resource economy.

pub mod network;
pub mod plasticity;
pub mod rate_balance;
pub mod resources;
pub mod structural;
pub mod structural_v2;
#[cfg(test)]
mod structural_v2_tests;

pub use network::{
    InputChannel, InputChannelId, InputFrame, Neuron, NeuronClass, NeuronId, Network,
    NetworkConfig, SimMs, Synapse, SynapseId, Tick, LIFParams,
};
pub use plasticity::{PairwiseStdp, PlasticityRule, StdpParams, Traces};
pub use resources::{ResourceConfig, ResourceMonitor};
pub use structural::{BirthTrigger, NoBirth, Reason, Signals, StructuralMonitor};
