//! D-70.1 deterministic 3D organism world (docs/phase3/d70-1-world-scaffold-protocol.md).
//!
//! Charter constraints: deterministic (same seed -> bit-identical world
//! state and retina frames), no pretrained models/GPUs/external engines,
//! I/O-boundary design only. Naming: the vision encoder is `retina`,
//! NOT E9/E10 (curriculum-experiment collision).
//!
//! Anatomy: `World` (arena + body + static primitives, deterministic
//! fixed-step physics), `Camera` (per-cell ray rasterizer -> luminance +
//! local contrast), `RetinaEncoder` implements
//! `anima_exp::encoders::SensoryEncoder` (48 vision + 8 proprioception
//! channels, Poisson trains over BEAT_MS — conventions identical to
//! io::symbol_trains_mode: FNV-1a hash, splitmix64 derive, per-cell
//! Xoshiro256PlusPlus, sorted by tick).
pub mod camera;
pub mod retina;
pub mod world;

pub use camera::{Camera, RETINA_COLS, RETINA_ROWS};
pub use retina::{PROPRIO_CHANNELS, RetinaEncoder, VISION_CHANNELS};
pub use world::{Body, MotorDrive, Primitive, Shape, World};

/// Total input channels of the retina encoder: 48 vision + 8 proprio.
pub const RETINA_TOTAL: usize = VISION_CHANNELS + PROPRIO_CHANNELS;