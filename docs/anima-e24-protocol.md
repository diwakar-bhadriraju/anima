# ANIMA E24 — Regime-selection capacity (FROZEN PROTOCOL)

Promoted from draft v2 (docs/draft-e24-regime-capacity.md, commit
4f54a2b) unchanged in design. Frozen 2026-09-19 BEFORE execution.
Category: measurement experiment on committed V2.1 substrate
(only registered V2.1 config fields + curriculum vary). No
mechanism change. No detector change. Not an amendment of Stage C
(that record stays retracted/not-executed and untouched).

## Question

Do endogenous dynamics retain information about prior drive
composition after drive offset (memory, not reflex)?

## Design (frozen)

- Substrate: V2.1, β=0.0046875, τ_s=5000 (X3 transition band;
  0/15 X3 aborts). v21probe schedule otherwise: S0 5 s silence |
  S1 drive: 40 presentations, 500 ms each, 2 s cadence | S2 20 s
  silence. Seed frozen per run.
- 6 seeds (pre-declared): 20260912, 424242, 9001, 123456, 777,
  31337.
- 5 arms (composition only; total presentations 40 in all arms):
  a   = 40×A interleaved
  c   = 40×C interleaved
  il  = 20×A + 20×C interleaved (seeded shuffle)
  bac = 20×A then 20×C (blocked)
  bca = 20×C then 20×A (blocked)
- 30 runs total, deterministic. Configs: configs/e24-s{seed}-{arm}
  .toml (scripts/gen_e24.py). No retries, no censoring, no
  mid-experiment changes of any kind. Aborts preserved.

## Primary endpoint (frozen; instrument e24_endpoint.rs)

M per run:
- run aborts on runaway-activity  ⇒ M = +∞ (top rank)
- otherwise                        ⇒ M = log10(1 + S10)
  S10 = endogenous (non-input, id≥24) spikes in
  [silence_onset, silence_onset + 10 s), silence_onset = last
  StimulusPresented t + 500 ms (relative window; delivery-tick
  convention arm-symmetric).

## Hypotheses (frozen; mutually exclusive patterns)

- H0 pure count: a = c = il = bac = bca (in M).
- H1 composition: a ≠ c (channel-identity effect).
- H2 recency/order: bac ≈ c and bca ≈ a; |bac − bca| ≈ |a − c|.
- H3 interference: il < max(a, c) (interleaving suppresses
  consolidation).

## Statistical procedure (frozen)

Within-seed paired contrasts on M; the seed is the statistical
unit (n=6). Exact two-sided sign test on paired differences
(min attainable p = 2/26 = 0.031) with median within-seed Δlog10
and exact CI. Aborts enter as +∞ ranks (all arithmetic on ranks;
for medians of Δ involving an abort the Δ is +∞-dominated and is
reported as such). Contrasts: C1 a vs c; C2 bac vs bca; C3 il vs
mean(bac, bca) (pair each seed's il against that seed's mean of
the two blocked arms). Secondary contrasts reported but
non-primary: bac vs c, bca vs a (H2's convergence form).

Verdict rules (frozen):
- A hypothesis Hk is SUPPORTED if its full pattern holds across
  contrasts and no simpler hypothesis explains the pattern; H0 is
  the null of "no curriculum information retained".
- Primary claim "endogenous dynamics retain drive-composition
  information" requires: C1 or C2 significant (p ≤ 0.05 exact
  sign) AND the winning pattern consistent (H1 or H2 shapes).
- All other outcomes = NOT SUPPORTED (retained as negative
  results).

## Secondary readouts (frozen list, descriptive/mechanistic)

Regime class (silent/sparse-core/pacemaker/irregular; thresholds
as in v21phase), drive-end top-u (neuron id + magnitude) and
top-3 u sum, winner-identity overlap across arms within seed,
silence trajectory (2 s bins, 20 s), output-class participation
in silence (ids + spike counts). Prediction checks (u-margin
hypothesis, falsifiable): P1 Spearman(M, top-u); P2 direction of
bac/bca matches second block's pure arm; P3 no per-trial A/C u
separation (optional tertiary).

## Freeze integrity

- Config generator scripts/gen_e24.py (committed pre-exec).
- Endpoint instrument crates/anima-exp/examples/e24_endpoint.rs
  (committed pre-exec; sanity-checked against X3 cells).
- This document is the protocol of record; execution record
  appended below after runs complete.

---

# E24 EXECUTION RECORD (2026-09-19, post-freeze commit 93bb672)

All 30 runs executed (runs/e24-s{seed}-{arm}-2026...Z), all
`curriculum-complete` — zero aborts (consistent with X3: 0/15
aborts at this cell). No retries, no recalibration, no censoring.
Deterministic provenance: single seeded Xoshiro per run; seeds as
frozen.

## Per-seed / per-arm results

Primary endpoint M = log10(1 + S10):

| seed | a | c | il | bac | bca |
|---|---|---|---|---|---|
| 20260912 | 3.343 | 3.954 | 3.406 | 2.702 | 3.869 |
| 424242 | 1.887 | 1.940 | 0.778 | 2.375 | 2.288 |
| 9001 | 1.580 | 0.000 | 1.785 | 3.410 | 1.146 |
| 123456 | 3.628 | 3.225 | 3.079 | 3.447 | 3.911 |
| 777 | 0.000 | 3.998 | 2.124 | 2.886 | 0.602 |
| 31337 | 3.966 | 3.815 | 4.145 | 3.649 | 3.655 |

Regime classes: silent×2 (777-a, 9001-c), sparse-core×8,
pacemaker×20. Zero aborts. Output participation: 11/30 runs have
output neurons in the silence-active set (e.g. 31337-il: output
74 carries 6,311 of 13,963 silence spikes; 20260912-bca/c:
outputs 64+70) — endogenous OUTPUT activity exists at this
operating point (E19-relevant precondition, now observed).

## Primary analysis (preregistered exact within-seed sign tests)

- C1 a vs c: pos=3 neg=3 ties=0, exact p=1.000, median Δ +0.151.
- C2 bac vs bca: pos=3 neg=3 ties=0, exact p=1.000, median Δ +0.087.
- C3 il vs mean(bac,bca): pos=3 neg=3 ties=0, exact p=1.000,
  median Δ +0.121.
- All pairwise reference contrasts also p ≥ 0.6875.

Per frozen verdict rules: C1 and C2 both far from significance;
no arm pattern is consistent across seeds (e.g. 777: a=0.000,
c=3.998; 9001: a=1.58, c=0.000 — opposite directions).

## Hypothesis results

- H0 (pure count): NOT REJECTED — no contrast significant.
- H1 (composition): NOT SUPPORTED (C1 p=1.000; directions
  inconsistent across seeds).
- H2 (recency/order): NOT SUPPORTED (C2 p=1.000).
- H3 (interference): NOT SUPPORTED (C3 p=1.000; il exceeds the
  blocked mean in 3/6 seeds).
- Primary claim "endogenous dynamics retain drive-composition
  information": **NOT SUPPORTED** at this operating point under
  this design.

## Secondary / mechanistic observations (separate from verdict)

- S1. P1 CONFIRMED: Spearman(M, drive-end top-u) = 0.968 over 30
  runs — M tracks the u-margin mechanism almost perfectly
  (falsifiable prediction of the u-margin hypothesis held).
- S2. Winner identity is curriculum-dependent WITHIN seed: each
  seed's five arms recruit 3–4 distinct winner neurons. [ERRATUM
  2026-09-20: original text overstated the primacy counts as 4/6
  and 5/6; re-verified from the run table: bac's winner matches
  the a-arm winner in 3/6 seeds (20260912, 424242, 123456);
  bca's winner matches the c-arm winner in 4/6 (424242, 123456,
  777, 31337) — 7/12 first-block matches vs 0/12 second-block
  matches. Among the 12 blocked comparisons, discordant pairs
  favor primacy 7:0 (sign-test p≈0.008, POST-HOC — not
  preregistered, descriptive only; motivates O-E24-1/E25).]
- S3. The winner-recruitment structure exists but does not
  propagate to a consistent M effect: when the recruited winner
  differs, its margin crosses or misses the regeneration
  threshold seed-by-seed (e.g. 777: a-arm winner n55 margin 0.83
  → silent; c-arm winner n41 margin 8.08 → 9,961 spikes).
- S4. Trajectory classes as in X3 (decay vs sustained), plus
  31337-il: il arm produced the LARGEST M of all 30 runs
  (4.145) — interleaving does not universally suppress.

## Limitations

- n=6 seeds gives min attainable two-sided p=0.031 per contrast;
  the observed p=1.000 patterns are not power-limited nulls
  (medians are small AND directions inconsistent).
- The transition band at β=0.0046875 was chosen for abort
  safety; margins near threshold amplify seed variance by design.
  A higher-β placement would raise margins but risk aborts.
- Curriculum information might live in readouts not captured by
  S10 (e.g. spike-timing structure within silence, winner
  identity as a discrete code) — S2's primacy effect is such a
  candidate, but it was not a preregistered endpoint and remains
  exploratory.

## Observations motivating future work (not designed here)

- O-E24-1: winner PRIMACY (first-recruited set dominates the
  drive-end state) — 7/12 first-block vs 0/12 second-block winner
  matches [corrected; see S2 erratum], the natural endpoint of a
  discrete-winner-code experiment.
- O-E24-2: endogenous output activity (11/30 runs) makes the
  closed-loop question (E19 paradigm under V2.1) newly reachable.
- O-E24-3: per-run margin variability under identical curriculum
  (777: 0 vs 9,961 between arms) suggests the band amplifies
  tiny drive differences into large regime differences — a
  potential amplification signature worth isolating from noise.

## Verdict (frozen-rule application)

**NOT SUPPORTED: endogenous regime (as measured by S10) does not
retain reliable drive-composition information after drive offset
at this operating point.** The mechanistic substrate for such
retention exists (S1–S3: margin tracks M at ρ=0.968; winner
recruitment is curriculum-sensitive with primacy) but the
curriculum-to-margin mapping is seed-dominated, not
curriculum-dominated. Negative result retained in full.

V2.1 and the historical Stage-C record untouched. No mechanism
introduced; no E25 proposed or executed. STOP.
