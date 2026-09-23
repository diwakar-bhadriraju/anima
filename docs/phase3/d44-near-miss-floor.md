# D-44 PRE-REGISTRATION + VERDICT: survival-duration fitness floor

Date: 2026-09-23. Forward amendment to D-38's death-gate. Applies only
to runs started AFTER this record (all seeds measured fresh).

## Registration (constant)
Dead/failed fitness = base * (beats/spec.beats) * FLOOR; FLOOR=0.25.
base = the living-fitness formula. Fully-surviving organisms keep base.

## VERDICT: PARTIALLY FALSIFIED (informative)
Treatment: 3-seed x 8-gen GA started fresh under D-44.

  seed      D-43 (flat 0) g7        D-44 (floor) g7      effect
  20260912  230, fit 0.621          230, fit 0.548        ~same trajectory
  9001      230, fit 0.537          160, fit 0.513        ~same
  424242    229, ALL 0.000          229, ALL 0.000        NO effect

CONFIRMED: the floor registers late activity-deaths (20260912 g4
137:0.07, g5 158:0.11; 9001 g6 139:0.00) - graded near-miss fitness
appears where death follows in-band viability.

FALSIFIED (prediction): 424242's gen-7 collapse was expected to become
graded. It did NOT - all four organisms still score exactly 0.000.
MECHANISM: 424242 dies by RECOGNITION collapse (r=0 -> viability
v=a*r*s = 0 -> base = 0), and the floor multiplies base: 0 * frac * 0.25
= 0. The floor ONLY grades activity-deaths (out-of-band, base stays
positive); recognition-collapse death is a hard zero through any
multiplicative floor because the viability signal itself is zero.

CONCLUSION: D-44 does not rescue recognition-collapse lineages (424242
at ~229 neurons) - those are hard deaths by construction of v(t). The
hypothesis that a survival-duration floor restores gradient across
424242's collapse is FALSIFIED. The floor is a minor refinement for
activity-deaths only; the recognition-collapse wall is a structural
(representational) limit, not a fitness-loss-gradient artifact.
STOP.
