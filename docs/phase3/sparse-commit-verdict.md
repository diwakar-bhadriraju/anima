# Phase III-A sparse-commit verdict — expression-separation vs allocation: a measured architectural tradeoff

Status: VERDICT RECORD, 2026-09-22. Protocol docs/phase3/
sparse-commit-protocol.md; runs clla-sps-*1523*Z; instruments d2eval,
respoverlap, rexp; identity PASS (flag-off byte-identical,
clla-ident-il 152301Z).

## Result (all 9 runs complete, 0 aborts)

1. S1 (gating precondition): FAIL 9/9. Second-block protected mass
   0.00-3.44 vs first 8.95-14.99 (ratios 0.00-0.38, bar 0.5). The
   second memory starves under dropout: committed neurons shed
   other-track afferents, so block-2 finds almost no substrate.
2. Responder separation (mechanistic endpoint): ACHIEVED spectacularly.
   Blocked re-exposure responder sets Jaccard 0.000-0.02; re-expression
   ρ(A) 0.85-0.94 (cosC = 0.000 — perfect separation) where the memory
   formed. But the C memory nearly vanished (union responders 0-1 in
   several cells) -> the high ρ is trivial (nothing to disambiguate).
3. Alternation (il): also broken (A responders collapsed to 5-28 of 52;
   ρ destroyed) — the dropout overshoots under alternation too.

## Failure mode: over-commit / first-arrival capture (protocol §5 #1)

During block 1, P_1 = 0 for every neuron, so R_0 = P_0/(P_0+0) = 1.0 for
ANY neuron with A-protected mass -> ~all A-recruited neurons commit to
track 0 and drop their C-afferents. The second block inherits only the
un-committed residue (too few neurons), so it cannot form. The dropout
is irreversible and one-directional: first-past-the-post wins
everything. This is the CLLA allocation signature re-expressed at the
assembly/dropout level.

## The measured tradeoff (the architectural finding)

| regime | responder separation | multi-memory access | re-expression |
|---|---|---|---|
| dense pool, no dropout (E-nogain) | Jaccard ~1.0 | S1 6/6 (blocked) | weak blocked / strong alternation (ρ 0.25-0.91) |
| forced dropout (sparse-commit) | Jaccard ~0.000 | S1 9/9 fail (blocked) | trivial (one memory absent) |

In the single shared pool, per-neuron first-contact commitment makes
blocked-order (sequential) multi-memory ACCESS and SELECTIVE
RE-EXPRESSION structurally incompatible: commitment that separates
destroys the later-arriving memory's capacity; commitment that keeps
capacity gives no separation. Alternation bypasses the asymmetry
(both patterns present from the start -> balanced commitment -> both
capabilities). This is a supported, measured boundary of the
single-pool-with-local-commitment family for the blocked-order
canonical test.

## Decisions (autonomy D-06)

1. sparse-commit is CLOSED as configured: S1 gating fails; no
   reparameterization per protocol (θ_commit/dropout are not free to
   tune on this result). The failure mode was pre-registered and is
   reported, not excused.
2. NO further rescue chain on this branch (patch-spiral guard): the
   tension is fundamental to the single shared pool, not a threshold
   artifact.
3. The E-nogain alternation result (formation S1 6/6 + alternation ρ
   0.25-0.91, general across channel groups and seeds) REMAINS the
   demonstrated partial capability and the mission's platform.
4. Next: two evidence-based forks (to be chosen by a small probe):
   (a) advance up the ladder (Level 4 temporal/prediction) on the
   alternation-retrieval base — the sequential-blocked single-pool
   re-expression is a bounded gap, not the mission's only path;
   (b) test a TOPOLOGY variant that separates pools without K-dup:
   e.g., a sparser initial afferent endowment (fewer neurons have both
   patterns' afferents -> disjointness from wiring, not dropout), which
   avoids the destructive first-past-the-post drop. Probe (b) is
   cheap and decisively separates "dropout-caused" from
   "wiring-limited" access.

## Unresolved / open

- Whether any per-neuron local commitment rule can give blocked-order
  equal-access AND separation in one shared pool (strongly negative so
  far: three independent mechanisms — CLLA budget, D-core claim,
  sparse dropout — each captured by the first pattern under blocked
  order).
- Whether sparser initial wiring (probe b) bypasses the drop-induced
  capture.
- Level 4+ (temporal, prediction, action, closed loop) — untested.

STOP — verdict recorded; branch decision pending probe (b) or the
ladder fork.