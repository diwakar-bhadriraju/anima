# ANIMA — Program Overview & Current State

Status: living document, updated 2026-09-20. Companion to the
formal records in this repo (`docs/*-protocol.md`, `docs/*-spec.md`,
`docs/unknowns-registry.md`, `docs/xplor*.md`). This file is a
navigational summary, NOT a protocol; no result claimed here
overrides a frozen record.

---

## 1. What we are trying to achieve

**Goal (charter).** Build an *experimental developmental
artificial nervous system*: we fix only the boundaries —
sensory input, output, environment, curriculum, resource limits,
and fundamental local plasticity laws — and the organism must
develop **useful, stable, adaptive internal organization**
through interaction with its environment, **without prescribing
its final organization**.

**Binding charter rules** (from the plan + decision log):
- Never hand-design internal architecture; define only I/O,
  environment, curriculum, plasticity laws.
- Major mechanisms: evidence-tagged candidates (GREEN/YELLOW/
  RED) with predictions + smallest distinguishing experiment,
  user approval BEFORE implementation.
- Local mechanisms only; no task-specific semantic variables;
  no labels/reward/credit (unless justified as an architectural
  hypothesis); no hidden supervision.
- Determinism: single seeded Xoshiro; same seed ⇒ byte-identical
  telemetry. Reproducibility and provenance are invariants.
- Input neurons are pure spike sources (D8); memory is internal
  neural structure (never an external database); organism output
  is separate from the human-readable decoder/instrumentation.
- Pre-registration discipline: protocols frozen (committed)
  before implementation; amendments user-approved and logged;
  negative results retained.
- Curriculum is configurable (patterns are examples, not
  preinstalled concepts).

**Operational measure of progress.** The goal decomposes into a
claim ladder, each rung requiring the previous:
1. substrate works (spiking, local plasticity);
2. internal organization forms;
3. organization is *retained* over time (memory);
4. retained organization *carries information* (distinguishable
   states from distinguishable experiences);
5. the organism *uses* that information in behavior (closed
   loop);
6. organization stays stable and adaptive under continued
   interaction.

Current location: we have walked rungs 1–2 repeatedly; rung 3
(what the substrate erases, and why) is where the past three
experimental eras have concentrated; rung 4 is the open frontier.

---

## 2. The architecture (current build)

### 2.1 Substrate (V2, frozen)
- LIF spiking neurons, 1 ms ticks, refractory 2 ms.
- 24 input channels / 40 internal / 12 output (small scales are
  deliberate; E8 scale studies).
- M1 dense-weak initialization (input p=0.5, recurrent p=0.2).
- **M2** normalization: per-neuron excitatory budget t_e=0.8,
  multiplicative rescale of live incoming weights.
- **M3/M4** candidate-silent synapses + pruning; **M5** slot
  budgets (b_e=40 exc); **M6** anti-Hebbian inhibition.
- **STDP** pairwise additive (a⁺=0.005, a⁻=0.0053, traces
  τ=20 ms), silence-pruning, passive weight decay 1e-6/tick.
- **E6** per-channel rate balance β (history-derived, no labels).
- Structural growth machinery exists (birth triggers: none /
  HomeostaticSaturation / PersistentError; wiring options) but
  EVERY experiment in this era ran `birth_trigger = "none"`.

### 2.2 Slow intrinsic state (V2.1, frozen)
- Per-neuron u: +β per spike, decays τ_s; fed into dv as +u
  (depolarizing; the missing degree of freedom after V2: every
  previous state was hyperpolarizing or read-only).
- Config: `slow_state_beta` (β), `slow_state_tau_ms` (τ_s).
  β=0 ⇒ byte-identical V2.
- Stable wedge (persistent endogenous activity, 0 failures):
  (β=0.00625, τ=5 s) and (β=0.003125, τ=10 s).

### 2.3 Bistable latch (V2.2, FROZEN NEGATIVE — do not modify)
- Per-neuron z bit + plateau: SET at u ≥ θ_i, RESET below φ_i;
  plateau ADDS to u; per-spike η subtraction (Y1).
- Stage 1 identity PASS; Stage 2 (120 runs, A-1 amendment)
  **NOT A SUCCESSFUL CARRIER**: latch-sets decode curriculum at
  chance in all arms; η_rel=1.0 self-cancels (latches nothing).
- Verdict frozen; negative retained; falsifiers F1/F3-antecedent/
  F5 confirmed.

### 2.4 Trace-partitioned M2 (V2.3 — implement/executed, PARTIAL SUPPORT)
- Per-synapse write-epoch bucket (m2_bucket 0/1) tagged at LTP
  time from the E1 window-clock parity (epoch = 40 windows =
  4 s); M2 normalizes per bucket with capacity-matched target
  T = t_e / n_populated (total ≤ t_e ALWAYS; never boosts).
- m2_buckets=1 ⇒ exact baseline path (identity gate PASSED).
- A3 vs A2: partition rescues collapsed-cohort cells (bca 6/6,
  incl. 0.00→0.92 recovery); bac masked by anchor baseline;
  E16 separation preserved; 0/12 aborts. PARTIAL SUPPORT.

### 2.5 Determinism/provenance
- All runs: `runs/<exp_id>-<UTC>Z/` with `telemetry/` chunks +
  `snapshots.bin.zst` + `metrics.json` + `report.md`; runs/ is
  gitignored (disk-only), records reference them; an integrity
  regression test (`execution_record_integrity`) fails on
  references to nonexistent runs.
- Instruments (committed examples): cross_cosine, read_rates,
  v21phase, v21contrast/contra2, v21dump, v21bmap, e24_endpoint,
  v22wdiv, v22during, v22blk, v22s2an, v22lat, v22ident.

---

## 3. The journey (era by era, with the key verdicts)

### E1–E11 (substrate + representation era): the organism works
- E1–E5: STDP-driven assemblies form, novelty response, U1
  adaptation; probe births.
- E6: rate balancing; E7, E8: scale/cross-seed; E9–E11:
  vision/audio encoders specified (retina/cochlea; no pretrained
  models).
- E3b: **the binding problem is representational, not
  dynamic** — lateral inhibition didn't improve separation
  (cross-cosines stuck ~0.74, selectivity ~0.37).

### E12–E17 (path-dependence era): structure is passive
- REV re-anchoring is passive re-expression (E12–E14).
- E15/E16: M2 is REQUIRED for the separation regime; STDP and
  M3/M4 individually non-necessary (E16/E17). The organization
  is carried by the weight structure, not by the timing rules
  textbook STDP provides.

### E18–E21 (capability failures era): temporal state does not exist
- E18: no antecedent-dependent divergence over 800 ms.
- E19/E20: outputs are stimulus-locked followers; **zero
  endogenous output activity**; closed loop can't even start.
- E21: temporal capacity < 50 ms (only gap 0 retains;
  D_L−NF = 0.0815).

### E23: consequence shaping exists but INVERTS
- First closed-loop demonstration: closed reflex arms fully
  antecedent-conditioned — but the mapping is INVERTED
  (punishment-channel sensory redistribution + M2 amplification;
  boundary STDP real but not sufficient). Valence-free
  substrate, no reward — the shaping is channel statistics.

### V2.1 (slow state era): persistence without information
- Stage A: β=0 identity; Stage B: stable wedge, endogenous
  activity sustained.
- Stage C: NEVER EXECUTED; a phantom v21c record was committed
  by mistake, discovered in the info audit, **retracted** (the
  record said capacity unchanged but no runs existed). Integrity
  regression added; the frozen selection rule remains available.
- Info audit: endogenous activity = deterministic pacemaking
  (Fano≈0, refractory ceiling, small cores, Gini hardening) —
  persistence WITHOUT representational structure; no A/C
  contrast had ever been run.

### E24 (regime-selection capacity, formal): NOT SUPPORTED
- 6 seeds × 5 curricula at the band; primary endpoint
  M = log10(1+S10) with abort=+∞. All contrasts p=1.000;
  directions inconsistent across seeds. Winner identity shows a
  primacy structure (first block's recruit dominates), but
  magnitude is seed-dominated. Negative result retained.

### X1–X3 + architectural review: where the missing capability is
- X1: transition band at β≈0.0047 — an intermediate regime
  (dynamic range, decaying tail, effective τ 8.2 s ≠ τ_u).
- X3: 45-run boundary map — top-u at drive end is a near-perfect
  regime predictor (ρ≈0.97); drive-composition shifts regime but
  seed dominates.
- Review verdict: **writable/competitive memory** is the missing
  capability; sensory organization EXISTS at the afferent layer
  (pure drives: 42–48/52 neurons dominant for that curriculum;
  interleaved keeps separable cohorts); during-stimulus
  responses separate (cos 0.30–0.89) but off-window do not
  (0.999+); persistent state is a seed-lottery twice over.

### SDE-B/C2/D (causal probes, config-only): the erase mechanism
- SDE-B: doubling t_e (capacity) does NOT restore coexistence —
  the surplus feeds the dominant trace. Capacity is not the
  lever.
- SDE-C2: disabling passive decay (1e-6 → 0) largely preserves
  the first block's cohort through the second block (6/6 bac) —
  **passive decay is the dominant eraser**; residual skew
  (≈0.07–0.09) remains.
- SDE-D: the STDP a⁻>a⁺ asymmetry is secondary/context-
  dependent (helps with decay on, reverses with decay off);
  symmetric STDP destabilizes (aborts). M2 reallocation = the
  leading UNRESOLVED residual mechanism (inference, not yet
  causal isolation).

### V2.2 (design → executed): the latch does not carry
(as §2.3; the honest negative that refocused the program).

### V2.3 (design → executed, current): first causal hit on M2
Exactly as §2.4 — the partition intervention (capacity-matched)
rescues the collapsed-cohort class. First mechanism-level
intervention that moved the coexistence metric in the predicted
direction.

---

## 4. How far are we — and what is left

To the goal (useful/stable/adaptive organization, self-chosen):

| Capability | Status | Where |
|---|---|---|
| Spiking substrate + local rules work deterministically | DONE | E1–E11 |
| Sensory-specific internal organization forms | DONE (afferent level) | E1–E15, v22wdiv |
| Endogenous persistent activity | DONE | V2.1 Stage B |
| Stability of the operating point | NARROW but principled | X3 map |
| RETENTION of organization (write/maintain) | PARTIAL — decay is the eraser; partition rescues collapse class | SDE-C2, V2.3 |
| Information in retained state (memories distinguishable) | NOT ESTABLISHED — no reliable carrier yet | E24, V2.2 |
| Closed-loop use of retained state | NOT TESTED (precondition = carrier) | E19/E23 showed the loop exists, inverted, without one |
| Long-horizon adaptive re-organization | NOT TESTED (needs retention) | U3 growth era, open |

**Open frontiers, in dependency order:**
1. Make retention reliable *and* curriculum-formative (the
   current arena: M2 partition + decay handling; then re-test
   curriculum decoding on retained cohorts, E24-style).
2. Give the persistent state the sensory specificity upstream
   structure already has (write-path consolidation).
3. Closed loop with endogenous output activity (E24 S4 showed
   outputs CAN join the core under V2.1 — the E19 precondition).
4. Structural growth (U3/E4-family, birth_trigger exists but
   was never used in this era) when routing proves binding.

---

## 5. Standing rules for working in this repo

- Frozen = frozen: V2, V2.1, V2.2, E1–E24 records and commits
  are historical; never reinterpret a frozen verdict.
- Stage C: NOT EXECUTED, record retracted; the frozen selection
  rule remains available for a future execution — do not
  fabricate its result again.
- Every experiment: freeze BEFORE implementation; amendments
  logged; negative results retained; runs preserved (aborts
  included); no post-hoc endpoint/threshold changes.
- Exploratory work (X-series, SDE-series) needs no E-number;
  promotion to E-numbers only when interesting + discriminating
  + reproducible.
- V2.3 is implemented with PARTIAL SUPPORT — next steps require
  the same discipline (frozen design, identity gate, causal
  anchor).

---

## 6. Reference map

| Topic | File |
|---|---|
| Goal/charter + unknowns | `ANIMA_PHASE_0_PLAN.md`, `docs/unknowns-registry.md` |
| Decisions | `docs/decision-log.md` |
| V2 substrate | `docs/anima-v2-protocol.md`, `docs/anima-v2-design.md` |
| V2.1 slow state | `docs/v2_1-spec.md` (+ erratum, info audit `docs/v2_1-info-audit.md`) |
| V2.2 latch (negative) | `docs/v2_2-spec.md` |
| V2.3 partition (partial) | `docs/v2_3-design.md` |
| E24 | `docs/anima-e24-protocol.md` |
| Exploratory logs | `docs/xplor-log.md`, `docs/xplor2-arch-review.md` |
| This summary | `docs/ANIMA_OVERVIEW.md` |