# Phase III Level-5 r2 Slice-1 verdict — closed-loop action

Status: FALSIFIED as-registered (D-17 criterion), with a MAP-CONDITIONING
confound that must be reported. 2026-09-22.
Instrument: examples/closedloop.rs (iterates the trained organism's
measured output-response map through the frozen tag split).
Protocol: docs/phase3/level5-r2-closedloop.md (+ D-17).

## Method (modeling reduction, not a shortcut)

next = f(output_prev) over the organism's DETERMINISTIC measured output
response is an iterated binary map on {A,C}; its attractor is exactly
computable from the trained reference responses (S1+S3 averaged, the
steady-state action surface). No live run needed; result = the attractor
of the trained organism under the closed loop.

## Result (3 seeds, all C-dominant in base: rates A/C = 50/159, 31/138,
## 46/118)

seed          from A: tagA/tagC -> next    from C: tagA/tagC -> next    attractor
s20260912     A: 301/15 -> A                C: 547/368 -> A              fixed {A}
s9001         A: 15/197 -> C                C: 423/406 -> A              2-cycle {A,C}
s424242       A: 182/72 -> A                C: 464/207 -> A              fixed {A}

## D-17 PRIMARY (frozen before any run)

"closed loop reaches a deterministic fixed point or 2-cycle whose
stimulus bias matches the trained output dominance (C > A) in >= 2/3
seeds."
- 1/3 (s9001) sustains the alternating 2-cycle A<->C = the organism's
  output, through the loop, self-perpetuates the trained alternation.
  GENUINE closed-loop agency on this seed.
- 2/3 (s20260912, s424242) collapse to fixed {A} (the NON-dominant
  stimulus) -> bias mismatch vs C-dominance.
=> NOT met (0/3, or 2/3 mismatch) => FALSIFIED as-registered.

## Confound (must be stated; do NOT overread the falsification)

The two collapses are driven by a MAP-CONDITIONING artifact, not shown
organism failure: the frozen tag split (output-id half 64-69 vs 70-75)
is degenerate on this output layer - neurons 64-69 are unconditionally
more active for BOTH stimuli (s20260912: from-A 301 vs 15, from-C 547 vs
368; s424242 similar). So the split is not discriminative and the loop
collapses to whichever half is frozen-more-active. The falsification
therefore does NOT cleanly demonstrate "the organism cannot sustain
closed-loop structure"; it demonstrates this particular pre-registered
map was poorly conditioned for 2/3 seeds (a design weakness in Slice-1's
tag), while the alternating 2-cycle on s9001 shows the loop CAN be
well-conditioned and self-sustaining when the split happens to separate
the responses.

## Honest boundary

- Confirmed (partial): on >= 1 seed the organism's output, through a
  fixed action->perception map, SUSTAINS the trained alternation (a real,
  if single-seed, closed-loop action capability hint).
- Not established: organism fails to sustain structure (the 2 collapses
  are map artifacts, not measured organism failure).
- The r2 action channel EXISTS but is weak (Slice-0: output selectivity
  0.47-0.81) and, under the frozen contiguous-tag map, the loop reads
  the output layer's unconditional asymmetry rather than stimulus-
  selective structure.

## Per protocol (no tuning)

Falsified as-registered at the D-17 bar; run results preserved. Next (a
NEW registration, not a patch; needs approval): make the action surface
well-conditioned BEFORE the loop - either a SpOtal output readout that
is contrastive (difference, not raw sum; avoids the unconditional-half
artifact) or a tag assignment that demonstrably separates the responses
- then re-test the attractor. Without conditioning, the closed-loop test
is uninformative on most seeds.

STOP - falsified-as-registered; confound recorded; single-seed positive
preserved.