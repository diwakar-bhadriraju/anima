# Phase III sY — survival loop (frozen protocol, D-22)

Status: FROZEN, 2026-09-22. Approved direction (user): bacterium-with-a-
brain-hat — minimal senses in, minimal actions out, with a
self-preservation drive so it THRIVES, as the precondition for
reproduction/selection/growth (all later slices). This freeze is the
pre-registration the fitness definition requires BEFORE the evolution
stage inherits it.

## Scope (what this slice IS and is NOT)

IS: a closed-loop ENVIRONMENT + a homeostatic DRIVE, on the fixed
committed E-nogain substrate (24 in / 40 pool / 12 out, STDP,
d_core/d_claim). No new substrate mechanism. This makes "thriving"
measurable and gives behavior to the organism.

IS NOT (recorded as later slices, NOT this one):
- no reproduction/crossover yet;
- no structural growth (new neurons) yet;
- no fly-connectome-derived wiring (charter: stats are an evaluation
  target / maturity signal, NEVER a hand-designed internal source).

## The loop (environment)

1. The world has a current stimulus (one of the known patterns A/C, or
   a novel D, or a quiet gap).
2. Present it to the organism (spikes on its channels).
3. Run the organism's dynamics (no retrain — pure response).
4. Decode its 12-neuron output via the FIXED pre-registered codebook
   (rate over output; argmax tag) -> an ACTION among {approach, keep,
   withdraw, unsure}.
5. The action UPDATES the world (approach -> stay near known /
   re-present; withdraw -> move away / novel re-enters), 1 transition
   per beat.
6. Loop. The organism's survival depends on the world it shapes.

The environment rule is ours to define (charter: environment + I/O
boundary). It is deterministic and pre-registered, not tuned on results.

## The DRIVE = pre-registered viability metric ("thrives to live")

Defined ONLY from quantities the substrate already produces (no new
hand-tuned scalar). On every beat the organism outputs a viability v(t)
in [0,1] as the product of three measured signals:

  v(t) = a(t) * r(t) * s(t)

  a(t) = ACTIVITY BOUNDEDNESS: 1 when pool firing rate is inside
         [a_lo, a_hi]; -> 0 if it goes quiet (below a_lo) or approaches
         the runaway guard (near a_hi). Keeps it alive, not catatonic,
         not seizing. (Bounds frozen from the committed E3/E4 rate
         calibration band.)
  r(t) = RECOGNITION-RATE of known patterns: on a known stimulus,
         does its response match the learned assembly (rho > 0, the
         committed retention convention)? Fraction of known beats
         recognized over a window.
  s(t) = STABILITY / NO-FAILURE: 1 while no runaway-activity or other
         failure has occurred; -> 0 otherwise.

Frozen constant a_lo/a_hi: from the committed E3 rate band
(~[quiet-floor, 50 Hz runaway-adjacent]), same convention as the runaway
guard, NOT re-tuned here.

## DEATH (user's framing, folded in)

Organism dies if viability v(t) collapses: recognition-rate r falls
below the frozen retention floor, OR activity leaves [a_lo, a_hi]
(sustained), OR any s-failure (runaway/quiet) triggers. Death is the
SELECTION EVENT the evolution slice will use; here it is instrumented
and reported per seed (does the organism survive the loop, and how long).

## ENDPOINT / FALSIFIER (frozen before any run)

PRIMARY: the closed loop produces MEASURABLY DIFFERENT behavior for
KNOWN vs NOVEL stimuli, across >= 2/3 seeds — i.e. the organism responds
differently (different action / different viability trajectory) to a
pattern it knows vs one it has never seen. Without this, the drive is
inert and there is nothing for evolution to select on.

SECONDARY: the organism PERSISTS — mean viability v(t) stays > frozen
floor across the loop, and DEATH (runaway/quiet/recognition-collapse)
does not occur before a pre-registered horizon in >= 2/3 seeds under a
benign environment.

FALSIFY / reject (no tuning): in < 2/3 seeds the organism's behavior
fails to differ between known and novel, OR it cannot persist at all
(dies immediately in every seed) — the loop adds no thrivability signal.
Record; do not tune; reconfigure the environment only by a new
registration.

## Execution
1. This protocol committed. 2. Implement closed-loop env mode + drive
   (config-flagged, identity-gated). 3. Run 3 seeds (E-nogain, no new
   substrate mechanism). 4. Measure known-vs-novel differential +
   persistence. 5. Verdict; autonomy-log; survival-loop closes (pass or
   falsified). Later slices (reproduction/crossover/NEAT-style growth,
   fly-statistics maturity target) are separate registrations.

STOP - frozen D-22. No run until approved to implement.