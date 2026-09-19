# ANIMA post-V2.1 exploratory research log (X-series)

Status: EXPLORATORY (agent-autonomous phase, 2026-09-19/20). No
E-numbers. No mechanism/config changes to ANIMA itself — every run
here is the committed V2.1 substrate (`β=0` identity default
preserved; exploratory runs only vary the two registered V2.1
config fields and drive composition). Historical record untouched;
all runs logged with configs + telemetry under `runs/xp*`.
Instruments: `examples/v21phase.rs`, `v21contrast.rs`,
`v21contra2.rs`, `v21dump.rs` (read-only over telemetry+snapshots).

## 1. Research review — what is actually missing

Established: V2 substrate (E1–E23); V2.1 Stage A (identity); Stage
B (persistent endogenous activity in a stable wedge). Unknown:
whether that endogenous activity carries antecedent-specific
information (Stage C never executed; A-only probe runs contain no
contrast). The prior audit also established two structural
warnings: pacemaker saturation (no dynamic range) and u-core
concentration (Gini hardening).

Candidate directions considered, each with what it would
distinguish and its smallest experiment:

- **D1 — Information content of endogenous activity** (the
  expected next question): does u/spiking differ after A vs C?
  Distinguishes "persistence ≠ information" from "carries
  antecedent identity". Smallest: one run, interleaved A/C drive,
  same schedule otherwise (X2 below). Chosen as one of two
  initial investigations — cheap, directly on the open question.
- **D2 — Operating-point structure of the persistence regime**:
  the Stage-B wedge is 2 cells at one seed; its shape (sharp
  onset? intermediate regimes?) determines whether "saturated
  pacemaker" is intrinsic to u or just the sampled points.
  Distinguishes "u ⇒ pacemaking necessarily" from "u admits a
  dynamic-range regime". Smallest: fine β/τ sweep flank +
  regime classifier (X1 below). Chosen — the audit's negative
  structural prior (saturation) is itself unverified as a
  NECESSARY property.
- **D3 — Closed loop under V2.1** (E19 paradigm + u): does
  self-generated activity change output behavior? Distinguishes
  "u is behaviorally inert" from "u participates". Larger
  (disruption harness + closed arm); deferred — depends on
  knowing whether the carrier regime (D1/D2) exists at all.
- **D4 — Readout-side mechanisms** (E5-era gating, decoder
  alternatives): treats the output side. Deferred: premature —
  the internal carrier question (D1/D2) gates it.
- **D5 — Richer slow-state dimensionality** (multiple u
  timescales): architectural addition, unjustified before D1/D2
  establish what a single u can/cannot carry.

Initial selection: **D2 + D1**, jointly — D2 maps where a carrier
regime could exist; D1 measures information at the located
points. (Discovery-first: the audit's saturation warning makes
"which regime are we even probing" prior to "does it inform".)

## 2. X1 — operating-point sweep (P1)

Method: v21probe schedule (S0 5 s | 40×A cadence 2 s | 20 s
silence), seed 20260912, τ/β as below; classifier on silence
activity (v21phase). Committed grid re-analyzed first:

| cell | outcome |
|---|---|
| (0.003125, 2500) | silent |
| (0.00625, 2500) | silent |
| (0.003125, 10000) | pacemaker (5 neurons, 487 Hz max) |
| (0.00625, 5000) | pacemaker (6 neurons, 487 Hz max) |
| all β ≥ 0.00625 with β·τ ≥ 62.5 | runaway abort (t≈10–40 s) |

Observations (kept separate from interpretation):
- O1.1 Onset in β at fixed τ=5000 is sharp between 0.003125
  (1 stray spike) and 0.0046875 (persistent, 2,890 spikes/20 s).
- O1.2 **An intermediate regime exists**: (0.0046875, 5000) has
  sustained endogenous firing at max 76.9 Hz, Fano 17.4 (top-5),
  4 active neurons — *below* refractory ceiling, with a decaying
  tail over the full 20 s (2 s-bin ratios ≈ 0.75 ⇒ effective
  decay τ ≈ 8.2 s vs configured u τ = 5 s).
- O1.3 τ-scan at β=0.003125: 3500 silent, 5000 (effectively)
  silent, 7000 pacemaker — onset in τ also present.
- O1.4 The upper "boundary" of persistence at larger β·τ is the
  P2 runaway detector (an instrument setting, 50 Hz/5 s), not an
  observation of unbounded divergence.

Interpretation (hypothesis, not finding): u-persistence is a
per-neuron bifurcation — below onset u decays without
self-regeneration; between onset and saturation a
dynamic-range regime exists; the committed wedge cells sit at
the saturated end. The 8.2 s effective decay > τ=5 s suggests
regeneration loops (spike→u→spike) extend the memory beyond the
bare time constant in the decaying regime.

## 3. X2 — antecedent contrast probe (P2)

Method: same schedule/drive length, `present = ["A","C"]`
(interleaved, 40 reps each within the same 2000 ms cadence), at
the three operating points; plus C-only and a second seed (424242).

Runs and endogenous-silence outcomes:

| run | drive | silence activity |
|---|---|---|
| xp2 (0.003125, 10000) | A/C | pacemaker (5 n., 29,768 spk) |
| xp2 (0.0046875, 5000) | A/C | dies ≈ 4.5 s (238 spk, 2 n.) |
| xp2 (0.00625, 5000) | A/C | **runaway abort t=40 s** |
| xp2c (0.0046875, 5000) | C-only | **pacemaker** (3 n., 15,415 spk) |
| xp1 (0.0046875, 5000) | A-only | decaying tail (3,745 spk) |
| xp2r (0.0046875, 5000, seed 424242) | A/C | silent (67 spk burst) |
| xp2rA (0.0046875, 5000, seed 424242) | A-only | dies ≈ 5.4 s (272 spk) |

Observations:
- O2.1 **At a fixed (β,τ,seed), the persistence attractor depends
  on drive composition**: C-only → self-sustained pacemaker;
  A-only → transient decay; interleaved A/C → rapid death (seed
  20260912). The endogenous regime is antecedent-dependent — but
  as a slow, history-level dependence, not a per-trial code.
- O2.2 **Seed-fragility**: at seed 424242 the same intermediate
  cell shows no sustained persistence under either drive. The
  intermediate regime is not robust across seeds (n=2).
- O2.3 Interleaving A/C at the Stage-B selected point
  (0.00625, 5000) crosses the runaway detector at t=40 s (during
  drive) — the committed Stage-C plan (E21-paradigm sweep at this
  point) would have hit P2 immediately. (Observation only.)
- O2.4 Within-run, per-presentation u vectors do NOT separate
  A from C: cross-class cosine ≈ within-class (gap +0.003,
  permutation p=0.12 intermediate; gap −0.002, p=0.35 pacemaker;
  nearest-centroid LOO 12/80 ≈ chance). The slow state at these
  points integrates drive jointly, not per-antecedent.
- O2.5 Post-presentation off-window spike vectors likewise
  class-indifferent (cos ≈ 0.999 throughout; the q3/q4 dip to
  0.978 at the intermediate cell tracks the dying regime, not a
  class effect).
- O2.6 In the decaying A-only intermediate run, silence-active
  neurons ARE the top-u neurons at drive end (n47 u=5.35, n53
  u=5.19 → 1847/1753 spikes; n39 u=1.71 → 101) — u is the
  proximal carrier of persistence, monotone in magnitude.

Interpretation (hypothesis): u carries INFORMATION about the
drive history at the level of *which attractor the network
settles into* (O2.1: attractor selection by antecedent mix), not
as a per-trial discriminative code (O2.4/O2.5). This reframes the
open question: the interesting capacity metric is not per-trial
separation (E21's D_L) but regime selection — and the regime
boundary is seed-sensitive, so any formal test needs multi-seed
design.

## 4. What ANIMA needs next (candidate conclusions, ungraded)

- The missing capability remains a temporal carrier, but the
  measurable signature is regime-level, not vector-level. A
  formal experiment should be built around attractor/regime
  readout with ≥3 seeds and drive-composition arms.
- The intermediate regime (dynamic range, decaying multi-second
  tail, effective τ ≈ 8 s > τ_u) is the most promising carrier
  found; it is seed-fragile, which is itself the next mechanism
  question (what stabilizes/destabilizes it across seeds).
- Negative retained: no per-trial A/C code in u or endogenous
  spiking at any probed point (2 points, 2 seeds, 3 drive
  compositions).

## 5. Provenance

All runs: `runs/xp1-*`, `runs/xp2*` (dirs listed in §2/§3
tables), configs `configs/xp1-*`, `configs/xp2*`, generated by
`scripts/gen_xp.py` + inline documented edits. Seed 20260912
(frozen) unless stated. Nice 10. Determinism: single-run
re-execution reproduces byte-identical telemetry (substrate
guarantee, unchanged).

STOP after this log per mandate. No E-number assigned; no
preregistration made.
