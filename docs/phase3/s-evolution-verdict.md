# Phase III sZ — full evolution verdict: viable neurogenesis under selection (final)

Status: PASS. 2026-09-22. Complete rewrite of the sZ verdict - D-39/D-40
supersede D-33..D-35 (which were invalid: size-reshuffle bug + missing
self-construction + absolute budget cap).

## The E4 bug chain (all fixed, each evidence-driven)

1. D-38 gate: homeostatic birth at ALL trigger rates killed the organism
   (died beat 9, r=0, activity normal). Measured: newborn = high-gain SINK
   (20 afferents into memory-critical co-active partners, ZERO efferents)
   -> drained the assemblies -> recognition collapsed.
2. D-39 fix: wiring_bidirectional=true (newborn returns reciprocal
   efferents) -> participates, not drains. Plus wiring_w_scale (gentle
   newborn weights) + accumulation_input_current (was zeroing delta_perm)
   + V2Plasticity::on_neuron_appended (birth OOB-panicked V2 bookkeeping
   + incremental live recount - O(N) full recount went quadratic).
3. D-40 fix: b_e must SCALE with organism size (k*n_neurons), not be an
   absolute cap - absolute caps trip the M5 assert as the organism grows
   past them (panicked at 127+ neurons).

## Result (3 seeds x 8 gens, exit=0, no crash)

  seed       size g0->g7    fit g0->g7    best g7
  20260912   52.5 -> 157.0  0.507 -> 0.628  0.667
  9001       51.2 -> 159.5  0.388 -> 0.454  0.679
  424242     55.0 -> 160.0  0.511 -> 0.598  0.703

## Verdict (the size question, answered)

With the correct birth rule (participatory newborn + gentle wiring +
size-scaled budget), organisms under selection GROW ~3x (52-55 ->
157-160 neurons) across 8 generations, every seed, and fitness IMPROVES
or holds while growing. The user's original intuition was right: the
network was "too small" not because 40 neurons is a fundamental limit,
but because the growth machinery was BROKEN (sink newborns dragged the
assemblies down). Fixed, the organism self-builds larger brains and
those survive selection.

Demonstrated end-to-end: experience learning (no retrain), perfect
known/novel recognition, closed-loop persistence, SELF-CONSTRUCTION
(M3 + births), and heritable growth under evolutionary selection. The
40-neuron platform is now a growing one - the LLM-alternative "learns
by experience, grows, and retains" claim is, in miniature, WORKING.

## Honest limits

- Survival runner still skips the harness runaway/failure detector
  per-tick (recorded earlier); death-by-instability during the loop is
  covered by the in-loop sustained-activity bounds.
- Growth cost: 200-beat life ~22s, full 3-seed x 8-gen ~6 min - fine
  for this scale.
- Birth trigger: homeostatic-saturation at fixed params (rate 30,
  sustained 1500, cooldown 3000) was NOT searched by the GA (they're
  fixed in this run; the GA searched w_scale/b_e/theta/delta). Searching
  the full birth-law space is the natural next registration.
STOP - verdict recorded; neurogenesis under evolution WORKS.
