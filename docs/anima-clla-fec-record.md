# First-exposure source-correction experiment — execution record & verdict

Status: EXECUTION RECORD, 2026-09-21. Authority: da8ba9a (impl audit)
+ the approved source-correction amendment. Implementation: commit
90e354f (redraw/eviction bias sourced from the module `fired` record
instead of afferent-derived fired_channels). 13 runs executed and
preserved. No tuning, no post-hoc changes, no E-number.

## 0. Run inventory (13 executions; telemetry-authoritative)

| run id | arm | seed | ended | failures |
|---|---|---|---|---|
| clla-ident-il-20260921T153305Z (flag OFF) | ident | 20260912 | curriculum-complete | 0 |
| clla-fe-s20260912-bac-20260921T153338Z | bac | 20260912 | curriculum-complete | 0 |
| clla-fe-s20260912-bca-20260921T153343Z | bca | 20260912 | curriculum-complete | 0 |
| clla-fe-s20260912-il-20260921T153348Z | il | 20260912 | curriculum-complete | 0 |
| clla-fe-s20260912-d-20260921T153428Z | d (info) | 20260912 | curriculum-complete | 0 |
| clla-fe-s424242-bac-20260921T153353Z | bac | 424242 | curriculum-complete | 0 |
| clla-fe-s424242-bca-20260921T153358Z | bca | 424242 | curriculum-complete | 0 |
| clla-fe-s424242-il-20260921T153403Z | il | 424242 | curriculum-complete | 0 |
| clla-fe-s424242-d-20260921T153433Z | d (info) | 424242 | curriculum-complete | 0 |
| clla-fe-s9001-bac-20260921T153408Z | bac | 9001 | curriculum-complete | 0 |
| clla-fe-s9001-bca-20260921T153413Z | bca | 9001 | curriculum-complete | 0 |
| clla-fe-s9001-il-20260921T153418Z | il | 9001 | curriculum-complete | 0 |
| clla-fe-s9001-d-20260921T153438Z | d (info) | 9001 | curriculum-complete | 0 |

All RunEnded = curriculum-complete @105001; failures = [].

## 1. Identity / integrity — PASS

- Ident (flag OFF): FNV 9647ea8a0ca4dbd2 (152,254 rows); 105/105
  frames byte-identical — source correction invisible flag-off.
- Suites: core 85 (corrected-source test), exp 58, telemetry 13,
  viz 5 — green.

## 2. S1 PRIMARY — FAIL 6/6

| run | arm | first | second | ratio | verdict |
|---|---|---|---|---|---|
| s20260912-bac | bac | 22.87 | 2.98 | 0.130 | FAIL |
| s424242-bac | bac | 20.29 | 3.92 | 0.193 | FAIL |
| s9001-bac | bac | 24.55 | 1.12 | 0.046 | FAIL |
| s20260912-bca | bca | 18.51 | 2.69 | 0.146 | FAIL |
| s424242-bca | bca | 23.86 | 1.27 | 0.053 | FAIL |
| s9001-bca | bca | 20.93 | 2.83 | 0.135 | FAIL |

Ratios 0.046–0.193; bar 0.5. First-block protected mass grew
further (18.5–24.6) — the corrected source benefits the ACTIVE
first block even more (264 vs 248 pre-correction events).

## 3. Causal path (bac s20260912; all measured)

1. VISIBLE FIRING CHANNELS: corrected source reads the module
   `fired` record → all 8 C channels observable every C
   presentation regardless of synapse survival. The afferent-
   derived visibility cap (0.71/neuron) is REMOVED (fixed by the
   amendment).
2. CHANNELS REJECTED: connected() still excludes live-afferent
   and pooled duplicates (per mandate, preserved). Rejection
   count not directly telemetered; the binding outcome shows the
   net effect.
3. ACTUAL NEW BINDINGS: not directly observable; proxied by C
   permanence 61 events — up from 41 (defective source) and 35
   (alloc-only) — the source correction tripled the effective
   supply vs the defective path.
4. PERMANENCE EVENTS/PRESENTATION: 61/20 = 3.05/pres (C block);
   vs A block 264/20 = 13.2/pres. First C permanence t=47,500
   (2.5 s after onset).
5. FIRST-BLOCK SUPPLY: 264 A events → 22.87 protected.
6. SECOND-BLOCK PROTECTED MASS: 2.98 (cC=55 synapses, mean 0.054
   w/synapse) vs A's 22.87 (cA=237, mean 0.096).
7. CHURN: working C afferents 29 → 11 across block 2 even while C
   presents (M2 shrinks working budget as P grows; newly protected
   C synapses at 0.02 initial do not lift the mean above 0.054 by
   end).
8. CAP: P per neuron ≤ 0.6 (Pmax 0.6000; no violation).
9. M2 TARGET: t_e − P > 0 throughout (cap invariant).
10. STABILITY: 13/13 curriculum-complete, 0 failures.
11. IL PRESERVATION: s20260912 0.500, s424242 0.444, s9001 0.403
    (balanced, as pre-correction).

## 4. Bottleneck classification (mandate a–e)

a) FIRST-ROUND VISIBILITY: FIXED by the correction — the module
   record observes every firing channel; the source is no longer
   the limiter (61 events vs 41 proves the bind supply tripled).
b) CANDIDATE CAPACITY: not limiting (61 binds; 6 slots fine).
c) PERMANENCE FLUX: improved but still asymmetric: 3.05/pres C vs
   13.2/pres A. The corrected supply reaches only ~23% of the
   A-block accumulation rate. This is a REAL residual contributor,
   but see (e) — flux and churn interact.
d) PROTECTED-CAP INTERACTION: NO (cap respected).
e) THE BINDING CONSTRAINT — POST-PERMANENCE WEIGHT GROWTH + WORKING
   AFFERENT CHURN: second-block C synapses consolidate at mean
   0.054 w vs A's 0.096. C protected mass ends at 2.98 even with
   61 permanence events because each new synapse is protected at
   w_c_permanent=0.02 and subsequent LTP (which took A synapses
   0.02→0.096 over block 1) is starved for C: the working C
   afferents keep dying (29→11) under M2's shrinking budget, so
   the protected C cohort receives diluted drive and weak LTP
   support. Churn: the corrected allocator wins substrate slots;
   it cannot stop M2 from crushing the STILL-WORKING (as-yet-
   unconsolidated) C afferents that would feed the new protected
   synapses' LTP.

## 5. Verdict (frozen binary)

BUNDLE: NOT SUPPORTED — S1 fails 6/6.
The source-correction amendment is VINDICATED as a mechanism fix
(visible firing channels fixed; supply 41→61), but the corrected
first-exposure capability is INSUFFICIENT to meet the S1 bar. The
residual bottleneck is classified from measurement:
    E — post-permanence weight growth + working-afferent churn:
    the allocator now delivers substrate (flux tripled) but the
    newly protected second-block synapses cannot grow to
    first-block weight because the surviving working afferents
    that would drive their LTP keep being destroyed by M2 during
    the second block itself.

This is a NEW finding: it is not visibility (fixed), not capacity,
not cap — it is that churn now operates on the POST-CONSOLIDATION
growth path, i.e., M2's working-mass squeeze removes the very
afferents that would strengthen the just-allocated representation.

## 6. What is established / not claimed

- Established: the module-fired source works and triples bind
  supply (41→61); the afferent-derived visibility cap was the
  vacuity and is removed; all sanity (cap/M2-target/stability/il)
  holds.
- NOT claimed: memory success (S1 failed); that (e) is trivially
  fixable (any M2/M4 change is a protocol amendment — not
  proposed here); D-arm evidence (informative only, complete 3/3).
- The experiment answers its question: first-exposure rebinding
  works but only partially removes second-block starvation; the
  residual limit is the second block's own working-substrate churn
  against the new protected structure, classified as (e).

## 7. Registered next question (for review, NOT executed)

The residual limit is the interaction between M2 working-mass
squeeze and the newly-consolidated second-block structure. Any
further experiment must distinguish:
    (e1) does protecting a FRACTION of second-block working
    afferents at consolidation-time (not first-exposure time) lift
    the protected-weight growth — i.e., is the churn the binding
    constraint on weight, not on count?
    (e2) is the 0.02→0.096 growth path itself (LTP on protected
    synapses) the binding constraint regardless of churn?
Both are M2-adjacent mechanism questions; neither is proposed for
implementation without approval.

F2/F6 11–20 correction retained; historical verdicts untouched.
All 13 runs preserved. STOP — experiment complete.