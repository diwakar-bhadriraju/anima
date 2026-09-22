//! anima-exp shared library: modules usable by both the anima-run binary
//! and auxiliary bins (evolve). Exposes config, io (codebook), survival
//! (closed-loop), env, harness.
pub mod config;
pub mod io;
pub mod survival;
pub mod env;
pub mod harness;
pub mod e19_world;
