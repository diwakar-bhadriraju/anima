# Dormant-candidate reserve — semantic freeze (no implementation yet)

Status: SEMANTIC FREEZE, 2026-09-21, per the substrate-preservation
audit (docs/x-clla-substrate-audit.md). Read-only; no code, no runs,
no tuning, no E-number. Resolution of the two load-bearing semantics
plus the experiment definition, per the accepted audit.

## 0. What the reserve preserves

A candidate is the triple (pre_channel, post_neuron, w). The reserve
retains EXACTLY this triple across inactive intervals. It retains
nothing else: no pattern identity, no channel-GROUP membership, no
trial index, no curriculum order, no global state. The pre_channel
is the synapse's own identity field (D8 sensory source address) —
the same locality as STDP eligibility ("this pre fires at me").

## 1. Candidate-pool pressure semantics (six slots full, new co-active input)

### 1.1 The pressure condition (when it can even arise)

Today a slot frees on every death/permanence (redraw fills it), so
"full pool" never blocks a draw. The reserve removes death-churn for
dormant candidates; pressure becomes binding iff:

    P1. all c_slots candidates are reserved-or-eligible-waiting, AND
    P2. a co-active input channel fired on this neuron this window
        that has NO candidate in the pool.

P2 is locally detectable: the neuron's per-window fired-channel set
(already accumulated for R via the delivery split) vs the pool's
6 pre fields — a per-window local scan, no global operation.

### 1.2 Eviction rule (deterministic, local, no new state)

When P1 ∧ P2, evict exactly ONE candidate, in this priority order:

    1. any un-reserved candidate (lowest w; tie: lowest pool index)
    2. any reserved candidate at the floor (lowest pool index)
    3. any eligible-waiting candidate (lowest pool index)

then draw the missing fired channel as the replacement.

Justification by evidence, not identity: un-reserved candidates have
zero co-activity evidence (weakest claim); reserved-at-floor have
"once co-fired" evidence but no recent signal; eligible-waiting have
the strongest claim (would have consolidated) — evicted only when six
eligible-waiting candidates block a genuinely new source, and only by
deterministic index. Pool index is a stable deterministic order
(swap_remove/push under fixed seeds) — an existing tie-break
convention (cf. M5's lowest-SynapseId).

### 1.3 Invariants

- NO unlimited candidate growth: pool size stays exactly c_slots=6.
- NO global comparison: eviction reads only this neuron's pool.
- NO labels: P2 tests "this firing source has no candidate" — the
  source is the actual firing channel, like an eligibility trace.
- NO hidden address: the pre field is the synapse's own identity.
- New future inputs ALWAYS acquire a slot: P2 ∧ P1 ⇒ eviction ⇒
  draw. If fewer than 6 candidates are reserved, un-reserved
  candidates are evicted first, so new sources displace only
  zero-evidence entries until the reserve genuinely saturates.

### 1.4 No age/timestamp state introduced

The reserved bit (§3.1) is set at the instant it is DERIVABLE from
existing state (first accumulation pushes w > w_c_init); it is not
an age, a timestamp, or a recency measure. Eviction uses w and index
only.

## 2. Protected-headroom exhaustion semantics

Invariant (absolute): P ≤ p_max_frac·t_e is NEVER bypassed; a
candidate cannot consolidate without headroom.

Behaviour when a candidate reaches θ_permanent but
P + w_c_permanent > p_max_frac·t_e:

    1. The candidate does NOT create a synapse.
    2. It enters eligible-waiting: pinned at w = θ_permanent
       (no further accumulation — clamp; no decay below θ_permanent
       — the eligibility is retained, not re-earned).
    3. It occupies its slot (counts toward c_slots; part of the
       finite pool).
    4. At EVERY subsequent M3 window the headroom check is retried:
       when P + w_c_permanent ≤ cap (headroom reappears, e.g., via
       the weak consolidated-LTD leak measured by F7), the candidate
       consolidates through the STANDARD permanence path (entry gate
       re-evaluated per window — no bypass, no new mechanism).
    5. On pool pressure (P1 ∧ P2) it is the LAST candidate evicted
       (§1.2.3); eviction discards it, deterministic.

Notes:
- Under the bounded design P is nearly absorbing (consolidated-LTD
  is the only shrink path), so headroom rarely returns; eligible-
  waiting candidates may hold slots for long intervals — bounded by
  c_slots, never by protected mass.
- A candidate NEVER bypasses the cap at any tick: the permanence
  trigger and the headroom check are the SAME existing entry-gate
  code path, re-checked per window.

## 3. Reserve death semantics

### 3.1 The reserved bit

- One bit per candidate (`reserved: bool`), set the first time the
  candidate's w strictly exceeds w_c_init (0.01) — i.e., after its
  first co-active accumulation (Δperm·β·g > 0 pushes w above the
  draw weight). At that instant the fact "this pre co-fired with
  this post" is locally observable; the bit stores it.
- Not serialized flag-off; no RNG; no new state size beyond 312
  bits (6 slots × 52 neurons).

### 3.2 Decay while dormant

- Reserved candidate: w decays per the EXISTING decay_c (0.99 per
  window — no new decay timescale) down to
  theta_die_reserve := theta_die (0.005, EXISTING constant).
- w NEVER goes below theta_die while reserved: the death trigger
  (w < theta_die → redraw) is suppressed for reserved candidates.
  Repeated non-coactivity CANNOT remove a reserved candidate —
  this is the reserve's purpose.

### 3.3 Return to ordinary dynamics

A reserved candidate rejoins normal candidate dynamics when ANY of:

    - its pre is co-active again (w accumulates from the floor via
      the standard Δperm·β·g path; can reach θ_permanent → ordinary
      permanence path, subject to §2);
    - it is evicted by pool pressure (§1.2) → redraw.

### 3.4 Can a reserve slot be permanently occupied?

Yes — a reserved candidate may hold its slot indefinitely while
dormant (that is the intended capability). The pool is still finite
(c_slots=6); a permanently dormant reserve occupies at most 6
slots/neuron and consumes NO protected mass and NO M2 working budget
(candidates are not live synapses). Permanent occupation is bounded,
not unlimited: it is exactly c_slots per neuron, the existing
resource, repurposed from random-churn storage to reserved storage.

## 4. NO-MAGIC test (proof of local-pre-only retention)

Retained state: (pre_channel, post_neuron, w, reserved_bit).
NOT retained: A/C/BAC/BCA labels (channels are sensory sources, not
labels; cohort grouping exists only in analysis instruments),
trial identity (no counter), curriculum order (no sequence
memory), global context (no cross-neuron read, no stage state).

Proof obligations for implementation review (F5-style):

    R1. The reserved bit is set ONLY from one candidate's own
        accumulation event (its pre fired at its post).
    R2. Eviction keys are w and pool index only (this neuron's pool).
    R3. Permanence path is the unchanged entry gate (§2 step 4).
    R4. No code path reads presentation index, stage, pattern id,
        or channel-group membership.
    R5. Flag-off: the bit, the pooling telemetry, and all reserve
        code paths are unreachable; byte-identical FNV.

## 5. Distinguishing the two failure modes (instrumentation)

The experiment must separate A (preservation failure: dormant
candidate lost before its pattern returns) from B (candidate-flux
failure: survives but too few co-active permanence events after
return). Candidate pool state is NOT currently in telemetry, so a
flag-gated telemetry row is added (instrumentation only, no
mechanism, no identity impact):

    CandidatePool { tick, neuron_id, pool: [(pre, w, reserved, eligible_waiting)] }
    emitted every 1000 ticks (matches snapshot cadence), flag-ON only.

Per blocked run, measured read-only from these rows:

    - candidates surviving the inactive block: count of reserved
      candidates with pre ∈ eventual-second-block channels at the
      first pool row after block onset (analysis-side cohort
      bucketing — instrumentation, not mechanism);
    - their w at reappearance (should be ≥ theta_die = 0.005);
    - permanence accumulation rate in the second block
      (existing PERM events, per cohort);
    - time to first permanence (first C-PERM tick − block onset);
    - number of surviving relevant candidates;
    - second-block protected mass (existing endpoint).

Failure-mode classification:

    A: zero (or near-zero) surviving relevant reserved candidates
       at onset ⇒ reserve failed to preserve (mechanism bug or
       wrong floor).
    B: relevant candidates survive at w ≥ 0.005 but second-block
       permanence rate stays ≈ 1.8/pres ⇒ candidate-flux
       (M3-coactivity throughput) is the binding limit — the audit's
       predicted next question; the answer is M3-flux augmentation,
       not another preservation layer.

## 6. Falsifiable minimal test (frozen design; NOT executed)

Matrix (mirrors the allocation-rule matrix, + the reserve flag):

    1  identity run (flag OFF): byte-identical FNV
       9647ea8a0ca4dbd2 (152,254 rows) + 105/105 frames
    3  bac × {20260912, 424242, 9001} (reserve ON)
    3  bca × {20260912, 424242, 9001} (reserve ON)
    3  il × {20260912, 424242, 9001} (reserve ON) — coexistence check
    3  d × {20260912, 424242, 9001} (reserve ON) — informative only
   --
   13  total

Config: e24 cell, seeds/timings frozen, p_max_frac 0.75,
w_consolidate_min 0.05, c_slots 6, b_e 40 — everything equals the
accepted rule-run set; the ONLY new config is `dormant_reserve`
(default false) + the CandidatePool telemetry flag (same gate).

Success/failure criteria (frozen, binary):

  S1. SECOND-ARRIVAL STARVATION PREVENTED: second-block protected
      mass ≥ 0.5 × first-block protected mass at drive end,
      6/6 blocked arms (primary endpoint).
  S2. ALTERNATING PRESERVED: il F1 (raw masses ≥ 0.5× refs) 3/3;
      il protected ratio 0.4–0.7; headroom > 0 at drive end.
  S3. CAP INVARIANT: P ≤ 0.75·t_e + 1e-6 at every snapshot, every
      neuron, every run (F3 unchanged).
  S4. M2 TARGET: t_e − P > 0 everywhere (W-target strictly positive).
  S5. STABILITY: 0 failures (F4) across all 13; no new failure/
      resource class.
  S6. FINITE POOL: pool size == c_slots at every CandidatePool row;
      no new resource, no hidden state (F5 static: R1–R5 pass).
  S7. FAILURE-MODE SEPARATION: §5 classification reported; A and B
      are mutually exclusive outcomes of the same run set.

VERDICT: reserve-supported iff S1–S6 all pass (S7 recorded as the
mechanism diagnosis regardless). This experiment answers ONLY:
"Does preserving dormant pre-associations remove second-arrival
substrate starvation?" No memory-capability claim; no E-number;
historical verdicts untouched.

## 7. F2/F6

The previously approved correction (per-pattern reps 21–40 →
reps 11–20) is retained for future capability experiments only. Not
rerun here; historical verdicts not altered.

STOP — semantics frozen; no implementation, no runs, no tuning.