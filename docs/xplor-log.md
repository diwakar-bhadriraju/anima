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

---

## 6. X3 — regime boundary map (frozen grid, executed 2026-09-19)

Grid frozen BEFORE execution: seeds {20260912, 424242, 9001,
123456, 777} × drive {A-only, C-only, A/C interleaved} × β
{0.003125, 0.0046875, 0.00625}, τ=5000, v21probe schedule. 45 runs
(`runs/xs-*`, configs `configs/xs-*` via `scripts/gen_xs.py`);
grid NOT expanded/optimized mid-run. Instrument: `v21bmap`.
Recorded per run: end reason, silence regime class, drive-end
top-u (id, magnitude), top-3 u sum, active count, first/last-5 s
silence spikes.

### 6.1 Full map (regime class; abort cells marked)

| seed | β=0.003125 | β=0.0046875 | β=0.00625 |
|---|---|---|---|
| 20260912 A | sparse-core(1 spk) | pacemaker | pacemaker |
| 20260912 C | silent | pacemaker | **abort@18 s** |
| 20260912 AC | silent | sparse-core | **abort@40 s** |
| 424242 A | silent | sparse-core | pacemaker |
| 424242 C | silent | sparse-core | **abort@10 s** |
| 424242 AC | silent | silent | **abort@10 s** |
| 9001 A | silent | sparse-core | **abort@23 s** |
| 9001 C | silent | silent | pacemaker |
| 9001 AC | **abort@23 s** | sparse-core(decay) | pacemaker |
| 123456 A | silent | pacemaker | pacemaker |
| 123456 C | silent | pacemaker | pacemaker |
| 123456 AC | silent | sparse-core | pacemaker |
| 777 A | silent | silent | pacemaker |
| 777 C | silent | pacemaker | pacemaker |
| 777 AC | silent | silent | pacemaker |

### 6.2 Observations (separate from interpretation)

- O3.1 β=0.003125: 14/15 silent (one 1-spike exception). Below
  onset at every seed and every drive tested.
- O3.2 β=0.00625: 0/15 silent — 10 pacemaker, 5 aborts on the P2
  runaway detector mid-drive. Above onset at every seed/drive.
- O3.3 β=0.0046875 is the transition band: 5 pacemaker, 6
  sparse-core, 4 silent across seeds/drives. Drive composition
  shifts outcome WITHIN the band at fixed seed (20260912: A→pm,
  C→pm, AC→sparse; 777: A→silent, C→pm, AC→silent) but not
  consistently across seeds.
- O3.4 Drive-end top-u is a strong single-number regime
  predictor across all 40 complete runs: top-u > 2.0 separates
  persistent (pacemaker/sparse-core) from silent at 16/22 vs
  17/17 (accuracy 0.85; all six misclassified persist-cells are
  marginal sparse-core with ≤96 spikes). Silent runs never
  exceed top-u 0.92; pacemakers always ≥ 6.
- O3.5 All 5 aborts are mid-drive runaway-detector crossings at
  larger β·(drive intensity), not silence instabilities; two
  anomalous cells (9001) abort in the "wrong" arm for their β,
  showing the transition band is also abort-fragile.
- O3.6 Silence trajectory classes: pacemaker = flat or slowly
  decaying 10 s⁴–10 d spikes/5 s; sparse-core = short tail (≤
  few hundred spikes, dies < 6 s); silent = zero.

### 6.3 Interpretation (hypothesis)

- The intermediate regime is not a wedge in seed space but a
  TRANSITION BAND at β≈0.0047 whose outcome per (seed, drive) is
  set by whether the drive-end winner neuron's u margin exceeds
  the self-regeneration threshold (~u≈2 at these settings).
  Drive composition perturbs which neuron wins and its margin —
  the boundary is history-dependent, exactly the lever a
  regime-selection capacity experiment needs.
- The E23-era warning returns at β=0.00625: the P2 detector
  (50 Hz/5 s) aborts 5/15 mid-drive cells. Any formal experiment
  at the wedge/above must either target the band or treat the
  detector threshold as part of the design.

### 6.4 Negative results retained

- No per-trial A/C code in u (X2 §3, permutation-tested) — X3
  does not revisit it.
- β=0.003125 universal silence (14/15) — the earlier single-cell
  silence at τ=2500 generalizes across 5 seeds and 3 drives.
- Drive-composition effect on regime is REAL but seed-inconsistent
  in the band (O3.3): 2 seeds shift, 2 do not.

## 7. Standing STOP honored

X3 executed as a frozen exploratory grid; nothing promoted to
E-number; draft protocol for the regime-selection capacity
experiment written separately (docs/draft-e24-regime-capacity.md)
and NOT executed; old Stage C NOT executed; V2.1 not modified.


---

## 8. E24 draft v2 revision (review turn, no new runs)

X3 re-reviewed per mandate; draft protocol rewritten in place
(docs/draft-e24-regime-capacity.md v2). Decisions:
- PRIMARY ENDPOINT: two-tier ordinal M (abort = +inf rank;
  else log10(1 + S10), S10 = endogenous spikes in first 10 s of
  silence). Grounds: phenotype not mechanism (hard 40/40 silent
  boundary vs 34/40 for top-u>2; X3 marginal cells resolved only
  by spiking); keeps 4 decades of magnitude; retains abort info;
  no new instrumentation.
- Top-u demoted to secondary/mechanistic (avoids conflating
  observation with the u-margin hypothesis it is supposed to
  test).
- Aborts = distinct divergence outcome class, ranked top; P2
  detector NOT recalibrated; E24 placed at band beta=0.0046875
  (0/15 aborts in X3).
- Seeds 6 (X3 five + 31337), 30 runs, paired within-seed exact
  permutation tests; cross-seed raw-margin comparison excluded as
  confounded (10x seed baselines).
- Arms sharpened to mutually exclusive hypotheses H0 (pure
  count), H1 (composition), H2 (recency/order), H3 (interleaved
  interference); blocked pair retained as the only order-
  isolating contrast.
No exploratory runs executed this turn; both open methodological
questions (blocked-stage semantics env.rs:340-346; silence
readout window safety) resolved from existing artifacts. V2.1
not modified. E24 NOT executed.


---

## 9. V2.2 architectural spec frozen (not implemented, not executed)

User approved the architectural slate as a candidate direction;
bundle decision: G1+G2+Y1 (smallest attributable intervention;
G1-alone untestable as a carrier — homogeneous threshold repeats
the seed-lottery; G1-without-Y1 unwritable — no return path).
Frozen: docs/v2_2-spec.md - exact local dynamics (two-variable
per-neuron state u,z; SET theta / RESET phi gate; plateau added
to u not replacing it; per-spike eta subtraction), update order,
LogNormal heterogeneity draws (theta_rel 2.0+/-0.35, U_rel 0.9
+/-0.35, tau-het off), identity nulls (enable=false bit-exact
V2.1, sd=0 skips RNG draws), stability criteria (P2 unchanged;
latch-saturation recorded not suppressed), success criteria
C-S1..C-S5 (decoding>chance, 2s latch-set stability >=80%,
<300Hz, no new instability, byte-identity), falsifiers F1-F6,
and the 4-arm x 6-seed x 5-curriculum Stage-2 discrimination
plan whose BETWEEN-arm paired differences are the component
attribution. STOP after freeze, per mandate.


---

## 10. V2.1 u-information reproducibility experiment (read-only + 3 runs)

Question restored: does the ORIGINAL ungated u carry
sensory/curriculum information after stimulus offset? Never
measured directly before (all prior A/C readouts were spike-
windows or gate experiments). No mechanism change: historical
V2.1 configs, gated write OFF, V2.3 partition OFF.

Runs:
- runs/v21rep-b0.00625-t5000-20260920T154419Z — EXACT historical
  config (configs/v21rep-b0.00625-t5000.toml, byte-identical to
  committed v21probe-b0.00625-t5000.toml save exp_id).
- runs/v21repac-b0.003125-t10000-20260920T154446Z — historical
  second-wedge config (b0.003125, t10000), S1 roster A/C
  interleaved (40+40 pres; organism/timing untouched).
- runs/v21repac-b0.00625-t5000-20260920T154428Z — interleaved
  A/C at the SELECTED cell: runaway-activity abort at S1
  (committed historical fact at that cell, X2 O2.3); preserved,
  not data.

IDENTITY/REPRODUCTION: v21rep-A-only = 511,333 non-marker rows,
FNV e4b018992c97d1a9 — BYTE-IDENTICAL to committed
v21probe-b0.00625-t5000-20260919T063803Z. A/C-0.003125 run
reproduces the committed xp2 numbers exactly: 29,768 silence
spikes, 5 active neurons, 487.6 max Hz, pacemaker regime.

INSTRUMENT (read-only): uvinfo (snapshot windows: during/post/
late at 1 s snapshot cadence) + urecon (deterministic
tick-exact u reconstruction from recorded spikes, sampling at
true intra-stimulus offsets 0/+50/+200/+250/+500/+1500 ms;
validated vs snapshots: 0.6% A/C cell, 5.7% A-only, bounded,
direction-neutral).

RESULTS (A/C cell, reconstructed u, per-offset cosine/dist):
offset:  +0ms  cos 0.9995 dist 0.57 ||A|| 18.23 ||C|| 18.31
         +50ms cos 0.9997 dist 0.45
         +200ms cos 0.9999 dist 0.20
         +250ms cos 0.9999 dist 0.25
         +500ms cos 0.9993 dist 0.77
         +1500ms cos 0.9993 dist 0.71  ratio A/C 0.996-1.015
Snapshot-read windows (uvinfo): during 0.9995, post 0.9993,
late 0.9994; top-10 u overlap Jaccard 0.900 at all windows.

INTERPRETATION (observation, not mechanism change): u at the
V2.1 wedge does NOT separate A from C at any temporal offset —
including DURING the stimulus (mid/end). The A/C condition
difference visible in afferent/spike structure does not enter u.
This is substrate diagnosis, matching interpretation D: even
during-stimulus u is not sensory-specific at these operating
points; norm ratio ~1.0 excludes a global-amplitude artifact.

CAVEATS kept separate:
- The A/C interleaved cell is the b0.003125/t10000 wedge (the
  selected cell aborts under interleaving — historical).
- Snapshot cadence aliasing closed via reconstruction; blocked-
  order u-cosines from committed E24 runs (0.82-0.95, top-10
  overlap 0.5-0.7) remain the only evidence of ANY u-level A/C
  separation, and those are at beta=0.0046875, not the
  historical wedge, with 1 s cadence.
- u carries the integrated spike history (all patterns mixed
  through shared recurrent pool) with no per-pattern separation:
  consistent with the earlier finding that during-window spike
  cosines separate (0.30-0.89) but u does not inherit it.

No E-number. No mechanism, no sweep, no gate. Records untouched.
