# ANIMA E23 — Reflex shaping (closed vs open loop)

Status: **FROZEN** (2026-09-19). Design `7b6acb0`, approved with
decisions: open-loop CO-EQUAL arm; vote window [100,500) of the
stimulus epoch; seed 20260912 primary.

## 1. Question

Can environmental consequences shape any output behavior when NO
temporal retention is required — can consequences shape a purely
stimulus-driven response routing under the frozen architecture?

## 2. Design (two arms, co-equal)

Trial (2000 ms cadence): stimulus A {0-7} or C {8-15} (committed,
20 Hz, 500 ms) -> consequence epoch [500,1000) -> ITI 1000.
- CLOSED arm: vote = majority of output groups 64-69 vs 70-75 over
  the stimulus epoch's [100,500) (first 100 ms excluded — response
  ramp; envelope shows sustained output 13967..8405 per 100 ms);
  A -> group 1 correct, C -> group 2; match -> silence, mismatch/
  no-action -> disruption (channels 16-23 @ 80 Hz, 500 ms, trains
  run-seeded in trial order); world = E19/E20 law verbatim.
- OPEN arm: identical stimulus schedule and vote readout (votes
  logged), consequence ALWAYS silence.
200 trials each; balanced 20/20 per 40-trial window; seed 20260912
(both arms same seed -> identical stimulus streams; the world is
the only difference). No probe, no gap, no hidden state, no
retention demand. No labels/reward: the A->g1/C->g2 mapping exists
only in the world.

## 3. Measurement (per arm, E/M/L = 1-40 / 81-120 / 161-200)

- BD(window) = |P(vote=g1|A) - P(vote=g1|C)| (no-action excluded,
  reported); BR (closed only); no-action rate; vote spike counts
  per group.
- Internal: stimulus-response divergence D per arm (A-epoch vs
  C-epoch internal vectors, L window; E18 metric); A-C sanity;
  output-neuron per-group rate maps across windows; bucket
  organization at block boundaries (E15 machinery).
- Gates: failures 0, max rate <= 250, permanence > 0; schedule
  audit; determinism.

## 4. Criteria + verdict (pre-registered)

- Effect of consequences := BD_closed(L) - BD_open(L); criterion
  > 0.30 (carried binomial margin).
- Verdict grid: (A) effect met AND internal divergence gain
  (D_closed(L) - D_open(L) > 0.05) = reflex shaping with
  representational change; (B) effect met, no internal gain;
  (C) internal gain, no behavioral effect; (D) neither (strong
  negative: consequences shape nothing, even reflexes); (E)
  shortcut/leak (protocol failure); (F) degenerate (no-action >
  0.5 in L in the closed arm — a genuine policy outcome here
  since activity exists in-window; if telemetry shows ~0 vote-
  window output spikes, reclassify as interface failure).
- Shortcut/leakage: fixed-action ceiling 0.5 (balanced);
  alternation fails (seeded random); no timing cue (constant
  template); disruption channels virgin; consequence strictly
  after the vote window (cannot contaminate the readout);
  determinism; no reward/teacher channel.

## 5. E23 is NOT

A memory experiment (nothing to retain); supervised classification
(no labels/reward — consequences only); a substrate test.

## Appendix: config hashes (pre-run)

- e23-closed.toml `286abe870b6b59a3`; e23-open.toml `1d92dfc3c495e40e` (recorded pre-run; arms identical except exp_id; world differs only in the closed loop).

## Appendix: amendments

- (none)