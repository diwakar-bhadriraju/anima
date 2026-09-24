//! D-59 pluggable sensory-encoder interface (foundation ONLY - no encoder
//! implementation in this plan).
//!
//! Contract: every sensor (current pattern source, future retina/cochlea)
//! emits deterministic spike trains over one BEAT_MS window, EXACTLY the
//! shape `io::symbol_trains_mode` produces: `Vec<(tick_offset, InputChannelId)>`
//! sorted by offset. Any future encoder is a drop-in at the survival-loop
//! train site.
//!
//! CHARTER CONSTRAINTS (docs/ANIMA_OVERVIEW.md): encoders are deterministic
//! filterbanks -> Poisson spike trains; NO pretrained models, CNNs, CLIP,
//! VLMs or embeddings. NAMING: do NOT call future encoders "E9"/"E10" -
//! those names are taken by the curriculum/temporal-order experiments
//! (docs/anima-e9-protocol.md / e10-protocol.md, config::CFG_E9...). Use
//! `retina` / `cochlea` for the vision/audio sensors.
//!
//! A future retina/cochlea registration implements `SensoryEncoder`
//! (channel_count = its filterbank channel count; frames = the Poisson
//! trains the network's InputChannels receive).
use anima_core::network::InputChannelId;

use crate::io;

/// A sensory encoder: deterministic spike-train source for the organism's
/// input channels over one beat window.
pub trait SensoryEncoder {
    /// Number of input channels this sensor drives (the network's input
    /// population for this modality).
    fn channel_count(&self) -> usize;
    /// The spike trains for one BEAT_MS presentation of the sensor's
    /// current content (seed => fully deterministic trains), same shape as
    /// `io::symbol_trains_mode` output: sorted `(tick_offset, channel)`.
    fn frames(&self, seed: u64) -> Vec<(u64, InputChannelId)>;
}

/// The existing pattern source as an encoder: one symbol of the current
/// alphabet -> its deterministic Poisson trains. The passthrough impl that
/// keeps the current survive-loop input path valid; retina/cochlea encoders
/// implement the trait the same way in their own registrations.
pub struct PatternEncoder {
    pub sym: String,
    pub mode: String,
}

impl SensoryEncoder for PatternEncoder {
    fn channel_count(&self) -> usize {
        io::alphabet(&self.mode)
            .iter()
            .find(|(s, _)| *s == self.sym.as_str())
            .map(|(_, c)| c.len())
            .unwrap_or(0)
    }
    fn frames(&self, seed: u64) -> Vec<(u64, InputChannelId)> {
        io::symbol_trains_mode(&self.sym, &self.mode, seed)
    }
}