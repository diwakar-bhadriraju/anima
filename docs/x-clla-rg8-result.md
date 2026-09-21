# k_g = 8.0 recruitment-gain experiment — result (FROZEN protocol 272e0e5)

Executed 2026-09-21. Authority: docs/x-clla-recruitment-design-review.md §10
as approved (272e0e5). Mechanism implemented behind `recruit_gain` flag;
k_g = 8.0 code-frozen constant (`recruit_gain_k()`, NOT config-settable).

## 1. Identity / integrity

- Flag-OFF byte-identity gate: PASS. `anima-run run --config
  configs/clla-fe-s20260912-bac.toml` (no recruit_gain key) vs committed
  baseline `runs/clla-fe-s20260912-bac-20260921T153338Z`: ALL telemetry
  chunks (parquet), index.json, snapshots.bin.zst, metrics.json
  byte-identical. The flag-off branch executes zero new arithmetic.
- Integrity suites: PASS. cargo test --workspace: 91 (anima-core, incl. 6
  new rg_* mechanism tests) + 47 (anima-exp) + 11 + 13 + 5 + 2 — all green.
  New unit tests: exact boost math (working-only, gate=1), R-gated boost
  (R=2/3 → gate 1/3), consolidated-never-boosted, silence/endogenous
  double-zero, window-boundary reset, flag-off identity.

## 2. Run matrix (all 9 gain arms + 9 committed baselines)

Configs: configs/clla-rg8-s{seed}-{order}.toml = committed clla-fe config +
exactly one added line (`recruit_gain = true` under [v2]; exp_id renamed).
Curriculum orders verified from telemetry: bac = blocked [A, C] (A pres
1–20, C pres 21–40); bca = blocked [C, A] (C pres 1–20, A pres 21–40);
il = interleaved.

| arm | gain run | ended | failures |
|---|---|---|---|
| s20260912-bac | clla-rg8-…171025Z | curriculum-complete | [] |
| s20260912-bca | clla-rg8-…171031Z | curriculum-complete | [] |
| s20260912-il | clla-rg8-…171037Z | curriculum-complete | [] |
| s424242-bac | clla-rg8-…171044Z | curriculum-complete | [] |
| s424242-bca | clla-rg8-…171050Z | curriculum-complete | [] |
| s424242-il | clla-rg8-…171055Z | curriculum-complete | [] |
| s9001-bac | clla-rg8-…171102Z | curriculum-complete | [] |
| s9001-bca | clla-rg8-…171108Z | curriculum-complete | [] |
| s9001-il | clla-rg8-…171113Z | curriculum-complete | [] |

Baselines: committed clla-fe-*20260921T1533{38,43,48,53,58,03,08,13,18}Z.

GUARD NOTE: metrics S1 stage_population_rate = 58.7 Hz exceeds the 50 Hz
guard threshold, yet no trip occurred (failures []): the stage mean is
presentation-burst-weighted (duty ≈ 0.2), while the guard requires mean
> 50 Hz sustained over 5 s. Measured arithmetic: even with pres-1's
391 Hz burst, the 5 s-window mean = 41 Hz — under threshold. No guard
inconsistency.

## 3. Predeclared endpoints

| # | endpoint (bar) | result | verdict |
|---|---|---|---|
| 4a | zero runaway-guard trips, ALL windows incl. block-1 pres 1–5 | failures [] × 9 arms; all curriculum-complete | PASS |
| 4b | block-1 pres 1–5 mean burst rate ∈ [100, 200] Hz (E3 band) | GAIN means 185.9 / 193.9 / 189.1 / 201.0 / 197.7 / 197.9 (6 blocked arms); pres 1 = 391–449 Hz (baseline pres 1 = 57–94 Hz) | FAIL |
| 4c | first-pattern posts/n at pres 20 ≥ 0.9 × baseline | 1.00 vs 0.96–1.00; ratios 1.00–1.04 | PASS |
| 4d | max protected mass P ≤ 0.6 | 0.6000 in all 9 arms (baselines also pin 0.6000 — cap engagement unchanged, no violation) | PASS |
| 1 | end-of-block-2 protected mean weight (novel cohort) ≥ 0.09 | bac C: 0.000 / 0.058 / 0.000 (S1/S9001/S424242) vs base 0.054–0.065; bca A (novel in bca): 0.013 / 0.000 / 0.000 vs base 0.051–0.056 | FAIL |
| 2 | first-novel-pattern posts/n ≥ 0.5 | bac first-C: 0.00 / 0.08 / 0.00 (base 0.06–0.12); bca first-A: 0.33 / 0.04 / 0.00 (base 0.15–0.35) | FAIL |
| 3 | novel permanence during block 2 ≥ 150 | bac C perms: 0 / 17 / 0 (base 29–65); bca A perms: 22 / 0 / 0 (base 25–57) | FAIL |
| 5 | IL preservation: C posts ≥ 0.8 × base; C permanence within ±20% of 98 | posts ratios 1.06 / 1.02 / 1.31 (PASS); permanence 93 / 61 / 96 (s9001 = 61 fails band [78, 118]); C end mass HIGHER 0.124–0.155 vs 0.094–0.114 | MIXED — 1/3 fail |
| 6 | gap activity within ±0.5 Hz of baseline | blocked gains 1.88–6.32 vs 0.46–5.94 (all 3 tested pairs |Δ| > 0.5); IL gains 10.1–24.0 vs 0.94–6.59 | FAIL |

## 4. Trajectories (gain s20260912-bac, vs baseline)

First block (A):
- pres 1: posts/n 1.00, burst **391 Hz** (base 85), network drive ≈ 108
  (boost ≈ 95.9 = 8 × R=0-gate × I_W 11.98; base 38).
- pres 2–5: 243 / 124 / 94 / 78 Hz (base ≈ 84). R decays 0 → 0.38 by
  pres 6 (identical trajectory shape to baseline — the gate is the
  allocation-rule R, observed).
- C-afferent churn during A-block: 195 / 202 / 215 prunes (base 164–187).
  ERASER DECOMPOSED: STDP excluded — C channels never fire in block 1, so
  C-cohort STDP events in the A-block = 0 (measured in the weight-growth
  audit); the eraser is M2 rescale pressure (per-neuron t_e targets under
  3×-drive activity) driving the surviving working C weights below
  theta_prune (0.005) → M4 prune. The prune EXCESS over baseline
  (+15 / +31 / +51) matches the baseline survivor populations — the
  boost's block-1 side effect removes the exact afferents the mechanism
  was built to amplify. End-of-block-1 working C current at first-C:
  0.000 vs 81.38 network-total (= 1.565/neuron baseline), verified both
  from snapshot-weighted telemetry and from the pruning record.

First novel exposure (C pres 21): IW_C = **0.000 /neuron total** (base
1.565; snapshot-verified) — every surviving working C afferent gone.
Under the frozen
equation's input-proportional bound (i_boost ≤ k_g·I_W), boost = 0. The
mechanism DID NOT ENGAGE at its target moment. Posts 0 (base 101 spikes).
Repeat in 2/3 bac arms and 2/3 bca arms (iwA = 0.000).

Second block: no recurrent engagement (Irec ≈ 0), no LTP opportunities, C
permanence 0–17, protected C mass 0.000–0.058. No bootstrap.

IL: C first exposure (full gate, intact substrate) recruits fine —
first-5 bursts (C/A) 420–449 Hz pres 1 → decay; C permanence 61–96,
C end mass 0.124–0.155 — but gap afterglow 10–24 Hz (base 0.9–6.6).

## 5. Diagnosis

FIXED-MECHANISM TEST at k_g = 8: **FAIL — recruitment-gain operating-
point failure**, classified FIRST per the frozen interpretation rule.

Chain (measured):
1. Block-1 first exposures carry the full gate (R = 0, I_W = 9–12) →
   boost ≈ 72–96 → drive ≈ 108–115 (~3× the strongest baseline regime);
   burst rates pres 1 = 391–449 Hz, pres 1–5 mean at/above the 200 Hz
   band edge (185.9–201.0) → endpoint 4b FAIL.
2. The hyperactive block 1 accelerates M2/M4 churn on the unseen
   pattern's afferents (C-prunes 195–215 vs 164–187): by block 2 the
   novel pattern's working substrate is ZERO (vs 1.565 baseline).
3. At first novel exposure, I_W = 0 ⇒ boost = 0 by construction ⇒ the
   recruitment gain cannot engage ⇒ posts 0–0.08 (no improvement over
   baseline 0.06–0.12) ⇒ no LTP, permanence 0–22, protected mass ≈ 0.
4. The mechanism's own block-1 side effect (substrate churn under 3×
   drive) destroys the substrate it is designed to amplify.

=> The WEAK-PATTERN RECRUITMENT HYPOTHESIS IS NOT FALSIFIED by this
test: the mechanism never fired at the target moment. The failure is the
OPERATING POINT chosen (k_g = 8 with a strong first block), not the
recruitment concept. Where the pattern arrives first (bca C, IL C/A),
recruitment is intact-to-improved under the same gain (posts ≥ baseline,
end mass 0.09–0.155) — consistent with the drive audit's mechanism
expectation — but the overshoot + gap afterglow persist there too.

## 6. Recorded alternatives (NOT adopted, per 272e0e5 §10.3)

- Threshold-completion cap: i_boost capped at max(0, (v_th − v)·τ_m/dt)
  — parameter-free, binds at strong drive (block-1 first exposures
  cross within 1–2 ticks → cap ≈ 0 → no 3× overshoot), leaves the
  weak-C operating point (first-C availability 12.5 < cap ~20) largely
  untouched. This is the natural follow-up design, requiring a SEPARATE
  registration and user decision.
- No tuning, no bracketing, no upward continuation, no E-number:
  k_g = 8.0 stays whatever it measured — this test is closed.

## 7. Binary verdict

FAIL (operating-point) — 4b, endpoints 1, 2, 3, 6 fail; 4a, 4c, 4d pass;
IL preservation mixed (posts pass, permanence band 1/3 fail). Per the
frozen outcome rules: STOP — no reparameterization in this registration.

STOP.