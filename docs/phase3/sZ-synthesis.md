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

## RULES FOR FUTURE DISK CLEANUP (from 2026-09-23 pass)
- BEFORE deleting .trash-jsonl/ (or any archive dir): check it for cited-run telemetry
  FIRST. The e1/e2a/e2b-082426Z runs had telemetry ONLY in trash (their runs/
  dirs held metrics/report/snapshots). Restore telemetry into the cited runs/
  dir, THEN delete the archive. This is now a standing rule, not tribal knowledge.
- A run dir is only 'complete evidence' when it has telemetry + metrics + report
  + snapshots together. Missing telemetry = incomplete, check trash.
- Delete runs/ only after the cited-in-docs check (keep = runs/ strings that name
  existing dirs in ANY docs/ file).

## CORRECTED HEADLINE FACTS (D-55 reconciliation, 2026-09-23)

The D-50..D-54 thread mischaracterized the organism as 'output-
degenerate / can't represent distinct symbols / caps at 2'. That was a
MEASUREMENT-ORDER ARTIFACT: capture_separation scored post-survival
responses against PRE-survival refs (evolve 338 vs 401 vs 426). With
refs matched to the tested (post-survival) state, the corrected facts:

1. TWO-SYMBOL discrimination is GENUINE (post-survival sep=1.000). The
   original survival-loop 1.00/0.91/1.00 was largely HONEST (fresh in-
   loop refs), not self-confirmation (D-53's claim softened).
2. THREE-SYMBOL strain is REAL but PARTIAL: 20260912/9001 separate at
   1.000 post-survival; only 424242 is strained (0.42-0.67). The shared
   pool is NOT capped at 2 - 424242's partial is SEED-SPECIFIC, not a
   shared-pool law.
3. The 'tonic-seizure degenerate output' (D-52) is the S1-ONLY immature
   state; the SURVIVAL LOOP's STDP/growth consolidation induces output
   selectivity over 30+ beats. outprobe (S1-only) measured the immature
   organism.
4. An additive output competition law (D-54) adds nothing - falsified
   on the correct (post-survival) state; survival dynamics already
   provide the selectivity.

=> MUCH MORE OPTIMISTIC AND ACCURATE close: the organism DISCRIMINATES
(2 symbols robustly, 3 symbols for 2/3 seeds). REMAINING OPEN: the
seed-specific 424242 3-symbol strain (0.42-0.67). Untested levers:
longer survival (more selectivity-consolidation beats), seed-specific
growth params, or genuinely structural. This replaces the 'caps at 2'
narrative entirely.

## THE REAL CAPABILITY STATEMENT (post-audit, 2026-09-23 - the session's actual bottom line)

After D-53 (self-confirmation artifact), D-53b (novelty tautology),
D-55 (stale-refs reconciliation), and the corrected uncapped baseline:

WHAT IS REAL (measured, deterministic):
1. GROWTH: heritable neurogenesis is real and uncapped at this scale
   (55 -> 244+ neurons in 8 gens with size-scaled cap; D-49's ~8x).
2a. TWO-SYMBOL known-vs-known discrimination: real post-survival.
2b. THREE-SYMBOL separation: >= 0.7 in ~67% of org-gens (mean 0.81),
    but carried by FRAGILE magnitude-argmax over near-identical refs
    (cos 0.96-1.00) - the 12-dim output codebook provides razor-thin
    margins. (The 67% figure was a 3-symbol measurement; 2-symbol is
    the cleaner/stronger sub-case.)
3. (REMOVED - was a leftover tautology framing that contradicted the
   contamination finding below. The honest novelty statement is item
   'NOT REAL' #1: novelty detection = 0.)

WHAT IS NOT REAL (falsified by honest measurement):
- NOVELTY DETECTION: WEAK-TO-ZERO, DEGRADING WITH GROWTH. The honest
  loop-side metric (gap-separated D-beats, the registered metric's
  source): detected_frac 0.14-0.43 in gen 0, collapsing to 0.00 by
  gen 1 (all D-beats decode to KNOWN symbols). The survival verdict's
  'novel 0.00' was a tautology (recognized requires is_known -> always
  false for D) and proved nothing. The 204-contamination number from
  the earlier probe INCLUDED back-to-back probe-D presentations (no
  gap, aftereffects-biased) - the loop-side number is the honest one
  and it is weak-then-zero, not exactly-zero-at-baseline.
  => the organism does NOT robustly detect novelty, and detection
  DECAYS ACROSS GENERATIONS under current selection. CAUSE UNTESTED:
  two candidates (a) growth consumes/absorbs the detection capacity
  [a growth-cost], (b) fitness is BLIND to novelty detection
  (fitness = viability*known_recog*sep rewards nothing about detection)
  so selection exerts no pressure to retain it (whatever erodes it -
  consolidation, growth, or drift - is not yet identified). The D-56
  fitness-blindness test (flag-gated fitness term)
  distinguishes them: decay stops -> selection-pressure cause; decay
  continues -> growth/codec cause. Do not state 'growth cost' as
  established - it is the correlation; the cause is the D-56 question.
- ROBUST multi-symbol separation: the codebook's margin structure is
  too thin to support more than 2-3 symbols reliably; 3-symbol strain
  (33% of org-gens below 0.7) is real.

TWO DISTINCT MEASURED PROBLEMS (the corrected roadmap):
- CODEC problem: refs too similar (cos~0.99) -> th_known never fires
  NOVEL -> novelty detection ~0. This is the 12-dim count-codebook
  margin structure, upstream of novelty AND capacity.
- POOL problem: 3-symbol strain (33% org-gens < 0.7) - the shared pool
  mixes patterns as it grows (E3b verdict), capacity is strained but
  real in majority org-gens.

THE GOAL, corrected: 'an organism that grows and remembers more as it
grows' currently has (a) growth - real; (b) memory - 2 symbols robustly
in-living-organism, 3 partially, novelty detection absent. The output
codec (a lossy count-collapse over a tonic-seizure-prone output band)
is the upstream blocker for both novelty detection AND robust capacity.
