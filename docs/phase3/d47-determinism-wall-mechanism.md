# D-47: determinism bug found + wall mechanism MEASURED as synapse-cap exhaustion

Date: 2026-09-23. Two findings that correct prior verdicts.

## 1. DETERMINISM BUG (evolve GA was nondeterministic run-to-run)
ROOT CAUSE: `breed_inner` built the parent weight map as a `std::collections::HashMap<(u32,u32),f32>` and iterated it for residual synapse adds + mutation. HashMap iteration order is per-process nondeterministic (randomized hasher), so the SAME command gave different offspring: measured twice with identical invoke, gen-1 mean_fit 0.532 vs 0.531 (and later hand diverged sizes 244 vs 267).
FIX: wmap -> BTreeMap (ordered iteration). VERIFIED: same command twice -> byte-identical gens 0-7 (DETERMINISTIC).
IMPACT: D-41/D-43/D-46 single-run verdicts were on ONE nondeterministic path each. Their directional claims (3-4x growth viable, wall real, output-inhibition per-seed) REPLICATED across independent runs, so conclusions stand - but "deterministic per seed" is only now true. Any future quantized fitness claim needs the BTreeMap fix.

## 2. WALL MECHANISM = RESOURCE-EXHAUSTION (synapse cap), NOT the D-45 attractor
Prior claim (8710b7d) attributed the ~229 wall to recognition collapse while alive (D-45 degenerate-output attractor telemetry, from a SMALLER org). D-47 instrumentation (thread died_at + fail kind into scoring) measured the wall directly:
  org rows at n>=220: dead=Some(0) fail=Some("resource-exhaustion: synapses 20933-39154 > cap 20000") fit=0.000, EVERY org, EVERY gen past the wall.
So: growth to ~231-267 neurons drove live synapse count past the 20000 budget, the ResourceMonitor tripped, and ALL organisms are monitor-dead at beat 0. The bounded resource is SYNAPSES (self-construction is synapse-explosive), not recognition and not neurons.
CORRECTION: 8710b7d's 'recognition collapse while alive' is WRONG - the wall is synapse-budget exhaustion. The a=1/rate-88 signature was from a smaller organism, not the wall.
REFRAMED FORK: not 'compartmentalize pools vs accept' but 'RAISE/restructure the synapse budget (or slow synapse growth) vs accept 20k ceiling'. The E4d reciprocal-excitation amplifier's effect is SYNAPSE explosion (positive-feedback M3 permanence / birth wiring adds synapses without bound), which trips the cap. Growth-shape regulation of synapse accumulation is now the direct lever.
STOP.
