# Phase III — decisive diagnostic and the sparsity fork

Status: DIAGNOSTIC + ARCHITECTURAL DECISION, 2026-09-22 (Phase III
mission). Read-only measurement over E-nogain blocked runs
(runs/clla-arex-*1326Z); instrument examples/respoverlap.rs.

## 1. The diagnostic (measured)

At blocked-order re-exposure (S3 probe), the internal responders to the
A re-block and the C re-block are:
- union A-responders 51-52 of 52; union C-responders 52 of 52;
- Jaccard overlap 0.981-1.000 (A vs C responder sets); inter 51-52;
- recurrent-only (neurons with neither A nor C afferents) = 0.

Causal reading: with M1 afferent density p_in = 0.5, almost every
internal neuron receives BOTH an A-channel and a C-channel afferent, so
every neuron fires for both patterns. The 52-dim re-expression vectors
differ only in PER-NEURON RATE (count) modulation (cos(A,A) 0.69-0.81
vs cos(A,C) 0.58-0.87), never in membership. The "alternation ρ win"
is the same rate-modulation phenomenon in a train-period where both
patterns' assemblies are fresh; it is NOT structural separation.

## 2. Why the dense shared pool cannot give blocked-order selectivity

Selective re-expression under recency requires that re-presenting A
activate a DIFFERENT (preferentially A-consistent) neural population
than re-presenting C. In a pool where every neuron integrates both
patterns' afferents, no such population exists by construction. The
budget/claim/floors of D-core fix PERSISTENCE and FORMATION (S1 6/6)
and the WRITE-side, but the EXPRESSION read is a fixed dense projector
onto a shared membrane population. This is the measured, architectural
basis of the E3b "binding problem is representational, not dynamic"
conclusion, now confirmed at the responder-set level.

## 3. The fork (evidence-based architectural decision)

The organism needs SPARSE, preferably DISJOINT assembly commitment: a
neuron that has committed to memory A should not integrate memory C's
afferent drive at expression time. The mechanism family that achieves
this from local signals (not labels, not global engines, not K copies):

- LOCAL COMMITMENT (exists): the track/claim machinery (E-nogain)
  assigns afferents to learned contexts.
- NEW: AFFINENT COMPETITIVE DROPOUT — a neuron whose protected
  track-t share R_t passes a local commitment threshold stops
  supporting other-track working afferents (paths them toward the
  churn floor / M4) so that its effective integration becomes
  track-selective. Neurons weakly recruited by the first pattern keep
  their other-track afferents, giving the SECOND pattern a genuinely
  disjoint cohort to recruit -> re-exposure membership separates.
- RESOURCE: the pool (52) splits across memories by commitment; each
  assembly ~N/2 at K=2; finite and explicit; allocation via (1-R)
  unexplained drive as before.

This is the competing-assemblies route (Phase II candidate A) realized
by the SLOW local commitment machinery (which E validated) instead of
the fast dynamics competition E3b falsified. It is a redesign fork, not
a rescue patch: the diagnostic uniquely requires expression-side
structural separation.

Risk to be pre-registered: if the first pattern over-commits (> ~N/2
neurons), the second starves (allocation failure, the classic CLLA
signature). The commitment threshold is the single new constant; its
scale is derived from the measured R shares / B budget, not tuned.

## 4. What is discarded / kept

- KEPT: E-nogain formation (S1 6/6), the learned tracks, the claim +
  churn-exempt floor, per-track budgets/caps, the identity gate, the
  CLLA stability result.
- DISCARDED (for expression): the expectation that a dense shared pool
  can give blocked-order selective re-expression. The alternation-ρ
  result is retained as rate-modulation evidence, not as structural
  retrieval.
- The rg8c gain: remains discarded (destabilizer).

## 5. Decision record (autonomy D-05)

Given evidence: dense-pool membership is ~100% shared => blocked-order
selective re-expression is architecturally impossible here. Next frozen
experiment: SPARSE-COMMIT — selective afferent dropout under local
commitment, tested on the E-nogain base, endpoint = blocked-order
re-expression ρ with responder-set disjointness as the primary
mechanistic check (Jaccard of responder sets must drop below ~0.5 for
selectivity). Freeze protocol before implementation; identity gate.

## 6. Stop-condition note

This is NOT a stop: it is a supported architectural fork that the
evidence demands. If sparse-commit also fails to give disjoint
responders, THAT is the evidence for a fundamental limit of the
single-pool family and the mission reconsiders the substrate
environment (failure-condition path 1/2).