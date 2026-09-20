# ANIMA memory-topology review — what u cannot do, and what already can

Status: READ-ONLY ARCHITECTURE REVIEW, 2026-09-20. No runs, no
implementation, no tuning, no frozen experiment. Builds on:
docs/x-segarch.md (proposed mechanism), the tsegsim trace results
(alternating u-cos → 1.0000 by k=20; newest-fraction 3.6% at
k=40; same-pattern u-cos decreasing 0.89→0.45), committed
measurements O2.4/O2.5 (real u per-presentation cos ≈ 0.999,
permutation p=0.12/0.35), bottleneck map (cntR 1.0000; windowed
0.62–0.67; affw 0.887→0.052, J10 0.0), xplor2-arch-review.

---

## 1. Critical question first: is single-vector u theoretically capable?

No. The requirement — preserve distinct episode-specific
information across repeated alternating experiences — cannot be
met by a per-neuron scalar u updated by any mechanism in the
family {u ← λu + ρb, λ∈[0,1]} (accumulate, or accumulate+decay).
Two independent reasons, one weighting, one representational:

### Reason 1 — dilution (weighting): any nonnegative accumulator
is dominated by history, exactly as k grows.

Same-run alternating readout (the case the trace sim measured):
u after the k-th A vs u after the k-th C differ by ONE episode
only (the k-th C). Either state is a sum over the shared
preceding history.

    cos(u_Ak, u_Ck) = cos( S_k , S_k + ρ·b_Ck )  → 1   as k → ∞

regardless of how well b_A and b_C separate — because the
difference is O(‖b‖) while the common part grows O(k‖b‖). The
trace sim confirms: k=1 0.976 → k=20 1.0000, newest-fraction
3.6% at k=40. Decay fixes THIS one: with exponential decay the
newest episode's weight is O(1), so this particular collapse
disappears — which is precisely why decay "changes magnitude
weighting." It is necessary but nowhere near sufficient.

### Reason 2 — collinearity (representation): decay cannot add a
direction that the write vectors don't have.

Unrolled, any decayed state is

    u(T) = ρ Σ_j α^{T−j} b_j + α^T u_0

a NONNEGATIVE linear combination of the same episode vectors.
The reachable set is always inside the cone spanned by {b_j};
decay only moves the coefficients (recency weighting), never the
cone. Separation of two histories requires the two states to
differ in DIRECTION — some component of b_A − b_C that survives
weighting. What does the substrate give?

  - real per-presentation u slices: cross-cos ≈ 0.999 (O2.4,
    O2.5 — permutation-tested, no class gap). The episode
    vectors themselves are near-collinear: their difference is
    ~1.4% of their norm. Every decayed mixture of near-collinear
    vectors is near-collinear. Decay has nothing to amplify:
    it cannot create the discriminant direction.
  - gated episode buffers b (tsegsim): cross-cos 0.52–0.68 —
    bounded away from 1. A decayed mixture of THESE would reach
    cos ≈ 0.85–0.9 (still shared-dominated; the residual A/C
    axis exists but is a small component of both states).

So: decay reweights magnitude along directions fixed by the
write vectors; representational separation requires directional
diversity in the write vectors themselves. The per-episode u
write vectors have none (shared recurrent pool, post-synaptic
aggregation); the substrate's only directionally diverse
persistent representation is the AFFERENT WEIGHT MATRIX (below).

### Reason 3 — why the accumulation site matters (the geometry)

Post-synaptic aggregation (u, spike counts) sums over all 24
input channels per neuron: 52 scalars, and every episode's
contribution is drawn from the same high-overlap pool (cross-cos
0.5–0.999). Channel identity — the only systematic A/C
difference — is discarded at the membrane. The per-synapse
space (24 × 52 = 1248 sites, each addressed by its own
pre-synaptic partner) keeps it. All post-synaptic scalars
inherit the pool's collinearity; the cone argument makes the
failure inevitable for any of them, at any timescale.

## 2. Information geometry — where A/C structure lives and dies

| representation | A/C cos (measured) | survives k → 40 alternation? |
|---|---|---|
| per-episode gated b (buffer) | 0.52–0.68 (tsegsim) | yes (it IS per-episode) |
| per-presentation windowed counts | 0.62–0.67 (bottleneck) | yes |
| per-tick spike sets during drive | 0.945 → 0.580 (bottleneck) | yes (learning, not memory) |
| cumulative counts cntR | 1.0000 (bottleneck) | no — destroyed |
| single-vector u, accumulated | → 1.0000 (tsegsim k≥20) | no — destroyed |
| single-vector u, decayed | ≈ 0.999 real / ≈0.85–0.9 gated | no (real) / marginal (gated) |
| afferent weight fingerprints affw | 0.887 → 0.052, J10 0.0 | YES — strongest separator in the organism |

Everything post-synaptic dies by accumulation (Reason 1) or was
born collinear (Reason 2). The afferent matrix is the single
representation that (a) survived 40A+40C interleaved in the
committed run, (b) has per-synapse directional diversity, (c)
already has a slow timescale (M2 epoch structure, passive decay
≈ τ 1000 s) and a self-organized competition (STDP + M2 zero-sum
budget → V2.3 epoch buckets).

Caveat recorded: affw's decorrelation is partly M2-budget
complementarity (va[i]+vc[i] bounded per neuron) — i.e., the
"separation" includes competition-induced mutual exclusion, not
only Hebbian potentiation. That caveat does NOT weaken the
memory argument: competition IS a legitimate self-organizing
storage mechanism (it is the substrate's own overwrite rule), and
the question is whether the resulting structure persists and is
readable — not which plasticity produced it. It persists (drive
end), and it is locally maintained (STDP/M2 at the synapse).

## 3. Architectural bottleneck — what capability is missing?

The memory already exists: the afferent matrix holds
channel-specific, mutually decorrelated fingerprints after
repeated alternation, maintained by substrate-native STDP/M2.
What is missing:

  1. A READ path from that store into anything slower than the
     instantaneous current. Weights gate spikes (always), but no
     state downstream of the channel sum retains the
     distinction (u discards it; O2.5: 0.999).
  2. An EPISODE-LOCAL accumulator that is allowed to be the
     content (b works; commit-to-u destroys it — trace sim).
     "Persistent content" ≠ "accumulated into u".

Capability missing = CHANNEL-ADDRESSABLE PERSISTENT READ:
the organism has no way to consult "how strongly my A-inputs
have been reinforced vs my C-inputs" except by presenting the
pattern again (which works — affw/inst learn — but only during
drive, never as a state).

## 4. Candidate classes

Common premises: curriculum-blind, reward-free, local; S1 as
validational signal only (x-segarch §3); no labels/trial
boundaries; identity default when off.

### A. Competitive overwrite / WTA persistent state
  1. DOF: 52 (u) + one competition scalar per neuron.
  2. Labels/boundaries: none needed.
  3. Local modification decision: NO — requires cross-neuron
     comparison (max/normalization) at write time; u has no
     lateral structure. E3b inhibition (dynamics-level) exists
     in substrate but failed to separate readouts.
  4. Coexistence: no — WTA overwrites by design; that's its
     point.
  5. Resemblance: reinforces the winner (merge).
  6. Alternation: would stop mixing in principle (nonlinear
     projection), but adds a NEW competitive computation on the
     scalar state; the substrate ALREADY competes at the synapse
     level (M2) — doing it twice is redundant.
  7. Overwrite/merge/preserve: overwrite.
  8. New resource/budget: competition signal = new mechanism.
  9. New timescale: competition needs its own time constant.
  10. M2/V2.3: orthogonal, duplicates M2's role.
  11. Curriculum-blind: yes.
  12. Retrieval: input-driven WTA (no external address), but the
      readout is just "current winner" — collapses to
      recency/dominance, not episode structure.
  13. Developmental: competition rule is a designed addition —
      borders on pre-encoding a memory architecture.
  Verdict: heavyweight, redundant with existing M2 competition,
  weakest retrieval semantics.

### B. Sparse addressed memory / multiple assemblies
  1. DOF: M × 52 (M = assembly count — a free parameter).
  2. Labels: allocation needs a novelty/matching decision
     (similarity vs existing assemblies) — content-based, no
     labels, but a threshold parameter appears.
  3. Local: matching = global comparison; allocation local only
     with lateral machinery we don't have.
  4. Coexistence: yes — that's the design.
  5. Resemblance: strengthens existing assembly.
  6. Alternation: yes — M slots absorb alternating content.
  7. Overwrite/merge: merge-on-similarity, allocate-on-novelty.
  8. New resource: assembly slots = a memory budget.
  9. New timescale: slow allocation dynamics.
  10. M2/V2.3: orthogonal — this is the Y3 structural-growth
      path (E4 family closed sink/reciprocal growth shapes;
      low-fan-in untested; stability gate failures on record).
  11. Curriculum-blind: yes (content similarity), but the
      similarity metric is a design choice.
  12. Retrieval: similarity probe — nearest assembly. Requires
      the probe computation (not present).
  13. Developmental: emergent assemblies = charter-compatible
      IN PRINCIPLE; a pre-set slot count/metric = encoding the
      architecture. Y3 machinery exists and is the only
      self-organized route, but every growth shape tested so far
      regressed (E4a-E4f).
  Verdict: the only route to PERSISTENT SEPARATE OBJECTS, but
  the heaviest; no stable growth shape exists yet.

### C. Partitioned persistent state (locally determined compartments)
  1. DOF: G × 52 (G = channel groups — the substrate's own
     input partition).
  2. Labels: none — channel identity IS the partition, known
     locally at every synapse (its pre-synaptic partner).
  3. Local: yes, trivially: each synapse accumulates its own
     channel's state.
  4. Coexistence: yes — partitions never mix by construction.
  5. Resemblance: merges within partition.
  6. Alternation: yes — A-channels and C-channels are disjoint
     groups (e1.toml: A = ch 0–7, C = ch 16–23); alternating
     writes touch disjoint compartments → the exact failure
     (cross-pattern mixing) is structurally impossible.
  7. Overwrite/merge: preserve + same-channel merge.
  8. New resource: G scalars per neuron (52×3) — or, if the
     read is per-neuron-per-channel: 52×24 = the weight matrix.
  9. New timescale: none (weight timescale suffices).
  10. M2/V2.3: orthogonal to M2 (M2 acts within a neuron's
      budget); V2.3 partition is the weight-level analogue and
      already demonstrated (bca rescue).
  11. Curriculum-blind: yes.
  12. Retrieval: the partitions ARE channel-addressed — input
      drive reads its own partition automatically.
  13. Developmental: the partition is substrate-given (input
      channel identity is an architectural fact of the sensory
      interface, like D8's input channels); no memory
      architecture invented.
  Verdict: minimal, local, alternating-proof. BUT its content is
  a per-channel sum of weights = exactly the affw fingerprints —
  i.e., C collapses into reading what weights already store (D).

### D. Synaptic-addressed memory (persistent structure in weights,
    read through channel identity)
  1. DOF: 24 × 52 = 1248 — ALREADY EXISTS (afferent matrix).
  2. Labels: none.
  3. Local: yes — STDP eligibility is per-synapse, pre-partner
     known locally; no comparison needed.
  4. Coexistence: demonstrated by affw at 40/40 alternation and
     V2.3 partitioned epochs (both cohorts held in the
     partition arm).
  5. Resemblance: overlapping exposure strengthens shared
     synapses (recognition storage by construction).
  6. Alternation: empirically yes — the only measured
     representation that did not collapse (0.05 / J10 0.0).
  7. Overwrite/merge: M2 zero-sum budget = substrate-native
     overwrite; passive decay the eraser; merge on similarity.
  8. New resource: none — store, timescale, competition all
     exist (STDP, M2, τ≈1000 s decay).
  9. New timescale: none.
  10. M2/V2.3: this IS the M2/STDP site; fully compatible.
  11. Curriculum-blind: yes.
  12. Retrieval: content-addressable by re-presentation (the
      forward path itself); no external address. This is the
      substrate's only non-trivial retrieval mechanism.
  13. Developmental: the memory form is the existing synapse
      topology — nothing invented; plasticity laws unchanged.
  Verdict: the minimal topology — because the topology already
  exists and already passes the alternation test. The deficit is
  ACCESS, not storage.

### E. The substrate suggestion (this review's finding)
The single-vector u experiment asked the wrong question twice:
first by aggregating post-synaptically (a cone), then by asking
an accumulator to be a content store. The substrate's answer to
"where can distinct episode information live" is already written
in the afferent matrix and in the gated episode buffer b. The
smallest mechanism is not new storage — it is a READ:
channel-resolved persistent readout of the afferent structure
(or, equivalently, treating b as the content store and gating
persistence by S1 without committing into u's shared cone).

## 5. The single smallest mechanism deserving a frozen experiment

D in its read-only form: CHANNEL-ADDRESSED PERSISTENT READ —
per-channel slow state derived from the afferent weight
structure (e.g., per (channel-group, neuron) weight-sum, the
affw representation, exposed as state the organism can act on),
with STDP+M2 left exactly as they are (the memory write already
works). Zero new timescales (weight τ used), zero new resources,
zero RNG, identity default (off = today's path byte-identical).

BUT before any freeze, a READ-ONLY GATE on committed artifacts
(the actual first distinguishing experiment, no organism change):

  G1. Decode A vs C (leave-one-presentation-out) from
      per-neuron afferent weight vectors at the last snapshot of
      the committed 40A+40C run (affw suggests strong signal:
      0.05 / J10 0.0). Chance = 50%.
  G2. Same decode from per-channel-group weight sums (the C
      representation) — is the collapse of u (0.999) really not
      present at the channel-resolved level?
  G3. Mid-run snapshots (5–10 along the drive): does the
      fingerprint survive the alternation THROUGHOUT, or only at
      the end? (Determines whether the store is stable or a
      transient.)
  G4. Contamination control: M2-budget complementarity could
      inflate G1 (va+vc ≈ budget per neuron). Control: decode on
      weights after dividing out per-neuron budget (project onto
      the budget-orthogonal subspace); if the decode survives,
      the memory is real structure, not just competition
      artifact.

If G1–G4 pass → freeze D (channel-resolved persistent read,
identity-gated) as the mechanism experiment, per standing
discipline (protocol → approval → identity gate → runs). If the
decode is at chance → the affw separation is pure budget
mechanics and the ONLY remaining route is structural growth
(Y3/B) — which would require first solving the growth-stability
failure (E4 family), a much larger program.

## 6. Charter check

D/C/E: no internal architecture invented — the memory form is
the substrate's own synapse topology and channel identity;
plasticity laws untouched; curriculum-blind; reward-free;
boundary-free (S1 remains evidence, not a component).
A: designed competition on the scalar state — charter risk.
B: charter-compatible only as emergent Y3 growth, which has no
stable shape yet.
The review's answer to "is the mechanism developmental or
pre-encoded": the minimal one (D) is developmental by
construction — its shape is inherited from the organism's
existing sensory interface.

STOP. Nothing frozen, nothing implemented, nothing run.