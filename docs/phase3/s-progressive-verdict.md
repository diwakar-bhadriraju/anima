# Phase III sX verdict — progressive acquisition demo (D-21)

Status: PARTIAL — progressive acquisition CONFIRMED; no-catastrophic-
forgetting NOT confirmed (measured memory cost). 2026-09-22.
Protocol: docs/phase3/s-progressive-protocol.md (frozen D-21).

## Setup (recap)
E-nogain platform; S1 forms A/C; NEW stage S1D introduces D (channels
16-23, 20 Hz, 30 reps blocked, never previously exposed) with NO A/C
exposure during S1D; S3A/S3C/S3D probe re-expression. Runs:
runs/clla-prog-s{seed}-il-20260922T18{1748,1801,1814}Z
(config clla-prog-s{seed}-il.toml, exp_id distinct). 3 seeds, all
curriculum-complete (no runaway). Controls: committed 1327Z no-D runs.

## Endpoint 1: PROGRESSIVE ACQUISITION — PASS (all 3 seeds)

D is formed online and retained: S3D re-expression cos-self vs its S1D
ref = 0.803 / 0.908 / 0.915; rho(D vs A) = +0.265/+0.226/+0.353;
rho(D vs C) = +0.001/+0.117/+0.465. D response is strongly self-similar
and (in 5/6 comparisons) exceeds similarity to A/C refs. A never-seen
pattern is acquired mid-run by experience, no retrain: CONFIRMED.

## Endpoint 2: NO CATASTROPHIC FORGETTING — FAIL (measured)

Control (no D): A S3A cos-self 0.967/0.982/0.914; C S3C cos-self
0.808/0.746/0.794.
After D trained in between (prog): A S3A cos-self 0.148/0.077/0.375;
C S3C cos-self -0.131/0.183/0.212. Severe re-expression degradation in
every seed/pattern. C selectivity sign flips in 2/3 seeds (S3C more
C-ref in s20260912 only; more A-ref or weak in s9001/s424242 - rho
-0.681 / +0.275... see numbers below). S3 mean_hz drops 45-76% vs
control (68/76/45% A; 38/46/40% C) - all > the frozen 40% bar.

Full rho rows (prog): A rho vs C: +0.263/+0.212/+0.301 (sign kept);
C rho vs A: -0.055/-0.681/+0.275 (sign lost 2/3); D: strongly self.

## Outcome class (frozen): PARTIAL

- Acquisition of NEW knowledge by experience: WORKS (the mission's core
  "learn by experience, no retrain" claim is evidenced).
- Retention of OLD knowledge while learning new: FAILS measurably
  (shared-pool reorganization; D's acquisition reuses/reorganizes the
  single 40-neuron pool at the old patterns' expense). The organism
  avoids retraining but exhibits the forgetting half of the LLM
  problem - in miniature.

## Mechanism (measured, no tuning)

The single shared all-excitatory pool is the binding constraint again:
40 neurons host A, C, AND D. d_claim/d_core track protection did NOT
prevent cross-pattern interference (both were ON). New-pattern
acquisition re-commits shared neurons, degrading old assemblies'
re-expression. This is the same representational ceiling as the
temporal line (D-09..D-19) - consolidation and reuse in one shared
pool.

## Recorded as the honest mission baseline

For the LLM-alternative goal (D-20): this substrate demonstrates
experience learning + persistent retrieval + no retrain, but at the
cost of old-knowledge degradation when new knowledge arrives. The next
mechanism-level question (NEW registration, approval required - NOT a
patch) is selective memory protection during new acquisition: how to
protect committed assemblies (the d_claim instinct) without blocking
new learning. Candidates: pattern-gated plasticity during acquisition,
or structured (non-shared) pool allocation - both are the same heavy
substrate-direction as the earlier blocked lines, now with a concrete
measured motivation.

STOP - verdict recorded; PARTIAL; no tuning; decision pending.

OBSERVATION (recorded, NOT tuned): the S1D acquisition block was 30
reps (long). Quantified here: D "scrambled" the S1 assemblies per cos-
self (A 0.97->0.15, 0.98->0.08, 0.91->0.38; C 0.81->-0.13, 0.75->0.18,
0.79->0.21), i.e. interference is structural (weight-space reuse), and
may scale with acquisition reps. Whether shorter/exposure-gated D
acquisition reduces interference is a NEW registration question, not a
parameter tweak of this demo. Fertilizes the next mechanism slice
(selective memory protection during acquisition).

AIR-TIGHTNESS: identical S1 rates per seed across prog/control (exact
match above) + identical seeds => identical S1 trajectories -> the S1
reference vectors used by retent.rs are BY CONSTRUCTION the same in both
arms. The measured S3 re-expression degradation is therefore attributable
to the D-interposed stages (S1D acquisition), NOT to reference drift.
