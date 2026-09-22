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
## D-2026-09-22-02 — E-nogain milestone + generalization; blocked-ρ gap prioritized

- OBSERVATION: E (with rg8c gain) forms but aborts 4 cells; E-nogain
  9/9 complete, S1 (blocked) 6/6, alternating re-expression ρ 0.25-0.77.
- OBSERVATION: clean-PN generalization (novel 16-23 group @ A statistics)
  forms S1 6/6 and alternates ρ 0.36-0.91 — NOT A/C-overfit. The earlier
  "AB" probe was confounded (config "B" = 40 Hz phase-variant over 4-11,
  overlapping A's channels); discarded, not evidence.
- H3 (probe re-learning) RULED OUT: blocked ρ flat across 5 re-exposure
  windows.
- DECISION: blocked-order re-expression is the highest-evidence gap
  (shared-pool recency signature: C/Pn re-exposure lights up both
  assemblies). Test H2 (recurrent-claim capture) next via an
  afferent-only-claim gated variant; then H1 if unchanged.

## D-2026-09-22-03 — blocked-ρ = expression-layer recency; checkpoint D-04

- OBSERVATION: per-track totals BALANCED at drive end in blocked runs
  (T0/T1 ≈ 8.97/9.22 for s20260912-bac E-nogain); second-block AFF mass
  ~2x first; blocked ρ contamination = both assemblies respond to one
  pattern (C re-exposure cosA~0.9 AND cosC~0.95) — shared
  membrane/recurrent pool recency (G3 superposition localized to the
  EXPRESSION layer). Budgets+addressing fix persistence/formation; the
  read is still pool-mixed when one memory dominates.
- DECISION: this exponential-family milestone (formation S1 6/6 × 2
  channel pairs; alternation ρ 0.25-0.91 general) is consolidated as a
  checkpoint; the next loop (D-04) targets blocked-order re-expression
  via an expression-layer intervention (per-track recurrent protection
  or a read gate), then capacity (K≥3), then closed-loop.

## D-2026-09-22-04 — sparse-commit fork (Phase III-A) frozen

- OBSERVATION (measured, respoverlap): blocked-order re-exposure
  responder sets are the ENTIRE pool, Jaccard 0.98-1.00, zero
  recurrent-only neurons -> dense p_in=0.5 pool has no structural
  separation; A/C expressions differ only by rate modulation. The
  alternation ρ win is rate-modulation too, not membership.
- HYPOTHESIS: selective re-expression under recency REQUIRES disjoint
  (sparse-committed) assemblies; budgets+addressing fix persistence/
  formation but not expression membership.
- DECISION: fork to sparse-commit (selective afferent dropout under a
  local commitment threshold R_t >= theta_commit=0.5), a redesign not a
  patch; the diagnostic uniquely demands it. Freeze
  docs/phase3/sparse-commit-protocol.md; implement next.
- REJECTED: further E-nogain patching for blocked-ρ (patch-spiral
  guard); moving up the ladder before closing this Level-3 gap (a
  shaky retrieval base).
- OPEN: does sparse-commit give Jaccard < 0.5 and blocked ρ >= 0.10
  without starving the second block (over-commit risk #1)?
