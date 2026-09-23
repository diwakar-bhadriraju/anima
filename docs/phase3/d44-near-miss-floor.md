# D-44 PRE-REGISTRATION: survival-duration fitness floor

Date: 2026-09-23. FORWARD amendment to D-38's death-gate. Applies only
to runs started AFTER this record (all seeds measured fresh, not as
continuations of c94f507).

## Problem (evidence)
c94f507 GA: seed 424242 grew to 229 neurons, then ALL 4 organisms died
at gen 7 (fit 0.000) - genuine lineage collapse past the metabolic
envelope (advisory-confirmed honest data, not bug). Under D-38's flat
"death -> fitness 0.0":
- A dead-gen produces NO selection gradient. With elitism, the elite is
  carried verbatim EVEN IF IT DIED, and offspring are bred from dead
  genes -> the lineage locks onto the cliff edge: it cannot tell
  "died beat 5" from "died beat 195" (both 0.0), so it cannot evolve
  around the collapse point. Missing information, not a tuning choice.

## Amendment (registered constant)
Fitness for a dead/failed organism = base * (beats/spec.beats) * FLOOR
where base = mean_viability * (0.5 + 0.5*known_recognized_frac) [the
living-fitness formula], and FLOOR = 0.25 (registered constant).

Fully-surviving organism keeps base (FLOOR never applies). A
near-completion lineage (195/200 beats) gets base * 0.975 * 0.25 ~
0.24x - enough to rank above an instant-death (5/200 -> ~0.006x) and
give selection a survival-duration gradient, but ALWAYS below any
survivor. Justification for 0.25: runaway-causing lineages reach long
near-miss survival; without a cap below 1.0 they could outrank weak
but viable survivors. 0.25 cleanly separates "died but nearly made it"
from "survived" while keeping gradient.

## Prediction (falsifiable)
Under D-44, seed 424242's collapse becomes GRADED collapse: gen 7 shows
non-zero near-miss fitness (not all 0.000), and selection has a
gradient to evolve the birth law back under the envelope -> the lineage
either recovers OR degrades gracefully instead of cliff-locking.

## Failure mode (honest)
If FLOOR is too high, near-runaway genes propagate and re-collapse each
gen (oscillation). 0.25 chosen below the living base to avoid this.
STOP.
