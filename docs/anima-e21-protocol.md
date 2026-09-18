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