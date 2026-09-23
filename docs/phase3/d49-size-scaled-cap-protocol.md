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

CONCLUSION (amended post-review): MIXED - growth past ~320 is REAL but
the k=200 cap SELECTIVELY RE-BOUND, so this run does NOT answer the
pure-slope question cleanly. Verified facts:
- Alive organisms NEVER fully collapse to fit 0.00 - every 0.000 slot in
  gens 13-15 is a MONITOR-DEAD org (runaway 310Hz at n=328; exhaustion
  at n=384/412/441). So 'recognition never fully collapses' for LIVING
  organisms is TRUE.
- BUT the cap re-bound for HIGH-SYNAPSE-DEMAND genomes: 3 exhaustion
  deaths (77185>76800 at n=384 [barely, 0.5%], 121279>82400 at n=412,
  124736>88200 at n=441) + 1 runaway (310Hz, n=328). The k=200 budget
  (~3% headroom at n=383, matching the advisor's warning) culls
  organisms whose synapse demand exceeds ~200/neuron.
- FINDING (new): synapse demand is GENOME-DEPENDENT (275/neuron for the
  121k@441 org vs ~180/neuron for survivors) - the E4d amplifier
  strength varies between genomes, so k=200 SELECTS for synapse
  efficiency. This is biologically interesting (selection on efficiency)
  but means the observed ~8x is '~8x for low-synapse-demand lineages',
  and the pure representational-slope question (cap fully non-binding,
  k=250-300) technically REMAINS OPEN.

REVISED CAPABILITY (honest): organism grows 55 -> ~434 neurons (~8x)
with recognition never fully collapsing in LIVING organisms, under a
size-scaled budget that selectively culls high-synapse-demand genomes.
The soft slope above ~350 oscillates 0.29-0.70 but is not a wall. The
pure-slope question (cap=0 binding) needs k=250-300.
NEXT (user): (a) extend gens past 15 / check 500+; (b) attack the soft
slope (compartmentalize or output-competition in the >350 regime);
(c) accept ~8x evident end-state as the honest self-construction bound.
STOP.
