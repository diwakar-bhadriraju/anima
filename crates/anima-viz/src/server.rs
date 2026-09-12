//! WS server + embedded web UI (step 8).
//!
//! axum (ws) + tokio; the sim runs on the caller's thread. Cross-thread:
//! `tokio::sync::broadcast` (events) + `std::sync::mpsc` (control).
//! Replay streams recorded JSONL through the identical WS protocol.
use std::sync::Arc;
use tokio::sync::broadcast;
use axum::extract::ws::{Message, WebSocket};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use anima_telemetry::events::{Envelope, EventKind};
/// One network view node.
#[derive(Debug, Clone, Serialize)]
pub struct UiNeuron {
    pub id: u32,
    /// "input" | "internal" | "output"
    pub cls: String,
    pub group: String,
    pub rate: f32,
    /// "normal" | "dormant" | "retired"
    pub state: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UiSynapse {
    pub id: u32,
    pub src: u32,
    pub dst: u32,
    pub w: f32,
}

/// Full UI state; replaced atomically every few ticks, serialized on join.
#[derive(Debug, Clone, Serialize, Default)]
pub struct VizState {
    pub tick: u64,
    pub paused: bool,
    pub speed: f32,
    pub neurons: Vec<UiNeuron>,
    pub synapses: Vec<UiSynapse>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "op")]
pub enum CtrlOp {
    #[serde(rename = "pause")]
    Pause,
    #[serde(rename = "resume")]
    Resume,
    #[serde(rename = "step")]
    Step { n: u64 },
    #[serde(rename = "speed")]
    Speed { factor: f32 },
    #[serde(rename = "seek")]
    Seek { tick: u64 },
}

/// Handle the driver (run or replay) uses to talk to the server.
pub struct ServerHandle {
    pub port: u16,
    pub ctrl_rx: std::sync::mpsc::Receiver<CtrlOp>,
    pub state: Arc<Mutex<VizState>>,
    pub tx: broadcast::Sender<String>,
    pub mode: &'static str,
    pub exp_id: String,
    pub duration_ms: u64,
}

impl ServerHandle {
    pub fn send_json(&self, s: String) {
        let _ = self.tx.send(s);
    }

    pub fn broadcast_event(&self, env: &Envelope) {
        // Wire decimation: per-synapse weight deltas and instrumentation
        // samples are thousands/s on the sim thread; a WS client cannot
        // consume them and would lag the broadcast channel permanently.
        // Weight state reaches clients via the 10-tick UI snapshot instead
        // (VizState); telemetry keeps full fidelity.
        match env.kind {
            EventKind::SynapseStrengthened
            | EventKind::SynapseWeakened
            | EventKind::PredictionError
            | EventKind::TickStats => return,
            _ => {}
        }
        let mut v = serde_json::to_value(env).expect("envelope serializes");
        // Message-type tag is separate from the envelope's sim-time `t`
        // (u64 ms) so clients can use `t` for clocks and replay scrubbing.
        v["msg"] = serde_json::Value::String("ev".into());
        let _ = self.tx.send(v.to_string());
    }

    pub fn update_state(&self, tick: u64, paused: bool, speed: f32, neurons: Vec<UiNeuron>, synapses: Vec<UiSynapse>) {
        let mut st = self.state.lock();
        *st = VizState { tick, paused, speed, neurons, synapses };
    }
}


const INDEX_HTML: &str = include_str!("../web/index.html");

/// Serve forever on the current tokio runtime. Called inside a spawned
/// runtime thread by [`start`].
async fn serve(port: u16, shared: Arc<Shared>) {
    use axum::extract::ws::WebSocketUpgrade;
    use axum::extract::State;
    use axum::response::Html;
    use axum::routing::get;
    use axum::Router;

    async fn index() -> Html<&'static str> {
        Html(INDEX_HTML)
    }

    async fn ws_handler(
        ws: WebSocketUpgrade,
        State(shared): State<Arc<Shared>>,
    ) -> impl axum::response::IntoResponse {
        let owned = (*shared).clone();
        ws.on_upgrade(move |socket| async move { client_loop(socket, owned).await })
    }

    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(ws_handler))
        .with_state(shared.clone());

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("viz bind {addr}: {e}"));
    axum::serve(listener, app).await.unwrap();
}

#[derive(Clone)]
struct Shared {
    tx: broadcast::Sender<String>,
    ctrl_tx: std::sync::mpsc::Sender<CtrlOp>,
    state: Arc<Mutex<VizState>>,
    hello: Arc<String>,
}

async fn client_loop(socket: WebSocket, shared: Shared) {
    use futures_util::{SinkExt, StreamExt};
    let (mut sender, mut receiver) = socket.split();
    // On join: hello + snapshot, then a single pump loop that both forwards
    // broadcast events and consumes control messages. Splitting into two
    // tasks raced on socket close and silently killed live connections.
    if sender
        .send(Message::Text((*shared.hello).clone()))
        .await
        .is_err()
    {
        return;
    }
    let snap = {
        let st = shared.state.lock().clone();
        let mut v = serde_json::to_value(&st).unwrap();
        v["t"] = serde_json::Value::String("snapshot".into());
        v.to_string()
    };
    if sender.send(Message::Text(snap)).await.is_err() {
        return;
    }

    // Batched pump: every 50 ms drain ALL pending broadcast frames into one
    // combined send, and poll control messages. A blocked TCP sink (hidden
    // background tab) can stall a per-frame pump forever; the interval
    // guarantees the loop keeps waking, drops stale frames instead of
    // queueing unbounded, and recovers the moment the client reads.
    let mut rx = shared.tx.subscribe();
    let ctrl_tx = shared.ctrl_tx.clone();
    let mut tick = tokio::time::interval(std::time::Duration::from_millis(50));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = tick.tick() => {
                let mut batch = String::new();
                let mut n = 0;
                while let Ok(text) = rx.try_recv() {
                    batch.push_str(&text);
                    batch.push('\n');
                    n += 1;
                    if n >= 256 { break; } // bound per-flush work
                }
                if !batch.is_empty() {
                    let frame = serde_json::json!({
                        "msg": "batch",
                        "lines": batch,
                    });
                    if sender.send(Message::Text(frame.to_string())).await.is_err() {
                        break;
                    }
                }
            }
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(op) = serde_json::from_str::<CtrlOp>(&text) {
                            let _ = ctrl_tx.send(op);
                        }
                    }
                    Some(Ok(_)) => {}
                    Some(Err(_)) | None => break,
                }
            }
        }
    }
}
/// Start the viz server on its own tokio runtime thread.
pub fn start(
    port: u16,
    mode: &'static str,
    exp_id: &str,
    duration_ms: u64,
) -> ServerHandle {
    let (tx, _rx) = broadcast::channel(8192);
    let (ctrl_tx, ctrl_rx) = std::sync::mpsc::channel::<CtrlOp>();
    let state = Arc::new(Mutex::new(VizState::default()));
    let shared = Shared {
        tx: tx.clone(),
        ctrl_tx,
        state: state.clone(),
        hello: Arc::new(
            serde_json::json!({
                "t": "hello",
                "mode": mode,
                "exp_id": exp_id,
                "duration_ms": duration_ms,
            })
            .to_string(),
        ),
    };
    let shared_for_serve = Arc::new(shared);
    std::thread::Builder::new()
        .name("viz-server".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("tokio runtime");
            rt.block_on(serve(port, shared_for_serve));
        })
        .expect("spawn viz thread");
    ServerHandle {
        port,
        ctrl_rx,
        state,
        tx,
        mode,
        exp_id: exp_id.to_string(),
        duration_ms,
    }
}

/// Replay driver: streams a recorded telemetry file through the same WS
/// protocol at `speed` ticks/s (0 = max). Handles pause/step/speed/seek.
pub fn replay(telemetry_path: &std::path::Path, port: u16, mut speed: f32) -> std::io::Result<()> {
    let events = anima_telemetry::read_telemetry(telemetry_path)?;
    let duration_ms = events.last().map(|e| e.t).unwrap_or(0);
    let exp_id = events
        .first()
        .map(|e| e.exp_id.clone())
        .unwrap_or_else(|| "replay".into());

    let handle = start(port, "replay", &exp_id, duration_ms);
    eprintln!("replay: {} events, open http://localhost:{port}", events.len());

    // Precompute ui-able structure from events (weights approximate: last
    // known w from synapse-created / strengthened / weakened).
    let mut struct_state = ReplayStructure::default();
    let mut idx: usize = 0;
    let mut paused = false;
    let mut step_pending: u64 = 0;
    let mut pace_anchor = std::time::Instant::now();
    let mut pace_anchor_t = 0u64;

    loop {
        // Controls.
        while let Ok(op) = handle.ctrl_rx.try_recv() {
            match op {
                CtrlOp::Pause => paused = true,
                CtrlOp::Resume => {
                    paused = false;
                    step_pending = 0;
                }
                CtrlOp::Step { n } => {
                    step_pending = step_pending.saturating_add(n);
                    paused = false;
                }
                CtrlOp::Speed { factor } => speed = factor.clamp(1.0, 10_000.0),
                CtrlOp::Seek { tick } => {
                    // Rebuild approximate structure from events ≤ tick and
                    // re-anchor pacing so playback resumes from the seek
                    // point at the requested speed (not fast-forwarding).
                    idx = events.partition_point(|e| e.t <= tick);
                    struct_state = ReplayStructure::default();
                    for e in &events[..idx] {
                        struct_state.apply(e);
                    }
                    pace_anchor = std::time::Instant::now();
                    pace_anchor_t = tick;
                    push_structure(&handle, &struct_state, tick, paused, speed);
                }
            }
        }
        if paused && step_pending == 0 {
            std::thread::sleep(std::time::Duration::from_millis(20));
            continue;
        }
        let Some(ev) = events.get(idx) else { break };
        // Periodic structure refresh + spike batching is inside send_event.
        send_event(&handle, ev, &mut struct_state);
        if ev.t % 10 == 0 {
            push_structure(&handle, &struct_state, ev.t, paused, speed);
        }
        idx += 1;
        // Pace: event t vs wall time, anchored at the last seek/join point.
        if speed > 0.0 {
            let target_elapsed_ms =
                (ev.t.saturating_sub(pace_anchor_t)) as f64 / speed as f64 * 1000.0;
            let actual = pace_anchor.elapsed().as_secs_f64() * 1000.0;
            if target_elapsed_ms > actual {
                std::thread::sleep(std::time::Duration::from_secs_f64(
                    (target_elapsed_ms - actual) / 1000.0,
                ));
            }
        }
    }
    eprintln!("replay complete");
    Ok(())
}

#[derive(Default)]
struct ReplayStructure {
    neurons: std::collections::BTreeMap<u32, (String, String, f32, &'static str)>,
    synapses: std::collections::BTreeMap<u32, (u32, u32, f32)>,
}

impl ReplayStructure {
    fn apply(&mut self, env: &Envelope) {
        use anima_telemetry::events::Payload;
        match &env.payload {
            Payload::RunStarted { params, .. } => {
                let ni = params.get("n_input_channels").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let nn = params.get("n_internal").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let no = params.get("n_output").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let gs = params.get("group_size").and_then(|v| v.as_u64()).unwrap_or(8) as u32;
                for i in 0..ni + nn + no {
                    let cls = if i < ni { "input" } else if i < ni + nn { "internal" } else { "output" };
                    let group = if i < ni {
                        ["A", "B", "C", "D"][((i / gs.max(1)) as usize) % 4].to_string()
                    } else {
                        String::new()
                    };
                    self.neurons.insert(i, (cls.into(), group, 0.0, "normal"));
                }
            }
            Payload::SynapseCreated { syn, pre, post, w, .. } => {
                self.synapses.insert(syn.0, (pre.0, post.0, w.unwrap_or(0.0)));
            }
            Payload::SynapsePruned { syn, .. } => {
                self.synapses.remove(&syn.0);
            }
            Payload::SynapseStrengthened { syn, delta } => {
                if let Some(s) = self.synapses.get_mut(&syn.0) {
                    s.2 = (s.2 + delta.unwrap_or(0.0)).clamp(0.0, 1.0);
                }
            }
            Payload::SynapseWeakened { syn, delta } => {
                if let Some(s) = self.synapses.get_mut(&syn.0) {
                    s.2 = (s.2 - delta.unwrap_or(0.0)).clamp(0.0, 1.0);
                }
            }
            Payload::NeuronCreated { n, .. } => {
                self.neurons.entry(n.0).or_insert(("internal".into(), String::new(), 0.0, "normal"));
            }
            Payload::NeuronDormant { n, .. } => {
                if let Some(v) = self.neurons.get_mut(&n.0) {
                    v.3 = "dormant";
                }
            }
            Payload::NeuronReactivated { n, .. } => {
                if let Some(v) = self.neurons.get_mut(&n.0) {
                    v.3 = "normal";
                }
            }
            Payload::NeuronRetired { n, .. } => {
                if let Some(v) = self.neurons.get_mut(&n.0) {
                    v.3 = "retired";
                }
            }
            _ => {}
        }
    }
}

fn send_event(handle: &ServerHandle, env: &Envelope, st: &mut ReplayStructure) {
    use anima_telemetry::events::Payload;
    if matches!(env.payload, Payload::Spike { .. }) {
        // Batch handled by caller? For replay simplicity send per-spike frames
        // grouped by identical t happens naturally at max speed.
        let ids = match &env.payload {
            Payload::Spike { n } => vec![n.0],
            _ => vec![],
        };
        let v = serde_json::json!({"msg": "spikes", "t": env.t, "ids": ids});
        let _ = handle.tx.send(v.to_string());
        return;
    }
    st.apply(env);
    handle.broadcast_event(env);
}

fn push_structure(handle: &ServerHandle, st: &ReplayStructure, tick: u64, paused: bool, speed: f32) {
    let neurons = st
        .neurons
        .iter()
        .map(|(id, (cls, group, rate, state))| UiNeuron {
            id: *id,
            cls: cls.clone(),
            group: group.clone(),
            rate: *rate,
            state: state.to_string(),
        })
        .collect();
    let synapses = st
        .synapses
        .iter()
        .map(|(id, (src, dst, w))| UiSynapse { id: *id, src: *src, dst: *dst, w: *w })
        .collect();
    handle.update_state(tick, paused, speed, neurons, synapses);
}
