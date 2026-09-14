# ANIMA v2 — Pre-registered Protocol (frozen before implementation)

**Status: PRE-REGISTERED. No code, no parameters changed after this
point, no runs executed.** Derived from `docs/anima-v2-design.md`;
supersedes none. All frozen values below may be changed ONLY by a
logged amendment requiring explicit user approval; a gate failure is
reported and blocked, never silently retuned.

---

## 0. Primary hypothesis (frozen wording)

> **H1: An initially under-structured network can develop
> input-specific receptive fields from experience alone** — using only
> local plasticity laws M1–M6, no labels, no channel-group knowledge,
> no stage information, no reward, and no changes to the E1/E3
> curriculum, STDP rule, seed, or thresholds.

H1 is decomposed into four pre-registered predictions P1–P4 (below)
and tested against the frozen decision tree. The v2-minus-M6 control
arm and the five-arm ablation plan (section 7) are registered with the
same gates.

---

## 1. Scope and invariants (frozen)

- Base organism/config: `configs/e3-armB.toml` values — curriculum,
  seed **20260912**, additive STDP (`stdp_tick`, a⁺ 0.005, a⁻ 0.0053,
  τ 20 ms), adaptation 200/0.05, amplitude 52, LIF τ_m 20 / V_th 1,
  caps, runaway detector 50 Hz / 5 s. **Unchanged.**
- No curriculum/stimulus changes; no mechanism may read `pattern_id`,
  channel-group labels, stage, rep, or epoch.
- Determinism: same seed ⇒ byte-identical chunk telemetry (existing
  sha256 validation extended to the new mechanisms).
- All structural random draws come from the run's single seeded
  `Xoshiro256PlusPlus` in a fixed, documented consumption order
  (below); iteration is over `Vec` index order only; tie-breaks by
  lowest NeuronId/SynapseId. No HashMap iteration anywhere.
- Analysis labels (pattern ids, stages) are used ONLY in evaluation
  instruments, never in mechanisms.

---

## 2. M1 — Dense-weak deterministic initialization (frozen)

| Parameter | Value | Meaning |
|---|---|---|
| `p_in` | 0.5 | Bernoulli: internal/output neuron ← each input channel |
| `w_in` | U(0.02, 0.06) | initial input-afferent weight |
| `p_rec` | 0.2 | Bernoulli: internal/output neuron ← each other non-input neuron (excluding self) |
| `w_rec` | U(0.005, 0.02) | initial recurrent weight |
| `w_max` / `w_min` | 1.0 / 0.0 | STDP bounds unchanged |

- Wiring rule replaces the p = 0.038 draw for non-input neurons.
- **Consumption order (frozen)**: for each non-input neuron in id
  order: (1) input afferents in channel-id order; (2) recurrent
  afferents in source-id order; (3) M6 inhibitory afferents
  (section 6); then (4) candidate pools (section 4) for all neurons
  in id order, candidate partner draw order = channel-id then
  neuron-id order.
- Input neurons remain pure spike sources (D8): no afferents, no
  candidates, no inhibition received.

## 3. M2 — Per-neuron excitatory normalization (frozen)

- Applied **once per structural window** (definition §8) to each
  non-input neuron's incoming **excitatory** weights (input +
  recurrent; candidates and inhibitory excluded).
- Rule: `w_i := w_i × T_e / Σw` for all live incoming excitatory
  weights, then clamp each to `[w_min, w_max]`.
- Invariant: post-pass `Σw ≤ T_e` always (scaling down can only lower
  the sum; scaling up is clamped at w_max which never exceeds the
  pre-scaling sum beyond T_e). Drive per neuron is bounded **by
  construction**.

| Parameter | Value |
|---|---|
| `T_e` | 0.8 |

## 4. M3 — Candidate-synapse lifecycle (frozen)

| Parameter | Value | Meaning |
|---|---|---|
| `C` | 6 | candidate slots per neuron |
| `w_c_init` | 0.01 | candidate starting weight |
| `Δ_perm` | +0.005 | per co-active window increment |
| `decay_c` | 0.9 | multiplicative decay per non-co-active window |
| `θ_permanent` | 0.05 | who-crossed-threshold becomes a real synapse |
| `w_c_permanent` | 0.02 | weight of a newborn synapse |
| `θ_die` | 0.005 | below → candidate withdrawn, redrawn |

Lifecycle (frozen order within a window):

1. Each neuron's candidates accumulate co-activity: if the candidate's
   pre fired AND the post fired during the window → `w_c += Δ_perm`;
   else `w_c *= decay_c`.
2. `w_c ≥ θ_permanent` → **permanence**: creates a real synapse with
   weight `w_c_permanent`, plastic, reason `candidate-permanence`.
   Slot accounting per M5 (evict if full).
3. `w_c < θ_die` → withdrawn, slot freed, redrawn from eligible
   partners: unconnected input channels first (channel-id order with
   seeded Bernoulli acceptance `p_cand_in = 0.5`), then unconnected
   non-input neurons (`p_cand_rec = 0.5`); no duplicate candidate may
   target an existing live synapse partner; candidates never target
   input neurons (D8).
4. Candidates consume no live slot until permanence.

**Successful structural change (frozen definition)**: a
`SynapseCreated` event attributed to `candidate-permanence` whose
synapse is still alive at **t+10,000 ms** (100 windows later) counts
as *established*; all counts of "structural changes" in this protocol
refer to established changes. Ephemeral permanence (pruned before
survival check) is churn and is reported separately.

## 5. M4 — Competitive pruning (frozen)

| Parameter | Value |
|---|---|
| `θ_prune` | 0.005 |
| `prune_windows` | 10 (1 s) |

- A live excitatory synapse with `w < θ_prune` for ≥ `prune_windows`
  consecutive structural windows is pruned (reason
  `competitive-prune`), freeing an M5 slot.
- The Phase-0 silence rule (`w < 0.02` for 60 s) remains as a
  backstop with reason unchanged; it is expected to be superseded.
- Candidates never trigger M4 (they have their own cycle).

## 6. M5 — Synapse-budget accounting (frozen)

| Parameter | Value |
|---|---|
| `B_e` | 40 | live excitatory slots per neuron |
| `B_i` | 10 | live inhibitory slots per neuron |

- A permanent candidate requires a free `B_e` slot; if full, **evict**
  the live excitatory synapse with the lowest weight (tie: lowest
  SynapseId), reason `budget-eviction`, before inserting.
- Budget invariant (checked in telemetry each window): live
  excitatory ≤ `B_e`, live inhibitory ≤ `B_i` per neuron; total live
  synapses ≤ `N_neurons × (B_e + B_i)`.
- This is the structural conservation law: total possible network
  drive is bounded by construction (E4 runaway class excluded
  structurally, not by detector).

## 7. M6 — Anti-Hebbian inhibitory synapses (frozen)

| Parameter | Value |
|---|---|
| `p_inh` | 0.3 | Bernoulli: non-input neuron ← each other non-input neuron |
| `w_inh_init` | U(0.01, 0.03) | |
| `a_inh` | 0.005 | per co-active window increment |
| `decay_inh` | 0.98 | per-window multiplicative decay |
| `w_inh_max` | 0.10 | hard cap |

- Inhibitory afferent: pre neuron → post neuron; per spike delivers
  `−(amplitude × w_inh)` into post's `i_syn` (same amplitude law as
  excitation, negative sign).
- Update per window (after M2): for each inhibitory synapse, if pre
  fired AND post fired in this window → `w_inh = min(w_inh + a_inh,
  w_inh_max)`; else `w_inh = min(w_inh × decay_inh, w_inh_max)`.
  Anti-Hebbian: co-activity strengthens suppression ⇒ learned
  decorrelation.
- No normalization of inhibitory totals; bounded by w_inh_max and B_i.
- Input neurons are never pre or post of inhibitory synapses (D8).

## 8. Update order and timescales (frozen)

- **Fast (per tick)**: existing trace update + `stdp_tick` (both
  rules), adaptation, network step. Unchanged.
- **Slow**: structural window `W = 100` ticks (100 ms). At the end of
  each window, in this exact order:
  1. M4 prune (frees slots)
  2. M3 candidate accumulate → permanence/die → redraw (M5 eviction
     inside permanence)
  3. M2 normalize excitatory afferents
  4. M6 inhibitory update
  5. budget-invariant check (assert + telemetry `ResourceUsage`
     fields)
- Wall-clock never enters any rule.

## 9. Telemetry and measurement (frozen)

- New structural events reuse existing payloads: `SynapseCreated`
  (reason `candidate-permanence`), `SynapsePruned` (reasons
  `competitive-prune` / `budget-eviction`), `NeuronDormant` etc.
  unchanged. No new wire event kinds.
- `ResourceUsage` gains budget fields: live excitatory/inhibitory
  counts per window (telemetry v2 columns already optional — additive,
  lossless).
- Snapshots every 1000 ticks unchanged (RF statistics are computed
  from snapshots in the analyzer/examples — measurement, not
  mechanism).

---

## 10. Pre-registered predictions (frozen values)

Analysis window for all S1 metrics: late-S1 = presentations in the
second half of S1; RF snapshot = snapshot nearest t = 715,000 ms
(S1 ends 725,000 ms).

### P1 — Receptive-field specialization (mechanism-neutral + labeled)

- **Quasi-private RF (frozen definition)**: for a neuron's live input
  afferents, normalized channel-weight vector `p_c = w_c / Σw_c` over
  the 24 input channels (zero for unconnected):
  - (a) **specialized**: Shannon entropy `H = −Σ p_c log₂ p_c` ≤ 1.5
    bits;
  - (b) **quasi-private**: specialized AND the channels with `w >
    0.5 × max(w)` all belong to a single channel group (group labels
    used ONLY here, in the analyzer).
- **P1 supported** iff ≥ 10 of the 40 internal neurons are
  quasi-private AND at least one quasi-private neuron exists for each
  of the three input channel groups.

### P2 — Stability

- Zero runaway failures (unchanged detector); S1 internal mean rates
  within [20, 250] Hz; budget invariant holds every window.
- **P2 supported** iff all three hold.

### P3 — Cross-pattern separation/selectivity (identical instruments
### to E1/E3/E3b)

- Late-S1 mean cross-pattern cosine < **0.60** AND median selectivity
  > **0.50** (both required).
- **P3 supported** iff both hold; "weak" if exactly one holds with all
  P2 conditions.

### P4 — Cross-seed self-organization (seeds frozen)

Seeds: **20260912** (canonical), **9001**, **424242**. For each seed
run the full frozen protocol.

- (i) Same statistics: pair-wise |mean H(seed_a) − mean H(seed_b)|
  < 0.20 AND quasi-private fraction within ±0.20 (absolute) across
  seeds.
- (ii) Different assignments: Jaccard similarity of the
  {neuron → top-contributing channel} map < 0.50 across every seed
  pair.
- **P4 supported** iff (i) AND (ii) hold for all pairs. This is the
  self-organization test: the STATISTICS are experience-selected, the
  ASSIGNMENT is not pre-wired.

---

## 11. Failure, degenerate, inconclusive (frozen definitions)

- **Failure (run-level)**: any `Failure` event (runaway, resource,
  fragmentation) — run aborts as before; endpoint verdicts not
  evaluated; count as REGRESSION of the arm.
- **Degenerate**: at the RF snapshot, ≥ 90% of internal neurons have
  zero live afferents (network died). Run is reported DEGENERATE and
  excluded from P1/P3 evaluation; P2 still evaluated.
- **Inconclusive**: (a) zero *established* structural changes
  (§4 definition) by S1 end — structural machinery did not engage;
  (b) gate failure (section 12); (c) P2 fails (instability makes P1/P3
  uninterpretable); (d) telemetry lost/determinism violation.
- Anything not Failure/Degenerate/Inconclusive is a valid arm for
  P1–P4.

## 12. Gate (frozen; identical for every arm)

Probe = S1 with 10 reps of each pattern (240 s S1 baseline shape),
same seed. Gate passes iff ALL of:

1. zero failures;
2. ≥ 3 **established** structural changes;
3. S1 internal mean rates within [20, 250] Hz.

Gate fail ⇒ arm blocked and reported; amendment (user-approved only)
required before rerun.

## 13. Control arm — v2-minus-M6 (frozen)

Full v2 minus M6 only (`p_inh = 0`, no inhibitory synapses; all else
identical, seeds 20260912 only).

- Prediction: P1/P3 fail — specifically late-S1 mean cross-cosine
  ≥ 0.60 — demonstrating M6's necessity for separation under H1.
- Same gates; verdict recorded even if P1/P3 fail as predicted.

## 14. Minimal ablation plan (frozen order; each arm its own
### registration row with identical gates/metrics)

| Arm | Removed | Predicted effect if mechanism is load-bearing |
|---|---|---|
| v2-full | — | P1–P4 supported |
| v2−M6 | inhibitory synapses | separation fails (cross ≥ 0.60) |
| v2−M2 | normalization | saturation resumes; P1 weak; possible runaway (guard) — E1-class endpoint |
| v2−M3/M4 | candidates + pruning (turnover frozen; no permanence, no recycle) | RF fixed at M1 lottery; specialization only where birth lottery already private ⇒ P1 fails |
| v2−M5 | budget (unbounded slots, M2 still on) | synapse count grows; drive accumulates; runaway risk returns (E4-class) |
| v2-M1-only | all of M2–M6 | static dense-weak net: the "lottery baseline" for honest attribution |

Each ablation: same seed 20260912, same gates, same verdict tree.
Ablations run only after the main arm and the −M6 control complete;
their results cannot alter the main arm's frozen verdict.

## 15. Execution order (frozen)

1. Main arm v2-full (gate → full → P1–P4).
2. v2−M6 control.
3. Ablations in table order.
4. Cross-seed P4 runs (after main arm; can run in parallel).

No parameter may be altered based on any result, including within
Ablation arms.
---

## Implementation audit record (pre-run; before any protocol experiment)

Mechanical audit: 19/19 protocol clauses verified against the code
(script `/tmp/audit_check.py`; manual review of the same sites).
Full existing suite: **76 tests green, 0 warnings**; release build 0
warnings. Unit coverage added for every mechanism: M1 dense-weak
connectivity + determinism; M2 conservation + invariant across 30
windows; M3 permanence under co-activity; M4 prune (excitatory-only
verified against inhibitory untouched); M5 eviction on full budget +
budget invariant across 20 windows; M6 sign/decay/cap + STDP non-
interference; v2-off legacy identity; full structural-cycle
determinism.

### Audit notes (interpretations/implementation decisions — no frozen
### value changed)

- **A-1 (M1×M5 interaction, REQUIRED interpretation)**: the frozen M1
  inhibitory draw (p_inh = 0.3 over 52 non-input sources) yields mean
  inhibitory in-degree ≈ 14.9 per internal neuron at E1 scale —
  exceeding the frozen M5 budget B_i = 10. The M5
  budget-invariant ("live inhibitory ≤ B_i per neuron, checked every
  window") is a frozen clause; M1's Bernoulli is a distribution over
  *which* sources, not an unbounded count. Implemented as: the M1
  inhibitory draw is capped at B_i per neuron (budget is the binding
  constraint; all frozen values unchanged). Consequence: for neurons
  that hit the cap, RNG draws for later sources are skipped, so the
  RNG stream differs from a hypothetical uncapped wiring — this is
  deterministic within v2 and does not affect any other arm.
- **A-2 (ablation switches)**: protocol §14 defines ablation arms but
  no implementation switch. Added `disable_m2/m3_m4/m5/m6` flags to
  the `[v2]` config section (default false = mechanism ON). These are
  protocol-given arm selectors, not parameters.
- **A-3 (observed dynamics in a parse/telemetry smoke run, NOT a
  protocol run; deleted)**: one non-gate full-curriculum smoke was
  run to verify telemetry plumbing (deleted afterward). It showed M1
  wiring (1,622 synapses), M4 pruning active (1,033
  competitive-prune events), budget telemetry populated (8,735
  rows), and **zero candidate-permanence events**. The gate probe
  will decide whether the candidate regime engages; per the frozen
  gate, a probe with < 3 established structural changes blocks the
  arm. No tuning performed or permitted.

---

## Gate execution record (frozen experiment, 2026-09-13)

Gate probe shape: S1 reps=40 (240 s S1), S2/S3 reps=2, per §12.
Gate bars per §12: (1) zero failures; (2) ≥ 3 established structural
changes; (3) S1 internal mean rates within [20, 250] Hz.
**Gate fail ⇒ arm blocked; amendment (user-approved) required.**

| Arm | config sha256 | failures | established changes | S1 mean-rate (snap) | Gate |
|---|---|---|---|---|---|
| v2-full | 48624aa8… | 0 | **0** | 0–47 Hz | **FAIL (cadence)** |
| v2-minus-m6 | 9943d04e… | 0 | **0** | 0–297.5 Hz | **FAIL (cadence + rates > 250)** |
| v2-nom2 | 2522596e… | 0 | **0** | 0–49 Hz | **FAIL (cadence)** |
| v2-nom3m4 | 953826d0… | 0 | **0** | 0–104.6 Hz | **FAIL (cadence)** |
| v2-nom5 | 7f18b568… | 0 | **0** | 0–47 Hz | **FAIL (cadence)** |
| v2-m1only | b9cb6599… | 1 runaway @ t=19.2 s (mean 234 Hz) | **0** | — | **FAIL (stability)** |

All six arms FAIL the gate. **No full curriculum runs were executed;
P1–P4 are not evaluable; cross-seed P4 runs are not authorized** (the
protocol's P4 clause runs them "after the main arm", and the main arm
is blocked). This is the registered endpoint, not a retunable result.

### Structural-change statistics (gate probes)

- All arms: candidate-permanence events = 0; established changes = 0.
- M1 initial wiring: 1,622 synapses (v2-full; 1,108 in v2-minus-m6
  because p_inh=0 removes ~514 inhibitory afferents — consistent).
- Competitive prunes (M4): 848–1,018 per 240 s probe → M4 is live and
  recycles; M3 permanence never engages.

### Mechanism diagnosis (analysis only; no tuning permitted)

M3 permanence is structurally unreachable under the frozen dynamics:
candidate w starts 0.01, gains +0.005 per co-active window, decays
×0.9 per non-co-active window. With 20 Hz Poisson channels and 1500
ms off-periods (15 consecutive ×0.9 windows per presentation cycle),
the candidate fixed point is ≈ 0.01–0.02 — far below θ_permanent =
0.05. Zero permanence across ~25,500 windows of gate probing
confirms a dynamics-level mismatch, not bad luck. The registered
cadence bar (≥ 3 established changes) cannot be met by the frozen
M3 in this curriculum.

### Additional observations (recorded, not verdicts)

- v2-minus-m6 shows S1 peak mean-rate 297.5 Hz (> 250 bar): within
  this dense-weak regime, M6 inhibition contributes to rate bounding
  (the opposite of E3b's sparse regime, where global inhibition
  worsened separation). Not a verdict — the arm failed cadence first.
- v2-m1only (all mechanisms off) is the only arm with a stability
  failure (runaway at t ≈ 19 s): M2 normalization is load-bearing
  for bounded rates in the dense-weak wiring. Again observation
  only.

### Receptive-field statistics observed at probe end (informational;
### P1 is NOT evaluable without a full run — no verdict)

v2-full gate probe RF snapshot (t = 259,000 ms, probe-S3):
quasi-private 15/40 (fraction 0.375), mean H = 0.142
per group [A,B,C,other] = [7, 3, 5, 0]. NB: with zero structural
changes the RFs are pure M1-initial weights post-STDP; not evidence
for P1.

---

## Amendment M3-1 — Candidate accumulation reachability correction

**Status: PROPOSED — NOT APPROVED. No code changed, no run executed.
Approval required before implementation.**

### 1. Problem (from gate evidence)

All six gate probes produced **zero** candidate-permanence events. The
mechanism diagnosis: under the frozen stimulus statistics, the M3
candidate fixed point lies *below* θ_die, so candidates die and
redraw forever and permanence is structurally unreachable.

### 2. Mathematical derivation (frozen stimulus statistics only)

Window W = 100 ms; channel Poisson λ = 0.02/ms (20 Hz):
P(pre fires in an ON window) = 1 − e^(−λW) = 1 − e^(−2) ≈ **0.8647**.
Presentation: 5 ON windows (500 ms); off: 15 windows (1500 ms).

Per cycle, weight recurrence (gain on co-active windows, decay on all
non-co-active windows after it):

- Max gain/cycle = Δ_perm · d^(15) · Σ_{j=0..4} d^j  (all 5 ON windows
  co-active, every gain then decays through the remaining windows).
- Fixed point w* = gain_max/(1 − d^15).

| | d (decay_c) | Δ_perm | max gain/cycle | w* (max p) | θ_die | θ_perm |
|---|---|---|---|---|---|---|
| **frozen** | 0.9 | 0.005 | 0.00422 | **0.00531** | 0.005 | 0.05 |
| **proposed** | 0.99 | 0.01 | 0.04215 | **0.301** | 0.005 | 0.05 |

Frozen: even with p_co = 1, w* ≈ θ_die → permanence impossible by a
factor of ~10. This is the reachability failure, not luck.

### 3. Exact new M3 parameters (the ONLY changes)

- `decay_c`: 0.9 → **0.99** (candidate retains evidence across
  off-periods: 0.99^15 = 0.860 survival vs 0.206 frozen).
- `delta_perm`: 0.005 → **0.01** (per co-active window).

All other M3 parameters unchanged: c_slots 6, w_c_init 0.01,
theta_permanent 0.05, w_c_permanent 0.02, theta_die 0.005,
p_cand_in 0.5, p_cand_rec 0.5. M1/M2/M4/M5/M6, curriculum, STDP,
adaptation, seeds, gates: **unchanged** (M1 wiring, M2 t_e, M4
pruning, M5 budgets, M6 p_inh etc. all keep frozen values).

### 4. Expected candidate dynamics under the proposal

- **Single-presentation reachability (p_co = 1)**: starting w_c_init
  = 0.01, five co-active ON windows give 0.01 + 5×0.01 = 0.06 →
  permanence triggers at the **4th co-active window of one
  presentation** (0.05 reached within 500 ms).
- **Typical co-activity**: with p_co ≈ 0.61 (0.8647 × p_post_on ≈
  0.7), expected accumulation ≈ 0.0255/cycle and ~**1.6
  presentations** to permanence for a candidate whose channel is
  being presented.
- **Candidate lifetime when silent**: 0.01 → θ_die (0.005) in
  0.99^n = 0.5 ⇒ n ≈ 69 windows ≈ **6.9 s** — a silent candidate
  still dies and redraws within ~3–4 presentation cycles, preserving
  the search semantics (structural exploration continues; no
  permanent stuck sites).
- **Predicted permanence volume in the gate probe** (240 s S1, 40
  presentations per pattern): a candidate on a presented channel
  accumulates ≈ 0.006/co-active-window over ~40 presentations ⇒
  near-certain permanence; with 40 internal × 6 candidate slots ≈
  240 candidate sites, expected established changes ≫ 3 gate bar if
  channels of all three patterns are represented. (Candidates whose
  channels never present — none here: every channel belongs to A/B/C
  — stay silent and recycle.)
- Conservative failure mode: if p_post_on is much lower in practice
  (post rarely fires in ON windows), permanence slows to ~4–8
  presentations but remains reachable (fixed point 0.30 ≫ θ_perm).

### 5. Why this is a reachability correction, not outcome tuning

- It restores the *defined* function of M3 — a reinforced candidate
  becomes a synapse — which the frozen parameters provably prevent
  (w* < θ_die for any p_co). The lifecycle (accumulate → permanence /
  die → redraw), thresholds, and the resulting synapse
  (w_c_permanent 0.02, plastic, M5-budgeted, M4-prunable) are
  untouched.
- The choice is derived **only** from the frozen stimulus statistics
  and the mechanism's own constants; no gate probe result beyond the
  zero-permanence observation (which is itself a reachability fact,
  not an outcome) and **no full-run result** of any kind was used.
- The amendment changes *whether* the structural-search event can
  occur, not what it does to P1/P3 — those endpoints remain exactly
  as registered, and the gates remain the arbiter.

### 6. Updated gate expectations (unchanged gate bars)

- **v2-full, v2−M2, v2−M5, v2−M6**: cadence bar (≥ 3 established
  changes) expected to PASS — M3 now engages; stability/rates bars
  as probed before (v2-full 0–47 Hz in band; v2−M6 rates bar
  previously over 250 Hz — that arm's rates issue is unchanged by
  this amendment and would still FAIL the rates bar if it recurs;
  recorded, not tuned).
- **v2−M3/M4, v2-M1-only**: cadence bar still FAILS by registration
  (M3 disabled) — unchanged.
- P1–P4: unchanged endpoints; full runs still gated.

### 7. Scope check

No curriculum/stimulus change; no gate change; no other mechanism
change. This amendment is exclusively the two M3 constants above.

**Awaiting approval.**

---

## M3-1 implementation record (approved)

### Code + verification

- Applied to all `configs/v2*.toml`: `decay_c = 0.99`, `delta_perm =
  0.01`. Verified programmatically that all other 27 frozen values in
  every arm config are unchanged (only expected exception: v2−M6
  `p_inh = 0.0` per registered A-5).
- 3 new unit tests (all green): permanence reachable within one
  presentation (4th co-active window); silent candidate dies at
  ~69 windows (6.9 s) and redraws; pre-only activity causes no
  permanence. Existing M1/M2/M4/M5/M6 + determinism tests unchanged
  and green.
- Full suite: **79 tests, 0 failed**; release build **0 warnings**.

### New config hashes (sha256, prefix)

| Arm | hash |
|---|---|
| v2-full | `2a4d1c7b…` |
| v2-minus-m6 | `6efea191…` |
| v2-nom2 | `2ee0f1c4…` |
| v2-nom3m4 | `c3b56d67…` |
| v2-nom5 | `d8b55c42…` |
| v2-m1only | `af457523…` |
| v2 (master) | `9505227d…` |

### Behavior notes

- M3 permanence now fires at the 4th co-active window of a single
  presentation (0.01 + 4×0.01 = 0.05 ≥ θ_permanent).
- The new synapse starts at w_c_permanent (0.02); M2 renormalizes the
  neuron's excitatory total to t_e in the same window — asserted via
  mechanism-relevant properties (plastic, excitatory, alive,
  above M4 bar) rather than the transient pre-normalization value.

---

## M3-1 gate re-run (2026-09-14, all six arms, fixed binary)

One implementation regression was found and fixed during the re-run:
`evict_for` indexed `low_windows` after M3 permanence grew the synapse
arena mid-window (OOB, visible only once permanence engaged). Fixed
with in-place resize + regression test (400-window permanence storm).
Suite: **80 tests green, 0 warnings**.

| Arm | failures | established changes | S1 mean-rate (snap) | Gate |
|---|---|---|---|---|
| v2-full | 0 | **768** | 0–46.8 Hz | **PASS** |
| v2-minus-m6 | 0 | **449** | up to 301.7 Hz | **FAIL (rates > 250)** |
| v2-nom2 | 0 | **901** | 0–58.0 Hz | **PASS** |
| v2-nom3m4 | 0 | 0 | up to 104.6 Hz | **FAIL (cadence; M3 disabled by registration)** |
| v2-nom5 | 0 | **768** | 0–46.8 Hz | **PASS** |
| v2-m1only | 1 (runaway t≈19 s) | 0 | — | **FAIL (stability; all mechanisms off by registration)** |

M3-1's prediction verified: candidates now engage (768–901 established
per probe), permanence fires within a single presentation in unit
tests, silent candidates still die in ~6.9 s. **v2-full passes the
gate** → proceed with the frozen execution order: full run, P1–P4,
then the v2−M6 control (its own rates-bar failure is recorded and
blocks it), then registered ablations.

Gate-row detail: v2-nom2 permanence 2063 events / 901 established;
v2-full 1991 / 768; v2-nom5 identical to v2-full (expected — M5
disable only changes eviction among already-established synapses at
this probe scale).

---

## v2-full execution record — MAIN ARM (2026-09-14)

Run: `runs/v2-20260914T093312Z`, seed 20260912, curriculum-complete,
0 failures, config `configs/v2-full.toml` (sha `2a4d1c7b…`, M3-1).

### Structural-change statistics (full run)

- candidate-permanence events: **4,532**; established at t+10 s:
  **2,272**; competitive prunes: 5,425. M3 search is continuously
  active; M4 recycles; budget invariant held every window (hard
  assert, no abort).

### P1 — Receptive-field specialization: **SUPPORTED**

RF snapshot t = 715,000 ms: mean entropy H = 1.029; **23/40 internal
neurons quasi-private (fraction 0.575)**; per group [A=2, B=12, C=9]
— all three channel groups represented. Bar: ≥ 10 quasi-private AND
every group present. ✓✓

### P2 — Stability: **SUPPORTED**

- Zero runaway failures (unchanged 50 Hz/5 s detector).
- S1 internal mean rates: snapshot range 0.0–46.8 Hz (bursts in the
  [20,250] band; floor is off-period silence).
- Budget invariant (live_e ≤ 40, live_i ≤ 10 per neuron) held every
  structural window.

### P3 — Separation/selectivity: **SUPPORTED**

- Late-S1 mean cross-pattern cosine = **(0.000 + 0.013 + 0.000)/3 =
  0.0043** < 0.60 ✓
- Median selectivity = **0.991** > 0.50 ✓ (n = 104)
- Within-pattern mean 0.958; assembly late-S1 = 0.9495.
- Retention (measured, not a pre-registered endpoint): A 0.823,
  B 0.955, C 0.690.

### Comparison vs E1/E3/E3b (identical instruments)

| metric (late S1) | E1/armA | E3 | E3b | **v2** |
|---|---|---|---|---|
| cross A-B | 0.746 | 0.753 | 0.768 | **0.000** |
| cross A-C | 0.604 | 0.603 | 0.695 | **0.013** |
| cross B-C | 0.667 | 0.670 | 0.758 | **0.000** |
| mean cross | 0.672 | 0.675 | 0.740 | **0.004** |
| selectivity median | 0.428 | 0.455 | 0.366 | **0.991** |
| within-pattern | 0.872 | 0.846 | 0.876 | **0.958** |

### P4 — Cross-seed self-organization: **SUPPORTED**

Runs: `runs/v2-seed9001-20260914T093702Z` (seed 9001),
`runs/v2-seed424242-20260914T093702Z` (seed 424242), both
curriculum-complete, 0 failures.

| seed | mean H | QP fraction | established |
|---|---|---|---|
| 20260912 | 1.029 | 0.575 | 2,272 |
| 9001 | 1.142 | 0.400 | 3,184 |
| 424242 | 1.169 | 0.500 | 3,617 |

(i) |Δmean H| ≤ 0.14 < 0.20 all pairs; |Δfrac| ≤ 0.175 < 0.20 all
pairs ✓. (ii) Jaccard agreement of {neuron → top channel} maps:
J(2026,9001) = 0.000, J(2026,424242) = 0.025, J(9001,424242) = 0.075
— all < 0.50 ✓. The statistics are experience-selected (reproducible
across seeds); the assignments are not pre-wired (near-disjoint).

### Interpretation (recorded, per protocol)

Every pre-registered primary prediction of the v2 architecture is
supported. The organism self-organized input-specific receptive
fields from experience alone under local laws M1–M6; separation
moved from cross-cosine ~0.67 (E1–E3b) to ~0.004 with the identical
evaluation instruments. Remaining registered items: the other arms'
full runs (control/ablations) — v2−M6 and v2−M3/M4, v2-M1-only
blocked at gate by registration; v2−M2 and v2−M5 passed their gates
and their full runs are the registered next step.

---

## Ablation full runs (2026-09-14, registered order; gate-passing arms only)

### v2−M5 (`runs/v2-nom5-20260914T094208Z`, seed 20260912)

| metric | v2-full | v2−M5 |
|---|---|---|
| mean cross-cosine | 0.0043 | **0.0043** (A-B 0.000, A-C 0.013, B-C 0.000) |
| selectivity median | 0.991 | **0.991** (n=104) |
| within-pattern | 0.958 | 0.958 |
| quasi-private fraction | 0.575 | **0.575** |
| mean H | 1.029 | 1.029 |
| established changes | 2,272 | **2,272** |
| failures | 0 | 0 |

**Verdict: M5 (budget/eviction) is not load-bearing at this
operating point** — outputs are byte-identical on every metric. The
per-neuron budget (B_e 40) rarely binds because M2 normalization caps
the weight total; eviction almost never fires. Recorded, not tuned.

### v2−M2 (`runs/v2-nom2-20260914T094208Z`, seed 20260912)

| metric | v2-full | v2−M2 |
|---|---|---|
| mean cross-cosine | 0.0043 | **0.789** (A-B 0.737, A-C 0.832, B-C 0.799) |
| selectivity median | 0.991 | 0.902 (n=99) |
| within-pattern | 0.958 | 0.996 |
| quasi-private fraction | 0.575 | 0.250 |
| mean H | 1.029 | **2.359** |
| established changes | 2,272 | 3,327 |
| failures | 0 | 0 |

**Verdict: M2 (per-neuron normalization to T_e) is the load-bearing
mechanism for separation.** Without it the candidate/STDP dynamics
still run (3,327 established changes — more, in fact) but weights
saturate toward uniformity (H 2.359, no competition between
afferents), cross-cosine returns to the E1-class 0.79, and
quasi-private fraction halves. This isolates the mechanism: the
conservation law (heterosynaptic competition) — not synaptogenesis or
STDP alone — is what converts structural search into separation.
Consistent with the design diagnosis and the v2−M5 null.

### Blocked arms (recorded, no full runs)

v2−M6 (rates bar > 250 Hz), v2−M3/M4 (cadence; M3 disabled by
registration), v2-M1-only (stability; all mechanisms off). Gate
records stand from the M3-1 gate re-run.

### Summary — mechanism attribution

- separation: M2 (essential); M3/M4 (necessary to supply candidates,
  but inert without M2's competition — v2−M3/M4 arm gate-blocked);
  M5 (not load-bearing at this operating point); M6 (rate bounding,
  not separation — v2−M6 gate-blocked on rates).
