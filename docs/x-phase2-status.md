# Phase II status — E-nogain milestone (autonomous loop checkpoint)

Status: CHECKPOINT RECORD, 2026-09-22 (autonomy charter). All runs
preserved; protocol/result records: docs/x-phase2-ar-protocol.md,
x-phase2-are-result.md, x-phase2-generalization.md, autonomy-log.md.

## Measured capability (E-nogain: d_core + d_claim, no recruitment gain)

| Capability | Result | Robustness |
|---|---|---|
| Sequential blocked-order formation | S1 6/6 (ratios 0.96–2.34) | 2 channel-group pairs × 3 seeds, 0 aborts |
| Surviving-substrate addressing | unclaimed→claim, churn-exempt floor | mechanism + unit tests |
| Alternating selective re-expression | ρ 0.25–0.91 (both patterns) | all seeds × both channel pairs |
| Generalization (novel 16–23 group) | forms S1 6/6 + ρ 0.36–0.91 | not A/C-overfit |
| Stability | 9/9 no-gain complete; gain variant aborts 4/9 | demonstrates gain is unnecessary + destabilizing |
| Prototype identity | blur 0.085–0.248 | no collapse |
| Resource invariants | per-track caps ≤ 0.34; totals ≤ t_e | unit-checked |

## Open (registered next)

1. BLOCKED-order selective re-expression (weak ρ ±0.1): shared
   membrane/recurrent-pool recency — the G3 superposition localized to
   the expression layer; budgets+addressing fix persistence/formation,
   not the read. Next candidates: per-track recurrent protection, or
   an expression-side read gate. (D-04.)
2. Capacity > K=2 (3+ memories): requires K≥3; a frozen change.
3. Closed-loop use (Level 7) and developmental robustness (Level 8).

## Why this is a checkpoint, not a stop

The ultimate canonical test (learn A, learn C, delay, present A ⇒
A-specific state re-expresses) is demonstrated under ALTERNATION and
under the FORMATION side of blocked order; the blocked-order
RE-EXPRESSION leg of that test is not yet demonstrated. Per the
charter's stop conditions, work continues (D-04 next). This record
consolidates the durable checkpoint at a clean commit boundary.

All commits since the E freeze: 0691f8a (protocol), a9563d4 (M1
audit), 607f43f (D-core impl), earlier, then this loop's milestone
commits. Tree clean.