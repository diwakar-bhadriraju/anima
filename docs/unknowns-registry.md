# ANIMA Unknowns Registry

Mechanism-level neuroscience unknowns are NOT silently decided. Each is
registered with candidates, predictions, and the experiment that resolves it.
Status: `open` → `addressed` (experiment run) → `re-registered` (new question).

Biology status: GREEN = robust evidence; YELLOW = exists but debated or
partial; RED = restricted in biology / our invention.

| # | Unknown | Biology status | Candidate mechanisms | Predictions | Resolved by | Status |
|---|---------|----------------|----------------------|-------------|-------------|--------|
| U1 | Neuron dynamics beyond bare LIF (adaptation, inhibition) | GREEN: spike-frequency adaptation, refractory periods robust. YELLOW: which currents matter at our scale | (a) bare LIF+refractory, (b) LIF+adaptive threshold, (c) LIF+adaptation current; lateral inhibition circuits | (b)/(c) bound firing rates, reduce runaway, change assembly stability vs (a) | E1 uses (a)+fixed refractory 2 ms; E3 tested (c): stabilization supported; E3b tested lateral inhibition on the E3 base: guards pass (stable, coherent, retained) but separation moved AWAY (cross-cosine 0.675→0.740) — same-tick competition cannot fix connectivity-level overlap; next candidates structural (U3 growth) or gating (U4) | partially tested (currents) |
| U2 | Synaptic plasticity rule | GREEN: timing-dependent LTP/LTD (Bi & Poo). YELLOW: window shapes, multiplicative vs additive bounds, triplet terms | (a) pairwise trace STDP, (b) multiplicative-bound STDP, (c) triplet STDP, (d) reward-modulated STDP | multiplicative bounds prevent saturation; triplets capture burst structure | E1 uses (a) additive; (b)/(c) A/B in E2; (d) in E6 | open |
| U3 | Structural growth trigger (neuron birth) | RED/YELLOW: adult neurogenesis restricted in biology; computational analogues are our invention | (a) homeostatic saturation, (b) persistent prediction error, (c) representational interference, (d) novelty-gated, (e) none | (a) births under load but may chase runaway; (b) births at learnable-frontier; (e) baseline — growth may never help | E4/E4c homeostatic-saturation + co-active-avoidance wiring: REGRESSION in BOTH active-cadence arms; E4d fan-out-matched (bidirectional) variant failed the CALIBRATION GATE — runaway at t=32 s after 4 births (mean 179 Hz), 17× faster than the sink shape: reciprocal newborn↔partner excitation is a positive-feedback amplifier in this all-excitatory organism. U3 class closed for both growth shapes under the unchanged 50 Hz guard; allocation hypothesis untestable this way. Next (user-gated): low-fan-in capacity growth, persistent-error trigger, or E5 gates. E4e (fan-in 4, no feedback): calibration gate — STABLE (first shape without regression) but INERT (2 births/240 s, below the ≥3 bar → no full run; projected ~6 < 15 informativeness floor) ⇒ INCONCLUSIVE-inert. Drive budget now bracketed: fan-in 4 peters out, fan-in 20+ regresses; a mid fan-in ∈ {8,12,16} sweep needs a new pre-registration (no tuning on results) — failure at the SAME sim time t≈540 s with 43 (uncontrolled) vs 35 (3 s cooldown) births; only the inert 2-birth arm (30 s cooldown) survives. Instability is allocation-shape-dependent (20-in/0-out high-gain sink newborns inflate the population mean), not cadence-dependent. E4b proved cadence control alone stabilizes iff growth barely happens. Next (user-gated): fan-out-matched growth shape, persistent-error trigger, or guard recalibration for growth arms | tested (homeostatic-saturation AND persistent-error): BOTH closed as untestable — 6 homeostatic arms inert/regressive; persistent-error structurally un-fireable as registered (EWMA horizon 5000 ms == sustained_ms 5000 ms ⇒ bound cannot be exceeded 5 s; A11/A12 fixed dead latch + pe_mean wiring first). Fan-in drive gap: ≤16 peters out, 20 regresses. E5 gates remain excluded; next = user decision |
| U4 | Learning gate ("should this experience write?") | YELLOW: neuromodulators gate plasticity (dopamine/ACh evidence); circuit details debated | (a) always-on, (b) novelty-gated, (c) prediction-error-gated, (d) global modulator scalar | gates should reduce interference between patterns vs always-on | (a) in E1; (b)–(d) in E5 | open |
| U5 | What a memory is here | RED: our choice; GREEN-adjacent: attractor/replay phenomena exist | (a) strengthened synapses only, (b) recurrent assemblies, (c) attractor states, (d) dormant-reactivable traces | (b)/(c) predict replay-driven reactivation and retention without re-presentation | E7 | open |
| U6 | Internal novelty/unknown signal | YELLOW: hippocampal novelty signals exist; computation debated | (a) external instrumentation-computed (E1), (b) learned internal mismatch signal | (a) is measurement not mechanism; (b) should correlate with (a) if internalization works | E1 uses (a); internalization in E5 | open |
| U7 | Reward/motivation architecture | YELLOW | scalar modulator field over synapse updates | modulated STDP should speed acquisition of repeated input→output paths | deferred to E6; schema reserves `modulator` field | open |
| U8 | Dormancy/retirement semantics | YELLOW: synaptic pruning GREEN-adjacent; neuron dormancy is our construct | thresholds on sustained rate + age | dormant neurons' synapses should be prunable without retention loss if redundancy exists | E4 | open |

## Unknown unknowns

(empty — appended as discovered, with the experiment that exposed them)

## v3 addition (2026-09-14, exposed by the overlap curriculum)

**Newly-discovered unknown — U9: feature-selection under overlapping
evidence.** With shared channels dominating co-activation statistics,
additive STDP + M2 conservation (T_e) allocate every neuron's budget
to the shared evidence and the organism NEVER recruits the
category-exclusive channels (v3-full: 0/21 specialized neurons use
channels 0-3 or 12-15; signatures {A+B}/{B+C} only; no pure
{A}/{B}/{C}). Outcome C verdict: separation collapses (selectivity
0.168), statistics are seed-dependent (P4 fails). The v2 mechanism
set has no veto for "channel fires at half the rate but carries all
the information". Candidates (unregistered; would need a new
amendment/protocol): (a) input-side decorrelation (pre-normalization
by channel firing rate), (b) M2 variant over co-activation-scaled
weights, (c) a novelty/information-gated plasticity (U4-class). No
candidate chosen; no mechanism added; status re-registered-open.

### U9 update (2026-09-15)

Candidate selected and approved by user: **plasticity-event rate
balancing (E6: scale STDP a⁺ and M3 Δ_perm by β = φ̄/φ_pre, local
per-channel rate EMA)** — docs/anima-e6-protocol.md. U7
(reward-modulated STDP) re-deferred: the E6 designation now names
the rate-balancing experiment per user direction; reward remains
open (placeholder E7).

### U7 update (2026-09-15)

E6 designation reassigned (see above); reward-modulated STDP
placeholder re-deferred to a future E7. No mechanism work done.

### U9 update (2026-09-14, E6 result)

E6 (plasticity-event rate balancing, construction B) executed:
exclusive-evidence feature selection RESTORED on the overlap
curriculum (7/40 exclusive users, pure {A}/{C} signatures; A-B
cross 0.697→0.078; C1 control proves the mechanism preserves the
disjoint regime, selectivity 0.993). Primary separation hypothesis
H6b/H6c NOT supported: B (⊆ A∪C, no private channels) cannot
separate from C via rate balancing alone — B-C cross 0.792;
selectivity 0.116; P4 seed-variable in magnitude. Conclusion:
per-event rate balancing is necessary-but-insufficient for
separation under overlap; the residual requirement is
absence-gating (a veto for shared evidence when exclusive evidence
is absent) — registered as an open mechanism question, no
amendment proposed.

### U9 / absence hypothesis update (2026-09-16, E7 result)

E7 (minimal absence experiment, frozen E6 organism, curriculum-
only; commits 5254369 + 49fc8ef incl. amendments A-1/A-2):
**OUTCOME A** — P (matched presence), A (absence: Y = {0-7} ⊂ X =
{0-15}, registered 2:1 residual), B (bridge: same residual,
presence) ALL PASS with the pre-registered L1 attribution (A: cross
0.413 raw = normalized, selectivity 0.972; cross-seed reproducible,
|dH| <= 0.067). The absence/disconfirmation hypothesis is NOT
supported at the minimal 2-category scale: the frozen organism
separates subset-from-superset categories. Registered scope
boundary: 2-way absence tasks are solvable by presence-detector +
complement default; the 3-way overlapping disambiguation that E6
outcome C exposed (absence-veto/context) remains untested.
Hypothesis surviving form: absence-DISAMBIGUATION among overlapping
categories, not absence-detection per se. No mechanism change.

### U9 / 3-way disambiguation update (2026-09-16, E8 registration)

E7 rejected the broad absence hypothesis (OUTCOME A). E8 is now
REGISTERED (docs/anima-e8-protocol.md) as a scale/no-D/cross-seed
robustness probe of E6-full's observed B-entanglement (A-B 0.078,
B-C 0.792, A-C 0.046): same v3 geometry, 60 reps, S0/S1/S3 only,
seeds 20260912/9001/424242, pairwise-verdict-table endpoint,
B-alignment readout. Registered geometric proof: a matched
private-evidence control for B is inexpressible (any 8-channel
balanced middle category ⊆ A∪C has zero private channels); the
internal pairwise trichotomy is the attribution. No mechanism
change; no absence gating.

### U9 / 3-way disambiguation update (2026-09-16, E8 result)

E8 (60 reps, no S2/D, frozen E6 organism; commits c4e89f4 +
execution): OUTCOME B (scale/regime dependence). Canonical seed
20260912 reproduces E6-full's verdict table exactly (A-B 0.073,
B-C 0.873, A-C 0.032 vs E6-full 0.078/0.792/0.046; L1-invariant).
Cross-seed 9001 = all pairs separated (0.592/0.165/0.048);
424242 = A-B entangled (0.763), B-C separated (0.075). Robust
core across ALL seeds: the private-evidence pair A-C separates
(<= 0.048); B never stands alone (min across both neighbors >=
0.165; entangled >= 0.763 in 2/3 seeds). B's coalition partner is
seed-labile. Interpretation: the 3-way limitation is real and
scale/no-D-robust on the reference seed, but its expression
(which neighbor absorbs the middle category) is stochastic;
A-C separability is universal. No mechanism implied, no changes.

### U9 / 3-way disambiguation update (2026-09-16, E9 registration)

E8 closed OUTCOME B (seed-labile B-coalition; A-C universally
separate). E9 is now REGISTERED (docs/anima-e9-protocol.md):
minimal temporal/contextual probe on the frozen E6 organism —
SEQ-B: B = {4-7}@40 Hz [0,250) then {8-11}@40 Hz [250,500) (marginal
20 Hz, totals and phi/beta matched vs static A/C), REV-B control
(reversed order, isolates direction), E8-static baseline. Endpoint:
B-independence = A-B < 0.60 AND B-C < 0.60 with L1 attribution;
cross-seed verdict reproduction on SEQ-B (9001/424242, P2+gated).
Derivation registered: readout limit (count-based), STDP tau 20 ms
(separation > ~100 ms), M3 100-ms windows (1/5 co-active windows
for B's groups), E6 phi/beta matched marginals. No mechanism work;
outcome A is a curriculum-level capability result only.

### U9 / temporal/contextual disambiguation update (2026-09-16, E9 result)

E9 (phase-sequenced B, frozen E6 organism; commit dbdc329 +
execution): OUTCOME B with recorded seed dependence. The temporal
cue is REAL and order-direction-sensitive (SEQ B-C 0.083 vs REV
B-C 0.819; trailing side absorbs B); A-C universally separated
(<= 0.019); selectivity doubled (0.70-0.87 vs E8 0.19-0.50);
B-independence achieved in 1/3 seeds (424242: 0.512/0.079),
missed by 0.030 on the canonical (0.630). Conclusion: curriculum-
level temporal capability exists but is insufficient/not robust
for independent B at the registered scale; no mechanism
implemented or implied.
