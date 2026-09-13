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

#[derive(Debug, Clone, Deserialize, PartialEq)]
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
    /// Shared with the WS acceptor: one-shot frames delivered on join.
    timeline_frame: Arc<Mutex<Option<String>>>,
}

impl ServerHandle {
    pub fn set_timeline_frame(&self, s: String) {
        *self.timeline_frame.lock() = Some(s);
    }

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
const SOCKET_WORKER_JS: &str = include_str!("../web/socket-worker.js");

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

    async fn worker_js() -> impl axum::response::IntoResponse {
        (
            [(axum::http::header::CONTENT_TYPE, "application/javascript")],
            SOCKET_WORKER_JS,
        )
    }


    async fn ws_handler(
        ws: WebSocketUpgrade,
        State(shared): State<Arc<Shared>>,
    ) -> impl axum::response::IntoResponse {
        let owned = (*shared).clone();
        ws.on_upgrade(move |socket| async move { client_loop(socket, owned).await })
    }

    // GET /runs: list recorded run dirs (for the replay picker). Each entry:
    // name, telemetry path if present, size in bytes, mtime (unix secs).
    async fn runs_index() -> axum::Json<serde_json::Value> {
        let mut runs: Vec<serde_json::Value> = Vec::new();
        if let Ok(rd) = std::fs::read_dir("runs") {
            let mut dirs: Vec<_> = rd
                .filter_map(|e| e.ok())
                .filter(|e| e.path().join("telemetry.jsonl").is_file())
                .collect();
            dirs.sort_by_key(|e| std::cmp::Reverse(e.file_name()));
            for e in dirs {
                let p = e.path().join("telemetry.jsonl");
                let (size, mtime) = match std::fs::metadata(&p) {
                    Ok(m) => (
                        m.len(),
                        m.modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs())
                            .unwrap_or(0),
                    ),
                    Err(_) => (0, 0),
                };
                runs.push(serde_json::json!({
                    "name": e.file_name().to_string_lossy(),
                    "telemetry": p.to_string_lossy(),
                    "bytes": size,
                    "mtime": mtime,
                }));
            }
        }
        axum::Json(serde_json::json!({ "runs": runs }))
    }

    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(ws_handler))
        .route("/runs", get(runs_index))
        .route("/socket-worker.js", get(worker_js))
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
    /// One-shot frames (timeline summary) delivered to every client on join.
    timeline_frame: Arc<Mutex<Option<String>>>,
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
    // One-shot frames broadcast before this client joined (e.g. the replay
    // timeline summary) are delivered here so late joiners see them.
    let tl_frame = shared.timeline_frame.lock().clone();
    if let Some(tl) = tl_frame {
        if sender.send(Message::Text(tl)).await.is_err() {
            return;
        }
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
    let timeline_frame = Arc::new(Mutex::new(None));
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
        timeline_frame: timeline_frame.clone(),
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
        timeline_frame,
    }
}

/// Replay driver: streams a recorded run (v2 chunks or legacy JSONL)
/// through the same WS protocol at `speed` ticks/s (0 = max). Handles
/// pause/step/speed/seek.
pub fn replay(telemetry_path: &std::path::Path, port: u16, mut speed: f32) -> std::io::Result<()> {
    // v2 run dir layout: <run>/telemetry/ + snapshots; legacy: a JSONL file.
    let chunk_dir = telemetry_path
        .join("telemetry");
    let events: Vec<Envelope> = if chunk_dir.is_dir() {
        let reader = anima_telemetry::TelemetryReader::open(&chunk_dir)?;
        let mut out = Vec::new();
        for c in 0..reader.chunk_index().len() {
            for row in reader.chunk_rows(c)? {
                out.push(row.envelope("replay")?);
            }
        }
        out
    } else {
        anima_telemetry::read_telemetry(telemetry_path)?
    };
    let duration_ms = events.last().map(|e| e.t).unwrap_or(0);
    let exp_id = events
        .first()
        .map(|e| e.exp_id.clone())
        .unwrap_or_else(|| "replay".into());

    let handle = start(port, "replay", &exp_id, duration_ms);
    eprintln!("replay: {} events, open http://localhost:{port}", events.len());

    // Timeline summary: 1 s bins over the whole run (spike counts by
    // population, stimulus markers, notable events). One precomputed frame
    // lets the client draw an activity-over-time scrubber without
    // replaying; curriculum phases come from stimulus-presented markers.
    let timeline = build_timeline(&events, duration_ms);
    let frame = serde_json::json!({ "t": "timeline", "bin_ms": TIMELINE_BIN_MS, "bins": timeline })
        .to_string();
    handle.set_timeline_frame(frame.clone());
    handle.send_json(frame);

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

/// Timeline bin size (1 s).
pub const TIMELINE_BIN_MS: u64 = 1000;

/// One precomputed 1 s bin of the timeline summary.
#[derive(Debug, Clone, Serialize)]
pub struct TimelineBin {
    /// Bin start in sim-ms.
    pub t: u64,
    /// Total spikes in the bin.
    pub spikes: u64,
    /// Spikes from input-class neurons.
    pub spikes_input: u64,
    /// Spikes from output-class neurons.
    pub spikes_output: u64,
    /// Count of notable events (structural, failure, novelty) in the bin.
    pub events: u64,
    /// Stimulus pattern presented at the bin start, if any ("silence" too).
    pub pattern: Option<String>,
    /// Curriculum stage at the bin start, if any.
    pub stage: Option<String>,
}

/// Precompute the timeline summary: coarse activity + markers over the
/// whole run so the client can render a scrubbable overview instantly.
pub fn build_timeline(events: &[Envelope], duration_ms: u64) -> Vec<TimelineBin> {
    use anima_telemetry::events::Payload;
    let n_bins = (duration_ms / TIMELINE_BIN_MS + 1) as usize;
    let mut bins: Vec<TimelineBin> = (0..n_bins)
        .map(|i| TimelineBin {
            t: i as u64 * TIMELINE_BIN_MS,
            spikes: 0,
            spikes_input: 0,
            spikes_output: 0,
            events: 0,
            pattern: None,
            stage: None,
        })
        .collect();
    let n_bins_max = n_bins.saturating_sub(1);
    let bin_of = |t: u64| ((t / TIMELINE_BIN_MS) as usize).min(n_bins_max);
    // Population membership from RunStarted params: input ids are
    // [0, n_input), output ids are the LAST n_output ids. Output spikes
    // are also counted via OutputActivity events (belt and braces).
    let (n_input, out_lo, n_total) = events
        .iter()
        .find_map(|e| match &e.payload {
            Payload::RunStarted { params, .. } => {
                let ni = params.get("n_input_channels").and_then(|v| v.as_u64()).unwrap_or(0);
                let nn = params.get("n_internal").and_then(|v| v.as_u64()).unwrap_or(0);
                let no = params.get("n_output").and_then(|v| v.as_u64()).unwrap_or(0);
                let total = ni + nn + no;
                Some((ni, total.saturating_sub(no), total))
            }
            _ => None,
        })
        .unwrap_or((0, u64::MAX, u64::MAX));
    for e in events {
        let b = &mut bins[bin_of(e.t)];
        match &e.payload {
            Payload::Spike { n } => {
                b.spikes += 1;
                let id = n.0 as u64;
                if id < n_input {
                    b.spikes_input += 1;
                } else if id >= out_lo && id < n_total {
                    b.spikes_output += 1;
                }
            }
            Payload::StimulusPresented { pattern_id, stage } => {
                let bi = bin_of(e.t);
                bins[bi].pattern = Some(pattern_id.clone());
                bins[bi].stage = Some(stage.clone());
            }
            Payload::NeuronCreated { .. }
            | Payload::NeuronDormant { .. }
            | Payload::NeuronReactivated { .. }
            | Payload::NeuronRetired { .. }
            | Payload::SynapseCreated { .. }
            | Payload::SynapsePruned { .. }
            | Payload::Failure { .. }
            | Payload::NoveltySignal { .. }
            | Payload::RunStarted { .. }
            | Payload::RunEnded { .. } => b.events += 1,
            _ => {}
        }
    }
    bins
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

#[cfg(test)]
mod tests {
    use super::*;
    use anima_core::network::{NeuronId, SynapseId};
    use anima_telemetry::events::{EventBuilder, Payload, ReasonPayload};

    #[test]
    fn timeline_bins_count_spikes_by_population() {
        let mut b = EventBuilder::new("t");
        let mut events = Vec::new();
        // 2 input / 1 internal / 1 output: ids 0,1 input; 2 internal; 3 output.
        events.push(b.build(0, Payload::RunStarted {
            config_hash: "h".into(),
            seed: 1,
            params: serde_json::json!({
                "n_input_channels": 2, "n_internal": 1, "n_output": 1,
            }),
        }));
        for t in [0u64, 500, 1500] {
            events.push(b.build(t, Payload::Spike { n: NeuronId(0) })); // input
        }
        events.push(b.build(1200, Payload::Spike { n: NeuronId(2) })); // internal
        events.push(b.build(2500, Payload::Spike { n: NeuronId(3) })); // output
        events.push(b.build(2500, Payload::OutputActivity { n: NeuronId(3) }));

        let bins = build_timeline(&events, 3000);
        assert_eq!(bins.len(), 4); // 0..=3000 ms in 1 s bins
        assert_eq!(bins[0].spikes, 2); // t=0, 500
        assert_eq!(bins[0].spikes_input, 2);
        assert_eq!(bins[0].spikes_output, 0);
        assert_eq!(bins[1].spikes, 2); // t=1200 internal + t=1500 input
        assert_eq!(bins[1].spikes_input, 1);
        assert_eq!(bins[2].spikes, 1); // t=2500 output
        assert_eq!(bins[2].spikes_output, 1);
        // RunStarted counts as an event in its bin.
        assert!(bins[0].events >= 1);
    }

    #[test]
    fn timeline_bins_carry_stage_and_pattern_markers() {
        let mut b = EventBuilder::new("t");
        let mut events = Vec::new();
        events.push(b.build(0, Payload::StimulusPresented {
            pattern_id: "A".into(),
            stage: "S1".into(),
        }));
        events.push(b.build(0, Payload::Spike { n: NeuronId(0) }));
        events.push(b.build(2000, Payload::StimulusPresented {
            pattern_id: "D".into(),
            stage: "S2".into(),
        }));
        events.push(b.build(2100, Payload::Failure {
            kind: "resource-exhaustion".into(),
            detail: "test".into(),
        }));
        let bins = build_timeline(&events, 3000);
        assert_eq!(bins[0].stage.as_deref(), Some("S1"));
        assert_eq!(bins[0].pattern.as_deref(), Some("A"));
        assert_eq!(bins[0].spikes, 1);
        assert_eq!(bins[2].stage.as_deref(), Some("S2"));
        assert_eq!(bins[2].pattern.as_deref(), Some("D"));
        assert_eq!(bins[2].events, 1); // failure counted, stimulus not
    }

    #[test]
    fn timeline_handles_empty_and_missing_params() {
        let mut b = EventBuilder::new("t");
        let events = vec![b.build(0, Payload::Spike { n: NeuronId(9) })];
        let bins = build_timeline(&events, 500);
        assert_eq!(bins.len(), 1);
        assert_eq!(bins[0].spikes, 1);
        assert_eq!(bins[0].spikes_input, 0); // no RunStarted: nothing is input
        assert_eq!(bins[0].spikes_output, 0);
        let empty = build_timeline(&[], 0);
        assert_eq!(empty.len(), 1); // single empty bin, clamped index
        assert_eq!(empty[0].spikes, 0);
    }
    #[test]
    fn ctrl_op_parses_seek_and_speed() {
        // Internally-tagged enum: the tag is "op"; extra keys (like the
        // client's "t":"ctrl" envelope marker) are ignored by serde.
        let op: CtrlOp =
            serde_json::from_str(r#"{"op":"seek","tick":12345,"t":"ctrl"}"#).expect("seek parses");
        assert_eq!(op, CtrlOp::Seek { tick: 12345 });
        let op: CtrlOp =
            serde_json::from_str(r#"{"op":"speed","factor":250.0}"#).expect("speed parses");
        assert_eq!(op, CtrlOp::Speed { factor: 250.0 });
        let op: CtrlOp = serde_json::from_str(r#"{"op":"step","n":7}"#).expect("step parses");
        assert!(matches!(op, CtrlOp::Step { n: 7 }));
    }

    #[test]
    fn reason_payload_simple_is_causal_shell() {
        let r = ReasonPayload::simple("initial-wiring");
        assert_eq!(r.trigger, "initial-wiring");
        assert!(r.contributing.is_empty() && r.thresholds.is_empty());
        let s = SynapseId(3);
        assert_eq!(s.0, 3);
    }
}
