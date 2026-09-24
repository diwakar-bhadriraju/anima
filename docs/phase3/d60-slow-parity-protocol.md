# D-60a: Reflex-Path Slow-State Parity (registered 2026-09-24)

Status: REGISTERED (written BEFORE any run; results appended after).

## Question

D-59 measured the in-life reflex signal to be noise-dominated: known
beats sit at min-L2 61-370 from their OWN just-overwritten templates,
D beats span 0.0-187 including exact template matches; distributions
fully overlap at th=60 (docs/phase3/d59-reflex-integration-protocol.md
#Diagnosis). Hypothesis: the reflex band's response to the IDENTICAL
input train varies by history because the band neurons' OWN slow
depolarization (slow_state, tau 5000 ms) retains 74% of its plateau
across the 1500 ms inter-beat gap (e^-1500/5000), shifting effective
thresholds per beat. The D-58 prototype's 3/3 came from immediate
state-matched capture; per-beat templates cannot outrun the carry-over.

## Intervention (frozen)

`D60_PARITY=1` (identity when unset): at the END of each beat's tick
loop (before decode), clamp `u_slow = 0.0` on the reflex band neurons
only ([base, base+k), the band's own slow state). The organism's pool
and the band's afferent input neurons are UNTOUCHED (minimal,
mechanism-isolating: the band reads input channels only, so pool
state cannot reach it directly). No other change; D-59 machinery and
constants (K=8, th=60, REFLEX_TH_FAM) unchanged.

Arm A = band-only clamp (this plan). If Arm A fails the falsifier,
Arm B (input-afferent u_slow clamp) is a SEPARATE registration; it is
NOT run automatically.

## Falsifier (staged, frozen)

Stage 1 (distribution, the mechanism test; 3 seeds x 1 gen,
D59_DEBUG=1, D50_MODE=1): with D60_PARITY=1,
- S1a known-beat frac(min-L2 > 60) <= 0.10 (margin restored: knowns
  read FAMILIAR against their own templates), AND
- S1b D-beat frac(min-L2 > 60) >= 0.80 (D stays NOVEL).
Control: same run with D60_PARITY unset must reproduce the D-59 noise
(frac ~0.7-0.8) - proves the comparison is apples-to-apples.
Any other outcome: stop, record the negative (carry-over is not the
mechanism, or the band never carried symbol information in-life).

Stage 2 (survival gate, ONLY if stage 1 passes): D-59 gate rerun
`D59_REFLEX=1 D50_MODE=1 D60_PARITY=1 ./target/release/evolve`
(3 seeds x 8 gens): mean `reflex=` column >= 0.8 through generations.

Stage 3 (motor consequence, ONLY if stage 2 passes): clean vs fault
cons= comparison (EVO_BEATS=100): faulted cons= must exceed clean
cons= by >= 0.2 AND clean cons= must be <= 0.2 (the always-firing
0.68-0.78 baseline must collapse to near-zero with the band stable).

- P1: u_slow carry-over is the noise driver -> S1a passes (known
  min-L2 collapses <= 60), S1b passes (D stays novel).
- F1: distribution unchanged -> the variance comes from the afferent
  input neurons' state or Poisson/beat-context effects, not the band's
  own slow state -> negative finding, Arm B is the next (separate)
  registration.
- F2: known collapses but D also collapses -> the band responds to
  input-drive MAGNITUDE only; D's 6-channel drive is inside the known
  band -> the D-58 prototype's separation does not generalize to the
  d50 6-channel alphabet in-life -> negative finding, stop.
- No post-hoc tuning of any constant (repo rule).

## Results (appended after runs)

Stage 1 (seed 424242, 1 gen, D50_MODE=1, D59_DEBUG=1):

| condition | known frac(minL2>60) | D frac(minL2>60) |
|-----------|----------------------|------------------|
| control (parity unset) | 0.68 (53/78) | 0.46 (11/24) |
| D60_PARITY=1 (band u_slow clamped) | 0.65 (55/84) | 0.74 (20/27) |

S1a bar: known <= 0.10. FAIL (0.65 ~= control 0.68). The band's own
slow-state carry-over is NOT the beat-to-beat variance driver; the
clamp changes nothing material on the known side. Per the frozen
decision rule, STOP here: Arm A falsified. (S1b improved 0.46 -> 0.74
but the stage-1 conjunction is broken by S1a; no bar adjustment.)

Status: NEGATIVE (Arm A). Arm B (input-afferent u_slow clamp) is a
separate registration if pursued - it changes what the whole organism
receives (the pool reads the same input neurons), so it is a bigger
intervention and needs its own frozen protocol doc.