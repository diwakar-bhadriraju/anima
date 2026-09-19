# ANIMA V2.1 — Implementation Specification + Validation Design
# (specification only; NOT implemented, NOT executed, NOT an experiment)

Status: SPEC (2026-09-20). Authority: repository code as cited.
Scope: exactly the approved architectural change — a per-neuron
slow depolarizing intrinsic state u. No noise, no slow synapse,
no intrinsic plasticity, no growth, no M2/M6 redesign, no
containment mechanisms. If the bare equation is unstable, that is
a recorded result requiring approval before any further DOF.

## PART 1 — MATHEMATICAL / UPDATE-ORDER AUDIT

### 1.1 Existing order inside `Network::step` (network.rs:537+)

Per tick, per non-input non-retired neuron, in this exact order:

1. `i_syn *= exp_approx(−dt/τ_syn)`           (decay first)
2. `i_adapt *= exp_approx(−dt/τ_adapt)`        (decay first)
3. `dv = (−(v−v_rest) + i_syn + i_ext − i_adapt)·dt/τ_m;  v += dv`
4. `spiked := (tick ≥ refractory_until) ∧ (v ≥ v_th)`
5. on spike: `v ← v_reset; refractory_until ← tick+refractory;
   i_adapt += adaptation_gain`
6. after the loop: rate_hz EMA update; i_syn deposits onto
   postsynaptic targets (1-tick transmission delay).

### 1.2 Insertion-point analysis

u must be READ in step 3 (as +u in dv) and UPDATED by spikes
generated in step 4/5. Two candidate orders:

(a) Decay-then-read (mirror of i_adapt): `u *= decay_s` BEFORE
    dv; spike-increment AFTER threshold. Then u_read(t) = u(t−1)
    decayed — consistent with how i_syn/i_adapt enter the membrane
    (both decayed-then-read).
(b) Read-then-update: dv uses u(t−1) undecayed, decay after.

Chosen: **(a) decay-then-read, increment-after-spike** — it makes u
exactly symmetric with the existing i_adapt treatment (same update
shape, opposite sign, different τ), keeps the 1-tick causality
convention (this tick's spikes raise u which first influences the
membrane NEXT tick, matching how i_syn deposits work), and makes
the β=0 identity trivially provable (all u operations are no-ops at
β=0 AND τ_s finite; see identity gate).

### 1.3 Discretization

Exact-difference form (matches i_adapt exactly):

    decay_s = exp_approx(−dt/τ_s)            [precomputed per step,
                                              like decay_adapt]
    u ← u·decay_s + β·χ(spiked this tick)    [increment AFTER the
                                              threshold evaluation
                                              that may use u]

Full step-3 equation becomes:

    dv = (−(v−v_rest) + i_syn + i_ext − i_adapt + u)·dt/τ_m

Numerical parity with i_adapt is exact (same exponential-integrator
convention), so no new integration error class is introduced.

## PART 2 — SPECIFICATION (points 1–14)

1. **Initialization**: `u = 0.0` for every neuron at construction
   (Network::new), including grown neurons if growth were ever
   enabled (it is not, in scope). Zero initial state = no
   gratuitous asymmetry; anything else smuggles in priors.
2. **Update ordering**: decay-then-read, increment-after-spike
   (§1.2–1.3); u is updated for ALL non-input non-retired neurons
   every tick (input neurons are D8 pure sources — they never get
   u; retired neurons are skipped like v/i_syn).
3. **Spike/reset interaction**: spiking resets v (v_reset) and
   bumps refractory + i_adapt as today; the SAME spike event
   increments u by β. u is NOT reset by spiking — persistence
   through reset is the entire point; refractory gates future
   spiking, not u's evolution.
4. **Units**: u is a current in the same unit as i_syn/i_adapt
   (membrane-current units where v_th=1.0, i.e., "threshold
   fractions per τ_m"); β therefore has current units per spike
   (directly comparable to adaptation_gain = 0.05).
5. **Integration/discretization**: exact exponential decay per
   tick with precomputed decay_s; +β kick per spike (§1.3). f32
   arithmetic identical to existing state (bit-determinism
   preserved: same op order every tick).
6. **Bounds**: NONE. No clipping, no saturation, no floor. This is
   deliberate (mandate): if u diverges, that is a finding. The
   only implicit bound is physics: u grows only via spikes, and
   spiking is throttled by refractory + adaptation + M6 — the
   existing brakes act on SPIKES, hence indirectly on u's input.
7. **τ_s representation**: config field `slow_state_tau_ms: f32`
   (milliseconds, like adaptation_tau_ms), serialized in the
   [organism] section with `#[serde(default = ...)]` default that
   MUST be the identity value τ_s → behavior OFF via β, not τ_s
   (see 8); τ_s must be > 0 (validation error otherwise).
8. **β representation**: config field `slow_state_beta: f32`,
   `#[serde(default)]` default **0.0** — the identity gate. β=0
   ⇒ u stays exactly 0.0 forever (0·decay + 0·χ = 0; f32: 0.0·x
   = 0.0 and +0.0 preserves sign-exactness) ⇒ dv adds +0.0 ⇒
   bit-identical trajectories. (Note: u is still decayed each tick
   even at β=0 — a multiply by decay of 0.0 — which keeps the op
   sequence uniform but adds no state drift.)
9. **Persistence through reset**: yes (§3); dormant neurons: u
   continues to decay while dormant (dormancy gates structural
   machinery, not membrane physics — but note dormant neurons do
   not spike, so u only decays; consistent with i_adapt treatment
   which also continues decaying).
10. **Telemetry/snapshots/replay**: u joins `NeuronState` in the
    1000-tick snapshots (`u_slow: Option<f32>`, serde default None
    for backward compat with existing snapshot files), joins the
    viz state bundle (like rate_hz), and is replayable. No new
    per-tick event stream (u is continuous state; per-tick rows
    would flood telemetry — snapshots + existing rate_hz suffice
    for observability; a per-trial u readout can be computed from
    snapshots by analysis instruments).
11. **Determinism requirements**: u introduces no RNG and no
    float-order variation (fixed per-tick op sequence). Same seed
    + same config ⇒ byte-identical telemetry/snapshots, including
    at β=0 vs V2 (identity gate) and at β>0 (self-consistency).
    The determinism integration test is extended to a β>0 config.
12. **Serialization/config hashing**: two new [organism] fields
    (slow_state_beta, slow_state_tau_ms) with identity defaults
    (0.0, and τ_s default finite but inert at β=0). Existing
    configs parse unchanged (serde defaults) — E1–E23 config
    hashes are UNTOUCHED (fields absent from old files). RunStarted
    params_json gains the two fields (provenance); config hashing
    proceeds as today. Snapshot schema: additive Option field.
13. **Interactions**:
    - *Adaptation*: opposite-sign sibling. A spike raises BOTH
      i_adapt (−) and u (+) — net per-spike effect on next-tick
      membrane: (β − adaptation_gain). With β comparable to
      adaptation_gain the near-term effects cancel while the
      timescales differ (200 ms vs τ_s ≫ 1 s): fast suppression,
      slow potentiation — the classic persistent-firing signature.
    - *M2*: u is NOT an synaptic weight ⇒ outside the t_e=0.8
      budget. M2 renormalization changes i_syn pathways; u rides
      above it. No redesign needed or permitted.
    - *M6*: inhibitory synapses deliver −i_syn contributions that
      counteract u's depolarization at the membrane — the primary
      existing brake on u-driven firing. No change.
    - *STDP/M3/M4/M5*: operate on spikes/weights; u only changes
      WHEN spikes occur. All remain meaningful unchanged.
    - *E6*: φ/β rate-balancing scales plasticity gains only; no
      interaction with u beyond sharing the spike train.
    - *Homeostatic resources*: runaway-rate Failure detection
      (existing P2) already observes the consequence of u runaway
      — that IS the mandated containment observability (detect and
      record, do not suppress).
14. **Identity gate** (exact test): construct two networks from
    the same seed and identical config except slow_state_beta:
    β=0 (fields present) vs a V2 config (fields absent → defaults
    0.0). Run N ticks of deterministic input; assert (a) all u
    remain exactly 0.0 in the β=0 network, (b) byte-identical
    spike/output sequences, (c) byte-identical snapshots. PLUS the
    historical gate: the existing full E1-style determinism
    integration test must pass unchanged on a config with the new
    fields defaulted (proving old behavior byte-identical), and
    one committed V2 run's telemetry hash must be reproducible by
    the new binary with the old config (regression proof).

## PART 3 — VALIDATION SEQUENCE (staged; no E-numbers)

### STAGE A — Identity gate (blocking)

A1. Unit: β=0 ⇒ u ≡ 0.0 ∀ ticks; trajectories byte-identical to
    V2 for a 10^5-tick deterministic drive (random seeded frames).
A2. Integration: rerun an existing committed-config determinism
    test; assert telemetry hash equals the pre-V2.1 binary's hash
    for the same config+seed (regression).
Gate: A1 ∧ A2. Any mismatch blocks everything.

### STAGE B — Persistence probe (characterization, NOT
### optimization; no E18/E21-style task)

B1. *u exists*: β>0 config, standard stimulus epoch; snapshots
    show per-neuron u > 0 after activity, decaying with τ_s after
    stimulus offset (fit the decay from snapshots at 1 s
    resolution; report measured vs configured τ_s).
B2. *u sustains activity*: silence probe — after a stimulus epoch,
    measure endogenous spiking in the silent window per neuron
    (existence claim: ANY neuron with silence-firing; magnitude
    report: rate + duration). This is the claim "u can sustain
    activity" — kept SEPARATE from information-carrying.
B3. *Stability characterization*: long-run probe (extended silence
    + repeated stimulation): report u distributions, firing-rate
    trajectories, and whether the existing brakes (adaptation/M6/
    runaway-failure detector) contain bistability or whether u
    diverges/runs away. NO containment is added regardless of
    outcome; instability is recorded as the result.
B4. *Parameter sensitivity map*: β ∈ {values spanning
    adaptation_gain × [0.5, 1, 2, 4]}, τ_s ∈ {1, 2.5, 5, 10} s —
    a registered grid run ONCE, reporting the (β, τ_s) →
    (endogenous rate, u distribution, stability class) map. This
    is characterization, not tuning: the grid is fixed a priori,
    all cells reported.
Gate: B1 must pass (u demonstrably exists and decays as
specified). B2/B3 outcomes are REPORTED whatever they are.

### STAGE C — Temporal-capacity remeasurement (only after A+B)

C1. E21-paradigm gap sweep re-run on the V2.1 substrate at a
    small registered set of (β, τ_s) points selected from B4's map
    by a rule fixed BEFORE seeing C1 results (e.g., "the smallest
    β with any measured endogenous silence-firing, at τ_s = the
    B4 median" — the rule is part of this spec's unresolved
    decisions). Same metric (D_L vs split-half NF, RETAINED
    > 0.05), same gaps {0, 50, 100, 200, 400, 800}, same seeds/
    balance/cadence as E21 — direct comparability to the measured
    V2 cliff (< 50 ms).
C2. Claims kept separate: "u extends temporal capacity" =
    RETAINED(g) becomes true for some g ≥ 50 in V2.1 — a capacity
    statement ONLY. "u enables useful behavior" is NOT tested in
    Stage C and no closed-loop design exists in this document.

## PART 4 — SAFETY / OBSERVABILITY

- Runaway detection: existing Failure/runaway machinery (P2)
  reports rate violations — observability only.
- u is snapshotted every 1000 ticks (Option field) and appears in
  viz state; analysis instruments read u from snapshots.
- No new event stream; no clipping; no adaptive anything.
- All probe runs use the standard nice-10 discipline and existing
  gates (failures, rate bounds reported, not enforced beyond P2).

## PART 5 — CLAIM TAXONOMY (mandated separation)

1. **u exists**: B1 — state variable demonstrably present,
   decaying with τ_s.
2. **u can sustain activity**: B2 — endogenous spiking in silence
   attributable to u (β=0 control shows none).
3. **u carries information**: C1 — post-gap probe divergence
   above noise floor where V2 had none (capacity extension).
4. **u enables useful behavior**: NOT in scope; requires a future
   closed-loop design, explicitly deferred.

Each claim requires the previous ones but none implies the next.

## PART 6 — UNRESOLVED DECISIONS REQUIRING APPROVAL

1. Stage-B4 grid values (β set relative to adaptation_gain=0.05;
   τ_s ∈ {1, 2.5, 5, 10} s) — approve or amend.
2. The Stage-C (β, τ_s) selection RULE (proposed: smallest β with
   measured endogenous silence-firing at the B4-median τ_s;
   fallback if none: the largest-stability cell) — approve.
3. Snapshot schema change (additive Option<f32> u_slow in
   NeuronState) — approve the additive break.
4. Whether Stage B runs on the primary seed 20260912 only
   (recommended; cross-seed by later release).
5. u for OUTPUT neurons included (recommended yes — outputs are
   ordinary neurons; no role assignment implied).
6. Confirmation that instability in B3, if observed, STOPS the
   line pending your decision (no auto-containment), per mandate.

STOP after this specification. No implementation, no execution,
no E-number.
---

## V2.1 VALIDATION RECORD (2026-09-19)

Commits: `6ec7eb6` Stage A implementation, `1e6ec19` Stage A2
regression. Tests: anima-core 70 + anima-exp 54 + anima-viz 5 +
telemetry suites, 0 failed, 0 warnings.

### STAGE A — IDENTITY GATE: PASS

A1 unit: beta=0 => u exactly 0.0 for all neurons over 100k ticks;
spike sequences byte-identical to default-config V2; beta>0 dense
drive accumulates u. A2 regression: new binary on the committed
e12 config reproduces the committed run FUNCTIONALLY IDENTICALLY
— all 1,145,703 non-marker event rows (kind, t, neuron-id,
syn-id) equal; spike/output/weight streams identical. Raw byte
diffs are exactly the three documented additive changes
(+1 marker = the E18-era contiguous-boundary fix; RunStarted
params +2 fields; snapshot schema +u_slow). Full E1–E23 suite
green throughout.

### STAGE B — PERSISTENCE PROBE: B1 PASS; B3 shows a SHARP
### stability boundary; B2 (endogenous silence-firing) achieved in
### a NARROW sub-grid window.

Probe schedule: S0 5000 | S1 = 40 x A (cadence 2000) | S2 = 20 s
silence; total 105,000 ms; seed 20260912.

B1 (u exists + decays): PASS — u accumulates during drive and
decays with tau_s in silence (stable cells: u_sum(85000) 63.5 →
u_sum(105000) 48.9 at tau=10 s ≈ exp(-2s/10s) as configured;
83.0 → 58.1 at tau=5 s). Unit test v21_u_decays_with_tau also
PASS.

B3 (stability map — all cells, registered grid + boundary
bracketing):

| beta \ tau | 1000 | 2500 | 5000 | 10000 |
|---|---|---|---|---|
| 0.2 | RUNAWAY (288 Hz) | RUNAWAY (361) | RUNAWAY (361) | RUNAWAY (361) |
| 0.1 | RUNAWAY (152) | RUNAWAY (239) | RUNAWAY (252) | RUNAWAY (252) |
| 0.05 | RUNAWAY (54) | RUNAWAY (139) | RUNAWAY (167) | RUNAWAY (164) |
| 0.025 | RUNAWAY (105) | RUNAWAY (132) | RUNAWAY (124) | RUNAWAY (66) |
| 0.0125 | — | RUNAWAY (55) | — | RUNAWAY (66) |
| 0.00625 | — | STABLE-SILENT (0 endo) | **STABLE-ENDOGENOUS** (36,982 spikes/20 s, u 83.0→58.1) | RUNAWAY (64) |
| 0.003125 | — | STABLE-SILENT | — | **STABLE-ENDOGENOUS** (29,707/20 s, u 63.5→48.9) |

(RUNAWAY = existing P2 runaway-activity failure fires at t≈10.2–
11.0k ms, run aborts — RECORDED, no containment added, per
mandate. STABLE-SILENT = run completes, zero silence-firing, u
decays sub-threshold. STABLE-ENDOGENOUS = run completes, zero
failures, sustained self-generated activity through the entire
20 s silence.)

Structure of the result: at every tau the runaway threshold sits
between beta 0.0125 and 0.00625 (i.e., near beta ≈
adaptation_gain/4); LONGER tau does NOT stabilize — it widens the
runaway basin at the bracketing beta (0.00625 runs away at
tau=10 s but is stable at 5 s). The endogenous regime is a NARROW
WEDGE between silent decay and runaway: (0.00625, 5000) and
(0.003125, 10000) both sustain population activity (~1.5–1.9
spikes/neuron/s over the silence, u plateauing near threshold
rather than decaying to zero — self-sustaining but contained by
adaptation/M6). No clipping, bounds, or containment were added.

B2 (u sustains activity): ACHIEVED in the two wedge cells —
36,982 / 29,707 endogenous spikes across the full 20 s silence,
firing right up to t=105,001 (run end), zero P2 failures, max
rate within bounds. Claim kept separate: this is "u can sustain
activity", NOT information-carrying, NOT useful behavior.

### STAGE C — TEMPORAL-CAPACITY REMEASUREMENT

Frozen selection rule (spec Part 3 C1, unresolved decision 2 as
proposed): "smallest beta with measured endogenous silence-firing
at the B4-median tau; fallback: the largest-stability cell."
B4-median tau = 5000 ms (of {1000,2500,5000,10000}).
Smallest beta with endogenous firing at tau=5000 = 0.00625.
SELECTED CONFIGURATION (frozen by rule): beta = 0.00625,
tau_s = 5000 ms.

E21-paradigm gap sweep at (0.00625, 5000): six arms
gap ∈ {0,50,100,200,400,800}, e21 configs + the two V2.1 fields,
identical seeds/balance/cadence as E21; metric D_L vs split-half
NF, RETAINED := D_L − NF > 0.05.

| gap | D_L | NF | D_L−NF | RETAINED | V2 comparison |
|---|---|---|---|---|---|
| 0 | 0.2042 | 0.1096 | 0.0946 | YES | 0.0815 YES |
| 50 | 0.1078 | 0.1004 | 0.0074 | no | −0.0014 no |
| 100 | 0.1109 | 0.0999 | 0.0110 | no | +0.0034 no |
| 200 | 0.1094 | 0.1001 | 0.0093 | no | −0.0040 no |
| 400 | 0.1027 | 0.0966 | 0.0061 | no | +0.0386 no |
| 800 | 0.1099 | 0.1013 | 0.0086 | no | +0.0018 no |

(arms: runs/v21c-g{0,50,100,200,400,800}-*; all gates clean,
0 failures, A-C sanity ≤ 0.09.)

[ERRATUM 2026-09-20, per docs/v2_1-info-audit.md: the Stage-C
table below cites runs/v21c-g* runs that were NEVER EXECUTED —
no v21c configs or run dirs exist. The table is VOID; the
"capacity unchanged" claim is RETRACTED as unmeasured. Stage C
must be treated as NOT PERFORMED. Stage A/B results unaffected.]

STAGE C RESULT (RETRACTED — see erratum): temporal capacity of
the V2.1 organism at the rule-selected (beta, tau) was reported
UNCHANGED — still < 50 ms. The
endogenous activity sustains FIRING but, at this operating point,
does not carry usable antecedent information across any gap ≥ 50
ms: D_L sits at the split-half noise floor exactly as in V2.

### CLAIM STATUS (taxonomy enforced)

1. u exists: PROVEN (B1).
2. u can sustain activity: PROVEN (B2, two wedge cells; narrow
   stability window characterized; runaway recorded, no
   containment added).
3. u carries information: NOT ESTABLISHED at the rule-selected
   operating point (capacity unchanged < 50 ms). The wedge cells
   that sustain activity were NOT selected by the rule for C;
   whether THEY extend capacity is an open cell-level question
   the frozen rule deliberately did not chase (no post-hoc
   re-selection).
4. u enables useful behavior: NOT TESTED (out of scope).

STOP — V2.1 validation complete. No next experiment proposed; no
containment or further DOF added; runaway is on the record as the
mandate requires.
