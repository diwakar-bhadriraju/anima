# Phase III Level-5 r2 — closed-loop action on retrieval (frozen protocol)

Status: FROZEN, 2026-09-22. r2 approved (accept Level-3 bound, climb the
action/closed-loop rung on the existing retrieval asset). No substrate
change, no temporal-prediction machinery. No post-hoc tuning.

## Slice-0 GATE (measured, committed base telemetry — no new runs)

Is there a usable action surface? Output (ids 64..75) stimulus-selectivity
under the trained alternation, cross-cosine of A-vs-C output reference
vectors (vs within ~0.9+):
  s20260912: 0.728
  s9001:     0.467
  s424242:   0.805
CONCLUSION: an action channel EXISTS (all below within-cosine; s9001
genuinely selective) but is NOISY (~0.5-0.8). Gate PASSED (an action
space exists); the closed loop must tolerate a weak decision surface.

## Slice-1 MECHANISM (closed-loop self-stimulation)

Environment extension (NEW env mode, deterministic, pre-registered; NO
labels, NO supervisor, NO gradient/target on weights — purely a fixed
rule coupling the organism's own output to the next input, per the
charter's "define environment + I/O boundary"):

After a starter presentation of A or C, each subsequent presentation is
chosen by the organism's OWN output vote on the previous presentation:
  vote(t) = sum of output neuron spike counts on output-tag-A (ids 64..69)
            vs output-tag-C (ids 70..75) over that presentation.
  next stimulus = C if vote_A <= vote_C else A.
Tags are pre-registered contiguous halves of the output layer — an
arbitrary, fixed, deterministic readout of the organism's own activity;
no "correct" reference is used.

This is pure self-generated action->perception: the organism's internal
dynamics decide which of the two learned stimuli it experiences next.

## ENDPOINT (frozen)

Does the organism GENERATE structured, self-consistent behavior in the
closed loop (agency), or collapse/drift?
PRIMARY — trajectory structure: the proportion of presentations where the
organism alternates (switches to the OTHER stimulus than the one just
seen) is NON-COLLAPSED AND > weighted chance, reproducibly across >= 2/3
seeds, AND the loop does not fall to a single fixed stimulus (a collapse
to "always pick C" is a FAIL, not structure).
SECONDARY — self-consistency: the output-driven choice and the resulting
perception form a stable bi-/multi-stable cycle (fixed point of the
loop), i.e. the action-perception pair is internally coherent.
FALSIFIER (reject): the loop collapses to a fixed stimulus in >= 2/3
seeds, OR the action sequence is indistinguishable from the fixed
curriculum / from noise (no self-generated structure beyond what any
fixed map+base dynamics would give). No tuning; runs preserved.

## Why this is the right r2 (evidence)
- No substrate/identity change (loop is environmental; base unchanged).
- Builds on the ONLY robust asset (retrieval of A and C).
- Tests a genuinely NEW capability (self-generated action structure =
  agency), not a repatch of the dead temporal-prediction family.
- Honest risk recorded: with output selectivity only ~0.5-0.8, the
  closed loop may still collapse (output too noisy to sustain a
  structured self-stimulation) - that is the falsifier, accepted up front.

## Execution
1. This protocol committed. 2. Implement closed-loop env mode (flag
   d_selfloop in config). 3. Run trained-organism -> closed-loop stage,
   3 seeds x N loop presentations. 4. Measure trajectory structure +
   collapse. 5. Verdict; autonomy-log. No tuning.

STOP — frozen (r2 Slice-1).
## AMENDMENT (D-17): falsifier realism from the Slice-0 gate

The gate (output selectivity 0.73/0.47/0.81, weak) changes the prior on
Slice-1. A closed loop next = f(output_prev) over a deterministic
output is an ITERATED BINARY MAP on {A,C} - it must end in a fixed point
or a 2-cycle; with an arbitrary tag split and no output->next learning
signal, the collapsed-fixed-point branch of the falsifier is the LIKELY
outcome, not a rare event.

Under the no-post-hoc-tuning rule this must be confronted BEFORE the run
counts: the honest classify is NOT "collapse = FAIL" but "which attractor
does the loop lock to, and what does it reveal about the learned
association?" Registration choice (frozen, pre-committed):
- PRIMARY is reframed as: the closed loop reaches a deterministic
  fixed point or 2-cycle that is DIAGNOSTIC - i.e. the attractor's
  stimulus bias matches the organism's trained output dominance
  (per-pattern rate C > A in base) in >= 2/3 seeds. This tests whether
  the organism's public action channel deterministically reflects its
  learned internal state through the loop (a real, if weak, action
  capability ladder claim).
- FAIL (reject) = the loop does not reach a stable attractor within the
  loop horizon (chaotic/noisy wander, cycle length > 2 oscillating), OR
  the attractor is inconsistent with the organism's own measured output
  dominance (would indicate the loop reads noise, not structure).
No tuning; the amended endpoint is frozen with D-17 and is decided
before any Slice-1 run.
