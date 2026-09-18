# ANIMA E18 — Developmental capability research design

Status: RESEARCH-DESIGN AUDIT (2026-09-18). Not a protocol; nothing
implemented, frozen, or executed. E17 closed; not reopened.

## 1. E18 research question

Can the frozen ANIMA organism develop, from exposure to a
developmental curriculum, a *temporal state* — an internal response
that encodes which of two experiences preceded the current stimulus —
that is not present in a fresh organism and not explainable by fixed
trace artifacts or schedule memorization?

This is the first experiment organized as

    developmental problem -> experience -> internal organization -> behavior

instead of

    stimulus -> representation -> measure representation.

## 2. Selected capability: ANTECEDENT-DEPENDENT PROBE RESPONSE
## (temporal state / short-term memory across a gap)

One capability: when the same probe pattern P follows antecedent A vs
antecedent C (separated by a silent gap), the organism's internal
response to P must diverge — encoding "which came before" — and that
divergence must *develop* with experience.

Why this capability, from the E1–E17 evidence:

- E9/E11/E12/E14 established that this organism is exquisitely
  sensitive to *which pattern preceded which*: B's representation
  attaches to the trailing side of its own order, and a single
  presentation of a new block context re-anchors it (E14: k* = 1).
  Preceding-experience sensitivity is real and fast — the open
  question this architecture poses is whether it *develops*
  organizational structure from such contingencies.
- E17 showed the flip survives with zero STDP/M3/M4 — re-expression
  is instant — which makes "present from trial 1" the strongest
  trivial alternative that E18's design must defeat (Section 10).
- E-series machinery (per-presentation internal vectors, cosines,
  alignment, structural buckets) already measures everything needed;
  the E12–E17 curricula already used A {0-7}, B {4-7 -> 8-11} phases,
  C {8-15}, and gap schedules.
- Unlabeled, incompatible with supervision, observable, and capable
  of informative failure (the organism's only persistence channels —
  E6 phi ~2.5 s window EMA, M6 inhibition decay ~0.98/window,
  adaptation tau 200 ms — may be insufficient; that negative is the
  research outcome, not a bug).

## 3. Research hypothesis (exact form)

"If ANIMA is exposed to a balanced curriculum of A->B and C->B
trials with an intervening silent gap (no labels, no reward), then
it will develop an antecedent-dependent probe response — the
internal response to B after A vs after C diverges beyond the
no-development baseline and grows with developmental exposure —
expressed through internal spike-response vectors and the
underlying weight/inhibitory organization, in a way that cannot be
explained by fixed trace-mediated re-expression present from trial 1
(adaptation, M6 inhibition residual, E6 phi asymmetry) or by
memorizing a deterministic schedule."

Null/failure: the divergence never exceeds the early-window
baseline, does not grow over development, is fully explained by the
fixed traces, or is absent entirely. The hypothesis is not assumed.

## 4. Environment, interfaces, experience, development, capability

A. ENVIRONMENT: the committed deterministic schedule harness; a
   seeded curriculum of trial epochs and silences. No feedback loop.

B. SENSORY INTERFACE: the 24 input channels only. Antecedents = the
   committed E12 patterns A {0-7} and C {8-15} (same rates,
   durations, jitter — frozen pattern definitions). Probe = the
   committed B {4-7 -> 8-11 two-phase} pattern. Channels 16-23 never
   active. No other information (no labels, no trial-type markers,
   no block announcements).

C. ACTION/OUTPUT INTERFACE: none (measurement-only). The observable
   "behavior" is the organism's internal response pattern; the 12
   output neurons remain passive observables. A true action->world
   loop does not exist in the architecture; adding one is the
   minimum architectural requirement for any *sensorimotor*
   capability and is explicitly NOT part of E18 (Section 14).

D. EXPERIENCE: S1 developmental block: >= 120 trials, each
   [antecedent X in {A, C} (seeded-random, balanced) -> gap G0 ->
   probe B -> inter-trial silence]. Antecedent identity is random,
   so no sequence is memorizable and B's predecessor carries no
   fixed schedule marker. S2 measurement block: same structure
   (measure with existing per-presentation machinery). S3
   persistence block: same trials with long gap G1 > G0 (gap is the
   only change — probes the memory-decay signature). S4 (optional,
   decision listed in Section 14): contingency-filter block
   (A->B-only then C->B-only runs) to test track/adaptation of the
   divergence. Antecedent order seeded and balanced at every block.

E. DEVELOPMENT: whatever emerges — M3 maturations, M6 inhibitory
   drift, E6 phi evolution, weight asymmetries across A-side/C-side
   afferents of the probe. Nothing prescribed (no neuron
   specialization, no synapse targets).

F. CAPABILITY: probe-after-A vs probe-after-C internal response
   vectors diverge beyond the no-development baseline, with the
   divergence growing across development and carrying a
   gap-persistence signature.

## 5. Information-leakage audit

Available to:
- ENVIRONMENT: the full curriculum (antecedent choices, timings).
- SENSORY INTERFACE: channel activity only; identical timing across
  trial types; silence gap; nothing else.
- ORGANISM: its afferent channels and its own internal state. The
  antecedent's identity reaches B only through (i) the silent gap —
  i.e., whatever persists internally (the legitimate substrate), and
  (ii) nothing else.
- ANALYZER: full telemetry, including trial-type labels (labels are
  analysis-side exclusively).
- EXPERIMENTER: everything.

MUST NOT reach the organism: trial-type IDs, the contingency table,
block boundaries (block timing changes are schedule structure, not
labels), any analyzer-derived quantity.

Shortcut checks:
- Direct sensory lookup: the probe input is byte-identical across
  trial types at onset (same B channels, same timing) — a response
  difference cannot come from the probe itself. PASS.
- Fixed channel identity: antecedent channels are silent during the
  probe; any influence must persist internally. PASS (that
  persistence is the tested thing).
- Stimulus frequency alone: A and C both use the committed 20 Hz —
  not rate-discriminable. PASS.
- Presentation timing alone: all trial types share one timing
  template (antecedent, G0, probe, ITI); timing encodes nothing
  about the antecedent. PASS.
- Hard-coded state IDs / analyzer labels entering the organism: no
  path exists (analyzer reads only telemetry; the harness schedule
  carries no labels). PASS — verified in the freeze by a
  no-label/no-feedback assertion on the config and harness data
  path.
- Deterministic schedule artifacts: antecedents are seeded-random,
  balanced; no fixed A->B adjacency exists to memorize. PASS.
- Trivial output mapping: no output interface. PASS.
- Memorization without temporal state: impossible by construction
  (random antecedent order); any B-response difference must derive
  from the antecedent's internal trace or from developed
  asymmetry. PASS.

## 6. What counts as "internal" (operational)

1. Externally observable behavior: the probe-epoch internal spike
   response pattern (per-presentation vectors, frozen E-series
   method) — the only behavioral readout used.
2. Internal state/representation: weight and inhibitory organization
   differences (E15 bucket machinery, snapshots at block
   boundaries); per-neuron selectivity (existing analyzer).
3. Temporal persistence: divergence measured at G1 > G0 — if the
   encoded antecedent survives the longer gap, the state persists;
   if it decays, the persistence time-constant is characterized.
4. Adaptation/plasticity: growth of the divergence across S1 (early
   vs late windows) and its reaction to the S4 contingency filter.
5. Causal dependence on prior experience: the same probe, same
   timing, same channels, differing only in what preceded it. The
   early-window baseline (trials 1-20, before any development) is
   the no-development reference.

A genuine internal state is distinguished from a direct
stimulus-response mapping by: identical probe input across trial
types (the mapping would be identical) and by developmental growth
(a fixed mapping would be constant).

## 7. Baseline / control logic (minimum set)

- EARLY-WINDOW BASELINE (within-run, free): divergence on trials
  1-20. This is the direct test against the strongest trivial
  alternative — fixed trace re-expression present from trial 1
  (E17's instant-re-expression result predicts exactly this). If
  early == late: the capability reduces to the trace; claim fails
  in its strong form.
- LONG-GAP BLOCK S3 (within-run): persistence signature; separates
  trace-decay-bound memory from developed weight-bound memory
  (weight-bound would NOT decay with the gap).
- SHUFFLED/CONTINGENCY CONTROL: one extra run with antecedent
  identity decorrelated from B trials (e.g., probes following
  silence only, antecedents elsewhere in the schedule) — included
  ONLY if the early-window baseline proves ambiguous at the freeze
  stage (registered as a decision, not a default). No factorial
  sweep.

Controls are chosen to defeat exactly two alternatives: (i)
"present from trial 1, no development" and (ii) "schedule
memorization". No others are included.

## 8. Developmental curriculum (smallest)

- Initial conditions: committed canonical organism config (E12
  base; all mechanisms on — STDP/M2/M3/M4/M5/M6/E6, adaptation);
  fresh RNG per seed; no pretraining.
- Developmental experience: S1 (>= 120 trials of the A|C -> gap ->
  B template), S2 measurement (>= 40 trials, same template),
  S3 persistence (>= 40 trials, gap G1), S4 optional contingency
  filter.
- Probe/test: the trials themselves are the test (no separate
  "test set" — measurement is continuous; block-segmented in the
  analyzer).
- Held constant: organism, parameters, patterns, rates, ITI
  structure, seed derivation per run.
- Changes over development: only accumulated experience (and the
  registered gap in S3, block structure in S4).
- Nothing prescribed internally.

## 9. Minimum measurements (existing machinery first)

- Behavioral: per-presentation internal vectors -> cos(B-after-A,
  B-after-C) divergence per trial pair within blocks (frozen
  per-epoch method; L1 == raw); B-after-A vs B-after-C alignment
  distributions; probe response vs A and vs C (E-series pair
  cosines).
- Internal organization: snapshot bucket weights by side at block
  boundaries (E15 instrument); per-neuron selectivity (existing
  analyzer); A-C separation sanity.
- Stability/persistence: block means + early/late split; S3
  divergence at G1.
- Adaptation: S4 (if included) divergence per filter phase.
- Resource: failures, snapshot rates, engagement (existing gates).
- No thresholds invented here: growth criterion, steady-state
  divergence, and persistence decay cutoffs are registered
  pre-registration decisions at the freeze stage.

## 10. Trivial-solution attack (adversarial)

1. FIXED TRACE RE-EXPRESSION (strongest): adaptation/M6/phi make
   the probe response differ from trial 1 with no development.
   Detected by: early-window baseline — growth required. If the
   effect is flat, the strong claim fails by design.
2. SCHEDULE MEMORIZATION: impossible (seeded-random antecedents) —
   detected by construction.
3. GAP-TIMING ARTIFACT: a longer gap could change internal state
   globally (fatigue), mimicking divergence changes. Detected: S3
   compares like-with-like (same trial structure, longer gap) and
   the divergence metric is relative to the same-block early
   baseline; a global activity drop would affect both trial types
   symmetrically.
4. ANTECEDENT-LENGTH/ENERGY ASYMMETRY: A vs C could differ in
   total drive (channel counts are 8 vs 8; same rates — verified
   identical marginal exposure in E12 configs; the freeze asserts
   A/C exposure equality).
5. ANALYZER-LABEL ARTIFACT: alignment labels computed analysis-side
   can't influence the organism — verified by config/harness
   audit (no labels in the schedule).

## 11. Success / partial / failure / inconclusive (qualitative)

SUCCESS: probe divergence at G0 grows from the early-window
baseline to a stable late-block level across development; the
developing component exceeds the trace baseline; a persistence
signature is characterized at G1; A-C separation and gates remain
clean. Claim: the organism develops organization that encodes the
antecedent (temporal state) beyond fixed traces.

PARTIAL: divergence present but flat (pure trace; no developmental
growth) — documents a short-lived trace without developmental
organization; or divergence grows but only at G0 with no G1
persistence; or divergence growth without a clean trace separation.

FAILURE: no divergence at any window under canonical mechanisms —
the tested capability does not emerge in this architecture; a
registered negative (valid result; no mechanism modification).

INCONCLUSIVE: gate failures (rates/failures/engagement), telemetry
or determinism faults, insufficient trial counts, early-window
baseline uninterpretable (e.g., S1 onset transients every window).

## 12. Relation to E1-E17

Depends on: E9 phase/trailing machinery and E12 blocked-context
findings (antecedent sensitivity exists); E14 per-presentation
measurement and k* rule (measurement basis); E15 bucket machinery
and snapshot instants (structural basis); E17's instant
re-expression (defines the strong trivial alternative the
early-window baseline must defeat); E10/E11 experience-scale and
order-balance findings (curriculum scales: >= 120 trials chosen to
be far past E10's 120-rep robustness point).

Provisional: the E6 phi->beta route and M6 residual inhibition
across gaps are hypothesized persistence channels but have never
been directly observed — E18 is mechanism-agnostic about them.

Does NOT rely on: E13 verdicts, E16/E17 ablation outcomes as
mechanism claims, v3/E6 overlap-curriculum entanglement results.

Genuine progression: E12-E17 changed the *order context* and
measured the resulting representation state; E18 gives the organism
a *developmental problem* (the contingency between an antecedent
and a probe across silence) and asks whether an observable
behavioral capability — encoding the antecedent in the probe
response — emerges and develops. The endpoint is a capability
(behavior + persistence + development), not a representation
property of the stimulus. A failed E18 changes the research
question meaningfully; repeating an E-series measurement would not.

## 13. Architectural sufficiency

E18 v1 requires NO new mechanism: the canonical organism, a new
schedule config (curriculum only), and analysis-side instruments
(per-trial-pair divergence — the per-presentation vector machinery
exists; a divergence summarizer is analysis plumbing, not an
organism change). If a sensorimotor capability is ever required,
the minimum architectural requirement is an output->environment
feedback channel — NOT designed or implemented here.

## 14. Unresolved design decisions requiring human approval

1. Probe pattern: B (committed plan — maximal E-series grounding,
   zero new patterns) vs a new straddling probe on {6,7,10,11}
   (cleaner φ contrast, but requires analyzer pattern registration
   and is a new stimulus definition). Recommend: B.
2. Gap values: G0 = 800 ms, G1 = 1600 ms (recommended) vs single
   gap G0 only (minimal v1; S3's persistence block becomes a
   follow-up).
3. S4 contingency-filter block included in v1 (recommended:
   yes, within-run, it reuses the proven re-anchoring framework
   and converts "adaptation" from a second experiment into a
   block) vs deferred.
4. Shuffled/silence-probe control run: default NOT included;
   triggered only if the early-window baseline is ambiguous at
   freeze analysis of pilot trials. Approve the trigger rule?
5. Endpoint thresholds (growth criterion, steady-state divergence,
   persistence ratio): not chosen here; decision items at the
   freeze stage.
6. Substrate: canonical E12 organism (recommended) — no ablation
   arm; mechanism hypotheses (φ/M6) stay untested in E18.

## 15. Next step

"freeze protocol" (after approval of Section 14 decisions) — or
"redesign" if the capability choice is rejected. No E19 proposal.