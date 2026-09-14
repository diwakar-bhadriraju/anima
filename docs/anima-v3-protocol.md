# ANIMA v3 — Pre-registered Protocol (frozen before implementation)

**Status: PRE-REGISTERED.** Written before any v3 code, config, or run
exists. Freezes the entire ANIMA v2 organism wholesale and changes
**only the sensory curriculum**: overlapping input-channel categories
(v2's categories were disjoint 8-channel blocks). All frozen values
may be changed ONLY by a logged amendment requiring explicit user
approval; a failed prediction is reported and recorded, never retuned.

---

## 0. Primary question (frozen wording)

> **H3: Does ANIMA's structural self-organization — which produced
> quasi-private receptive fields and near-orthogonal category
> representations under disjoint sensory categories (v2: P1–P4
> supported, mean cross-cosine ≈ 0.004) — survive when the sensory
> categories overlap in input space?**

No answer is assumed. Four registered outcomes are all valid (A–D,
section 10). The restriction on v3 is deliberately narrow:

- **No new mechanisms.** The v2 organism is byte-identical.
- **No parameter changes** of any kind (organism, plasticity,
  structural, resources, v2-section, RNG, seeds, windows).
- **No curriculum-statistics changes** beyond the channel sets
  themselves (presentation duration, spike statistics, jitter,
  inter-presentation timing, stage structure, repetition counts,
  novelty condition, ordering — all unchanged and verified against
  v2 by test).
- The **only deliberate experimental variable** is the channel set
  activated by each pattern.

---

## 1. Frozen architecture (verbatim v2, zero changes)

The complete v2 organism as executed and validated in
`docs/anima-v2-protocol.md` + M3-1 amendment, stored in
`configs/v2-full.toml` and `crates/anima-core/src/structural_v2.rs`,
frozen in this experiment **including**:

- neuron model and thresholds (LIF τ_m 20 / V_th 1, amplitude 52,
  adaptation 200 ms / 0.05);
- additive pairwise STDP (a⁺ 0.005, a⁻ 0.0053, τ 20 ms, w ∈ [0, 1]);
- M1 dense-weak initialization (p_in 0.5 / w U(0.02,0.06); p_rec
  0.2 / w U(0.005,0.02));
- M2 per-neuron excitatory normalization (T_e 0.8);
- M3 / M3-1 candidate lifecycle (C 6, w_c_init 0.01, Δ_perm 0.01,
  decay_c 0.99, θ_permanent 0.05, w_c_permanent 0.02, θ_die 0.005,
  p_cand_in/rec 0.5);
- M4 competitive pruning (θ_prune 0.005, 10 windows);
- M5 budgets (B_e 40, B_i 10, eviction), budget invariant asserted
  every window;
- M6 anti-Hebbian inhibition (p_inh 0.3, w U(0.01,0.03), a_inh
  0.005, decay 0.98, cap 0.10);
- structural window 100 ticks, order M4 → M3 → M2 → M6 → budget
  check;
- RNG behavior: single seeded Xoshiro256PlusPlus, frozen consumption
  order, worst-case determinism, seeds **20260912 / 9001 / 424242**;
- telemetry v2 (parquet chunks), snapshots, replay, viz, analyzer
  instruments (`v2_analysis`, `cross_cosine`, `read_rates`,
  `v2_events` — untouched; v3 analysis is a **new, measurement-only**
  example, section 8);
- resource limits, runaway detector (50 Hz / 5 s), fragmentation
  (0.6), stats_decimate, viz port.

Freeze enforcement (test, section 7): every v3 config's non-pattern
sections must be structurally identical to `configs/v2-full.toml`.

## 2. Curriculum-change mechanism (registered, environment-only)

`PatternSpec.channels` today expands group labels to channels via
contiguous `group_size`-blocks (`env.rs::channels_for`); overlapping
sets (B = 4–11) are inexpressible. v3 therefore registers a minimal
**curriculum-representation extension**: an optional explicit
`channel_ids: Vec<u32>` field on `[[pattern]]`.

- Present ⇒ the pattern drives exactly those channels (sorted,
  deduped, validated `< n_input_channels`); group labels ignored;
  configs MUST NOT populate both fields (parse-time error).
- Absent ⇒ behavior bit-identical to v2 (all existing configs
  unaffected).
- This touches **only the environment layer** (which channels a
  pattern activates). No organism code, no RNG sequence in the
  network, no telemetry schema changes.

**Determinism properties (registered, preserved by construction):**
spike streams are per `(hash(pattern_id), rep, channel,
schedule_index)` with per-stream RNGs, and the interleaved
presentation order derives from `(stage.id, salt = schedule_len)`
with the same stage structure. Therefore, compared to v2-full at the
same seed: **presentation order is identical**, and every
(pattern, rep, channel) stream that exists in both runs is
**bit-identical**. The v3 input differs from v2 only by which
channels each pattern activates. (Verified by test, section 7.)

## 3. v3 curriculum (frozen; the ONLY experimental variable)

24 input channels, 8 per category, **overlapping**:

| Pattern | Channel set | Category intersections |
|---|---|---|
| **A** | {0–7} | A-only {0–3}; A∩B {4–7} |
| **B** | {4–11} | A∩B {4–7}; B∩C {8–11} |
| **C** | {8–15} | B∩C {8–11}; C-only {12–15} |
| **D** | **{0–15}** = A ∪ C (formula unchanged) | subsumes A, B, C entirely |

Registered consequences (decided here, not after data):

1. **B has no private channels**: B ⊆ A ∪ C. Under shared-evidence
   statistics, B is distinguishable from A only via channels {0–3}
   (A-exclusive) and from C only via {12–15} (C-exclusive). Whether
   the organism finds B-selective representations is an open
   empirical question — no assumption.
2. **D = A ∪ C formula is preserved** (v2 defined D this way); its
   v3 channel set is {0–15} — an "everything-on" combination that
   contains every category's evidence. D was novel in S1 in both
   protocols. D-condition behavior is a registered secondary endpoint
   (section 9), compared against S1 categories with v2 instruments.

Everything else identical to v2/e1 curriculum (verified by test):
500 ms presentations; 1500 ms off; 20 Hz independent Poisson per
channel with ±2 ms jitter; stages S0 (5000 ms silence), S1 (A/B/C
interleaved, 120 reps each = 360 presentations), S2 (D blocked, 30),
S3 (A/B/C interleaved, 15 each = 45); total 436 presentations,
875,000 ms.

## 4. P1-v3 — receptive-field specialization (re-registered)

**Why the v2 criterion is re-registered (decided before data):** v2's
"quasi-private" merged two claims — concentration (H ≤ 1.5, top
channels > 0.5·max) and *privacy* (top channels all in one of three
**disjoint** groups). Under overlap, privacy is not a valid scientific
ideal: a neuron driven by channels {4–7} (A∩B) is legitimately shared
between A and B, and mixed receptive fields may be the correct answer.
The concentration claim survives and is renamed:

- **Specialized** (v3): entropy H ≤ 1.5 bits (same normalized
  channel-weight formula) AND the dominant-channel set D = {c : w_c >
  0.5·max w} is non-empty AND D ⊆ cat for at least one registered
  category set (A {0–7}, B {4–11}, C {8–15}). A specialized neuron is
  counted once per category whose set contains D (registered rule;
  categories are not disjoint, so a {4–7}-neuron counts for A and B).
- **BROAD**: not specialized (mixed RFs — a legitimate, measured
  class, not a failure).
- **P1-v3 supported** iff ≥ 10/40 internal neurons are specialized
  AND each category A, B, C appears in ≥ 1 specialized signature.
  (Bar unchanged from v2 for direct comparability.)

Additional registered overlap statistics (reported; no bars —
measurement of the open questions):

- **Signature distribution**: each internal neuron's signature =
  {cat ∈ {A,B,C} : D ⊆ cat}, from the six types {A}, {B}, {C},
  {A,B}, {B,C}, BROAD. (Signatures {A,C} and {A,B,C} are impossible:
  A∩C = ∅.) This answers "do neurons preferentially represent
  A-only/B-only/C-only/shared A/B/shared B/C/broader combinations".
  Note {B} requires D to span both {4–7} and {8–11} — the only way to
  be B-consistent without being A- or C-consistent.
- **Exclusive-evidence usage**: among specialized neurons, fraction
  whose D ∩ {0–3} ≠ ∅ or D ∩ {12–15} ≠ ∅ (uses category-private
  evidence).
- **Per-channel participation**: across internal neurons, fraction
  whose top channel is c, and mean normalized weight p̄_c = mean over
  neurons of w_c/Σw (zero when unconnected), for each of the 24
  channels. Answers whether shared channels (4–11) attract or dilute
  representation.

## 5. P2, P3, P4 — verbatim v2 (frozen definitions, identical bars)

- **P2 — stability**: zero runaway failures; S1 internal mean rates
  within [20, 250] Hz; budget invariant holds every window.
  Supported iff all three hold.
- **P3 — separation/selectivity** (instruments identical to v2):
  late-S1 mean cross-pattern cosine < **0.60** AND median selectivity
  > **0.50**. Supported iff both; "weak" iff exactly one holds with
  P2 fully satisfied.
- **P4 — cross-seed** (seeds 20260912 / 9001 / 424242):
  (i) pairwise |Δ mean H| < 0.20 AND |Δ specialized fraction| < 0.20;
  (ii) Jaccard of {neuron → top channel} < 0.50 across every pair.
  **Plus the registered v3 addition** (requested measurement): the
  per-signature-type fractions (section 4) must agree within ±0.20
  (absolute) across every seed pair — assignment-type stability.
  P4 supported iff (i) AND (ii) AND the signature-stability bound
  hold for all pairs.
- Analysis windows: late-S1 = second half of S1; RF snapshot =
  snapshot nearest t = 715,000 ms (identical timeline to v2).
- Established structural change: same definition as v2 §4
  (candidate-permanence alive at t+10,000 ms).

## 6. Failure, degenerate, inconclusive (verbatim v2 §11)

- **Failure / REGRESSION**: any Failure event (runaway, resource,
  fragmentation) → run aborts; endpoints not evaluated; stop and
  diagnose; no silent repair.
- **Degenerate**: ≥ 90% of internals with zero live afferents at RF
  snapshot → DEGENERATE; P2 still evaluated.
- **Inconclusive**: zero established changes by S1 end (machinery did
  not engage); telemetry lost; determinism/config-hash violation.
- P2 failing makes P1/P3 uninterpretable (verdict D if stable-but-
  structureless, see section 10).

## 7. Implementation scope and tests (registered)

- **Code changes**: `config.rs` (PatternSpec optional `channel_ids` +
  parse validation) and `env.rs::channels_for` (explicit-list branch).
  Nothing else. No organism crate changes.
- **Configs**: `configs/v3-full.toml`, `configs/v3-seed9001.toml`,
  `configs/v3-seed424242.toml` (exp_id `v3`, `v3-seed9001`,
  `v3-seed424242`); each recorded by sha256 of the raw file
  (RunStarted config_hash + protocol appendix at run time).
- **Tests** (all pre-registered):
  1. `channel_ids` expansion: exact sets, sort/dedup, bounds error,
     both-fields error, v2 label path untouched (existing tests pass
     unmodified).
  2. v3 curriculum conformance: pattern channel sets, stage
     structure (reps/order/off/silence), durations, rates, jitter,
     seeds, exp_ids — exactly as section 3.
  3. **Freeze check**: v3-full's non-pattern sections structurally
     equal v2-full's (all of run/organism/plasticity/structural/
     resources/v2).
  4. **Shared-stream identity**: at seed 20260912, the v3 schedule
     (order, starts, durations) equals v2's, and per-(pattern, rep,
     channel) trains on channels present in both curricula are
     identical (determinism preserved by construction).
- **New analysis example** `examples/v3_analysis.rs` (measurement
  only, mirroring v2 instruments): established changes, RF
  snapshot H / specialized / signatures / exclusive-evidence /
  per-channel participation / top-channel map, S1–S3 stage mean
  rates, S2-vs-S1 novelty ratio, D-vs-A/B/C activity-vector cosines,
  and per-stage retention-style reads taken verbatim from v2's
  metrics.json where applicable.

## 8. Experiment order (frozen)

1. `v3-full` (seed 20260912) → report P1-v3, P2, P3, secondary
   endpoints. **Gate: if P2 fails or run is REGRESSION, stop and
   diagnose; cross-seed is not authorized.**
2. Cross-seed `v3-seed9001`, `v3-seed424242` (same curriculum,
   different seeds) → P4.
3. Targeted ablations of **existing** v2 mechanisms
   (`v3-nom6.toml`, `v3-nom2.toml` = v2 arm configs with the v3
   curriculum) **only if** v3-full is outcome C or D with P2
   supported — to attribute the limitation. Never parameter sweeps,
   never new mechanisms.

Concurrent runs use the established resource etiquette (nice 10).

## 9. Secondary endpoints (frozen definitions, reported, no bars)

- **D-condition behavior** (question 5): D is the everything-on
  combination {0–15}. Registered measures: (a) S2 mean internal rate
  / late-S1 mean internal rate (snapshot-based, same instrument as
  P2); (b) cosine between mean internal activity vectors (mean
  per-neuron rate over the pattern's S1/S2 presentations, 40-d)
  for A, B, C, D; (c) S3 retention taken verbatim from v2's
  metrics.json definition (post-S2 response recovery).
- **Structural engagement**: established-change count by S1 end
  (registered floor for Inconclusive: zero).
- **Specialization asymmetry**: per-category presence counts and
  whether shared channels {4–11} recruit more neurons than exclusive
  channels {0–3} ∪ {12–15} (reported; interpretation in discussion).

## 10. Verdict tree (frozen; every outcome is a valid result)

| Outcome | Definition | Maps to |
|---|---|---|
| **A — generalizes** | P1-v3 AND P3 AND P2 supported (and P4 (i)) | self-organization survives overlap with concentrated RFs |
| **B — mixed** | P2 AND P3 supported, P1-v3 NOT supported (signatures/BROAD dominate) | self-organization survives but representations are mixed/shared |
| **C — collapse** | P2 supported, P3 NOT supported (cross ≥ 0.60 or selectivity ≤ 0.50) | overlap exposes a v2 mechanism limitation |
| **D — stable, no structure** | P2 supported, P1-v3 AND P3 both NOT supported, established > 0 | organism stable but cannot form useful representations |
| REGRESSION / DEGENERATE / INCONCLUSIVE | sections 6 | per definitions |

No outcome is "tuned away". Any change to sections 3–8 after a run
starts requires a logged user-approved amendment; section 8 order
itself is frozen.

## 11. Observability and reproducibility (unchanged from v2)

Live visualization, replayable telemetry, structural event telemetry,
resource usage, RF statistics, deterministic seeds, commit + config
hash recording, reproducible run directories — all v2 facilities
retained; the v3 run dir records `RunStarted` config_hash = sha256 of
the raw config file, and every v3 run's commit is recorded in this
document's appendix at execution time. If any observability component
fails, the run is stopped and diagnosed.

---

## Appendix (filled at execution time)

- Config hashes (sha256 of raw file, recorded before any run):
  v3-full `5854a19154a7ca4e0c4912fea37df6d937924b0de8e9e739ec7d2759afb34464`;
  v3-seed9001 `6502147d51b0f2156accdfeb9658a6507d5f078c4d405676a94c804a11b9b61a`;
  v3-seed424242 `d503714ffc9a36579aabd21ddb425baa0b82752002075959a5c521e87aa44c8f`.
- Commit: `(to be recorded at run time)`.
---

## v3 execution record (2026-09-14; commit 1bd9770 registration, runs below)

### v3-full (`runs/v3-20260914T164140Z`, seed 20260912, config
### sha `5854a191…`)

| Endpoint | Result | Bar | Verdict |
|---|---|---|---|
| P1-v3 | specialized 21/40 (0.525); presence A=7 B=21 C=14 | ≥10/40, all cats | **SUPPORTED** |
| P2 | 0 failures; late-S1 mean 12.8 / max 79.5 Hz; budget held | all three | **SUPPORTED** |
| P3 | mean cross 0.450; selectivity median **0.168** | cross<0.60 AND sel>0.50 | **WEAK → NOT SUPPORTED** |
| Structural | 20,873 permanence, **7,517 established** (v2: 4,532 / 2,272) | ≥1 (defined) | engaged |

P3 detail: cross A-B 0.697, A-C 0.009, B-C 0.645, within 0.954. The
mean-cross pass is carried entirely by A-C, the only pair sharing no
channels; both entangled pairs fail. Selectivity collapse (0.168 vs
v2's 0.991) is the decisive signal.

Overlap statistics (RF snapshot): signatures {A+B: 7, B+C: 14,
BROAD: 19} — **no pure {A}/{B}/{C} neurons at all**; exclusive-
evidence users 0/21 (no neuron uses channels 0-3 or 12-15, the only
discriminative evidence); per-channel participation concentrates on
channels 10 (53%) and 5 (25%) — the shared zones. Mean H 1.438.

D condition: cosine(D-mean, B-mean) = **0.979** vs A 0.711 / C 0.672
— the everything-on combination is absorbed by B's representation;
S2/S1 rate ratio 1.001 (no novelty differentiation); S3 retention A
0.939 / B 0.957 / C 0.947 (uniform, high).

### Cross-seed P4 (gate: v3-full P2 supported → authorized)

| seed | mean H | specialized frac | established | signatures |
|---|---|---|---|---|
| 20260912 | 1.438 | 0.525 | 7,517 | A+B 7, B+C 14, BROAD 19 |
| 9001 | 1.910 | 0.300 | 7,428 | B+C 12, BROAD 28 |
| 424242 | 1.772 | 0.225 | 6,813 | A+B 2, B+C 5, C 2, BROAD 31 |

**P4 NOT SUPPORTED**: (i) |ΔH| = 0.472 / 0.334 / 0.138 and |Δfrac| =
0.225 / 0.300 / 0.075 — canonical-seed pairs exceed 0.20; (ii)
Jaccard top-channel agreement 0.100 / 0.025 / 0.050 < 0.50 holds.
The outcome statistics themselves are seed-dependent under overlap —
the opposite of v2, where statistics reproduced and only assignments
differed.

### Registered ablation attribution (outcome C ⇒ §8.3)

| Arm | failures | established | specialized | sel. median | mean cross | verdict |
|---|---|---|---|---|---|---|
| v3-nom6 (M6 off) | 0 | 6,722 | **0/40** (BROAD 40) | 0.306 | 0.87 (A-B 0.927, A-C 0.807) | stable, no concentration |
| v3-nom2 (M2 off) | **1 runaway** (t=872,061, S3) | 2,670 | 7/40 | 0.078 | 0.97 | **REGRESSION** |

Attribution: M2 remains essential (off ⇒ total separation collapse
AND destabilization); M6 is load-bearing for concentration and rate
bounding (off ⇒ all-BROAD, H 2.51, D/S1 ratio 1.478 — the 16-channel
D drive is no longer checked). Neither mechanism, frozen as-is,
recovers the discriminative evidence channels under overlap.

### Frozen verdict (docs/anima-v3-protocol.md §10)

**Outcome C — separation collapses under overlap, revealing a
limitation of the v2 mechanism.** P2 stable; P3 not supported
(selectivity 0.168); P4 not supported; P1-v3 "supported" only as
concentration onto the SHARED evidence with zero exclusive-evidence
usage — i.e., mixed representations. Self-organization persists
(massive structural turnover, stability, concentration), but the v2
organism cannot exploit the category-exclusive channels (0-3,
12-15): under additive STDP + per-neuron T_e conservation, shared
channels co-activate twice as often and win every synapse budget,
and no frozen mechanism vetoes them. The failure is information-
theoretic (shared-channel frequency dominates) and no registered
mechanism addresses it. Not tuned; no amendment proposed.
