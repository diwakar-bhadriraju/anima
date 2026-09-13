# E3 Protocol — Spike-Frequency Adaptation / Homeostasis (U1)

**Status**: DRAFT — pre-registration to be finalized and frozen before
execution. Storage v2 (T-CHUNK) is a prerequisite and is now in place.

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
