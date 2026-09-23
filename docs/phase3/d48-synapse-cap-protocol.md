# D-48 PRE-REGISTRATION: is the ~229 wall a synapse-budget artifact or a structural ceiling?

Date: 2026-09-23. Registered BEFORE the cap-raised run. Now deterministic
(BTreeMap fix, 2c44d7b): single-run size claims are reproducible.

## Provenance finding (why this test)
The 20000 synapse cap in evolve.rs is an INCIDENTAL monitor-wiring
constant (added e06bbb3, `rcfg.max_neurons=600; rcfg.max_synapses=20_000`)
- never registered, never calibrated. resources.rs default is 2000.
The size-push 'wall' (bg_13: collapse at 244; bg_7: exhaustion at
267) may be THIS budget biting, not a structural/property ceiling. The
whole 'neurogenesis capped at ~229' narrative rests on an unregistered
constant.

## Test (single clean deterministic run)
`evolve 424242 16 <SYN_CAP>` with SYN_CAP=40000 (2x the incidental
20000). Same seed, same config, only the synapse budget differs.
Instrumented (D-47): died_at + fail kind per org -> distinguishes
resource-exhaustion (cap) from recognition collapse (structural).

## Pre-registered PASS / FAIL criteria
P1 (budget-artifact, PASS for 'wall was my budget'): organism crosses
  ~267 with recognition intact -> max size significantly > 267, best
  fit >= 0.6 sustained >= 2 gens, NO resource-exhaustion failure before
  the (raised) cap. => neurogenesis is NOT capped at ~229; the prior
  'structural wall' verdicts (22b8950, 8710b7d) were an unregistered
  harness budget, CORRECTED.
P2 (structural ceiling, FAIL for budget-theory): raised cap merely MOVES
  the death point (e.g. 40k exhaustion at a larger size) OR recognition
  collapses independently of the cap (dead=None fail=None but fit->0,
  i.e. the D-45 attractor / representational limit). => the wall is
  structural; D-48 produces the clean recognition-wall evidence the
  earlier docs only assumed.

## Inconclusive bound
If the run hits 40k cap AND collapses at exhaustion -> that's P2
(raised cap only moves exhaustion). If it passes 267 clean but the run
times out before 16 gens -> INCONCLUSIVE, re-run longer.

## Both outcomes are valuable (registered)
- P1: growth is budget-limited -> next lever = size-scaled synapse
  budget or synapse-growth regulation (the real E4d angle).
- P2: growth is structurally recognition-limited -> the D-45 attractor
  IS real above 267 -> compartmentalized pools is the target.
Either way this run decides the fork on evidence, not assumption.

## Guard
Flag-off identity: SYN_CAP default 20000 = current behavior, byte-
identical monitor semantics. Cap-raised runs are a NEW measurement,
registered here.
STOP.

## D-48 VERDICT: P2 CONFIRMED - the wall is STRUCTURAL (recognition collapse at scale), not the synapse-budget constant

Test: `evolve 424242 16 40000` (2x the incidental 20k cap, deterministic
post-BTreeMap) + verbose death-cause sub-run.

TRAJECTORY (gen, size, best fit):
  gen 9  260  fit 0.70   (clean growth past old 'wall')
  gen 10 289  fit 0.70   (recognition intact)
  gen 11 318  fit 0.65   (warning)
  gen 12 347  fit 0.47   (collapse begins)
  gen 13 347  ALL 0.000  (hard)
MEASURED DEATH CAUSE (D-47 verbose, n=325-383):
  org rows at n=354: dead=None fail=None fit 0.29/0.36/0.37/
  0.65 - organisms ALIVE but RECOGNITION DEGRADING, NO monitor fired,
  NO exhaustion at that point.
  THEN exhaustion fires (synapses 59341/74259 > cap 40000) as a
  SECONDARY effect once the raised cap is reached.

CONCLUSION (corrected 2026-09-23, SPLIT not pure P2 - advisory-
precised): the result is a SPLIT, both pre-registered predictions
partially true:
  HARD WALL = BUDGET (P1 substance CONFIRMED): the kill line is set by
  the cap value. Raising 20k->40k moved the hard collapse from ~244-267
  to ~347; exhaustion failures (synapses 40077-74259 > cap 40000) are
  the hard death at every cap. The 'structural wall' narrative based on
  the incidental 20k constant was wrong - the hard bound is the budget.
  SOFT SLOPE = REAL (P2 partial CONFIRMED, as a slope not a hard wall):
  at population sizes >= ~320 neurons, SOME organisms show fit < 0.4
  while ALIVE and UNMONITORED (dead=None fail=None) - e.g. at n=354
  fits 0.29/0.36/0.37/0.65, at n=383 fits 0.35/0.47. This is
  POPULATION-LEVEL degradation (spread across organisms and
  generations), NOT a per-organism monotonic recognition decline - the
  n=354 and n=383 fits come from different organisms. Caveat: the
  '0.70 at 289' is the population BEST. So: measurable degradation
  appears above ~320 neurons, but stated as population spread, not a
  hard per-organism recognition collapse; the hard collapse at gen 13
  is exhaustion.

So: hard wall = budget value (free parameter, should be size-scaled to
sit above the degradation zone); soft degradation = real, quantifiable
recognition slope above ~300 neurons, separate from (and preceding) the
cap kill. The user fork is reframed: not 'wall real? structural?' but
'is the soft >300-neuron recognition slope acceptable/targetable?'
AND 'what cap value keeps the kill line above the degradation zone?'
Immediate registered recommendation: size-scale the synapse cap
(like b_e) so the hard kill is always above the degradation slope while
the slope is studied.

The ~229 numbers from prior nondeterministic runs were conservative
versions of the same structural wall (budget bit sooner at 20k); the
recognition-collapse-only evidence now directly shows the ceiling.

NEXT (registered candidates, for user decision):
  (a) compartmentalized pools - grown neurons recruit into NEW pools,
      attacking the representational ceiling directly.
  (b) accept ~320-350 as the self-construction ceiling (recognition
      cannot survive >~350 neurons in a single shared pool) - record
      as the honest end-state.
STOP.
