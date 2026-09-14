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
