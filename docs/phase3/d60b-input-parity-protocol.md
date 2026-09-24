# D-60b: Input-Afferent Slow-State Parity (registered 2026-09-24)

Status: REGISTERED (written BEFORE any run; results appended after).

## Question

D-60a (docs/phase3/d60-slow-parity-protocol.md) falsified the band's
own slow state as the source of the in-life reflex noise: clamping the
band's u_slow=0 at beat end left known-beat frac(min-L2 > 60) at
0.65 (control 0.68; bar <= 0.10). The remaining isolation candidate:
the reflex band reads ONLY the 24 input-afferent neurons (fixed
synapses from the input channels' targets, network.rs construction),
so if THOSE neurons' own slow-state context (tau 5000 ms, 74%
retained across the 1500 ms gap) shifts their relay of the identical
train, the band's counts vary with history even with the band itself
state-clean.

## Intervention (frozen)

`D60B_PARITY=1` (identity when unset): at the END of each beat's tick
loop, clamp `u_slow = 0.0` on the input-afferent neurons (ids
0..net.channels.len(), the neuron each InputChannel drives; they are
the band's afferent source AND the pool's input). Registered caveat:
this changes what the WHOLE organism receives (bigger intervention
than D-60a); the pool's dynamics change under the flag. Everything
else unchanged; D-59/D-60a machinery and constants (K=8, th=60)
untouched. D60_PARITY (band clamp) is NOT part of this run; combining
flags is a separate registration if this one partially passes.

## Falsifier (staged, frozen — same bars as D-60a)

Stage 1 (distribution; 3 seeds x 1 gen, D59_DEBUG=1, D50_MODE=1):
with D60B_PARITY=1,
- S1a known-beat frac(min-L2 > 60) <= 0.10, AND
- S1b D-beat frac(min-L2 > 60) >= 0.80.
Control: D-60a control (424242: known 0.68, D 0.46) + fresh controls
for the other seeds. Any other outcome: stop, record the negative.

Stage 2 (survival gate, ONLY if stage 1 passes): `D59_REFLEX=1
D50_MODE=1 D60B_PARITY=1 ./target/release/evolve` (3 seeds x 8 gens):
mean `reflex=` column >= 0.8 through generations.

Stage 3 (motor consequence, ONLY if stage 2 passes): clean vs fault
cons= (EVO_BEATS=100): faulted cons= > clean cons= by >= 0.2 AND
clean cons= <= 0.2.

## Predictions / failure modes

- P1: input-afferent slow-state carry-over is the driver -> S1a
  passes (known collapses <= 60), S1b passes (D stays novel).
- F1: known stays noisy -> the variance is NOT slow-state at all (e.g.
  adaptation/latch dynamics of the input neurons, or the pool's
  recurrent drive back onto... note: input neurons have no presynaptic
  recurrent synapses, but V2/M2 normalization may re-scale) ->
  negative, the slow-state line closes with two falsified arms.
- F2: known collapses but D also collapses (<= 60) -> the band reads
  input-drive MAGNITUDE only; D's 6ch drive sits inside the known
  band in-life -> the d50 alphabet is not separable by this band,
  negative, stop.
- No post-hoc tuning of any constant (repo rule).

## Results (appended after runs)

Stage 1 (3 seeds x 1 gen, D50_MODE=1, D59_DEBUG=1):

| seed | control known / D frac(>60) | D60B_PARITY known / D frac(>60) |
|------|------------------------------|----------------------------------|
| 424242 | 0.68 (53/78) / 0.46 (11/24) | 0.68 (53/78) / 0.46 (11/24) |
| 9001 | 0.66 (41/62) / 0.70 (14/20) | 0.66 (41/62) / 0.70 (14/20) |
| 20260912 | (no control) | 0.62 (49/79) / 0.74 (23/31) |

S1a bar: known <= 0.10. FAIL (0.62-0.68). Verdict-for-verdict
IDENTICAL counts vs control on both controlled seeds: clamping the
input-afferent neurons' u_slow is a LITERAL no-op. Mechanism
evidence: input-class neurons relay channel delivery
unconditionally (no threshold decision at delivery), so their slow
state never has a path into the band's input. D-60b falsified; the
slow-state line closes with TWO falsified arms (band: D-60a, input:
D-60b).

Remaining candidate sources of the in-life band variance (NOT
registered, evidence-tagged for the next decision): the band's own
residual state chain (g_drive afferent-drive EMA scaling the
identical train, i_syn tail, refractory/latch carry-over) or V2
machinery renormalizing the band's afferents mid-life (M2
normalization / M3 candidate writes onto band neurons), or template
staleness (symbol intervals 3-10 beats) against a band that is
intrinsically magnitude-only. Next step needs its own frozen doc.

Status: NEGATIVE (Arm B).