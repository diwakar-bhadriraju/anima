# D-44 FINAL VERDICT: survival-duration fitness floor FALSIFIED (net-negative)

Date: 2026-09-23. Forward amendment to D-38's death-gate. Applies only
to runs started AFTER this record (all seeds measured fresh).

## Registration (constant)
Dead/failed fitness = base * (beats/spec.beats) * FLOOR; FLOOR=0.25.
base = the living-fitness formula. Fully-surviving organisms keep base.

## VERDICT: FALSIFIED (net-negative) - reverted 174477e+
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

CONCLUSION: D-44 FALSIFIED (net-negative), not merely partial. TWO
independent failures:
(a) Recognition-collapse not rescued: 424242 all four organisms died
    gen 7 at base=0 (v=a*r*s=0 when r=0), floor multiplies 0 -> stays
    0.000. The floor cannot reach recognition-collapse deaths because
    the viability signal itself is zero at collapse.
(b) Selection WHERE the floor DID act (activity-deaths) drifted toward
    mid-fitness dying organisms: they bred into the pool and REGRESSED
    seed 9001 by 30% (c94f507 230 neurons -> D-44 160). Promoting
    'almost-dies' phenotypes is a SELECTION DISTORTION, not a gradient
    rescue.

Net: D-44 was net-negative. Reverted evolve.rs to c94f507's flat-0
death fitness (byte-identical functional code). The falsified amendment
stays in history (2871328 registration, 174477e verdict) as documented
negative result, NOT applied. c94f507 is the active registered baseline.
RESOLVED: the recognition-collapse wall at ~229 neurons is a
STRUCTURAL (representational) limit - shared-pool readout saturates as
self-construction grows the pool - not a fitness-gradient artifact the
loss function can rehabilitate.
STOP.
