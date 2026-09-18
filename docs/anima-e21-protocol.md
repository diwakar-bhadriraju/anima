# ANIMA E21 — Temporal capacity (gap sweep)

Status: **FROZEN** (2026-09-19). Design `7b6acb0`, approved with
decisions: 6 arms incl. 400 ms; split-half noise floor; seed
20260912 primary.

## 1. Question

What antecedent->probe separation can the frozen E6/E12 organism
still resolve (antecedent information measurable in the probe
response)?

## 2. Design

E18 paradigm verbatim per arm: antecedent A|C (500 ms, committed)
-> gap G -> probe B (500 ms, all-SEQ via variant_block 100000) ->
ITI; cadence 2000 ms (ITI = 1000 - G); 200 trials; balanced 20/20
per 40-trial window; seed 20260912; S0 silence 5000. ARMS: G in
{0, 50, 100, 200, 400, 800} ms — six runs, ONLY gap (and exp_id)
differ. G=0 = zero-retention reference (probe starts at antecedent
offset). No world, no interface, no output readout: E21 is
measurement-only. Organism/mechanisms/parameters frozen (JSON-
isolated vs e12).

## 3. Measurement (per arm)

- D_E, D_M, D_L = 1 - pairwise-mean cos(probe-after-A,
  probe-after-C) over trials 1-40 / 81-120 / 161-200 (probe epochs;
  raw == L1; L primary — the E-window settling transient is a
  known artifact).
- NOISE FLOOR NF := mean of split-half within-condition divergence
  over the L window: 1 - mean cos(B|A first-half, B|A second-half)
  and the C analogue.
- A-C sanity (L, < 0.60); gates (failures 0, max rate <= 250,
  permanence > 0); schedule audit (lag-1, balance).

## 4. Retention criterion + cliff (pre-registered)

RETAINED(G) := D_L(G) - NF(G) > 0.05. TEMPORAL CAPACITY :=
largest G in the set with RETAINED(G) and RETAINED(G') for all
G' < G in the set (monotone prefix); non-monotone breaks are
reported as anomalies, never re-selected. If only RETAINED(0):
capacity < 50 ms. If all RETAINED: capacity >= 800 ms (extension
only by a new approved release). Resolution = the sampled grid.

## 5. Verdict logic

- MEASURED: the cliff + full D/NF table.
- PROTOCOL FAILURE: G=0 shows no divergence; any leakage test
  fails; determinism fails.
- An arm failing gates is excluded; the cliff uses the remaining
  prefix only if contiguous, else INCONCLUSIVE.

## 6. E22 dependency (preregistered here)

g_B := largest RETAINED G with G >= 50. If none, E22 is NOT
executed; registered outcome "temporal capacity insufficient for a
closed-loop retention task in the frozen substrate". g_B derives
from this experiment's table ONLY, before any E22 run.

## Appendix: config hashes (pre-run)

- e21-g0.toml `99334d9daead7bfe`; e21-g50.toml `0e302b16451939c5`; e21-g100.toml `2a54976089e739fc`; e21-g200.toml `875acf72fc341da3`; e21-g400.toml `7a0926452f5ed213`; e21-g800.toml `7e31b86f89b39722` (recorded pre-run).

## Appendix: amendments

- (none)
---

## E21 execution record (2026-09-19)

Implementation `ca185aa` (6 configs, hashes pre-run; identical
seeded antecedent sequences across arms — registered, tested).
Runs e21-g{0,50,100,200,400,800}-20260918T1817{26,28,30,32,34,36}Z.
140 tests green, 0 warnings; analysis deterministic. Verification:
substrate JSON-isolated vs e12; grids/cadence/balance verified;
A-C sanity <= 0.09 all arms; gates clean (0 failures, max rate
120.1-184.0 Hz, permanence 8,757-12,446).

### Capacity table (frozen metric: D_L vs split-half noise floor NF)

| gap | D_E | D_M | D_L | NF | D_L-NF | RETAINED (>0.05) |
|---|---|---|---|---|---|---|
| 0 | 0.2929 | 0.1251 | 0.1889 | 0.1074 | 0.0815 | **YES** |
| 50 | 0.3785 | 0.0930 | 0.0337 | 0.0351 | -0.0014 | no |
| 100 | 0.2416 | 0.1191 | 0.1030 | 0.0996 | 0.0034 | no |
| 200 | 0.1896 | 0.0900 | 0.0804 | 0.0844 | -0.0040 | no |
| 400 | 0.3012 | 0.2685 | 0.0966 | 0.0579 | 0.0386 | no |
| 800 | 0.3532 | 0.0830 | 0.0235 | 0.0217 | 0.0018 | no |

### Verdict: MEASURED — temporal capacity < 50 ms

Only gap 0 retains antecedent information (D_L - NF = 0.0815 >
0.05); every gap >= 50 ms shows D_L statistically at its own
split-half noise floor (|D_L - NF| <= 0.04). The gap-0 reference
shows divergence (paradigm valid). The cliff sits between 0 and
50 ms — consistent with, but now MEASURED rather than assumed
from, the ~100 ms synaptic-decay envelope.

### E22 dependency resolution (preregistered rule B.2)

g_B = largest RETAINED gap >= 50 ms: **NONE EXISTS**. Therefore
E22 is NOT EXECUTED. Registered outcome: "temporal capacity
insufficient for a closed-loop retention task in the frozen
substrate." This is a valid E21-derived result, not an E22
failure. No E22 protocol will be frozen on this substrate for a
retention task.
