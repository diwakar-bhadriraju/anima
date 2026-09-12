//! `anima-viz`: WebSocket server + embedded web UI for live and replay views.

mod server;

pub use server::{
    replay, start, CtrlOp, ServerHandle, UiNeuron, UiSynapse, VizState,
};
