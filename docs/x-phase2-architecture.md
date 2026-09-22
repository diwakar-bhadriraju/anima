# Phase II architecture study — multiple persistent representations under sequential experience

Status: READ-ONLY ARCHITECTURE STUDY, 2026-09-22. Phase I is closed and
immutable (paper: `paper/`; records: `docs/anima-clla-*`, `docs/x-clla-*`,
`docs/x-synmem-audit.md`, `docs/v2_1-info-audit.md`, commit lineage).
No implementation, no experiments, no tuning, no E-number.

Prior architecture reviews this study builds on and extends:
- `docs/x-assembly-arch.md` (candidate classes A–E → CLLA; CLLA executed
  and closed);
- `docs/x-memtopo-review.md` (the deficit framed as ACCESS, not storage;
  channel-addressed read proposed — Phase II relaxes that proposal's
  channel-identity key per the no-metadata mandate, see §4/§8);
- `docs/x-segarch.md` (episode segmentation → persistent write;
  buffer+commit with population edge coverage ≈ 1.0);
- the allocation/first-exposure sequence records and the two recruitment
  gain registrations.

---

## 1. Phase II research question

**Formal target capability:**

    sequential experience:   A learned, C learned, A absent, C absent,
                             A presented again
    required outcome:        the A-specific internal state re-expresses
                             preferentially over C, from the organism's
                             own dynamics.

**Operational retrieval definition (first-class requirement):**

    learn A (≥ K_A presentations)
    learn C (≥ K_C presentations)
    remove both stimuli (silence ≥ 2 s)
    present A
    measure whether the previously established A-specific internal
    configuration re-emerges preferentially:
        ρ(A) = cos(v_A_late, v_A_ref) − cos(v_A_late, v_C_ref) ≥ δ
    with v_A_ref / v_C_ref the during-learning response references and
    δ pre-registered (design in §15; not assumed here).

Constraints on the solution: no external episode boundary, no A/C label,
no reward, no trial counter, no global similarity search, no hard-coded
slot identity. The memory representation must emerge from local
dynamics. Explicitly NOT allowed: K copies of the network; externally
assigned memory slots; curriculum/patter structure as architectural
metadata (channel-group membership, order, or count).

**Phase II-A question (the smallest first question):** under BLOCKED
sequential experience, can any resource-structural intervention that
respects the constraints restore *formation* of the second
representation (Phase I S1 criterion) while preserving the first —
before any retrieval machinery is added?

## 2. Phase I evidence constraints

The design space is bounded by the twelve Phase I findings (paper
§14/ladder) plus the following load-bearing measurements:

1. Afferent structure can emerge; per-presentation activity preserves
   sensory information during the window (during-cosine 0.30–0.89;
   off-window 0.999+).
2. Long-horizon accumulation destroys episode identity (adjacent-state
   cosine 0.972 → 0.990 under alternation; blocked order overwrites).
3. `u` is not a sufficient primary carrier (saturation, pacemaker
   degeneracy, retracted Stage-C).
4. Synaptic structure is the persistent substrate (G6: structure
   persists, activity decays 3.5–7×; structure is the store).
5. Bounded protection stabilizes; unbounded protection runaways (F3/F4).
6. Alternating coexistence works; blocked-order allocation starves.
7. The starvation site is the WORKING SUBSTRATE, churned during the
   first block before the novel pattern's first exposure (rule record:
   wC 197→77; fec: 29→11 during block 2; passive decay = dominant
   eraser, SDE-C2 6/6).
8. First-exposure binding is possible and fast once substrate survives
   (fec: first permanence 2.5 s; supply 41→61).
9. Novel-pattern recruitment is a separate bottleneck: 17× post
   collapse (1,697 → 101 spikes), 7.7× afferent-drive deficit
   (1.57 vs 11.98/neuron), IL control symmetric (posts 1.00; permanence
   98 ≈ 95).
10. A membrane-side, input-proportional recruitment gain cannot fix a
    churned substrate (i_boost ≤ k_g·I_W bound; rg8/rg8c), and at full
    gate it destabilizes already-strong representations (rg8 4b: block-1
    pres-1 391–449 Hz). The threshold-completion cap (rg8c) repairs the
    aggregate operating point but not first-exposure rate.
11. Fast, dynamics-level competition does not separate readouts (E3b
    lateral inhibition; cross-cosine 0.673 → 0.740, selectivity
    down).
12. Capacity without structure does not help (SDE-B: doubling t_e feeds
    the dominant trace).
13. Budget-structural interventions are the ONLY measured lever that
    moved blocked coexistence in the predicted direction (V2.3
    trace-partitioned M2: bca rescue 6/6).
14. Structural growth regresses in every tested shape (E4 series: sink
    shape-locked at ~35–43 births t≈540 s; reciprocal runaway at 32 s;
    low-fan-in inert) — growth as CAPACITY is closed under the tested
    substrate conditions.
15. Retrieval by re-presentation exists at the trace level (REV1:
    zero plasticity events, ≤3% weight change, readout flip) but is
    NOT established at the level Phase II requires (G8; adjacent-state
    identity under alternation).

## 3. Design requirements derived from the evidence

- R1 **Retrieval-first**: ``different weights'' is not retrieval; only
  the operational test of §1 counts.
- R2 **No program metadata**: the organism may use only (a) its own
  per-neuron/per-synapse state, (b) the input channel statistics it
  experiences (co-occurrence is environment, not curriculum), (c)
  substrate constants. It may NOT use channel-group membership, order,
  presentation counts, or phase/epoch parity as a memory key.
- R3 **No global engine**: no population novelty/similarity machine; no
  supervised readout; no WTA arbitration beyond what the substrate's
  M2 budget competition already provides.
- R4 **Explicit finite cost per degree of freedom**: every new state
  has a budget; exhaustion is a reportable outcome, not a failure to
  be hidden.
- R5 **Locality**: decisions at the synapse/neuron level.
- R6 **No persistent-firing store** (G6): memory lives in synaptic
  structure; reactivation is transient forward dynamics. Persistent
  firing is neither required nor used.
- R7 **Stability preserved**: any Phase II candidate carries the
  identity-gate + bounded-protection discipline; the rg8 lessons bind
  any gain component (cap at threshold completion; never full-gate a
  strong, already-recruited representation).
- R8 **Developmental**: structure emerges from experience; pre-
  partitioned resources (fixed slot tables, fixed channel keys) are
  disallowed as architecture answers (§2 finding 2 / mandate).

## 4. The central architectural question: dimensions or plasticity?

Phase I now permits a sharper answer than either framing alone:

- SDE-B removed ``more capacity'' (double t_e → surplus feeds dominant
  trace). rg8/rg8c removed ``louder plasticity'' for the blocked case
  (a membrane-side boost cannot amplify a churned substrate, and at
  strength it destabilizes). V2.3 positively attributed the only
  predicted-direction movement to a RESOURCE-STRUCTURAL change —
  partitioning one per-neuron budget into two independent targets.
- The V2.3 key (time-epoch parity) is program metadata and cannot
  generalize; but the RESOURCE PRINCIPLE survives: the shared additive
  budget is the interference medium, and a per-neuron budget that can
  be committed along an EXPERIENCE-DERIVED axis is the natural
  ``more dimensions'' step (paper §9/§16; synmem §7/§11).

Therefore: **the evidence favors ``more representational dimensions''
in the resource sense (structured budgets), provided the dimension key
is learned from experience rather than injected.** The central Phase II
A question is whether such keys exist locally (§8).

**Can a single neuron legitimately participate in multiple memories?**
Phase I's strongest coexistence data (alternation; G3: both channel
groups retained in one neuron's budget at 0.5–0.6 balance, selectivity
halved 0.91→0.46; V2.3: two epoch partitions on one neuron) say YES in
persistence terms: a neuron can hold multiple protected configurations
in the same weight matrix. Whether it can EXPRESS them selectively is
unmeasured — the entire Phase II-B problem. This reframes the question:
not ``separate neurons per memory?'' but ``what makes expression
selective given coexistence?'' The candidates differ exactly here.

## 5. Candidate A — competing assemblies

Representation: a memory = a subset of neurons that fires together for
the pattern and is wired together (recurrent + afferent), selected by
local competition for recruitment.

| # | Question | Answer |
|---|---|---|
| 1 | Representation | Sparse co-firing neuron subset (assembly) + its afferent/recurrent wiring |
| 2 | Allocation | Slow competition for membership: neurons recruited to the assembly of currently-firing cohorts; requires a locality-conformant recruitment signal — Phase I has none that works fast (E3b) and none slow (growth era regressed) |
| 3 | Consolidation | Co-activity permanence as today (M3) + bounded protection (CLLA) |
| 4 | Protection | Per-assembly protected mass (CLLA machinery, per-neuron cap) |
| 5 | Interference | ONLY if assemblies are disjoint/sparse enough; the Phase I record measures overlap instead (within-pattern 0.97–0.99 stable but cross-pattern 0.70–0.81; E3b could not de-overlap) |
| 6 | Retrieval | Re-presentation reactivates the assembly when afferent mass survives (REV1 precedent); selective only if assemblies don't share the readout pool |
| 7 | Capacity | Total neurons ÷ effective assembly size × per-neuron budget; bounded by b_e = 40 afferents/neuron and synapse cap 2000 |
| 8 | Locality | Membership decision needs only own activity + own budget (NO lateral comparison — E3b's inhibition is exactly the forbidden/ineffective form) |
| 9 | Novelty | Unexplained-drive fraction (1−R) — the validated CLLA rule signal; warrants allocation to an un-claimed assembly |
| 10 | Recruitment | THE open problem (Phase II recruitment must solve the 17×/7.7× deficits on a substrate that SURVIVES — see rg8 lessons) |
| 11 | Reuse | Partial: a neuron's budget can host several assemblies' weights (V2.3/G3 show the persistence is possible) — but selective expression unproven |
| 12 | Saturation | All neurons committed to existing assemblies ⇒ new pattern starves (measured signature: E24 primacy, allocation-rule block-1 capture) |
| 13 | Forgetting | Passive decay + M4 of un-reinforced assembly wiring (SDE-C2: decay = dominant eraser) |
| 14 | Collision | Near-similar patterns merge onto the shared assembly (D-composition adjacency: cos(v_D, v_A+v_C) 0.82–0.91) |
| 15 | Closed loop | Expression = forward dynamics; outputs join the core (E24 S4) — compatible |

Explains: an object-level account of memories; why alternation (always
co-driven) never prunes either assembly.
Leaves unresolved: the membership-selection machinery (all Phase I
competition attempts failed or regressed); the recruitment step;
selective re-expression under sharing.
Distinguishing experiment: assembly-disjointness probe — measure
whether two co-trained assemblies share neurons beyond chance after a
slow local competition gate; if sharing is irreducible, A alone cannot
retrieve selectively.

## 6. Candidate B — synaptic compartments / addressed sub-budgets

Representation: a memory = the pattern of per-neuron sub-budget
occupancy across the population; each neuron hosts multiple budgets.

| # | Question | Answer |
|---|---|---|
| 1 | Representation | Per-neuron sub-budget (compartment) occupancy vectors; one neuron = several compartments |
| 2 | Allocation | First exposure must choose WHICH compartment — the hard point (x-assembly-arch §2A: fixed slot selectors are labels; learned assignment is the unsolved part) |
| 3 | Consolidation | Per-compartment permanence + bounded protection (V2.3-style target per compartment) |
| 4 | Protection | Per-compartment cap; total ≤ t_e (V2.3 invariant) |
| 5 | Interference | By construction — partitions never mix (V2.3 bca 6/6 precedent) |
| 6 | Retrieval | Re-presentation reads the compartment the afferents write to — NECESSARILY selective IF the key is the afferent structure itself; a time/epoch key gives no selective read |
| 7 | Capacity | K compartments × N neurons × (t_e/K per neuron); exhausts at K × saturation |
| 8 | Locality | Partition key must be locally available per synapse: candidate keys — pre-channel identity (metadata: FORBIDDEN by R2), write-time co-activity cluster (experience-derived: allowed), epoch parity (metadata: forbidden) |
| 9 | Novelty | (1−R) per compartment: an unexplained compartment is allocatable |
| 10 | Recruitment | Unchanged bottleneck: partition alone does not grow the churned substrate; must combine with substrate-survival/recruitment work |
| 11 | Reuse | YES by definition — the whole point; G3/V2.3 persistence evidence |
| 12 | Saturation | K full ⇒ no new memory (reportable F3b-style); fragmentation: one pattern spread over compartments or two merged in one (x-assembly-arch §2A) |
| 13 | Forgetting | Per-compartment decay + eviction under budget pressure |
| 14 | Collision | Two afferently similar memories write the same compartment ⇒ merge (needs the novelty threshold) |
| 15 | Closed loop | Compatible (compartments are write-side; expression unchanged) |

Explains: why V2.3 worked blocked; why SDE-B failed (no structure);
the persistence half of coexistence.
Leaves unresolved: the KEY (metadata keys are forbidden; learned keys
are the open design point); retrieval remains to be measured even with
a good key; recruitment unchanged.
Distinguishing experiment: is there ANY locally learnable key that both
separates compartments and retrieves selectively (Phase II-A/B
registration below is built to answer this).

## 7. Candidate C — local synaptic tagging + consolidation

Representation: a memory = the set of consolidated (tagged) synapses
whose tags co-originated from one experience stream.

| # | Question | Answer |
|---|---|---|
| 1 | Representation | Tagged/consolidated synapse set; tag = local per-synapse eligibility state (M3 candidates + consolidated flag are this machinery already) |
| 2 | Allocation | First-exposure binding (fe/fec machinery: supply 41→61; 2.5 s permanence) — WORKS when substrate survives |
| 3 | Consolidation | Co-activity permanence → protected class (exists) |
| 4 | Protection | Bounded protected mass (exists; V1–V4) |
| 5 | Interference | Tags are per-synapse; interference returns at the BUDGET level (M2) unless compartments also exist |
| 6 | Retrieval | Quietly absent: tagging says nothing about selective re-expression — C alone fails the operational test |
| 7 | Capacity | Candidate slots (6/neuron) + protected budget; both measured |
| 8 | Locality | Fully local (already implemented) |
| 9 | Novelty | The reserve record's verdict applies: ``ever-coactive'' can only tag what was already active; a genuinely novel pattern has nothing tagged (0 reserved C-candidates at onset) — tagging needs recruitment first |
| 10 | Recruitment | Unchanged bottleneck (same as B) |
| 11 | Reuse | Yes at the neuron level (slots are per-neuron) |
| 12 | Saturation | c_slots full ⇒ eviction ladder (theta_die floor; measured) |
| 13 | Forgetting | Tag decay (theta_die 0.005, decay 0.99/window) — measured mechanics |
| 14 | Collision | Two patterns co-activating the same window tag the same synapses ⇒ merge |
| 15 | Closed loop | Compatible |

Explains: allocation+consolidation — the parts Phase I already measured
working.
Leaves unresolved: retrieval (untouched), interference at the budget
level, recruitment.
Distinguishing experiment: tagging is already executed (reserve/fe/fec)
— Phase II uses C as the shared allocation/consolidation layer of the
other candidates, NOT as a standalone architecture. No further
standalone experiment warranted by the record.

## 8. Candidate D — experience-partitioned commitment substrate (``context tracks'')

The fundamentally different option, constructed from Phase I evidence
rather than biological precedent. Its three design moves each answer a
measured failure:

1. **Budget dimensions with an experience-derived key.** Per neuron,
   K context tracks, each with a sub-budget (total ≤ t_e, V2.3
   invariant). A track's key is NOT injected: it is a soft cluster of
   the neuron's OWN afferent usage — the neuron observes which input
   channels co-fire in its dendritic history and maintains K internal
   prototypes (locally computable; co-occurrence statistics are
   environment data, R2-compliant). A synapse is routed to the track
   whose prototype best matches the co-activity at write time (its
   pre-spike context). This converts V2.3's epoch key (metadata) into
   a learned key (experience) — the single change Phase I's regression
   history points to (SDE-B × V2.3 ⇒ structure, not capacity, with an
   emergent key).
2. **Track-locked bounded protection + track-conditional novelty.**
   CLLA protection per track (each track capped at p_max·t_e/K);
   novelty = unexplained fraction PER TRACK (1−R_t) — the validated
   allocation-rule signal restricted to the track's afferent share.
   A track that explains its drive well is write-resistant; an
   unexplained track is writable. This keeps allocation local and
   label-free while adding the dimension Phase I says is missing.
3. **Recruitment only within an unexplained track, capped.**
   The rg8/rg8c registrations bound any gain: input-proportional
   (i_boost ≤ k_g·I_W) and capped at threshold completion
   (max(0,(v_th−v)·τ_m/dt)), applied only while (1−R_t) > 0 and only
   on the track's working afferents. The cap's measured limitation
   (pres-1 rate-sustaining) is accepted and its purpose here is
   subordinate: recruitment support for FIRST formation, not
   maintenance.

| # | Question | Answer |
|---|---|---|
| 1 | Representation | A memory = a track's protected weight configuration across the population (sub-budget occupancy + wiring tag) |
| 2 | Allocation | First exposure binds into the most unexplained track via fe machinery (measured working); key learned, not assigned |
| 3 | Consolidation | Track-local permanence (M3 per track) + bounded protection (CLLA per track) |
| 4 | Protection | Per-track cap; total ≤ t_e — V2.3 invariant + CLLA V1–V4 |
| 5 | Interference | Budget-level: tracks do not share; measured precedent: epoch partition rescued blocked cohorts (V2.3); the open risk is key blur (below) |
| 6 | Retrieval | Re-presentation drives the afferents → activates the track that owns them → the PER-NEURON track readout is the sparse subset whose track matches — expression becomes track-selective by construction IF the key generalizes from write-time to read-time context (the II-B question) |
| 7 | Capacity | K × N × (t_e/K) explicitly; synonyms: fewer tracks than memories ⇒ forced merge (reportable); more tracks than the firing structure supports ⇒ dead tracks (fragmentation) |
| 8 | Locality | Prototype updates, track routing, (1−R_t), caps: all per-neuron/per-synapse |
| 9 | Novelty | (1−R_t) > threshold ⇒ writable (existing signal family); no global comparison |
| 10 | Recruitment | Capped unexplained-track gain (rg8c design) + fe binding; substrate survival still must come from allocation timing (a track's unused sub-budget is NOT churned by other tracks' activity — a structural property the rg8 age lacked) |
| 11 | Reuse | Yes — the architecture's point; K > 1 is the resource model, not a slot table |
| 12 | Saturation | All tracks protected ⇒ no writable track ⇒ F3b-style exhaustion report (not deadlock: forgetting, below, frees tracks) |
| 13 | Forgetting | Per-track passive decay + track re-claim: an unused track's prototype drifts toward current input statistics until its sub-budget is re-allocated (local, continuous; SDE-C2 decay is the engine) |
| 14 | Collision | Afferently similar memories converge on one track's prototype ⇒ merge within the track (bounded; the D-pattern composition case: cos(v_D, v_A+v_C) 0.82–0.91 adjacency suggests merges are graded, not catastrophic) |
| 15 | Closed loop | Expression unchanged (forward dynamics; outputs join the core, E24 S4); tracks are write-side only |

Failure modes: key blur (write-time context ≠ read-time context ⇒
misrouting); prototype collapse (one track absorbs K memories);
fragmentation (a memory split across tracks ⇒ diluted gains);
recruitment starvation INSIDE a track if its sub-budget is below the
writable floor (quantitative note: with t_e = 0.8 and K = 2, each
track's cap is 0.3 and working target ~0.4 — above the measured
single-pattern needs 0.24–0.29, but K = 4 drops the cap to 0.15 —
below single-pattern needs ⇒ K must be small or budgets asymmetric).

Explains: V2.3's partial success (structure with the wrong key);
the rg8/rg8c bindings (gain must be capped, unexplained-only, and
substrate-preserving); the alternation coexistence (tracks absorb
alternating writes without mixing); the X-review's ``writable memory''
framing (tracks are writable resources); why a first exposure is
allocatable while an exposure to an existing track is not ((1−R_t)).
Leaves unresolved: retrieval selectivity (II-B must measure it — no
Phase I datum exists either way); key-learned dimensions of freedom;
recruitment with sub-budgets.
Distinguishing experiment: the Phase II-A registration below.

## 9. The dimension-vs-plasticity verdict (evidence-based)

- Plasticity-strengthening attempts with deterministic outcomes:
  E3 adaptation (no separation change), E3b inhibition (worse),
  rg8 gain (operating-point failure; hypothesis bounded by substrate
  absence), E4 growth (regressions). 
- Resource-structural attempts: V2.3 partition (only predicted-
  direction result), SDE-B (capacity without structure: null),
  CLLA boundaries (stability, then allocation frontier).
- Conclusion: the Phase I record supports treating the MINIMAL
  next intervention as a resource-dimension change with an
  experience-derived key, with plasticity changes only as bounded
  recruitment support inside those dimensions. This is candidate D.
- Persistent firing: not necessary (G6) — no candidate uses it as a
  store; all store in weights and reactivate (R6).

## 10. Decision matrix (descriptive only — no ranking)

| Axis | A: assemblies | B: compartments | C: tagging | D: context tracks |
|---|---|---|---|---|
| Locality | partial (needs recruitment signal) | key-dependent | full | full |
| New state | membership/recruitment | K prototypes/track budgets | none (exists) | K prototypes + track budgets + per-track gain caps |
| New parameters | competition timescale, size | K, prototype rate | none | K, prototype rate, per-track caps (0 = identity) |
| Finite capacity | yes (neurons × budget) | yes (K × t_e) | yes (slots + P-cap) | yes (K × t_e; explicit) |
| Interference resistance | low unless disjoint | high (by construction) | budget-level only | high (track-level); key-blur risk |
| Allocation difficulty | HIGH (no working member-selection) | MEDIUM (the key problem) | LOW (measured) | MEDIUM (key is learned; fe machinery exists) |
| Retrieval difficulty | HIGH (shared pool) | UNMEASURED (key-dependent) | HIGH (no read side) | UNMEASURED (II-B target) |
| Phase I compatibility | low on dynamics (E3b), none on growth | V2.3 direct precedent; keys need re-derivation | measured working layer | V2.3 + rule + rg8c components all precedent |
| Falsifiability | clear (assembly overlap metric) | clear (S1 criterion + II-B ρ) | already falsified as standalone (S1 6/6) | clear (S1 criterion + II-B ρ) |

## 11. What each architecture explains / leaves unresolved / and the experiment that would distinguish

- **A** explains object-level wiring and why alternation retains; is
  silent on member selection and selective expression. Distinguisher:
  co-training overlap probe (does any local recruitment rule produce
  disjoint assemblies at Phase-I scale? — if not, A cannot retrieve
  selectively).
- **B** explains V2.3's rescue; the only free design point is the key.
  Distinguisher: II-A with a learned key vs. no key (Phase I S1
  criterion), then II-B.
- **C** explains allocation+consolidation; standalone falsified.
  Distinguishers exhausted by the record.
- **D** explains the same evidence as B plus the rg8/rg8c bindings and
  the novelty/recruitment constraints; its open point is exactly
  retrieval selectivity. Distinguisher: II-A (formation) then II-B
  (re-expression). **D and B are the same experiment with a different
  key policy** — the key policy being precisely the R2 boundary
  (experience-derived vs forbidden metadata); II-A is therefore the
  single experiment that separates ALL candidates: A-failures show as
  failed S1, B/D-failures show as failed II-B, C is the control arm.

## 12. Resource models (explicit, per candidate)

| Candidate | What consumes capacity | Maximum (Phase I numbers) | Exhaustion |
|---|---|---|---|
| A | assembly membership + wiring | ≤ 52 neurons / assembly size; ≤ 2000 synapses; b_e 40/neuron | new pattern starves (E24 primacy, rule record) |
| B | K sub-budgets/neuron | K × 52 × t_e/K = 41.6 total weight units | K full ⇒ F3b-style exhaustion; fragmentation on partial fill |
| C | candidate slots + protected mass | 6 slots/neuron; P ≤ 0.6/neuron | eviction ladder (theta_die floor); P-cap full ⇒ write blocked |
| D | K tracks × (sub-budget + prototype) | K × 52 × t_e/K; prototypes bounded (K vectors/neuron, e.g. 24 dims × K) | no writable track ⇒ reportable exhaustion; forgetting frees tracks |

Forgetting is available in every candidate via the measured engines
(passive decay 1e-6/tick — the dominant eraser; M4; per-track decay in
D) — none requires a new global rule.

## 13. Failure modes (cross-candidate)

- Key blur / misrouting (B/D) — write-time vs read-time context
  mismatch; measured analogue: G3 superposition when the store cannot
  discriminate recent episodes.
- Prototype collapse (D) — one track absorbs all memories under
  highly similar input statistics; bounded, reportable via track
  occupancy entropy.
- Fragmentation (B/D) — one memory spread over tracks; diluted
  per-track drive ⇒ recruitment starvation inside tracks (the
  fec 0.054-weight-growth symptom at track level).
- Sub-budget floor (B/D) — per-track cap below the writable floor
  (0.15 < single-pattern need 0.24–0.29 for K=4 ⇒ K ≤ 2 at Phase I
  scales).
- Retrieval collision (all) — two afferently similar memories; merge
  is graded (D-composition adjacency 0.82–0.91), not catastrophic.
- Commitment errors (C/D) — a novel exposure consolidates into the
  wrong track when prototypes are ambiguous.
- Operating-point overdrive (all, if gain is added) — the rg8 4b
  lesson: any gain must be threshold-completion-capped and
  unexplained-only, or the first-exposure rate sustains runaway-adjacent
  bursts.

## 14. Tradeoffs

- K (dimensions) vs writable floor: K = 2 is the only setting at
  Phase I scale giving both separation (two tracks, balanced budgets)
  and writable sub-budgets (cap 0.3, single-pattern need 0.24–0.29).
  K = 1 is today's substrate (control); K ≥ 3 risks sub-threshold
  tracks. This is an ARCHITECTURAL FLOOR argument, not tuning.
- Learned keys vs. plasticity: a learned key is a plasticity process
  (prototype drift); too fast ⇒ tracks follow noise (fragmentation);
  too slow ⇒ tracks lag the curriculum (capture by the first pattern —
  the E24 primacy signature). The prototype rate is the one new
  timescale of D and must be pre-registered, not fitted.
- Recruitment support vs. stability: the rg8c cap is the only
  measured-safe gain form; phase A of the ladder should NOT include
  gain at all — formation first, then recruitment support
  (ladder order, §16).

## 15. The smallest distinguishing experiment (Phase II-A; pre-registration sketch — NOT executed)

Question: does an experience-derived budget dimension restore
sequential formation that the substrate alone fails (S1 criterion)?

Arms (per seed, frozen Phase-I cell β=0.0046875/τ_s=5000; seeds
20260912/424242/9001; schedules bac/bca/il as in Phase I — the arm
table itself mirrors the Phase I capability matrix so every metric and
reference transfers):

1. control: committed clla-fe configs (already on disk; no new runs).
2. D-core: context tracks K=2 with learned prototypes, per-track
   bounded protection, NO recruitment gain (the minimal dimension
   change; identity-gated).
3. key-less partition active control (epoch key as in V2.3, archived
   machinery) — isolates ``partition'' from ``learned key''.

Endpoints (reuse Phase I bars so the comparison is literal):
- S1: second-block protected mass ≥ 0.5 × first-block protected mass
  (the Phase I allocation sequence bar);
- F1-style coexistence ≥ 0.5 × per-seed references (capability record
  bars);
- stability/cap/maxP ≤ 0.6; IL intactness (as Phase I);
- NO new failure class; identity gate byte-identical.

Then Phase II-B (same registration family, after A): retrieval metric
per §1: ρ(A) = cos(v_A_late, v_A_ref) − cos(v_A_late, v_C_ref), with
δ = 0.10 pre-registered (justified: the record's discrimination
baselines — G3 adjacent 0.99 vs within 0.85–0.89; E3b separation gap
0.05–0.22 — δ = 0.10 sits inside the measured discrimination band),
window = last 10 presentations of a re-exposure block, exclusions =
all-zero vectors (< 25%) per CLLA precedent. A retrospective NEGATIVE
control already exists on the record: alternation's adjacent-state
cosine 0.99 ⇒ ρ ≤ 0 under current substrate — the experiment measures
a positive ρ against that.

Distinguishing power: arm 2 failing S1 falsifies the dimension
hypothesis at its minimal strength (then the growth/competition route,
candidate A, is the next registration); arm 2 passing S1 but failing
II-B localizes the gap to EXPRESSION, which Phase I never measured —
a new, well-posed Phase II result either way.

## 16. Proposed Phase II capability ladder

- II-A — sequential memory formation: two protected configurations
  coexist after blocked order (S1 criterion). Success = arm-2 S1 pass
  × 3 seeds, identity gate.
- II-B — selective re-expression: ρ(A) ≥ 0.10 with C retained
  (post-A re-check of C-coherence), on a holdout re-exposure block.
- II-C — interference/capacity: K-tracks stress (K = 2 vs 3,
  three-pattern curricula; saturation: fill all tracks then present a
  fourth pattern — measure F3b-style exhaustion and forgetting-driven
  recovery); D-pattern composition probe analogue (track merge
  grading).
- II-D — closed-loop use (ONLY after A–C): output engagement during
  re-expression (E24-S4-style join measurement) — no behavioral
  training.

Do not skip to II-D; II-D is not a Phase II-A milestone.

## 17. Recommendation (evidence-based; nothing implemented)

1. **Freeze candidate D-core (context tracks, K = 2, learned
   prototypes, per-track bounded protection, no gain) as the Phase
   II-A registration** when the user approves: it is the only
   candidate whose components all have Phase I precedent (V2.3
   partition; CLLA bounded protection; the (1−R) allocation signal),
   whose key policy satisfies R2 (experience-derived), whose capacity
   model is explicit, and whose falsifier is the already-frozen S1
   criterion. It converts the measured closed question (allocation
   starvation) into the open one (selective expression).
2. **Candidate A (competing assemblies) is the SECOND registration
   candidate**, to be frozen only if arm-2 S1 fails: Phase I's
   dynamics-competition failure (E3b) and growth regressions (E4
   family) make it the higher-risk first bet, not the recommended
   first test.
3. **Candidate C is not registered standalone** — its parts are
   executed history (reserve/fe/fec); it serves as the shared
   allocation/consolidation layer inside D.
4. **Candidate B with any injected key (channel-group, epoch, index)
   is rejected by R2**; the V2.3 epoch arm survives only as the
   archived active control the paper already reports.
5. Determinism/identity: D-core ships flag-off byte-identical; the
   FNV anchor 9647ea8a0ca4dbd2 and snapshot-frame gate apply as in
   every Phase I implementation.
6. No E-number until the frozen protocol passes integrity review
   (standing rule).

STOP — architecture study complete. No implementation, no experiments,
no tuning, no E-number.