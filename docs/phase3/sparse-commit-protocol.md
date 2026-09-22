# Phase III-A frozen registration — sparse-commit: selective afferent dropout

Status: FROZEN PROTOCOL, 2026-09-22 (Phase III mission). Authority:
docs/phase3/diagnostic-and-fork.md (the responder-overlap diagnostic
that uniquely requires expression-side structural separation).
Builds on the E-nogain base (docs/x-phase2-ar-protocol.md, result
records). No implementation yet; E-number withheld per the standing
integrity-review rule.

## 0. Question

Does selective afferent dropout under local commitment produce DISJOINT
neural responder sets, restoring blocked-order selective re-expression
(and, by construction, keeping the alternation result) under the
E-nogain formation-plus-addressing base?

## 1. Mechanism delta (behind `d_core` + `d_claim`; flag `d_sparse`)

Local commitment threshold: once a neuron's protected track-t share
R_t = P_t / (P_0 + P_1) crosses `theta_commit`, the neuron is COMMITTED
to track t. From then on, at each window end (the existing cadence), any
WORKING afferent of the OTHER track (track != t) that is unclaimed
(track == 2) OR claimed to the other track is routed: its weight is
driven multiplicatively toward the churn floor (decay factor `floor_drop`
per window) and it becomes M4-prune-eligible (the E churn-exemption is
lifted for the other-track afferents of a committed neuron ONLY). The
committed track's own afferents are untouched; the committed track's
protected mass is never reduced. This is the ONLY change over E-nogain.

Signals used (all local, existing): R_t (per-neuron protected share),
the synapse's track tag, the weight, theta_prune (floor constant).
NO labels, NO channel-group membership test, NO order/count, NO global
state; threshold and floor are derived constants (below).

Parameters (`theta_commit`, `floor_drop`):
- theta_commit: the commitment point. Derived, not tuned: the measured
  protected-share at which a track's assembly is "established" —
  take 0.5 (majority) for symmetry at K=2 (each memory claims ~N/2
  neurons when drives balance). Pre-registered; a single value.
- floor_drop: multiplicative decay toward theta_prune per window for
  other-track working afferents of committed neurons. Take 0.9/window
  (≈ 0.5 in ~7 windows — a few presentations), matching the measured
  churn timescale and the existing decay family scale. Pre-registered.
  theta_prune is reused as the floor (M4 threshold).

Resource/capacity: pool splits across committed tracks; with K=2 and
theta_commit=0.5, expected ~26/26 disjoint assemblies (finite,
explicit). Same total budget/caps as E-nogain; no new storage beyond
the existing tags; per-neuron working budget unchanged.

Locality boundary: decision per (neuron, synapse): reads P_0,P_1 (own
incoming), the synapse tag, theta_prune, theta_commit, floor_drop.

## 2. Curriculum/arms

Same E-nogain matrix (this is a single-change control on the SAME 9
configs, via `d_sparse=true`):
- arm 0: the committed E-nogain runs (no new runs) — control.
- arm 1 (sparse-commit): 9 runs {bac, bca, il} x {20260912, 424242,
  9001}, d_sparse=true, everything else identical (incl. the S3 probe).
- arm 2: repeated-seed reproducibility (seed 20260912 bac/bca/il x2
  extra runs) with d_sparse to estimate run-to-run variance.

## 3. Identity / integrity

- Flag-off byte-identity: rerun of the committed anchor (flag off) vs
  the 153305Z baseline (event FNV + snapshots). d_sparse path
  unreachable flag-off (skip-serialize semantics identical to E).
- Unit tests: commitment trigger at R_t >= theta_commit; other-track
  working afferents of committed neurons decay toward floor and become
  M4-prune-eligible; committed-track afferents untouched; protected
  mass invariant; determinism no-RNG; flag-off no tag/no drop.
- Zero-exposure regularity (E's floor semantics unchanged): the
  d2a-ident-sil comparison reused with d_sparse.

## 4. Endpoints (frozen)

PRIMARY — blocked-order selective re-expression (the canonical test):
ρ(A) = cos(v_A_late, v_A_ref) − cos(v_A_late, v_C_ref) and ρ(C), over
the S3 re-exposure blocks (as in x-phase2-ar §5), bar ρ >= 0.10;
S1 remains a gating precondition (must still pass 6/6 at the
midpoint/drive-end convention — sparse-commit must not regress
formation).

SECONDARY — mechanistic (the point of the fork):
- responder-set Jaccard between A- and C-re-exposure responder sets
  < 0.5 (must drop from the measured 0.98-1.00);
- alternation ρ retained (≥ the E-nogain values within variance);
- per-track protected mass balance (26±10 / 26±10 expectation),
  per-track cap invariants R-track-1..3;
- stability: failures = [] in every run (a sparse-triggered runaway
  records a FAIL cell, reported);
- allocation check: second-block formation must not starve (S1 6/6),
  measured as in E-nogain;
- F1-il raw-mass preservation for the il arm.

## 5. Failure modes registered a priori

1. OVER-COMMIT: A claims > ~40 of 52 neurons (R_0 crosses 0.5 on most
   neurons) -> C starves (allocation failure, S1 fails) — the classic
   signature; report per-commitment counts.
2. UNDER-COMMIT: too few neurons commit (R_t stays < 0.5) -> no
   dropout -> Jaccard stays ~1 -> same as E-nogain (no separation).
3. DISJOINTNESS WITHOUT BOTHNESS: responder sets disjoint but S1
   formation fails (the two halves can't both support formation) —
   a merge/fragmentation outcome.
4. FLOOR INTERACTION: other-track dropout driving weights below
   theta_prune causes M4 to remove synapses the SECOND pattern needed —
   the allocation trap; reported via per-cohort prune counts.
5. STABILITY: dropout-driven activity shifts -> runaway; report.

## 6. Execution order

1. This protocol committed. 2. Implement d_sparse behind d_core+d_claim
   (identity-first). 3. Suites + unit tests. 4. Identity runs. 5. 12-run
   matrix (9 + 3 repeat). 6. Measurement (S1 gating, ρ, Jaccard, caps,
   allocation). 7. Verdict; frozen. 8. Autonomous next (ladder Level 4).

No post-hoc tuning; all runs preserved incl. failures; Phase I/II
immutable; tree clean.

STOP — protocol frozen.