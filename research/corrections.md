# ANIMA corrections journal

A running record of claims that were corrected by later evidence, and hypotheses that were tested
and rejected. This is part of the scientific record, not an appendix of embarrassment. Each entry
states what was claimed, what was measured, and where the corrected account lives.

## Corrected claims

- **Steering sign bug (the most consequential).** For a long period the creature appeared to fly
  *away* from food. We treated this as a general failure. It was a single sign error: the command
  that should turn right actually turned left when food was on the right. Fixing it turned fleeing
  into pursuit (food distance 11.5 to 6.7; touches 0 to 9 at a reachable radius). This is why early
  "no foraging" results are not a fair baseline. See the white paper, section 6.

- **"The learning rule is inert."** Several reward-gated plasticity rules looked inactive. Direct
  measurement (SW_PROBE) showed the wires did change (roughly 4.9x more weight movement in the
  reward window). The real issue was that the change was too small and too non-selective for the
  coarse, death-gated selection to notice. Claim corrected from "does nothing" to "does something
  that selection cannot see."

- **"The mushroom-body experiment is inert / locus is wrong."** The KC-locus reward bump was
  verified to fire (256 of 256 internal neurons eligible; about 8,635 synapses bumped per meal),
  but every neuron was eligible because the band saturates. So the bump was a uniform global wash,
  not selective credit. Corrected: not a wiring failure, not a locus failure as such; the
  sparse-selective premise of the mushroom-body idea was never satisfied. Status: unproven, not
  falsified.

- **"The home task is behaviorally impossible."** The nest experiment looked hopeless early. We
  traced the immediate cause to a units bug in the fatigue constant (velocity vs distance scale),
  not to fundamental impossibility. After correction, the organism ranged farther and could
  complete a foraging-and-return trip; one of two seeds then developed recurring returns under a
  homing reward. See the white paper, sections 6 and 13.

- **"The re-saturation is a slow gradual build-up."** The cold-to-warm transition looked like a
  slow-state ramp. Fine-grained sampling showed the band stays sparse for roughly 20 beats and then
  flips to fully saturated in one step. Corrected attribution: a sharp phase transition / hysteresis
  flip, not a gradual accumulation. This changes what to attack next (the transition, not the ramp).

## Rejected hypotheses (recorded negatives)

- **Curiosity reward** (novelty as reward): explored more, did not survive better. Kept as a
  measured negative.
- **Density and persistence fitness terms** (`SL2_DENSITY`, `SL2_ARS`): changed the score numbers,
  did not escape the ceiling. Measured negative.
- **De-saturation levers** (competition `SL2_IINH`/`SL2_INH`, threshold `SL2_THETA`, input scaling
  `SL2_AMPL`, heterogeneity `SL2_HET`): all bounded negative; none produced sustained partial
  internal activation. The band is drive-dominated and phase-transitional. See the white paper,
  section 8.

## Principle

A claim stays in the white paper only if it survives measurement here. Corrections above are not
re-writes of the record; they are additions that preserve both the original result and the reason
it changed. If you find a stronger statement anywhere in this repository than the evidence below
supports, treat the weaker, measured statement as authoritative and open a correction.