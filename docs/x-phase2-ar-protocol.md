# Phase II-AR frozen registration — first-contact claim (candidate E)

Status: FROZEN PROTOCOL, 2026-09-22, under the autonomous research
charter. Authority: docs/x-phase2-m1-audit.md (the exact M1 problem and
the two-condition diagnosis), docs/x-phase2-a-protocol.md (D-core
mechanics it builds on), docs/x-phase2-a-result.md (S1 FAIL 6/6).
This is a formal registration; E-number assignment follows the standing
integrity-review rule (withheld until the gate passes).

## 0. Question

Does candidate E — first-contact claim of pre-existing (M1) synapses
into learned context tracks, with churn-exempt dormancy for the
unclaimed class — restore sequential blocked-order formation (S1) AND,
where S1 passes, give selective re-expression (S3 ρ probe)?

## 1. Mechanism delta over committed D-core (all behind `d_core`)

1.1 M1 INITIALIZATION: at construction, when d_core is ON, every M1
synapse (afferent AND recurrent) initializes track = 2 (UNCLAIMED),
instead of the D-core default 0. Flag OFF keeps track = 0 (identity:
tag 2 never appears, serde skip-when-0 unchanged).

1.2 UNCLAIMED semantics (track == 2):
- not subject to per-track M2 targets;
- not eligible for consolidation (M3 permanence is unaffected —
  permanence claims the new synapse directly to the neuron's current
  context, as in D-core);
- M4/prune exempt (never pruned on low weight);
- post-M2 weight clamped to ≥ θ_prune (0.005, EXISTING constant) —
  the churn floor;
- on first co-activation: tag := the post neuron's current context c*
  (§1.3), after which normal track semantics apply.

1.3 CLAIM RULE (first-contact; same for afferent and recurrent
synapses; at window end inside ctx_update, after c* is chosen): for
each incoming synapse of neuron i with track == 2, if its PRE fired
during the window (afferent: its input channel fired; recurrent: the
internal pre neuron fired), set tag := c*. Deterministic; no RNG; no
new timescale (structural window); the evidence is only "pre fired
while the post was in learned context c*" — no channel-group, no
pattern id, no order/count.

1.4 M2 TWO-REGIME NORMALIZATION (replaces the D-core per-track branch;
flag-off = unchanged):
Let B = t_e − P_tot (per neuron; B ≤ 0 ⇒ skip). Let U = unclaimed
working sum, C_ = claimed working sum (= Σ_t claimed_t). Let floor units
f = min(θ_prune, B / n_unc) per unclaimed (n_unc = count of unclaimed
working synapses; n_unc = 0 ⇒ no unclaimed; B could be < θ_prune·n_unc
⇒ floor scales down, invariant kept). Ordered, deterministic:
  (a) normalize the claimed portion of each claimed track t to
      (B − f·n_unc) / n_claimed (capacity-matched over claimed tracks
      with working mass; n_claimed ≥ 1), scaling only claimed working
      synapses of that track;
  (b) floor-clamp each unclaimed working synapse to f;
  (c) after (a)+(b), if the per-neuron total working sum still exceeds
      B (claimed upscale + floor interaction), rescale ONLY the claimed
      working set down so total = B; if claimed total = 0 (all
      unclaimed) the per-unclaimed floor f already bounds the total to
      B, so no further step (invariant: sum_working ≤ t_e − P_tot at
      every window, checked by R-track-2).
Rejected simpler rule: "defer per-track targets while unclaimed mass
exists" — it would normalize the freshly-claimed survivors under the
Phase-I single budget and prevent the measured track-1 upscale.

1.5 All other D-core behavior retained: per-track caps (p_max·t_e/K +
ε), per-track LTP clip, per-track novelty (1−R_t) + rg8c-capped gain
(on claimed working afferents of an unexplained track), prototypes,
tags, determinism. Recruitment gain gate unchanged (rg8c).

## 2. Curriculum (amendment): in-run re-expression probe

The main matrix runs the committed Phase I schedules (S0 / S1 bac|bca|il
/ S2 20 s silence) unchanged. ADDITIONALLY, a third stage is appended
(a curriculum amendment, explicit — not silent):
- S3: 5 × A (500 ms @ 20 Hz, 1.5 s off), 10 s silence, 5 × C, then
  20 s silence (S2 tail).
This gives, per run, an in-run re-expression probe: the LAST-3-
presentation response vectors of each S3 re-exposure block vs the
S1 late-window references.

Config: the committed clla-fe schedule files are edited by inserting the
S3 stage after S2 ([[stage]] S3). All other fields identical.

## 3. Arms

- arm 0: the 9 committed clla-fe runs (no new runs) — endpoints
  recomputed with the same instruments (S1 bar + ρ(control) for the
  S3 probe is measured in arm 1 only; arm 0 has no S3).
- arm 1 (E): clla-are-s{seed}-{order}.toml = clla-fe + [v2]
  d_core=true, recruit_gain=true (the E mechanism incl. §1). 9 runs:
  {bac, bca, il} × {20260912, 424242, 9001}.
- arm 2 (control for the probe): none new; the D-core runs (clla-d2a,
  committed) serve as the mechanistic control for formation; arm-1
  results are compared against both arm-0 (baseline) and arm-2.

Total NEW runs: 9 experimental + 1 identity (flag-off) + 1 zero-exposure
(automated) = 11.

## 4. Identity / integrity

- Flag-off byte-identity: the harness d_core-off path is byte-exact
  (verification (i) rerun); tag defaults to 0 when the flag is off, so
  flag-off artifacts never contain tag 2 (skip-serialize). Same-seed
  determinism guarantee.
- Unit tests: M1-init tag 2 under d_core; claim rule (pre-fired window
  → tag = c*; pre-not-fired → stays 2); M4-exemption of unclaimed;
  floor clamp + invariant (sum ≤ B) with unclaimed-only and mixed
  neurons; two-regime M2 arithmetic (claimed upscale present, not
  deferred); flag-off no-tag-2.

## 5. Endpoints (frozen)

PRIMARY — S1 (arm 1): second-block protected mass ≥ 0.5 × first-block,
6/6 cells (bac+bca × 3 seeds), exactly the Phase I criterion and
convention (first at t = 44,000, second at drive end t = 84,000).
SECONDARY:
- S-a F1-il preservation (arm-1 il raw masses ≥ 0.5 × per-seed refs);
- S-b stability: failures = [] in every arm-1 run (a gain-regime abort
  records a FAIL cell, reported with the runaway details — the rg8
  lesson is a registered finding, not a retry);
- S-c per-track caps (≤ 0.34) + R-track-1..3;
- S-d total track/floor resource (unclaimed floor ≤ B accounts for it;
  no hidden budget);
- S-e no hidden metadata (static scan: claim/M2/floor read only
  fired sets, tags, P_t, weights, constants);
- S-f identity parity;
- S3 re-expression probe (interpretable ONLY if S1 passes, else
  reported uninformative-preserved):
    ρ(A) = cos(v_A_late, v_A_ref) − cos(v_A_late, v_C_ref) ≥ 0.10,
    ρ(C) ≥ 0.10 symmetric, where v_A_ref / v_C_ref are the mean
    response vectors over the S1 late windows (reps 11–20 of each
    pattern in the order present) and v_A_late / v_C_late over the
    LAST 3 presentations of each S3 re-exposure block; all-zero-vector
    exclusions < 25% (CLLA precedent). δ = 0.10 justified in
    x-phase2-architecture §15.

## 6. Failure modes registered a priori

1. PROTO-BLUR (as before);
2. FLOOR-CAPACITY COLLISION: unclaimed floor eats the claimed budget
   (B small, many unclaimed) ⇒ claimed tracks sub-threshold ⇒ the
   first block itself weakens (S1 first-block shrinks) — reportable
   via per-track working shares;
3. CLAIM-OVERWRITE: a first-fire claim pulls an afferent that "should"
   belong to an earlier context — impossible by definition (first-fire
   = has never co-activated before) but the RUN must verify no
   afferent is claimed twice (invariant: tag monotonically
   2 → {0,1}, never back);
4. GAIN OVERDRIVE: rg8c cap insufficient at claim (S3 first re-exposure
   burst band) — report with pres-1..5 band;
5. RECURRENT CLAIM INSTABILITY: recurrent claim changes recurrence
   topology during exposure (a recurrent synapse flips tag mid-block)
   ⇒ operating point shift — report via recurrence-tag churn rate.

## 7. Execution order

1. This protocol committed. 2. Implement behind d_core (control-path
   identity first). 3. Suites + unit tests. 4. Identity runs (flag-off
   vs committed anchor; zero-exposure on/off). 5. Static conformance.
   6. 9-run matrix. 7. Measurement (S1, S-a..g, S3 ρ). 8. Verdict.
   9. Autonomous next-branch decision in the autonomy log.

No post-hoc tuning; all runs preserved incl. failures; Phase I and
Phase II-A immutable; tree clean at each commit boundary.

STOP — protocol frozen.