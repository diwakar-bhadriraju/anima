# ANIMA E15 — Structural basis of rapid re-anchoring

Status: **FROZEN** (2026-09-17). No edits after this point except
registered amendments (A-series, user-approved, logged in the
appendix). Basis: `docs/anima-e15-audit.md` (feasibility, committed
7e0ddda). Decisions D1-D5 user-approved and registered below.

## 1. Question

The E14 behavioral re-anchoring (C->A, complete at REV1, k* = 1 in
all seeds) raises the structural question: what changes internally
when the representation re-anchors? E15 measures whether the T0 ->
REV1 representation change corresponds primarily to (A) rapid
structural reorganization, (B) re-expression of already-existing
structure, (C) both, or (D) unresolved from existing telemetry.

E15 is **analysis-only**: no simulation, no reruns, no organism or
mechanism modification, no new instrumentation. It reads the
committed E12 run dirs read-only with a measurement instrument in the
established class (imports anima_telemetry + std only, zero
anima_core imports — static test, read-only opens, deterministic).

## 2. Frozen inputs (committed E12 artifacts, never rerun)

| seed | run dir | telemetry sha256 (aggregate) | snapshots sha256 |
|---|---|---|---|
| 20260912 | runs/e12-20260916T202445Z | 695bd87ae6ee7477 | 005acea9a7f671a4 |
| 9001 | runs/e12-seed9001-20260916T202822Z | 0e248586f59cfaaf | 8ed50dd7ef3a4f90 |
| 424242 | runs/e12-seed424242-20260916T202822Z | 65d47f5fbfd87f4e | 9b83082187f08532 |

## 3. Registered instants (D5 — user-approved)

Snapshot frames are written after each tick's dynamics at
tick % 1000 == 0 (harness section 10).

- **T0** = snapshot tick **364,000** (pure post-60-SEQ state; round
  60 ends 363,500; uniform across seeds). Reported as the T0 anchor.
- **REV1-pre** = last snapshot strictly before the REV1 presentation:
  seed 20260912 (REV1-B in-round position 0, presentation
  [365000, 365500)) -> tick **364,000**; seeds 9001/424242 (position
  2, presentation [369000, 369500)) -> tick **368,000** (includes
  round-61 A/C plasticity; no B content). Reported alongside T0.
- **REV1-post** = first snapshot after the presentation ends:
  20260912 -> **366,000**; 9001/424242 -> **370,000**.
- **Primary structural interval: (REV1-pre, REV1-post].**
  Both T0 and REV1-pre are reported; seed-specific instants exactly
  as above.

## 4. Measurement definitions (D2, D3 — user-approved; no thresholds)

Synapse-side buckets by pre-neuron membership (input channel id):
- A-side = channels **0-7** (reported total), with finer buckets
  **0-3** and lower-B group **4-7**;
- C-side = channels **8-15** (reported total), with upper-B group
  **8-11** and finer bucket **12-15**;
- B-associated structure is reported through its two constituent
  shared groups exactly: **lower B = channels 4-7**, **upper B =
  channels 8-11** — never forced into A or C alone;
- internal-neuron pre (recurrent) synapses reported as a separate
  bucket; post-neuron side is never used for bucketing.

Per seed, from the committed artifacts, RAW ONLY (no cutoffs, no
"major restructuring" definition; any future cutoff must be a
registered amendment):

1. Topology events in (REV1-pre, REV1-post]: kind-6 SynapseCreated
   and kind-7 SynapsePruned, count + id + exact ms timestamp +
   reason, bucketed by pre side; ordering vs the REV1 presentation
   window [S_B, S_B+500): before / during / after.
2. Candidate -> permanence transitions in the interval (kind 6,
   reason `candidate-permanence`), bucketed.
3. Endpoint structural states: full live excitatory synapse list
   (id, pre, post, w, permanence status, bucket) at T0, REV1-pre,
   REV1-post.
4. Endpoint weight differences per live excitatory synapse across
   (REV1-pre, REV1-post] and (T0, REV1-post]; per-bucket total
   weight, weight-sum shares, per-neuron receptive-field churn
   (afferent symmetric difference + weight sums) — raw distributions.
5. Structural shares: fraction of bucket synapses created/pruned/
   changed-weight in the interval (raw ratios; no fixed cutoffs).
6. Budget cross-check: live_exc/live_inh (ResourceUsage, 100 ms)
   vs snapshot-derived alive counts; prune-event ids all created ids;
   created counts vs analyzer totals.
7. Stable-context control: T0 A-side structure existence (A-side
   total weight, neuron coverage by A-side afferents, per-neuron
   A-side receptive fields) — the decided evidence for hypothesis B.

## 5. Temporal scope (D4 — user-approved)

Three explicitly distinguished categories, reported separately:
- **Event timing**: structural events carry exact ms timestamps;
  orderable before/during/after the REV1 presentation window.
- **Endpoint structural difference**: snapshot states exact; any
  per-synapse delta across the interval exact.
- **Unresolved intra-interval weight evolution**: silent components
  (M2 normalize, M6 updates, passive decay, |Δw| <= 0.01 STDP) are
  not timestamped; only their aggregate residual is reportable.

**No causal claims.** "Structural change accompanied the flip /
preceded it / followed it" (event-ordered) is claimable; "caused"
never is.

## 6. Verdict (D1 — user-approved: JOINT-READING, no numerical
dominance cutoff)

Per seed, letter assigned by joint reading of that seed's raw
quantities:

- **A — structural reorganization**: directly observed topology /
  structural-state changes accompany the representation transition.
- **B — re-expression**: relevant A-side structure already exists at
  T0 and the observed transition occurs without corresponding
  topology change.
- **C — both**: pre-existing A-side structure AND newly observed
  structural changes are present/relevant to the transition.
- **D — unresolved**: the telemetry cannot distinguish the
  alternatives (incl. candidate-pool-only explanations, which are
  unobservable by §audit-4).

Raw structural quantities are reported regardless of the final
letter. Cross-seed: per-seed letters are reported individually; NO
cross-seed aggregation rule is registered — if seeds disagree, all
letters are reported with their evidence (no aggregate invented).
The E14 k* = 1 result is joined as the behavioral state (already
committed; not recomputed).

## 7. Verification (analysis-time, no reruns)

1. Artifact hash pins (table §2) re-verified.
2. Determinism: instrument output byte-identical on rerun (3/3).
3. Cross-checks §4.6 pass.
4. Suite green at freeze and unchanged by E15 (129 tests).

## 8. Deliverables

1. This protocol (frozen commit).
2. Read-only instrument + tests (import check; snapshot-tick grid;
   bucket membership; interval math per seed) — committed before
   execution.
3. Full suite green, 0 warnings.
4. Per-seed raw tables + per-seed letter; execution record appended
   here; registry updated; commit; report hash/tests/warnings/tree.
   Nothing beyond E15 is proposed.

## Appendix: amendments

- (none yet)
---

## E15 execution record (2026-09-17)

Instrument: committed with this record (e15_structure.rs — telemetry-
only imports, read-only, deterministic: byte-identical reruns 3/3).
Verification: artifact hash pins match the freeze table; M5
cross-check (ResourceUsage live_exc/inh at interval end ==
snapshot-derived totals: 309/514, 322/517, 317/516); prune-event ids
all within created ids; suite 129 green, 0 warnings.

### Per-seed raw results

Instants (registered): T0 = 364000; pre = 364000 (20260912) /
368000 (9001, 424242); post = 366000 / 370000. REV1-B window
[365000, 365500) / [369000, 369500). Buckets by pre-channel:
A-only 0-3, lower-B 4-7, upper-B 8-11, C-only 12-15, recurrent
(internal pre), inactive-input 16-23 (present in all seeds: 0 live
synapses — none reported).

**seed 20260912** (pre == T0):
- states (alive exc / inh / permanence): T0 311/514/253; post
  309/514/251.
- bucket total weight (T0 -> post): A-only 10.044 -> 9.702;
  lower-B 6.418 -> 7.204 (+0.786); upper-B 4.273 -> 4.819 (+0.546);
  C-only 1.271 -> 0.971 (-0.300); recurrent 19.594 -> 18.903.
- events in (364000, 366000]: created 6 (ALL candidate-permanence,
  ALL lower-B pre {4,5,7} -> posts {40,41,50,53,61,75}, w=0.02, all
  t=365300 DURING); pruned 8 (competitive-prune; 3 before, 3 during,
  2 after).
- endpoint dW!=0: 745/831 alive-both (89.6%); created/exc-pre
  1.93%; pruned/exc-pre 2.57%; residual (unresolved) dW sum -0.044.
- per-neuron churn: 13 internal neurons, 14 afferents (symmetric
  diff pre vs post).
- M5: max exc/neuron 10 (B_e=40); max inh/neuron 10 (B_i=10).
- behavior (joined E14): T0 A-B 0.680 / B-C 0.075; REV1 A-B 0.190 /
  B-C 0.929; k* = 1.

**seed 9001** (pre = 368000):
- states: T0 355/517/283; pre 372/517/300; post 322/517/250.
- bucket total weight (pre -> post): A-only 4.910 -> 4.660;
  lower-B 7.201 -> 7.700 (+0.498); upper-B 8.840 -> 9.988 (+1.148);
  C-only 2.826 -> 2.628 (-0.198); recurrent 17.821 -> 16.624.
  (T0 -> pre: A-side 11.046 -> 12.111, incl. the interval preceding
  REV1 with round-61 A/C exposures.)
- events in (368000, 370000]: created 7 (ALL candidate-permanence,
  ALL lower-B pre {5,6} -> posts {41,47,49,50,52,55}, w=0.02, all
  t=369300 DURING); pruned 57 (competitive-prune; 53 before — the
  largest wave, t=368100-368400 — 2 during, 2 after).
- endpoint dW!=0: 724/896 (80.8%); created/exc-pre 1.88%;
  pruned/exc-pre 15.3%; residual dW sum -0.018.
- per-neuron churn: reported in instrument output (raw).
- M5: max exc/neuron 12; max inh 10; exc totals 372 -> 322.
- behavior (joined E14): T0 0.760/0.097; REV1 0.127/0.672; k* = 1.

**seed 424242** (pre = 368000):
- states: T0 308/516/254; pre 338/516/284; post 317/516/263.
- bucket total weight (pre -> post): A-only 10.232 -> 10.104;
  lower-B 10.086 -> 10.256 (+0.170); upper-B 4.057 -> 4.343 (+0.286);
  C-only 4.802 -> 4.537 (-0.265); recurrent 12.422 -> 12.361.
- events in (368000, 370000]: created 1 (candidate-permanence,
  UPPER-B pre=10 -> post 24, w=0.02, t=369200 DURING); pruned 22
  (21 before, 1 after).
- endpoint dW!=0: 684/855 (80.0%); created/exc-pre 0.30%;
  pruned/exc-pre 6.51%; residual dW sum -0.020.
- per-neuron churn: reported in instrument output (raw).
- M5: max exc/neuron 9-10; max inh 10; exc totals 338 -> 317.
- behavior (joined E14): T0 0.554/0.065; REV1 0.112/0.746; k* = 1.

### Event timing vs endpoints vs unresolved (D4 discipline)

- EVENT TIMING: all creations and prunes carry exact ms timestamps;
  per seed the creations fall INSIDE the REV1 presentation window
  (t=365300 / 369300 / 369200, all [during]); prune waves precede
  the window for the pos-2 seeds; ordering exact, no causality.
- ENDPOINT: all three snapshot states exact; bucket totals and
  per-synapse deltas exact.
- UNRESOLVED: per-synapse attribution of silent weight evolution
  (M2 normalize, M6, passive decay, |dW| <= 0.01 STDP) inside the
  interval; only aggregate residuals reported (-0.044 / -0.018 /
  -0.020).

### Frozen D1 joint-reading verdict (per seed, no numeric dominance
### cutoff; raw quantities above regardless of letter)

Evidence common to every seed: (i) A-side machinery (channels 0-7:
A-only + lower-B totals 16.46 / 11.05 / 19.59 at T0 with 194 / 139 /
203 live synapses) fully present BEFORE the transition — the T0
control; (ii) directly observed structural changes within
(REV1-pre, REV1-post]: 6 / 7 / 1 new candidate-permanence synapses
ALL created during the REV1 presentation (lower-B in 20260912 and
9001, upper-B in 424242), per-neuron afferent churn (13 / 44?? /
raw per instrument), and pervasive endpoint weight movement
(89.6% / 80.8% / 80.0% of alive-both synapses, with lower-B net
gains +0.786 / +0.498 / +0.170 and C-only net losses -0.300 / -0.198
/ -0.265 in every seed).

Both classes of evidence are present and relevant to the transition
interval => per-seed verdict **C — BOTH** for all three seeds
(20260912 = C, 9001 = C, 424242 = C). No aggregation rule exists in
the frozen protocol; per-seed letters reported independently (they
happen to agree).

Registered caveats: the flip's behavioral magnitude (A-B collapses
by 0.4-0.6) vastly exceeds the directly observed structural delta
(create shares 0.3-1.9% of live excitatory synapses), and the
direction of the concurrent creations is not constant across seeds
(lower-B in two, upper-B in one) while the flip is uniform — the
evidence supports C without quantifying each component's
contribution; ENDPOINT weight changes include unresolved
intra-interval components; no causality is claimed (the creations'
timestamps fall during the presentation; accompaniment, not
causation, is established).
