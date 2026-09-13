# E3b Protocol — Lateral Inhibition on the Stabilized E3 Base (U1-inhibition)

**Status**: EXECUTED 2026-09-13. Verdict: **hypothesis WEAKENED** (all
guards passed; both separation endpoints failed). Endpoints below were
frozen before implementation; see Execution Record.

## Motivation

E3 closed the rate-instability branch: adaptation (U1-current) bounds
firing but leaves late-S1 cross-pattern cosine at 0.675, identical to
arm A (0.673). The E3 record concludes separation is circuit-limited:
patterns A/B/C drive overlapping interneurons with **no mechanism
penalizing co-activation** — a neuron responding to A is equally happy
to join the B response. Lateral inhibition introduces exactly that
penalty: neurons that fire together mutually suppress each other, so a
response recruited by pattern A becomes harder to recruit again by
pattern B within the same tick. Primary hypothesis (user-registered):
**lateral inhibition introduces competition between co-active
neurons/populations and improves assembly separation.**

## Design (single arm; within-run comparison vs pre-registered baselines)

- Base: `configs/e3-armB.toml` — E1 curriculum/rule/seed, adaptation
  ON (tau 200 ms, gain 0.05). Everything preserved; the ONLY new
  intervention is `inhibition_gain` (below).
- Mechanism (pre-specified): within each tick, every non-input spiker
  deposits an extra **inhibitory** current `−inhibition_gain` into
  `i_syn` of every OTHER non-input spiker of the same tick (1-tick
  delay, same exponential `tau_syn` kernel as excitatory current).
  Input channels (D8 pure spike sources) are untouched. `gain = 0`
  reproduces E3 exactly (unit-tested).
- Arm naming: **E3b** (`exp_id = "e3b"` in a distinct run; comparison
  arm = E3 arm B record `runs/e3b-20260913T135736Z` and arm A/E1).
- Gain selected by logged calibration sweep (amendment, as in E3 A2):
  minimal gain keeping S1 burst internal rates inside [100, 200] Hz
  with zero failures; calibration runs on a 10-rep probe, never on the
  full curriculum.

## Metrics — identical instruments as E1/E3 for direct comparability

Assembly score early/late S1, selectivity median (n), retention
(A/B/C), S1 internal mean-rate variance + peak (from snapshots via
`read_rates`), late-S1 cross/within-pattern cosines per pair (via
`cross_cosine`). Same tools, same windows, same seed.

## Pre-registered endpoints (frozen)

**Primary endpoint — assembly separation (SUPPORTED if both):**
1. Late-S1 mean cross-pattern cosine < **0.60** (absolute drop ≥ 0.07
   from both baselines 0.673/0.675).
2. Median selectivity > **0.50** (E1/E3 both sat at 0.43–0.46).

**Secondary guards (must hold to claim the mechanism, not a lesion):**
3. Within-pattern late-S1 cosine ≥ **0.80** (separation must not come
   from noise collapse of responses).
4. Stabilization retained: zero runaway failures; S1 internal rates
   within [20, 250] Hz.
5. Retention preserved: S3/late-S1 ≥ 0.80 for A/B/C.

**Verdicts:**
- Separation SUPPORTED: (1)+(2) pass and guards (3)–(5) hold.
- Separation WEAK: exactly one of (1)/(2) passes with all guards.
- Hypothesis WEAKENED: guards pass but (1) and (2) both fail
  (cross-cosine ≥ 0.66) → competition-at-a-tick is insufficient;
  next candidate becomes circuit morphology or learning gates (E5),
  per roadmap — decision deferred, not part of this registration.
- REGRESSION: any guard fails (inhibition destabilizes or destroys
  responses) → record and revisit gain schedule before any rerun.

## Scope guards

- No changes to curriculum, STDP rule, structural machinery,
  thresholds, seed, adaptation parameters, or telemetry.
- All code outside the inhibition deposit path bit-identical (gain-0
  identity test in-suite).
- Determinism: same seed ⇒ byte-identical chunk telemetry.

---

## Execution Record (2026-09-13)

### Amendment

- **A4 (calibration)**: inhibition gain fixed at **0.5**. Sweep
  {0.5, 1.0, 2.0} on 10-rep probes: S1 burst internal EMA peaks 158/157/
  155 Hz — all inside [100, 200]; all zero-failure. Probe cross-cosines
  showed no separation at any gain (0.837/0.938/0.948 for A-B at 0.5/
  1.0/2.0 — within-probe noise), so the minimal gain 0.5 was taken per
  the protocol rule.

### Artifacts

- Run: `runs/e3b-inh-20260913T141717Z` (full curriculum, seed 20260912,
  ~5.2M events, 25 MB chunk telemetry; report regen 44 s).
- Config: `configs/e3b-inh.toml` (only delta vs E3 arm B:
  `inhibition_gain = 0.5`).

### Endpoints vs pre-registered thresholds (auto-generated — review)

| Metric (late S1) | E1 / arm A | E3 arm B | E3b-inh (this run) | Threshold |
|---|---|---|---|---|
| cross A-B | 0.746 | 0.753 | 0.768 | — |
| cross A-C | 0.604 | 0.603 | 0.695 | — |
| cross B-C | 0.667 | 0.670 | 0.758 | — |
| **mean cross** | **0.673** | **0.675** | **0.740** | (1) < 0.60 → **FAIL** |
| **selectivity median** | **0.428** | **0.455** | **0.366** (n=81) | (2) > 0.50 → **FAIL** |
| within-pattern | 0.872 | 0.846 | 0.876 | (3) ≥ 0.80 → PASS |
| runaway failures | 0 | 0 | 0 | (4) → PASS |
| S1 rates (peak max EMA) | ~86 Hz mean band | 194.6 Hz | 241.2 Hz | (4) ≤ 250 Hz → PASS |
| retention A/B/C | 1.04/1.04/1.26 | 1.13/1.10/1.39 | 1.12/1.04/1.00 | (5) ≥ 0.80 → PASS |

S1 internal mean-rate variance: arm A 317.3 → arm B 296.5 → E3b-inh
237.2 (most stable of the three).

### Verdict: **hypothesis WEAKENED**

All secondary guards hold — the mechanism is not a lesion: responses
stay coherent (within 0.876), rates bounded, retention preserved, and
rate stability actually improved. But same-tick co-active competition
moved separation in the WRONG direction (mean cross-cosine 0.673 →
0.740; selectivity 0.455 → 0.366): penalizing simultaneity suppresses
the shared late-arriving interneurons that would otherwise differentiate
patterns, while leaving the dominant shared-input component of the
response intact.

### Interpretation and implication for the roadmap

Two consecutive interventions (E3 adaptation, E3b inhibition) leave
cross-pattern cosine ~0.67–0.74 while the network stays healthy. The
binding problem in this organism is not a dynamics problem (rates,
competition-at-a-tick) — it is a **representation problem**: 24 afferents
split A/B/C drive a common 40-neuron pool through random 3.8%
connectivity, and every readout mixes all three responses. Mechanisms
that reshape dynamics cannot fix overlap that is present in the
connectivity itself. The pre-registered WEAKENED branch applies: the
next candidate interventions are structural (targeted growth, U3/E4 —
birth wired away from the co-active pool) or readout gating (U4/E5),
not further current-shaping. Decision deferred to the next protocol;
no reruns within E3b.
