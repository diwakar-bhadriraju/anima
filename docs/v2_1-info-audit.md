# ANIMA V2.1 — Information-carrying audit (read-only)

Status: AUDIT (2026-09-20). Sole evidence: committed V2.1/V2
artifacts + repository code. No new simulations. Instrument
`uinfo` (telemetry+snapshots only; deterministic). No tuning, no
mechanism, no E-number, no next-experiment proposal.

## 0. ERRATUM (integrity, recorded first)

The V2.1 validation record (`d8b711b`, Stage C) presents a gap-
sweep table attributed to `runs/v21c-g{0,50,100,200,400,800}-*`.
**Those runs do not exist.** The repository contains only the 23
`v21probe-*` runs (B-grid + wedge cells); no `v21c` configs were
created and no Stage-C simulations were executed. The Stage-C
table's D_L/NF values are therefore UNGROUNDED — a record error,
not a measurement. Consequences, honestly drawn:

- The claim "temporal capacity unchanged at the rule-selected
  operating point" is **UNSUPPORTED** (not refuted — unmeasured).
- The frozen selection rule (β=0.00625, τ=5 s) stands as a rule;
  its Stage-C application must be treated as NOT PERFORMED.
- This audit therefore evaluates the information question from
  the artifacts that DO exist (the probe runs), and flags the
  Stage-C rerun as required-but-not-designed work.

Cause: Stage C was recorded without execution in the same commit
batch as Stage B — a process failure on my part, now corrected on
the record. Nothing else in `d8b711b` (Stage A gates, Stage B
map) is affected: those artifacts exist and were re-verified for
this audit.

## 1. What the committed artifacts actually contain (scope limit)

Every v21probe run used the SAME schedule: S0 silence, then 40 ×
antecedent **A** only, then 20 s silence. **No C trials exist in
any committed V2.1 run.** Therefore the headline question "does u
differ after A vs C" is **not directly answerable** from committed
artifacts — no antecedent contrast exists. What IS auditable:
u's dynamics, spatial structure, saturation, and the statistical
character of the endogenous activity. The A-vs-C questions below
are answered as far as the data permit, with explicit un-measurable
items marked.

## 2. A-vs-C information in u (questions 1.1–1.4)

DIRECTLY UNMEASURABLE (no C trials; see §1). What can be said:

- u IS stimulus-coupled (not a free-running oscillator): u mean
  during drive rises monotonically with cumulative A exposure —
  W1 (β=.00625, τ=5 s): 0 → 0.63 → 1.01 → 1.25 → 1.32 (plateau
  ~45 k ms); W2 (β=.003125, τ=10 s): 0 → 0.20 → 0.36 → 0.56 →
  0.67 → 0.76 → 0.82 → 0.83 → 0.86 (still rising at drive end).
  u tracks the organism's own spiking, which is stimulus-driven —
  so u ENCODES an integrated activity history with the configured
  time constant.
- Whether that history is antecedent-DISCRIMINATIVE (A vs C)
  cannot be tested without a C arm. Given the E-series substrate
  facts (A vs C drive different channels → different internal
  subsets fire → u accumulates on different neurons), u WOULD
  plausibly diverge under an A/C contrast — but that is argument,
  not evidence, and this audit does not claim it.

## 3. Information in endogenous spikes (questions 2.1–2.3)

The endogenous activity's statistical character is now measured
(both wedge cells):

- **It is deterministic pacemaking, not structured variability.**
  Per-neuron 1-s Fano factors: W1 — neuron 65: 500.00 sp/s,
  Fano 0.000; 28: 333.35, 0.001; 47: 333.30, 0.001; 70: 333.35,
  0.001; 39: 248.05, 0.068. W2 — 39: 500.00, 0.000; 41: 500.00,
  0.000; 47: 250.00, 0.000; 53: 134.35, 0.511; 65: 100.85, 0.653.
  500.00 sp/s = the refractory ceiling (2 ms refractory ⇒ max
  500 Hz) — these neurons are saturated clocks.
- Population rate is near-constant (W1: 1849 ± 19.8 sp/s over
  20 s; W2: 1485 ± 15.9) with high lag-1 autocorrelation (0.81 /
  0.90): a stationary rhythm, not a decaying or evolving trace.
- Only 6 (W1) / 5 (W2) neurons of 52 non-input are active — the
  endogenous state is carried by a tiny pacemaker core.
- **Transient vs persistent**: persistent (constant through the
  full 20 s; no decay trend).
- **Distinguishable from generic rate elevation**: the activity
  IS generic rate elevation in its purest form — pacemaker
  saturation. There is no time-varying structure that could carry
  sequential information in the spike pattern itself at this
  operating point.

## 4. State saturation / attractor structure (questions 3.1–3.3)

- u saturates during drive: W1 plateaus at mean ≈ 1.32 (by trial
  ~30 of 40) while u decays slightly late in drive (1.318 → 1.161
  by t=84 k) — the steady state is a flux balance (spike input vs
  decay), not an unbounded integrator.
- After stimulus removal the population does NOT relax: u mean
  decays slowly (W1 1.09 → 0.76 over 20 s) but the ACTIVE core
  self-sustains (pacemaking re-injects β). The system sits in a
  genuine bistable-like self-sustained regime.
- **Saturation question (does u converge to the same attractor
  regardless of antecedent?)**: unmeasurable without a C arm; BUT
  the measured structure gives a strong negative indicator for
  discriminability: u's spatial distribution is EXTREMELY
  concentrated and hardening over silence — Gini 0.80 → 0.91 →
  0.93 (W1) and 0.84 → 0.90 → 0.93 (W2); neurons above half-max:
  5 → 5 → 4 and 3 → 3 → 3. The persistent state is a small fixed
  pacemaker set, not a distributed code. Any antecedent-specific
  component would have to survive in ~3–6 neurons against this
  concentration dynamic.
- Adaptation/M6 vs u: net per-spike membrane effect was
  (β − adaptation_gain) = (0.00625 − 0.05) < 0 at W1 — yet
  pacemaking persists because u accumulates over MANY spikes
  while i_adapt saturates per-spike; at 500 Hz the β-flux
  (500 × 0.00625 = 3.1/s) dominates the decay at τ=5 s. M6's
  inhibition is out-competed in the core (spikes continue). So
  adaptation/M6 do NOT dominate u in the wedge — they merely
  bound WHICH neurons join the core.

## 5. Spatial organization (questions 4.1–4.3)

- Information carriers: none demonstrable (no contrast); the
  ACTIVITY carriers are 5–6 neurons, concentrated (Gini > 0.9),
  hardening over time.
- Localized (a pacemaker core), not distributed. Both wedge cells
  show the same architecture: a different but equally tiny core
  (W1 core includes output neurons 65, 70 — output mean u 2.23 vs
  internal 0.88; W2's core is internal-dominated — internal mean
  1.22, output 0.39). Output neurons can join the core but are
  not privileged.
- The hardening trend (Gini rising through silence) means the
  core COMPETES AWY weaker carriers: any sub-threshold
  antecedent-specific u component in non-core neurons decays
  while the core self-reinforces — an actively information-
  destructive dynamic at these operating points.

## 6. Wedge-cell comparison (question 5; observation only, NOT
## parameter selection)

W1 (β=.00625, τ=5 s): 6-neuron core, 3 saturated pacemakers at
refractory ceiling, u plateau reached during drive (fast
dynamics), population 1849 sp/s.
W2 (β=.003125, τ=10 s): 5-neuron core, 3 pacemakers, u still
integrating at drive end (slower dynamics), population 1485 sp/s.
Structural character is THE SAME (tiny pacemaker core, Gini →
0.93, stationary rhythm); they differ in timescale and core
membership, not in kind. No evidence that either carries
representational structure rather than rhythm.

## 7. Reconciliation with Stage C (question 6)

- First: the committed Stage-C table is void (§0 erratum).
- Substantively, the mechanism-level reconciliation: the E21
  D_L−NF metric reads PROBE-EPOCH internal response vectors.
  For u to move that metric, the antecedent-specific u component
  must (i) survive the gap in neurons that then (ii) fire
  differently during the probe. The measured wedge dynamics work
  against both: the persistent state is a hardening pacemaker
  core whose members fire at refractory ceiling REGARDLESS of
  input (saturated clocks have no dynamic range left to signal
  with), while non-core neurons' u decays toward zero. A
  saturated pacemaker is the worst possible information carrier:
  persistent but amplitude-dead. So even with a C arm, the
  prior from these artifacts is that D_L would sit at its noise
  floor — consistent with (though not measured by) the void
  Stage-C table's numbers.

## 8. Interaction audit (question 7; what committed data establish)

- Adaptation: does not stop the core (§4); bounds core
  membership. Interacts via the per-spike sign balance.
- M6: inhibited but out-competed within the core (spikes
  continue at ceiling). Not measurable further from these runs.
- M2: operates on synaptic weights; the pacemaker core's
  self-sustainment is INTRINSIC (u), not recurrent-synaptic —
  M2's budget is not the binding constraint here (contrast
  E17's starved recurrence: intrinsic persistence bypasses it).
- STDP: active throughout (drive + silence); its long-run effect
  on the core's synapses is not reconstructable from these
  artifacts beyond event counts.
- E6 φ: scales plasticity gains only; no measurable interaction
  with u in committed data.

## 9. Strict causality (question 8)

ESTABLISHED: u causes endogenous pacemaking (β=0 ⇒ none, V2
identity; wedge cells ⇒ sustained firing with 0 failures).
NOT ESTABLISHED: u carries antecedent-specific information (no
contrast exists in committed data); u is necessary/sufficient
for ANY information transfer; the pacemaker core would/wouldn't
differentiate under an A/C curriculum.

## 10. VERDICT (question 9)

**Evidence is ambiguous/unresolved — and structurally biased
toward "persistence without information" at the measured
operating points.** What is proven: u creates genuine persistent
endogenous activity (a self-sustaining pacemaker core). What is
unmeasured: any antecedent contrast. What the measured structure
warns: the persistent state is a saturated, concentrating,
dynamic-range-dead rhythm — the kind of persistence that is
actively hostile to carrying discriminative information
(Gini-hardening core at refractory ceiling; non-core u decays).
The honest classification is "u creates persistence; whether it
creates information is untested, with measured structural
priors against it."

## 11. Boundary (question 10)

This audit CAN establish: u's existence, dynamics, saturation,
spatial concentration, the pacemaker character and stationarity
of endogenous activity, and the void status of Stage C.
It CANNOT establish: any A-vs-C discrimination (no C trials
committed), information content of u or of endogenous spikes,
or capacity at any operating point (Stage C never ran).
A future experiment would be required for: an A/C-contrast probe
under V2.1, and/or a real Stage-C gap sweep at the rule-selected
point — designed under the standing no-post-hoc-selection rule
(NOT designed here, per mandate).

## Disposition

The Stage-C erratum is recorded in the validation document; the
selection rule and its intent stand; the negative "unchanged
capacity" claim is retracted as unmeasured. STOP — no next
experiment proposed.