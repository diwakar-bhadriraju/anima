# ANIMA E8 — Pre-registered Protocol (frozen before implementation)

**Status: PRE-REGISTERED.** Written before any E8 code, config, or
run. E8 is a **scale / no-D / cross-seed robustness probe** of a
structure ALREADY OBSERVED in E6-full (`runs/e6-20260914T204947Z`:
A-B 0.078, B-C 0.792, A-C 0.046, selectivity 0.116, 0 failures, on
the exact v3 geometry, seed 20260912). E8 does NOT claim to
discover the B-entanglement limitation; it asks whether that
limitation survives (1) fewer repetitions, (2) a clean S1-only
curriculum without S2/D, (3) across seeds. The E6 organism is
frozen wholesale; the ONLY change is the curriculum/repetition
schedule. Frozen values change ONLY via a logged, user-approved
amendment; no outcome-driven changes of any kind.

---

## 0. Scientific framing (frozen)

> **H8: The E6-full B-entanglement structure — A and C separated,
> B (⊆ A∪C, zero private positive evidence) entangled with its
> neighbors — persists at a registered minimal scale (60 reps),
> without the S2/D phase, and reproduces pairwise-verdict-wise
> across seeds 9001 and 424242.**

Either outcome is valid: reproduction strengthens the evidence for
a genuine 3-way overlapping-disambiguation limitation; divergence
reveals scale/history dependence. E8 does NOT justify any mechanism
change in either case. It is a robustness probe, not a mechanism
search.

## 1. Freeze (verbatim E6 organism)

Complete E6 organism as executed (commits `563bc10`, `56399f2`):
M1–M6, E6 rate balancing (φ EMA, β, both application sites), STDP
pairwise additive + LTD, adaptation, neuron dynamics, thresholds,
structural budgets, window order, telemetry v2, snapshots, replay,
viz, resource limits, RNG consumption order, seeds. No: absence
gating, new inhibition, anti-correlation, reward, M7, new
plasticity, homeostasis, tuning. `[e6]` remains enabled in every E8
config. Analyzer and telemetry untouched (anima-telemetry); the
only analyzer-facing change is the registered v3_analysis window
parameterization (already shipped in E7, §5 below).

## 2. Curriculum (frozen; exact v3/E6 geometry)

Channels, 24 inputs, 8-channel groups:

| Pattern | Channels | Private | Intersections |
|---|---|---|---|
| A | {0-7} | {0-3} | A∩B = {4-7} |
| B | {4-11} | **none (B ⊆ A ∪ C)** | B∩C = {8-11} |
| C | {8-15} | {12-15} | A∩C = ∅ |

Spike statistics identical to v2/v3/E6: 20 Hz independent Poisson
per channel, ±2 ms jitter, 500 ms presentations, 1500 ms off.
**Expected activity per pattern: 80 ± 9 spikes.** Duty cycle:
shared channels {4-11} fire in 2 of 3 patterns, exclusive channels
{0-3} ∪ {12-15} in 1 of 3 → **shared : exclusive ≈ 2 : 1**.

**E6 is ACTIVE in E8** (AMENDMENT A-3, 2026-09-16, user-approved —
replaces the static 2:1/2:3–4:3 arithmetic, which the frozen EMA
dynamics disproved; see appendix): φ is a τ = 2.5 s EMA over
bursty Poisson windows. The 2:1 presentation duty compresses at
equilibrium to ratios ≈ 1.1–1.3 (E6-full's own readout: exclusive
3.58 vs shared 4.22 Hz-equiv, ratio 1.18), and the end-state φ is
tail-dominated (last presentation dominates the ≈ 25-window decay
tail), so channel orders can even invert near the floor. β is
therefore TIME-VARYING across ≈ 0.7–1.5 during S1, always inside
the frozen [0.5, 2] clamp; there is NO static β_shared < 1 <
β_excl invariant. E8 is an E6-active condition (φ differs across
channels — the mechanism engages); it is never described as inert.
Engagement evidence comes from the φ readout structure, not from
static ratio claims.

### Repetition count (frozen, deliberate)

**60 repetitions per category** in S1 — between the earlier
low-repetition probes (40) and the 120-repetition v3/E6 curriculum.
Chosen at freeze time; never tuned after data.

### Stage structure (frozen)

- S0: 5000 ms silence.
- S1: A/B/C interleaved, **60 reps each** = 180 presentations ×
  2000 ms = 360,000 ms → [5000, 365000). Late-S1 = second half
  (event-defined; window args below).
- S3: A/B/C interleaved, 15 reps each = 45 presentations →
  [365000, 455000). Retention read only.
- **NO S2/D** (intentional, §13): E8 isolates the S1 3-way
  representation problem; E6-full's post-S2 rewiring effects
  (incl. the A-retention 2.414 observation) do not enter.

Timeline total: 455,000 ms. RF snapshot target: **365,000**
(S1 end — registered replacement for 715,000). Analyzer args
(registered): `185000 365000 0 0 365000 455000 365000`.

## 3. No separate "private-B control" (registered geometric proof)

A matched-statistics condition in which B has private evidence is
**not expressible** under the 24-channel / 8-channel balanced-overlap
construction:

- An 8-channel middle category B′ with |B′∩A| = 4 and |B′∩C| = 4
  (preserving the 4/4 overlap symmetry and 80±9 activity) must
  place 4 channels in A and 4 in C. Any such set B′ ⊆ A ∪ C has
  **zero private channels** — all its channels belong to A, C, or
  both. The unique balanced config is B = {4-11}.
- Granting B private channels requires either (a) unbalancing the
  overlaps (B∩A 2 vs B∩C 4 → unequal exposure/duty per shared
  channel), (b) breaking the chain (B∩C = ∅), or (c) changing
  channel counts/rates (activity or drive confounds). All three
  corrupt the causal attribution.

**The primary condition therefore contains its own internal
attribution structure (§4). No compromised control is created.**

## 4. Internal pairwise trichotomy (frozen; the key attribution)

With the existing cross_cosine measurement and the established
< 0.60 threshold (~v2 P3 bar):

| Outcome pattern | Prediction |
|---|---|
| A-B high ∧ B-C high ∧ A-C high | generic overlap difficulty (all reps mix) |
| A-B high ∧ B-C high ∧ A-C **low** | specific "B lacks private evidence" limitation |
| B-C high ∧ A-B low ∧ A-C low | B-entanglement (E6-full pattern: B → C coalition) |

Both B-structures (A-B or B-C entangled with A-C separated) count
as 3-way disambiguation failure; the E6-full reference pattern is
A-B low, B-C high, A-C low. Registered primary structural verdicts
per seed: the three booleans (A-B < 0.60, B-C < 0.60, A-C < 0.60)
plus the continuous values.

## 5. Endpoints (frozen; existing instruments verbatim)

- **M1 engagement**: established changes ≥ 1 by S1 end (v2 §11
  floor); zero ⇒ INCONCLUSIVE. Turnover reported. (A 60-rep
  regime with inadequate engagement is a scale/regime result per
  this paragraph if nonzero, or INCONCLUSIVE per the floor — never
  a parameter signal.)
- **M2 stability (P2 verbatim)**: 0 Failure events; S1 internal
  rates in the established [20, 250] Hz reading; budget invariant
  per window.
- **M3 separation**: late-S1 per-pair cross cosines A-B, B-C, A-C
  (cross_cosine, verbatim < 0.60 bar); median selectivity (S1,
  analyzer definition) reported — no a priori bar for the 3-way
  case beyond reporting (E6-full reference 0.116; the trichotomy
  is the endpoint, not selectivity).
- **M4 B-alignment (registered secondary, from existing outputs)**:
  `neighbor(B) = argmin(cosine(B,A), cosine(B,C))` — the coalition
  partner. Turns E6-full's qualitative "B∪C" into a reproducible
  quantity. Reported per seed.
- **M5 reported, no bar**: mean H + per-channel participation at
  snapshot 365,000; S3 retention (metrics.json; no D in the run);
  E6 φ readout (replay-recomputed) as engagement evidence.
- **L1 attribution** (E7's registered check) applied per pair on
  the primary run: structural vs drive-magnitude separation.

## 6. Cross-seed (frozen)

Seeds: **20260912** (primary), **9001**, **424242** — E8-left:
`configs/e8.toml`, `configs/e8-seed9001.toml`,
`configs/e8-seed424242.toml` (identical curriculum; only run.seed +
exp_id differ).

- **Gate**: 9001/424242 authorized iff the 20260912 run passes M2
  (P2) AND M1 (engagement).
- **Endpoint (categorical, registered — not replaceable by
  continuous comparison after data)**: the per-pair verdict
  boolean table (A-B, B-C, A-C < 0.60) must be **identical across
  all three seeds** for "verdict reproduction"; any mismatch is
  reported as seed-labile entanglement classification (a result,
  not a failure). Continuous statistics always reported alongside.

## 7. Verdict tree (frozen; every outcome valid)

Preconditions: Failure event ⇒ REGRESSION (stop, diagnose);
0 established by S1 end ⇒ INCONCLUSIVE; telemetry/determinism/
config-hash violation ⇒ INCONCLUSIVE; P2 fail ⇒ D (INCONCLUSIVE).

| Outcome | Definition | Meaning vs E6-full |
|---|---|---|
| **A — robust limitation** | 20260912 verdict table == {A-B < 0.60, B-C ≥ 0.60, A-C < 0.60} (or the mirrored A-B ≥ 0.60, B-C < 0.60 form) AND table reproduces in 9001/424242 | B-entanglement survives the 60-rep / no-D / cross-seed probe → strengthens the 3-way disambiguation limitation evidence |
| **B — scale/regime dependence** | verdict table differs from E6-full on 20260912 (any pair flips), or table differs across seeds | limitation is scale/history-dependent; valid result; no organism change |
| **C — generic overlap** | all three pairs ≥ 0.60 (no separated pair) | generic overlap failure interpretation |
| **D — INCONCLUSIVE** | §7 preconditions | gates failed |

B-alignment reported in every outcome. No mechanism is declared
missing solely from E8, and no mechanism is implemented regardless
of outcome.

## 8. Implementation scope and isolation (frozen)

- Configs only: `e8.toml`, `e8-seed9001.toml`, `e8-seed424242.toml`
  derived from `e6-full.toml` with pattern/stage blocks replaced
  (ids A/B/C — analyzer-selectivity-compatible — channel_ids
  exactly §2), S1 reps 60, S3 reps 15, no S2, [e6] enabled; all
  other sections byte-identical (freeze test).
- No anima-core, anima-telemetry, or analyzer-definition changes.
- v3_analysis invoked with the registered §2 window args.

## 9. Tests (registered, before any run)

1. Exact A/B/C channel sets + B ⊆ A∪C + intersection counts
   (A∩B = B∩C = 4, A∩C = 0).
2. Schedule: 60 reps, S0/S1/S3 only, no S2/D, timeline
   455,000 ms.
3. Activity: per-pattern 80 ± 9 bands; shared:exclusive duty 2:1.
4. E6 engagement (A-3): replay the frozen EMA on the exact E8
   streams; assert φ is structured (channel spread ≫ 0 — mechanism
   engaged), mechanism β values differ across channels and lie
   exactly within the frozen [0.5, 2] clamp. No static ratio or
   <1/>1 assertions.
5. Deterministic repeated generation (existing integration idiom,
   e8 layout).
6. Organism freeze: non-pattern/stage sections byte-identical to
   e6-full; [e6] values identical.
7. Analyzer compatibility: v3_analysis window args + no-S2 path
   (inherited from E7, re-verified).

## 10. Experiment order (frozen)

1. Freeze (this document, committed).
2. Implement configs + tests (release-gated).
3. Tests green.
4. Run 20260912. 5. Gate ⇒ 9001. 6. Gate ⇒ 424242.
7. Pairwise verdict table + B-alignment per seed. 8. Compare vs
   E6-full. 9. Interpret (scale/no-D/cross-seed axes).
Concurrent runs at nice 10. No sweeps, no alternate curricula, no
extra seeds "because it looks unstable".

## 11. Observability (unchanged)

Live viz, replayable telemetry, structural events, resource usage,
RF stats, deterministic seeds, commit + config-hash recording,
reproducible run dirs. Any observability failure stops the run and
is diagnosed.

## 12. Interpretation boundaries (frozen)

- Reproduction ⇒ "the E6-full structural limitation is robust
  across scale and seeds" — and NOTHING about a mechanism.
- Any divergence ⇒ "SCALE/REGIME DEPENDENCE" — valid, recorded,
  and an argument for replication studies (e.g., full-120 no-D),
  never for organism changes.
- D is excluded permanently from E8; it is not restored "for
  completeness".

---

## Appendix (filled at execution time)

- Config hashes (sha256 of raw file, recorded at creation before any
  run): e8 `7dfa8d48a260b00ec5b2b4c69c0b8a710f08ef85aa796e84dd732a3553d63ae6`;
  e8-seed9001 `9a0b6ac54d8e236a985b264f58d59826eaaf32a0ef9438608d4a3ee2a81a3723`;
  e8-seed424242 `102f2ee7386d1d452dc2b8a48f2d55b38f95680b77d6a357d5cf5a0a7854d70a`.
- E6-full reference (verbatim): A-B 0.078, B-C 0.792, A-C 0.046;
  selectivity 0.116; established 5,332; 0 failures
  (`runs/e6-20260914T204947Z`).
- Amendment A-3 (2026-09-16, user-approved): §2/§9.4 φ/β arithmetic
  corrected — static 2:1/2:3–4:3 claim superseded by the measured
  EMA dynamics (equilibrium compression ≈1.1–1.3, tail-dominated
  end state, β time-varying in [0.5, 2], no static invariant).
  Applies also to the E7 A-1 P-condition arithmetic (same dynamics;
  E7's recorded outcome A is unaffected — no endpoint depended on
  the β constants).
- Commit: `…` (recorded at run time).
---

## E8 execution record (2026-09-16)

Implementation `c4e89f4` (configs + tests; amendment A-3 committed
with it); 108 tests green, 0 warnings. Run dirs: primary
`runs/e8-20260916T160816Z`, cross-seed
`runs/e8-seed9001-20260916T161058Z`, `runs/e8-seed424242-20260916T161058Z`
— all curriculum-complete, telemetry intact, zero failures.

### Primary (seed 20260912, 60 reps, no S2/D)

| Endpoint | Value | vs E6-full |
|---|---|---|
| established | 2,358 ✓ (≥1) | 5,332 |
| P2 (failures; late-S1 rates) | 0; 13.0 / 88.5 Hz ✓ | 0; 11.9/98.0 |
| **A-B** | **0.073** ✓ | 0.078 |
| **B-C** | **0.873** ✗ | 0.792 |
| **A-C** | **0.032** ✓ | 0.046 |
| L1 attribution (AB/BC/AC) | 0.073 / 0.873 / 0.032 (all structural) | — |
| selectivity median | 0.205 (reported) | 0.116 |
| **B-alignment** | argmin(B-A, B-C) = **A** (0.073 < 0.873) | A |

The E6-full verdict table **{A-B ✓, B-C ✗, A-C ✓} reproduces
exactly** at 60 reps without S2/D, with continuous values close to
E6-full — the B-entanglement structure is robust across the scale
and no-D axes on the reference seed.

### Cross-seed (9001, 424242) — verdict-table reproduction: **NOT IDENTICAL**

| seed | A-B | B-C | A-C | verdict (AB, BC, AC < 0.60) | B-alignment | |ΔH| vs canon | |Δest|/est |
|---|---|---|---|---|---|---|---|---|
| 20260912 | 0.073 | 0.873 | 0.032 | (✓, ✗, ✓) | A | — | — |
| 9001 | 0.592 | 0.165 | 0.048 | **(✓, ✓, ✓)** | C | 0.477 | 0.204 |
| 424242 | 0.763 | 0.075 | 0.033 | **(✗, ✓, ✓)** | C | 0.210 | 0.109 |

L1-normalized values equal the raw values in every pair of every
seed — all separation is structural.

### Frozen verdict: **OUTCOME B — scale/regime dependence** (§7 tree)

Canonical-seed table matches E6-full; cross-seed tables differ ⇒
the "table differs across seeds" branch is taken. Recorded
findings:

1. **Robust core**: A-C (the two categories WITH private evidence)
   separates in every seed (≤ 0.048). 
2. **B is never independently represented**: in no seed is B
   strongly separated from BOTH neighbors (20260912: B-C 0.873;
   424242: A-B 0.763; 9001: A-B 0.592 — borderline pass, B pulled
   toward C). When B entangles, it coalesces with exactly ONE
   neighbor; the partner is seed-dependent (A in 20260912, C in
   424242, none-hard in 9001) — the middle category's absorption is
   stochastic, its failure to stand alone is universal.
3. E6-full's limitation reproduces on the reference seed (scale/
   no-D robust) but its expression is seed-labile — the per-seed
   placement of the coalition, not its existence-minimally, varies.
4. Interpretation boundary honored: this records the limitation's
   robustness/lability characterization. NO mechanism is implied,
   NO organism change follows; scale/regime dependence is the
   registered outcome.
