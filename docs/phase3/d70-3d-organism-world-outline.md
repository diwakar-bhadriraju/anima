# D-70 Outline: Deterministic 3D Organism World (planning — NOT yet registered)

Status: OUTLINE for user approval. A frozen protocol (registration
header, constants, falsifiers) is written BEFORE implementation, per
repo convention.

## Why (evidence)

The synthetic symbol world has four measured negatives (D-59 gate,
D-59 motor, D-60a, D-60b) and one mechanism win (D-61: V2 was
rewiring the reflex band; exclusivity fix validated — readout drift
0.62-0.68 -> 0.05-0.26). The remaining D-side shortfall (D frac
0.35-0.71 vs 0.80 bar) may be a property of the 6-channel symbol
alphabet (magnitude-only band coding). A 3D world changes the input
semantics fundamentally: spatial structure, self-motion, and physical
consequences — matching the user's stated priorities (see, move,
"knows this does that exactly").

## Charter compliance (frozen commitments)

- I/O-boundary design only: the network, plasticity laws, and reflex
  rule are UNTOUCHED. Only the environment + encoders change (both
  are user-defined per charter).
- Determinism: fixed-timestep sequential physics (no parallel
  reductions), seeded RNG, scene STATIC within a beat; retina
  emission = a deterministic function of (scene, seed) — same
  convention as `io::symbol_trains_mode` (derive_seed64/hash_str
  reused). Same seed -> identical run, byte-identical.
- NO pretrained models / CNNs / CLIP / VLMs / embeddings / GPU.
  Vision = software pinhole raster of flat-shaded primitives -> a
  deterministic filterbank (luminance + local contrast) -> per-cell
  Poisson rates. Encoder named `retina` (NOT E9/E10 — those names
  belong to the curriculum experiments).
- The D-61 readout exclusivity applies to the retina afferents:
  sensor readout lines are fixed projections, excluded from V2
  machinery (already implemented, env-free for d58 band; retina lines
  get the same treatment at construction).

## Shape (proposed, adjustable by user)

- World: bounded arena (e.g., 100x100 units), flat ground, a few
  static primitives (cube, sphere, pyramid...) at fixed positions.
  Gravity-free top-down 2.5D at first (heading + x/y motion) —
  "3D" in the sense of a rendered spatial scene; full 3D physics is
  a later milestone (determinism risk higher).
- Body: 1-2 DOF agent — heading theta + forward speed v. Motor
  surface: the D-59 `io::motor` contract (per-output-neuron rate ->
  per-beat velocity demand, linear proportional, clamped). 12 output
  neurons -> 12 motor commands (e.g., turn-L/R, thrust, brake, ...);
  unused neurons map to no-op channels.
- Retina: 6x8 grid (48 cells) pinhole view of the arena directed by
  heading; per-cell rate = f(luminance, contrast); 48 channels.
  Proprioception: heading + speed quantized -> 8 channels. Total
  input 56 channels (vs 24 today).
- Beat: 500 ms spike beats (BEAT_MS, network cadence unchanged);
  world steps once per beat (body moves by the motor command); the
  next beat's retina frames the NEW scene. This is the closed loop:
  action -> motion -> visual consequence.
- Novelty: three known shapes present during formation training; a
  NOVEL shape placed at a pre-registered beat -> the reflex rule
  (K=8 nodes on the 56 input channels... K re-derivable per registered
  sweep, NOT tuned post-hoc) must flag first sight.
- Life: formation stage (train on known scenes, STDP on) + survival
  loop (approach-known / withdraw semantics from the D-23 codebook
  generalized to spatial approach: approaching a known shape keeps
  it in view, etc.). Evolution: D-32 GA unchanged.

## Milestones (each a registration with its own frozen doc)

- D-70.1 scaffold: new crate `anima-world` (world + rasterizer +
  retina + proprioceptron + motor driver); determinism conformance
  test (same seed twice -> identical spike trains); identity gate
  (flag-off: symbol world unchanged, byte-identical).
- D-70.2 motion closed loop: body responds to motor; visual field
  changes with heading; telemetry (heading, speeds, retina cells).
- D-70.3 falsifiers: (a) novelty-on-first-sight registration
  (pre-registered bars); (b) motor-fault reversal (world moves the
  body OPPOSITE to the command at beats >= T; organism-side
  consequence signal must rise) — the D-59 fault test with a PHYSICAL
  consequence (visual-flow reversal) instead of a rate tweak; (c) the
  D-59-style GA gate if (a)+(b) pass.

## Open decisions (user)

1. Scene/body scale and DOF (2.5D top-down vs full 3D physics).
2. Retina resolution (6x8=48 vs 8x8=64) and proprioception channels.
3. Whether to run the cheap D-62 registration first (exclusivity +
   band start-state parity; ~10 min of runs; it decides if the
   symbol-world reflex line can ALSO close, informing the D-70
   novelty design).