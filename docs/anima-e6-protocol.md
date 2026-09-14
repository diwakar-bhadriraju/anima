# ANIMA E6 — Pre-registered Protocol (frozen before implementation)

**Status: PRE-REGISTERED.** Written before any E6 code or config.
Freezes the entire validated v2 organism + v3 overlap curriculum and
adds exactly ONE registered construction — **plasticity-event rate
balancing (E6)** — selected by the user on 2026-09-15 from
evidence-tagged candidates (charter: candidate approval before
implementation). Any frozen value may change ONLY via a logged,
user-approved amendment; failed predictions are recorded, never
retuned.

> **Naming note (registered):** the phase-0 plan listed "E6 =
> reward-modulated STDP" (unknowns-registry U7 placeholder). Per user
> direction 2026-09-15, the E6 designation now names THIS experiment
> (resolving U9). U7 remains open, re-deferred.

---

## 0. Primary question (frozen wording)

> **H6: Under the overlapping v3 curriculum (outcome C: shared
> channels co-activate at 2× the rate of category-exclusive channels
> and win every plasticity contest), does per-event plasticity rate
> balancing — scaling an input channel's plasticity increments by the
> inverse of its locally-estimated event rate — restore
> category-exclusive feature selection and representation
> separation?**

No answer is assumed. All four registered outcome classes
(section 10) are valid.

## 1. Diagnosis being tested (frozen background, from v3 execution)

- v3-full (seed 20260912): selectivity median 0.168 (≤ 0.50 ⇒ P3
  not supported); cross A-B 0.697 / A-C 0.009 / B-C 0.645; zero of
  21 specialized neurons used the category-exclusive channels {0-3}
  or {12-15}; signatures {A+B}×7, {B+C}×14, BROAD×19 only.
- Channel mean rates over S1: exclusive channels {0-3} ∪ {12-15}
  fire in 120 presentations (1.67 Hz mean); shared channels {4-11}
  fire in 240 (3.33 Hz) — a 2:1 co-activation/rate advantage.
- Registered conclusion (U9): with additive STDP and the M2 T_e
  conservation law, every plasticity contest (potentiation,
  candidate permanence, budget) is won proportionally to
  co-activation count, so the organism wires the shared feature and
  ignores the only discriminative evidence.

## 2. Freeze (everything from v2/v3, EXCEPT the single registered
### construction)

The complete v2 organism as validated (v2 protocol + M3-1 +
v2-full.toml values) and the v3 overlap curriculum (v3 protocol §3)
are frozen: neuron model, thresholds, adaptation, STDP (pairwise
additive: a⁺ 0.005, a⁻ 0.0053, τ 20 ms, w ∈ [0,1], trace semantics),
M1, M2, M4, M5, M6, M3 candidate lifecycle (C 6, w_c_init 0.01,
Δ_perm 0.01, decay_c 0.99, θ_permanent 0.05, w_c_permanent 0.02,
θ_die 0.005, p_cand 0.5), structural window 100 ticks
(M4→M3→M2→M6→budget), RNG behavior and seeds (20260912 / 9001 /
424242), telemetry v2, snapshots, replay, viz, resource limits,
analysis instruments (v3_analysis, cross_cosine, metrics.json —
untouched). The v3 curriculum is used as-is (pattern channel sets
A {0-7}, B {4-11}, C {8-15}, D {0-15}; 500 ms / 1500 ms; 20 Hz ±2 ms;
S1 120×3, S2 30, S3 15).

**The registered exception (the whole experiment):** per-event
plasticity increments on input-channel afferents are multiplied by a
local rate-balancing factor β (§3). Everything else — including LTD
(a⁻), M6 increments, M4, M5, M2's rule — is untouched.

## 3. The E6 construction (exact, frozen)

### 3.1 Channel rate estimate φ

- Per input channel j (0..23), an event-rate EMA over structural
  windows, updated at the **start** of every structural window from
  the **previous** window's counts (one-window lag, registered):
  `φ_j ← (1 − α)·φ_j + α·(c_j / 100)`
  with `c_j` = number of ticks in the previous window on which
  channel j emitted ≥ 1 spike (input events, as delivered by the
  environment; per-tick events, not multiplicity).
- `α = 1/25` (τ_φ = 25 windows = 2.5 s); `φ_init = 0.02`
  events/tick (≈ 20 Hz reference); floor `φ_min = 0.001`
  events/tick. All three frozen.
- φ is a pure function of the organism's observed input history —
  no RNG, no labels, no stage/pattern knowledge (D9), no config
  leaks. Same seed ⇒ same φ.
- S0 (5 s silence) drives φ toward the floor; ratios stay ≈ 1
  during silence (all channels equal), so the pre-S1 transient is
  inert (β ≈ 1), registered.

### 3.2 Balance factor β

- For every synapse or candidate whose **pre is an input channel j**
  on a given non-input neuron n:
  `β_j(n) = φ̄_n / φ_j`, where `φ̄_n` = mean of φ over the neuron's
  **live input-channel afferents** (M3 candidates: over the
  candidate pool's eligible input partners — the same set
  definition), else β = 1 (recurrent, inhibitory, all other sites).
- β clamped to `[0.1, 10]` (registered safety range; under this
  curriculum the natural range is [0.5, 2], so the clamp is
  inert-by-construction — recorded as a guard, not a knob).
- Local: a neuron needs only its own afferents' φ values.

### 3.3 Where β is applied (the ONLY plasticity changes)

1. **STDP potentiation (a⁺ branch)**: for an input-channel afferent
   synapse, `Δw = a⁺·β_pre` on each potentiation event. All other
   STDP terms (a⁻ LTD, traces, saturation, coincidence skip)
   **unchanged**.
2. **M3 candidate co-active accumulation**: on a co-active window,
   `w_c += Δ_perm·β_pre` (pre = the candidate's partner channel).
   Decay, θ_permanent, θ_die, redraw: **unchanged**.
3. **M6, M4, M5, M2: unchanged.**

### 3.4 Identity, determinism, boundedness (registered properties)

- **Identity**: if all φ_j were equal (incl. frozen init), β ≡ 1 and
  E6 reduces EXACTLY to v2/v3 plasticity. Verified by unit test
  (φ-init window behavior) and end-to-end by control C0 (§6).
- **Determinism**: φ is history-derived, no RNG; same seed ⇒
  byte-identical telemetry (existing property preserved).
- **Boundedness**: β ∈ [0.1, 10] and increments stay ≤ 10× the
  frozen rates; no new unbounded state.
- **Engagement readout (registered)**: final φ per channel and the
  β distribution over afferents are reported by the analyzer; in a
  converged S1, channel 0-3/12-15 φ must measure ≈ half of 4-11 φ
  (1.67 vs 3.33 Hz) — proof the mechanism ran.

### 3.5 M3-1 permanence timing re-derived (registered math)

M3-1 per-event co-active increment becomes `Δ_perm·β`. Under the
frozen stimulus statistics (P(pre fires in an ON window) = 0.8647;
OFF windows decay ×0.99), with β ∈ [0.5, 2]:

| β | w_c after 4 co-active windows | co-active windows to θ_permanent | consequence |
|---|---|---|---|
| 1.0 (v2) | ≈ 0.05 | 4 | M3-1 reference |
| 1.5 (exclusive in mixed neuron) | ≈ 0.07 | 3 | exclusive candidates reach permanence FIRST |
| 0.75 (shared in mixed neuron) | ≈ 0.04 | 5–6 | shared candidates delayed |
| 0.5 (shared, β floor of natural range) | ≈ 0.03 | ≈ 8 | last, on par with v2's slowest |

Registered consequence: E6 inverts the v3 permanence ordering for
mixed-afferent neurons (per-event fairness ⇒ exclusive-bias in
synaptogenesis) while keeping permanence reachable within a
presentation for every channel class. Strike-through rule: if
measurements show permanence NOT reachable for any class (0
established by S1 end), the arm is INCONCLUSIVE per §7 (machinery
malfunction), not a tuning signal.

## 4. Configs and parameters (frozen; hashes recorded at creation)

- `configs/e6-full.toml` — v3 curriculum + `[e6]` section
  (`enable = true`, α, φ_init, φ_min, β clamp). exp_id "e6".
- `configs/e6-v2curriculum.toml` — control C1: **v2 disjoint
  curriculum** (v2-full pattern sets) + `[e6] enable = true`.
  exp_id "e6-v2cur".
- `configs/e6-seed9001.toml`, `configs/e6-seed424242.toml` — v3
  curriculum + e6, seeds 9001 / 424242.
- All other sections byte-identical to their v3/v2 sources
  (enforced by test). `enable = false` ⇒ mechanism inert ⇒
  byte-identical behavior (C0 test).
- RunStarted records sha256(config file) as before.

## 5. Endpoints and predictions (frozen, a priori bars)

Identical instruments to v2/v3 throughout (v3_analysis,
cross_cosine, metrics.json; v2 definitions verbatim: established =
candidate-permanence alive at t+10 s; RF snapshot t = 715,000 ms;
late-S1 = second half of S1; P1-v3 = specialized fraction with v3
category sets).

- **H6a — exclusive-evidence feature selection restored**: ≥ **5 of
  40** internal neurons specialized AND using exclusive evidence
  (D ∩ ({0-3} ∪ {12-15}) ≠ ∅). Rationale: balanced per-event
  leverage makes the half-rate discriminative channels competitive;
  5 = minimal detectability bar above the v3-full outcome of 0.
- **H6b — separation**: late-S1 selectivity median > **0.50**
  (verbatim P3 bar).
- **H6c — cross-pattern separation**: mean cross-cosine < 0.60
  AND — registered stricter refinement, motivated by the v3 finding
  that the mean can be carried by the shared-channel-free A-C pair —
  **every pair mean < 0.60** (A-B, A-C, B-C). Pre-registered now,
  not after data.
- **H6d — concentration preserved**: P1-v3 specialized fraction
  ≥ **0.25** (≥ 10/40, the standing bar — no regression of
  specialization itself).
- **H6e — stability**: P2 verbatim (0 failures; S1 rates in
  [20, 250] by the established reading; budget invariant every
  window).
- **Secondary, reported, no bar**: D-condition (S2/late-S1 rate
  ratio; cosine(D, A/B/C) — v3-full measured 0.979 vs B);
  signature distribution incl. pure {B} (absent-detection may exceed
  this mechanism — registered expectation, not failure);
  retention (verbatim metrics.json).

## 6. Controls (frozen)

- **C0 (inertness, test-level)**: `e6.enable = false` ⇒ structural
  code path identical to v3-full; unit test asserts β ≡ 1 at init
  and disabled config produces byte-identical short-run telemetry
  vs the same config without the [e6] section.
- **C1 (known-good regime)**: `e6-v2curriculum` — E6 on the
  DISJOINT v2 curriculum must keep v2's regime: P2 supported AND
  P3 supported (verbatim bars) AND P1-v3 fraction ≥ 0.25. If C1
  fails, E6 damages the known-good regime → verdict D regardless of
  E6-full.
- **Comparator**: the registered v3-full run
  (`runs/v3-20260914T164140Z`, config sha `5854a191…`) — same seed,
  same curriculum, same instruments; E6-full differs only by the
  mechanism. No rerun required.
- **M3-1 correspondence**: v3's P3 mean-cross refinement (§H6c) is
  carried only inside E6's own bars; v2-pair comparisons in the
  report use per-pair values, never a changed bar for v2/v3 verdicts.

## 7. Failure, degenerate, inconclusive (verbatim v2 §11)

Failure/REGRESSION = any Failure event → stop and diagnose (no
silent repair). Degenerate = ≥ 90% internals with zero live
afferents at RF snapshot. Inconclusive = zero established changes by
S1 end; telemetry loss; determinism/config-hash violation. P2
failing makes P1/P3 uninterpretable.

## 8. Gates and run order (frozen)

1. **E6-full** (seed 20260912) → report H6a–H6e + secondary
   endpoints. Gate: P2 (H6e) supported AND complete telemetry →
   authorize C1 + cross-seed; else stop and diagnose.
2. **C1** (`e6-v2curriculum`, seed 20260912) — control arm.
3. **Cross-seed** (9001, 424242, v3 curriculum + E6) → P4 verbatim
   (|Δ mean H| < 0.20, |Δ specialized fraction| < 0.20, top-channel
   Jaccard < 0.50, signature-type agreement ± 0.20).
4. No further arms: the construction is a single mechanism; C0 is a
   test, not a run. No parameter sweeps, no new mechanisms, no
   retuning. Concurrent runs at nice 10.

## 9. Analysis (frozen; measurement-only, reusing v3 instruments)

v3_analysis (extended verbatim with the §3.4 φ/β engagement readout
— a new measurement print, no metric redefinition), cross_cosine,
metrics.json (selectivity/retention/assembly). All comparisons use
the SAME code paths as v2/v3 runs. No metric is altered after data.

## 10. Verdict tree (frozen; every outcome valid)

| Outcome | Definition | Meaning |
|---|---|---|
| **A — feature selection restored** | P2 + H6a + H6b + H6c | per-event rate balancing re-opens the discriminative channels and separation returns under overlap |
| **B — separation without exclusivity** | P2 + H6b + H6c, H6a fails | weights rebalance (selectivity) but RFs stay on shared evidence |
| **C — insufficient construction** | P2 holds, H6b OR H6c fails | shared-channel dominance is deeper than per-event leverage; negative result, registered |
| **D — damaging** | P2 fails, P1-v3 fraction < 0.25, or C1 fails | E6 harms the frozen organism; revert, register |
| REGRESSION / DEGENERATE / INCONCLUSIVE | §7 | per definitions |

Nothing is "tuned away". Outcome C or B is a complete, honest
result; the follow-up (if any) is a new registration.

## 11. Observability and reproducibility (unchanged)

Live viz, replayable telemetry, structural events, resource usage,
RF stats, deterministic seeds, commit + config hashes, reproducible
run dirs. Any observability failure stops the run and is diagnosed.

---

## Appendix (filled at execution time)

- Config hashes (sha256 of raw file, recorded at creation before any
  run): e6-full `2cdc8beef39913a3e25c9be9c9043917422756ae79eccd9b855b583f789e0ecc`;
  e6-v2curriculum `359403470b716c322333e053caad0436e9090ebcb8c597151269f3abf2845204`;
  e6-seed9001 `0aaeec2757118d824eed265b8e44e70b57a2e567ca6189f10903aa8026ea8955`;
  e6-seed424242 `3c882b1763738959b25e607a17863427d3b381b81352850641c8083bb2e8eee0`.
- Commit (implementation): `(recorded at run time)`.
- Permanence reachability check (recorded at run time): established
  ≥ 1 by S1 end (else INCONCLUSIVE per §7).