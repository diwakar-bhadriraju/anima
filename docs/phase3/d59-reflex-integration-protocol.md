# D-59: Reflex Novelty Integration + Motor Surface + Encoder Interface

Status: REGISTERED (pre-registration written BEFORE the gate run; results appended after).

## Questions

1. D-59a (reflex in-life): does the verified D-58 reflex rule (K=8 novelty
   nodes, min-L2 > th_fam=60) detect new-vs-known during REAL closed-loop
   life (survival loop beats, STDP on, growth on) — and does detection
   survive selection-driven growth across generations?
2. D-59b (motor surface): with a rate-proportional motor mapping, does a
   closed-loop consequence (action -> world -> afferent rate change) let
   the organism's own reflex circuit register "this does NOT do that"?
3. D-59c (encoder interface): pluggable SensoryEncoder trait so future
   retina/cochlea encoders are drop-in (foundation only in this plan).

## Registration (constants frozen, NOT tuned on outcomes)

- Reflex: K = 8 reflex nodes (d58-verified optimum), th_fam = 60.0,
  templates = per-symbol reflex-band spike counts overwritten on every
  known beat (state-matched by construction), verdict = min-L2 >
  th_fam, no extra probe presentations (rides real beats).
- Motor world: drive = mean(12 output rates)/1000 clamped [0,1];
  consequence eff = clamp(1.0 +/- drive * 0.2, 0.5, 2.0) applied to the
  NEXT beat's per-channel Poisson rate (MOTOR_GAIN = 0.2);
  organism-side consequence-novel = known-symbol beat whose reflex
  response (min-L2 vs all templates) exceeds th_fam, judged BEFORE the
  beat's template overwrite;
  harness ground-truth (instrumentation only): per-symbol EMA (a=0.2)
  expected drive, violation = |drive - exp| > 3*sigma (sigma = EMA of
  |residual|), judged from the 3rd sample of each symbol;
  fault injection (verification only): D59_FAULT=1 reverses the effect
  (eff = 1.0 - drive*0.2) for beats >= 60.
- Gate env: D59_REFLEX=1 (in-life reflex), D59_MOTOR=1 (motor world),
  D59_FAULT=1 (fault). Flag-off (unset) = d58_reflex 0 = byte-identical
  baseline (verified: identity gate below).

## Gate (pre-registered)

- D-59 gate run: `D59_REFLEX=1 D50_MODE=1 ./target/release/evolve`
  (3 seeds x 8 gens). ACCEPT if mean `reflex_novel_detected_frac` >= 0.8
  through the generations. Else: FINDING (does detection survive growth?
  the D-56 question), no re-tuning.
- Motor discriminating test: same seed twice (EVO_BEATS=100 so the
  beat-60 fault engages), with/without D59_FAULT=1. Organism-side
  consequence knowledge SHOWN iff faulted `cons=` (mean
  consequence_novel_frac) > clean `cons=`, cross-checked by harness
  `wv=` (world_consequence_violation_frac) rising too. If cons= does not
  rise while wv= does: honest verdict = consequence knowledge NOT SHOWN
  in this regime (negative results are the project's norm).

## Implementation (commits)

- reflex.rs: shared module (REFLEX_K, REFLEX_TH_FAM, l2, signature,
  novelty_verdict) - promoted from library.rs prototype; library.rs
  now calls the shared module (D58_REFLEX prototype behavior unchanged).
- io.rs: MotorCommand { rates_hz } + motor() (linear proportional
  spike-count -> Hz mapping); symbol_trains_mode_eff() (rate-scaled
  trains; eff=1.0 byte-identical).
- survival.rs: run_world_full(net, ..., mode, reflex_k, motor_mode);
  per-beat reflex-band counting in the existing pass; per-symbol
  templates; D-beat verdict (reflex_novel_detected_frac); motor-mode
  consequence verdict (consequence_novel_frac + telemetry) and harness
  ground-truth (world_consequence_violation_frac).
- evolve.rs: D59_REFLEX/D59_MOTOR env gates; per-gen summary appends
  `reflex= cons= wv=` columns ONLY on D-59-active runs (flag-off
  byte-identical line); birthprobe.run_world_v2 callers pass (0,false).
- encoders.rs: SensoryEncoder trait + PatternEncoder passthrough
  (interface only; retina/cochlea named NOT E9/E10 - collision with
  curriculum experiments).

## Verification (as run)

1. Identity gate (flag-off): exact committed baseline.
   `gen 0: fit=[55:0.64 55:0.49 55:0.46 55:0.46] mean_sz=55.0 mean_fit=0.511 best=0.642`
   `gen 1: fit=[70:0.66 70:0.66 70:0.44 70:0.36] mean_sz=70.0 mean_fit=0.532 best=0.663`
   PASS (byte-identical, pre- and post-change).
2. Library prototype (shared module): stages 1-3 `reflexDet=3/3
   known_fp=0` at D58_K=8. PASS.
3. Motor telemetry: `MOTOR beat= drive= eff= cons_novel= world_viol=
   cons_frac= world_frac=`; eff = 1 + drive*0.2 confirmed (beat0 drive
   0.129 -> beat1 eff 1.026; faulted mirror eff 0.982 vs clean 1.018 at
   beat 90). PASS (machinery).
4. Motor discriminating test (EVO_BEATS=100, seed 424242, 1 gen):
   clean `reflex=0.50 cons=0.68 wv=0.06` vs faulted `reflex=0.54
   cons=0.67 wv=0.06`. cons= does NOT rise; wv= does not rise either.
   FINDING: organism-side consequence knowledge NOT SHOWN in this
   regime; the harness tracker also cannot see the fault. Mechanism:
   drive*0.2 <= 0.2 (drive<=1) -> the eff clamp [0.5,2.0] never binds
   and the sensed rate change is +-2..18%; both the reflex band and the
   drive EMA move within their normal noise band. Pre-registered
   constants kept; NO re-fitting.
5. Unit suite: `cargo test --release` -> 186 passed, 0 failed. PASS.

## D-59 gate run (3 seeds x 8 gens, D59_REFLEX=1 D50_MODE=1)

(pre-registered acceptance: mean reflex_novel_detected_frac >= 0.8
through generations; results table filled below after the run)

| seed | gen0 | gen1 | gen2 | gen3 | gen4 | gen5 | gen6 | gen7 | verdict |
|------|------|------|------|------|------|------|------|------|---------|
| 20260912 | 0.73 | 0.32 | 0.25 | 0.19 | 0.25 | 0.44 | 0.19 | 0.00 | FAIL |
| 9001 | 0.74 | 0.43 | 0.58 | 0.14 | 0.61 | 0.33 | 0.00 | 0.00 | FAIL |
| 424242 | 0.49 | 0.25 | 0.18 | 0.26 | 0.18 | 0.36 | 0.00 | 0.00 | FAIL |

RUN 2026-09-24: `D59_REFLEX=1 D50_MODE=1 ./target/release/evolve`
(3 seeds x 8 gens, 564 s). After the quick probe's gen-0 numbers
(424242: 0.49) which matched the full run (0.49). All seeds: mean
reflex_novel_detected_frac peaks at gen 0 (0.49-0.74) and decays to
0.00 as selection-driven growth pushes populations to the ~250-neuron
viability wall (gen 6-7: all organisms dead -> no D beats -> 0.00).
GATE FAILS: never >= 0.8 at any generation.

## Diagnosis (D59_DEBUG instrumentation; NO tuning)

Why both signals fail — measured, not inferred:

- Reflex band noise floor: known beats sit at min-L2 61-370 from their
  OWN just-updated templates (beat-to-beat), with occasional exact 0.0;
  D beats span 0.0-187, INCLUDING exact template matches (0.0).
  Distributions fully overlap: th=60 sits inside the ambient noise band.
  Mechanism: the reflex nodes' own slow-state (tau 5000 ms > 1500 ms
  inter-beat gap) plus adaptation carry across beats, so the IDENTICAL
  input train (fixed world seed) produces 0-370 L2 responses depending
  on history. The D-58 prototype avoided this with immediate
  state-matched capture; in-life per-beat templates cannot outrun it.
- Motor consequence magnitude: observed drives 0.09-0.30 -> eff deltas
  +/-0.018-0.060 (+/-2-6% afferent rate); the [0.5, 2.0] clamp never
  binds. On known beats, frac(minL2 > 60) = 0.78 CLEAN == 0.78 FAULT
  (EVO_BEATS=40, seed 424242, 1 gen): the consequence check fires 78%
  of the time in BOTH worlds; the fault shifts nothing outside noise.
- Conclusion: with the registered constants (K=8, th=60, 1500 ms gap,
  no slow-state reset), the in-life reflex signal does not exist; the
  D-59 gate and the motor discriminating test both measured this noise
  — hence FAIL / NOT SHOWN. Constants kept; next move is a new
  pre-registration (user-gated, repo rule: no post-hoc tuning).

Candidate directions for the NEXT registration (evidence-tagged, user
decides): (a) slow-state/inter-beak reset parity (rest gap >= 5x slow
tau or slow-state clamping) to restore the prototype's state-matched
margin; (b) drive-normalized reflex readout (per-beat magnitude
normalization, since L2 is magnitude-aware and magnitude is
context-dominated); (c) motor gain regime where the consequence exceeds
the noise floor (drive-dependent eff amplitude or eff clamp ["0.5,2.0"]
actually binding). Each needs its own frozen protocol doc first.

## Findings

- D-59a reflex in-life: wiring VERIFIED (fields live, per-gen columns,
  gen-0 detection 0.49-0.74 mean with individual orgs at 0.86-1.0), but
  the gate FAILS: in-life detection is below the prototype's 3/3 and
  decays under growth (D-56 answer: growth does NOT preserve reflex
  novelty detection in this regime). No re-tuning (pre-registered).
- D-59b motor world: machinery VERIFIED (eff = 1 +/- drive*0.2 applied
  to next beat's rate; telemetry; fault reversal engages at beats >= 60:
  eff 1.018 -> 0.982 mirror). Organism-side consequence-novelty NOT
  discriminative: clean cons=0.68 vs faulted cons=0.67 (EVO_BEATS=100,
  seed 424242, 1 gen); harness ground-truth wv=0.06 both. The fault's
  sensed change (+-2..18% afferent rate, clamp [0.5,2.0] never binds at
  drive<=1) stays inside both signals' noise band. Negative finding
  recorded; constants NOT re-fitted (pre-registered).
- D-59c encoder interface: foundation in place; retina/cochlea
  registrations need their own frozen protocol docs first (repo
  convention); names retina/cochlea, NOT E9/E10.