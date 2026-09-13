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
| U3 | Structural growth trigger (neuron birth) | RED/YELLOW: adult neurogenesis restricted in biology; computational analogues are our invention | (a) homeostatic saturation, (b) persistent prediction error, (c) representational interference, (d) novelty-gated, (e) none | (a) births under load but may chase runaway; (b) births at learnable-frontier; (e) baseline — growth may never help | E4/E4c homeostatic-saturation + co-active-avoidance wiring: REGRESSION in BOTH active-cadence arms — failure at the SAME sim time t≈540 s with 43 (uncontrolled) vs 35 (3 s cooldown) births; only the inert 2-birth arm (30 s cooldown) survives. Instability is allocation-shape-dependent (20-in/0-out high-gain sink newborns inflate the population mean), not cadence-dependent. E4b proved cadence control alone stabilizes iff growth barely happens. Next (user-gated): fan-out-matched growth shape, persistent-error trigger, or guard recalibration for growth arms | tested (homeostatic) |
| U4 | Learning gate ("should this experience write?") | YELLOW: neuromodulators gate plasticity (dopamine/ACh evidence); circuit details debated | (a) always-on, (b) novelty-gated, (c) prediction-error-gated, (d) global modulator scalar | gates should reduce interference between patterns vs always-on | (a) in E1; (b)–(d) in E5 | open |
| U5 | What a memory is here | RED: our choice; GREEN-adjacent: attractor/replay phenomena exist | (a) strengthened synapses only, (b) recurrent assemblies, (c) attractor states, (d) dormant-reactivable traces | (b)/(c) predict replay-driven reactivation and retention without re-presentation | E7 | open |
| U6 | Internal novelty/unknown signal | YELLOW: hippocampal novelty signals exist; computation debated | (a) external instrumentation-computed (E1), (b) learned internal mismatch signal | (a) is measurement not mechanism; (b) should correlate with (a) if internalization works | E1 uses (a); internalization in E5 | open |
| U7 | Reward/motivation architecture | YELLOW | scalar modulator field over synapse updates | modulated STDP should speed acquisition of repeated input→output paths | deferred to E6; schema reserves `modulator` field | open |
| U8 | Dormancy/retirement semantics | YELLOW: synaptic pruning GREEN-adjacent; neuron dormancy is our construct | thresholds on sustained rate + age | dormant neurons' synapses should be prunable without retention loss if redundancy exists | E4 | open |

## Unknown unknowns

(empty — appended as discovered, with the experiment that exposed them)
