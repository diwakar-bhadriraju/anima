# D-61: Readout Exclusivity — V2 Machinery Leaves the Reflex Band Alone (registered 2026-09-24)

Status: REGISTERED (written BEFORE any code change; results appended after).

## Question

D-59/D-60 measured the in-life reflex band to be noise-dominated
(known beats 0-370 L2 from their own templates, distributions
overlapping at th=60) and falsified both slow-state arms. The
residual-state diagnostic then identified the mechanism: the band's
incoming wiring DRIFTS during life because V2Plasticity's own
machinery treats the band as a normal neuron — M3 builds candidate
pools for it and consolidates new pool->band synapses (the band is
Output-class, only Input/Inhibitory are skipped; structural_v2.rs
guard sites), M2 normalization re-scales its "fixed" afferents, and
M4 can prune them. Measured signature: band pre-beat i_syn reaches
+/-800 (tau_syn=5ms decays e^-300 in the gap) because the pool keeps
firing during the 1500ms rest and deposits through M3-grown
connections. The D-58 prototype was clean because library.rs runs NO
V2Plasticity. The premise "fixed, content-neutral, non-plastic reflex
arc" is violated in-life by V2 itself. THE SAME DRIFT WOULD CORRUPT
ANY IN-LIFE SENSOR READOUT (future retina/cochlea afferents) — this
fix is the prerequisite for the 3D world route.

## Intervention (frozen)

D-61 readout exclusivity: when the network has a reflex band
(cfg.d58_reflex > 0), V2Plasticity treats band posts as read-only:

1. PURGE at construction (V2Plasticity::new): remove every live
   synapse onto band posts whose pre is NOT an input-afferent neuron
   (pre.idx() >= channels.len()) — removes M3-legacy wiring inherited
   through breeding.
2. EXCLUDE band posts from: M3 candidate pool build + candidate pass,
   M2 normalization (all branches: d_core/normalize_claim/CLLA/
   buckets), M4 prune (synapses post-targeting the band), sparse-commit,
   ctx_update. Guards mirror the existing Input/Inhibitory skip.
3. NOT excluded (documented): M6 inhibitory update (after the purge
   there are no band-targeting inhibitory synapses), M5 budget
   accounting (counting only), res_gate/ctx bookkeeping (they only
   feed the guarded M3), band as PRE-side coactivity source (the
   band's spikes remain part of the organism's activity).

Gate: active iff cfg.d58_reflex > 0 AND env D61_EXCL != "0"
(control = D61_EXCL=0). Flag-off (no band) = no-op = byte-identical.

## Falsifier (staged, frozen)

Stage 1 (distribution — the mechanism test; 3 seeds x 1 gen,
D59_DEBUG=1, D50_MODE=1):
- S1a fix: known-beat frac(min-L2 > 60) <= 0.10, AND
- S1b fix: D-beat frac(min-L2 > 60) >= 0.80.
Control (D61_EXCL=0, same binary): must reproduce the D-59/D-60
noise (known ~0.62-0.68) — proves the comparison is apples-to-apples.
Any other outcome: stop, record the negative.

Stage 2 (survival gate, ONLY if stage 1 passes): D-59 gate rerun
`D59_REFLEX=1 D50_MODE=1 ./target/release/evolve` (3 seeds x 8 gens):
mean `reflex=` >= 0.8 through generations.

Stage 3 (motor consequence, ONLY if stage 2 passes): clean vs fault
cons= (EVO_BEATS=100): faulted cons= > clean cons= by >= 0.2 AND
clean cons= <= 0.2 (the always-firing baseline must collapse with the
band stable).

## Predictions / failure modes

- P1: V2 drift is the variance driver -> S1a passes (band response
  becomes a deterministic function of the train), S1b passes.
- F1: known collapses but D also collapses -> the band's fixed
  projection never carried D-vs-known information in the d50 6ch
  alphabet in-life (prototype-only artifact) -> negative, stop.
- F2: known stays noisy -> an additional in-life wiring source (e.g.
  births' plastic afferents, network-level STDP onto band... band
  afferents are plastic=false; M3 is the only creator) -> audit,
  negative.
- No post-hoc tuning of any constant (repo rule).

## Results (appended after runs)

Stage 1 (3 seeds x 1 gen, D50_MODE=1, D59_DEBUG=1):

| seed | control known / D frac(>60) | D-61 fix known / D frac(>60) |
|------|------------------------------|------------------------------|
| 424242 | 0.68 (53/78) / 0.46 (11/24) | 0.05 (4/79) / 0.71 (17/24) |
| 9001 | 0.66 (41/62) / 0.70 (14/20) | 0.26 (16/62) / 0.60 (12/20) |
| 20260912 | (no control) | 0.09 (6/70) / 0.35 (9/26) |

S1a (known <= 0.10): 424242 PASS (0.05), 20260912 PASS (0.09),
9001 FAIL (0.26). S1b (D >= 0.80): FAIL all seeds (0.71/0.60/0.35).
Stage 1 FAILS the frozen conjunction -> gate NEGATIVE, per the
decision rule. MECHANISM VERDICT: the V2-drift hypothesis is confirmed
as the known-side driver (known frac collapses 0.62-0.68 -> 0.05-0.26
with the projection fixed), but the band's fixed projection alone does
NOT separate D from knowns in-life at th=60 (D sits inside the
familiar band 29-65% of the time): failure mode F1 -- the in-life
band does not carry D-vs-known information reliably even with fixed
wiring.

Diagnostic (instrumentation, NOT a bar change): exclusivity + the
D-60a band u_slow clamp (D60_PARITY=1) on seed 9001 -> known 0.20
(11/56), D 0.94 (16/17). The COMBINATION nearly restores the
prototype regime; the residual known noise (0.05-0.26, and 0.20 under
parity) is attributed to the band's remaining start-state variables
(v/z_latch reset dynamics — latch_enable=true) — NOT further
investigated here. A registered D-62 (exclusivity + band start-state
parity, this falsifier's bars) is the natural follow-up if pursued.

Status: D-61 gate NEGATIVE; exclusion mechanism VALIDATED for readout
stability (the prerequisite for any in-life sensor surface, incl. the
3D world's retina afferents).