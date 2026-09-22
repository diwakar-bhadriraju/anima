# Phase II-AR generalization — clean novel-channel-group (PN) result

Status: RESULT RECORD, 2026-09-22 (autonomy). Runs clla-ag2-*1342*Z
(novel pattern PN = channels 16-23 @ 20 Hz, A's statistics; A+PN
curricula bac/bca/il × 3 seeds; E-nogain mechanism). An EARLIER "AB"
probe (clla-ag-*1339*Z) was CONFOUNDED and is NOT evidence: the config's
pattern "B" is a 40 Hz phase-variant over channels 4-11 which overlaps
A's channels (0-7) — not a clean second group; discarded.

## Formation (S1, d2eval t=44k/84k)
- A→PN (bac): ratios 2.34 / 1.54 / 2.01 (s20260912 / s424242 / s9001)
- PN→A (bca): ratios 0.96 / 1.94 / 1.36
- 6/6 blocked cells PASS (bar 0.5) — formation GENERALIZES to a
  never-presented channel group, identical to the A/C result.

## Re-expression (ρ, S3 probe, first re-presentation window)
- ALTERNATING (il): ρ(PN) 0.91 / 0.62 / 0.65; ρ(A) 0.81 / 0.54 / 0.36
  — robust selective re-expression in every seed / both patterns,
  quantitatively matching the A/C alternation win (0.25–0.77).
- BLOCKED (bac/bca): weak (±0.1, 7/12 cells < 0.10) — the SAME
  blocked-ρ gap as A/C.

## Interpretation

The E-nogain mechanism does NOT overfit A/C: a genuinely novel channel
group (never before used) forms and re-expresses under alternation
exactly as A/C do. The measured capability is therefore reproducible
and general at the level of:
- sequential blocked-order formation (S1 6/6, both channel pairs);
- alternating-store selective re-expression (ρ 0.36–0.91, both pairs).
The recurring open gap is BLOCKED-order selective re-expression
(second-pattern recency in the shared membrane/recurrent pool), present
in both channel pairs — a general interference property, not a
curriculum artifact.

## Claims (charter language)

- SUPPORTED and GENERAL: formation + alternating re-expression under
  E-nogain across 2 channel-group pairs × 3 seeds (12 runs, 0 aborts).
- NOT ESTABLISHED: blocked-order re-expression; >2-memory capacity
  (K=2); closed-loop use; developmental robustness beyond the fixed
  cells/seeds.

## Next (autonomy decision D-03)

The single highest-evidence bottleneck is blocked-order re-expression.
Test H2 (recurrent-claim capture of the shared pool): afferent-only
claim (skip recurrent synapses in the claim rule) as a gated variant;
measure blocked ρ. If it improves → recurrent partition refinement;
if unchanged → H1 (first-block size / expression pool) and a
differently-scoped fix. No parameter changes to E-nogain otherwise.