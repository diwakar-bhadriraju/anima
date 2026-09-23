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
  never fully collapses in LIVING organisms, ~8x growth (seed 424242,
  k=200 run - single-seed/lineage qualifier applies; cap selectively
  culled high-synapse-demand genomes).
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

## CORRECTIONS LEDGER (what we believed vs. what we measured)
The single most important section for future sessions. Each row = a
conclusion this thread overturned, with the discipline failure that
caused it. Skip these at your own risk - you WILL re-tread the wall.

| # | Claimed (commit) | Measured truth (commit) | Causal error |
|---|---|---|---|
| 1 | ~229 "structural wall" recognition collapse (22b8950, 8710b7d) | Wall = incidental 20k synapse cap (e06bbb3); exhaustion, not recognition; moved 229->267->320->434 as budgets removed (D-47/D-48/D-49) | UNREGISTERED harness constant treated as biological limit; UNINSTRUMENTED inference (no death-cause logged) |
| 2 | "Recognition collapses while alive" (8710b7d) | All 0.00 orgs are MONITOR-DEAD (runaway/exhaustion); alive orgs never fully collapse (380a4d9) | D-47 verbose run NOT set; inferred from a smaller-org telemetry, not the wall size |
| 3 | D-45 "degenerate-output attractor" is the wall mechanism | Attractor exists at small sizes (beat-22, 8 births) but the SGD wall is exhaustion (D-47); attractor attribution to the wall was WRONG | Extrapolated a small-org mechanism to the large-org wall without measuring the wall's death cause |
| 4 | D-41/D-43 growth sizes deterministic | GA was NONDETERMINISTIC (HashMap in breed); single-run sizes were one path each (2c44d7b) | No determinism gate on the GA; HashMap iteration order un-tested |
| 5 | D-46 gene-8 x fitness "no association" (wrong-seed data) | Retracted; evolve IGNORED argv, ran all 3 seeds; '424242' parses were 20260912 (9489cc1) | argv seed filter didn't exist; single-seed runs weren't single-seed |
| 6 | D-49 "P1 confirmed, growth not capped" (e156f3b) | Mixed: k=200 cap re-bound for high-synapse genomes (3 exhaustion + 1 runaway); pure-slope open (380a4d9) | k=200 = ~3% headroom vs measured 180-194 syn/neuron; cap selectively culled heavy-demand genomes |

PATTERN: every reversal came from ONE of (a) unregistered constant,
(b) uninstrumented inference, (c) wrong-seed / argv data, (d) untested
nondeterminism. Discipline that prevents them: REGISTER the constant,
LOG the death cause, VERIFY the seed, GATE determinism.

## Standing caveats (from D-49)
- k=200 cap selects for synapse-efficiency (genome-dependent demand:
  180 vs 275 syn/neuron) - ~8x is lineage-dependent, not universal.
- First GA-path runaway observed at n=328 (310 Hz) under scaled cap -
  runaway monitor becomes the binding constraint if synapse demand drops.
- Pure representational-slope question (cap fully non-binding, k=250-300)
  remains OPEN.
- Soft slope above ~350: population fit oscillates 0.29-0.70 (alive,
  unmonitored) - real competition/quantization effect, not a wall.

## RECOMMENDED: monitor/harness constants audit (before any further run)
Every constant probed this session was mis-set or binding in unintended
ways: the 20k synapse cap (incidental, drove 3 'structural wall'
verdicts), the 50 Hz runaway threshold (false-positived on healthy
58.9 Hz pool), the 280 Hz ceiling (needs margin; first GA-path runaway
at 310 Hz n=328). A one-page audit of ALL ResourceConfig + harness
constants (provenance, measured operating range, registered value)
would prevent a fifth round of budget-artifact reversals. Low priority
but cheap.

## NEXT-STEP OPTIONS (post-D-49 - the old compartmentalize-vs-accept
fork is OBSOLETE; replaced by three evidence-live questions)

(a) PURE-SLOPE with k=300 (cheapest, ~12 min): fully non-binding cap ->
   does the representational slope above ~350 ever become a real wall,
   or does it oscillate forever? Closes the one open quantitative
   question from D-49. Recommend first (cheap, de-risks everything else).

(b) SYNAPSE-DEMAND AS SELECTABLE TRAIT: the k=200 efficiency culling is
   arguably a FEATURE - selection discovered a pressure (genome-dependent
   synapse demand: 180 vs 275/neuron). Register whether the GA converges
   on low-synapse-demand genomes (adaptation to the budget) or maintains
   diversity. Biologically interesting: efficiency as evolved trait.

(c) RUNAWAY AT SCALE: first GA-path runaway observed at n=328 (310 Hz).
   With synapse demand dropping under (b), the runaway monitor becomes
   the nearest binding constraint at large sizes. Understand the
   runaway trigger's behavior above 400 neurons.
STOP.

## Cleanup note (2026-09-23): disk + evidence hygiene
- Reclaimed ~13.4G: removed target/debug (12G build cache) + 628 uncited
  runs/ dirs + emptied .trash-jsonl. Repo 16G -> 2.6G.
- Rescued cited telemetry: e1, e2a, e2b-082426Z telemetry .zst files were
  ONLY in .trash-jsonl (their runs/ dirs had metrics/report/snapshots but
  no telemetry). Restored each into its runs/ dir - cited runs now
  complete. This is the correct pattern: telemetry is evidence, always
  lives with the run.
- DANGLING CITATION: runs/e2b-20260913T082332Z is cited in docs but
  missing on disk (no dir anywhere, pre-existing). Do not chase it.
- All 57 doc-cited run dirs preserved; 186 tests green; tree clean.
