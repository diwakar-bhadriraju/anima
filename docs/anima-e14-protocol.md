# ANIMA E14 — Transition-resolution probe

Status: **FROZEN** (2026-09-17). No edits after this point except
registered amendments (A-series, user-approved, logged in the
appendix).

## 1. Question

E13 (Outcome D) bounded the C->A re-anchoring to the first 10 REV
presentations in all seeds. E14 resolves **where within REV1..REV10
(and onward) the registered representation first moves**. This is a
measurement-resolution probe — no simulation, no mechanism, no new
classification machinery.

## 2. Feasibility audit (decision: ANALYSIS-ONLY)

Every required measurement is reconstructible from the committed E12
run artifacts at per-REV-presentation resolution:

| metric | telemetry source (all read-only) |
|---|---|
| per-presentation internal vector | Spike rows (kind 3: `t`, `n` >= N_IN) within a presentation's [start, start+500) window |
| presentation boundaries / identity | StimulusPresented (kind 5, envelope "e"): pattern, stage, start |
| A-B / B-C / A-C cosines | pairwise-mean cosine over count vectors (frozen formula, raw == L1) |
| B-alignment | argmin(A-B, B-C), frozen |
| B-independence | frozen 0.60 rule (raw and L1) |
| selectivity | per-neuron (best-2nd)/best median over window (analyzer formula) |
| permanence / engagement | SynapseCreated (kind 6) with reason.trigger == "candidate-permanence" |
| P2 / failure status | Failure (kind 17) events + `snapshots.bin.zst` internal rate_hz |

No missing measurement => E14 runs the existing E12 artifacts only.
**Never rerun E12.** No new sim-side instrumentation of any kind.

## 3. Estimator (D1 — user-approved: fixed ref = rounds 51-60)

Reference representations: A_ref / C_ref = the 10 A / 10 C
presentations of S1 rounds 51-60 (E13's T0 window, the last pure-SEQ
state). For each REV presentation k (rounds 61-120, one B per round):

- ab_k = mean over i in A_ref of cos(B_k, A_i)       (10 pairs)
- bc_k = mean over i in C_ref of cos(B_k, C_i)       (10 pairs)
- B-alignment_k = argmin(ab_k, bc_k), frozen
- B-independence_k = frozen 0.60 rule on (ab_k, bc_k)

k=0 (T0) uses B presentations of rounds 51-60 against the same refs:
the identical 10x10 pair set as E13's T0 => **bit-identical
verification anchor** vs the E13 record (0.680/0.075 seed 20260912,
0.760/0.097 seed 9001, 0.554/0.065 seed 424242).

Supporting columns per checkpoint, with E13-identical methods:
- A-C: pairwise mean over A/C of the trailing window (equal to E13's
  Wk columns at T10..T60 — byte-identical verification);
- selectivity: per-neuron median over the trailing window; for k in
  1..9 the trailing window = rounds 51..60+k; for k >= 10 = Wk
  (rounds 60+k-9..60+k), i.e. exactly E13's windows;
- permanence / failures / rates: in-window counts (same windows).
- per-REV spike totals (diagnostic, no threshold).

## 4. Checkpoints

T0, REV1..REV10, REV20, REV30, REV40, REV50, REV60 — reported as the
complete per-presentation table REV1..REV60 plus the checkpoint
table. REV k presentation = S1 B in global round 60+k; start =
5000 + 6000(59+k) + 2000*pos_k with seed-dependent in-round
position pos_k (reported per seed; E13 values: round-61 positions
0/2/2, round-120 1/2/2).

## 5. Earliest-movement point (D2 — user-approved: sustained first
## A-alignment)

k* = smallest k in 1..60 with B-alignment_k = A **and**
B-alignment_j = A for every j in k..10 (sustained through the first
10 REV presentations; E13 established stability of the flip, so the
suffix rule is registered at the 10-presentation horizon that E13
already measured). Transition bound: **(REV(k*-1), REV(k*)]** for
k* >= 2; **(T0, REV1]** for k* = 1. The first unsustained A-flip
(if any) is also reported for transparency. No exact-time claims:
"between REV3 and REV4" is valid, "REV3.4" is not. No new
thresholds.

## 6. E13-classification disambiguation (no new machinery)

Report per seed: k*, the per-REV (ab_k, bc_k) trajectory, and
whether the crossing is monotone. E13's D / minority-C is then
re-examined with the raw per-REV facts:

- if k* is close across seeds and the per-REV shapes agree, E13's
  seed dependence reflected **coarse sampling / intermediate
  dynamics**, not timing;
- if k* differs materially, E13 reflected **genuine timing
  differences**.
Both readings reported from the raw tables; no verdict threshold,
no mechanism claims.

## 7. Verification (analysis-time, no reruns)

1. T0 anchor: e14 T0 A-B/B-C == E13 record values (bit-identical
   pair set).
2. Determinism: run the instrument twice on each run dir =>
   byte-identical output.
3. Supporting columns at T10..T60 == E13 table values where the
   methods coincide (selectivity except where E13's n=52 vs the same
   window; permanence, failures, A-C).
4. Run-dir telemetry + snapshot hashes re-verified vs the E13 pins.

## 8. Deliverables

1. This protocol (frozen commit).
2. `e14_resolution.rs` (read-only, telemetry-only, no anima_core
   import — static test) + unit tests (REV-presentation grid;
   reference window composition; k-suffix rule).
3. Full suite green, 0 warnings.
4. Per-seed tables + k* + disambiguation reading; execution record
   appended; registry updated; commit; report hash/tests/warnings/
   tree. Nothing proposed beyond E14.

## Appendix: amendments

- (none yet)