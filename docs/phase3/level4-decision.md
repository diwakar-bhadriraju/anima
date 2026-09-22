# Phase III Level-4 decision + feasibility — temporal model branch

Status: DECISION + FEASIBILITY RECORD, 2026-09-22 (Phase III autonomy).
Instrument examples/gapstate.rs over committed E-nogain runs.

## D-06 resolution: Level-4 temporal branch (selected over sparse-wiring)

Rationale (evidence-based, not architecture-preservation):
- Three independent single-pool mechanisms (CLLA budget, D-core claim,
  sparse dropout) each captured by the first pattern under blocked
  order; sparse-commit additionally PROVED the access-vs-separation
  tension is structural (Jaccard 0, S1 9/9 fail). The blocked-order
  single-pool re-expression gap is a bounded, well-understood
  limitation — refining it further (sparse-wiring fork) does not climb
  the mission ladder.
- The ultimate organism objective is temporal: relationships,
  prediction, action, closed loop. Level 4-5 (temporal model,
  prediction) is the strongest un-attempted capability and the
  platform (E-nogain formation + alternation retrieval) is ready.
- Therefore: advance to Level 4. Sparse-wiring is recorded as the
  lower-value alternative (only refines Level 3).

## Feasibility probe (measured; gapstate on E-nogain il runs)

1. CROSS-GAP ACTIVITY EXISTS and is substantial: internal spikes in the
   1500 ms gaps ~1200 (s20260912-il), ~300 (s9001-il), ramping then
   plateau — NOT the near-zero decay the V2.1-era G6 regime implied.
   The E-nogain recurrent pool sustains persistent cross-gap firing.
2. PREDECESSOR-DISTINCTNESS: LATE-gap internal state differs after A
   vs after C in 2/3 seeds: within-A cosine 0.955-1.000 vs cross-A-C
   cosine 0.625 / 0.759 (s9001, s424242) — the plateau state carries
   "what just happened". (s20260912: cross 0.977 = weak/absent.)

Interpretation: the substrate's own dynamics provide a temporal bridge
(per-neuron gap state reflecting the preceding event) — the
prerequisite for prediction. It is PARTIAL (1/3 seeds weak) and it is
activity-based (the state is firing, not stored structure); whether it
can be shaped into PREDICTION (anticipation of the NEXT event) via a
transition-learning mechanism is the Level-4 experiment.

## Level-4 minimal design (frozen sketch; next iteration implements)

TASK (paired-associate temporal prediction, deterministic transition):
the il schedule IS a deterministic alternation (…, X, Y, X, Y, … with
Y = flip(X)). Prediction = after presenting X, during the gap the
organism's state should anticipate Y. Measure (no readout training):
during the late gap after X, cosine of the gap state against Y's
learned response reference MINUS against the other pattern's reference
— a "prediction index" Π(X) = cos(gap_after_X, Y_ref) −
cos(gap_after_X, X_same_ref). Positive Π = the gap carries
anticipation of the imminent next stimulus.

MECHANISM (minimal, local, finite): a per-neuron eligibility trace
modulating next-event plasticity. When event Y arrives while the X-gap
state is still active, LTP of Y's afferent/recurrent synapses is
scaled by the neuron's recent activity (eligible if it fired in X's
gap). This makes the X-gap state strengthen the Y-response path, so
future X-gap states pre-activate Y. One new per-neuron state (a
bounded decaying eligibility e_i, tau_E = the gap timescale ~1-2 s,
derived from the measured plateau persistence), one plasticity term,
zero new labels (the transition is learned from co-occurrence of gap
state and next event). Flag-off identity; E-number withheld.

FALSIFIER: if Π stays ≈ 0 after training (or the gap state cannot be
shaped toward the successor across seeds), temporal prediction is not
achievable by this route and the mechanism is rejected (no patch).

## Decisions (D-07)

1. Branch selected: Level 4 temporal/prediction. Rationale above.
2. Feasibility: endogenous predecessor-distinct gap state confirmed in
   a majority of seeds — a real substrate, not an injection.
3. Next iteration: implement the eligibility-trace mechanism +
   paired-associate protocol (freeze -> gate -> execute -> audit);
   measure Π and reject/adopt on evidence.

STOP — decision + feasibility recorded; branch in progress.