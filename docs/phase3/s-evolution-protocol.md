# Phase III sZ — evolution / death-selection population loop (D-32, frozen)

Purpose: turn the measured "thrives" (D-31 survival verdict) into an
EVOLUTIONARY SELECTION process. Each organism lives its survival loop;
viability is its fitness; the fittest reproduce (copy + variation);
the rest die. Repeated across generations. This is the first time
selection can act on a measured thrive-quantity in this program.

## The research question (user's, made testable)

"Is the 40-neuron network too small to process/remember?" -> in an
evolving population where brain SIZE varies, does selection push
population size UP (bigger brains process better -> survive better),
push it DOWN, or keep size flat (size is irrelevant / growth is
neutral-or-lethal)? Three outcomes, each a finding.

## Mechanism (frozen; existing machinery only, no substrate change)

Variation = (a) BRAIN SIZE mutation: offspring n_internal drawn from a
bounded neighborhood of the parent (mutation step +/-4, band [32,56],
the size hypothesis is explicit), and (b) WEIGHT mutation: each synapse
of the copied brain perturbed +/-U(0,10%) with prob 0.10 (post-copy,
pre-life). Everything else identical (same seeds, same S1 curriculum,
same survival loop, same codebook/viability).

LIFE = formation (S1 A/C 20 reps interleaved, live STDP, as harness/
talk.rs) + closed survival loop (D-22..D-31 machinery: 30 beats,
forced-novelty p=0.2, deaths recorded).
FITNESS = mean viability v = a*r*s over the survival loop (the exact
D-22 metric; runs are deterministic per seed).

SELECTION (frozen, elitist): population N=4; rank by fitness; keep
top 2; each keeper spawns 2 offspring (copy + mutations) -> next gen
same N. DEATH is real: the bottom 2 of each gen never reproduce.
GENERATIONS: 8 pre-registered. SEEDS: 3 independent evolution runs
(20260912 / 9001 / 424242) to test reproducibility.

## Endpoints (frozen falsifiers)

PRIMARY (size question): does population mean n_internal INCREASE
across generations (selection favors bigger brains), stay flat, or
decrease? Measured as mean g8 vs g0, per seed + pooled. Any consistent
direction >= 2/3 seeds = the size answer; flat in >= 2/3 = "size not
selected" (the 40-neuron pool is not the binding constraint in this
task).

SECONDARY (fitness improves): best/mean fitness difference g8 - g0
positive in >= 2/3 seeds = selection is acting (variation + thrift
works); flat = selection has nothing to find (no heritable variation
that helps => the substrate may be at a local optimum for this world).

TERTIARY (the forgetfulness thread): per organism, survival-loop
recognition-rate (known beats recognized, exact-match) tracked across
gens - does selection reward better retention? If fitness correlates
with known-recog, selection is implicitly selecting for memory.

## Honesty / protocol notes (frozen)

- Deterministic per (seed, variants); no post-hoc tuning; runs preserved.
- Variation is registered (range, step, prob) BEFORE any run.
- The survival runner is the SAME post-loop runner as D-31 (governor
  limitation stands; benign-world, consistent across all organisms, so
  selection comparisons are fair).

STOP - frozen D-32. Implementation: anima-exp example evolve.rs
reusing io + survival machinery; 3 runs x 8 gens; verdict D-33.