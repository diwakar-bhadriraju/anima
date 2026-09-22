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
## FROZEN ACTION SET + CODEBOOK + WORLD-UPDATE RULE (explicit, D-23)

Action set is exactly {approach, keep, withdraw, unsure}. The codebook
maps the 12-neuron OUTPUT state (measured per beat) to an action via a
pre-registered rule, and each action deterministically updates the world.
All thresholds below are frozen registration values (set once, not tuned).

References: output ref vectors A_ref, C_ref = mean 12-dim output vector
over S1 A / S1 C training presentations (the measured 100%-separable
code). cos = centered cosine (rate-level insensitive, matches retent.rs).

                                          per-beat output state
   CODEBOOK (12-dim output vector v, 1 beat)
   ------------------------------------------------------------------
    rhoA = cos(v, A_ref), rhoC = cos(v, C_ref),  amp = sum(|v|)/12
   ------------------------------------------------------------------
    if amp < q_floor              ->  UNSURE    (quiet: re-sample)
    elif rhoA > th_known and rhoA > rhoC        ->  APPROACH-A   (known A)
    elif rhoC > th_known and rhoC > rhoA        ->  APPROACH-C   (known C)
    elif max(rhoA,rhoC) <= th_known             ->  WITHDRAW     (novel/unknown)
    else                                          ->  UNSURE      (ambiguous)
   ------------------------------------------------------------------
   FROZEN constants: q_floor (quiet activity floor), th_known (known-match
   cosine threshold); both set from the committed S1 measurement once,
   before any survival run.

   WORLD-UPDATE RULE (deterministic, per action)   current world stim S
   ------------------------------------------------------------------
    APPROACH-A -> next stimulus = A            (move toward known A)
    APPROACH-C -> next stimulus = C            (move toward known C)
    KEEP(merge) -> next stimulus = S           (stay on current, reinforce)
    WITHDRAW    -> next stimulus = GAP (quiet) (move away from unknown)
    UNSURE      -> next stimulus = S           (re-sample the same)
   ------------------------------------------------------------------
   NOTE: KEEP is merged into APPROACH-X (approaching the recognized
   pattern equals keeping it present); the 4-action set collapses to
   3 world-update branches (toward-A / toward-C / gap) + re-sample.
   The organism therefore shapes its own world toward the known patterns
   it recognizes and away from novelty - the thrivability behavior.

DEATH hook (unchanged): viability v(t) < floor -> record death here too.

## EXECUTION - closed-loop env mode is a NEW harness capability (D-23)

The existing environment PRE-GENERATES a static schedule; a closed loop
(next stimulus from LIVE output) cannot be expressed statically. This is
a new harness mode:
- New config flag, e.g. `loop_mode: "closed"` (default `"schedule"` =
  current static path, byte-unchanged). Identity: when loop_mode
  = schedule, the harness takes the exact existing code path (zero RNG/
  wiring divergence; the committed baseline stays FNV-identical).
- The closed mode runs beat-by-beat: present -> run -> decode -> world-
  update -> next, rather than consuming a pre-generated schedule.
- Implement as its own work item with its own unit tests; the flag-off
  path is untouched (identity-gated).
1. Freeze (this doc). 2. Implement closed-loop harness mode (flag-gated,
   identity) + drive + codebook. 3. Unit tests + identity gate. 4. Run 3
   seeds. 5. Measure known-vs-novel differential + persistence. 6. Verdict.
STOP - frozen D-22 + D-23.
