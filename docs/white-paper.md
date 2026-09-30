# ANIMA: A Developmental Approach to a Self-Organizing Forager
## Experimental Findings, Registered Negative Results, and the Two Walls

**Status:** White paper · Research record · September 2026
**System:** `world_survival` (crates/anima-world), ANIMA developmental artificial nervous system in Rust
**Reproducibility:** deterministic (seeded), byte-identical defaults, all mechanisms env-gated

---

## 1. Executive Summary

ANIMA is a from-scratch, self-organizing artificial nervous system: a spiking LIF network receives
retinal/proprioceptive frames from a physical 3D body in an arena and controls it to forage for food.
Under a binding charter it uses **no hand-designed internal architecture, no pretrained models, no
external memory modules**, and the only reward is food.

This paper records a full characterization campaign. We found and fixed a real control bug, built a
working forager, gave it a second drive (a home/rest site), and confirmed it can produce
**repeated return-home behavior when that is rewarded**. But we also isolated — with measured
evidence, not inference — **two walls** that block the original open-ended goal:

1. **Internal-band neural saturation**: the network's internal (expansion) layer fires at 100% every
   tick (`out_sum 6000`, `kc_spikers 256/256`). Every reward-gated learning mechanism therefore
   becomes a non-selective global wash that the evolutionary loop ignores.
2. **Fitness-economy ceiling**: evolution pins at ~121–137.6 fitness regardless of mechanism stack.

A four-lever campaign (competition, spike threshold, input amplitude, per-neuron heterogeneity)
failed to de-saturate the band — and revealed it is **temporally bistable**: cold→sparse, then a
sharp phase-transition to hot→saturated. Selective learning requires sustained sparsity and is
therefore structurally absent. Tracked next levers are listed in §9.

---

## 2. Motivation

Goal: an organism that autonomously forages — finds food, consumes it — and, ultimately,
self-improves open-endedly ("make it like us"). The project favors **laboratory discipline over
best-effort results**: every mechanism is registered, falsifiable, and reproducible.

## 3. The Agent and the World

- **Body**: 6-DOF physics (yaw/pitch/thrust/strafe/lift/brake), arena 100×100, gravity 9.81 applied
  in the simulator only, ground clamp, energy tank (starts 100, drain 1.0/beat, death at 0).
- **Sensory**: 48-channel retina (5 primitives, analytic ray raster) + 8 proprioceptive channels,
  plus optional 6-channel food-smell (body-relative gradient).
- **Network**: LIF spiking, 1 ms ticks, seeded `Xoshiro256PlusPlus`, STDP + structural growth (V2),
  24-56 inputs / 40-(128)-(256) internal / 12 motor outputs.
- **Reward**: food only. Steering is the action, never rewarded directly.
- **World dynamics**: food placement (static seed / curriculum / escalating / stable), energy refill
  on contact, death at tank zero.

## 4. Binding Constraints (the Charter)

- No hand-designed internal architecture; no pretrained/CNN/CLIP; no external memory modules.
- All mechanisms **env-gated**; with every gate unset, output is byte-identical to a frozen baseline.
- Determinism: same args + env ⇒ identical output (registered).
- **Pre-registration**: expected direction and success bars are set before running; negatives are
  recorded, constants are not tuned on outcomes.
- Charter amendments (e.g. adding a home need) require explicit operator approval.

## 5. Method

**Protocol.** Each mechanism is introduced env-gated, then verified: (1) default identity,
(2) determinism (two-run diff), (3) observability (gate on vs off), (4) workspace suite, (5) a
registered success bar. Failures are recorded as evidence-backed negatives — never silently
retuned. Results below are measured; unverified causal claims are flagged as such.

## 6. Measured Results

### 6.1 Verified wins

| # | Mechanism | Result | Evidence |
|---|---|---|---|
| 1 | **Control bug fix** | Steering yaw channel was sign-inverted; body flew *away* from food | post-fix food_dist 11.5→6.7, touches 0→9 at reachable radius |
| 2 | **Foraging under shaping** | Up to 126 touches / life; strong local pursuit | curriculum + smell stack |
| 3 | `SL2_STASIS` (stable food + re-entry guard + tank cap) | 3 → **36 genuine touches** / life; energy bounded; no farm | unsaturable only via geometry-exact re-entry guard |
| 4 | `SL2_HMEXP` (hunger-modulated search) | settle-when-fed (0.4× noise floor), roam-when-hungry; early-survivor signal in evolve | single-life ≈ baseline; evolve best 0→33.9 by gen 5 (8-gen window) |
| 5 | `SL2_NEST` + `SL2_NESTFIT` (home/rest second drive + homing reward) | **First repeated return-home behavior** (elite_ret 1→3 on seed 7) | home = nearest non-food landmark; stamina = capability constraint; reward per return |
| 6 | Progressive survival (`SL2_PROG`) | survived 2230 beats (vs ~60–200), food escalates until defeated | big lifespan + escalating difficulty |

### 6.2 Registered negative results

| Mechanism | Outcome | Notes |
|---|---|---|
| `SL2_NOVELTY` (MCC archive selection) | observably steers selection; **ceiling unchanged** | rank-order changes, no escape |
| `SL2_RSTDPP` (dopamine × STDP gain) | fires correctly (4.9× Δw in window) but **invisible to evolution** | ~1e-3/synapse below mutation floor |
| `SL2_ARS` / `SL2_DENSITY` | fitness-observable; **ceiling unchanged** | same survivor, bigger number |
| `SL2_MB` (mushroom-body locus) | wiring verified firing; **no sparse premise** | 256/256 KCs eligible — non-selective |
| De-saturation campaign (§7) | all four levers bounded negative | competition / threshold / amplitude / heterogeneity |

**Cross-cutting finding:** within-life plasticity (any locus) is ~1e-3–0.24/synapse but the GA's
selection is **death-gated and coarse** (±10% mutation) — so learning that is not selectivity-visible
never changes outcomes. Evolution "listens" only to selection-level changes (novelty rank-order,
the homing reward NESTFIT — the one lever that moved behavior).

## 7. The Two Walls

### 7.1 Wall 1 — Internal-band saturation ⇒ selective learning is impossible

The internal/expansion layer fires at 100% every tick. Under this regime any reward-gated
plasticity is a **uniform global wash** the GA trivially mutates around (measured: evolve
trajectories byte-identical with and without the reward bump).

**De-saturation campaign (all measured):**

| Lever | Range | Result | Mechanism |
|---|---|---|---|
| Pairwise competition (`SL2_IINH`/`SL2_INH`) | 0.5–4.0 | 256/256 always | Poisson drive rebuilds the one-tick inhibitory dip |
| Spike threshold (`SL2_THETA`) | 2–8× | 256/256 always | 52-amplitude input drive crosses any spread |
| Input amplitude (`SL2_AMPL`) | 52→3 | 100%→0% **cliff** | uniform scaling is bistable — no graded regime |
| Heterogeneity (`SL2_HET`) | 0.2–1.0 | 256/256 alone; transient at corner | lognormal threshold spread can't sustain |

**Final mechanism (discriminator, KCPROBE_EVERY=10):** the band is **temporally bistable** —
cold at life start (2/256), then a **sharp phase-transition to saturated** by beat ~25–30. It is a
collective recurrent-excitation/hysteresis flip, not a gradual slow-state ramp (a linear rate-constant
knob likely won't tame it).

### 7.2 Wall 2 — Fitness-economy ceiling

Evolution converges to a scoring-family-dependent pin (not one flat number):
- **Plain-survival scoring** (`final_energy + 3·touches`): pins 121.8 / 121.0 (HMEXP+NEST, seeds 1/2).
- **Density/ARS scoring** (adds `20·touches²/beats` and `4·persistence`): pins 137.5–137.6
  (ARS run seed 7; the higher number is the bonus terms, not a real escape).
- **NESTFIT scoring** (plain + `40·nest_returns`): 121.8 / 179.4 (seed 1 / seed 7), where 179.4
  includes `+40×3` from three returns — again additive, not a genuine survival gain.

`final_energy + 3·touches` saturates once food is reachable; across every scoring family the
ceiling is the fitness-economy saturation, and the ~150–180 bests are additive bonuses.

## 8. Discussion

- The organism is **real and capable**: it forages strongly, survives long, and can sustain a
  home-return behavior *when rewarded* — the first causal win for the "second drive" direction.
- The walls are **structural, not tuning**: saturation is immune to competition/threshold/amplitude/
  heterogeneity at every tested magnitude, and the fitness economy is a property of the reward shape.
- The iron rule of the campaign: **within-life learning cannot clear a coarse, death-gated
  selection**; only selection-level changes are evolution-visible.
- **Full-stack non-additivity**: running the coherent verified organism
  (`STASIS`+`HMEXP`+`NEST`+`NESTFIT`+`OINH`+`pool128`) reproduces the NESTFIT baseline exactly at
  15 gens (seed 20260924: 121.8/ret=0; seed 7: 155.0/ret=1) — the mechanisms are orthogonal to the
  fitness economy, not additive.

### 8.5 Limitations

The claims above are bounded by the measurement window and should be read with these limits:
- **Small N seeds**: only two evolve seeds per run; seed-dependence (e.g. `elite_ret` 0 vs 3) is a
  real finding but a 2-seed sample cannot establish robustness.
- **Short generation windows**: several series (MB, full stack) ran 12–15 gens per seed within a
  3500 s tool cap; pin values at longer horizons (e.g. 179.4/ret=3 at gen 20–30) come from a subset
  of runs.
- **Single-threaded CPU stack**: 256-pool evolution exceeded the hour cap; some negatives
  (MB locus, larger expansions) were bounded by compute, not measured to conclusion.
- **Partial windows**: a few registered negatives rest on overlapping-generation comparisons
  (e.g. gens 0–5) rather than full-architecture sweeps.
- No GPU, no parallelization; determinism is enforced serially.

## 9. Recommended Next Steps (registered levers)

1. **Attack the phase-transition, or keep the band cold**: target the recurrent-excitation/hysteresis
   that flips the band to saturation, or maintain continuous-cold sparsity — the precondition for any
   selective-learning re-test (`SL2_MB` locus remains unproven, not falsified).
2. **Reform the fitness economy**: sustained-foraging / survival density shaping so selection has a
   gradient to climb past the ceiling.
3. **GPU/parallel step** (engineering): the fly-connectome literature runs 138k neurons real-time on
   GPU; this single-thread stack can't afford a biological-size expansion in evolution. Removing the
   compute tax unblocks every experiment above.

## 10. Reproducibility

- Deterministic by construction (seeded LCG-free Xoshiro; no HashMap iteration order).
- Frozen default identity: `world_survival 20260924 77 60` ⇒
  `WL2 RESULT beats=60 died_at=none final_energy=40.0 food_touches=0 novel_flags=0`.
- Test rig: `KCPROBE`/`SW_PROBE`/`MB_PROBE`/`NEST_PROBE` (env-gated, stderr) + registered
  `SL2_*`/`WL2_*` knobs; workspace suite 198/0.
- Workflow discipline captured in the `anima-experiment-protocol` managed skill.

## Appendix A — Reproduction specification

Everything needed to recreate the experiments, with exact equations and registered constants.
All mechanisms are **env-gated**; with no env set, the default run and its frozen output are
byte-identical to the recorded baseline.

### A.1 World & body dynamics

- Start energy **100.0**; `SL2_PROG` overrides to **PROG_LIFE 4000.0** (life-span mode).
- Per beat: `energy -= 1.0` (plus `explore` metabolic cost `METABOLIC_SPEED × |vel|`); on food
  contact within **6.0** units, `energy += 30.0`; death exactly at `energy == 0`.
- `SL2_STASIS` (stable food): food does **not** teleport on touch. Re-entry guard — a touch counts
  only if the body left the **6.0** eat radius since the last counted touch; tank caps at the
  life-start energy (`min(energy, tank_cap)`; `tank_cap` re-read after any PROG override).

### A.2 Network construction (`build_net`)

Defaults: `connectivity 0.038, w_init 0.2, amplitude 52.0, adaptation_tau_ms 200.0,
adaptation_gain 0.05, inhibition_gain 0.0, slow_state_tau_ms 5000.0, slow_state_beta 0.0046875,
latch_enable true, theta_rel_mean 1.0, theta_rel_sd 0, u_plateau_rel_sd 0, tau_het_rel_sd 0,
phi_rel 0.5, eta_rel 0.0, output_inhibition_gain 0.0, d58_reflex K` (K = the 8-neuron reflex/familiarity band that follows
the outputs; see `const K: usize = 8`). Env overrides:
`WL2_OINH` → `output_inhibition_gain 0.15`; `SL2_POOL` → internal pool size (default 40);
`SL2_MB` → pool **256**, `connectivity 0.10`; `SL2_IINH` → `internal_inhibition_gain`;
`SL2_THETA` → `theta_rel_mean`; `SL2_AMPL` → `amplitude`; `SL2_HET` → `theta/u_plateau/tau_het sd`
(per-neuron lognormal, applied after wiring). Wiring is seeded-via-`Xoshiro` with no `HashMap`
iteration order, so identical args+env ⇒ identical network.

### A.3 Fitness per scoring family (evolution `evolve_run`)

D-38 death gate: a dead organism scores **0** unconditionally. Alive:

- **Plain-survival**: `fit = final_energy + 3·touches`  (TOUCH_BONUS = 3.0)
- **Density** (`SL2_DENSITY`): `+ 20·touches² / beats`
- **ARS** (`SL2_ARS`, subsumes density): `+ 4·persistence`, where
  `persistence = ΣT exp(− gap_k / 20)` over consecutive touches (beats since previous touch)
- **NESTFIT** (`SL2_NESTFIT`, adds to any alive base): `+ 40 · nest_returns`
  (`nest_returns` = count of false→true re-entries into the nest radius — see A.5)

### A.4 Behavioral drives (env-gated)

- **Hunger**: `hunger = 1 − energy / start_energy`.
- **`SL2_HMEXP`** (hunger-modulated exploration): motor-noise gain
  `noise_scale = 0.4 + 0.6·hunger` (fed ⇒ 0.4× baseline noise, hungry ⇒ 1×). Applied to all three
  explore-noise sites.
- **`SL2_RSTDPP`** (three-factor): on food touch set `dopamine = 1.0`; per beat `dopamine ×= 0.6`;
  STDP gain `= min(1 + 3·dopamine, 4)`.
- **`SL2_NOVELTY`** (MCC selection): behavior characterization `bc = [x_bin, z_bin, alive]` where
  `x_bin = clamp((px+50)/10, 0..9)`, same for z; novelty of an agent =
  `Σ_archive (diff(a,bc))/3 / max(|archive|,1)`; selection rank key `(alive, novelty, fit)`
  descending; archive cap **50** (evict oldest).

### A.5 Home / rest (`SL2_NEST` + `SL2_NESTFIT`)

- Nest = the non-food scene primitive nearest the spawn (deterministic).
- **Stamina** (capability constraint): starts 100; away from the nest,
  `stamina −= 0.4 × |vel|` per beat; within `NEST_RADIUS = 8.0` of the nest,
  `stamina += 60` (cap 100). Motor rates scale by `min(1, stamina / 25)` — exhaustion immobilizes.
- `nest_returns` increments on each false→true entry into the 8.0 nest radius.

### A.6 Mushroom-body locus (`SL2_MB`)

Sparse expansion (256 KC, ~6 inputs/KC) + reward-gated input plasticity at the KC band: per touch,
for every KC with eligibility `e > 0`, its input synapses get `Δw = 0.03 · e · 8` (clamp 1.0).
Measured saturation (A.7) currently makes `e > 0` for all 256 KCs, so the bump is non-selective.

### A.7 De-saturation measurements (the four-lever campaign)

`KCPROBE` (env-gated) reports distinct internal-band spikers per sampled beat (`KCPROBE_EVERY`
sets the beat interval). Results: pairwise competition 0.5–4.0 ⇒ 256/256; threshold 2–8× ⇒
256/256; amplitude 52/20/8 ⇒ 256/256, amplitude 3 ⇒ 0/256 (a cliff); heterogeneity 0.2–1.0 ⇒
256/256 alone, and at `amplitude=4, het=1.0` the band is temporally bistable: beat 0 = 2/256,
beat 20 = 3/256, beat 30+ = 256/256 (a sharp phase transition).

### A.8 Verification battery and frozen identity

1. Build: `cargo build --release -p anima-world --bin world_survival`.
2. Default identity (no env): `./target/release/world_survival 20260924 77 60` must print exactly
   `WL2 RESULT beats=60 died_at=none final_energy=40.0 food_touches=0 novel_flags=0`.
3. Determinism: run any gated config twice, `diff` empty.
4. Suite: `cargo test --workspace` → 198 passed / 0 failed.
5. Probe output is on **stderr** — capture with `2>&1 >/dev/null | grep`, not `2>/dev/null`.

### A.9 Provenance notes

The **36-touch** (STASIS) and **2230-beat** (`SL2_PROG`) figures are recorded-lesson values
(session-learn log); logs for those long runs were overwritten by later runs, so they carry
recorded provenance rather than a surviving artifact file. Every other cited number in this paper
was verified directly against a surviving run log.

### A.10 Genetic algorithm structure (`evolve_run`)

- Population **N = 4** per generation; seeded `net = build_net(net_inputs(smell), derive_seed64(esec,0,i))`.
- Per-generation curriculum ramp: `gen_start = min(8 + 3·g, 40)`.
- Selection: deterministic sort by fitness (or `(alive, novelty, fit)` under `SL2_NOVELTY`);
  death gate (dead ⇒ fit 0) holds.
- **Elite carry**: `order[0]` carried unchanged (weights copied via `breed_inner`, `mutate=false`);
  after the same seed is best for **ELITE_STALL_GENS = 5**, the elite is carried **with** mutation
  (Fix B, anti-stall).
- Offspring: best → 2, second → 1; `breed_inner(parent, derive_seed64(esec,g+1,slot), mutate=true)`
  copies parent weights by `(pre,post)` and mutates each with **p = 0.10, amplitude 0.10**
  (`W_MUT_P` / `W_MUT_AMP`), then re-adds parent-born V2 synapses while under the growth cap.
- `SL2_NOTARGET` suppresses the early TARGET break (`best > 40`); `--generalize` is champion mode
  (probes the elite per gen on `BEATS_LONG` natural food, stops at `ULTIMATE_BAR`).
- Beats per life: `BEATS_EVOLVE = 100` (evolve+explore), `BEATS_LONG` for champion probes.

### A.11 Env-gate → exact-run-command table

Core (documentation only, not all commands run daily):
- Default / identity gate: `./target/release/world_survival 20260924 77 60`
- Stability + settle + home reward (full stack):
  `SL2_STASIS=1 SL2_HMEXP=1 SL2_NEST=1 SL2_NESTFIT=1 WL2_OINH=1 SL2_POOL=128 SL2_NOTARGET=1 ./target/release/world_survival --evolve --explore --smell --until 15`
- Per-mechanism observability runs use the same binary with a single gate added; probes
  (`KCPROBE`, `MB_PROBE`, `SW_PROBE`, `NEST_PROBE`) are env-gated and print to **stderr** —
  capture `2>&1 >/dev/null | grep <probe>`.
- Seeds used: `20260924` and `7` (both `EVOLVE_SEEDS`).

## 11. Selected References

- FlyWire adult Drosophila connectome and its simulation in embodied bodies (NeuroMechFly, Nat.
  Methods 2024; whole-brain embodied fly sims; Fly-connectomic graph models) — evidence that small
  spiking networks drive 3D behavior and that large-scale sims run real-time on GPU.
- Mushroom-body associative learning (sparse random expansion + dopamine-gated input plasticity;
  trace conditioning; biased PN–KC connectivity) — the biological scheme ANIMA's `SL2_MB` was drawn
  from, whose sparse-coding premise the saturation wall currently denies.
- Novelty search / minimal-criterion coevolution (Lehman & Stanley; MCC) — the selection-level
  change that observably steers but does not escape the ceiling.

---

*All results in this paper are reproducible from the repository; every claim is grounded in a
measured run recorded in the session-learn log. Unverified causal attributions are explicitly
marked.*