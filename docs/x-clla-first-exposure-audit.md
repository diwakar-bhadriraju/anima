# First-exposure allocation audit (READ-ONLY)

Status: 2026-09-21. Uses only the committed reserve/rule/bounded
CLLA runs (+ implementation source). No implementation, no runs,
no tuning, no E-number. Follows the closed dormant-reserve result
(4fc9581): reserve preserved the co-fired cohort but was vacuous
for first-seen patterns. This audit examines the alternative:
on-demand binding at first exposure.

## 1. Exact current M3 creation semantics

Source: structural_v2.rs `draw_candidate` (read verbatim).
- CREATION TRIGGERS: (a) bootstrap in V2Plasticity::new (pool
  filled to c_slots=6 from nothing), (b) permanence redraw
  (candidate consumed → draw one), (c) death redraw (w < θ_die →
  swap_remove → draw one).
- PRE SELECTION: uniformly random over input channels and
  recurrent neurons, each accepted with Bernoulli p_cand_in /
  p_cand_rec from the seeded RNG; iteration in channel-id /
  neuron-id ORDER but acceptance is RNG-gated ⇒ effectively
  uniform. Skips pres already connected via a live synapse OR
  already in the pool (`connected()`), but AN ABSENT PRE IS
  DRAWABLE — creation does NOT require a live synapse for that
  input.
- COACTIVITY: not required for creation. A candidate is created
  exactly at bootstrap/redraw; co-activity only ACCUMULATES its
  w afterward.
- OBSERVABILITY OF FIRING INPUT: draw_candidate receives
  (net, pool, post, params) — NO fired-channel information.
  It cannot prefer a currently active input.
- NEW-SYNAPSE QUESTION: draw_candidate may pick a channel with no
  live synapse (connected() only skips live-or-pooled); on
  permanence, add_synapse creates the live synapse on that pre.
  So creation for an absent input IS supported mechanically.

## 2. Exact first-exposure information available locally

At C-block onset (committed bac runs; t=44000-46000 pool rows +
rule-run R/headroom/substrate data):
- CURRENTLY FIRING INPUT CHANNEL: observable, but ONLY through
  surviving live afferents. The fired_channel set per neuron is
  built from the R-split delivery scan, which iterates live
  outgoing synapses — a channel with all its afferents M2/M4-
  pruned is INVISIBLE to that neuron. Measured: 65 surviving C
  afferents / 52 neurons = 1.25 visible C channels/neuron at
  onset (vs 197 originally).
- FREE CANDIDATE SLOTS: 154-203 unreserved+unwaiting slots per
  boundary row (poolbound), = 3.0-3.9 free slots/neuron measured
  at t=44000.
- PROTECTED-EXPLAINED FRACTION R: measured 0.0 at first C
  (resratio on rule runs: R-med 0.000 pres 21-24) — the arriving
  input is maximally unexplained.
- REMAINING PROTECTED HEADROOM: 0.249-0.294/neuron (rule runs;
  reserve runs 0.28 at boundary).
- M2 WORKING MASS (C-cohort): 3.24/neuron-total at onset (down
  from 10.72) — the substrate deficit.
- POOL OCCUPANCY: 312 slots, of which 108-124 already have
  C-cohort pre at w 0.007-0.016 (just above θ_die=0.005) — they
  die within ~7 s and redraw randomly.
So ALL FOUR local conditions the mandate names are measurable:
  active input ✓ (visibility-capped at 1.25/neuron),
  no protected explanation ✓ (R=0),
  available capacity ✓ (3.0-3.9 free slots/neuron),
  substrate deficit ✓ (1.25 visible channels).

## 3. Is on-demand binding feasible on the existing substrate?

YES, with one hard cap:
- draw_candidate CAN create for an absent pre (mechanical fact),
  and permanence then materializes the live synapse — so binding
  a free slot to the currently active channel requires NO new
  synapse machinery.
- The pre-selection needs only a passed-in fired-channel set —
  draw_candidate's signature gains the local per-neuron
  fired_channels (already tracked by the allocation rule for a
  different purpose; zero new state).
- The cap: first-round binding is limited to VISIBLE channels
  (live-afferent-surviving). Neurons whose C afferents were all
  pruned cannot see C at first exposure — but permanence on a
  bound channel creates a live synapse, so after the first
  binding round the visibility self-amplifies (each permanence
  adds a live afferent). The first round is capped at ~1.25
  bindings/neuron.
- Feasibility verdict: the substrate supports it cleanly (local
  fired-channel set exists; draw can source from it; permanence
  materializes); the visibility cap is the real information
  limit and must be measured, not assumed.

## 4. Candidate-capacity analysis (c_slots=6 unchanged)

- First block evidence: A achieves 152-172 permanence events
  (≈3/neuron) → 16.85 protected mass. C needs the same scale.
- Six slots/neuron hold 6 pre-associations; 8 C channels fit
  easily across 52 neurons with redundancy (each neuron needs
  only one bound C channel to contribute; the pattern's full
  channel set is covered collectively).
- Redraw-on-permanence REBINDS to the still-active pattern while
  C is presented — sustained supply requires no more than 1
  bound slot/neuron once exposure is ongoing.
- c_slots=6 is NOT the limiting resource (3.0-3.9 free slots at
  onset); no increase needed, no new resource class.

## 5. Full-slot behavior (all 6 occupied, new channel fires)

Least-arbitrary rule using EXISTING candidate state only
(w, reserved, permanence, index):
  when P1 (all slots reserved/eligible-waiting) ∧ P2 (fired
  channel with no pool candidate):
    evict per the frozen ladder (un-reserved lowest-w → reserved-
    at-floor → eligible-waiting; tie = lowest pool index),
    bind the missing fired channel.
This is EXACTLY the reserve's eviction ladder
(docs/x-clla-dormant-reserve.md §1), already implemented and
unit-tested — the on-demand allocator REUSES it, with one change:
the pool-pressure trigger applies equally to NON-reserved pools
(the reserve restricted P1 to all-held; the on-demand rule engages
whenever a fired channel lacks a candidate and the slot it would
take is evictable). No new age/timestamp/context state.

## 6. NO-MAGIC test

The on-demand allocator uses ONLY:
- currently active local input (fired_channels per neuron);
- existing protected/working state (R via res_ip/res_iw, P, cap);
- free candidate capacity (pool occupancy, reserved bit, w, index).
It does NOT know A/C, BAC/BCA, pattern identity, presentation
number, or future usefulness — the binding condition is the
mandate's local conjunction:
    currently active input ∧ R < 1 ∧ headroom > 0 ∧ slot available.
No novelty detector is invented; the "novelty" of a channel is
expressed by its own absence from the pool + the protected pool
not explaining it (R), both local.

## 7. Dormant preservation vs first-exposure allocation

DIFFERENT CAPABILITIES, now empirically separated:
- Dormant reserve: REQUIRES PRIOR EVIDENCE (the candidate must
  have co-fired once; its ever-coactive bit). It retains knowledge
  about patterns that EXIST in local history. Closed 4fc9581: it
  cannot help a first-seen pattern because no evidence precedes it.
- First-exposure allocation: OPERATES AT THE MOMENT NOVEL
  EVIDENCE APPEARS. It binds capacity to a channel that is firing
  NOW, before that channel ever co-accumulated. It needs no prior
  history; it consumes exactly the free capacity.
The experiment's failure was NOT that preservation is weak — it
was that preservation's trigger (evidence) is exactly what a
first-arrival lacks. On-demand binding is the complementary,
evidence-free capability.

## 8. Architectural decision

CANDIDATES:
A. bind an existing free candidate to a currently active input —
   the draw's pre is replaced by the fired channel. Smallest in
   code (draw source swap) but "free candidate" must be defined
   when full (§5).
B. create a NEW candidate from a currently active input when none
   exists — mechanically identical to A given pools hold exactly
   c_slots and redraw replaces: "create" and "rebind" differ only
   in framing; B adds nothing beyond A's rebind (a slot exists
   always).
C. allocate a temporary live synapse at first exposure — NEW
   resource class (transient live synapses outside M3/M4/M2
   accounting), conflicts with b_e, M2, protection semantics.
   REJECTED: heaviest, new resource.
D. nothing else in the substrate is smaller.

DECISION: A (implemented as rebind) — modify draw_candidate to
prefer the per-neuron fired-channel set when the allocation rule
is on: on any redraw, FIRST try channels in fired_channels (loop
in channel-id order, skip connected/pooled, skip already-pooled
pres), fall back to the existing random draw. Full-slot case:
before drawing, apply the §5 ladder eviction. This is:
- one function (draw signature gains `&self.fired_channels[post]`
  or a filter arg);
- zero new state (fired_channels already exists; reserved bit
  exists; no timestamp);
- zero new resource (c_slots, b_e, caps unchanged);
- local and label-free (no pattern membership, no identity);
- identity flag-off: rule off ⇒ draw ignores fired set ⇒ random
  draws identical.

## 9. Counterfactual result (committed data, information test)

Bound candidates at first C exposure (reserve-run boundary data):
~65 first-round binds (1.25 visible/neuron × 52) + the 108-124
random C-cohort candidates already present at onset re-bind to
the active set instead of dying-to-random.
- w trajectory to permanence: 0.01 → 0.0516 after ONE co-active
  presentation (Δ+0.05 over 5 windows × g=1.0, off decay 0.99^15)
  → bound candidates reach θ_permanent within one presentation.
- supply: ~65 events vs observed 35 (no-reserve) / 38 (reserve) —
  a ~1.9× increase; 0.38× of the first-block 170-event scale.
- IF permanence rate were the only limiter, the second/first
  protected-mass ratio could plausibly reach ~0.38-0.5 (bar 0.5):
  borderline SUFFICIENT, not certain.
- The hard cap: visibility (1.25/neuron). Neurons with zero
  surviving C afferents are blind to C at first exposure; only
  post-permanence self-amplification adds visibility. Hence the
  counterfactual supports feasibility but NOT a guarantee that
  0.5× is met — the correct next step is the falsifiable
  experiment, not a claim.

## 10. The SINGLE smallest architectural capability replacing the reserve

REBIND-ON-EXPOSURE: when the allocation rule is on, candidate
draws source from the neuron's currently-firing input channels
(fired_channels, channel-id order, skip already-connected/pooled,
fall back to random), with the existing eviction ladder applied
when a needed slot is occupied. Nothing else changes.

This replaces the dormant-reserve idea (which is vacuous for
first-arrival by its trigger) with a mechanism that acts at the
moment first evidence appears — the exact gap the reserve could
not fill. It reuses: fired_channels (exists), the eviction ladder
(exists, tested), draw_candidate (exists), c_slots (unchanged).

## 11. Minimal falsifiable experiment (NOT executed)

Matrix mirrors the prior blocked protocol:
  1 identity (flag OFF; draw signature flag-gated → byte-identical)
  3 bac × seeds
  3 bca × seeds
  3 il × seeds (alternating preserved; rebind active in il too)
  3 d × seeds (informative-only)
  = 13 runs, e24 cell, everything frozen except the draw-bias flag.

Primary endpoint: S1 — second-block protected mass ≥ 0.5 ×
first-block protected mass, 6/6 blocked arms.
Preserved: il coexistence (F1-il), P cap, M2 target > 0,
stability 0 failures, finite c_slots, static conformance.
S7 classification retained: mechanism-vs-flux separation via the
existing PERM/CandidatePool telemetry (bound candidates at
onset, w at reappearance, permanence rate, second-block mass).
Verdict binary; no memory-capability claim; no E-number; F2/F6
11-20 correction retained for future protocols; historical
verdicts untouched.

STOP — audit complete; nothing implemented, nothing run.