# ANIMA architectural review — post-V2.2 (exploratory, read-only)

Status: EXPLORATORY REVIEW, 2026-09-20. V2.1/V2.2 frozen as
records; not modified, verdicts not reinterpreted. No new
mechanism implemented. Read-only analyses + the diagnosis below.
New instruments (read-only): v22wdiv, v22during.

---

## 1. Current architectural diagnosis

**The substrate's residual carrier is the afferent weight vector
itself; everything downstream of it is a seed-dominated
winner-take-all that destroys curriculum information at every
stage where it could be retained.** Evidence chain, all from
committed artifacts:

- D-1 SENSORY ORGANIZATION EXISTS AND IS GENUINE. Drive-end
  plastic afferent structure is strongly curriculum-specific:
  pure-A runs leave 42–48/52 internal neurons A-dominant, pure-C
  runs 42–43/52 C-dominant; every top-u winner sits on a ~10^8
  afferent dominance ratio (wdiv). Even interleaved drives
  maintain separable cohorts (a1: 32 A-dom vs 14 C-dom; 777 il:
  28 vs 17). E1–E15's assembly formation was never the failure.
- D-2 THE SYNAPTIC LAYER IS WINNER-TAKE-ALL UNDER MIXED DRIVE.
  Blocked curricula collapse to single-dominance (bac → 5A/43C,
  bca → 34A/12C — the SECOND block nearly erases the first's
  afferents). M2's zero-sum excitatory budget forces A- and
  C-afferents to compete for the same t_e. This is the E17
  finding (M2-off destroys separation) inverted: M2-on destroys
  COEXISTENCE. Sequential memory is structurally impossible
  while the two traces share one budget.
- D-3 DURING-STIMULUS RESPONSES ARE PARTIALLY SEPARABLE (during-
  window A-vs-C cosine 0.30–0.89, seed-dependent), OFF-WINDOW
  RESPONSES ARE NOT (0.999+ in every E24/X2 run). The
  curriculum signal exists exactly while the stimulus exists and
  vanishes within 500 ms of its offset.
- D-4 PERSISTENT STATE IS A SEED-LOTTERY. V2.1: margins at fixed
  curriculum span 0-to-9,961 across seeds (E24); V2.2: latch
  sets are 0–2 neurons, seed-dominated (within-seed Jaccard
  0.174 vs cross-seed 0.022), decode at chance (F1). Two
  independent carrier mechanisms failed the SAME way:
  the identity of the persistent state is set by which neuron
  the seed's wiring lottery favors, not by the curriculum.
- D-5 THE MISSING CAPABILITY IS THE WRITE/MAINTAIN SEPARATION:
  the network can form sensory-specific structure (D-1) but
  cannot hold two structures simultaneously without one
  erasing the other (D-2), and its persistent intrinsic state
  (u, latch) does not inherit the sensory specificity that
  exists one synapse upstream (D-3/D-4: u's winner is
  margin-lottery, not curriculum-coded).

## 2. The four candidate deficits, adjudicated

### (a) Insufficient persistent state — WEAKENED by V2.2
- For: E18–E21 (<50 ms capacity); V2.1's carrier carries no
  per-trial code (X2).
- Against: V2.1 already sustains activity indefinitely (Stage B);
  V2.2 added a second persistent degree of freedom and it
  changed nothing (chance decoding both). Adding MORE state of
  the same kind is now evidence-contraindicated: two independent
  persistent-state mechanisms failed identically (D-4).

### (b) Insufficient sensory-specific organization — REFUTED as primary
- For: E3b's original reading (binding problem representational).
- Against: today's wdiv analysis (D-1) — organization is
  excellent at the afferent layer under every drive; E8
  cross-seed replication of separation. The substrate
  ORGANIZES; it cannot RETAIN coexisting organizations (D-2),
  which is a different deficit.

### (c) Insufficient population-level redundancy — PLAUSIBLE CONTRIBUTOR, NOT PRIMARY
- For: winner-take-all everywhere (D-2/D-4); 1–2-bit codes.
- Against: E24's il arms held 28–32 vs 14–17 neuron cohorts
  DURING the drive — the population HAS redundancy available;
  it is discarded at offset (D-3) and competed away under
  blocking (D-2). Redundancy is present upstream, absent
  downstream — a routing/budget problem, not a count problem.

### (d) Insufficient writable/competitive memory — STRONGEST
- For: D-2 (single-budget exclusion of coexisting traces);
  D-4 (twice-replicated seed-lottery in persistent state);
  D-3 (signal dies at offset precisely when a write would need
  to happen); E12/E15 re-anchoring (new block passively
  overwrites old — the read side of the same coin);
  V2.2-F5 (frozen reset default self-cancelling).
- Against: V2.2's specific latch implementation failed for
  placement reasons (theta above typical u) — its falsification
  is about THAT constant, partially shielding the general
  writable-memory hypothesis (FO-1). But two direct attempts
  (V2.1 margin, V2.2 latch) both landed on seed-dominated
  outcomes, which the placement story alone does not explain.

### (e) OTHER: competitive exclusion at the budget layer (M2)
- Not on the user's list, but the evidence points here as the
  MECHANISM of (d): M2 normalization is a shared resource
  (t_e = 0.8 total) across ALL of a post-synaptic neuron's
  excitatory afferents. Two curricula cannot both be written
  because they draw from one budget. E16 showed M2-off
  collapses separation entirely — so it cannot simply be
  removed; it must be partitioned or duplicated, not deleted.

## 3. Competing next-step hypotheses

- **H-A (budget partitioning / trace multiplexing)**: give
  coexisting traces non-shared normalization (per-source-class
  budgets, or eligibility-tagged budgets) so bac keeps BOTH the
  A- and C-afferent structures that il provably maintains.
  Strongest for: D-2 collapse is exactly what this predicts and
  nothing else on the list predicts it.
  Against: adds a second-order rule; risks E16-style separation
  collapse if partitioning weakens global normalization.
- **H-B (sensory-specific projection of persistent state)**: make
  the persistent state inherit the afferent specificity — e.g.
  u's write-path gated by the neuron's own afferent activity
  (still strictly local). Strongest for: D-3/D-4 gap (u is
  curriculum-blind one synapse away from curriculum-coded
  structure).
  Against: V2.2 just failed at the write side; this shares the
  write-side risk; also needs the two traces to coexist first
  (H-A prerequisite in blocked arms).
- **H-C (population redundancy via distributed code)**: force
  many-neuron codes (e.g., winner-share caps, lateral
  spread). For: D-4's 1–2-bit codes. Against: redundancy exists
  upstream and is discarded (D-3); treating the symptom.
- **H-D (structural growth / Y3)**: U3/E4 birth wired away from
  co-active pool. For: fixes routing by ADDING neurons rather
  than re-budgeting; registry's standing U3. Against: it does
  not touch the offset-decay (D-3) or the shared-budget
  exclusion (D-2) directly — new neurons inherit the same M2;
  highest blast radius; last unexecuted big lever.

## 4. Smallest distinguishing experiments

- **SDE-A (H-A, no mechanism change yet)**: read-only
  re-analysis — in the 30 E24 runs, measure whether the
  during-window separability (cos 0.30–0.89) correlates with the
  post-drive M / latch outcome across seeds. If high-separability
  seeds still show offset-death, the deficit is purely write/
  budget; if separability predicts retention, organization is
  the binding constraint. ZERO runs.
- **SDE-B (H-A probe, config-only)**: one exploratory arm with
  M2's t_e temporarily raised (config field exists: t_e = 0.8 →
  1.6) under bac/bca curricula — if the second-block collapse
  (D-2) softens, competitive exclusion at the budget is
  causally implicated. 12 runs, no code. (Exploratory only;
  t_e is a frozen V2 mechanism constant — this is a probe, not
  a proposal.)
- **SDE-C (H-D scout)**: birth_trigger exists but is "none" in
  every config of this era; a config-only arm with an existing
  trigger enabled (if one is implemented) — check structural.rs
  first; if no non-none trigger is implemented, SDE-C is code
  work and is NOT the smallest experiment.

## 5. Recommendation

**Primary: H-A (writable coexisting traces via budget
partitioning), gated by SDE-A now and SDE-B as the first
exploratory probe.** Rationale: it is the only hypothesis that
explains the full triad (D-2 collapse, D-3 offset-death given
D-1 organization, twice-replicated seed-lottery in persistent
state as the competition's residue). H-B is the natural SECOND
stage (once two traces coexist, give persistence their
specificity). H-C treats a symptom. H-D remains the big untried
lever but is not indicated FIRST by this evidence — it should be
re-evaluated after H-A's probe, and only if routing (not budget)
proves binding.

STOP per mandate. No mechanism implemented; no records modified.


---

## 6. SDE-A executed immediately (read-only, zero new runs)

Correlation of drive-end afferent cohort balance (min/max of
A-dom vs C-dom counts, wdiv) with post-drive M across the 30 E24
runs: rho = -0.113 overall, +0.098 blocked-arms-only. **No
relationship.** The best-organized runs (il arms, balance 0.48-
0.88) do not retain more; the collapsed blocked runs retain as
much as the balanced ones. SDE-A outcome: **retention is not
limited by how well-organized the drive-end afferent structure
is** — organization (present) and retention (absent) are
independent failure axes, consistent with D-3/D-4: the signal
dies at offset regardless of upstream structure quality.
This sharpens the diagnosis: the missing capability is at the
WRITE/MAINTAIN transition (offset-time consolidation), not at
the sensory-organization layer and not ameliorated by it.

## 7. Recommendation (unchanged, now evidence-backed)

H-A (writable coexisting traces) remains primary; SDE-B (t_e
1.6 probe on bac/bca, config-only, 12 exploratory runs) is the
first causal probe of the budget-exclusion mechanism. SDE-C
feasibility checked: structural.rs implements non-none birth
triggers (HomeostaticSaturation, PersistentError exist as
BirthTrigger impls) — a config-only Y3 scout is possible later
if routing proves binding.


---

## 8. SDE-B EXECUTED (12 config-only runs, exploratory, no E-number)

Design: t_e {0.8, 1.6} x {bac, bca} x 6 pre-declared seeds;
otherwise frozen e24 substrate (V2.1 null — no latch, no V2.2).
Primary question: does doubling the shared excitatory budget
reduce second-block overwrite? All 12 runs preserved (incl. one
runaway abort: te1.6-bca-s424242). Instruments: v22wdiv,
e24_endpoint.

### Results (within-seed paired, n=6)

COEXISTENCE (balance fraction = balanced-neuron fraction; and
first-block cohort survivors after the second block):
- te0.8: bac bal 0.079+-0.060, 4.2 first-block survivors;
  bca 0.071+-0.042, 11.0.
- te1.6: bac 0.054+-0.028, 5.5; bca 0.045+-0.058, 7.2.
- Coexistence DID NOT improve — balance fraction slightly
  DECREASED at te1.6 in both curricula. Doubling the budget does
  not let two traces coexist.

BUDGET OCCUPANCY: plastic input synapse count rose ~45-25%
(112->163 bac; 214->266 bca) — the extra budget is consumed by
the DOMINANT trace's expansion, not shared.

RETENTION (M, paired within seed):
- bac: te1.6 > te0.8 in 6/6 seeds (median delta +0.96 log10).
- bca: 6/6 (median +2.93; one te1.6 abort = +inf rank).
- Retention INCREASED uniformly — but by amplifying the
  single-trace outcome, not by preserving both.

DOWNSTREAM: te1.6 shifts regimes upward (more pacemakers, one
runaway abort) — consistent with M2's known role of setting the
separation regime (E16); more budget = stronger winner.

### Interpretation (separate from observations)

**SDE-B does NOT support H-A as formulated** ("budget
partitioning will enable coexistence"): giving the network more
total excitatory capacity does not soften second-block overwrite
— the dominant trace absorbs the surplus and the subordinate
trace is still erased. The write/maintain failure is NOT a
resource-scarcity problem; it is a COMPETITION problem (the
normalization rule itself reallocates, regardless of pool size).

The uniform retention increase at te1.6 is a capacity effect on
the single surviving trace (stronger winner), orthogonal to
coexistence — consistent with E16's M2-off result (no
normalization -> no separation) bracketing the other side.

### What remains unexplained in the write/maintain transition

The erase mechanism itself: WHY does the second block's
plasticity decrement the first block's afferents? Candidates
not yet discriminated: (i) STDP decay/renormalization real-
location (shared-budget is one implementation; the probe shows
pool size is not the lever, but the RULE may still be), (ii)
silent-synapse decay (silence_w=0.02 after silence_ticks=60000
— unused during drive; unlikely), (iii) pre-synaptic rate
competition (A- and C-channels firing into the same post-
neurons; second block's higher recent correlation wins the
correlation-based rule outright). The evidence now points at
(iii) or (i-as-rule-not-pool): competition is active, total-
capacity-independent.

### Follow-up (smallest causal experiment, NOT executed)

**SDE-C2 — decay-isolation probe**: one exploratory arm with
plasticity decay set to 0 (decay = 1e-6 -> 0.0 exists as
config) under bac/bca, 6 seeds = 12 runs, config-only. If
second-block erase persists with decay off, the erase is
active reallocation (STDP LTP/LTD competition) — pointing to
the RULE, and the architectural fix would be eligibility-
tagged or per-trace-partitioned plasticity (H-A refined). If
erase vanishes, passive decay is the eraser and the fix is a
per-synapse maintenance signal (closer to H-B's write path).

STOP per mandate. No mechanism introduced; no tuning after
observation; no E-number; records untouched.


---

## 9. SDE-C2 EXECUTED (12 config-only runs, exploratory, no E-number)

Design: plasticity decay {1e-6 (baseline), 0.0} x {bac, bca} x 6
seeds; all else frozen e24 (t_e=0.8, V2.1 null). All 12 runs
preserved (one abort: d0-bca-s9001, runaway during drive).

### Primary result: overwrite PERSISTS but is substantially WEAKENED — passive decay is a major eraser component, not the whole story

First-block survivors (paired within seed, decay-off vs on):
- bac: decay-off > decay-on in **6/6 seeds** (A-cohort 4.2 ->
  15.5 mean; e.g. s123456 3->23, s20260912 5->23).
- bca: 2/6 up, 2/6 down (11.0 -> 18.7 mean but s9001 aborted
  mid-drive with 0; excluding it, mixed).
The absolute picture (bac, decay-off, drive end): BOTH cohorts
present in every run (A 8-23 / C 20-38) vs decay-on's near-total
erasure (A 2-6). Coexistence fraction (balanced neurons) stays
low (~0.07) because both cohorts grow without balancing — the
dominant-trace skew remains — but the first trace is no longer
DELETED.

### Mechanism resolution (from existing telemetry)

- Tick-resolved cohorts (first-block end t=44k vs drive end
  t=85k, decay-off bac): IDENTICAL rows — no further structural
  change after the first block completes under decay=0? No —
  identical because the analysis used the last snapshot <= t;
  at 44k the second block has not run; the equality of 44k and
  85k rows shows the A-cohort established by block 1 is still
  present after block 2 (persistence, not stasis).
- Plastic synapse count: 267 (d0-bac) vs 112 (d1-bac) — with
  decay off, more afferents survive above the wdiv dominance
  threshold (weak synapses no longer bleed to w_min).
- M2 is STILL ACTIVE in both arms (t_e unchanged): its
  reallocation is what keeps the ~0.07 coexistence skew — the
  residual competition after passive decay is removed.
- Therefore the erase decomposes: **passive weight decay (major,
  ~70-80% of first-block erasure in bac) + M2/STDP active
  competition (residual skew)**. The a_minus > a_plus asymmetry
  (0.0053 vs 0.005) also contributes a slow active LTD floor.

### Retention (M, descriptive)

Mixed: d0-bac [3.50, 3.86, 1.90, 3.49, 0.00, 3.47] vs d1-bac
[3.45, 2.70, 3.65, 2.37, 2.89, 3.41]; d0-bca has one abort
(+inf) and one 0.00. No uniform retention gain — coexistence
does not translate into more endogenous activity (consistent
with E24: retention is margin-lottery, downstream of structure).

### Verdict on the primary question

Second-block overwrite is **decay-mediated to first order**:
disabling the (tiny, 1e-6/tick) passive decay preserves the
first block's afferent cohort through the second block in 6/6
bac seeds. The residual imbalance is M2-normalization/STDP
competition (active), which no decay setting removes.

### Smallest follow-up (defined, NOT executed)

**SDE-D — maintenance-vs-competition split**: decay back at
1e-6, but with M2's reallocation disabled for input-class
synapses only if a config exists (check: M2 applies to all
excitatory afferents; no input-only exemption exists in config)
=> NOT config-only. Alternative config-only split: a_minus ->
0.005 (symmetric STDP) x decay {1e-6, 0.0}: if symmetric-STDP +
decay-on still erases, the passive term is confirmed sole major
eraser; if it preserves like decay-off did, the a_minus>a_plus
asymmetry is the active eraser. 12 runs, config-only.

STOP per mandate. No mechanism introduced; no tuning; no
E-number; records untouched.


---

## 10. SDE-D EXECUTED (full 2x2x2x6 = 48 config-only runs, exploratory, no E-number)

The mandated 12 conditions (a_minus x decay x curriculum) with
seeds fixed = the full crossing; both am53 cells replicate the
SDE-C2 arms under one binary (provenance hygiene). 6/48 runs
aborted on runaway (all during drive): am50-d1: bac 9001/31337,
bca 20260912/31337; am53-d0-bca 9001 (same cell as SDE-C2);
am50-d0: bac 31337, bca 20260912. All preserved. NOTE: symmetric
STDP is destabilizing (4 of the 6 aborts are am50 cells) —
removing the LTD excess trades erasure for runaway risk.

### First-block survivors (per-seed; AB = aborted)

| cell | bac | mean | bca | mean | aborts |
|---|---|---|---|---|---|
| am53-d1 (baseline) | 5,4,5,3,2,6 | 4.2 | 12,14,8,10,12,10 | 11.0 | 0 |
| am50-d1 | 10,8,AB,12,3,AB | 8.2 | AB,8,8,11,10,AB | 9.2 | 4 |
| am53-d0 | 23,18,10,23,8,11 | 15.5 | 12,11,AB,14,15,9 | 12.2 | 1 |
| am50-d0 | 8,8,10,17,6,AB | 9.8 | AB,16,21,16,32,13 | 19.6 | 1 |

### Paired asymmetry contrasts (within seed, non-aborted pairs)

- decay ON, bac: am50 > am53 in 4/4 (5->10, 4->8, 3->12, 2->3)
  — the asymmetry contributes to first-block erasure when
  passive decay is active.
- decay ON, bca: 1 up / 2 down / 1 tie — no consistent effect.
- decay OFF, bac: am50 < am53 in 4/5 — REVERSED.
- decay OFF, bca: am50 > am53 in 4/4 — direction present.
- Coexistence balance fraction: ~0.07-0.09 in ALL cells
  (including both am50 cells): the asymmetry does NOT move the
  dominant-trace skew. M2 reallocation remains the residual
  competitor.

### Interpretation (narrow, per the wording constraint)

- **Passive decay is the dominant, consistent eraser** (SDE-C2;
  confirmed here: am53-d0 bac 15.5 vs am53-d1 4.2; every decay
  contrast same-direction).
- **The a_minus > a_plus asymmetry is a SECONDARY, interaction-
  dependent contributor, not a stand-alone eraser**: its removal
  helps bac under decay-on (4/4) but reverses under decay-off
  (4/5) and is inconsistent in bca. With decay off, symmetric
  STDP lets the second block potentiate MORE freely (no LTD
  brake), which can overwrite the first block by LTP crowding —
  opposite sign of effect.
- **The data do NOT establish any sole remaining eraser.** After
  decay is controlled, residual competition shows: (i) M2
  reallocation (balance fraction unmoved in all 4 cells — the
  only mechanism present in every condition that predicts a
  persistent dominant-trace skew), and (ii) symmetric-STDP LTP
  crowding (sign-flipped asymmetry effect). Cannot rank these
  two from this design; M2's contribution is inferred from the
  invariance of the skew, not from a direct M2 manipulation
  (none is config-expressible).
- **Instability cost**: symmetric STDP aborts 4/24 cells vs 1/24
  asymmetric — the asymmetry is load-bearing for stability
  (consistent with E2-era runaway history: the LTD excess is a
  brake).

### Dominant-remaining-mechanism assessment (for the intervention mandate)

The evidence supports M2 normalization/reallocation as the
persister of residual competition (skew invariance across all
four cells), but SDE-D does not isolate it causally. The
smallest CAUSAL architectural intervention targeting it:

**Proposed (NOT implemented): trace-partitioned normalization —
per-source-cohort excitatory budgets.** Local rule: each
post-synaptic neuron maintains its M2 budget separately for
afferents whose pre-synaptic channels were last co-active with
distinct drive epochs (implementable as an eligibility tag set
per synapse at write time — still strictly local: tag =
plasticity-timestamp bucket, no global identity, no labels).
Prediction: blocked curricula retain both cohorts at balance
fraction >> 0.07 while single-curriculum runs are unchanged
(identity at 1 bucket). Falsifier: if partitioning the budget
reproduces E16's separation collapse (M2-off phenotype), the
shared budget is load-bearing for organization and the fix must
instead be a maintenance signal (per-synapse activity-gated
decay exemption). Smallest distinguishing experiment for the
two: the proposed partition run vs an activity-gated decay-exempt
run, 6 seeds x bac/bca, endpoints = cohort coexistence + E16
separation metric.

STOP per mandate. Nothing implemented; no tuning beyond the
mandated cells; no E-number; records untouched.


---

## 11. V2.3 candidate design (NOT implemented): capacity-matched trace-partitioned M2

Per mandate, the smallest causal test of shared-M2 causality.
Design doc: docs/v2_3-design.md. Key honesty point recorded
there: naive partitioning confounds partition with capacity
(t_e x n_buckets); the design CAPACITY-MATCHES (per-bucket T =
t_e / n_populated) so total excitatory mass never exceeds
baseline — and SDE-B already measured that capacity alone does
nothing, so the partition arm isolates SHARING. No smaller
clean intervention exists: config-expressible M2 manipulations
are only disable (E16, organization destroyed — measured) and
t_e scale (SDE-B, capacity — measured). Bucket = write-epoch
tag from the synapse's own LTP history + per-neuron epoch
counter on the existing structural window clock (no labels, no
input identity; the one global element — the window clock — is
E1-frozen substrate). 3-arm comparison reuses SDE-D anchors
(A1=am53-d1, A2=am53-d0 already run): the new contrast is
partition+no-decay vs shared+no-decay, 12 fresh runs + null
gate. Endpoints: coexistence (paired sign), E16 separation
non-collapse (<=0.85 in >=4/6), stability, byte-identity null
(n_buckets=1). Failure modes F-P1..F-P5 pre-specified,
including the tag-churn and bucket-monoculture diagnostics.
STOP: design only; nothing implemented/executed; no E-number.
