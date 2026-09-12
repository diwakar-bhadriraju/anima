# ANIMA Decision Log

Every locked platform decision gets an entry. Format: **question / alternatives /
evidence class / experiment / decision / rationale / remaining uncertainty**.
Evidence classes: GREEN (robust biology), YELLOW (debated/partial), RED (our
invention, biology restricted or absent), TEAL (engineering choice, no biology
claim).

---

## D1 — Implementation language

- **Question**: What language implements the organism and harness?
- **Alternatives**: Rust; Python (numpy); Julia; C++.
- **Evidence class**: TEAL.
- **Experiment**: none (platform).
- **Decision**: Rust workspace, deterministic, seeded everywhere.
- **Rationale**: Determinism is a hard requirement (replay determinism check:
  same seed ⇒ identical telemetry hash). Rust gives this without a GC pause
  story and keeps the sim single-threaded by design.
- **Remaining uncertainty**: none blocking; parallelism re-registered when a
  mechanism demands it.

## D2 — Neural paradigm

- **Question**: Rate-coded or spiking neurons?
- **Alternatives**: rate units; spiking LIF; Izhikevich; conductance-based.
- **Evidence class**: TEAL, informed by GREEN biology (LIF captures
  spike-timing dependence at minimal cost).
- **Experiment**: E1+.
- **Decision**: Spiking LIF, event/tick based, 1 ms tick, timing-native.
- **Rationale**: The research program is about development through timing
  (STDP first); timing must be in the substrate, not bolted on.
- **Remaining uncertainty**: neuron dynamics variants (U1) — E3.

## D3 — Visualization

- **Question**: How do we observe the organism?
- **Alternatives**: terminal; desktop app; browser over WebSocket.
- **Evidence class**: TEAL.
- **Experiment**: none (platform).
- **Decision**: Browser UI over WebSocket; live and replay share one protocol.
- **Rationale**: Zero-install, canvas 2D suffices at E1 scale, replay = feed
  recorded events through the same socket.
- **Remaining uncertainty**: scale limits of naive canvas beyond ~10⁴ edges.

## D4 — First deliverable

- **Question**: What is the first end-to-end artifact?
- **Alternatives**: learning benchmark; the minimal observable loop.
- **Evidence class**: TEAL.
- **Experiment**: E1.
- **Decision**: tiny organism → raw events → telemetry → live viz → analysis →
  auto report.
- **Rationale**: Observation before intervention; the loop makes every later
  experiment cheap to run and inspect.
- **Remaining uncertainty**: none.

## D5 — Sim time model

- **Question**: Fixed-timestep batch runs or interactive time?
- **Alternatives**: fixed steps headless-only; free-running with controls.
- **Evidence class**: TEAL.
- **Experiment**: none (platform).
- **Decision**: Free-running with adjustable speed, pause/step controls;
  full-fidelity event log for replay; wall-clock never enters events.
- **Rationale**: Observation is interactive; determinism preserved because
  pacing is outside the event stream.
- **Remaining uncertainty**: none.

## D6 — Telemetry format

- **Question**: How is run data recorded?
- **Alternatives**: SQLite; binary framed; JSONL.
- **Evidence class**: TEAL.
- **Experiment**: none (platform).
- **Decision**: Append-only JSONL `runs/<id>/telemetry.jsonl`; envelope
  `{seq, t, exp_id, kind, payload}`; `t` = u64 sim-ms; snapshots every 1000
  ticks to `snapshots.bin.zst` frames (full network state).
- **Rationale**: Self-describing, streamable (viz replay reads the same bytes),
  adequate at E1 scale (~10⁶ events). If a run exceeds ~1 GB, switch to
  length-prefixed bincode frames with a JSONL converter — schema unchanged.
- **Remaining uncertainty**: size ceiling; measured per-run.

## D7 — E1 plasticity default

- **Question**: Which plasticity rule for E1?
- **Alternatives**: pairwise additive STDP; multiplicative bounds; triplet;
  reward-modulated.
- **Evidence class**: U2 — timing-dependent LTP/LTD GREEN (Bi & Poo), window
  shapes/bounds YELLOW.
- **Experiment**: E1 uses pairwise additive; E2 A/B tests bounds; E6 reward.
- **Decision**: Pairwise trace-based STDP, additive bounds, A+=0.005,
  A-=0.0053, tau±=20 ms, weights [0,1].
- **Rationale**: Simplest rule that is timing-dependent; asymmetry toward
  depression guards against runaway potentiation at these bounds. Recorded as
  experimental default, **not settled truth**.
- **Remaining uncertainty**: window shapes, multiplicative vs additive bounds,
  higher-order terms (E2/E3).

## D8 — Input boundary semantics

- **Question**: Are input neurons part of the organism or part of the body?
- **Alternatives**: plastic input layer; pure spike sources.
- **Evidence class**: TEAL (boundary convention).
- **Experiment**: none (platform semantics).
- **Decision**: Input channels are pure spike sources — the last deterministic
  stage of the organism's "body", not plastic tissue. No membrane, no
  dynamics, no plasticity on the channel itself; plasticity begins at
  input→internal synapses where delivered input spikes are pre-synaptic
  events.
- **Rationale**: "Pattern A presented" maps 1:1 to afferent spikes, so weight
  change attribution to stimulus is exact.
- **Remaining uncertainty**: none at this boundary; encoder realism is E9/E10.

## D9 — Stimulus medium for E1–E5

- **Question**: What drives the input channels in early experiments?
- **Alternatives**: real media through encoders; synthetic config-defined
  patterns.
- **Evidence class**: TEAL.
- **Experiment**: E1–E5 synthetic; E9 retina; E10 cochlea.
- **Decision**: Synthetic config-defined patterns (`configs/e1.toml`: pattern
  id, channel group, rate_hz, duration_ms, jitter_ms, off_ms; stages ordered,
  interleaved|blocked). Harness expands to deterministic seeded Poisson trains.
- **Rationale**: Every afferent spike is config-specified ⇒ attribution of
  weight change to stimulus is exact. An encoder between stimulus and network
  would confound "organism learned" with "retina encoded it that way".
- **Remaining uncertainty**: none for E1–E5; encoder design enters at E9.
