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
---

## E23 execution record (2026-09-19)

Implementation `ca185aa` (2 configs, hashes pre-run; identical
stimulus streams — same seed, arms differ only in the loop).
Runs e23-closed/e23-open-20260918T1817{38,40}Z. 140 tests green,
0 warnings; analysis deterministic; schedule lag-1 -0.565
(anti-correlated seeded shuffle — balanced, no exploitable
structure); 2 NoAction trials each arm (0.5%).

### Results (vote [100,500) of stimulus; groups 64-69/70-75)

CLOSED arm:

| window | P(g1|A) | P(g1|C) | BD | BR | out g1/g2 | D_stim |
|---|---|---|---|---|---|---|
| E | 0.800 | 0.800 | 0.000 | 0.500 | 6332/4309 | 0.5694 |
| M | 0.200 | 0.900 | 0.700 | 0.150 | 4374/3610 | 0.8494 |
| L | 0.000 | 1.000 | **1.000** | 0.000 | 4154/3165 | 0.9219 |

OPEN arm (no consequences):

| window | P(g1|A) | P(g1|C) | BD | BR | out g1/g2 | D_stim |
|---|---|---|---|---|---|---|
| E | 0.300 | 0.850 | 0.550 | — | 7479/6973 | 0.4761 |
| M | 0.850 | 1.000 | 0.150 | — | 6472/4643 | 0.9859 |
| L | 0.950 | 0.850 | 0.100 | — | 5978/4471 | 0.9993 |

Effect of consequences: BD_closed(L) - BD_open(L) = 1.000 - 0.100
= **0.900 > 0.30 CRITERION MET**. Internal: D_stim_closed(L) -
D_stim_open(L) = 0.922 - 0.999 = **-0.077 (no gain; open arm's
stimulus representations MORE separated)**.

Direction check (the crucial pattern): the closed arm's votes
moved from E's neutral 0.8/0.8 (BD 0.0, BR 0.5 = chance) to L's
PERFECT ANTI-MAPPED votes: A -> group 2 (P(g1|A)=0.000), C ->
group 1 (P(g1|C)=1.000) — BD = 1.0 with BR = 0.0. The world's
registered mapping is A -> g1 correct; the organism converged to
the EXACT OPPOSITE mapping: every closed-arm trial after
mid-development chose the punished action. The open arm wandered
(BD_L = 0.1, benign 81 — vote-logged only).

### Shortcut/leakage audit

Vote window [100,500) reads stimulus-driven activity (the honest,
registered property); consequence strictly after the vote
(cannot contaminate); no reward/teacher channel; balanced
antecedents; deterministic arms; disruption channels virgin. No
information beyond current stimulus + consequences reaches the
organism. Shortcut analysis: fixed-action ruled out (BD 1.0 with
both actions used conditionally); alternation ruled out
(antecedent-conditioned); quiescence absent (2/200). The
anti-correct convergence is NOT a leak — it is a systematically
learned INVERSION of the world's mapping.

### Verdict: GRID (B) — behavioral effect without internal-
### representational gain... with a critical qualifier

The preregistered effect criterion is met (0.900 >> 0.30): the
votes became perfectly antecedent-conditioned UNDER consequences
and only weakly conditioned without them. But the direction is
ANTI-CORRECT: benign rate fell to 0.0 — the closed-loop organism
reliably chose disruption. Interpretation (bounded): consequences
powerfully shaped stimulus-conditioned response routing (the
reflex-shaping primitive WORKS — this is the first demonstration
in the E-series that closed-loop consequences alter behavior),
but the learned routing is inverted relative to the world's
beneficial mapping. Plausible mechanism hypotheses (NOT tested
here, flagged for future work): disruption on 16-23 drives
channels that share M1 input wiring with... C-side (8-15) is
closer to 16-23 than A-side (0-7) in channel space; punishing a
wrong A-trial (which voted g1) delivers 16-23 drive that
potentiates g1-correlated paths — i.e., the punishment itself may
reinforce the punished mapping via Hebbian co-activity, a
classic sign-reversal artifact of punishment-only regimes in
purely Hebbian substrates. No mechanism claim made.

### What E23 establishes / does not

ESTABLISHES: consequences can shape stimulus-driven output
routing in the frozen organism (behavioral plasticity under
closed-loop pressure — affirmative); the shaping can invert
relative to the world's benefit mapping (punishment-only regimes
in this substrate do not implement gradient-following). DOES NOT:
anything about retention/memory (no gap); anything about reward
(no reward signal existed); mechanism identity of the inversion.
