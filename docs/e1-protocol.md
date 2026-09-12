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

- **Run**: `runs/e1-20260912T194732Z/` (headless-equivalent live run at max
  speed; telemetry byte-identical to headless per determinism check).
- **End**: `curriculum-complete`, no Failure events. 5,446,061 events.
- **Results vs thresholds**: assembly score 0.123 → 0.165 (late ≥ 2× early
  NOT met; selectivity median 0.43 > 0.5 met ⇒ per-rule **inconclusive**);
  novelty D-to-nearest 497 vs learned pairwise max 967 (≥ 2× NOT met ⇒
  **inconclusive**); retention A 103.7% / B 104.2% / C 126.2% (≥ 80% ⇒
  **supported**).
- Verdicts as printed in `report.md` are `auto-generated — review`.
  Phase 0 verifies the pipeline end-to-end; hypothesis interpretation is
  deferred to review.

## Roadmap (each built only when reached)

E2 STDP bounds A/B (U2) · E3 adaptation (U1) · E4 birth triggers (U3) ·
E5 learning gates (U4) · E6 reward modulation (U7) · E7 memory (U5) ·
E8 output decoder · E9 retina encoder · E10 cochlea encoder.
