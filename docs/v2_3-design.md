# ANIMA V2.3 candidate — trace-partitioned M2 normalization (DESIGN ONLY, not implemented)

Status: ARCHITECTURAL DESIGN, 2026-09-20. Frozen SDE-B/C2/D
evidence base (docs/xplor2-arch-review.md §8–10). Nothing
implemented, nothing executed, no E-number. V2.1/V2.2 records
immutable.

---

## 0. Causality honesty, first

The proposed intervention does NOT cleanly test "M2 causality"
alone: partitioning the budget changes TWO things at once —
(i) reallocation no longer crosses trace boundaries, and
(ii) the effective excitatory ceiling per neuron rises
(t_e × n_buckets when both buckets are populated). SDE-B already
showed that raising capacity alone (t_e 0.8→1.6) does NOT restore
coexistence (balance fraction decreased) — so capacity is
evidence-excluded as a sufficient explanation. But a partitioned
run that restores coexistence still admits the composite
explanation "partition + capacity". The clean discriminator is
NOT available config-only; the smallest clean design is
**capacity-matched partitioning** (§4): partition runs where the
PER-BUCKET budget is t_e/n_active so the total never exceeds
baseline t_e. Then any coexistence recovery is attributable to
partitioning per se, with capacity held constant. This is the
design below. A smaller distinguishing intervention does NOT
exist: the only config-expressible M2 manipulations are disable
(E16, destroys organization — already measured) and t_e scale
(SDE-B — capacity, already measured). Partitioning is the
minimal untested causal probe of the SHARING itself.

## 1. Eligibility / write-epoch bucket — precise definition

A bucket is a per-synapse integer tag b ∈ {0, 1} plus per-neuron
bookkeeping, with NO reference to input identity, pattern ids,
stimulus markers, or any global clock of presentations:

- **Tag value**: b is assigned from the SYNAPSE'S OWN plasticity
  history: a synapse that receives an LTP update in structural
  window W carries, from the next window, the tag
  b = hash(post-neuron, window-parity) mod n_buckets — i.e. the
  bucket is determined by (which post-neuron, which structural
  window epoch) locally at write time. Equivalent formulation for
  implementation: per-neuron alternating epoch counter e(post),
  incremented every E_epoch structural windows (E_epoch = 40
  windows = 4 s); a synapse's bucket is the epoch parity at its
  LAST LTP event. No labels: the epoch counter is driven by the
  network's own structural clock (window_ticks), which advances
  identically in silence and drive.
- **Entry**: a synapse enters bucket b(post, e) on its next LTP
  event after an epoch flip. Synapses with no LTP since the last
  flip RETAIN their previous bucket (their tag only changes on
  write — "write-epoch" semantics: the bucket records WHEN the
  synapse was last potentiated, nothing else).
- **Leave**: a synapse never actively leaves a bucket; it can be
  RE-TAGGED only by a new LTP event in a later epoch (natural
  re-consolidation), or pruned by M4/M5 as today (prune is
  bucket-blind).
- **Bucket count**: n_buckets = 2 (frozen for this design).
  Rationale: the discriminating curriculum pair (bac/bca) has
  exactly two drive epochs; more buckets dilute the test.

Locality audit: tag depends on (own LTP events, own post-neuron's
epoch counter, global structural window count — which is already
a frozen substrate clock, not a supervisor). No input identity,
no labels, no reward, no cross-neuron state. (One acknowledged
non-local element: the window/epoch clock is network-global by
construction since E1 — unchanged from the existing substrate.)

## 2. M2 across buckets (capacity-matched — the core rule)

Per post-neuron, per normalization pass (same cadence as today,
end of each structural window):

```
for each bucket k in 0..n_buckets:
    S_k = sum of w over live non-inhibitory incoming synapses with tag k
# capacity matching: total budget NEVER exceeds baseline t_e
T = t_e / max(1, n_populated_buckets)      # populated = S_k > 0
for each populated bucket k:
    factor_k = T / S_k   if S_k > T
    rescale bucket-k synapses by factor_k (clamp w_min/w_max)
```

- **What this preserves**: within-bucket competition (the E16-
  demonstrated organizer) is INTACT and IDENTICAL in form —
  same rescale-to-target, same clamp, same pass cadence.
- **What this removes**: cross-bucket reallocation — a bucket
  over target no longer draws down the other bucket's synapses.
- **Total-budget-exceeded case**: cannot happen by construction
  (T scales down with populated count; total ≤ t_e always). If
  clamping (w_min floor) prevents a bucket from reaching T, the
  excess is simply NOT reclaimed from the other bucket — the
  invariant becomes sum ≤ t_e per neuron, as today.
- **Empty-bucket case**: unpopulated buckets cost nothing (T =
  t_e when only one bucket is populated ⇒ exact baseline
  behavior).

### Partitioning vs capacity — the explicit distinction

| | partition (this design) | capacity (SDE-B t_e=1.6) |
|---|---|---|
| per-trace ceiling | t_e/2 when both traces live | t_e (each trace could use all) |
| cross-trace reallocation | none | full (shared pool, bigger) |
| total excitatory mass | ≤ t_e (unchanged) | ≤ 1.6·t_e |
| prediction if sharing is the mechanism | coexistence recovers | no recovery (measured: none) |
| prediction if capacity is the mechanism | no recovery | recovery (measured: none — refutes) |

SDE-B's null on capacity + this design's capacity-matching make
the partition arm the isolated test of SHARING.

## 3. Decay, STDP, and all other mechanisms

- **Passive decay**: operates IDENTICALLY in all buckets (decay
  is per-synapse, tag-blind). The decay=0 arm of the causal
  comparison (§4) additionally removes the dominant eraser so the
  M2 residual is measured on a clean field.
- **STDP (a±, traces)**: unchanged, tag-blind.
- **M3/M4/M5/M6, E6**: unchanged, bucket-blind (M5 eviction
  remains budget-slot-based, not weight-based — no interaction
  with tags).
- **V2.1 u / V2.2 latch**: out of scope (arms run at β=0).

## 4. Null behavior + exact causal comparison

**Null (n_buckets = 1)**: tag exists but normalization ignores
it ⇒ bit-identical to baseline (byte-identity gate, C-S5-style:
100k-tick telemetry FNV match vs the same-config baseline run).
n_buckets=1 must consume NO new RNG draws (tags derived from
deterministic counters only — no RNG anywhere in this design).

**Comparison (the M2-causality test), 6 seeds × 2 curricula
(bac/bca) × 3 arms = 36 runs:**

| arm | M2 | decay | what it isolates |
|---|---|---|---|
| A1 baseline | shared (today) | 1e-6 | reference (SDE-D am53-d1, already run) |
| A2 shared+no-decay | shared | 0.0 | passive eraser removed (SDE-D am53-d0, already run) |
| A3 partition+no-decay | partitioned (T=t_e/n_pop) | 0.0 | sharing removed ON TOP of eraser removal |

The causal contrast is **A3 vs A2** (identical except M2
partitioning; capacity matched). A1/A2 anchors come from the
already-executed SDE-D runs — zero rerun needed (same binary
would be required; a re-run under the new binary's null gate
doubles as the byte-identity check).

Predictions:
- If shared M2 is the residual competitor: A3 first-block
  survivors ≫ A2, balance fraction ≫ 0.07–0.09, with total
  excitatory mass per neuron ≤ t_e (capacity held constant).
- If M2 sharing is NOT the residual competitor (e.g. LTP
  crowding under no-decay dominates): A3 ≈ A2 — partition
  changes nothing; the write/maintain failure lives elsewhere
  (candidate: spike-timing competition itself).

## 5. Endpoints (frozen before any run)

- **Coexistence (primary)**: first-block cohort survivors
  (A-dom for bac, C-dom for bca, v22wdiv definitions) and
  balanced-neuron fraction; paired within seed vs A2, exact
  sign test, n=6 ⇒ min p = 0.031.
- **E16 separation metric (organizational preservation,
  secondary)**: late-drive cross-pattern response cosine
  (cross_cosine instrument) must NOT collapse to the M2-off
  phenotype (E16 measured T0 A-B ≈ 0.99 with M2 off; healthy
  organization ≈ 0.6–0.75). Pass band: ≤ 0.85 in ≥ 4/6 seeds
  (i.e., partition must not destroy separation the way
  disable_m2 does).
- **Stability**: P2 abort count; total per-neuron excitatory
  sum ≤ t_e + 1e-6 (invariant assertion in-run, reported).
- **Identity**: n_buckets=1 null byte-identical (gate, blocking).

## 6. Failure modes and stability criteria

- F-P1 **Separation collapse** (E16 phenotype: cross-cosine →
  0.99+): partitioning weakens within-trace competition below
  the organizational threshold ⇒ M2 sharing is load-bearing;
  the maintenance-signal alternative (activity-gated decay
  exemption) becomes the candidate instead.
- F-P2 **Runaway/abort**: T-scaling does not bound total below
  t_e when clamps bind — watch P2; ≥3 aborts/12 partition runs
  = design instability, recorded, not suppressed.
- F-P3 **Tag churn**: frequent re-tagging (every LTP flips
  buckets) would recreate shared dynamics; mitigated by epoch
  granularity (4 s ≥ one block-half); diagnostic = tag-flip
  count per synapse reported in telemetry (if median flips
  > 2/synapse, the epoch is too fine — recorded as a design
  miss, not tuned mid-experiment).
- F-P4 **Bucket monoculture**: one bucket captures all synapses
  (all LTP in one epoch) ⇒ T returns to t_e ⇒ silently reduces
  to baseline; diagnostic = populated-bucket histogram per run.
- F-P5 **No effect** (A3 ≈ A2): M2 sharing causally excluded as
  the residual competitor; remaining candidate = timing-based
  LTP crowding; next probe would be STDP gating, NOT designed
  here.

## 7. Provenance/immusability

- Additive config: `m2_buckets: u32 = 1` (+ optional
  `m2_epoch_windows: u32 = 40`), serde-defaulted; absent =
  today's behavior, byte-identical (no RNG, no pass changes).
- Historical runs/configs untouched; V2.3 runs isolated under
  new prefixes; the intervention ships behind the same
  one-binary discipline (null gate first).
- Determinism: tags from counters only; capacity matching uses
  only per-neuron sums — no ordering sensitivity beyond the
  existing pass order (frozen as today's).

## 8. What would promote this to a formal experiment

A3-vs-A2 difference in the predicted direction with identity
gate green and no F-P1 collapse ⇒ M2-sharing causality
established ⇒ the write/maintain transition's residual is
localized to the normalization rule's cross-trace term —
at that point an E-numbered experiment (coexistence capacity
across curricula) is warranted. Otherwise: negative result
retained, next hypothesis branched per F-P5.
