# D-70.1: 3D World Scaffold — anima-world crate (registered 2026-09-24)

Status: REGISTERED (written BEFORE any code; verification appended after).

## Scope (frozen)

New crate `crates/anima-world` (workspace auto-member `crates/*`):
deterministic full-3D world scaffold — arena, rigid primitive shapes,
a body (3D position, yaw/pitch, velocity) driven by a motor-command
velocity demand, a pinhole rasterizer, and a retina encoder emitting
Poisson spike trains per cell. NO organism wiring yet (that is D-70.2)
and NO change to the survival loop (identity: anima-exp untouched by
this milestone — its own suite must stay 186 green).

## Determinism contract (frozen, the core acceptance)

Same seed -> bit-identical world state after every step AND
bit-identical retina frames. Conventions mirror the verified io.rs
conventions: FNV-1a hash + splitmix64 derive (seed, symbol/cell),
Xoshiro256PlusPlus per cell (rand 0.8 / rand_xoshiro 0.6, workspace
deps), fixed-timestep sequential f32 integration, painter's-algorithm
raster over the low-res grid (no Z-buffer float compares). No
parallelism, no HashMap iteration ordering, no external physics.

Conformance tests (in-crate):
1. `determinism_world`: two fresh worlds, same seed, 100 steps with
   scripted motor commands -> identical state bytes (positions,
   orientations, velocities) after every step.
2. `determinism_frames`: two fresh worlds, same seed -> identical
   retina frame (rate matrix) and identical emitted spike trains.
3. `seed_sensitivity`: two seeds -> frames differ (bit-level) within
   10 steps.
4. `survival_identity` (anima-exp): flag-off suite unchanged -> the
   workspace suite stays 186 passed / 0 failed.

## Anatomy (frozen)

- Body: position (Vec3 xyz), heading yaw + pitch (radians), velocities
  (v_lin Vec3, v_ang Vec2), friction; `step_physics(dt, motor)`
  applies the motor's per-channel rate demands (io::motor convention:
  rate -> velocity demand, linear proportional, clamped):
  12 output channels -> [thrust, strafe, lift, yaw_L/R, pitch_U/D,
  brake + no-ops]; gravity optional off (registered: off for D-70.1,
  ground plane at z=0, body clamps to it).
- Primitives: cube / pyramid / sphere (fixed pose, flat color, size);
  placed by the world builder (scene table, seeded).
- Camera/rasterizer: pinhole at body pos, yaw/pitch orientation; view
  plane 8 (azimuth) x 6 (elevation) = 48 cells, vertical FOV fixed;
  per cell: front-most primitive coverage-weighted color -> luminance
  in [0,1] + local contrast (|lum - neighbor mean|); painter's
  painter-ordered polygons (back-to-front by depth, deterministic).
- Retina encoder: `SensoryEncoder` impl (anima-exp::encoders trait):
  channel_count = 48 (vision) + 8 (proprioception: quantized heading,
  pitch, speed, altitude); frames(seed) = Poisson trains over
  BEAT_MS (500 ticks) with per-cell rate = f(luminance, contrast)
  clipped to [0, RATE_MAX_HZ], convention identical to
  io::symbol_trains_mode (FNV/splitmix/xoshiro, sorted by tick).
- Proprioception channels: 8 cells encoding [yaw bins(4), pitch
  bin(1), speed bin(2), altitude bin(1)] as constant-rate cells.

## Acceptance

1. `cargo build --release` clean; 4 conformance tests pass.
2. Workspace suite: 186 passed / 0 failed (nothing outside anima-world
   changed).
3. Determinism verified by test 1-2 (bit-identical), sensitivity by
   test 3.

## Non-goals (frozen for D-70.1)

- No survival-loop integration (D-70.2: motion closed loop; D-70.3:
   novelty + motor-fault falsifiers, each its own registration).
- No pretrained models / GPUs / external engines (charter).
- No E9/E10 names (curriculum experiment collision) — the encoder is
   `retina`/`cochlea` naming.

## Results (appended after the build)

- `crates/anima-world` scaffold built (world.rs: full-3D body kinematics,
  arena bounds, 5 primitive scene with seeded placement inside the camera
  cone; camera.rs: per-cell analytic ray rasterizer (cube AABB / sphere /
  pyramid Möller-Trumbore) -> luminance + 4-neighbor local contrast;
  retina.rs: SensoryEncoder impl, 48 vision + 8 proprio channels,
  Poisson emission with io-convention seeds (FNV-1a + splitmix64 +
  Xoshiro256PlusPlus), sorted by tick; io::hash_str/derive_seed64 made
  pub so the convention is SHARED across crates).
- Conformance: 10/10 tests pass — same-seed world state bit-identical
  over 100 steps; same-seed frames and trains identical; different
  seeds diverge (scene and frames); body trajectory seed-independent
  BY DESIGN in D-70.1 (primitives static, no vision-steering yet —
  divergence enters in D-70.2); bounds/floor clamps hold; channel
  count 56; trains sorted, in-window, in-range.
- Workspace suite: 196 passed / 0 failed (186 pre-existing + 10 new),
  nothing outside anima-world changed (io gained only pub visibility).
- Flag-off survival-loop identity: `gen 0/1` byte-identical after all
  changes.

Acceptance: PASS. D-70.2 (motion closed loop: retina frames drive the
beat trains, motor commands move the body, world steps per beat) is
the next registration.