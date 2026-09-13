//! `anima-telemetry`: event schema (D6), recorder, analyzer, report.

pub mod analyzer;
pub mod chunks;
pub mod events;
pub mod recorder;
pub mod report;

pub use analyzer::{analyze, Metrics};
pub use chunks::{ChunkMeta, ChunkedTelemetry, Row, TelemetryReader, CHUNK_MS, CHUNK_ROWS};
pub use events::{EventBuilder, EventKind, Envelope, Payload, ReasonPayload};
pub use recorder::{read_snapshots, read_telemetry, Recorder, NetworkStateSnapshot};
pub use report::{generate_report, verdicts};

