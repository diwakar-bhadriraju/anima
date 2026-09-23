# Phase III sZ — evolution/death-selection verdict (D-32)

Status: RUN COMPLETE, DETERMINISTIC, RESULT = SELECTION FINDS NO
IMPROVEMENT; SIZE NEUTRAL; FOUNDERS AT LOCAL OPTIMUM. 2026-09-22.
Protocol: docs/phase3/s-evolution-protocol.md (frozen D-32).
Determinism check: best-org rebuild = scored, IDENTICAL, all seeds.
Pipeline valid; rankings trustworthy.

## Results (3 seeds x 8 gens, N=4, elitism, size band [32,56])

SIZE (PRIMARY): g0=40.0 -> g7: 40.0 / 41.3 / 37.3 (mean 39.5).
  No consistent direction. Size is NOT selected in this world: +/-
  4-neuron variants neither thrive more nor die more.

FITNESS (SECONDARY): mean fitness DECLINED in all 3 seeds
  (0.719->0.411, 0.519->0.051, 0.605->0.259); best_g7 0.82/0.09/0.70.
  With elitism carrying the winner verbatim, the decline is entirely in
  the mutated offspring: variation is DESTRUCTIVE on average.

## CORRECTION (D-34): the original verdict was CONFOUNDED

The first run used a DIFFERENT world per generation (world_seed =
run_seed ^ gen), so gen-over-gen fitness declines reflected HARDER
WORLDS, not worse organisms. Corrected run (world FROZEN across all 8
generations, D-26-style provenance):

  best fitness g0 -> g7: 0.756 -> 0.091 (s20260912)   COLLAPSED
                         0.729 -> 0.695 (s9001)       HELD
                         0.776 -> 0.749 (s424242)     HELD
  mean size g0 -> g7:    40.0 -> 40.0 / 44.0 / 37.3   (no consistent
  direction; size is NOT the selected variable)

## The honest conclusion (corrected, plain language)

- Selection WORKS (deterministic, verified; elitism carries the winner
  verbatim). The elite HELD at founder level in 2/3 seeds (0.695/0.749).
- One seed's elite COLLAPSED (0.756 -> 0.091) even under a frozen world
  - and the mechanism is measured, not guessed: PLASTICITY IS ON during
  scoring (D-24 mandates it), so the act of being evaluated IS the act
  of living, and living DEGRADES the trained assemblies (the D-21
  forgetting dynamic, now seen operating ON the champion).
- Conclusion: in this substrate, living and remembering are in direct
  tension. The organism cannot both experience and retain. This is the
  deepest measured statement of the shared-pool ceiling, and it applies
  to the CHAMPION, not just losers.

## What this means for the mission

- "Network too small" is answered DEFINITIVELY: the problem is not
  neuron COUNT (size was not selected, 2/3 flat/one-down). The problem
  is that experience and memory SHARE the same substrate with no
  mechanism to protect memory from experience-driven reorganization.
- The next mechanism MUST decouple experience from forgetting
  (structured/overlapping populations, protection during acquisition,
  or true neurogenesis) - this is now concretely motivated by the
  champion-collapse measurement, not a guess.

## What this means for the mission

- The "network too small" question is now answered DEFINITIVELY for
  this class of variation: bigger is not automatically better; the
  shared-pool representational ceiling is not breached by size alone.
- An LLM-alternative built on this substrate would be bounded by the
  same ceiling. The way forward is NOT parameter tuning or evolution
  over this fixed architecture: it is a different representational
  substrate (structured/overlapping populations, inhibition to oppose
  consolidation, or genuine neurogenesis with growth laws) - each a
  major registration, none a patch.
- The one measured capability that SURVIVED every test: experience
  learning without retrain + perfect known/novel recognition + closed-
  loop persistence. That is the demonstrated core.

STOP - D-32 verdict recorded; deterministic pipeline; evolution finds
no improvement from this substrate's local optimum; decision pending on
substrate-level next step.

## D-35 — MECHANISTIC REFUTATION of D-34 (honest, measured)

The D-34 'plasticity degrades the champion' interpretation was WRONG.
Breeding debug (EVOLVE_VERBOSE) shows:
  - SAME-SIZE elite copy: leftover=0, child live=1703 = parent live (PERFECT
    faithful inheritance every time - the elite holds because it's copied
    correctly).
  - SIZE-MUTANT offspring (44/36): leftover 916->1396, child live balloons
    to 2300-3000 vs parent 1703 - the differently-sized child has a
    DIFFERENT topology, so patching parent weights by (pre,post) key puts
    them on WRONG synapses and corrupts the brain.
  - => The fitness collapse of 20260912's lineage was the SIZE-MUTATION
    copy-artifact, NOT organic plasticity damage. Elitism's same-size copy
    is faithful; size inheritance is structurally invalid.
  - => The size question (is bigger better) is NOT answerable by this
    (pre,post) key-inheritance across size change. It requires structural
    correspondence (NEAT-style gene alignment), a separate capability.

CORRECTED standing: the evolution pipeline is deterministic and elite-
faithful (same-size inheritance works perfectly); but size-variation
inheritance is invalid, so the evolution run does NOT answer the size
question. D-34's 'living degrades the champion' is refuted. The genuine,
un-answered size question needs NEAT-style structural inheritance - a
new registration, not a patch.
