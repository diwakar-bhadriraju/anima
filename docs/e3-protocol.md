# E3 Protocol — Spike-Frequency Adaptation / Homeostasis (U1)

**Status**: EXECUTED 2026-09-13. Verdict: **adaptation stabilization
supported; assembly separation inconclusive** (see Execution Record).
Pre-registered sections below were frozen before execution; amendments
A2 (calibration) logged under Scope guards.

## Motivation

E1 (additive STDP) formed stable-but-overlapping assemblies; the weight
distribution saturated (73% at ceiling) and froze adaptation. E2 arm B
(multiplicative bounds) collapsed into a population seizure because
nothing counteracts runaway recurrent excitation at low weights. Both
failures point at the same missing mechanism: **a slow negative feedback
on each neuron's own firing** — spike-frequency adaptation / homeostasis
(U1 in docs/unknowns-registry.md).

## Hypothesis

Adding an adaptation current (or equivalent homeostatic gain control)
to internal LIF neurons stabilizes the organism under strong plasticity
and enables pattern-selective assemblies to separate: across arms with
adaptation, cross-pattern response cosine DECREASES over S1 while
within-pattern stability stays high, without triggering runaway
failures and without losing retention.

## Design (A/B, only the adaptation parameter varies)

Organism, curriculum, seed, thresholds, pattern structure: identical to
E1/E2 (`configs/e1.toml` base). Add to `[organism]`:

- `adaptation_tau_ms` (decay of the adaptation current)
- `adaptation_gain` (current injected per spike)

- **Arm A (control)**: adaptation_gain = 0 (behaviorally identical to E1;
  expected to re-confirm the E1/E2a result, including its saturation).
- **Arm B**: adaptation_gain tuned on a short probe run so that peak
  internal rates during S1 stay in the 100–200 Hz band (calibration runs
  are logged as amendments, not tuning-to-verdict).

Arms share seed 20260912. Arm A may reuse the E1/E2a record instead of
re-running (identical config modulo adaptation constants = gain 0).

## Metrics

Same machinery as E1/E2, plus one new instrumentation metric derived
from existing snapshots (no new wire events):

- **Adaptation stability**: per-neuron rate variance across S1 bursts
  (variance should FALL vs arm A).
- Assembly score, selectivity median, novelty ratio, retention,
  saturation fraction — all as pre-registered in E1/E2.

## Verdict thresholds (pre-registered)

- **Stabilization supported (E3's own gate)**: zero runaway-activity
  failures AND internal mean rate during S1 bursts within [20, 250] Hz
  for arm B; else inconclusive.
- **Assembly separation**: late-S1 cross-pattern cosine < 0.7 (E1 sat at
  0.70–0.81) AND median selectivity > 0.5; one ⇒ weakly supported.
- **Retention preserved**: S3 ≥ 80% of late-S1 for A/B/C.

## Scope guards

- Adaptation is a *neuron* mechanism; plasticity rule stays E1 additive
  (isolate homeostasis first — bound variants re-enter after U1).
- No threshold/curriculum/morphology changes.
- Deterministic: same seed ⇒ identical chunk telemetry (sha256 over
  `telemetry/` dir, per verification test).
- Telemetry: T-CHUNK storage; report regeneration streams.

## Open items before execution

1. Adaptation current implementation in `anima-core/src/network.rs`
   (per-neuron adaptation variable + decay; wired from config).
2. Unit tests: adaptation decays with tau; injected per spike; gain 0 is
   exactly E1 behavior (regression guard).
3. Probe grid for arm B gain (documented as amendments).

---

## Execution Record (2026-09-13)

### Amendments

- **A2 (calibration, logged pre-execution)**: arm B gain fixed at
  **0.05** (`adaptation_tau_ms = 200` from protocol default). Calibration
  sweep {0.05, 0.10, 0.20, 0.30} on a 10-rep probe: burst-peak internal
  EMA ~148 Hz at 0.05, inside the pre-registered [100, 200] Hz band;
  0.05 chosen as minimal sufficient intervention. (Probe at 0.15 first:
  peak 146 Hz; sweep re-confirmed.)
- **A3 (arm A reuse)**: arm A (gain = 0) reuses the E2a record
  (`runs/e2a-20260913T082247Z`), justified by the unit-tested identity:
  gain 0 leaves trajectories bit-identical
  (`adaptation_gain_zero_reproduces_bare_lif`).

### Artifacts

- Arm B run: `runs/e3b-20260913T135736Z` (5,101,983 events, 21 chunks,
  24 MB chunk telemetry; sim wall ~8 s; report regen 44 s).
- Analysis helpers: `crates/anima-exp/examples/{read_rates,cross_cosine}.rs`.

### Verdicts (auto-generated — review)

1. **Adaptation stabilization: SUPPORTED.** Zero runaway failures in arm
   B (E1-threshold detector never fired); S1 internal rates bounded
   (snapshot EMA max 194.6 Hz, well under the 250 Hz ceiling; arm B
   mean-rate variance 296.5 vs arm A 317.3 over S1 snapshots).
2. **Assembly separation: INCONCLUSIVE.** Late-S1 cross-pattern cosine
   arm B: A-B 0.753, A-C 0.603, B-C 0.670 (mean 0.675) vs arm A (E1):
   0.746, 0.604, 0.667 (mean 0.673) — indistinguishable. Within-pattern:
   0.846 vs 0.872. Selectivity median 0.455 (B) vs 0.428 (A) — below the
   0.5 bar in both arms. Neither pre-registered branch crossed.
3. **Retention preserved: SUPPORTED (trivially).** S3/late-S1 =
   1.127/1.101/1.385 (A/B/C), ≥ 0.80 in all; baseline firing keeps
   responses strong in both arms — retention is not diagnostic here.

### Interpretation

Adaptation does what U1 predicts mechanistically — bounds rates, no
instability — but does NOT by itself separate assemblies. The binding
problem E1 identified (cross-cosine ~0.67 driven by shared output-layer
activity, not overlapping assemblies) persists identically with
adaptation ON. This upgrades the working hypothesis: separation is
limited by circuit-level integration (all patterns drive the same
recurrent pool; no inhibition, U1-inhibition branch, or gating, U4),
not by rate instability. E5 (learning gates) or the lateral-inhibition
variant of U1 is the natural next intervention.

### Follow-ups

- Consider the lateral-inhibition candidate from U1 in a future E3b
  variant if assembly separation is prioritized before gating.
- Retention metric should be conditioned on matched response magnitude
  to stay diagnostic under adaptation.
