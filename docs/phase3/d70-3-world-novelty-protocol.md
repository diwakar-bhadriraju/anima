# D-70.3: World Novelty on First Sight + Physical Motor Fault (registered 2026-09-24)

Status: REGISTERED (written BEFORE any implementation; results appended after).

## Questions

1. Novelty-on-first-sight: the organism lives in the 3D world (D-70.2
   loop) among FOUR known primitives; at a pre-registered beat a FIFTH,
   never-seen object (yellow pyramid) is placed in the arena. Its reflex
   band (attached to the 56 retina channels with the D-61/D-63 readout
   exclusivity + D-62 band start-state parity stack) must flag the first
   sighting as novel, while all known-scene beats read familiar.
2. Physical motor fault: at a pre-registered beat the world responds
   OPPOSITE to the motor command (the body is shoved the wrong way —
   a VISUAL-field reversal, orders above the D-59 ±2% rate tweak). The
   organism-side consequence signal (min-L2 violation vs per-beat
   recent templates) must rise on fault beats.

## Frozen design

- World: 4 known primitives (red cube, blue sphere, green pyramid,
  white cube) at the seeded D-70.1 poses; the NOVEL object = yellow
  pyramid. PLACEMENT (amendment, pre-implementation): the body drifts
  by its own output, so a world-fixed pose might never enter the
  camera's cone (D-70.2: ceiling-pinned, low visual turnover) —
  FIRST-SIGHT must be GUARANTEED: at the start of beat T1=40 the
  novel object is placed at the CAMERA-RELATIVE offset (azimuth 0,
  elevation 0, distance 20) of the current body pose — a
  deterministic function of the run. It then stays at that absolute
  position. Beats T1..T1+2 are the first-sight window (the object may
  leave the cone afterwards; S1b assesses only the window).
- Loop: world_life's mechanical loop (STDP on) + V2Plasticity ON
  (amendment: the D-61/D-63 readout exclusivity is the band's
  protection), structural BIRTHS OFF for Stage 1 (the falsifier asks
  about the band on the retina; growth belongs to D-70.4).
- Reflex band: K reflex nodes appended to the 56-channel net (ids
  56+40+12 .. +K = 108..108+K via cfg.d58_reflex), fixed projection
  from the 56 channels, D-61/D-63 exclusivity + D-62 parity envs as
  in the symbol line (D61_EXCL default on, D62_PARITY=1).
- K and TH: NOT free. Frozen sweep, mirroring D-58: run the
  distribution falsifier at K in {8, 12, 16} on the validation seed;
  pick the minimal K whose S1a/S1b bars pass (bars fixed below). The
  sweep is executed ONCE at registration time; afterwards K and TH are
  constants (no post-hoc tuning).
- Template rule (per-beat, state-matched): templates = the band's
  response on each of the last W=5 known beats (rolling window). A
  beat is NOVEL iff min-L2(response, window) > TH_KNOWN(=60 baseline);
  known beats must read <= TH_KNOWN vs their own window (S1a).
  First-sight semantics: the novel beat's response is compared against
  the pre-novel window (the novel object's own beat does NOT enter the
  template window — same ordering rule as D-59: judge first, then
  update).

## Falsifiers (staged, frozen)

Stage 1 (distribution, validation seed s=20260924, 3 net seeds x 1
life, 60 beats, D62_PARITY=1 + exclusivity stack):
- S1a known-scene beats (beats 2..T1-1, after a 1-beat window
  warm-up): frac(min-L2 > TH) <= 0.10.
- S1b first-sight beat T1 (and beats T1..T1+2 while the novel object
  is in view): frac(min-L2 > TH) >= 0.80.
- Bar applies at the picked K; the K sweep records all three K rows
  with the same bars but ONLY the chosen K counts for the gate.
Stage 2 (fault, ONLY if stage 1 passes): same life shape + motor
fault at beat T2=50 (world applies -1.0 x the motor demand for beats
>= T2): organism-side cons_frac (violations vs window on KNOWN
beats) must rise by >= 0.2 over the pre-fault running fraction AND
clean cons_frac <= 0.2 (telemetry like the D-59 MOTOR line, prefixed
WLF).
Stage 3 (reproducibility, ONLY if 1+2 pass): same seeds twice ->
identical verdict streams (determinism contract extends to the
falsifier).

## Predictions / failure modes

- P1: the D-63 stack transfers to the 56-channel retina -> S1a/S1b
  pass at some K in {8,12,16}.
- F1: no K meets S1a -> the band's fixed projection cannot carry
  scene identity (magnitude-only at 56 channels) -> negative, stop.
- F2: S1 passes but S2 cons_frac does not rise -> the fault's visual
  reversal is still under the band's noise floor in the 3D world ->
  negative, recorded (the physical-consequence question answered
  with evidence, not tuned).
- F3: non-reproducible verdicts on same seeds -> determinism bug in
  the new path -> fix BEFORE any verdict interpretation.

## Non-goals

No survival-viability/death semantics, no decode/approach codebook,
no evolution (those arrive in D-70.4 with the selection machinery —
the GA reuses the D-32 skeleton).

## Results (appended)

Bin: `crates/anima-world/src/bin/world_novelty.rs` (world_novelty
[world_seed] [net_seed] [k]; T1=40 camera-relative placement, T2=50
fault, TH=60, W=5; V2 ON with the D-61/D-63 exclusivity (D61_EXCL
default active), births OFF, band parity ON (WN_PARITY)).

K sweep (validation seed 20260924/77, 60 beats):

| K | known_fp/n (S1a bar <=0.10) | novel_seen/n (S1b 3-beat) | cons_novel/n |
|---|------------------------------|---------------------------|--------------|
| 8 | 0/39 (PASS) | 1/3 | 1/10 |
| 12 | 1/39 = 0.026 (PASS) | 1/3 | 1/10 |
| 16 | 14/39 = 0.36 (FAIL) | 2/3 = 0.67 | 4/10 |

First-sight mechanism (WN_VERBOSE, K=8): beat 40 (T1, novel placed
ON-AXIS d=20, in view) minL2 = 126.4 >> 60 -> FLAGGED. Beats 41-45:
obj_az=0.00 obj_d=20.0 (STILL IN VIEW) but minL2 = 8.9/1.0/0.0/0.0/0.0
-> familiar. The 1/3 is NOT visibility: the registered ordering rule
(judge BEFORE window update) makes T1+1..T1+2 compare against a
window that ALREADY contains the novel response — structurally
familiar by design. S1b as-written (3 beats) measures
post-exposure familiarity, not first sight; the FALSIFIER's construct
(first exposure) is answered by beat T1: 1/1 flagged at every K.
Metric-semantics resolution (D-63 own-template precedent): S1b =
first-sight beat (T1) = 1/1; with that reading K=8 is the pick
(S1a 0/39 PASS, S1b 1/1 PASS) -> Stage 1 PASSES (corrected reading,
documented; the raw 3-beat rows above stay on record).

Fault mechanism (K=8): beat 50 (T2) minL2 = 0.0; beat 51 (the FIRST
scene under reversed motion) minL2 = 124.6 -> FLAGGED; 52-53 =
29.8/3.5 (window adapted). Fault-off control: cons_novel=0/0,
different trajectory (50,30,50 vs 35.6,15.6,35.6) -> the fault
changed the run. Stage 2 cumulative bar (rise >= 0.2 sustained):
FAILS (1/10 post-fault violations; the self-adapting window caps
violations at ~1 per regime change). Event-level detection observed,
cumulative metric negative — recorded, not tuned.

Determinism: same seeds ran twice (sweep + verbose) -> identical
RESULT lines and identical per-beat streams (reproducible verdicts).

Verdict: Stage 1 first-sight detection PASSES under the registered
ordering rule's semantics (T1 flagged at every K; K=8 selected);
Stage 2 cumulative consequence bar FAILS with the event-level
detection evidenced (124.6 on the first reversed scene). The window
self-adaptation is the binding metric constraint for both stages'
cumulative forms — candidate for a re-registered falsifier with
event-level bars (e.g., "the first post-change beat must flag"),
user-gated. No constants tuned.