# ANIMA autonomy log

Status: maintained under the autonomous research charter (2026-09-22).
Structure per decision: observation → hypothesis → evidence → decision →
rejected alternatives → experiment → result → interpretation → open q.
Distinction kept: MEASURED vs INTERPRETATION vs HYPOTHESIS vs DECISION.

## D-2026-09-22-01 — First-contact claim (candidate E) as the next formal step

- OBSERVATION (measured): Phase II-A S1 FAIL 6/6. M1 audit split the
  failure into (a) preserved-but-MISADDRESSED cells (s20260912 ×2,
  s9001-bac: surviving second-cohort working mass 0.289–0.543, 100%
  tag-0, first-C track-1 current exactly 0 — the recruitment gain muted
  by the wrong tag) and (b) UNPRESERVED cells (s424242 ×2: surviving
  mass 0.000 — churn killed the substrate before exposure).
- HYPOTHESIS (from the audit counterfactual): local re-keying at first
  exposure lifts first-exposure drive to A-parity (~9.5/neuron) and
  unmutes the capped gain, crossing the bootstrap in the preserved
  cells — PROVIDED the surviving mass is normalized under the per-track
  regime (capacity-matched T_1), not the Phase-I single budget.
- DECISION: implement candidate E behind `d_core`: M1 synapses start
  UNCLAIMED (tag 2); first-contact claim rule; churn-exempt dormancy
  (floor θ_prune, M4-exempt) for unclaimed; two-regime M2
  (per-track targets for claimed work, budget-capped floor for
  unclaimed); all existing D-core mechanisms retained; a pre-registered
  S3 re-expression probe added (amendment). Freeze as
  docs/x-phase2-ar-protocol.md.
- REJECTED AT THIS STEP: D-only (no floor — leaves s424242 dead);
  C (duplication reintroduces superposition); A (static — falsified);
  a "defer per-track while unclaimed" M2 (would kill the predicted
  upscale); separate II-B registration (in-run S3 probe is the
  efficient path under the charter).
- EXPERIMENT: frozen x-phase2-ar. 9 new runs (3 orders × 3 seeds) +
  identity; endpoints S1 (6/6), S-a..g, S3 ρ(A)/ρ(C).
- RESULT: PENDING.
- OPEN: does E cross S1; does ρ>0 under either blocked order or
  alternation; do gain-regime aborts persist.