# ANIMA E9 — Pre-registered Protocol (frozen before implementation)

**Status: PRE-REGISTERED.** Written before any E9 code, config, or
run. E9 asks whether a category with **no private instantaneous
positive evidence** (the v3/E8 middle category B, B ⊆ A∪C) can
become independently representable when the CURRICULUM supplies
discriminative **temporal/contextual** structure — without any
organism change. E8's static result (OUTCOME B: A-C universally
separate; B never stands alone; coalition partner seed-labile) is
the baseline. E9 does NOT test a temporal mechanism; it tests
whether the frozen E6 organism can exploit richer curriculum
information. Frozen values change ONLY via a logged, user-approved
amendment; no outcome-driven changes.

---

## 0. Scientific question (frozen wording)

> **H9: Under the v3 spatial geometry (B ⊆ A∪C, B has zero private
> channels), can B acquire an independently useful representation
> if the discriminative information for B exists as TEMPORAL
> structure — a within-presentation phase separation of its two
> shared channel groups — rather than as private instantaneous
> evidence?**

Outcome A (B separates from BOTH neighbors) is a curriculum-level
capability result, NOT a mechanism discovery. Outcome B (still
absorbed) means the frozen organism could not exploit the
registered temporal distinction. Both are valid.

## 1. Freeze (verbatim E6 organism)

Complete E6 organism as executed (commits `563bc10`, `56399f2`):
M1–M6, E6 rate balancing, STDP pairwise additive + LTD, adaptation,
neuron dynamics, thresholds, budgets, structural window order,
telemetry v2, snapshots, replay, viz, resources, RNG semantics,
seeds 20260912 / 9001 / 424242, [e6] enabled. NO: timing-rule
changes, eligibility traces, temporal credit assignment, new
recurrent dynamics, new adaptation, reward, absence gating, M7,
parameter sweeps.

## 2. Derivation of the minimal temporal construction (registered)

Constraints derived from the frozen organisms' machinery:

1. **Readout limit**: all E8/E9 endpoints read per-presentation
   spike counts. Within-presentation timing is INVISIBLE to the
   readouts unless it reshapes, through plasticity, WHICH neurons
   respond per pattern. The temporal cue must therefore alter
   weights during training — it must be resolvable by STDP.
2. **STDP scale**: pairwise traces, τ = 20 ms. Spike pairs ≥ ~5τ
   apart contribute nothing (LTP and LTD both). Cross-group spike
   separation must exceed ~100 ms for the groups to be
   dissociated in weight space.
3. **M3 scale**: co-activity windows are 100 ms wide (structural
   window). Events in the same window count as co-active. A
   250 ms separation puts the two groups in the same window only
   at the phase boundary (1 of 5 windows vs 5 of 5 when static).
4. **E6 φ/β scale**: φ is a 100-ms-window EMA. If a group's
   marginal rate is preserved (spikes per 500 ms identical),
   per-window event statistics are matched ⇒ φ and β are matched
   vs the static categories (A-3 arithmetic: no static-ratio
   claims, but the MATCHING holds — see §4 audit).

The minimal construction satisfying all four:

- A static: {0-7} @ 20 Hz × 500 ms (as E8).
- C static: {8-15} @ 20 Hz × 500 ms (as E8).
- **B sequential (SEQ-B, primary)**: {4-7} @ 40 Hz × [0, 250) ms,
  then {8-11} @ 40 Hz × [250, 500) ms.

Per-channel marginal exposure: 40 Hz × 250 ms = 10 spikes = A/C's
20 Hz × 500 ms → **totals matched (80 ± 9 per presentation), per-
channel marginals matched, φ/β matched**. The temporal cue: B's
two shared groups never co-fire within STDP τ (250 ms apart) and
co-occur in only 1 of 5 M3 windows — vs A's {0-3}/{4-7} and C's
{8-11}/{12-15} pairs, co-active 5 of 5 windows.
**Instantaneous property preserved**: at every instant B's active
set ⊆ A's or C's active sets (or the shared-only union); B has no
private channel at any instant (registered: the instantaneous
active sets are {4-7} ⊂ A's set and {8-11} ⊂ C's set).

The 40 Hz within-epoch rate is the temporal implementation; the
marginal (presentation-level) rate is 20 Hz per channel, matched
(registered; the release's "no firing-rate change" prohibition
refers to marginal identity, which is preserved).

## 3. Conditions (frozen)

| Condition | Role | Construction |
|---|---|---|
| **SEQ-B** | E9 primary | B = {4-7}@40 Hz [0,250) → {8-11}@40 Hz [250,500); A/C static as E8 |
| **REV-B** | registered control (order) | B = {8-11}@40 Hz [0,250) → {4-7}@40 Hz [250,500); A/C static |
| **E8-static** | baseline (existing runs, NOT rerun) | B static {4-11}@20 Hz × 500 ms (E8 runs: `runs/e8-*`) |

Causal structure: SEQ-vs-E8-static = "temporal separation vs
co-activity, marginals matched"; SEQ-vs-REV = "direction of the
order" (same content, reversed) — isolates whether ORDER per se
matters or only the separation (if SEQ and REV give the same
B-outcome, separation is the operative variable; if B-alignment
flips between them, direction matters). No other control is needed
for the registered question; residual differences are registered in
§4.

Curriculum around the conditions: S0 5000 ms silence; S1
interleaved, 60 reps each (180 presentations, 360,000 ms →
[5000, 365000)); S3 15 reps each → [365000, 455000). No S2/D
(E8 §13 precedent: S1-focused). Timeline 453.5 s actual / 455 s
nominal (E8 convention). Analyzer args:
`185000 365000 0 0 365000 455000 365000`. Pattern ids A/B/C
(analyzer-selectivity path).

## 4. Activity and E6 audit (frozen math, before data)

| Quantity | A | C | SEQ-B | REV-B |
|---|---|---|---|---|
| channels | 8 | 8 | 8 | 8 |
| spikes/channel/pres | 10 ± √10 | 10 ± √10 | 10 ± √10 | 10 ± √10 |
| total spikes/pres | 80 ± 9 | 80 ± 9 | 80 ± 9 | 80 ± 9 |
| per-window event mean (E[c/100]) | ≈ 0.02 | ≈ 0.02 | ≈ 0.02* | ≈ 0.02* |
| **φ/β vs baseline** | matched | matched | **matched** | **matched** |
| shared-group co-active windows (of 5) | 5 (0-3/4-7) | 5 (8-11/12-15) | **1** ({4-7}/{8-11}) | **1** |

*B channels pulse at 40 Hz in 250 ms spans: per 100-ms window the
expected tick-exceedance count equals the static channels'
(≈ 2 events/window; Poisson identity at matched spike-per-window
expectation). Residual differences (registered, quantified):
(a) within-epoch instantaneous rate 40 vs 20 Hz (inherent to any
temporal structure; MARGINAL equal); (b) B's M3 co-activity for
its two groups is 1/5 windows vs A/C's 5/5 — this IS the
temporal cue; (c) φ transients at phase boundaries (2 windows per
presentation, bounded by the EMA's τ = 2.5 s at ≈ 4% duty —
negligible, registered).

## 5. Endpoints (frozen; E8 definitions verbatim)

Per condition (SEQ-B primary; REV-B control):

- **M1 engagement**: established ≥ 1 by S1 end (v2 §11 floor);
  zero ⇒ INCONCLUSIVE.
- **M2 P2 verbatim**: 0 failures; S1 internal rates in the
  established [20, 250] Hz reading; budget invariant.
- **M3 pairwise verdicts (E8 structure)**: late-S1 A-B, B-C, A-C
  cross-cosines with the < 0.60 bar, plus the L1 attribution
  (E7/E8 registered check) per pair.
- **B-independence (the categorical primary endpoint)**: B is
  independently represented ⇔ A-B < 0.60 AND B-C < 0.60 (with L1
  attribution on both). A-C reported.
- **B-alignment**: argmin(cosine(B,A), cosine(B,C)) (reported).
- **M5 reported, no bar**: selectivity median (S1), mean H +
  per-channel participation (snapshot 365,000), E6 φ readout,
  retention.

## 6. Cross-seed (frozen)

Seeds 20260912 (primary), 9001, 424242 on **SEQ-B only**, gated on
the primary's M2 + M1. Endpoint: the B-independence verdict
(booleans A-B < 0.60, B-C < 0.60) must reproduce in all three
seeds, categorical (per E8 §6 discipline). Continuous values
reported. REV-B is a single-seed (20260912) attribution control —
no cross-seed registered.

## 7. Verdict tree (frozen)

Preconditions: Failure ⇒ REGRESSION (stop, diagnose); 0 established
⇒ INCONCLUSIVE; P2 fail / telemetry / determinism ⇒ D.

| Outcome | Definition | Meaning |
|---|---|---|
| **A** | B-independent (both pairs < 0.60, L1-attributed) on 20260912 AND the B-independence verdict reproduces in 9001/424242 | the frozen organism CAN exploit temporal/contextual structure to represent the evidence-less category — curriculum-level capability, NO mechanism claim |
| **B** | A-C < 0.60 but B still ≥ 0.60 with at least one neighbor | temporal/contextual info (as registered) did NOT overcome the limitation |
| **C** | generic collapse (no separated pair, or A-C ≥ 0.60) | temporal curriculum damages the structure |
| **D** | gates | INCONCLUSIVE |

Registered sub-findings (interpreted in every outcome):
(a) SEQ vs REV difference — order-direction sensitivity;
(b) SEQ vs E8-static — separation's effect; (c) B-alignment
partner per seed.

## 8. Interpretation boundaries (frozen)

- Outcome A records: "the frozen E6 organism can form an
  independently distinguishable representation for a category
  lacking private instantaneous positive evidence when
  discriminative temporal/contextual information is present." It
  does NOT claim a temporal mechanism was discovered — the
  mechanism (whatever STDP/M3 interaction produced the change)
  would be a separate future registration.
- Outcome B records: "the frozen organism did not exploit the
  registered temporal/contextual distinction sufficiently to
  produce independent B representation." No temporal mechanism is
  implemented.
- Any seed dependence is recorded exactly. Baseline = E8 runs
  (never modified); differences are curriculum/regime effects.

## 9. Implementation scope (frozen, after release)

- Environment layer only: optional within-presentation phase
  support in `PatternSpec` (`[[pattern.phases]]`: `from_ms`,
  `to_ms`, `channel_ids`, `rate_hz`; absent ⇒ current single-phase
  behavior — v2/v3/E6/E7/E8 configs unchanged, byte-identical).
  Trains generated per phase per channel with the existing
  per-(pattern, rep, channel, schedule-index) seed derivation
  (deterministic; phase membership added to the stream only).
- Configs: `e9-seq.toml`, `e9-rev.toml`, `e9-seq-seed9001.toml`,
  `e9-seq-seed424242.toml` — from e6-full, organism sections
  byte-identical, [e6] on, pattern ids A/B/C.
- No anima-core / anima-telemetry / analyzer-definition changes;
  v3_analysis args as §3. Hashes recorded at creation.

## 10. Tests (registered, before data)

1. Phase semantics: SEQ/REV channel-vs-time maps exact (250 ms
   boundaries, 40 Hz epochs, marginal 20 Hz math).
2. Per-channel marginal exposure audit: 10 ± √10 per channel per
   presentation in both conditions; totals 80 ± 9; per-window
   event means matched vs a static reference (≈ 2/window).
3. M3 co-activity accounting: B's groups co-occur in exactly 1 of
   5 windows (grid-aligned), A/C pairs 5 of 5.
4. E6 matching: feed the frozen EMA with both conditions' streams
   — per-channel φ and β equal to the E8-static streams' within a
   registered band (5%).
5. Determinism: repeated generation + short-run telemetry identity
   (existing integration idiom, e9 layout).
6. Organism freeze: sections byte-identical to e6-full; freeze
   test; existing suites green.

## 11. Experiment order (frozen)

1. Freeze (this document). 2. Implement + tests (release-gated).
3. REV-B (control) @ 20260912. 4. SEQ-B (primary) @ 20260912.
5. Gate (M2 + M1 on primary) ⇒ SEQ-B @ 9001, 424242. 6. Analysis
   (verdict table, B-alignment, L1, comparison vs E8). 7. Verdict
   + interpretation + registry.

No tuning; no alternative temporal constructions after data.

## 12. Observability (unchanged)

Viz, replay, telemetry, structural events, resources, RF stats,
deterministic seeds, config-hash recording, reproducible run dirs.
Failures stop the run and are diagnosed.

---

## Appendix (filled at execution time)

- Config hashes: e9-seq `…`; e9-rev `…`; e9-seq-seed9001 `…`;
  e9-seq-seed424242 `…` (recorded at creation, before runs).
- E8 baseline (verbatim): 20260912 (0.073/0.873/0.032), 9001
  (0.592/0.165/0.048), 424242 (0.763/0.075/0.033); B-alignment
  A / C / C.
- Commit: `…` (recorded at run time).