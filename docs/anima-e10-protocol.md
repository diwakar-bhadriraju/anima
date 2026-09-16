# ANIMA E10 — Pre-registered Protocol (frozen before implementation)

**Status: PRE-REGISTERED.** Written before any E10 code, config, or
run. E10 is an **experience-scale replication**: the E9 organism and
curriculum are frozen completely; the ONLY experimental variable is
repetitions 60 → 120. E10 is a scale/consolidation test, NOT a
mechanism test. E9's run directories (60 reps) are the untouchable
baseline. Frozen values change ONLY via a logged, user-approved
amendment; no outcome-driven changes.

---

## 0. Question (frozen wording)

> **H10: Does the temporal/contextual capability observed in E9
> (SEQ-B: phase-sequenced B; A-B 0.630/0.723/0.512; B-C
> ≤ 0.152; B-independence in 1/3 seeds) become robust with MORE
> experience — B independently separating from BOTH neighbors in
> 2-3 of 3 seeds — when repetitions double from 60 to 120?**

E10 is a two-point scale comparison only: **E9 = 60 reps, E10 =
120 reps. No third repetition point is permitted under E10.** A
third point requires a separate registration.

## 1. Freeze (verbatim E9 creature + curriculum)

Complete E9 configuration (`configs/e9-seq.toml`, commit `dbdc329`):
SEQ-B phases ({4-7} @ 40 Hz × [0,250) then {8-11} @ 40 Hz ×
[250,500)); A/C static 20 Hz × 500 ms; channel sets A {0-7}, B
{4-11}, C {8-15}; E6 rate balancing enabled; S0/S1/S3; no S2/D;
all timing parameters (500 ms presentations, 1500 ms off, ±2 ms
jitter); organism byte-identical to E6 (`e6-full.toml` sections);
telemetry, replay, analyzer, seeds 20260912 / 9001 / 424242.

**The only change: S1 reps per category 60 → 120.** No organism
changes, no parameter tuning, no phase-duration changes, no third
repetition, no mechanism work.

## 2. Consolidation hypothesis: explicitly two-sided (frozen,
### no directional prior)

E9's residual failure was B's **leading-group inheritance** (A-B
0.51–0.72 while B-C ≤ 0.152). Additional experience consolidates
network weights; it may either:

- **dissolve** the leading-group absorption (B-unique conjunctive
  statistics accumulate), or
- **strengthen** it (the leading group's character hardens).

Both directions are registered as live outcomes. NO directional
prior is assigned; post-hoc rationalization of either outcome is
prohibited.

## 3. Endpoints (frozen; E9 definitions verbatim)

- **M1 engagement**: established ≥ 1 by S1 end (v2 §11 floor);
  zero ⇒ INCONCLUSIVE.
- **M2 P2 verbatim**: 0 failures; S1 rates in the established
  [20, 250] Hz reading; budget invariant.
- **M3 pairwise verdicts**: late-S1 A-B, B-C, A-C < 0.60 with the
  L1 attribution check per pair; A-C always reported.
- **B-independence (categorical primary)**: A-B < 0.60 AND B-C <
  0.60 (L1-attributed), per seed.
- **B-alignment**: argmin(cosine(B,A), cosine(B,C)).
- **M5 reported, no bar**: selectivity median (S1), mean H +
  per-channel participation (snapshot 725,000), E6 φ readout,
  retention.
- **Per-seed scale curve (registered §3)**: for every pair and
  every seed, the two-point curve (E9_60, E10_120) plus the
  categorical verdict flips (which seeds change any pair's < 0.60
  verdict).

## 4. Seed rules and verdict (frozen)

Seeds 20260912 (primary), 9001, 424242 — all on the E10 config;
cross-seed authenticated by the registered P2 + engagement gate on
the primary (launched after it completes).

**B-independence interpretation rule (registered, strict):**

| Seeds with B-independence | Verdict |
|---|---|
| **3/3** | robust consolidation evidence — more experience stabilizes the temporal representation |
| **2/3** | stochastic at 120 — NOT "Outcome A"; record the failing seed exactly |
| **1/3** | still stochastic |
| **0/3** | no evidence that doubling experience resolves it |

Partial success is NEVER converted into a stronger verdict. A-C
and per-pair values are reported; the categorical rule is not
replaceable by continuous statistics after data.

## 5. Timeline (frozen; established convention)

- S0: 5000 ms silence.
- S1: A/B/C interleaved, **120 reps each** = 360 presentations ×
  2000 ms = 720,000 ms → **[5000, 725000)**. Late-S1 = second
  half → **[365000, 725000)**.
- S3: A/B/C interleaved, 15 reps each = 45 presentations →
  **[725000, 815000)**.
- **Snapshot target: 725,000**.
- **Actual duration: 813,500 ms (≈ 813.5 s)** — nominal **815 s**
  (established 1.5 s nominal/actual trailing-gap convention, as in
  v2/v3/E8/E9).
- Analyzer args (registered): `365000 725000 0 0 725000 815000 725000`.

## 6. Scale-commensurability audit (frozen, pre-data)

The E9/E10 comparison is valid only if the doubled experience does
not cross an implicit mechanism timescale. Registered audit
(asserted by freeze test):

| Mechanism | Time constant | E9 (60 reps, 365 s S1) | E10 (120 reps, 720 s S1) |
|---|---|---|---|
| E6 φ EMA | τ = 25 windows = 2.5 s | converged | converged |
| adaptation | τ = 200 ms | converged | converged |
| M3 structural window | 100 ms | unchanged | unchanged |

All ≪ both S1 durations — no implicit crossover; the freeze test
additionally asserts the E10 config is byte-identical to e9-seq
except reps = 120 and exp_id.

## 7. Configs (frozen at creation; hashes in appendix)

`configs/e10.toml` (exp_id e10, seed 20260912),
`configs/e10-seed9001.toml` (e10-seed9001, 9001),
`configs/e10-seed424242.toml` (e10-seed424242, 424242) — derived
from `e9-seq.toml`; the ONLY differences are S1 reps 60 → 120 and
the [run] identity fields.

## 8. Tests (registered, before data)

1. e10 == e9-seq except reps (120) + exp_id/seed — section-level
   byte comparison.
2. Phase definitions identical to e9-seq (channel sets, 250 ms
   boundaries, 40 Hz).
3. Timeline: S1 [5000, 725000), S3 [725000, 815000), actual
   duration 813,500 ms, snapshot 725,000, analyzer args sane.
4. Marginal/activity statistics identical to e9-seq (per-channel
   10 ± √10; totals 80 ± 9; M3 co-activity 1/5 vs 5/5).
5. Determinism: repeated generation (existing integration idiom).
6. Existing suites green (phase machinery unchanged).

## 9. Experiment order (frozen)

1. Freeze (this document). 2. Implement configs + tests.
3. Run 20260912. 4. Gate (M2 + M1) ⇒ 9001, 424242. 5. Scale
   curves + verdict tables + B-alignment + L1. 6. Compare vs E9
   (baseline runs, never rerun). 7. Verdict + interpretation +
   registry.

## 10. Interpretation boundaries (frozen)

- Outcome statements describe ONLY the two-point scale curve.
- "B-independent in N/3 seeds at 120" is the primary sentence.
- Any consolidation (per-pair drops toward 0.60) is reported as
  a scale effect on the E9 capability, NOT as a mechanism
  discovery and NOT as permission to tune or extend.
- If E10 matches E9's 1/3 or worsens, record "doubling experience
  did not stabilize the temporal representation at this scale".
- No phase-duration, rate, E6, or organism changes follow from
  any outcome.

---

## Appendix (filled at execution time)

- Config hashes (sha256 of raw file, recorded at creation before any
  run): e10 `0f2fb5775dcd1249b23d625cc429eecff39d9cbaa0f6398b2e876752ecd26db0`;
  e10-seed9001 `ad7438860b19b1549917d876dffaa59e87d52c210c04ec9a8ba6dd8f98898afa`;
  e10-seed424242 `3a16f327c6ed31400633b0690536ed46e7f35488c43aed03db5107bea1af02ca`.
- E9 baseline (verbatim): 20260912 (0.630/0.083/0.000), 9001
  (0.723/0.152/0.019), 424242 (0.512/0.079/0.000); B-independence
  F / F / T.
- Commit: `…` (recorded at run time).
---

## E10 execution record (2026-09-17)

Implementation `6700090` (configs + freeze/timeline tests); 116
tests green, 0 warnings. Run dirs: primary
`runs/e10-20260916T184902Z`, cross-seed
`runs/e10-seed9001-20260916T185308Z`,
`runs/e10-seed424242-20260916T185308Z` — all curriculum-complete,
telemetry intact, zero failures.

### Two-point scale curves (E9 = 60 reps baseline vs E10 = 120 reps)

| seed | pair | E9 (60) | E10 (120) | flip |
|---|---|---|---|---|
| 20260912 | A-B | 0.630 | **0.697** | . |
| 20260912 | B-C | 0.083 | 0.081 | . |
| 20260912 | A-C | 0.000 | 0.003 | . |
| 9001 | A-B | 0.723 | **0.760** | . |
| 9001 | B-C | 0.152 | **0.080** | . |
| 9001 | A-C | 0.019 | 0.000 | . |
| 424242 | A-B | 0.512 | **0.624** | **Y (away from bar)** |
| 424242 | B-C | 0.079 | 0.080 | . |
| 424242 | A-C | 0.000 | 0.000 | . |

**B-independence at 120: 0/3 seeds** (20260912 F, 9001 F,
424242 F). Engagement ≈ doubled with experience (4,276 / 3,517 /
4,006 vs 2,384 / 2,017 / 2,214); selectivity 0.662 / 0.802 /
0.902; P2 supported in all seeds (0 failures; rates 9.9/87.1 Hz
canonical); L1 attribution clean everywhere.

### Frozen verdict: **0/3 — no evidence that doubling experience
### resolves the limitation** (registered N/3 rule)

1. **The strengthening branch won**: every A-B value moved AWAY
   from the bar (+0.067, +0.037, +0.112) — additional experience
   CONSOLIDATES the leading-group (A-side) absorption rather than
   dissolving it. The only categorical flip across all nine
   comparisons was the loss of 424242's E9 B-independence
   (0.512 → 0.624).
2. **B-C is experience-robust**: the temporal cue's separation
   from the trailing side held and even improved (9001: 0.152 →
   0.080) — the de-coalescence from C is not the limiting axis.
3. Interpretation per §10: "doubling experience did not stabilize
   the temporal representation at this scale"; the two-point
   curve exhibits a hardening (not consolidation) trend on the
   A-side pair. Registered boundary: this is a scale effect on
   the E9 capability — NOT a mechanism discovery, NOT a license
   to tune; a third repetition point or any parameter change
   requires a separate registration.
4. A-C separability remains universal (≤ 0.003 vs E9's ≤ 0.019).
