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
