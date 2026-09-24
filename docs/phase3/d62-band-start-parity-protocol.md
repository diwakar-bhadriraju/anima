# D-62: Band Start-State Parity (registered 2026-09-24)

Status: REGISTERED (written BEFORE any code change; results appended after).

## Question

D-61 fixed the projection drift (known frac 0.62-0.68 -> 0.05-0.26)
but the D-side stayed short of the bar (0.35-0.71 vs 0.80) and known
beats still show 0.05-0.26 residual noise. Diagnostic: D-61 exclusion
+ the D-60a band u_slow clamp gave seed 9001 known 0.20 (11/56) and
D 0.94 (16/17) — the band's remaining START STATE (v residual at beat
start, z_latch — latch_enable=true — and any i_syn tail) shifts the
integration of the identical train per history. Hypothesis: resetting
the band's full start state at beat end makes its response a pure
deterministic function of the train, restoring the D-58 prototype
regime (knowns read familiar, D reads novel) IN-LIFE.

## Intervention (frozen)

`D62_PARITY=1` (identity when unset): at the END of each beat's tick
loop, reset the reflex band's full start state:
u_slow = 0, v = LIFParams.v_rest, z_latch = 0, i_syn = 0.
Composes with the D-61 readout exclusivity (active by default when
the band exists; D61_EXCL=0 gives the fix-only control). D60_PARITY
(u_slow-only) is NOT part of this run. All constants (K=8, th=60)
untouched.

## Falsifier (staged, frozen — same bars as D-61)

Stage 1 (3 seeds x 1 gen, D59_DEBUG=1, D50_MODE=1, D62_PARITY=1):
- S1a known-beat frac(min-L2 > 60) <= 0.10, AND
- S1b D-beat frac(min-L2 > 60) >= 0.80.
Control = D-61 fix-only measurements (D 0.71/0.60/0.35; known
0.05/0.26/0.09) — the same binary with D62_PARITY unset.
Any other outcome: stop, record the negative.

Stage 2 (survival gate, ONLY if stage 1 passes):
`D59_REFLEX=1 D50_MODE=1 D62_PARITY=1 ./target/release/evolve`
(3 seeds x 8 gens): mean `reflex=` >= 0.8 through generations.

Stage 3 (motor consequence, ONLY if stage 2 passes): clean vs fault
cons= (EVO_BEATS=100): faulted cons= > clean cons= by >= 0.2 AND
clean cons= <= 0.2.

## Predictions / failure modes

- P1: start-state carry-over is the residual driver -> S1a collapses
  to ~0, S1b passes (the D-58 regime restores in-life).
- F1: known collapses but D still < 0.80 -> the FIXED band genuinely
  lacks D-vs-known separation in the d50 6ch alphabet (magnitude
  overlap) -> the symbol-world reflex line closes as a recorded
  negative; the 3D retina (D-70) becomes the novelty source.
- F2: known stays noisy -> an additional variance source (input
  neurons' relay reliability) -> audit, negative.
- No post-hoc tuning of any constant (repo rule).

## Results (appended after runs)

Stage 1 (3 seeds x 1 gen, D50_MODE=1, D59_DEBUG=1, D62_PARITY=1):

| seed | D-61 fix-only known / D frac(>60) | D-62 known / D frac(>60) |
|------|-----------------------------------|--------------------------|
| 424242 | 0.05 (4/79) / 0.71 (17/24) | 0.29 (20/68) / 0.90 (19/21) |
| 9001 | 0.26 (16/62) / 0.60 (12/20) | 0.28 (16/58) / 0.94 (16/17) |
| 20260912 | 0.09 (6/70) / 0.35 (9/26) | 0.11 (9/84) / 0.91 (29/32) |

S1b (D >= 0.80): PASS all seeds (0.90/0.94/0.91) - the D-58
state-matched regime IS restorable in-life. S1a (known <= 0.10):
FAIL (0.29/0.28/0.11). Stage 1 FAILS the frozen conjunction -> gate
NEGATIVE. Note: the full start-state reset made known beats NOISIER
than fix-only on 424242 (0.05 -> 0.29): the clean band integrates the
afferent spread unfiltered.

REMAINING SOURCE (code-verified, not tuned): mid-life births. In
evolve, mon.wiring_bidirectional=true adds newborn->partner synapses
for the 20 highest/lowest-rate partners - the band (Output class) is
eligible; the D-61 purge only runs at V2Plasticity::new (life start),
so births re-drift the band's afferents DURING life. D-63
(registration: extend readout exclusivity to the structural birth
machinery, same control env) is the close-out; its falsifier = this
stage's bars (control = the D-62 rows above).

Status: NEGATIVE (stage 1), mechanism direction CONFIRMED (D-side at
bar, residual known-side cause identified in code).