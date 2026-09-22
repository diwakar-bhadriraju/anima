# Phase III Level-4 «I» — frozen design: local inhibitory gating of temporal prediction

Status: FROZEN DESIGN (not implemented). 2026-09-22. Branch 1 (inhibitory
gating). Supersedes the CLOSED eligibility-LTP family (D-09/D-10).
No runs until this design is internally complete. Protocol: freeze ->
gate -> execute -> audit -> verdict; no post-hoc tuning.

---

## 0. The problem (re-framed from the falsified evidence)

Requirement to satisfy:
  persistent predecessor-specific gap state exists
  AND LTP-based consolidation destroys it
  => prediction learning currently consumes the substrate it needs.
We must break that coupling.

## 1. Causal failure analysis (frozen evidence)

Substrate: LIF + pairwise STDP, 24 in / 40 pool (ids 24..64) / 12 out
(64..76), all-excitatory, local, finite, identity-gated. Curriculum:
deterministic alternation (il), 500 ms presentation, 1500 ms gap.

Base E-nogain (committed, no mechanism) — the substrate we must not lose:
- pool sustains predecessor-specific cross-gap firing: late-gap internal
  state predecessor-distinct in 2/3 seeds (cross A-C cosine 0.625 / 0.759
  vs within ~0.99); pool late-gap ~40-239 spikes/gap; OUTPUT fires in the
  gap too (162.9/gap, seed s20260912) — a real readout-level temporal
  substrate. Pooled prediction index PI ~ +0.02 only (per-pattern split
  +-0.3 is a rate-decay artifact, not anticipation). So base = memory of
  "what came", not prediction of "what's next" — the missing piece is
  LEARNING the transition into the next-event anticipation.

Falsified mechanisms (both identity-gated, FNV 9647ea8a0ca4dbd2):
- d_elig (slow-trace LTP on ALL edges): pool+out gap firing -> 0; rates
  collapse (A 50.6->21.0, C 159.2->59.4 Hz).
- d_elig_ro (slow-trace LTP readout-only): pool gap WEAKENED (5.7-81.9
  vs base 40-239), OUTPUT gap -> 0 in all 3 seeds (output still fires
  147-189/pres). Pool PI ~ 0, out PI = 0.

LEADING CAUSAL HYPOTHESIS (evidence-tagged; used as a hard constraint):
the slow eligibility trace was implemented as a SUBSTITUTE for the fast
20 ms pre-trace inside the SAME LTP formula (both LTP sites set
ltp_trace := elg), not as an additive second pathway. Rewiring this
stability-critical within-event potentiation starved/changed the
self-sustaining dynamics that produce BOTH the presentation burst AND
its persistent tail; the pool slid toward a quieter, stimulus-locked
regime and the tail (the very substrate for anticipation) collapsed.
YELLOW: the exact sub-mechanism is not fully isolated (slow-trace
magnitude elg~1 vs pre_t<<1 also raised LTP size), but the DESIGN
CONSTRAINT is robust to that uncertainty: any learning pathway added for
prediction MUST NOT perturb the recurrent pool's fast within-event
STDP, which is verified by an identity gate on the pool's dynamics.

Corollary (the coupling to break): the pool is BOTH the generator of the
predecessor tail AND a locus of consolidation. Therefore prediction
learning must (a) live on a SEPARATE readout pathway (pool -> out), (b)
use an ADDITIVE slow trace there, never a substitute, and (c) be GATED so
its high-rate window (the event onset, where consolidation runs away and
locks the output phasic) is exactly the window where learning is
suppressed.

## 2. Which inhibitory form does the evidence support?

Investigated all four from the brief; the evidence picks a combination.

(a) Pure feedforward onset inhibition (IN caps pool peak). Partial:
could keep the pool out of the consolidated phasic-lock, but does NOT by
itself time or gate the readout learning; without a second signal it
leaves the substitution hazard unresolved. Not sufficient alone.

(b) Inhibitory gating of plasticity (chosen, primary). The gate is
activity-derived, so it is HIGH where the drive is strong (event onset /
peak) and LOW in the gap trough (where the predecessor tail lives and
the next event is pending). Making readout slow-LTP inversely scale with
this gate directly closes learning in the runaway window and opens it in
the state-relevant quiet window. This is the causal tie: the gate bounds
consolidation where consolidation would otherwise destroy the substrate.

(c) Local disinhibition at event transition (chosen, secondary). The
gate's open window is aligned to the LATE stage of the gap (the
predecessor tail is predecessor-distinct there; PI feasibility showed the
late-gap structure), so the readout learns the pool-tail -> next-output
association exactly when the tail is informative and the output is not
being driven by live stimulus.

(d) "More inhibition" as a blanket: REJECTED — the brief warns against
assuming it; shunting is applied only at the onset burst to cap the
peak, sized so formation (S1 6/6) still passes, and its gate role is
timed (event-active), never tonic across the whole gap (which would
erase the tail we need).

Hence the mechanism = (b)+(c): a local event-driven inhibitory gate that
(shunts the onset peak to protect the pool) AND (opens readout
slow-LTP only in the late-gap trough). The pool's fast STDP is left
byte-identical.

## 3. Mechanism specification (frozen)

FLAG: d_ing (identity-gated; false => no INs fire, no gate, additively
byte-identical to the committed E-nogain baseline D-> pass FNV gate).

### 3.1 Added substrate (minimal, finite, local, unlabeled)
- A small fixed population of INHIBITORY interneurons, Nin = 8, ids
  76..83 (after output), neuron type Internal-inhibitory. Each IN:
  - receives feedforward input from a LOCAL radius of the input channels
    via fixed (non-plastic) seeded weights (same seeded p_rec/w_rec
    convention as the base; locality = the seed connects each IN to a
    contiguous channel neighborhood — NO semantic labels, no A/C
    identity, deterministic).
  - projects shunting inhibition onto a local cohort of pool neurons and
    onto the readout edge, fixed seeded wiring, negative sign.
  - is itself a standard LIF neuron (finite, refractory, no plastic
    afferents; plastic-free so it cannot itself consolidate).
- No new global signals, no external memory, no predictor.
- RNG SAFETY (identity): the 8 INs' wiring is drawn from a SEPARATE
  deterministic sub-stream, Xoshiro256PlusPlus::seed_from_u64(fnv1a(
  "d_ing", run_seed)) - NEVER from the base construction stream. The IN
  sub-network (neurons + fixed IN afferents + IN->pool/out projections)
  is constructed IDENTICALLY regardless of the flag value, and the base
  network draws ZERO extra RNG samples when d_ing is on. Hence flag-off
  construction is byte-identical to the committed E-nogain baseline by
  construction (INs present-but-silent or absent both preserve the same
  base draws), keeping the FNV identity gate exact.

### 3.2 Local signals (item 1)
- elg_i: existing per-neuron slow spike trace (tau 1500 ms, bounded 1.0)
  — RETAINED, but now USED ONLY on readout (pool->out) LTP AND only
  when the gate is open. Never on recurrent (pool) edges.
- g_c(t): per-cohort gate = the cohort's IN firing rate, low-passed with
  tau_g ~ 300 ms (>= event window, < gap). g is the decayed inhibitory
  output current of the cohort's 8 INs. Normalized to [0,1] by a fixed
  divisor derived from the base peak (a constant, not learned).
- Gate function: gate(t) = 1 / (1 + k_g * g_c(t)), k_g pre-frozen
  (e.g. 2.0). gate -> ~0 during strong event drive; -> 1 in the trough.

### 3.3 What is inhibited / protected (items 3,5)
- During event onset (g high): the cohort's pool neurons receive
  shunting inhibition => the phasic onset peak is capped. Purpose: keep
  the pool from sliding into the consolidated stimulus-locked regime and
  bound the drive that would otherwise feed runaway readout-LTP.
- The pool's FAST within-event STDP: UNTOUCHED (identity). This is the
  explicit fix vs the falsified family (no trace substitution).
- The gate CLOSES readout slow-LTP while cohort c's own g_c is high
  (item 5: closes at a cohort's own onset, opens as its drive recedes
  through the NEXT event's onset — per-cohort, per §3.5/3.5a, NOT a
  global "event onset / late gap" sweep).

### 3.4 What remains plastic (item 4)
- Recurrent pool (internal->internal): BASE pairwise STDP only.
- Input->pool afferents: BASE STDP only (formation, unchanged).
- Readout pool->output: base fast STDP PLUS, when the gate is open, an
  ADDITIVE slow term:
      dw += a_elig * gate(t) * elg_pre * (1 - w/w_max)
    (a_elig frozen ~ a_plus; bounded by w_max; elg_pre = the pool PRE's
    slow trace — so "this pool neuron was active back in X's gap").
- INs: no plastic afferents.

### 3.5 Gating timing (item 2) — PER-COHORT, keyed to the PRE
The gate is LOCAL and keyed to the PRE's own cohort: gate_c is closed
when cohort c's own event drive is recent/strong, OPEN when cohort c has
receded into its predecessor-tail regime. IMPORTANT invariant (fixes an
earlier draft's inconsistency): the readout slow-LTP's post is the
NEXT event's output response, which arrives while the PRE's cohort
(X's cohort) is long quiescent — so gate_c is open at exactly the
moments the association needs to fire.
- Closed: from a cohort's own onset through ~one peak window (~300 ms).
  Rationale: while cohort c is freshly driven, (i) its pool is prone to
  consolidation into phasic-lock (the d_elig_ro failure), and (ii) its
  output drive is live-stimulus — learning there would lock output to
  the present event, not the next. So NO slow readout term on c's
  synapses and shunting caps c's onset peak.
- Open: once cohort c's drive has receded (mid/late-gap AND through the
  next event's onset). Rationale: c is now in its predecessor-tail
  regime (elg_pre>0, predecessor-distinct, low-rate -> small
  self-limiting increments), and the next event's output response
  (the post) arrives just here. The association pool(X-tail)->output(Y)
  therefore fires at Y-onset.
- The association cannot fire during X's own onset (X-cohort closed) nor
  at X's output drive (same reason) — only X-tail -> next-Y. This is the
  precise break of the "learning consumes its own substrate" coupling.

### 3.5a Why Y-learning is NOT gated shut (robustness, explicit)
The gate is keyed to the PRE's cohort (gate_c for synapses whose pre is
in cohort c), never to the post/current drive. Timing arithmetic: the
PRE (X-tail) fired >= 1500 ms before the next (Y) onset, while the gate
low-pass tau_g ~ 300 ms. Since 1500 ms >> tau_g (3 low-pass time
constants), g_c(X) has fully decayed to ~0 by Y onset, so gate_X is
~1 through the ENTIRE Y presentation (X never re-fires during Y to
re-arm its own gate). Hence the open window covers the output's full
Y-response with wide margin — no dependence on catching a fast transient.
The only synapses gated shut at Y onset are those whose PRE is in Y's
own cohort (Y->Y self-association), which is exactly the term we want
suppressed. Thus "gate closed whenever the post fires to Y" is false:
it is closed only for Y-self pre/source cohorts. tau_g need NOT be
tuned against the window; the 1500 ms gap dominates.

### 3.6 Stability bounds (item 6)
- All weights in [0, w_max]; readout slow term capped by (1 - w/w_max)
  and by gate<=1 -> no unbounded growth.
- IN afferents fixed (<= w_cap_in); IN firing bounded by LIF refractory.
- gate() ~ 1 in the trough => max readout increment a_elig*elg<=a_elig;
  in practice the trough is low-rate so effective increments are small.
- Formation gate: S1 6/6 must still pass (same stored convention);
  failures = [] (no runaway) required.
- No RNG in the mechanism (deterministic; identity).

### 3.7 Resource cost (item 7)
- +8 neurons (out of a finite pool; negligible), +1 scalar/neuron (elg,
  already present), +1 gated LTP term on readout edges, +fixed IN wiring.
  No external memory, no global tables.

### 3.8 Why the predecessor state remains available (item 8)
- The producer of the tail (recurrent pool fast STDP) is untouched by
  construction (identity-gated; pool fast LTP never substituted or
  gated-off). The onset shunting caps only the peak, not the low-rate
  tail. Therefore the pool continues to sustain and preserve the
  predecessor-specific gap state as in base.

### 3.9 How Y learning still occurs (item 9)
- After X, the X-cohort pool sustains the predecessor tail (elg_pre>0 on
  the X-active neurons; X-cohort gate is OPEN because X has receded).
- When the NEXT event Y drives the output, the output's Y-response fires
  as the LTP post while those X-neurons are still the eligible PRE
  (gate_X open, elg_pre ~ e^-1 of peak). The open-gate slow term
  potentiates pool(X-tail)->output(Y) readout synapses.
- Repeating the alternation accumulates: after X, the pool tail
  increasingly drives the output toward Y's response BEFORE Y arrives —
  anticipation expressed as output gap firing that resembles Y more than
  X (measured PI_out = cos(out_gap_after_X, Y_ref) - cos(out_gap_after_X,
  X_ref)). Learned additively on the readout; pool fast STDP untouched;
  gate_X closed during X's own onset prevents output from locking to X's
  live drive (the d_elig_ro failure).

## 4. Endpoint (frozen falsifier)
PRIMARY (adopt if all hold across >= 2/3 seeds, il, training then test):
- output PI = cos(out_gap_after_X, Y_ref) - cos(out_gap_after_X, X_ref)
  > +0.05 reproducibly in the LATE gap (mean over presentations), AND
- the pool tail is preserved: pool late-gap firing non-empty (>= base
  order of magnitude) with late-gap predecessor-distinctness (cross
  cosine < within - 0.1) maintained, AND
- formation not regressed (S1 6/6), failures = [].
SECONDARY (mechanism truth): for each cohort c, gate_c high during c's
own onset, ~0 in c's late gap / next-onset window; readout slow-LTP
events cluster in the pre-next-onset window of the PRE cohort.
FALSIFY / reject (no tuning): output PI <= 0, OR pool tail destroyed, OR
S1 regressed, OR runaway.

## 5. Identity / integrity
- Flag off (d_ing=false): INs do not fire, gate(t)=1 (fixed), no slow
  term (a_elig*0), pool/readout exactly base STDP -> flag-off run must be
  byte-identical to the committed E-nogain baseline (event FNV
  9647ea8a0ca4dbd2, snapshot frames). Identity gate BEFORE the matrix.
- Unit tests: gate(high drive)->~0 / gate(trough)->~1; readout slow term
  additive capped by w_max and by gate; pool fast STDP unchanged when
  d_ing on; flag-off byte identity; no label/drive leak into wiring;
  RNG INVARIANCE: base wiring consumes identical RNG samples with d_ing
  on vs off (IN wiring from the fnv("d_ing", run_seed) sub-stream, so
  flag-off construction is base-identical by construction).

## 6. Execution order
1. This design frozen & committed. 2. Implement d_ing (identity-first).
3. Unit tests + full suite. 4. Identity gate rerun (flag off). 5. Run
3-seed il matrix (d_ing) + reuse base controls. 6. Measure output PI,
pool preservation, formation, gate timing. 7. Verdict (adopt/reject);
autonomy-log. No post-hoc tuning; runs preserved incl. failure.

STOP — frozen design. Awaiting approval to implement d_ing.