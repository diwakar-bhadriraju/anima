# Future-memory substrate preservation audit (READ-ONLY)

Status: 2026-09-21. Uses only committed allocation-rule/bounded-CLLA
runs + telemetry/snapshots (instrument: examples/subaudit.rs).
No implementation, no runs, no tuning, no E-number. The allocation
rule (92e41ec) is closed; this audit examines what the rule cannot
fix: the working substrate's destruction during the inactive
interval.

## 1. Exact M2/M4 starvation decomposition (blocked BAC, rule run, s20260912)

C-cohort (channels 8-15, dormant during the 0-41 s A block):
- live unconsolidated count: 197 → 65 (−132, −67%)
- working mass: 10.72 → 3.24 (−70%)
- protected C mass: 0 (nothing consolidates while dormant)
- A-cohort protected mass grows 0 → 16.38 → working target
  t_e − P per neuron falls 0.8 → 0.485.

Prune events (telemetry-authoritative):
- A block: 740 competitive-prunes, ZERO silence-prunes
  (silence_ticks = 60 s > block length 40 s → silence-prune cannot
  fire during the dormant interval).
- Pre-prune weights: **all ≤ 0.0050 = exactly the M4 theta_prune
  threshold** (mean C 0.0027, A 0.0016, O 0.0020).
- C block: 112 competitive-prunes, again all ≤ 0.0050.

Attribution:
- **M4 is the count-deleting operator** (every lost synapse was
  removed by a competitive-prune event; zero silence-prunes).
- **M2 is the mass-destroying and condition-setting operator**:
  every pruned synapse had already been crushed below theta_prune
  by M2 working-target rescaling (mean pre-prune w 0.002-0.003 ≈
  the M2 floor of a starved working pool). M4's threshold is the
  EXECUTION point; M2 sets the condition.
- O-cohort (never-presented noise) suffers IDENTICAL degradation
  (204 → 67, mass 11.04 → 3.53): the destruction is cohort-blind;
  no local variable distinguishes dormant-C from dormant-O at this
  stage.
- Sufficiency: the 65 surviving C afferents at ~0.003 w each
  deliver IC ≈ 3.0 (vs A's 27 settled) — the surviving substrate
  is insufficient in principle to establish the later
  representation (C-block permanence rate 1.8/pres vs A 8.5/pres).
- Rule-vs-no-rule: identical degradation (no-rule C count 197→54,
  mass 10.72→2.95) — the allocation rule does not touch the
  substrate; it operates downstream (candidate accumulation).

## 2. Timeline of second-block substrate destruction

| t (ms) | uC count | C mass | O count | cA count | event context |
|---|---|---|---|---|---|
| 1000 | 197 | 10.72 | 204 | 0 | S1 starts |
| 11000 | 99 | 4.71 | 98 | 82 | M2 target falling with cA growth |
| 21000 | 83 | 4.38 | 87 | 123 | continued M2 crush → M4 begins |
| 31000 | 77 | 3.63 | 81 | 139 | |
| 41000 | 65 | 3.24 | 67 | 165 | **C block onset: 67% count gone, 70% mass gone** |
| 51000 | 50 | 3.19 | 51 | 168 | C presentation: IC 3.0, permanence 1.8/pres |
| 81000 | 18 | 2.49 | 33 | 168 | |
| 101000 | 15 | 2.62 | 22 | 168 | |

The destruction front is EARLY: by t=11 s (3 presentations) C mass
already halved; by t=21 s two-thirds of count gone. M4 follows M2
with the 10-window low-weight accumulator lag.

## 3. Does existing local state justify dormant preservation?

Answer: **NO cohort-specific justification exists — and that is
now a measured fact, not a design opinion.** At the C-block onset
the substrate contains, per neuron: silent afferents at ~0.003 w
(C and O indistinguishable), protected A mass, headroom 0.28, and
candidate pools drawn from random channels. There is NO local
quantity at t=41 s that says "this synapse was C-relevant" — every
synapse-local variable (weight, silent_ticks, activity trace)
behaved identically for C and O for 40 s. Any preservation
mechanism keyed on per-synapse state would preserve noise equally;
any mechanism that distinguishes cohorts uses forbidden channel
identity.

The only local structure that DID know "a dormant input channel set
might be useful" was the M3 candidate's pre-channel association —
and candidates die in ~69 inactive windows = 6.9 s (θ_die 0.005,
decay_c 0.99), i.e., long before the 40 s first block ends. The
candidate's dormant-memory content (its pre channel) is destroyed
and redrawn randomly before the second block arrives.

## 4. Candidate preservation mechanisms

A. Working-synapse protection (freeze w from M2/M4 while dormant):
   requires cohort knowledge or preserves C and O identically
   (noise hoarding — O count 204 is the poison; bounded only by
   the full working pool = effectively freezing the network).
   REJECTED on evidence: the O cohort shows the executor cannot
   tell useful dormant from noise.

B. Dormant-candidate reserve (keep M3 candidate pre-association
   across the inactive interval): the candidate KNOWS its pre
   channel (a local, label-free fact — "this channel fired with me
   before"); preserving it costs IF the mechanism slows candidate
   death (longer decay/θ_die floor) OR stalls redraw while headroom
   exists. Bounded naturally by c_slots (6/neuron, existing
   budget). Cannot distinguish C from O either — but noise O
   candidates are bounded by c_slots per neuron, and the cost is
   only "reserved slots", not protected mass. EVIDENCE-SUPPORTED
   as the smallest unit.

C. Slower/conditional pruning: M4 prune rate changes or prevents
   pruning while headroom exists. But heads-up: M4's count deletion
   is NOT the binding damage — M2 crushes masses regardless; even
   with M4 disabled the C afferents sit at w≈0.003 (insufficient,
   §1). Conditional M4 alone does not restore learnability; it only
   saves counts of useless-weight synapses. PARTIAL.

D. Synapse-local eligibility retention (per-synapse recent-coactive
   trace persisting beyond inactivity): a new per-synapse state
   (time-of-last-coactivity) that survives dormancy. It could
   justify "keep synapses that co-fired recently with my neuron"
   but again preserves O-equivalents that co-fired… O NEVER
   co-fired, so this DOES distinguish C from O — but requires a
   long-timescale per-synapse state that survives 40 s = a new
   timescale/state (the audit's criteria 5/6: new state yes; no
   identity yes). The M3 candidate pre-associated channel IS the
   existing form of this, just shorter-lived.

E. Other: none in the substrate supports better.

## 5. Resource/capacity implications

- The consolidated cap (0.75·t_e) is NOT touched by any preservation
  of dormant candidates — candidates carry near-zero w and never
  enter protected mass.
- The natural budget for a dormant reserve is c_slots (6/neuron,
  existing explicit limit) — same pool already finite; preserving
  candidates just changes their death/redraw schedule, not the pool
  size. Total = 6 × 52 = 312 candidate associations, explicit,
  finite, no new resource.
- Boundedness argument: preserved candidates hold w ≤ θ_permanent
  (0.05) and never consume M2 budget (candidates are not live
  synapses). Dormant reserve is therefore cost-free to M2/M3/M4
  budgets by construction.
- No unlimited hidden memory: the pool is pre-bounded by c_slots.

## 6. The SINGLE smallest architectural capability worth a frozen experiment

**Dormant-candidate reserve**: preserve M3 candidate pre-channel
associations across their inactive lifetime — specifically, make
candidate death/redraw conditional so that a candidate whose pre
channel has EVER co-fired with the neuron is retained (its w decays
per decay_c but stops at a floor θ_die-reserve > 0 and does NOT
redraw), until either (a) it becomes co-active again, (b) the
neuron's candidate pool needs the slot for an actively-coactive
draw, or (c) headroom is exhausted. Slots stay finite (c_slots),
the reserved candidate carries no protected mass, and no channel
identity is used — the rule is "a channel that once drove me
remains a candidate", which is pre-local (the association exists in
the candidate's pre field; the history "co-fired before" exists as
the candidate having been drawn-and-accumulated, itself a local
fact).

Why this is the smallest: (1) it operates on the EXACT structure
that already encodes dormant channel relevance (M3 candidate pre);
(2) it is bounded by the existing c_slots; (3) it consumes none of
the consolidated or M2 budgets; (4) it is cohort-blind and
label-free; (5) it is the unique existing state variable that
survives "the channel fired with me" semantics — every alternative
(A/C/D) either hordes noise, adds a new timescale, or needs
identity. The result of preserving it: at C-block onset the
C-channel candidates survive (instead of being redrawn to random
channels at t≈7 s), so permanence accumulation can begin from the
THIRD presentation onward instead of after redraw churn.

FALSIFIABLE against the six required outcomes:
1. blocked A→C: C-block permanence rate rises toward A-block rate
   (measured 1.8 → target comparable); second-block protected mass
   ≥ 0.5 × first-block.
2. alternating il: coexistence intact (the reserve preserves
   candidates already active in il — no degradation path).
3. M2 bounded: reserve holds no live weight; M2 budget untouched
   (P ≤ cap invariant).
4. M4 not permanently disabled: redraw-unused candidates still die
   when pool pressure demands; only "ever-coactive" candidates are
   retained, and only within c_slots.
5. protected cap unconsumed: reserved candidates carry w ≤ θ_die-
   reserve ≈ 0.005, never enter P.
6. no hidden context/address: the rule reads only candidate.pre +
   candidate w + pool occupancy + headroom — all local, no channel
   membership test beyond the candidate's own pre (which is not a
   cohort label; it is the synapse's existing identity field).

The smallest experiment to falsify: blocked bac/bca × 3 seeds with
the reserve active, same e24 cell/seeds/params as the previous
execution, endpoints as in §6 of the rule protocol (second-block
protected ≥ 0.5 × first-block; il F1/F7/F3 preserved; F4 zero
failures; F5 static).

IMPORTANT: if after this experiment the reserve does NOT lift the
blocked second-block rate, the correct conclusion is that M2's mass
destruction of the dormant SYNAPSES (not the candidate pool) is the
binding limit for permanence-rate AND the permanence rate is not the
limiting step — i.e., the bottleneck is then the intra-block
candidate-flux itself and the next smallest question becomes
M3-flux augmentation, NOT another preservation layer.

F2/F6: the previously recorded correction to reps 11-20 is retained
for future capability protocols; historical verdict untouched.

STOP — audit complete; nothing implemented, nothing run, no
mechanism proposed beyond this single smallest capability.