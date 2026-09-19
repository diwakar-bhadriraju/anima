# DRAFT v2 — regime-selection capacity experiment (E24 candidate)

Status: REVISED DRAFT after X3 review (this file supersedes the v1
draft of the same name). NOT executed, NOT preregistered, NOT
frozen. No E-number assigned. V2.1 not modified. No new
exploratory runs were executed for this revision (the two open
methodological questions were answerable from X3 artifacts; see
§1.4).

Formal question: **do endogenous dynamics retain information
about prior drive composition after drive offset** (memory, not
reflex)?

---

## 1. X3 review findings that drive the redesign

### 1.1 The 0.85 top-u result and its six marginal cells

The six persistent cells with top-u ≤ 2.0 (123456/AC 1.34,
20260912/A@0.003125 1.17, 20260912/AC 1.57, 424242/A 1.72,
424242/C 1.77, 9001/A 1.50) are NOT ambiguous outcomes — they are
resolved perfectly by silence spiking: all 17 silent cells have
exactly 0 endogenous spikes in the first 5 s of silence; all 22
persistent cells fire ≥ 1 (min = 1, max = 9,849). Top-3 u sum also
fails at the boundary (silent range 1.48–2.73 overlaps marginal
range 2.60–3.96). Conclusion: top-u/top3 are *mechanism*
variables with a soft boundary; the *phenotype* (endogenous
firing) has a hard boundary. Endpoint choice follows (§2).

### 1.2 Transition-band seed/winner dependence

At β=0.0046875, every seed shows within-seed spread of drive-end
top-u across the three drives (ratios 9.7×, 7.7×, 7.4×, 2.9×,
2.2×), but seeds differ ~10× in absolute margin (777 all-low,
123456 all-high). Composition effects and seed-dependent
threshold crossing are separable ONLY by within-seed pairing:
each seed experiences every arm; the statistical unit is the
seed; the test is on paired within-seed differences. Cross-seed
comparison of raw margins is confounded by construction and is
excluded from the analysis plan.

### 1.3 The five mid-drive P2 aborts (β=0.00625)

All five aborts are runaway-detector crossings DURING drive
(mean internal rate > 50 Hz for 5 s), none during silence. Under
the u-margin hypothesis these are not instrument failures but the
upper end of the same variable: a winner whose margin blows
through saturation during drive = a third regime (divergence).
Decision for E24: (a) the detector is NOT recalibrated (rejecting
the E2b-style amendment here — mid-paradigm instrument change
would confound the record); (b) aborts are pre-registered as a
distinct outcome class ranked ABOVE all silence counts (see §2),
retained and analyzed, never retried or censored; (c) E24 is
placed at the band (β=0.0046875) where X3 saw 0/15 aborts, so
divergence should be rare but informative when it occurs.

### 1.4 Methodological questions resolved without new runs

- Blocked-stage support: `order = "blocked"` presents all reps of
  the first pattern, then all of the second (env.rs:340–346) —
  the A→C / C→A arms need no harness change.
- Silence-window onset: every persistent X3/X2 run fires its
  first endogenous spike within the first 5 s (v21contrast:
  silence+0 ms in all inspected runs) — a 10 s readout window is
  safe; no onset-alignment ambiguity remains (E18's +1 delivery
  tick affects marker timestamps only, and the readout is
  relative to the last presentation end in telemetry, consistent
  across arms).

---

## 2. Primary endpoint (decision, with argument)

**Primary endpoint: a two-tier, information-preserving ordinal
statistic M per run, constructed as:**

- **Tier 1 (divergence):** run aborted on runaway-activity during
  drive ⇒ M := +∞ class (ranked above every finite value).
- **Tier 2 (retention magnitude):** otherwise M := log10(1 + S10),
  where S10 = number of endogenous (non-input) spikes in
  [silence onset, silence onset + 10 s).

Secondary endpoints (descriptive/mechanistic, never primary):
regime class (silent / sparse-core / pacemaker, v21phase
thresholds frozen at protocol commit), drive-end top-u (neuron id
+ magnitude) and top-3 sum, active-neuron set, full silence
trajectory (2 s bins), output-neuron participation (E19-relevant).

**Why this is the most informative endpoint available from
existing telemetry:**

1. **It is the phenotype, not the mechanism.** The formal question
   is whether *endogenous dynamics* retain drive information.
   S10 observes endogenous dynamics directly. Top-u is the
   hypothesized INTERNAL CAUSE of those dynamics; making it
   primary would conflate observation with interpretation
   (violates the observation/interpretation separation and
   biases the test toward the u-margin hypothesis).
2. **It has a hard boundary where top-u is soft.** X3: silent vs
   any-persistence separates 40/40 on first-5-s spiking vs 34/40
   on top-u>2 (§1.1). The six marginal cells are exactly the
   cells where a top-u primary endpoint would misclassify.
3. **It discards no magnitude information.** S10 spans 4 decades
   (1 → 9,849) within "persistent"; the discrete regime class
   collapses 1 and 9,849 into one label. log10 keeps the ordering
   that the u-margin hypothesis predicts is monotone (P1) while
   remaining robust to the heavy tail.
4. **The two tiers do not discard the abort information.** A pure
   spike-count endpoint would have to censor or drop aborts; the
   ordinal construction keeps them as the top rank, which is
   exactly what the hypothesis says they are (extreme margin).
5. **It needs no new instrumentation** — computed from committed
   telemetry kinds (spike rows + StimulusPresented markers),
   deterministic, arm-symmetric.

Rejected alternatives: discrete regime class (coarse, discards 4
decades, threshold-frozen classifier adds arbitrary cutoffs);
continuous top-u (soft boundary, mechanism variable, 0.85);
top-3 sum (overlaps silent range).

---

## 3. Arms and hypotheses (sharpened)

Five curricula, all at (β=0.0046875, τ=5000), same schedule shape
(40 presentations, 500 ms, 2 s cadence), differing ONLY in
composition/order:

| arm | composition | isolates |
|---|---|---|
| A | 40×A | reference |
| C | 40×C | channel-identity effect (vs A) |
| IL | 20×A + 20×C interleaved (seeded shuffle) | temporal structure at matched 50/50 composition |
| BAC | 20×A then 20×C (blocked) | order at matched composition |
| BCA | 20×C then 20×A (blocked) | order, opposite |

Distinguishable hypotheses:

- **H0 (pure count):** only total afferent drive matters ⇒
  A = C = IL = BAC = BCA in M.
- **H1 (composition):** antecedent identity matters (different
  recruited winner sets) ⇒ A ≠ C. Says nothing about order.
- **H2 (recency/order):** the final block dominates the drive-end
  margin ⇒ BAC ≈ C and BCA ≈ A (converging on the pure arms of
  their SECOND block), and |BAC − BCA| ≈ |A − C|.
- **H3 (integration/interference):** interleaving prevents any
  winner from consolidating ⇒ IL < max(A, C) (or IL at noise).

These are mutually exclusive in pattern, not just in sign: e.g.
H2 predicts the blocked pair DIFFERS by the same amount the pure
pair differs; H3 predicts the blocked pair behaves like the larger
pure arm while IL is suppressed; H0 predicts nothing differs. The
blocked pair is retained because it is the ONLY arm pair that
isolates sequential order at matched composition and matched
gross structure — the minimal sequential-history signature. (In
X3, drive-composition sensitivity existed at 5/5 seeds, but IL
suppressed margins at 2/4 clean seeds — H3 vs H2 is genuinely
open.)

## 4. Design

- Seeds: **6**, pre-declared: {20260912, 424242, 9001, 123456,
  777, 31337} (X3's five + one fresh, fixed before execution).
- Runs: 6 seeds × 5 arms = **30 runs**, one per cell (runs are
  deterministic; repetition within a cell is meaningless — the
  statistical unit is the seed, not the run).
- Placement: band cell β=0.0046875, τ=5000 (X3: 0/15 aborts
  here; 14/15 silent one β step down; 0/15 silent one step up).
- Analysis (pre-specified, non-parametric, paired within seed):
  - Primary contrasts: (i) A vs C; (ii) BAC vs BCA; (iii) IL vs
    mean(BAC, BCA). Test: exact permutation/sign test on within-
    seed paired differences of M (6 seeds ⇒ two-sided min p =
    2/2^6 = 0.031); report median within-seed Δlog10 with exact
    CI. Aborts enter as +∞ ranks.
  - Secondary (mechanistic, flagged as such): Spearman(M, drive-
    end top-u) pooled and within-seed (P1); winner-neuron identity
    overlap between arms per seed (P2 mechanism); trajectory
    shapes (decay vs sustained).
- Predictions of the u-margin hypothesis (falsifiable, from X1–X3):
  - P1: M rank-correlates with drive-end top-u (monotone margin).
  - P2: where BAC ≠ BCA, the direction matches the second block's
    pure arm (recency through last-recruited winner).
  - P3 (retained negative): no per-trial A/C separation in
    within-drive u vectors (X2 replication, tertiary, optional).
- Failure-state handling: abort = divergence class (§1.3,
  §2 Tier 1); no detector recalibration; no retries; aborted
  cells are reported in the primary table.
- Constraint check: local mechanisms only (V2.1 untouched — only
  the two registered config fields and curriculum differ);
  curricula are environment-side schedule choices (no organism
  semantics); no labels/reward/credit; deterministic provenance
  (seeded, single Xoshiro); negative results retained; regime
  class + top-u reported as observation, u-margin framing
  confined to the hypothesis section.

## 5. What promotion requires (not done here)

User approval of this draft as-is or amended; then freeze
(configs + endpoint code + seeds + analysis plan committed) BEFORE
execution; then implement/execute as a numbered E-series
experiment. The old frozen Stage C remains untouched and
unexecuted; superseding it (given X2 O2.3: its E21-paradigm drive
aborts at the selected point) is a separate user decision.
