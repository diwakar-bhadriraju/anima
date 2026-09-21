# First-exposure allocator — implementation/spec audit (READ-ONLY)

Status: 2026-09-21. Source-trace audit of the closed fe experiment
(e6f9c7a) plus the frozen spec (fb11b67). No implementation, no
runs, no tuning, no E-number.

## 1. Source observability — exact traces

Two distinct observation channels exist in the code:

(A) `fired: Vec<bool>` — V2Plasticity.tick (structural_v2.rs:138):
    `for &n in spikes { self.fired[n.idx()] = true; }`.
    The harness passes `step.spikes` (harness.rs:383), which
    CONTAINS INPUT-CHANNEL NEURON IDS (0..23): network.rs
    deliver_input returns channel targets as spikes. Therefore the
    plasticity module already observes "input channel j fired this
    window" for EVERY channel, with NO synapse and NO candidate.
    THIS is the vector M3 co-activity uses (`fired[pre.idx()]`,
    structural_v2.rs:352) and M6 uses (line 667).

(B) `fired_channels[post]: BTreeSet<u32>` — accumulate_input_current
    (structural_v2.rs:175-205): for each fired pre<24, iterates
    `net.outgoing[pre]` (LIVE synapses), and inserts pre.0 into
    fired_channels[s.post] for each live target. Requires a LIVE
    afferent j→i. Candidates are NOT in net.outgoing (they live in
    self.candidates), so a pooled candidate does NOT make j
    observable via this path.

Per case:
- live afferent j→i: observable via BOTH A (fired[j]) and B
  (fired_channels[i]∋j) and via membrane current i_syn.
- pooled candidate for j, no live afferent: observable via A ONLY
  (fired[j]); NOT via B; NOT via membrane.
- neither: observable via A ONLY (fired[j]).

So the module-level M3 capability is strictly richer than the
synapse-level view — and M3 ALREADY relies on it for co-activity.

## 2. Current redraw path — why the bias is vacuous

Trace: death/permanence redraw → candidate_pass computes
`fe_fired = fired_channels[post].clone()` (alloc rule on) → passes
to draw_candidate → bias loop:

    for &ch in fired:
        if pre < n_input && !connected(pre): return pre
    // fall through to random

`connected(pre)` (draw_candidate) = a LIVE incoming synapse from
pre OR a pooled candidate with pre. But fired_channels[post] by
construction contains ONLY channels that have a live afferent to
post (§1-B). Therefore:

    ch ∈ fired_channels[post]  ⇒  connected(ch) == true  ⇒  skipped

The bias ALWAYS falls through to random. Structurally vacuous, in
the exact configuration of the fe experiment (no reserve, so all
bound candidates are un-reserved and eviction is the only bind
path, requiring a full pool).

## 3. Locality

The question "can a post neuron observe channel j firing without a
synapse" has TWO answers depending on the architectural boundary:

- PER-SYNAPSE/SOMA boundary (membrane current i_syn): NO — a
  neuron only receives current through afferents. A channel with
  all afferents pruned is invisible to the neuron as a biological
  object.
- PLASTICITY-MODULE boundary (V2Plasticity): YES — the `fired`
  vector is a per-window record of ALL spikes, input channels
  included, and M3 co-activity and M6 ALREADY read it for candidate
  pres that may have no live synapse. This is the ARCHITECTURAL
  FACT: the frozen mechanism's own permanence accumulation is not
  synapse-gated; it is spike-record-gated.

The fe implementation chose the synapse-level source (fired_channels)
even though the module boundary (fired) was already observed by the
same mechanism it was trying to accelerate. The global-list caveat
of the mandate ("do not assume a global list is available to every
neuron") is satisfied in the SYNAPSE reading but NOT REQUIRED by
the existing M3 boundary, which is module-level by design. State
this explicitly: under the module boundary, observability without a
synapse EXISTS and is already load-bearing; under the synapse
boundary it does not.

## 4. Two semantics — which is true

A (module sees the channel event without an afferent): TRUE in the
implementation — via `fired[j]` — and already used by M3 co-activity.
B (only through an existing afferent/candidate): TRUE only for the
afferent-derived view (`fired_channels`, membrane); false for the
module view.

The fe implementation bound itself to B's source (fired_channels)
and thereby made the redraw bias vacuous (§2), while A's machinery
(fired) sat unused in the same object.

## 5. Visibility cap

What was reported (~0.71/neuron at t=44k): LIVE AFFERENT SOURCES —
the count of C channels with a surviving synapse to each neuron.
It is NOT:
- currently represented input sources (pool held 108-124 C-cohort
  candidates at switch — more than the 37 afferent sources);
- source events observable by the neuron at the module boundary
  (all 8 C channels fired every presentation; `fired[j]` true for
  all of them every window).

CONCLUSION: the visibility cap is an ARTIFACT of the chosen
observation source (fired_channels), not a fundamental
architectural limit. The fundamental limit would require strict
per-synapse locality, which the implementation does not have (M3
co-activity already transcends it). A corrected source (fired)
removes the cap without changing the locality boundary at all.

## 6. Counterfactual — corrected redraw bias (read-only, committed traces)

IMPLEMENTABLE counterfactual (uses only existing `fired` vector +
existing redraw sites; no new state, no locality change):
bias sources from `fired` (module record) instead of
`fired_channels`: on redraw, try channels with fired[ch] && ch<24
&& !connected(ch). connected() then excludes only channels with a
live-afferent duplicate (0.71-1.25/neuron) or a pooled duplicate;
the remaining ~6.3-7.3 C channels per neuron become bindable on
every redraw.

Expected scale (from committed bac data):
- pool death-churn redraws: the 108-124 C-cohort candidates at
  w 0.007-0.016 die within ~7 s (θ_die/decay) — that is ~120
  redraw events in early block 2, each bindable to a fired C
  channel under the correction instead of random.
- permanence per bound candidate: measured ~1-2 presentations
  (first C permanence at 2.4 s), g=1.0 (R=0).
- additional C permanence events: bounded by slot cycle
  (6 slots × 52 neurons, each maturing in ~1.5-2 presentations)
  ≈ 150-250 events across 20 presentations, i.e., approaching the
  A-block scale (152-248) and ≥ 0.5× first-block protected mass.
- This is the counterfactual's strong form: supply gap (35-41 →
  150+) explained entirely by the vacuous redraw bias.

IMPOSSIBLE/locality-violating counterfactual (NOT proposed): binding
to a channel the neuron cannot observe even at the module boundary
(e.g., a channel that never fired this window, or a future pattern
not yet presented). That remains impossible under both boundaries —
and is also not needed: the second block IS firing when it must be
bound.

The two counterfactuals are cleanly separable: corrected-source is
implementable and testable; anything prognosticating with no firing
record is not.

## 7. Spec status

Frozen design language (fb11b67 §8): "modify draw_candidate to
prefer the per-neuron fired-channel set when the allocation rule is
on: on any redraw, FIRST try channels in fired_channels (loop in
channel-id order, skip connected/pooled, skip already-pooled pres),
fall back to the existing random draw."

Implementation: EXACTLY this. fired_channels was the chosen source,
connected() the chosen skip, fallthrough as specified. The
implementation is faithful to the frozen spec.

Therefore correcting the redraw source is NOT a bug fix (no
spec/impl divergence); it is a PROTOCOL AMENDMENT — a spec-language
change of the observation source from fired_channels to the
module's fired record. It is the SMALLEST possible amendment: same
function, same redraw sites, same skip semantics (minus the
vacuous self-exclusion), zero new state, zero new parameters,
identity unchanged flag-off. It is justified by the spec's own
intent ("currently firing local input" — the audit's §10 opening)
and by the implementation fact that the module already holds the
firing record for M3 co-activity.

## 8. Next experiment (frozen design; NOT executed)

Smallest distinguishing test: the SAME 13-run matrix (identity +
bac/bca/il ×3 + d×3 informative) with the ONE amendment — draw
bias sourced from `fired` instead of `fired_channels`. Nothing
else changes.

Endpoints (unchanged from the fe protocol):
- S1 primary: second-block protected ≥ 0.5 × first-block, 6/6;
- S2-S6 preserved (il, cap, M2 target, stability, finite pool);
- causal path re-measured: C-cohort permanence rate; time to first
  permanence; second-block supply vs A-block scale.

Interpretation (frozen):
- supply jumps toward A-block scale (~150+, S1 passes 6/6) ⇒ the
  failure was the IMPLEMENTATION-PATH DEFECT (vacuous redraw bias);
  corrected fe allocation is the missing capability.
- supply stays ~35-41 despite the corrected source (S1 still
  fails) ⇒ a FUNDAMENTAL limit remains; per §5 that limit cannot
  be source observability (module view exists) — it would then be
  co-activity rate, permanence threshold, or the post-exposure M2/
  M4 churn on the NEWLY created synapses, and each is measurable in
  the same run (co-active windows, θ time, post-permanence synapse
  survival).

Deliberately NOT proposed: parameter sweeps, new thresholds, new
state, reserve revival.

## 9. Summary

- The fe failure is a SPEC-LEVEL defect, not an implementation
  error: the frozen design picked a bespoke afferent-derived source
  while the module's existing firing record (already used by M3
  co-activity) contained the required observability.
- The visibility cap is an artifact of that choice, not an
  architectural limit.
- The correction is a minimal protocol amendment, implementable
  with zero new state and no locality change, for which the
  committed traces predict a 3-6× supply increase (35-41 →
  150-250 permanence events), sufficient to reach the S1 bar if
  flux and post-permanence survival hold.

No implementation, no runs, no tuning, no E-number. STOP.