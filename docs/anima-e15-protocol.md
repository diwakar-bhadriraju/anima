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