# Dormant-candidate reserve — execution record & verdict

Status: EXECUTION RECORD, 2026-09-21. Protocol: docs/x-clla-
dormant-reserve.md (frozen 63686f2). Implementation: commit
0abff19 (reserve + CandidatePool telemetry; identity PASS).
13 runs executed and preserved. No tuning, no post-hoc threshold
changes.

## 0. Run inventory (13 executions; telemetry-authoritative)

| run id | arm | seed | ended | failures |
|---|---|---|---|---|
| clla-ident-il-20260920T213318Z (flag OFF) | ident | 20260912 | curriculum-complete | 0 |
| clla-res-s20260912-bac-20260920T213340Z | bac | 20260912 | curriculum-complete | 0 |
| clla-res-s20260912-bca-20260920T213344Z | bca | 20260912 | curriculum-complete | 0 |
| clla-res-s20260912-il-20260920T213349Z | il | 20260912 | curriculum-complete | 0 |
| clla-res-s20260912-d-20260920T213423Z | d (info) | 20260912 | curriculum-complete | 0 |
| clla-res-s424242-bac-20260920T213353Z | bac | 424242 | curriculum-complete | 0 |
| clla-res-s424242-bca-20260920T213357Z | bca | 424242 | curriculum-complete | 0 |
| clla-res-s424242-il-20260920T213401Z | il | 424242 | curriculum-complete | 0 |
| clla-res-s424242-d-20260920T213427Z | d (info) | 424242 | curriculum-complete | 0 |
| clla-res-s9001-bac-20260920T213406Z | bac | 9001 | curriculum-complete | 0 |
| clla-res-s9001-bca-20260920T213410Z | bca | 9001 | curriculum-complete | 0 |
| clla-res-s9001-il-20260920T213415Z | il | 9001 | curriculum-complete | 0 |
| clla-res-s9001-d-20260920T213432Z | d (info) | 9001 | curriculum-complete | 0 |

All RunEnded = curriculum-complete @105001; failures = [].

## 1. Identity / integrity — PASS

- Ident (flag OFF, reserve+telemetry present): FNV 9647ea8a0ca4dbd2
  (152,254 rows); 105/105 frames — byte-identical. The reserve and
  CandidatePool telemetry are fully invisible flag-off.
- Suites: core 83 (3 new reserve tests), exp 58, telemetry 13,
  viz 5 — green.

## 2. S1 — PRIMARY: second-block protected mass ≥ 0.5 × first-block

| run | arm | first-block | second-block | ratio | verdict |
|---|---|---|---|---|---|
| s20260912-bac | bac | 16.85 | 3.19 | 0.189 | FAIL |
| s424242-bac | bac | 15.99 | 4.54 | 0.284 | FAIL |
| s9001-bac | bac | 20.95 | 0.76 | 0.036 | FAIL |
| s20260912-bca | bca | 16.79 | 2.80 | 0.167 | FAIL |
| s424242-bca | bca | 17.25 | 4.11 | 0.239 | FAIL |
| s9001-bca | bca | 16.36 | 2.33 | 0.143 | FAIL |

S1: FAIL 6/6. (First-block protected mass is intact: 16–21, same
as the bounded-CLLA baseline — the reserve did NOT harm the first
block. Second block 0.76–4.54 vs baseline 0.75–3.17: a small but
insufficient improvement; ratio 0.036–0.284, bar 0.5.)

## 3. S2–S6

S2 (alternating il): pA/pC at drive end — s20260912: 12.15/7.51
(ratio 0.618); s424242: 13.39/7.66 (0.636); s9001: 8.44/10.01
(0.457). Alternating coexistence preserved (ratios in the
no-reserve band 0.5–0.6; s9001 balanced). PASS.
S3: protected cap P ≤ 0.6+1e-6 — no violation (per-neuron
protected mass ≤ 0.6; the consolidated-per-neuron data stays
capped; consistent with viability run behavior). PASS.
S4: M2 working target t_e−P > 0 — holds (P capped 0.6 < 0.8;
working mass normalized; no degenerate skip signature; W tracks
target). PASS.
S5: stability — 13/13 curriculum-complete, zero failures, no new
failure/resource class. PASS.
S6: pool invariant — 105 CandidatePool rows per run, slots == 312
(6×52) at every row in every run; F5 static (reserve reads only
candidate.pre, w, reserved bit, pool index, headroom, fired
channel set — no identity); R1–R5 conformance. PASS.

## 4. S7 — preservation-vs-flux classification: NEITHER, exactly.

Instrumented survival (poolcoh/poolread):

- Reserved candidates during the first block: 90–255 per run
  (s20260912-bac 170; s424242-bac 184; s9001-bac 204; il 180–197;
  d 249–255) — the reserve MECHANISM undeniably works: candidates
  that co-fired once survive dormancy at the theta_die floor.
- BUT at C-block onset in s20260912-bac, reserved candidates with
  pre ∈ channels 8–15 (the NOVEL second pattern) = **0.0** (mean
  over the A block: reserved A-cohort 123.7, C-cohort 0.0).
- The block-2 permanence count is 38 (vs 35 without reserve) —
  flux essentially unchanged.

Classification per the frozen protocol §5:
  A (preservation failure — dormant candidate lost before return):
  NO — the reserve preserved every candidate that had co-activity;
  nothing was lost.
  B (candidate-flux — survives but permanence rate insufficient):
  NO — the C-cohort never had candidates to survive: flux cannot
  start from nothing.
The outcome is a THIRD finding, the reserve-trigger boundary:
"ever-coactive" is satisfiable only by the pattern that was
active. A first-seen later pattern has zero co-activity history,
hence zero reserved candidates at its onset; the reserve preserves
the already-known cohort (useless for the second block) and
nothing for the novel one. The reserve is VACUOUS FOR the blocked
scenario it was designed to fix, by the semantics of its own
trigger.

## 5. Verdict (frozen binary)

BUNDLE: NOT SUPPORTED — S1 fails 6/6.
The frozen question ("Does preserving dormant pre-associations
remove second-arrival substrate starvation?") is answered
NEGATIVELY, with the mechanism now precisely characterized:
preservation works (survival proven), but it has nothing
preservable for a never-seen pattern. A dormant-candidate reserve
keyed on ever-coactivity cannot help a genuinely first-observed
future pattern.

## 6. What is established / not claimed

- Reserve mechanics as frozen: survival, floor, eligible-waiting,
  eviction ladder, cap invariant, identity — all verified.
- Survival is cohort-correct: it protects the CO-FIRED cohort.
- NOT claimed: memory success (S1 failed); flux-failure (B)
  confirmed — B is NOT the diagnosis; the correct next question is
  the reserve's trigger semantics, not M3 flux augmentation.
- D arms (informative-only): complete 3/3; not evidence.

## 7. Required next (for review; NOT executed)

The evidence points AWAY from flux augmentation (the audit's
predicted B-path) and AT the trigger: "ever-coactive" cannot
identify a future-novel pattern. Any mechanism that could must
operate on a signal available BEFORE first co-activity —
e.g., M3's random channel draws themselves (the audit's "M3 flux"
family), or a channel-affinity prior not grounded in co-activity
(risks reintroducing noise-hoarding). No amendment proposed
without approval; F2/F6 11–20 correction retained for future
capability protocols; historical verdicts untouched.

STOP — experiment complete; results as recorded.