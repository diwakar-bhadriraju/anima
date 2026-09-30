# ANIMA Phase 0 — Minimal Observable Experimental Loop

## Context

ANIMA is a research program building an experimental developmental artificial nervous system: we fix boundaries (sensory in, output out), environment, curriculum, resource limits, and fundamental plasticity laws; the organism grows its own internal topology. The repository root holds this plan plus the first real experiments.

User-locked platform decisions (recorded as decision-log entries D1–D8):
- **Language**: Rust (workspace, deterministic, seeded).
- **Neural paradigm**: spiking LIF neurons, event/tick based, timing-native (1 ms tick).
- **Visualization**: browser UI over WebSocket, live + replay.
- **First deliverable**: the minimal observable loop — tiny organism → raw events → telemetry → live viz → analysis → auto report.
- **Sim time**: free-running with adjustable speed, pause/step controls; full-fidelity event log for replay.
- **Input boundary semantics (D8)**: input neurons are pure spike sources — the last deterministic stage of the organism's "body", not plastic tissue; plasticity begins at their input→internal synapses.

End state after this plan: `anima-run` executes experiment E1 (STDP-driven assembly formation + novelty response) with a live browser view of spiking/topology change, writes complete JSONL telemetry + snapshots to `runs/<id>/`, and auto-generates a pre-registered research report. Mechanism-level neuroscience unknowns are NOT silently decided — they are registered with candidates and assigned to experiments E2+.

## Research basis (know / don't know, evidence class, where each gets resolved)

| # | Unknown | Biology status | Candidate mechanisms | Resolved by |
|---|---|---|---|---|
| U1 | Neuron dynamics beyond bare LIF (adaptation, inhibition) | GREEN: spike-frequency adaptation, refractory periods are robust. YELLOW: which currents matter at our scale | (a) bare LIF+refractory, (b) LIF+adaptive threshold, (c) LIF+adaptation current; lateral inhibition circuits | E1 uses (a)+fixed refractory; (b)/(c) compared in E3 |
| U2 | Synaptic plasticity rule | GREEN: timing-dependent LTP/LTD exists (Bi & Poo). YELLOW: exact window shapes, multiplicative vs additive bounds, triplet terms | (a) pairwise STDP trace-based, (b) multiplicative-bound STDP, (c) triplet STDP, (d) reward-modulated STDP | E1 uses (a) additive bounds; (b)/(c) A/B in E2; (d) in E6 |
| U3 | Structural growth trigger (neuron birth) | RED/YELLOW: adult neurogenesis is restricted in biology; computational analogues are our invention | (a) homeostatic saturation (persistent over-activity), (b) persistent prediction error, (c) representational interference, (d) novelty-gated, (e) none (growth never useful) | E4 A/B/C/D — machinery built now, trigger is config-selected, E1 sets `none` for internal growth except probe births |
| U4 | Learning gate ("should this experience write?") | YELLOW: neuromodulators gate plasticity (dopamine/ACh evidence) but circuit details debated | (a) always-on, (b) novelty-gated, (c) prediction-error-gated, (d) global modulator scalar | (a) in E1; (b)–(d) in E5 |
| U5 | What a memory is here | RED: our choice; GREEN-adjacent: attractor/replay phenomena exist | (a) strengthened synapses only, (b) recurrent assemblies, (c) attractor states, (d) dormant-reactivable traces | E7 |
| U6 | Internal novelty/unknown signal | YELLOW: hippocampal novelty signals exist; computation debated | (a) external instrumentation-computed (E1), (b) learned internal mismatch signal | E1 uses (a); internalization in E5 |
| U7 | Reward/motivation architecture | YELLOW | scalar modulator field over synapse updates | deferred E6; schema reserves `modulator` field |
| U8 | Dormancy/retirement semantics | YELLOW: synaptic pruning GREEN-adjacent; neuron dormancy is our construct | thresholds on sustained rate + age | E4 |

Nothing in U1–U8 blocks Phase 0: every candidate is expressible as a config-selected module behind fixed interfaces (`PlasticityRule`, `BirthTrigger`, `LearningGate` traits).

## Approach

Ordered steps; each leaves the tree building (`cargo build`) and tests green.

### 1. Scaffold + governance docs
- `git init`; workspace `Cargo.toml` (edition 2021, `[workspace] members = ["crates/*"]`), `.gitignore` = `target/ runs/`.
- Crates: `anima-core` (organism), `anima-telemetry` (event schema, recorder, analyzer, report), `anima-viz` (WS server + embedded UI), `anima-exp` (binary `anima-run`).
- `docs/decision-log.md` seeded with D1–D8 (D6: JSONL telemetry v1; D7: E1 uses pairwise additive STDP as experimental default, not settled truth; D8: input channels are pure spike sources — exact stimulus control, plasticity starts downstream). Format per entry: question / alternatives / evidence class / experiment / decision / rationale / remaining uncertainty.
- `docs/unknowns-registry.md` seeded with U1–U8 rows above, each with: biology, uncertainty, candidates, predictions, experiment ref, status `open`, "unknown unknowns" empty section.
- No equivalent prior structure exists (empty repo) — this seed content is new by necessity.

### 2. `anima-core`: organism state + deterministic stepping
- `Network` owns `Vec<Neuron>`, `Vec<Synapse>` (adjacency via `Vec<Vec<u32>>`), seeded `rand_xoshiro::Xoshiro256PlusPlus` (rand 0.8).
- **Numeric conventions (all crates)**: neural state in `f32` (membrane v, weights w, traces, currents, rates); seeds in `u64`. Ids: newtypes `NeuronId(u32)`, `SynapseId(u32)`, `InputChannelId(u32)`; grown neurons continue the `NeuronId` index space (birth = next index ⇒ `NeuronCreated` needs no schema change). Time: `Tick(u64)`, 1 tick = 1 ms, reported as `SimMs = tick` (u64). Telemetry/WS serialization: f32 → JSON number with NaN/∞→`null` guard; ids/times → JSON integers.
- LIF, fixed tick `dt = 1 ms`: `v += (-(v - v_rest) + i_syn + i_ext) * dt / tau_m`; params v_rest=0, tau_m=20 ms, v_th=1.0, v_reset=0, refractory=2 ms. Synaptic current: exponential kernel, `i_syn` decays with tau_syn=5 ms, fixed amplitude×weight, current-based.
- `step(&mut self, input: &InputFrame) -> Vec<Event>`: pure w.r.t. seeded RNG state; same seed + same stimulus script ⇒ identical event stream (wall-clock never enters events). `InputFrame = { tick, spikes: Vec<InputChannelId> }` — the per-tick delivered spike set.
Input boundary (D8): `InputChannel { id, group }` is a **pure spike source** — no membrane state, no dynamics, no plasticity on the channel itself; it emits exactly the spikes the environment/encoder delivered, so "pattern A presented" maps 1:1 to afferent spikes and weight changes stay attributable. Its outgoing synapses ARE plastic tissue: input→internal synapses undergo STDP with delivered input spikes as pre-synaptic events.
- Stimulus medium (D9): E1–E5 stimuli are **synthetic config-defined patterns, no media files** — `configs/e1.toml` defines patterns (`id`, `channels` group, `rate_hz`, `duration_ms`, `jitter_ms`, plus `off_ms` in the stage entry) and stages (ordered `[stage] id / present / order = "interleaved"|"blocked"`); the harness expands them to deterministic seeded Poisson trains on input channels. Real images enter only at E9 via the deterministic retina encoder (brightness/contrast/edges/temporal change → `InputFrame` spikes; no CNNs/pretrained models per charter §4), audio at E10 via the cochlea encoder. Isolation rationale: with synthetic patterns every afferent spike is config-specified, so attribution of weight change to stimulus is exact; a retina between stimulus and network would confound "organism learned" with "retina encoded it that way".
- Output boundary: the 12 output neurons are ordinary LIF internal-class neurons — `NeuronClass::{Input, Internal, Output}` is a role tag, not a cell type; output neurons receive plastic synapses like any internal neuron, and their spikes are additionally emitted as `OutputActivity` events. The organism only emits spike patterns; interpretation into text/words is the external decoder's job, built in E8+.

### 3. Plasticity engine (`plasticity.rs`)
- Trait `PlasticityRule { fn update(&mut self, net: &mut Network, traces: &Traces, gate: f32) -> Vec<Event> }`.
- Implementation 1: pairwise STDP, trace form (per-synapse pre/post eligibility traces, tau_plus=tau_minus=20 ms; A+=0.005, A-=0.0053; weights clamped [0, 1.0]; update applied per tick). Weight deltas emit `SynapseStrengthened/Weakened` events only when |Δw| > 0.01 (coalesced; full precision in periodic snapshots).
- Passive decay: w -= 1e-6 per tick; synapses with w < 0.02 for > 60 s sim-time and age > 30 s emit `SynapsePruned` (reason: `silent-synapse`) and are removed.

### 4. Structural plasticity (`structural.rs`) — machinery now, trigger is config
- Trait `BirthTrigger { fn should_birth(&self, net: &Network, signals: &Signals) -> Option<Reason> }`; impls: `None`, `HomeostaticSaturation` (region mean rate > 25 Hz sustained 2 s), `PersistentError` (running prediction error > μ+2σ sustained 5 s). Reason object = `{ trigger, contributing: Vec<(factor, value)>, thresholds }` — causal metadata mandatory (§15).
- Birth: append neuron, wire 20 sparse synapses to most-recently-coactive partners (seeded), emit `NeuronCreated { reason }`.
- Dormancy: rate < 0.1 Hz for 30 s ⇒ `NeuronDormant`; recovery > 1 Hz ⇒ `NeuronReactivated`; dormant > 300 s ⇒ `NeuronRetired` (synapses pruned with reason `neuron-retired`).
- E1 runs `None` for internal births (birth machinery still unit-tested); E4 selects among triggers.

### 5. Resource economy + failure states (`resources.rs`)
- Caps in config: `max_neurons`, `max_synapses`, `births_per_window`. Metabolic ledger: cost = neurons×c_n + synapses×c_s + spikes×c_spike per tick, emitted in `ResourceUsage` each 100 ticks; exceeding caps emits `Failure { kind: "resource-exhaustion", ... }`.
- Detectors: runaway (mean rate > 50 Hz for 5 s), fragmentation (largest weakly-connected component < 60% of neurons, checked each 1000 ticks) ⇒ `Failure` event + graceful `RunEnded { reason }`, telemetry file closed and preserved (§22). Failed run is data; harness never auto-restarts.

### 6. `anima-telemetry`: schema, recorder, analyzer, report
- `events.rs`: serde-tagged envelope `{ seq, t (sim ms), exp_id, kind, payload }`. Kinds: `RunStarted` (config hash, seed, params), `RunEnded`, `TickStats` (rates, active counts — every tick, cheap), `Spike`, `OutputActivity`, `StimulusPresented { pattern_id, stage }`, `SynapseCreated/Pruned`, `SynapseStrengthened/Weakened`, `NeuronCreated/Dormant/Reactivated/Retired`, `PredictionError` (E1: instrumentation-computed input-mismatch), `NoveltySignal`, `ResourceUsage`, `Failure`. All structural events carry `reason`.
- Serialization contract (telemetry + WS wire share it): envelope `t` = u64 sim-ms, `seq` = u64 monotonic, ids = u64, floats = f32-valued JSON numbers with NaN/∞→`null` (analyzer treats `null` as missing).
- Recorder: append-only JSONL `runs/<id>/telemetry.jsonl`; snapshot `network-state` (full neurons+synapses) every 1000 ticks to `snapshots.bin.zst` frames; run dir = `runs/<exp>-<UTC timestamp>/`, `create_dir_all`, refuse existing dir. Recorder write error ⇒ abort run, keep partial file.
- Analyzer (pure function telemetry→`metrics.json`): per-neuron per-pattern mean rate; selectivity index `(r_best − r_2nd)/r_best` per internal neuron; assembly score = mean within-pattern cosine similarity of neuron response vectors vs cross-pattern (computed early-S1 vs late-S1 windows); novelty response = response-profile distance of pattern D to each learned pattern + D's population rate vs learned baseline; retention = A/B/C response late vs post-D; structure timeline (births, prunes, dormancies, synapse count, weight histogram over time); resource timeline.
- Report generator: emits `report.md` with all §21 sections. Hypothesis/setup/organism-config from protocol+config; results/structural/behavioral/resource from metrics; **interpretation verdicts are rule-based from pre-registered thresholds** (below), each labeled `auto-generated — review`; failure modes from `Failure` events; next-experiment pointer to the protocol's follow-up.
### 7. Environment + stimulus scheduler (`env.rs` in `anima-exp`)
- Pattern spec (TOML): group of input channels, rate (Hz), duration, jitter. Deterministic Poisson spike trains from seeded RNG (seed derived from run seed + pattern id).
- Curriculum script: named stages, each = ordered list of (pattern, repetitions); probes = 2 s silence markers emitting `StimulusPresented{pattern:"silence"}`.
- E1 curriculum (pre-registered in `docs/e1-protocol.md`):
  - Organism: 24 input (groups A/B/C × 8), 40 internal, 12 output. Caps: 200 neurons / 2000 synapses. Birth trigger `none`.
  - S0: 5 s silence baseline probe.
  - S1: 360 presentations (120 × A, B, C interleaved uniformly): 500 ms burst at 20 Hz on the group's 8 channels + 1500 ms off.
  - S2: 30 presentations of novel pattern D (never in S1), same timing.
  - S3: 45 re-test presentations (15 × A, B, C).
- Pre-registered verdict thresholds (auto-report uses exactly these):
  - "assembly formation **supported**": assembly score late-S1 ≥ 2× early-S1 AND median selectivity of internal neurons > 0.5; **weakly supported** if only one holds; else **inconclusive**.
  - "novelty discrimination **supported**": D's response-profile distance to nearest learned pattern ≥ 2× the max A↔B↔C pairwise distance; else **inconclusive**.
  - "retention **supported**": S3 response ≥ 80% of late-S1 for all of A/B/C; 50–80% **weakly supported**; < 50% **inconsistent**.

### 8. `anima-viz`: server + embedded web UI
- `axum` (ws feature) + `tokio`; sim runs on a dedicated std thread; cross-thread via `tokio::sync::broadcast` (events) + `std::sync::mpsc` (control). `--headless` skips server. `replay <file>` subcommand streams a recorded JSONL through the identical WS protocol at recorded pace (or max speed).
- WS protocol (JSON): server→client `{t:"snapshot", neurons:[], synapses:[]}` on join, then `{t:"ev", ...}` for every event (spikes batched per 10 ticks to bound traffic), `{t:"stats"}` decimated by speed factor. Client→server `{t:"ctrl", op:"pause"|"resume"|"step", n}` and `{op:"speed", factor}` (1–10000×, default 1000 ticks/s).
- Web UI: single `index.html` + vanilla JS/Canvas, embedded via `include_str!`, no build step. Views (registered in a client-side view registry so more can be added without touching organism code, §18):
  - **Network**: force-directed layout (deterministic, seeded by neuron id), inputs pinned left column, outputs right, internal free; node fill = recent firing rate, ring = structural state (normal/dormant/retiring), flash on spike; edge width/opacity = weight, red/green flash on weaken/strengthen, appearance/disappearance animated.
  - **Event stream**: scrolling feed of structural + error events with reasons (NEW NEURON / NEW SYNAPSE / … per §17).
  -Inspector: click node → id, age, creation reason, rates, in/out synapse lists, recent spike history (client-side ring buffer); click edge → src/dst, w, age, last Δw.
  - **Controls**: pause / resume / step×1 / step×100 / speed slider / sim-time display. Timeline scrub is replay-mode only (v1).

### 9. `anima-exp`: harness
- Config: single TOML per experiment (`configs/e1.toml`) — organism params, plasticity selection, structural trigger, caps, curriculum ref, seed, viz port (default 8788).
- Run loop: apply pending control commands → `env.step()` → `net.step()` → recorder + broadcaster; speed gate = sleep to hold ticks/sec; `RunStarted` records config hash + seed for reproducibility.
- `anima-run run|replay|report` subcommands (`report` re-analyzes an existing run dir).

### 10. E1 protocol + execution
- Write `docs/e1-protocol.md` (hypothesis, pre-registered thresholds from step 7, follow-up = E2: STDP bound variants A/B; E3: adaptation; E4: birth triggers; E5: learning gates; E6: reward modulation; E7: memory mechanisms; E8: word/output decoder; E9: retina; E10: cochlea — roadmap recorded in unknowns registry, built only when reached).
- Execute E1 headless-full + one live-observed run; generate reports; commit scaffold + docs + configs (not `runs/`).

## Critical files & anchors

- `crates/anima-core/src/network.rs` — LIF `Neuron`, `Synapse`, `Network::step`; the organism's only mutable entry point.
- `crates/anima-core/src/plasticity.rs` — `PlasticityRule` trait + STDP impl; where E2/E3 variants will live.
- `crates/anima-core/src/structural.rs` — `BirthTrigger` impls; reason/causal-metadata objects.
- `crates/anima-telemetry/src/events.rs` — event envelope + kinds; every consumer (recorder, viz, analyzer) depends on this schema.
- `crates/anima-viz/src/server.rs` + `crates/anima-viz/web/index.html` — WS protocol, control ops, canvas rendering.

## Verification

0. Prereq: `cargo --version` ≥ 1.75 (if absent: install rustup — user-approved step).
1. `cargo test --workspace`: LIF spike-at-threshold + decay math; STDP pre-before-post potentiates / post-before-pre depresses (sign tests with crafted spike pairs); weight clamp at bounds; input semantics: an `InputChannel` emits exactly the delivered spike set across consecutive frames (no state, no drift) while its input→internal synapse weight changes only via STDP; birth/prune/dormant events fire with reasons; recorder round-trip (write events → parse → equal); replay determinism: two headless E1 runs, same seed → `sha256sum` of both `telemetry.jsonl` identical.
2. Live: `cargo run -p anima-exp -- run --config configs/e1.toml --live` → open `http://localhost:8788`: observe spikes flashing on input groups during bursts, weights emerging between input/internal nodes, event feed scrolling with reasons; pause freezes activity, step×100 advances exactly, speed slider changes tick pace; click a neuron → inspector shows creation reason + synapses. (Manual browser check — this is the §16 acceptance bar.)
3. Headless E1 → `runs/e1-*/report.md` exists, contains all §21 sections, `metrics.json` has selectivity/assembly/novelty/retention values well-formed (finite, non-NaN). NOTE: verification asserts pipeline correctness, not that hypotheses pass — outcome is data either way.
4. Failure path: crafted config with `max_neurons: 45` and forced birth trigger → run ends with `Failure{resource-exhaustion}` in telemetry, report's failure-modes section populated, files preserved.
5. Replay: `anima-run replay runs/e1-*/telemetry.jsonl` → same UI renders recorded events at paced speed.

## Assumptions & contingencies

- **JSONL telemetry suffices** at E1 scale (~10⁶ events/run). If a run exceeds ~1 GB, switch recorder framing to length-prefixed bincode events with a JSONL converter tool — schema unchanged (D6 amendment, no conversation needed).
- **axum/tokio** for WS; if a dependency conflict blocks it, fall back to `tungstenite` standalone with the identical protocol — UI unaffected.
- Novelty is computed by instrumentation in E1 (U6a); the report must not claim the organism "represents" novelty — wording pinned in the report template.
- Auto-interpretation is threshold-rule output labeled `auto-generated — review`; humans own final interpretation language (§2 discipline).
- Sim is single-threaded by design (determinism > speed at this scale); parallelism deferred until a mechanism demands it and is re-registered as a decision.
- If E1's default STDP params produce no weight movement (all-dead or all-saturated), that is an E1 result (recorded as such), and the protocol's pre-registered fallback is one param sweep (A± ∈ {0.002, 0.005, 0.01}) — still within E1's scope, logged as an amendment.
- Browser is local-only (`127.0.0.1` bind); no auth in v1.
