# ANIMA architecture revision proposal — consolidation-locked local allocation (CLLA)

Status: FULL ARCHITECTURE REVIEW, 2026-09-20. No implementation, no
runs, no tuning, no E-number. This is a design proposal grounded in
the committed record: E16/E17 causal audits, SDE-B/C2/D, V2.3
partition, tsegsim, x-synmem-audit.

## 0. Bottleneck restated

The substrate already creates channel-addressed structure
(x-synmem G1: A/C-trained matrices cosine 0.12, LOPO 91%), but has
NO mechanism that (a) reserves persistent structure against later
plasticity, or (b) allocates new structure to distinct degrees of
freedom. Consequence, measured: blocked order → recency overwrite
(BAC A-mass 0.208→0.068); alternation → superposition into a third,
undiscriminating configuration (after-A_k vs after-C_k cos
0.97→0.99). The missing capability is allocation + protection; the
evidence says the storage medium is fine.

## 1. Evidence base used

- V2.3: M2 budget partitioned by write-epoch → blocked-order bca
  rescued (6/6). PARTITION WORKS as protection; the epoch key is
  its limitation: under alternation at 2 s cadence vs 4 s epochs,
  A/C land in alternating buckets — time keys re-mix interleaved
  patterns, so epoch-partition cannot fix the failure it was
  tested against. Used as evidence, not solution.
- E16/E17: the REV1 flip = re-expression of pre-existing weights
  (0 plasticity events needed). Consequence: RETRIEVAL is forward
  dynamics on stored structure; plasticity is not needed to read.
  A store that survives intact is therefore sufficient for
  retrieval, given only that input statistics still reach it.
- SDE-C2/D: passive decay = ~70–80% eraser; M2 reallocation
  inferred residual competitor. Any protection must cover BOTH
  passive decay and M2 reallocation.
- E16: M2-off removes the separation regime (selectivity 0.48).
  M2 stays; we change what it normalizes.
- x-synmem G6: structure (weights) persists across 20 s silence
  while u/spikes decay 3.5–7× — the persistent medium is weights.
- X-series/tsegsim: u fails as memory substrate under every test
  (interference, decay, drive-gate, buffer+commit).

## 2. Candidate classes A–E

Criteria per candidate: capacity; interference; allocation;
consolidation; retrieval; local info; M2/M3-M4/M6 interaction;
resource accounting; failure modes; coexistence; does it hide a
label/address.

### A. Multiple synaptic compartments/slots per neuron
- Capacity: K slots × N neurons; slot count = fixed budget.
- Interference: none between slots (partitioned), the V2.3 proof.
- Allocation: WHICH slot gets a new pattern? Needs a slot
  selector. Fixed slots (channel-majority, time-parity, index)
  are labels/addresses — forbidden; learned slot assignment is
  the open problem, not solved by A itself.
- M2: per-slot budget (V2.3 already does this).
- Failure: slot identity and fragmentation (one pattern spread
  over slots; two patterns merged in one slot).
- Verdict: the partition is necessary but the KEY is the
  architecture; A alone hides an address (the slot tag).

### B. Sparse competing memory assemblies (neuron-level)
- Capacity: N neurons × per-neuron profile span.
- Allocation: which neurons take a new pattern — needs local
  recruitment signal; hard without competition; M6 inhibition is
  the only competitive machinery and E3b showed dynamics-level
  inhibition does not separate readouts.
- Assembly identity emergent from weights (good, no label) but
  sparse-assembly allocation typically requires a global
  similarity/novelty detector (forbidden) or winner-take-all
  (new global mechanism).
- Verdict: emergent assemblies are the right OBJECT model, but
  B alone needs a forbidden detector.

### C. Synaptic tagging + slow consolidation
- Tag = local per-synapse state; consolidation = protected class
  with per-neuron capacity cap. Allocation decouples from any
  label: a synapse consolidates when its OWN co-activity with
  the neuron's activity is persistent and the neuron has
  headroom.
- Retrieval: forward path (E16/E17: weights + input statistics).
- M2: consolidation classes = self-keyed buckets (the V2.3
  partition with a learned key).
- Failure: tag/consolidation thresholds; no graceful forgetting;
  but capacity is explicit (per-neuron protected mass).
- Verdict: C supplies the missing allocation+protection with
  purely local variables. This is the core of the proposal.

### D. Combination (A+B+C)
- Partition as protection classes (A), assemblies as emergent
  property of consolidated cohorts (B), tagging+consolidation as
  the mechanism (C). Each element is justified by the record;
  none requires global state. This is the proposal.

### E. Other: graded metaplasticity (weight-history resistance)
- No new state: plasticity scale factor declines with synapse
  age/strength-history. Smooth, fully local.
- Failure: SDE-D showed symmetric/asymmetric STDP variants
  destabilize; graded resistance under strong drive is weak
  protection (M2 still reallocates); least testable identity
  gate. Not chosen now; noted as a graded alternative to the
  binary flag.

## 3. The architecture: CLLA

Three changes to the existing V2 substrate, one flag.

### 3.1 New state variables
- Per SYNAPSE (existing Synapse struct):
  - `consolidated: bool` — set on permanence transition (see
    3.4); when set, the synapse is a protected memory element.
- Per NEURON (existing fields suffice): no new field;
  consolidated mass P computed on demand from incoming synapses
  (local).
- Per run (config): `assembly_protect: bool = false` (identity
  default), `p_max_frac: f32` (protected-mass cap as fraction of
  t_e), `w_consolidate_min: f32` (minimum weight to consolidate).
  No RNG, no new timescales, no new events.

### 3.2 Exact locality boundaries
- Every decision uses only: (a) the synapse's own (pre-id, w,
  consolidated flag, co-activity counter), (b) its post neuron's
  incoming live-synapse list and sums (already the M2/Normalize
  iteration boundary), (c) the neuron's own M5 live counts.
- Explicitly NOT used: any cross-neuron variable, any global
  counter, presentation counts, input identity/labels, u,
  episode timing, population activity.

### 3.3 Resource/budget model
- Per neuron: total live excitatory mass = P (consolidated) + W
  (working) ≤ t_e (unchanged M2 invariant).
- Cap: P ≤ p_max_frac · t_e (default 0.6, to be registered in
  the protocol, never tuned). This is the explicit capacity.
- Working budget: W ≤ (1 − p_max_frac) · t_e for NEW learning
  (candidates, unconsolidated plasticity).
- Slot cap unchanged: live_e ≤ b_e (M5).
- Global capacity = Σ_neurons p_max_frac·t_e — finite, explicit,
  no hidden memory. A neuron's headroom (p_max_frac·t_e − P) is
  a local quantity: "an unused representational configuration
  exists here" is readable from the neuron's own state.

### 3.4 Allocation rule (local, no novelty detector)
- A candidate synapse consolidates when: (1) it has reached M3
  permanence (existing co-activity accumulation rule —
  permanently co-active pre/post pair), (2) its weight ≥
  w_consolidate_min, (3) post neuron headroom ≥ its weight:
  P + w ≤ p_max_frac·t_e.
- No "new episode" signal exists: the FIRST strongly co-active,
  persistent pattern that drives a neuron with headroom locks
  structure. Later patterns can only use headroom that remains.
  The organism's decision "this deserves an unused configuration"
  = the neuron's local headroom + the synapse's persistent
  co-activity. This is capacity-driven allocation by first-
  exposure, not novelty detection.
- Candidate creation stays M3-random; consolidation is the
  selector. (Optional accelerator, NOT core: bias candidate
  creation toward recently active pre-channels — local, but the
  proposal does not depend on it.)

### 3.5 Plasticity rule
- Unconsolidated synapses: exactly today's STDP + M6 + decay.
- Consolidated synapses: STDP still applies (pre-gated: their
  pre must fire, so cross-pattern interference is structurally
  zero — a C-presentation never fires an A-synapse's pre).
- Passive decay: consolidated synapses exempt (the SDE-C2
  eraser is the measured killer; exemption is the point of
  consolidation).

### 3.6 Consolidation rule
- Trigger: M3 permanence transition (existing machinery —
  candidate → permanent after sustained co-activity). NO new
  timescale: permanence already exists and is healthy (E16
  permanence counts in band).
- On permanence: if P + w ≤ cap → `consolidated = true`.
- Consolidation is one-way under normal operation (no
  de-consolidation); capacity exhaustion is graceful: when
  headroom hits zero, new synapses never consolidate — old
  memories persist, new ones stay transient.

### 3.7 Protection rule
- M2 normalization (structural_v2.rs `normalize`): the sum and
  the scaling act on WORKING synapses only: W = Σ unconsolidated
  live exc; target W ≤ t_e − P; factor = (t_e − P)/W applied to
  working synapses only; consolidated synapses are excluded —
  their weights change only via STDP/deactivation, never via
  normalization. (This is exactly V2.3's partitioned normalize,
  but the bucket boundaries are self-keyed by consolidation
  rather than clock parity.)
- M4 pruning / silence pruning: consolidated synapses exempt.
- Marginal case: passive decay exempt (3.5). Everything else
  (M6, E6, M5) unchanged.

### 3.8 Retrieval / reactivation rule
- None new: pattern → its channels → strong consolidated
  afferents → cohort fires (E16/E17: retrieval is forward
  dynamics). The protected structure IS the memory; input
  statistics read it for free. No readout machinery added.

### 3.9 Interactions
- M2: partitioned by consolidation class; identity path
  (assembly_protect=false) byte-identical.
- M3/M4: permanence becomes the consolidation gate; M4 exempts
  consolidated; candidate pool otherwise unchanged.
- M6: unchanged; its inhibitory dynamics operate on activity,
  not on protected weights.
- E6: unchanged (rate-balances input channels; protected
  synapses still deliver current, E6 modulates their gain — no
  interaction with the class).
- u: retired from memory role (see §7); not used anywhere here.

### 3.10 Why this is not "exempt forever" (the standing constraint)
Capacity is defined and bounded per neuron (P cap), enforced at
consolidation entry and by the working budget. Exemption applies
only to synapses admitted under the cap; the cap is a config
constant. Exemption from M2 reallocation is exactly the
partitioning V2.3 validated; exemption from passive decay is
bounded by the cap and is what makes stored structure persist.

## 4. Expected behaviors

### Repeated A/C (alternating)
- A-presentations drive A-channels → A-permanent co-active
  afferents reach permanence → consolidate where headroom.
  C same. Under alternation both classes exist on overlapping
  neuron sets but on disjoint synapse sets.
- M2 normalizes only working synapses: cross-pattern reallocation
  stops. No superposition into a third vector; both structures
  persist side by side, each gated by its own pre-channel set.
- Retrieval: alternating input → A-response ≠ C-response
  (per-presentation during-window vectors separate), because the
  two cohorts' currents are channel-disjoint. (Contrast:
  unmodified runs show during-window separation only partially
  and off-window none — x-synmem G3 state mix.)

### BAC/BCA
- Block 1 consolidates (A in BAC). Block 2 (C) cannot displace
  protected A; C consolidates into the same neurons' headroom or
  neurons with spare P. End state: BOTH cohorts present (A-mass
  does not collapse to 0.068). This is the prediction the 6/6
  V2.3 blocked rescue generalizes to the unpartitioned case.

### Novel D (A+C co-activation)
- D drives the A- and C-cohorts simultaneously → D response ≈
  blend of the two assemblies (sum of both drives). No new
  assembly for D (its channels are already owned — no headroom
  for a distinct D structure on A/C-committed neurons, and no
  label distinguishes it). Honest: D is not discriminated as a
  third object; it is the superposition the substrate gives for
  a genuinely overlapping pattern.

## 5. Failure modes
1. Consolidation too eager (one exposure locks noise) → capacity
   junk. Mitigation: consolidation requires M3 permanence
   (multi-window co-activity), plus w_consolidate_min gate.
2. Capacity exhaustion → new patterns never persist (graceful,
   explicit) — indistinguishable from "no learning" if headroom
   is tiny; measure headroom in protocol.
3. Post-consolidation LTP can push P above p_max_frac·t_e
   (entry-gated cap only). Bounded by w_max; document, don't
   clamp (clamping = a second normalization = forbidden).
4. STDP depression on consolidated synapses when their pre fires
   in a non-reinforcing pattern: pre-gated, so cross-pattern
   depression is impossible; within-pattern depression is
   normal Hebbian dynamics (allowed: memory of a pattern must
   stay responsive to that pattern).
5. P2 runaway: protected strong weights → stronger drive →
   higher rates. Guard unchanged (freeze protocol); monitor.
6. No forgetting/updating: a consolidated memory cannot be
   overwritten by later conflicting experience — the organism
   trades flexibility for stability. This is a deliberate,
   explicit design consequence (fails "updatability" tests by
   design; the falsifier set must not require overwrite).
7. Same-neuron multi-profile with OVERLAPPING channels (D-like):
   blends as above — architectural limit, not bug.

## 6. Minimal falsifiable experiment (proposal only)
Single flag arm, all other config frozen at the e24 cell
(β=0.0046875, τ=5000, m2_buckets=1, seed 20260912):
- construct configs: e24-il/a/c/bac/bca + novel-D + identity
- Arms: assembly_protect=true × {il(A/C), bac, bca} ; control =
  committed same-seed runs (false)
- Frozen endpoints (pre-registered, no post-hoc thresholds):
  1. coex: protected A-mass and C-mass > 0 at drive end under
     both orderings (vs BAC collapse 0.068/0.239 — threshold:
     each ≥ 0.5 × single-pattern end-state value);
  2. react: per-presentation during-window response-vector
     cos(A_pres, C_pres) < 0.9 by rep 20 under alternation
     (frozen; current unmodified cell gives 0.95–0.99);
  3. capacity: max(P)/neuron ≤ p_max_frac·t_e at every snapshot;
  4. identity: flag=false run byte-identical (FNV
     135,293-row anchor d452d028ffaec973);
  5. novel-D: D-response ≈ α·A-response + β·C-response (freeze
     the fit criterion pre-run).
- No tuning: p_max_frac and w_consolidate_min frozen at protocol
  freeze, justified from V2.3 (t_e split) and permanence
  threshold (0.05) respectively.

## 7. u: retire from primary memory, keep as physiology
- RETIRE: u as memory carrier fails every committed test
  (wedge cos 0.9995 during stimulation; decay; drive-gate;
  buffer+commit mixture; tsegsim k=40 → cos 1.0). No design
  continues to build on u as the memory.
- KEEP: u as intrinsic slow physiological state — V2.1 default
  (β=0) already makes it inert; its documented positive role is
  sustaining endogenous regime activity (X1 O1.2; O2.6 top-u↔
  spikes), an arousal/pacemaking property, not storage. Leave
  the code path intact (identity), stop extending it. CLLA
  adds no u dependency.

## 8. Why the proposal satisfies the constraints
- Allocation: local headroom + permanence, no novelty detector,
  no label, no trial boundary, no reward.
- Coexistence: protection classes = self-keyed partition; the
  V2.3 result is the supporting evidence.
- Capacity: explicit per-neuron P cap; no hidden memory.
- Identity: single flag, default false; consolidated flag never
  serialized when off (Optional<...> skip pattern as g_drive);
  M2 normalize path untouched when off → byte-identical history
  (e24 FNV anchor).
- Locality: every rule at synapse/neuron boundary; no global
  similarity or population reset.
- No K× network duplication; assemblies share neurons.

## 9. What would falsify the architecture
- If protection works but response separation does NOT emerge
  under alternation (protected structures coexist yet input
  evokes mixtures), CLLA's retrieval claim is false — the store
  is protected but unreadable, forcing a readout mechanism.
- If blocked-order collapse persists despite protection at
  multiple consolidation speeds (permanence θ → 0), the
  erasure is not M2/decay-mediated — contradicting SDE-C2/D's
  attribution; then no protection of weights can help and
  competition must be activity-level (B-style, global).
- If capacity exhausts after one pattern across all neurons
  (headroom ≈ 0 at drive end always), allocation is
  indistinguishable from locking: the first pattern monopolizes
  the substrate — requirement 1/5 fail.
- If novel-D produces a NEW discriminated response (not the
  A+C blend), the architecture secretly encoded pattern
  identity somewhere — violation of "no magic context".

STOP. Proposal only; nothing implemented, nothing run, nothing
frozen into a protocol.