# D-64: Afferent Relay Start-State Parity (registered 2026-09-24)

Status: REGISTERED (written BEFORE any code change; results appended after).

## Question

D-63 achieved PERFECT D-side separation (1.00 all seeds) but known
beats still carry 0.09-0.11 noise (bar 0.10). The last hypothesized
source: the input-afferent RELAY neurons' membrane state. Input frames
deliver channel spikes onto the relay neurons (class Input, ids
0..channels.len()); a relay that is mid-refractory or above-threshold
at a delivery can miss/alter the spike, so the band's actual afferent
train is a state-dependent distortion of the emitted train (D-60b
ruled out their u_slow; v/refractory remain). The relay state at beat
start varies with the previous beat's history -> jitter -> the
occasional >60 L2 on known beats.

## Intervention (frozen)

`D64_PARITY=1` (identity when unset): at the END of each beat's tick
loop, reset the input-afferent neurons' start state:
v = LIFParams.v_rest, refractory_until = Tick(0), u_slow = 0
(ids 0..channels.len()). Registered body-level semantics: the pool's
afferent input becomes EXACTLY the emitted trains (deterministic
relay), which is also the channel design the 3D retina will use.
Composes with D62_PARITY (band reset) + D-61/D-63 exclusivity (all
previously registered pieces ON). Any other combination is a
different registration.

## Falsifier (staged, frozen — same bars as D-61/D-62/D-63)

Stage 1 (3 seeds x 1 gen, D59_DEBUG=1, D50_MODE=1, D62_PARITY=1,
D64_PARITY=1):
- S1a known-beat frac(min-L2 > 60) <= 0.10, AND
- S1b D-beat frac(min-L2 > 60) >= 0.80.
Control = D-63 rows (known 0.09-0.11; D 1.00) — this run's D-side is
expected to stay 1.00.
Any other outcome: stop, record the negative.

Stage 2 (survival gate, ONLY if stage 1 passes):
`D59_REFLEX=1 D50_MODE=1 D62_PARITY=1 D64_PARITY=1 ./target/release/evolve`
(3 seeds x 8 gens): mean `reflex=` >= 0.8 through generations.

Stage 3 (motor consequence, ONLY if stage 2 passes): clean vs fault
cons= (EVO_BEATS=100): faulted cons= > clean cons= by >= 0.2 AND
clean cons= <= 0.2.

## Predictions / failure modes

- P1: relay start-state is the whisker -> known <= 0.10 on all seeds,
  D stays 1.00; stage 1 passes and the symbol-world reflex line CLOSES
  as a registered PASS.
- F1: known stays 0.09-0.11 -> the residual is elsewhere (audit: the
  emitted-train equality itself? latched pool feedback toward the
  band? band latent state beyond the D-62 reset) -> negative, stop.
- F2: stage 1 passes but stage 2 fails -> detection does not survive
  growth at these constants -> recorded negative for the gate.
- No post-hoc tuning of any constant (repo rule).

## Results (appended after runs)

WITHDRAWN — PREMISE FALSIFIED BEFORE MEANINGFUL EVALUATION.

Code evidence (network.rs:1138-1148): `deliver_input` pushes the
relay target into the spike list UNCONDITIONALLY for every delivered
channel spike ("input neurons spike exactly when the frame says so —
no state, no drift", D8 semantics). There is no threshold/refractory
path that can miss or alter a delivery; the band already receives the
exact emitted train. Empirically confirmed (the run completed anyway,
as a control replication): D64_PARITY=1 produced verdict-for-verdict
IDENTICAL distributions to the D-63 rows (known 0.11/0.09/0.11, D
1.00/1.00/1.00) — the relay clamp is a literal no-op.

The known-side residual investigated in this registration was a
METRIC-ELIGIBILITY ARTIFACT (see D-63 results update): each symbol's
FIRST occurrence per organism has no own template and legitimately
reads >60 cross-symbol; the observed 0.09-0.11 matches the
first-occurrence ceiling (12 beats per seed) almost quantitatively.
With own-template eligibility, S1a is expected ~0 (recomputed in
D-63's updated results).

Status: WITHDRAWN (premise contradicted by code; no behavior change
retained — the clamp code is reverted; the env gate is inert).

## D-64 anmendment v2 (metric correction)

The D-64 clamp code was removed from survival.rs (no-op). The
MISREGISTERED premise and its falsification are recorded here for
the audit trail. The D63-known-residual question is closed by the
eligibility correction (D-63 doc).