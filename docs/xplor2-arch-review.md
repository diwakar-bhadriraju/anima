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
