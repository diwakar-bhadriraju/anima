# D-70.2: Motion Closed Loop — world_life (registered 2026-09-24)

Status: REGISTERED (written BEFORE any code; verification appended after).

## Scope (frozen)

New bin `crates/anima-world/src/bin/world_life.rs`: the mechanical
closed loop, NO falsifiers yet (D-70.3 owns novelty / fault):

per beat (BEAT_MS = 500 ticks):
1. scene = current world state (static within the beat);
2. RetinaEncoder::from_world(&world) -> per-cell rates;
3. frames(seed) -> Poisson trains (56 channels) -> network step loop
   over BEAT_MS with STDP ON and V2Plasticity OFF (mechanical loop;
   learning machinery and growth enter with the falsifiers);
4. 12-dim output spike counts -> MotorDrive::from_counts (the D-59
   linear proportional contract) -> body.step(rates);
5. world.step -> next beat's scene reflects the motion.

Network anatomy (frozen): Network::new(cfg, 56 inputs, 40 internals,
12 outputs, seed_n); cfg mirrors the v2cfg() values used by the
survival family (V2.2 identity params); STDP params identical to
evolve's params(). No survival loop, no decode, no refs, no death.

Telemetry (frozen): per beat `WL beat= pos=(x,y,z) yaw= pitch=
speed= out_sum= frame_changed=` where frame_changed = 1 if any
vision-cell luminance moved > 1e-3 vs the previous beat, else 0.
Goes to stderr, gated by env WL_VERBOSE (identity/no-op unset).

## Determinism contract (extends D-70.1)

Same (world_seed, net_seed) -> bit-identical telemetry stream AND
final body pose. No parallel reductions anywhere.

## Acceptance (frozen)

A1 determinism: two runs, same seeds, 60 beats -> identical WL lines
(file-diff).
A2 closed loop: with motor active, the body MOVES (final |pos| > 1)
and frame_changed=1 appears (the organism's output changed what it
sees).
A3 motor-off contrast: WL_MOTOR_OFF=1 (frozen diagnostic env: zero
motor rates) -> body stays at origin and frame_changed=0 everywhere
(loop is organism-driven, not autonomous).
A4 workspace suite stays 196/0 (nothing outside anima-world touched;
the new bin compiles).

## Non-goals (frozen)

No novelty/known decoding (D-70.3), no V2/growth (D-70.3+), no
evolution, no survival death. The organism is a blind-moving body
that SEES its own motion.

## Results (appended after the runs)

Bin: `crates/anima-world/src/bin/world_life.rs` (world_life
[world_seed] [net_seed] [beats]); network 56/40/12 — NOTE the output
band computes from structure (96..108), NOT the survival anatomy's
64..76 (a scout bug caught in review; counting read internal neurons).

- A1 determinism: two runs (20260924/77, 60 beats, WL_VERBOSE=1) ->
  identical 61-line streams (bit-identical telemetry + summary).
- A2 closed loop: WL RESULT beats=60 moved=1 frames_changed=7
  pos=(9.400,30.000,9.110) — organism output moved the body through
  the world and the camera saw the change.
- A3 motor-off contrast: WL_MOTOR_OFF=1 -> moved=0 frames_changed=0
  pos=(0.000,1.500,0.000) — body stays at spawn; the loop is
  organism-driven. (moved = displacement from spawn; the first check
  used absolute position and falsely reported the spawn height —
  fixed.)
- Seed sensitivity (bonus): world/net seed changes -> different
  trajectories (50.0/30.0/8.4 vs 8.9/30.0/7.2 vs 9.4/30.0/9.1).
- OBSERVATION (D-70.3-relevant): pos.y = 30.000 = the arena ceiling in
  every motor-on run — the lift channel (motor[2]) dominates and pins
  the body at the ceiling; only 7/60 beats changed the visual frame
  (low turnover for the first 60 beats). Not a correctness issue for
  A2, but the D-70.3 novelty falsifier's power analysis must expect
  low per-beat visual turnover, and motor-channel balance (lift vs
  steering) is a steering-relevant knob for later milestones.
- Workspace suite: 196 passed / 0 failed.

Acceptance: PASS. D-70.3 (registered novelty-on-first-sight + motor
fault falsifiers with the survival-loop machinery) is next.