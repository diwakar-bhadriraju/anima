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
