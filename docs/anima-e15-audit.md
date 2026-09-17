# ANIMA E15 — Structural-basis feasibility audit (protocol-design stage)

Status: AUDIT (2026-09-17). Not a frozen protocol. No implementation,
no execution, no simulation. Basis: committed E12 artifacts
(runs/e12-20260916T202445Z, runs/e12-seed9001-20260916T202822Z,
runs/e12-seed424242-20260916T202822Z) + telemetry schema
(anima-telemetry events.rs / chunks.rs / recorder.rs:300) + emission
sites (harness.rs, plasticity.rs, structural_v2.rs, network.rs).

## 0. Artifact inventory (per run dir)

- Columnar telemetry: Spike (3), StimulusPresented (5), SynapseCreated
  (6: syn, pre, post, w, reason), SynapsePruned (7: syn, reason),
  SynapseStrengthened/Weakened (8/9: syn, delta), Neuron* (10-13),
  ResourceUsage (16: neurons, synapses, live_exc, live_inh,
  spikes_window), Failure (17), TickStats (2).
- `snapshots.bin.zst`: full NetworkStateSnapshot frames every 1000
  ticks (t = tick % 1000 == 0, written AFTER that tick's dynamics):
  `neurons[]` (id, class, v, rate_hz, dormant, retired, born) and
  `synapses[]` (id, pre, post, w, plastic). E12 spans 813,500 ms =>
  814 frames, including ticks 364000/366000/368000/370000 (all
  multiples of 1000). Frame integrity already exercised by
  e12_transition/e13_trajectory (read_snapshots).
- metrics.json/report.md: analyzer aggregations (10,833
  SynapseCreated / 10,017 SynapsePruned for seed 20260912; zero
  failures).

## 1. Snapshot semantics (deterministic facts)

- Snapshot at tick T = full network state AFTER tick T's step
  (step/plasticity/windows/events precede the snapshot call in the
  harness loop; resources/snapshot are sections 9-10).
- Synapse ids are monotonically assigned at creation (`id = len`),
  never reused; pruned synapses are tombstoned (silent), not
  renumbered — creation/prune events join exactly onto ids.
- Inhibitory synapses are created with `plastic = false`
  (add_synapse_full(.., false, true, ..) in Network::new, M6 block);
  input/recurrent afferents and candidate-permanence synapses carry
  `plastic = true`. M4/eviction skip inhibitory synapses; M6 creates
  none after setup => inhibitory set is fixed, id-stable, and exactly
  identified by `plastic == false` in every snapshot.
- Initial topology is fully emitted at t=0 (harness bootstrap
  SynapseCreated events) — the complete creation history is in
  telemetry from the first synapse.
- SynapseStrengthened/Weakened events carry per-tick coalesced STDP
  deltas only when |ΣΔw| > 0.01; below-threshold STDP, per-tick
  passive decay (1e-6), M2 normalize rescaling, and M6 updates are
  SILENT (never emitted). Snapshot weights are the exact state at
  snapshot ticks; between ticks only the emitted component is
  timestamped.

## 2. Measurement classification

| # | Measurement | Feasibility | Source / exactness |
|---|---|---|---|
| 1 | Live excitatory synapse state (identity, pre, post, w) | **EXACT at snapshot ticks** | synapses[] plastic=true; w exact f32; candidates vs permanence: permanence = id with a kind-6 event reason `candidate-permanence` (id join; events never lost); initial = bootstrap reason |
| 2 | Live inhibitory synapse state | **EXACT at snapshot ticks** | synapses[] plastic=false; never pruned; cross-check live_inh |
| 3 | Synapse creation events (count, id, t, reason; pre/post/w) | **EXACT** | kind 6, per-event timestamp (ms), reason payload (bootstrap / candidate-permanence) |
| 4 | Synapse pruning events (count, id, t, reason) | **EXACT** | kind 7 (competitive-prune / budget-eviction; pre/post recoverable by id join) |
| 5a | Candidate -> permanence transitions (network side) | **EXACT** | kind 6 reason `candidate-permanence`, t, w = w_c_permanent |
| 5b | Candidate pool internals (pre choices, candidate weights, theta_die deaths, redraws, maturation latency) | **NOT RECONSTRUCTABLE** | live only in V2Plasticity.candidates; never snapshotted, deaths/births silent |
| 6 | Budget occupancy (M5) | **EXACT per window (100 ms)** | ResourceUsage live_exc/live_inh + totals, every window; independent cross-check from snapshots (count alive by id/event join) |
| 7 | Per-neuron structural changes | **EXACT at snapshot ticks; events exact** | Neuron* events (E12: birth_trigger none — expect zero, verify by count at analysis) + per-neuron synapse diff |
| 8 | Receptive-field changes (per-neuron afferent set + weights) | **EXACT at snapshot ticks** | excitatory afferents per post neuron from synapses[]; deltas across snapshots exact |
| 9 | Structural change specifically in [T0, REV1] | **EXACT topology + EXACT endpoint weights; PARTIAL intra-interval weight attribution** | topology: kind-6/7 events with ms timestamps; weights: endpoint snapshots exact, per-synapse Δw exact; the silent component (M2/M6/decay/STDP<0.01) is only an aggregate residual — not placeable in time |

## 3. Recoverable (exact list)

1. Complete creation/prune history with ms timestamps + reasons.
2. Complete synapse identity table (id -> pre/post/w at any snapshot
   tick; permanence status via event reason join; excitatory vs
   inhibitory via plastic flag).
3. Per-synapse endpoint weights at T0/REV1-pre/REV1-post (and any
   tick % 1000) — exact.
4. Per-synapse weight DELTAS across any snapshot pair — exact.
5. Per-neuron live excitatory/inhibitory counts + budgets at 10 Hz.
6. Per-neuron receptive fields (excitatory afferents + w) at snapshot
   ticks.
7. Behavioral alignment at REV1 (E14, already committed) — same
   artifacts.
8. Timestamped STDP weight events (|Δw| > 0.01 per tick) — a lower
   bound of the time-attributable plasticity within the interval.

## 4. NOT recoverable

1. Candidate pool contents and kinetics (births/deaths/weights/
   redraws; only maturations are visible).
2. Intra-interval temporal attribution of silent weight components
   (M2 normalize, M6 updates, passive decay, sub-threshold STDP) —
   aggregate residual only.
3. E6 RateBalance φ/β internal trajectory (effects embedded in
   weights).
4. Anything above the 1 s snapshot granularity for silent paths —
   no finer claim than "endpoint-exact, event-ordered".

## 5. T0 -> REV1 structure/behavior alignment — POSSIBLE

Registered instants (from E14: REV1 = the first REV-B presentation;
round 61, seed-dependent position pos: 20260912 pos 0, 9001/424242
pos 2; presentation windows [365000, 365500) / [369000, 369500)):

- T0 anchor (post-60-SEQ state): snapshot tick **364,000** (after
  tick 364000; nothing happens in [364000, 365000) — round 60 ends
  363,500). Same instant for all seeds.
- REV1-pre: snapshot at **364,000** (pos 0) / **368,000** (pos 2) —
  last frame strictly before S_B; for pos-2 seeds tick 368000
  includes round-61 A/C plasticity (no B content).
- REV1-post: snapshot at **366,000** / **370,000** — first frame
  after the presentation ends (365,500 / 369,500).
- Topology events: any kind-6/7 event with t in (REV1-pre, REV1-post]
  is exactly ordered against the presentation window — before/within/
  after resolvable at ms precision.
- Emitted STDP events (|Δw| > 0.01) similarly timestamped.

Temporal-relation claims allowed: "topology changes occurred before/
during/after the presentation window"; "weight endpoints changed by
X". Claims NOT allowed: causality ("X caused the flip"), attribution
of the silent residual to a sub-interval, any exact-time weight
trajectory inside a 1 s span.

The T0 control (does the A-side machinery already exist at T0?) is
answerable EXACTLY: the A-side structure at 364,000 is fully
observable (channels 0-7 afferent weights, per-neuron receptive
fields). This is the decided evidence for hypothesis B (re-expression
of existing structure) when combined with the (T0, REV1] topology
delta for hypothesis A (new structure).

## 6. E15 CAN BE ANALYSIS-ONLY

All quantities in sections 3 and 5 are reconstructable from the
committed artifacts with a read-only instrument (same class as
e12_transition/e13_trajectory: imports anima_telemetry + std only, no
anima_core, no Environment construction, read-only opens, deterministic).
No new organism behavior, no sim changes, no reruns.

### 6.1 Minimum frozen analysis (proposal — to be frozen after
### decisions in §8)

Per seed, from the committed runs:

1. Snapshot triple at ticks (364000, 366000|368000, 370000):
   (a) topology delta: kind-6/7 events with t in (pre, post], by
       reason, split by A-side/C-side pre-channel membership (channels
       0-7 = A-side, 8-15 = C-side; internal-recurrent reported
       separately); ordering vs the B presentation window.
   (b) per-synapse Δw for live excitatory synapses (endpoint diff);
       raw distributions only (no cutoffs).
   (c) A-side vs C-side total incoming weight (channels 0-7 vs 8-15
       -> non-input posts) at pre and post; the T0 control = same
       quantities at 364,000.
   (d) per-neuron receptive-field churn (afferent set symmetric
       difference, weight sums) — raw.
2. Cross-checks (gate): live_exc/live_inh at 10 Hz vs snapshot-derived
   alive counts; pruned-event ids all match created ids; determinism
   (rerun byte-equality); artifact hash pins (E13 freeze table).
3. Existence booleans (no thresholds): any new A-side synapse created
   in (pre, post]? any candidate-permanence maturation in the
   interval? any topology change at all? any A-side afferent weight
   change at endpoints?
4. Behavior join: same-interval alignment states from E14 (already
   committed, no recomputation).

## 7. Seed comparison

Same schema and instants for all three seeds; per-seed synapse ids
and per-seed structure reported separately; no cross-seed identity
assumptions. Deterministic per seed. (E14 k* = 1 everywhere; the
structural audit preserves seed-specific trajectories incl. the
pos-0 vs pos-2 REV1-pre offsets above.)

## 8. Protocol decisions to resolve before freezing (unresolved by
## design — flagged, NOT invented here)

- **D1 — verdict structure (A/B/C/D)**: the user's hypothesis set
  (A rapid structural reorganization, B re-expression of existing
  structure, C both, D unresolved) needs a registered mapping onto
  the existence booleans/raw evidence. Proposal for decision: map
  purely on existence booleans + endpoints (no numeric cutoffs):
  A-side topology delta empty AND A-side endpoint weights shifted =>
  B; topology delta non-empty and dominant over weight renormalization
  => A; both => C; ambiguous/none => D. Exact dominance definition
  needs approval (or verdict deferred to joint reading of raw tables —
  E13 precedent).
- **D2 — A-side/C-side boundaries**: channels 0-7 / 8-15 by pre
  membership, internal-recurrent separate — needs registration
  (mirror of the analyzer's pattern sets; B's own channels 4-7/8-11
  span both sides — flagged: B's SEQ phase drives 4-7 then 8-11, so
  B-associated structure is NOT cleanly one side; classification must
  handle B-side channels explicitly).
- **D3 — "major restructuring"**: undefined by design; E15 will
  report raw shares (e.g., fraction of live synapses with |Δw| >
  0, fraction created/pruned) with NO cutoff; any cutoff required for
  a verdict is a registered amendment, not an analysis default.
- **D4 — intra-interval claims**: restricted to emitted events +
  endpoints (§5); confirm no finer claims.
- **D5 — pre instant vs T0 anchor**: report BOTH 364,000 (T0,
  uniform) and the REV1-pre frames (368,000 for pos-2); primary
  interval = (REV1-pre, REV1-post].

## 9. Summary

E15 is feasible, analysis-only, and requires NO new instrumentation.
Sequence: resolve D1-D5 -> freeze docs/anima-e15-protocol.md ->
implement read-only instrument + tests -> execute -> classify ->
commit. Nothing executed beyond this audit.