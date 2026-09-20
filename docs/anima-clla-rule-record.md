# CLLA allocation-rule experiment — execution record & verdict

Status: EXECUTION RECORD, 2026-09-21. Protocol: docs/x-clla-
allocation-rule.md (frozen 92e41ec; run matrix corrected df60cbe).
Implementation: commit (this record's parent). Identity + 12 rule-ON
runs executed and preserved. No tuning, no post-hoc threshold
changes, no protocol edits.

## 0. Run inventory (13 executions)

| run id | arm | seed | ended | failures |
|---|---|---|---|---|
| clla-ident-il-20260920T202228Z (rule OFF) | ident | 20260912 | curriculum-complete | 0 |
| clla-rule-s20260912-bac-20260920T202246Z | bac | 20260912 | curriculum-complete | 0 |
| clla-rule-s20260912-bca-20260920T202251Z | bca | 20260912 | curriculum-complete | 0 |
| clla-rule-s20260912-il-20260920T202256Z | il | 20260912 | curriculum-complete | 0 |
| clla-rule-s20260912-d-20260920T202336Z | d (informative) | 20260912 | curriculum-complete | 0 |
| clla-rule-s424242-bac-20260920T202302Z | bac | 424242 | curriculum-complete | 0 |
| clla-rule-s424242-bca-20260920T202307Z | bca | 424242 | curriculum-complete | 0 |
| clla-rule-s424242-il-20260920T202312Z | il | 424242 | curriculum-complete | 0 |
| clla-rule-s424242-d-20260920T202342Z | d (informative) | 424242 | curriculum-complete | 0 |
| clla-rule-s9001-bac-20260920T202318Z | bac | 9001 | curriculum-complete | 0 |
| clla-rule-s9001-bca-20260920T202323Z | bca | 9001 | curriculum-complete | 0 |
| clla-rule-s9001-il-20260920T202328Z | il | 9001 | curriculum-complete | 0 |
| clla-rule-s9001-d-20260920T202348Z | d (informative) | 9001 | curriculum-complete | 0 |

All end reasons from RunEnded telemetry; failures = [] everywhere.
(NOTE: s9001-bac started at 202318 vs matrix listing order — all 12
rule-ON runs complete; typo risk noted, IDs above are authoritative.)

## 1. Identity / integrity — PASS

- Ident arm (rule OFF, clip present): FNV 9647ea8a0ca4dbd2
  (152,254 rows) byte-identical; 105/105 snapshot frames identical
  to the bounded-CLLA ident run — the rule is invisible flag-off.
- Suites: core 80 (incl. 2 new rule tests), exp 58, telemetry 13,
  viz 5 — all green.

## 2. Frozen criteria

### Primary allocation endpoint (bac/bca × 3, rule ON):
### second-block protected mass ≥ 0.5 × first-block protected mass

| run | arm | 1st-block p | 2nd-block p | ratio | verdict |
|---|---|---|---|---|---|
| s20260912-bac | bac | pA 16.53 | pC 1.88 | 0.114 | FAIL |
| s20260912-bca | bca | pC 16.88 | pA 1.34 | 0.079 | FAIL |
| s424242-bac | bac | pA 17.70 | pC 2.30 | 0.130 | FAIL |
| s424242-bca | bca | pC 16.48 | pA 3.17 | 0.192 | FAIL |
| s9001-bac | bac | pA 22.04 | pC 0.17 | 0.008 | FAIL |
| s9001-bca | bca | pC 16.50 | pA 0.94 | 0.057 | FAIL |

**PRIMARY: FAIL 6/6.** ratios 0.008–0.192 (bar 0.5).

### Alternating intactness (il × 3)

| run | pA | pC | ratio A/(A+C) | headroom |
|---|---|---|---|---|
| s20260912-il | 12.26 | 7.89 | 0.608 | 0.212 |
| s424242-il | 9.89 | 10.08 | 0.495 | 0.216 |
| s9001-il | 9.32 | 6.59 | 0.586 | 0.294 |

F1-il (raw masses vs committed refs): computed below.

### Safety

- F4: 0 failures all 12 rule runs.
- F3: P ≤ 0.6+1e-6 (cap) — Pmax 0.6000000 everywhere (checked via
  clla_traj Pmax on rule runs).
- F5: rule reads only per-neuron input current split by the
  existing consolidated flag + P + cap. Static conformance PASS.
- No new resource/failure class; no serialized new state.

## 3. Mechanism findings (from trajectories)

1. THE RULE WORKS EXACTLY AS SPECIFIED:
   - g = 1.000 during the entire C block (R-med 0.000 through
     pres 24; residual drives full allocation priority).
   - Headroom RESERVED: 0.249–0.294 vs 0.155–0.161 in the no-rule
     bounded runs — first block no longer consumes the budget to
     the floor.
   - First-block permanence decelerated: A-block cum-4→170 events
     (8.5/pres) vs 364 (18.2/pres) no-rule — the g<1 self-
     termination is real.
2. THE PRIMARY FAILURE IS UPSTREAM OF THE RULE:
   - The C-block working substrate is DESTROYED during the A
     block: wC (live unconsolidated C-channel afferents) falls
     197 → 77 across the A block (M2 normalizes the shrinking
     working budget; M4 prunes), INDEPENDENT of g.
   - At C-block start, few surviving unconsolidated C afferents
     → little C-drive reaches neurons → few co-active C-candidate
     events → permanence rate 1.8/pres vs A's 8.5/pres.
   - g=1 with nothing to accumulate: the gate is not the
     bottleneck; substrate availability is.
3. Protected-C acquisition is slow but present: pC count climbs
   0→35 during the block (t≈51k onward), reaching 1.88 — the
   entry gate (P + w ≤ cap) is NOT blocking (headroom 0.28).

## 4. Verdict (frozen criteria)

BUNDLE: NOT SUPPORTED — primary endpoint FAIL 6/6.

The frozen question — "Can the local allocator prevent first-
arrival starvation of a later distinct configuration?" — is
answered NEGATIVELY with a precise mechanism: the allocator's g
couples novelty to candidate accumulation, but the starvation
occurs in the working synaptic substrate (unconsolidated C
afferents pruned during block 1) before candidate accumulation is
even reached. The rule reserves headroom and prioritizes novel
drive (verified: g=1, headroom 0.28) yet cannot allocate because
the novel cohort's working pathway was dismantled.

Alternating preservation (il): coexistence intact per-ratio
(0.50–0.61) and headroom preserved, but F1-il raw-mass ratings
must be computed before the intactness criterion is scored —
reported in §5.

## 5. F1-il (raw mass, frozen: ≥0.5× committed refs)

Computed from clla_meas/synmem on the rule il runs (raw drive-end
A/C channel masses):

| seed | A-mass | A-ref | 0.5× | C-mass | C-ref | 0.5× | il-intact |
|---|---|---|---|---|---|---|---|
| 20260912 | 0.298 | 0.290 | 0.145 | 0.191 | 0.239 | 0.120 | PASS |
| 424242 | 0.242 | 0.249 | 0.124 | 0.252 | 0.221 | 0.111 | PASS |
| 9001 | 0.215 | 0.284 | 0.142 | 0.161 | 0.156 | 0.078 | PASS |

F1-il: PASS 3/3 (measured with the same synmem instrument as the
capability record). Alternating coexistence fully preserved under
the rule — the rule's headroom reservation does not harm the
concurrent-arrival regime.

## 6. What is established / not claimed

- Established: the rule behaves per spec (gate math, headroom
  reservation, first-block deceleration); the C-block starvation
  persists and is now localized to the WORKING SUBSTRATE, not the
  allocation gate; alternating coexistence intact.
- NOT claimed: any memory capability change (F2/F6 windows remain
  under the frozen 11–20 amendment, not executed here); D-arm
  evidence (informative only); that a different rule form (e.g.,
  gating M4 pruning or protecting working afferents) would fix
  the substrate destruction — that is a NEW mechanism question.

## 7. Required next (not executed; for review)

The starvation site has MOVED from "first-come cap consumption"
(original) to "working-substrate destruction during block 1"
(now measured). Any further amendment must address substrate
preservation (e.g., protecting a cohort's working afferents from
M2/M4 during its inactive blocks, or allocating admission slots
at M3 creation rather than at permanence). This is a mechanism
design question beyond the frozen rule; no amendment proposed
here without approval.

STOP — experiment complete; results as recorded.