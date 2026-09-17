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
---

## E14 execution record (2026-09-17)

Instrument: af3794d + two output-side fixes at HEAD (both
definition-neutral): (1) trailing-window selectivity loop sized
per-neuron arrays before patterns were fully collected -> index
out-of-bounds panic after the per-REV table (fixed by two-pass
collection); (2) A-C comparison used matched-index pairs instead of
E13's full pairwise mean (424242 T40 0.003 -> 0.002, now
E13-identical). Raw/L1 both printed (identical at every checkpoint,
as in E12/E13). No measurement definition, threshold, checkpoint, or
reference-window change.

### Verification (all pass)

1. Hash pins: telemetry + snapshot aggregates byte-identical to the
   E13 freeze table for all three run dirs.
2. Determinism: every run's instrument output byte-identical on
   rerun (3/3).
3. T0 anchors bit-identical to E13's record: 20260912 0.680/0.075,
   9001 0.760/0.097, 424242 0.554/0.065 (raw == L1).
4. Supporting columns at T10..T60 byte-identical to E13's record
   (selectivity, permanence, failures=0, rates; A-C now exact).
5. REV-B in-round positions match E13 (round 61: 0/2/2; round 120:
   1/2/2).

### Per-REV trajectory (reference-anchored; raw == L1 everywhere)

T0 values are the E13 anchor (B rounds 51-60 vs A/C refs 51-60);
REV k = the k-th REV-B presentation (rounds 61-120).

seed 20260912 (k* = 1):

| cp | A-B | B-C | align | indep |
|---|---|---|---|---|
| T0 | 0.680 | 0.075 | C | F |
| REV1 | 0.190 | 0.929 | A | F |
| REV2 | 0.265 | 0.901 | A | F |
| REV3 | 0.166 | 0.936 | A | F |
| REV4 | 0.216 | 0.938 | A | F |
| REV5 | 0.205 | 0.941 | A | F |
| REV6 | 0.177 | 0.908 | A | F |
| REV7 | 0.196 | 0.936 | A | F |
| REV8 | 0.243 | 0.918 | A | F |
| REV9 | 0.287 | 0.888 | A | F |
| REV10 | 0.230 | 0.905 | A | F |
| REV20 | 0.223 | 0.894 | A | F |
| REV30 | 0.208 | 0.878 | A | F |
| REV40 | 0.360 | 0.816 | A | F |
| REV50 | 0.279 | 0.822 | A | F |
| REV60 | 0.270 | 0.837 | A | F |

seed 9001 (k* = 1):

| cp | A-B | B-C | align | indep |
|---|---|---|---|---|
| T0 | 0.760 | 0.097 | C | F |
| REV1 | 0.127 | 0.672 | A | F |
| REV2 | 0.239 | 0.625 | A | F |
| REV3 | 0.125 | 0.663 | A | F |
| REV4 | 0.189 | 0.628 | A | F |
| REV5 | 0.174 | 0.634 | A | F |
| REV6 | 0.249 | 0.619 | A | F |
| REV7 | 0.213 | 0.642 | A | F |
| REV8 | 0.199 | 0.642 | A | F |
| REV9 | 0.178 | 0.632 | A | F |
| REV10 | 0.317 | 0.495 | A | **T** |
| REV20 | 0.169 | 0.600 | A | **T** |
| REV30 | 0.123 | 0.676 | A | F |
| REV40 | 0.237 | 0.614 | A | F |
| REV50 | 0.199 | 0.600 | A | F |
| REV60 | 0.158 | 0.610 | A | F |

seed 424242 (k* = 1):

| cp | A-B | B-C | align | indep |
|---|---|---|---|---|
| T0 | 0.554 | 0.065 | C | F |
| REV1 | 0.112 | 0.746 | A | F |
| REV2 | 0.113 | 0.790 | A | F |
| REV3 | 0.146 | 0.801 | A | F |
| REV4 | 0.146 | 0.750 | A | F |
| REV5 | 0.146 | 0.749 | A | F |
| REV6 | 0.097 | 0.775 | A | F |
| REV7 | 0.136 | 0.763 | A | F |
| REV8 | 0.156 | 0.766 | A | F |
| REV9 | 0.114 | 0.771 | A | F |
| REV10 | 0.130 | 0.700 | A | F |
| REV20 | 0.183 | 0.529 | A | **T** |
| REV30 | 0.204 | 0.604 | A | F |
| REV40 | 0.246 | 0.484 | A | **T** |
| REV50 | 0.153 | 0.608 | A | F |
| REV60 | 0.225 | 0.516 | A | **T** |

Supporting columns (trailing-window; permanence = candidate-
permanence events; rates = internal mean/max): selectivity jumps at
the first full REV window and stays high (20260912: 0.527 -> 0.869;
9001: 0.685 -> 0.836; 424242: 0.933 -> 0.907 at REV10; E13-identical
at T10..T60). Permanence monotone through the early REV windows
(716..1256) then settles 415-640/10-rounds — structural engagement
active at every checkpoint. Failures 0 everywhere; rates 7.4-10.8
mean / 73-102 max Hz (inside the 250 Hz gate). A-C <= 0.002 at all
T10..T60.

### D2 result (frozen rule)

k* = 1 for ALL THREE seeds (first REV presentation is already
A-aligned, and alignment stays A through REV10 and through REV60 in
every seed; first unsustained flip = REV1 == k*, i.e. the flip never
wobbles back at this resolution). Registered transition interval per
seed: **(T0, REV1]** — the entire C->A re-anchoring completes within
the first REV presentation (presentation-level bound; no in-
presentation claim).

### E13 comparison and frozen interpretation

E13's coarse bound (<= 10 REV presentations, F1-majority) is
resolved: the transfer is complete at the FIRST REV presentation in
all seeds (A-B collapses 0.680/0.760/0.554 -> 0.190/0.127/0.112; B-C
rises to 0.929/0.672/0.746 — already past the crossing).

E13's D (seed-dependent dynamics) re-examined with the raw per-REV
facts (frozen protocol section 6): k* is identical across seeds =>
E13's seed dependence does NOT reflect transition timing. The per-REV
shapes disagree only in the post-flip tail: transient B-independence
episodes in 9001 (REV10, REV20) and 424242 (REV20, REV40, REV60),
absent in 20260912. E13's minority-C signature (424242) is confirmed
and extended at presentation resolution (recurring coexistence
episodes; 9001 shows the same feature weakly). Reading recorded:
**E13's seed-dependent classification reflects different
intermediate (post-flip) representation dynamics — transient
coexistence episodes — not transition timing, and not merely coarse
sampling of the flip** (the flip itself was fully captured at the
first E13 post-T0 checkpoint; what the coarse grid under-sampled was
the episodic independence windows, which E13's 10-round windows saw
once in 424242 and never in 9001).

Final E14 verdict (frozen machinery only, no new category):
transition bound **(T0, REV1], uniform 3/3 seeds**; E13's D resolved
to intermediate-dynamics origin with timing excluded.
