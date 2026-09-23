# sZ Synthesis: self-construction under selection — the full record (E4 → D-49)

Date: 2026-09-23. Consolidates the E4 neurogenesis thread (D-38..D-49)
into one coherent, evidence-based story. All claims measured on
DETERMINISTIC runs (BTreeMap determinism fix, 2c44d7b) unless noted.

## The arc in one paragraph
The organism could grow neurons but the *birth machinery was broken*
(newborns were high-gain sinks that drained recognition). Fixing that
unlocked real neurogenesis-under-selection; successive probing moved the
apparent "structural wall" ever higher (229 -> 244 -> 267 -> 320 ->
434) until the size-scaled synapse budget showed the "wall" was largely
a harness constant, not a biological limit. Growth is viable to ~8x
(55 -> 434 neurons) with recognition that oscillates but never fully
collapses.

## Findings, in order (each committed)
- D-38: homeostatic birth fired but killed at every rate - debugged the
  E4 death bug: newborn = 20-afferent sink into memory-critical partners.
- D-39 (79b385d): bidirectional participatory newborns + wiring_w_scale
  + size-scaled b_e = VIABLE neurogenesis (2/3 seeds survive 67 born).
- D-40/41: evolution grows 3x (40->160) with improving fitness; size
  question answered: bigger brains survive once growth machinery correct.
- Determinism fix (2c44d7b): GA was nondeterministic (HashMap iteration
  in breed) - BTreeMap, byte-identical verified.
- D-45/46: readout-gating + output-inhibition BOTH closed. D-45: the
  collapse ISN'T readout dilution - it's a degenerate-output attractor
  (organism dumps all drive on 2 output neurons). D-46: output-band
  lateral inhibition is a per-seed rescue, net-negative, not selectable.
- D-47: wall mechanism MEASURED = resource-exhaustion (synapses > cap
  20000) at ~231-267, NOT the D-45 attractor.
- D-48: cap raised 20k->40k. HARD wall = budget value (moved to ~347).
  SOFT slope = real population-level recognition dilution above ~320
  (alive, unmonitored, fit<0.4 spread) - a slope, not a wall.
- D-49 (e156f3b): size-scaled cap (k=200 syn/neuron). GROWTH NOT CAPPED
  at ~320: 55 -> 434 (~8x), recognition recovers to .703 past the old
  shelf, persists at 470+ (fit 0.30-0.55 oscillation). ~320 ceiling was
  SUBSTANTIALLY a budget artifact.

## Corrected narrative (vs. earlier in-thread claims)
- The "why is the network too small" question: growth was BROKEN (sink
  birth), then BUDGET-LIMITED (incidental 20k cap), not representational.
- The ~229/~320 "structural wall" was an unregistered harness constant
  (e06bbb3) compounding an initially-wrong attractor diagnosis.
- With budget and machinery correct: viable self-construction to ~8x.

## Measured capability end-state (deterministic)
- Learns without retrain (STDP/plasticity in-loop).
- Perfect known-vs-novel recognition (1.00/0.91/1.00 vs 0.00).
- Closed-loop survival (own output vote -> next stimulus, viability v=a*r*s).
- Heritable neurogenesis: 55 -> 434 neurons over selection, recognition
  never fully collapses, ~8x growth.
- Deterministic per (seed, config); 186 tests green.

## Honest open problems (for future registrations)
1. SOFT SLOPE: above ~350 neurons, population fit oscillates 0.29-0.70
   (alive, unmonitored). Real competition/quantization effect to
   understand - candidate: compartmentalize pools or >350 output-competition.
2. Beyond 434: unmeasured (window ended at 16 gens / ~8x). Further
   growth unknown.
3. Single shared pool: all patterns mix as it grows (E3b verdict), the
   slope is likely this mixing.

## Reproducibility
Every size claim since 2c44d7b is deterministic. Earlier (D-41..D-46)
single-run numbers were one nondeterministic path each; their directional
conclusions replicated across independent runs and stand, but exact sizes
(229/244/267) were path-specific conservative values of the same findings.
STOP.
