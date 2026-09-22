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

## D-24 — GAP semantics + ref-drift policy (frozen before implementation)

(1) GAP world state semantics: WITHDRAW -> GAP presents one single
silence beat (no stimulus, duration = one 500 ms beat), THEN the world
re-presents a stimulus (the loop continues; the organism is not
stranded in silence). GAP beats are EXCLUDED from the recognition-rate
r(t) accumulation (r(t) counts only beats that carry a known stimulus
A/C; gap beats and novel beats are not "recognition failures" - they are
separately reported). Rationale: r(t) must measure recognition of known
stimuli, not appearances of non-known ones.

(2) PLASTICITY DURING THE LOOP + REF DRIFT policy: the loop runs with
STDP plastic LY ON (the "learns by experience, no retrain" claim must
hold inside the loop; the organism keeps adapting as it lives). Because
plasticity is on, A_ref / C_ref (the codebook references) are not
static:

  FROZEN POLICY: refs are FIXED AT S1-END - captured once after the
  formation stage, then held constant for the entire closed loop.
  Rationale: (a) a live organism's "template" for what it knows is its
  memory at the time it entered its world; (b) re-estimating refs each
  beat would make the codebook chase the organism and erode the
  known-vs-novel distinction (a novel stimulus that starts to be learned
  would silently promote itself to "known" via ref drift, confounding
  the falsifier). We accept that S1-end refs may drift out of date as
  the organism learns in-loop; that drift is MEASURED (report ref-to-
  current-response cosine over the run) and, if large, becomes evidence
  about learning dynamics - not a reason to re-normalize mid-run.

  Consequence to pre-register: the frozen th_known / q_floor thresholds
  are calibrated against S1-end refs; if in-loop plasticity moves the
  responses far from the S1-end refs, the codebook's confidence may
  degrade over the run - that degradation is part of the measured
  outcome (report final ref-response drift per seed), not a tuning
  target.

No post-hoc tuning; both policies are decided here and documented in the
falsifier's interpretation.

## D-28 — registration correction (pre-matrix, smoke-tested, NOT tuning)

Smoke test (clla-surv-s20260912) died at beat 0, mean_v=0 — a broken
loop, discarded. Cause: a_bounds=[10,50] conflicted with the committed
baseline's OWN healthy busy firing (pool 50-159 Hz during A/C
presentations; D-22 said the activity-band comes from the committed
rate data). The 50 Hz value is the SUSTAINED runaway threshold (over
5000 ms) — wrong as a per-beat upper bound. Correction (from committed
data, pre-matrix):
  a_bounds = [5, 250]  (a_lo just above silence; a_hi above busy peak
  159 with headroom; runaway remains guarded separately by the harness
  50 Hz / 5000 ms detector).
Death rule tightened while here: activity-out-of-bounds OR recognition
collapse must hold for a full r_window (10 beats), not 1 beat, to trip
death (a single quiet/fast beat is not death). Regression: run must
reach the full beat horizon or a clearly attributable mid-run death.

## D-30 — survival-loop first finding: the autonomous loop locks onto one known pattern (degenerate closed loop)

Smoke (clla-surv-min, warm-start S1->SURV): 30 beats, no death, mean_v=
1.000, but ALL beats present and decode A (A=30 C=0 W=0 Q=0). The
organism orbits a single known pattern forever: world starts on A, it
recognizes A, world keeps A, it keeps recognizing A. Viability is
trivially perfect and the known-vs-novel falsifier CANNOT fire because
the organism never encounters D (novel). This is a REAL result (an
organism that prefers its comfortable known state and stays there is
behaviorally sensible) but it makes the frozen PRIMARY endpoint
undetectable: with full-observed-ministry, known-vs-novel can't be
measured and kvs_diff is computed on an empty novel set (garbage).

DECISION (registration, not tuning): the environment must occasionally
force novelty so the falsifier is measurable, while keeping the organism
autonomy otherwise. Amend the world-update rule: with probability p_novel
(e.g. 0.2, frozen), the world presents the NEVER-TRAINED D probe instead
of following the organism's action; otherwise (0.8) the organism's
action selects the next stimulus (approach known / withdraw). This
preserves organism autonomy while guaranteeing the closed loop actually
tests known-vs-novel (D is never trained, stays NOVEL, refs fixed at
S1-end - D-24/D-26). p_novel and the D-probe are pre-registered here
(no post-hoc tuning on results).
Recorded: the first, uninformed closed-loop finding (self-selected
single-pattern orbit) is preserved; the amended rule makes the frozen
falsifier testable.
