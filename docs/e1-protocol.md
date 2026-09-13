# E1 Protocol — STDP-Driven Assembly Formation + Novelty Response

**Status**: pre-registered before execution. Thresholds below are the ONLY
rules the auto-report uses; verdicts are labeled `auto-generated — review`
and humans own final interpretation.

## Hypothesis

Pairwise additive STDP (D7) on an LIF organism (24 input / 40 internal /
12 output), driven by repeated patterned input, forms pattern-selective
synaptic assemblies; a never-seen pattern D produces a measurable novelty
response (instrumentation-computed, U6a — NOT a claim of internal
representation); S3 re-testing measures retention of the S1 assemblies.

## Organism

| item | value |
|---|---|
| Input channels | 24, groups A/B/C × 8 (pure spike sources, D8) |
| Internal (LIF) | 40, sparse seeded wiring p=0.038, w init ~[0.1, 0.3] |
| Output (LIF) | 12 (role tag — ordinary plastic neurons) |
| Synaptic amplitude | 52.0 (current per spike = amplitude × w; see amendment A1) |
| STDP | pairwise trace, tau±=20 ms, A+=0.005, A-=0.0053, w∈[0,1], decay 1e-6/tick (D7) |
| Birth trigger | none (E4 will A/B/C/D triggers; machinery live, U3) |
| Learning gate | always-on (U4a; E5 gates) |
| Caps | 200 neurons / 2000 synapses / 4 births-per-window |
| Seed | fixed in `configs/e1.toml`; deterministic Poisson per (pattern, rep, channel) |

## Curriculum

| stage | content |
|---|---|
| S0 | 5 s silence baseline probe (`StimulusPresented{pattern:"silence"}`) |
| S1 | 360 presentations: 120 × {A, B, C} interleaved; 500 ms burst @ 20 Hz on the group's 8 channels; 1.5 s off |
| S2 | 30 presentations of D = A+C co-activation (novel, never in S1), same timing |
| S3 | 45 re-test presentations: 15 × {A, B, C} interleaved |

## Pre-registered verdict thresholds

- **Assembly formation supported**: assembly score (late-S1 within−cross
  pattern cosine delta) ≥ 2× early-S1 score AND median selectivity of
  internal neurons > 0.5. Exactly one ⇒ **weakly supported**; else
  **inconclusive**.
- **Novelty discrimination supported**: D's response-profile distance to the
  nearest learned pattern ≥ 2× max A↔B↔C pairwise distance; else
  **inconclusive**. Novelty is instrumentation-computed (U6a) — the report
  must not claim the organism "represents" novelty.
- **Retention supported**: S3 response ≥ 80% of late-S1 for all of A/B/C;
  50–80% ⇒ **weakly supported**; < 50% ⇒ **inconsistent**.

## Pre-registered fallback

If default STDP parameters produce no weight movement (all-dead or
all-saturated), one parameter sweep A± ∈ {0.002, 0.005, 0.01} remains within
E1's scope, logged as an amendment. Outcome is data either way — the
pipeline, not the hypothesis, is what Phase 0 verifies.

## Amendment A1 (pre-execution, logged)

Initial estimates (p=0.15, amplitude 3.0) produced no internal activity:
steady-state synaptic current ≈ 0.07·v_th, far under threshold. Parameter
probes showed the failure mode inverts sharply: p ≥ 0.045 at usable
amplitudes drives recurrent self-sustainment (activity persists through
off-periods → runaway-activity Failure). Chosen operating point from the
probe grid: **p = 0.038, amplitude = 52.0** — bursts elicit internal
responses, silence stays quiet, curriculum completes. Logged before the E1
run; tuning targeted observability of the mechanism, not the verdict.

## Execution record

- **Run**: `runs/e1-20260912T215824Z/` (the canonical record; three
  byte-identical runs of this seed existed — determinism confirmed by
  sha256 — duplicates and partial probe runs deleted 2026-09-13).
- **End**: `curriculum-complete`, no Failure events. 5,446,061 events.
- **Results vs thresholds**: assembly score 0.123 → 0.165 (late ≥ 2× early
  NOT met; selectivity median 0.43 > 0.5 met ⇒ per-rule **inconclusive**);
  novelty D-to-nearest 497 vs learned pairwise max 967 (≥ 2× NOT met ⇒
  **inconclusive**); retention A 103.7% / B 104.2% / C 126.2% (≥ 80% ⇒
  **supported**).
- Verdicts as printed in `report.md` are `auto-generated — review`.
  Phase 0 verifies the pipeline end-to-end; hypothesis interpretation is
  deferred to review.

## Post-execution evidence pass (2026-09-13, snapshot trajectory)

Recomputed from the 873-frame snapshot stream (100 ms-state every 1 s) and
telemetry, beyond the two summary numbers in the report:

- **Weights moved strongly, then stabilized**: mean w 0.215 → 0.79 by
  late-S1 (init std 0.05 → 0.36); ~50% of synapses above 0.9 within 60 s
  of S1 onset. Weight structure is *not* flat — the "all-dead" failure
  mode is excluded.
- **Saturation is the real story**: at end-of-run 73% of synapses sit at
  w > 0.9 (ceiling), 18% below 0.05. Once saturated, Δw = 0, so the
  additive rule stops adapting — pattern responses freeze, which explains
  the flat 0.123 → 0.165 assembly score (it plateaued, it wasn't absent).
  Median top-1 weight share per neuron = 0.57: moderately dominant, not
  winner-take-all.
- **Assemblies are stable, just not separated**: within-pattern response
  profile correlation across S1 halves is 0.97–0.99 for A/B/C (the same
  neurons respond consistently). But cross-pattern profile cosine stays
  high (A·B 0.79→0.73, A·C 0.75→0.70, B·C 0.81→0.81 early→late S1): the
  three assemblies overlap heavily instead of specializing. This is the
  mechanism behind the failed 2× threshold — a *binding/specificity*
  problem, not a *plasticity* problem.
- **Retention confirmed at snapshot level**: mean internal rate during a
  pattern's burst, S3 vs late-S1: A 0.92, B 0.92, C 0.77. The analyzer's
  presentation-level ratios (1.04/1.04/1.26) run higher because they use
  a different denominator; both agree memory persists.
- **Novelty D** produced a distinct profile (D distance 497) but under the
  2× bar partly because the learned patterns themselves are so similar to
  each other — the pairwise-max baseline (967) is inflated by the same
  overlap. A run with better-separated assemblies should improve both
  verdicts at once.

**Interpretation for E2 design**: the binding problem dominates. Pairwise
additive STDP with a hard w ceiling saturates quickly, and saturated
synapses cannot participate in competition, so co-active patterns pool
onto the same internal neurons. E2's multiplicative bound (Δw ∝ w(1−w))
is the pre-registered, mechanistically-targeted next step.

## Roadmap (each built only when reached)

E2 STDP bounds A/B (U2) · E3 adaptation (U1) · E4 birth triggers (U3) ·
E5 learning gates (U4) · E6 reward modulation (U7) · E7 memory (U5) ·
E8 output decoder · E9 retina encoder · E10 cochlea encoder.
