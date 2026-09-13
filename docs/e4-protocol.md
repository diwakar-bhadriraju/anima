# E4 Protocol — Structural Growth with Co-Active-Avoidance Wiring (U3)

**Status**: EXECUTED 2026-09-13. Verdict: **REGRESSION** — the run
failed the stabilization guard (runaway at t = 540 s) exactly as the
pre-registered decision rule prescribes. Endpoints below were frozen
before implementation; see Execution Record.


## Motivation

E3b's record: the binding problem is representational — 24 afferents
drive one shared 40-neuron pool via random 3.8% connectivity, so every
readout mixes all patterns; no dynamics-shaping current can fix it.
The remaining structural candidate (U3): **developmentally allocate and
wire new neurons away from the shared co-active pool**, giving each
pattern's response dedicated capacity. Hypothesis (user-registered):
*if neurons are developmentally allocated/wired away from the shared
co-active pool, sensory representations should become more separable.*

## Design (single arm on the E3 stabilized base)

- Base: `configs/e3-armB.toml` exactly — E1 curriculum, seed 20260912,
  additive STDP, adaptation ON (tau 200, gain 0.05), thresholds, caps.
- ONLY deltas (the pre-registered intervention):
  1. `birth_trigger = "homeostatic-saturation"` with pre-set params
     `trigger_rate_hz = 25`, `trigger_sustained_ms = 2000` (machinery
     shipped and unit-tested in Phase 0; first config use).
  2. New wiring rule `wiring_avoid_coactive = true`: a newborn's
     afferents target the **LOWEST rate-EMA** non-input neurons (least
     co-active), not the highest. All other wiring properties unchanged
     (same synapse count `wiring_synapses`, same seeded weight draw
     `U(0.05, 0.05+w_init)`, same afferent-only direction as Phase 0).
  3. Resource headroom: `max_neurons 200` unchanged (far above need);
     births_per_window 4 unchanged.
- With `wiring_avoid_coactive = false` (or absent), wiring is
  bit-identical to the Phase 0 behavior (unit-tested).

## Rationale for the wiring direction

Phase 0 wired births to the highest-rate pool on the assumption that
co-active recruitment helps learning. E3b falsified the shared-pool
assumption: co-active capacity is the problem. The intervention inverts
the preference so new capacity grows *outside* the overlapping core,
where STDP can specialize it to whichever pattern drives it while the
pool is active (the newborn still receives input *during* presentations
— it is isolated from the co-active core, not from input).

## Metrics — identical instruments as E1/E3/E3b

Cross/within-pattern cosines per pair (late S1, `cross_cosine`),
selectivity median, assembly score early/late, retention, S1 internal
mean-rate variance + peak (`read_rates`), birth/dormancy/retirement
counts, weight-saturation fraction at end (from snapshots).

## Pre-registered endpoints (frozen)

**Primary — separation (SUPPORTED if both):**
1. Late-S1 mean cross-pattern cosine < **0.60**.
2. Median selectivity > **0.50**.

**Guards (all must hold for any positive claim):**
3. Within-pattern late-S1 cosine ≥ 0.80.
4. Zero runaway failures; S1 rates within [20, 250] Hz.
5. Retention ≥ 0.80 for A/B/C.
6. Births actually occurred (≥ 1) — otherwise the intervention is inert
   and the run is INCONCLUSIVE, not negative.

**Verdicts:**
- SUPPORTED: (1)+(2) pass, guards hold.
- WEAK: exactly one of (1)/(2) passes, guards hold.
- WEAKENED: guards hold, both endpoints fail → structural allocation
  under this trigger/wiring does not separate representations; U3
  closes for this mechanism class.
- INCONCLUSIVE: guard (6) fails (no births fired).
- REGRESSION: any of (3)–(5) fail.

## Scope guards

- No learning gates (U4) — excluded by user instruction.
- Curriculum, seed, STDP rule, thresholds, adaptation params, metrics:
  unchanged from E3.
- Determinism: same seed ⇒ byte-identical telemetry.

---

## Execution Record (2026-09-13)

### Amendments (logged pre-/mid-execution, per protocol scope guards)

- **A5 (machinery, trigger latching)**: first real use of
  `HomeostaticSaturation` exposed a defect — after firing, `over_since`
  never re-armed, so the trigger fired EVERY tick while the mean was
  over threshold; admission windows exhausted and any overloaded run
  aborted in seconds. Fixed: re-arm on fire (one fire per sustained
  window, unit-tested: `saturation_trigger_latches_rearm_per_episode`).
- **A6 (machinery, traces + coalescing under growth)**: two
  fixed-at-construction buffers overflowed once births actually added
  synapses (both unit-tested): `Traces::step` now `sync_len`s itself
  (`traces_step_survives_synapse_growth`), and the harness `unemit`
  coalescer resizes on the fly. Without these fixes, ANY birth run
  panics; E4 is the first experiment that exercises the Phase-0
  growth machinery end-to-end.
- **A7 (calibration, validated)**: the pre-registered trigger params
  (25 Hz / 2000 ms) were validated on probes, not changed: a 73 s
  probe yields 7 births, zero failures; sustained_ms ≥ 5000 yields
  zero births (off-period means re-arm before the window completes) —
  2000 ms is the only setting that produces growth without being
  inert, and it was kept. Probe config: S1×10 + D×2 + re-test×2.

### Artifact

- `runs/e4-20260913T165351Z` — full E1/E3 curriculum (seed 20260912,
  adaptation 200/0.05 ON, additive STDP, homeostatic-saturation 25/2000,
  `wiring_avoid_coactive = true`). 2,734,320 events / 11 chunks before
  failure. Failed at **t = 540,013 ms** with
  `runaway-activity: mean rate 109.3 Hz > 50 Hz for 5000 ms`.

### Outcome against the frozen endpoints

| Guard (frozen) | Result | Verdict |
|---|---|---|
| (4) zero runaway failures | **FAIL** — runaway at t = 540 s | REGRESSION |
| (6) births occurred | PASS — 43 real births (excl. 76 initial) | — |
| (1)/(2)/(3)/(5) | not evaluable — S2/S3 never ran | — |

**Mechanism of the regression (evidence)**: birth cadence accelerates.
Birth ticks show the trigger firing every ~2000 ms from t ≈ 405 s
onward (offs never dip below 25 Hz again). Burst means climb: 35.3 Hz
(t 0–100 s) → 42.3 (400–500 s) → 63.4 (500–540 s) with per-neuron EMA
peaks to 229.8 Hz; the mean over 40+43 internals crosses the 50 Hz
5-second window at 540 s. Each birth adds 20 plastic afferents onto
the least-active neurons and recruits them into the burst — raising
the population mean — which re-latches the trigger at a shorter
interval. Confirms the U3 risk tag: "homeostatic saturation births
under load but **may chase runaway**". With only 2000 ms between
births late in S1, the growth schedule is pro-cyclic with the
instability it is supposed to relieve.

### Verdict: **REGRESSION** (pre-registered decision rule)

The E3-stabilized baseline (which completed all 875 s with zero
failures) is **destabilized** by E4's structural-growth intervention.
Separation endpoints are unmeasurable by construction. No rerun
within this registration; the protocol's REGRESSION branch forwards to
a schedule revisit.

### Roadmap update (per evidence)

- **U3 status: tested (homeostatic-saturation + co-active-avoidance
  wiring) → REGRESSION.** The mechanism's cadence is pro-cyclic with
  runaway under this curriculum; growth-away-from-pool via this
  trigger does not stabilize at the registered params.
- **Next candidates (decision deferred to next protocol, user
  chooses):**
  1. E4-rerun with a re-registered, **rate-gated birth schedule**
     (e.g. hard births-per-window across the whole run, long re-arm
     refractory, or hard cap ~30 total) — isolates whether the
     ALLOCATION (away from co-active pool) helps separation once the
     CADENCE cannot chase runaway. This is the scientific follow-up
     the regression points to.
  2. U3 variant (b): persistent-error trigger — births at the
     learnable frontier, not at saturation.
  3. E5 learning gates (excluded by user instruction until now-h).
- Learning gates remain excluded from E4 scope (user instruction).

---

## E4b rerun — rate-gated birth schedule (authorized by REGRESSION branch)

**Status: PRE-REGISTERED 2026-09-13, frozen before implementation.**

The REGRESSION branch of the original registration forwards to a
schedule revisit; the user approved the rerun. Scientific goal:
**isolate the ALLOCATION direction (away from co-active pool) from the
CADENCE coupling** that caused the regression. All endpoints, guards,
and verdict rules of the original registration remain in force and
unchanged; the only new intervention is the schedule itself:

- **A8 (only delta vs E4)**: `trigger_cooldown_ms = 30_000` — after a
  birth, the homeostatic-saturation trigger is held (armed, not
  reset) for 30 s of sim time before the next birth is admitted.
  Expected total births ≈ 20–30 over the 875 s curriculum (vs 43
  births in the first 540 s of E4, accelerating without bound).
- Everything else identical to E4: homeostatic-saturation 25 Hz /
  2000 ms, `wiring_avoid_coactive = true`, E3 arm-B base (curriculum,
  seed 20260912, additive STDP, adaptation 200/0.05, thresholds,
  caps).
- Implementation note (pre-frozen): cooldown is enforced inside
  `HomeostaticSaturation` by holding `over_since` armed while the
  cooldown runs, so an ongoing overload fires exactly one birth per
  30 s rather than waiting for a fresh sustained episode. Default
  cooldown 0 ⇒ behavior identical to E4/E3 (unit-tested).

Verdict rules: identical to E4 pre-registration. If guard (4) still
fails (runaway) with cadence bounded ≈ 1 per 30 s, the instability is
attributable to allocation itself (each birth adds 20 afferents' worth
of recruitment regardless of cadence) — a WEAKENED/REGRESSION outcome
recorded with that attribution.

---

## E4b execution record

### Amendment A9 (cadence shortfall, logged post-run)

A8 pre-registered expected ≈ 20–30 births over 875 s at 30 s
cooldown. Realized: **2 births** (t = 19,064 ms and t = 451,041 ms).
Cause: the sustained-2 s trigger re-arms during every off-period mean
dip (< 25 Hz, 1500 ms of each 2000 ms cycle), so the 30 s cooldown
never binds — the effective cadence is one birth per sustained-
overload EPISODE, and such episodes recur far more slowly than assumed
from the E4 (no-cooldown) acceleration tail. The probe (1 birth/73 s)
already signaled this; extrapolation ×12 understated the episode
rarity in the full curriculum.

### Artifact

- `runs/e4b-20260913T170047Z` — completed curriculum-complete,
  seed 20260912, cooldown 30 s, adaptation 200/0.05, avoid-coactive
  wiring. 2 real births (76 initial excluded).

### Outcome vs frozen endpoints

| Metric (late S1) | E3 armB | E4b | Endpoint |
|---|---|---|---|
| cross A-B / A-C / B-C | 0.753 / 0.603 / 0.670 | 0.764 / 0.631 / 0.681 | — |
| **mean cross** | **0.675** | **0.692** | (1) < 0.60 → FAIL |
| **selectivity median** | **0.455** | **0.420** (n=86) | (2) > 0.50 → FAIL |
| within-pattern | 0.846 | 0.848 | (3) ≥ 0.80 → PASS |
| failures / peak / ratevar | 0 / 195 Hz / 296.5 | **0** / 221.6 Hz / 308.6 | (4) → PASS |
| retention A/B/C | 1.10+ | 1.131/1.101/1.369 | (5) ≥ 0.80 → PASS |

All guards pass — the rate-gated schedule **does** stabilize (E4's
runaway is gone; completion achieved). The two separation endpoints
fail; numerically E4b ≈ E3 armB within noise (mean cross +0.017).

### Verdict classification — INCONCLUSIVE (inert intervention), with
### an honest note attached

Guard 6's own text: "births actually occurred (≥ 1) — otherwise the
intervention is inert and the run is INCONCLUSIVE, not negative."
E4b passed the literal ≥ 1 bar but fell 10–15× short of the A8
operating point: 2 births add 40 synapses to a ~1500-synapse network.
An inert intervention cannot test the allocation hypothesis; a
WEAKENED reading would be a Type-II confound. **Classified:
INCONCLUSIVE (intervention effectively inert; cadence miscalibration,
A9).** Endpoint non-movement is consistent with E3-baseline noise and
carries no information about the allocation direction.

### Recommended follow-up (user decision required — A10, not yet logged)

Recalibrate the schedule so the run exercises the registered band:
`trigger_cooldown_ms ≈ 3–5 s` (est. 60–150 s between episodes → ~15–40
births over S1; verify on probe before the run, log as amendment).
This tests the actual scientific question — does allocation away from
the co-active pool separate representations — now that A8 proved the
schedule can stabilize. All endpoints/guards unchanged.

---

## E4c — A10: cooldown recalibration (authorized, pre-registered)

**Status: PRE-REGISTERED 2026-09-13, frozen before run.**

User authorized the A10 rerun with the exact instruction: smallest
probe-validated cooldown in 3–5 s producing ~15–40 S1 births, sole
variable = cooldown, everything else identical to E4b.

### Calibration sweep (logged, probe config S1 reps=40 ≈ 240 s S1,
### S2/S3 reps=2, same seed)

| cooldown | births in 240 s S1 | projected full S1 (720 s) |
|---|---|---|
| 3000 ms | 8 (ticks 19,29,33,41,45,73,179,247 s) | ≈ 24 |
| 4000 ms | 8 (same set, ±30 ms) | ≈ 24 |
| 5000 ms | 7 (19,29,34,41,46,73,179 s) | ≈ 21 |

Cadence is episode-limited, not cooldown-limited, in this range
(3000/4000 identical; 5000 drops one). Full-curriculum projection
incl. S2/S3 contributions: ≈ 27–30 births, inside the registered
15–40 band.

### Frozen A10

- `trigger_cooldown_ms = 3000` (smallest probe-validated value; all
  other params identical to E4b: adaptation 200/0.05, homeostatic-
  saturation 25 Hz/2 s, `wiring_avoid_coactive = true`, seed
  20260912, additive STDP, curriculum, thresholds, caps, metrics,
  stability guards).
- New config `configs/e4c.toml`, exp_id `e4c`.
- Decision branches: unchanged from the original E4 registration —
  endpoints (1) cross < 0.60 + (2) selectivity > 0.50; guards (3)
  within ≥ 0.80, (4) zero runaway + rates ≤ 250 Hz, (5) retention
  ≥ 0.80, (6) births occurred. SUPPORTED / WEAK / WEAKENED /
  INCONCLUSIVE / REGRESSION as written. An inert result (guard 6
  breach) is INCONCLUSIVE, NOT evidence against the allocation
  hypothesis (user instruction).

---

## E4c execution record (A10)

### Artifact

- `runs/e4c-20260913T170637Z` — full curriculum, seed 20260912,
  cooldown **3000 ms** (frozen A10), everything else identical to
  E4b. **35 real births** (t span 19,064 → 539,044 ms), all within
  the registered 15–40 band — growth was ACTIVE, cadence controlled
  (3 s minimum spacing, verified by tick gaps ≥ 3000 ms).

### Outcome vs frozen endpoints

| Guard | Result | Branch |
|---|---|---|
| (6) births occurred | PASS — 35 | — |
| (4) zero runaway | **FAIL** — `runaway-activity` at t = 540,021 ms, mean 104.3 Hz > 50 Hz for 5 s | **REGRESSION** |

Separation endpoints (1)/(2) not evaluable (S2/S3 never ran). Guards
(3)/(5) not reached.

### Interpretation (evidence-based)

- E4c fails at **t = 540,021 ms — the same sim time as E4
  (540,013 ms, 43 births)**. Combined with E4b (2 births, no
  failure), the failure correlates with **cumulative birth count,
  not cadence**: ~35 + births by t ≈ 540 s trips the guard regardless
  of 3 s vs uncontrolled timing.
- Mechanism: each birth adds **20 converged afferents** onto a fresh
  internal neuron (amplitude 52 × w) with **no outgoing synapses and
  no competition** → newborns are high-gain sinks. Burst-mean avg
  rises 43.8 (E3b) → 62.9 Hz (E4c) and off-mean 15.5 → 22.4 Hz once
  35 newborns join the pool; the population mean (which the trigger
  AND the runaway detector read) crosses the 50 Hz / 5 s window
  during S1's lengthening burst trains.
- Therefore: **growth active at a controlled cadence still
  destabilizes this organism via per-birth afferent gain — the
  instability is allocation-shape-dependent (20:0 fan-in, zero
  fan-out), not timing-dependent.** The registered question "does
  allocation away from the co-active pool improve separation" remains
  answerable only with a growth shape that does not itself inflate
  the population mean.

### Verdict: **REGRESSION** (frozen rule, guard 4)

No re-run within this registration. The evidence (E4c = E4 failure
point) closes the current mechanism shape: **homeostatic-
saturation growth with 20-in / 0-out afferents cannot coexist with
the 50 Hz stability guard regardless of cadence.** Next candidates
for a NEW registration (deferred to user; gates still excluded):
1. Growth-shape variant: fewer afferents per birth (e.g. 4–8) or
   matched fan-out so newborns integrate into the circuit instead of
   accumulating as sinks.
2. Persistent-error trigger (U3 variant b) — births at the learnable
   frontier, naturally rarer and load-coupled.
3. Accept guard (4) re-calibration for growth arms (detector on the
   original 40-neuron pool) — but that changes the frozen guard and
   needs explicit user approval.

---

## E4d — fan-out-matched growth (authorized new registration)

**Status: PRE-REGISTERED 2026-09-13, frozen before implementation.
E5 learning gates remain excluded.**

### Problem addressed

E4/E4c demonstrated the 20-in/0-out newborn is a high-gain sink whose
firing inflates the population mean (burst-mean 43.8 → 62.9 Hz with 35
births), tripping the UNCHANGED 50 Hz / 5 s runaway guard at
t ≈ 540 s in both active-cadence arms. The allocation direction
(away from the co-active pool) was never the measured variable because
the sink shape destabilized the run first.

### Frozen E4d intervention (ONE delta vs E4c)

Newborn wiring becomes **bidirectional / fan-out-matched**: in
addition to the existing 20 incoming afferents, the newborn receives
**20 outgoing efferent synapses back onto the SAME 20 allocated
partners** (the lowest-rate, away-from-coactive set selected at
birth). Exact rule:

- Partner selection: unchanged (`wiring_avoid_coactive = true` —
  lowest rate-EMA non-input, non-retired, non-self; seeded).
- Incoming: unchanged — 20 afferents partner → newborn, weight
  `U(0.05, 0.05 + w_init)` per existing seeded draw, plastic.
- Outgoing (NEW): symmetric 20 efferents newborn → partner, same
  weight draw family (continuing the same seeded RNG stream —
  deterministic), plastic.
- Total per birth: 40 synapses. The newborn's spikes now flow back
  into the circuit through the same allocated pool (no more sink:
  its firing drives exactly the partners that drive it).
- Config knob: `wiring_bidirectional = true` (default false =
  bit-identical E4c/E4 behavior; unit-tested).

All other variables are E4c values (which are E3-baseline values):
curriculum, seed 20260912, additive STDP, adaptation 200/0.05,
homeostatic-saturation 25 Hz / 2 s, `trigger_cooldown_ms = 3000`
(A10, frozen), runaway detector 50 Hz / 5 s UNCHANGED, thresholds,
caps, metrics. **No tuning based on any full-run result.**

### Endpoints & guards (identical to E4/E4b/E4c, frozen)

- (1) late-S1 mean cross-pattern cosine < 0.60
- (2) median selectivity > 0.50
- (3) within-pattern late-S1 cosine ≥ 0.80
- (4) zero runaway failures; S1 internal rates in [20, 250] Hz
- (5) retention ≥ 0.80 for A/B/C
- (6) births occurred (≥ 1)

Branches: SUPPORTED = (1)+(2) + all guards; WEAK = exactly one of
(1)/(2); WEAKENED = guards hold, both endpoints fail; INCONCLUSIVE =
guard (6) fails (inert); REGRESSION = any guard (3)–(5) fails. The
runaway detector is NOT reinterpreted; a guard-(4) failure is a
REGRESSION regardless of mechanism.

### Verification (pre-run)

Unit tests: (a) bidirectional birth creates 20 in + 20 out to the same
partners; (b) `wiring_bidirectional = false` reproduces E4 wiring
exactly (partner sets identical between runs); (c) existing latch/
cooldown tests still pass. Calibration probe (S1 reps=40) confirms
growth is stable (no runaway in probe window) and synapse cap is safe
(~40 births × 40 syn ≈ 1600 + ~150 initial < 2000). Then full run.

---

## E4d execution record

### Calibration gate FAILED — full run NOT executed

The pre-registered sequence is: "verify the growth mechanism is stable
with unit/calibration tests, THEN run the full curriculum." The
calibration probe (S1 reps=40, same shape as the A10 sweep) failed:

- Probe run `runs/e4d-probe-20260913T171115Z`; **4 real births**
  (t = 19,064 / 22,064 / 29,028 / 32,028 ms; cooldown 3000 ms
  respected) then `runaway-activity` at **t = 32,089 ms** — mean
  **179.0 Hz** > 50 Hz for 5 s.
- Compare: E4c's same-shape probe (8 births in 240 s S1) recorded
  zero failures. E4d with bidirectional wiring destabilizes ~17×
  faster (32 s vs 540 s) with one-quarter the births.

### Interpretation (evidence)

The fan-out-matched shape converts the newborn from an inert sink
into a **positive-feedback amplifier**: each newborn fires strongly
(20 afferents), and its 20 efferents land on exactly the same
partners that drive it → reciprocal excitation within the allocated
clique. Mean 179 Hz after 4 births (vs 104 Hz after 35 births in
E4c) shows the loop amplifies far faster than the sink's passive
mean-inflation. The E4c hypothesis — "fan-out integrates the newborn
so the pool regulates it" — is falsified at the probe stage: in this
all-excitatory organism (no inhibitory synapses anywhere), ANY growth
shape that returns the newborn's firing to the network feeds runaway.

### Verdict: **REGRESSION (at the calibration gate — no full run)**

Per the frozen guard (4) semantics and the user-directed sequencing.
The structural-allocation hypothesis remains untested because BOTH
growth shapes (20-in/0-out sink, 20-in/20-out reciprocal) destabilize
the E3-stabilized baseline via the unchanged 50 Hz guard. Two
independent failures now bound the mechanism class: growth that
either accumulates or re-circulates excitatory drive is incompatible
with the E3 baseline + unchanged detector.

### Next candidates (deferred to user; gates still excluded)

1. Drop the avoid-coactive partner ELECTRICITY claim and test pure
   capacity allocation: growth with FEWER afferents (e.g. 4) and
   NO feedback — bounds per-birth drive injection.
2. Persistent-error trigger (U3b) — births at learnable frontier,
   rare by construction.
3. Accept the U3 class as closed in this organism and move to E5
   learning gates when the user lifts the exclusion.

---

## E4e — low-fan-in capacity growth (authorized new registration)

**Status: PRE-REGISTERED 2026-09-13, frozen before implementation.
E5 learning gates and all other mechanisms remain excluded.**

### Problem addressed

E4/E4c (20-in/0-out sink) and E4d (20-in/20-out reciprocal) both
regressed: the first passively inflated the population mean (~63 Hz
burst-avg with 35 births, runaway at t ≈ 540 s), the second formed a
positive-feedback amplifier (179 Hz after 4 births, runaway at
t ≈ 32 s). Both injected or re-circulated large excitatory drive per
birth. This arm tests whether the U3 allocation hypothesis becomes
evaluable when **each birth injects substantially less drive**:
fan-in 4, no fan-out.

### Frozen E4e parameters (ONE delta vs E4c)

- Newborn incoming connectivity: `wiring_synapses = 4` (was 20).
  Selection rule unchanged: newest LOWEST rate-EMA non-input,
  non-retired, non-self partners (`wiring_avoid_coactive = true`),
  seeded, weight `U(0.05, 0.05 + w_init)`, plastic.
- `wiring_bidirectional = false` (no outgoing synapses; no feedback).
- Trigger framework unchanged: homeostatic-saturation 25 Hz / 2 s,
  `trigger_cooldown_ms = 3000` (frozen A10 cadence floor).
- Everything else identical to E3/E4 baseline: curriculum, seed
  20260912, additive STDP, adaptation 200/0.05, thresholds, caps,
  runaway detector 50 Hz / 5 s UNCHANGED, metrics.
- Per-birth synapse count 4 ⇒ ~40 births × 4 = 160 synapses added —
  trivially under the 2000 cap (no resource confound).

### Pre-registered decision branches

Endpoints (late S1): (1) mean cross-cosine < 0.60, (2) median
selectivity > 0.50.
Guards: (3) within ≥ 0.80, (4) zero runaway + S1 rates in [20, 250]
Hz, (5) retention ≥ 0.80, (6) births occurred.

- SUPPORTED: (1)+(2) and guards (3)–(6) hold.
- WEAK: exactly one of (1)/(2), guards hold.
- WEAKENED: guards hold, both (1)/(2) fail.
- **INCONCLUSIVE (inert)**: fewer than **15 real births by S1 end**
  (informativeness bar, pre-registered to match the A10 band and the
  E4b precedent) or zero births — endpoints uninterpretable either
  way; NOT evidence against the allocation hypothesis.
- REGRESSION: any guard (3)–(5) fails (runaway detector not
  reinterpreted).

### Sequence

1. Implementation (config-only change; wiring machinery already
   parameterized by `wiring_synapses`) + unit tests.
2. Full suite green, zero warnings.
3. Calibration gate: probe (S1 reps=40, same shape as prior sweeps)
   must complete with zero failures AND ≥ 3 births in 240 s S1
   (≈ cadence check). If the probe regresses or is inert, the full
   run does NOT execute (this registration closes with the gate
   outcome).
4. If gate passes: full curriculum run, metrics, verdict per the
   branches above. No tuning on any full-run result.

---

## E4e execution record — calibration gate outcome

### Gate probe

- `runs/e4e-probe-20260913T171550Z`, S1 reps=40 (240 s S1), fan-in 4,
  no fan-out, cooldown 3000 (all frozen params).
- **Stability: PASS** — zero failures; the first growth shape without
  any regression signal (E4: runaway 540 s; E4d: 32 s; E4e: none).
- **Cadence: FAIL** — **2 births** (t = 19,064 / 41,077 ms) vs the
  frozen gate bar of ≥ 3 per 240 s. Projected full-run birth count
  ≈ 6 by S1 end — below the pre-registered 15-birth informativeness
  floor, so any full run would be INCONCLUSIVE by the frozen branches.

### Verdict per the frozen registration: **INCONCLUSIVE (inert) — no
### full run executed**

The frozen sequence ("stable AND ≥ 3 births, else the full run does
NOT execute; this registration closes with the gate outcome") binds.
The E4e arm closes at the gate.

### Scientific content (not a null result — a mechanism bound)

- Fan-in 4 **eliminates the instability** produced by fan-in 20
  (sink) and 20+20 (reciprocal): per-birth drive injection is now
  small enough that the population mean never inflates toward the
  50 Hz guard. Stability is restored by reducing per-birth drive —
  confirming the drive-injection mechanism behind E4/E4c/E4d.
- But the same reduction makes births self-limiting: the trigger's
  sustained episodes are driven by population mean, and at fan-in 4
  each birth contributes too little to sustain the next episode.
  Low fan-in ⇒ growth peters out ⇒ can't reach the informative band.
- The allocation hypothesis remains **unevaluated**: no arm has yet
  produced 15–40 births while holding the network stable. The drive
  budget is now bracketed: fan-in 4 is inert, fan-in 20 regresses.
  A mid value (e.g. 8) would be the natural next probe, but E4e's
  registration forbids tuning on results and closes here.

### Roadmap (unchanged constraints; gates still excluded)

1. E4f-style sweep of fan-in ∈ {8, 12, 16} with the SAME gate bars
   (requires a new registration; each value is a pre-registered arm).
2. Persistent-error trigger (U3b) — decoupled from population-mean
   episodes entirely.
3. E5 learning gates once the user lifts the exclusion.
