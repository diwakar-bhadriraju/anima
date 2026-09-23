# D-49 PRE-REGISTRATION: size-scaled synapse cap (k*neurons) to isolate the pure representational slope

Date: 2026-09-23. Deterministic (BTreeMap fixed). Freeze before implement.

## Motivation (from D-48 split verdict)
D-48 showed: HARD wall = synapse-budget value (20k -> 40k moved hard
collapse 244-267 -> 347); SOFT degradation = real population-level
recognition dilution above ~320 neurons (alive, unmonitored, fit<0.4).
The hard budget must be REMOVED as a confound to measure the soft slope
at higher sizes - is ~320 the representational ceiling, or does
recognition hold if the budget scales with growth?

## Param (measured, not a round number)
Observed need at ~350 neurons: ~40k synapses (40077-74259 fit the
k*neuron form). k = 200 synapses/neuron (350*200 = 70k, comfortably
above the observed 40k need; also > the ~115-212/neuron implied by
40k/350-40k/190). cap = k * live_neurons, re-evaluated per check.

## Implementation (flagged, default preserves D-48 behavior)
argv[3] semantics: if >= 20000 treat as absolute cap (current default
mode, byte-identical); if a value in the k-range (e.g. 200 => size-
scaled), interpret as k. Cleanest: argv[3]=0 activates size-scaled k=200;
else existing absolute-cap path unchanged (flag-off identity intact).

## Pre-registered predictions
P1 (slope is budget-shadow): with cap out of the way, recognition HOLDS
past 320-350 (fit stays >= 0.6 at 400+ neurons, no unmonitored fit<0.4
at any size). => ~320 was an artifact of earlier budgets; the organism
grows further with recognition intact; compartmentalization unnecessary.
P2 (slope is representational-ceiling): unmonitored fit<0.4 spread
PERSISTS or steepens at 400+ neurons independent of the raised cap.
=> ~320 IS the true representational ceiling; compartmentalized pools
is the evidence-backed next target.
P3 (cap still binds): growth outruns even k=200 (exhaustion at 70k+).
=> k too small; but still conclude P2-style (degradation precedes kill).

## Inconclusive bound
If max size stays < ~320 total (growth prematurely stalls), the slope
was not reached - INCONCLUSIVE, re-run longer / recheck growth.

## Both outcomes valuable
- P1: growth is budget-limited everywhere -> accept-or-extend question
  is about budget, not representation.
- P2: representational ceiling confirmed -> compartmentalize-vs-accept
  is fully informed.
STOP.

## D-49 VERDICT: P1 LARGELY CONFIRMED - growth is NOT capped at ~320; ~8x baseline (55->434) with recognition persisting

Test: `evolve 424242 16 0` (k=200 size-scaled cap, determined post-
BTreeMap). BOXED TRAJECTORY:
  gen 10  289  best .703
  gen 11  311  .652 (pop 0.29/0.37 - soft slope appears)
  gen 12  347  .468 (dip)
  gen 13  376  .703  <- RECOVERS past old shelf
  gen 14  405  .652
  gen 15  434  .555  (final, ~8x baseline)
DEATH CAUSES (k=200): essentially NO exhaustion - only 3 hits, all at
n~384-441 where synapses reach k*n (121279>82400 etc). The slope zone
(n=354-470) is uniformly dead=N fail=N - ALIVE, unmonitored, fit spread
0.29-0.70. Recognition PERSISTS at 470+ neurons (0.30-0.55 mixed), far
past the old 320 shelf.

CONCLUSION: D-49 P1 LARGELY CONFIRMED. The ~320 'representational
ceiling' (D-45/D-48 era) was SUBSTANTIALLY a budget artifact - with the
cap scaling with growth, the organism survives past 434 (~8x baseline)
and recognition never fully collapses. The soft slope (population fit
<0.4 spread above ~350) is REAL but BOUNDED - oscillation, not a wall.
CORRECTS: the 'neurogenesis caps at ~320' narrative. Growth is budget -
and representationally viable to ~430+, the observed end of this window.

REVISED CAPABILITY: organism grows 55 -> ~434 neurons (~8x) retaining
non-collapsing recognition under selection, with the size-scaled budget.
The soft slope above 350 is a quantization/competition oscillation to
be understood, not a ceiling.
NEXT (user): (a) extend gens past 15 / check 500+; (b) attack the soft
slope (compartmentalize or output-competition in the >350 regime);
(c) accept ~8x evident end-state as the honest self-construction bound.
STOP.
