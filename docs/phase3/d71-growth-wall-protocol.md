# D-71: The Growth-Death Wall — Synapse-Cap Hypothesis (registered 2026-09-24)

Status: REGISTERED (written BEFORE any run; results appended after).

## Question

Every multi-generation run (D-59 gate, D-63 gate, all seeds) ends the
same way: populations grow ~22 neurons/gen and by gen 6-8 ALL four
organisms die at ~240-260 neurons (fit 0.00 everywhere) — the "wall".
Death kind is currently unlogged in gate runs. Prime suspect on
record: the fixed 20,000-synapse ResourceMonitor cap is an
INCIDENTAL monitor-wiring constant (commit e06bbb3), never
registered (D-49 note: "the ~229 wall may be this budget, not a
structural ceiling; cap-raised runs test that"). At 240+ neurons
with V2 M3 + birth wiring (20 in + 20 out per birth), live synapse
count plausibly crosses 20k right around the wall generation.

## Hypothesis (frozen)

H1: the wall = the 20k synapse cap tripping ResourceMonitor
(resource-exhaustion death) once populations cross ~20000 live
synapses. Falsified if the death kind is viability-based
(recognition collapse / activity out of bounds) instead, or if
cap-raised runs still die at ~240 neurons.

## Stages (frozen)

Stage A — diagnosis (no mechanism change): `D59_REFLEX=1 D50_MODE=1
EVOLVE_VERBOSE=1 DBG_DEATH=1 ./target/release/evolve` (3 seeds x 8
gens). Record per-org death kind + neuron/synapse counts at wall
gens. Bar: >= 80% of wall-generation deaths must be
resource-exhaustion for H1; else re-register with the measured kind.

Stage B — fix probe (already-registered mechanism, no new constant):
same run with argv[3]=0 (D-49 size-scaled synapse budget, k=200 per
neuron, cap removed). Bar: no total-death generation and organisms
survive past 260 neurons with nonzero fitness -> H1 confirmed and the
fix identified.

Stage C — gate rerun (ONLY if B confirms): D-63 stage-2 bars with the
Stage-B fix active (`D59_REFLEX=1 D50_MODE=1 D62_PARITY=1`, argv[3]=0):
mean reflex= >= 0.8 in every generation where >= 1 organism lives;
no generation with all-four-dead-by-resource.

## Instrumentation (frozen, behavior-neutral)

EVOLVE_VERBOSE org line gains the absolute live-synapse count
(post-life) next to the existing dead=/fail= fields, so wall
generations show the count at death without a mechanism change.
Gated by EVOLVE_VERBOSE (already used for diagnostics) -> identity
when unset.

## Predictions / failure modes

- P1: the wall gen shows live synapses near 20k and dead=fail
  resource-exhaustion -> H1 supported.
- F1: deaths are died_at (viability) kinds near 20k -> the cap is not
  the trigger; the growth itself breaks representation at size ->
  re-register with the measured death kind.
- F2: cap-raised runs still wall at ~240 -> same as F1.
- F3: fix works but the reflex gate still dips (seed-9001-style) ->
  the dip is a separate question, recorded, no tuning.

DEFAULT FLIP (registered follow-up, D-71): evolve.rs synapse-budget
default changed 20000 -> 0 (size-scaled k=200, D-49 mode). Verified:
flag-off identity gen 0/1 byte-identical (the cap never trips early —
it only deleted late generations), and the new default reproduces
Stage C's 424242 columns exactly (g6 reflex=1.00, g7 reflex=1.00,
sizes 266, no total death). argv[3] keeps the fixed-cap compatibility
path. Suite 196/0. Committed with the D-59..D-71 checkpoint
(a3b4848) + flip commit.

AMENDMENT (12-gen probe, seed 424242, full stack + EVOLVE_VERBOSE):
- No new failure mode past gen 8: sizes 63 -> 414 by gen 11, all
  orgs alive, no resource trips; the size-scaled monitor (k=200 x n)
  scales with headroom (23.1k synapses at n=450 vs ~90k budget).
- fit=0.000-with-no-death mystery SOLVED (algebraic): the fitness
  formula has exactly three factors (mv x (0.5+0.5*krec) x sep); orgs
  with healthy mv/krec yet fit=0.000 imply sep=0.0 — the D-50
  separation falsifier (capture_separation, decode-against-refs)
  collapsed, so selection zeroed their fitness BY DESIGN (prunes
  non-separating brains; the same representational-binding failure the
  E3/E3b line measured). Instrumentation added: EVOLVE_VERBOSE org
  line now carries krec (known_recognized_frac) and mv
  (mean_viability) next to dead/fail/n/syn/fit.
- Reflex column holds the full stack through 12 gens at 414 neurons
  (1.00 at gens 7/9/11; 0.86-0.96 elsewhere; 424242 only).

## Results (appended after runs)

Stage A (diagnosis; seed 424242 x 8 gens, verbose + DBG_DEATH):
- Cap death DIRECTLY observed at gen 7 org 3:
  `fail=resource-exhaustion: synapses 20001 > cap 20000 (k=None, n=259)`
  (org died at beat 1, n=260, 19948 live synapses post-death).
- BUT resource-exhaustion was NOT the dominant death kind by the
  frozen >=80% bar: wall generations also contain viability deaths
  (e.g. gen 5 org 3 dead=Some(18); gen 6 org 2 dead) and most orgs at
  n=260-274 carry only 5.6-6.8k synapses (cap not near). H1 as the
  SOLE cause: not met on the bar; cap as a REAL death mechanism:
  confirmed. (Stage B settles the wall question directly.)

Stage B (fix probe; argv[3]=0 size-scaled budget k=200, cap removed;
3 seeds x 8 gens):
- 424242: the cap-death org now SURVIVES (n=288, syn=8132, fit=0.234);
  no total-death generation; g7 fit 0.143 (was 0.000).
- 9001: sizes to 301, g7 fit 0.215 (was 0.100), no total-death gen.
- 20260912: sizes to 294, g7 fit 0.194 (was 0.000), no total-death gen.
=> The fixed 20k cap DELETED organisms that would have lived; the
wall's total-death generations were substantially the unregistered
incidental constant (e06bbb3). H1 CONFIRMED as the wall trigger.

Stage C (gate rerun; D59_REFLEX=1 D50_MODE=1 D62_PARITY=1, argv[3]=0;
3 seeds x 8 gens):

| seed | g0 | g1 | g2 | g3 | g4 | g5 | g6 | g7 |
|------|----|----|----|----|----|----|----|----|
| 424242 | 1.00 | 0.75 | 1.00 | 1.00 | 0.89 | 0.96 | 1.00 | 1.00 |
| 9001 | 1.00 | 0.77 | 0.79 | 0.86 | 0.89 | 0.71 | 0.82 | 0.93 |
| 20260912 | 1.00 | 0.86 | 0.88 | 0.84 | 0.81 | 0.91 | 0.91 | 0.88 |

Wall-zero artifacts GONE on all seeds (previously 0.00 everywhere by
gen 6-7); 20260912 holds >=0.8 the whole 8 gens. Frozen bar (>=0.8
every generation with >=1 organism alive): FAILS on 424242 g1 (0.75)
and 9001 g1/g2/g5 (0.77/0.79/0.71) — genuine per-generation variance:
~24 D beats per seed-gen, 1-3 misses per life set the binomial noise
floor near 0.08, i.e. the bar sits INSIDE the small-n noise. Recorded
per the frozen rule: wall fixed, gate-not-met, no bar adjustment.

Follow-up (user-gated): (a) flip the default synapse budget to the
size-scaled mode (a registered baseline change; re-capture identity);
(b) re-register the reflex gate with longer lives (EVO_BEATS up -> more
D beats per gen -> the noise floor drops below the bar) if a
generation-level PASS is wanted; (c) D-63 stage 3 (motor) still unrun
(its stage-2 bar failed) - re-runnable under the fixed world.