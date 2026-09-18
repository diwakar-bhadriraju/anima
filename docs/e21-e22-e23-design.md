# ANIMA E21/E22/E23 — Three orthogonal follow-up experiments
# (design + audit; NOT frozen, NOT implemented)

Status: DESIGN/AUDIT (2026-09-19). Basis: E19 `2ce7072`, E20
`439a7b1`, action-interface audit `f3bf699` (with its recorded
erratum), envelope measurements below. No implementation, no
freezing, no execution, no substrate proposals.

## 0. Repository findings (authority for these designs)

- Frozen substrate: e12 sections (JSON-isolation test method,
  E16/E18/E19/E20); E6 on; M1-M6 on; birth_trigger none; 24/40/12
  neurons; outputs 64-75 ordinary LIF (network.rs:331-345); v_rest
  0, no noise/tonic drive (network.rs:555) — no endogenous
  activity exists at any silent time (E19/E20 measured).
- OUTPUT RESPONSE ENVELOPE (measured on runs/e20-*, trial-aligned,
  per 100 ms from epoch onset; output neurons only):
  - antecedent A/C (20 Hz, 500 ms): 13967/11926/10835/9242/8405
    then 0 — SUSTAINED output activity through the whole stimulus,
    dying within 100 ms of offset.
  - probe B (40 Hz, 2-phase, 500 ms): 11441/11698/8328/0/0 — the
    output response is phase-1-dominated and ends ~300 ms in.
  - disruption (16-23 @ 80 Hz): 870/710/536/584/553 then 0.
  Consequence (design-relevant): output activity EXISTS whenever a
  stimulus is being driven; the only zero-activity readouts are
  silent ones. E20's D1 [1800,1950) read silence because the probe
  response was already over — probe-ANCHORED (not probe-offset-
  anchored) readouts have activity.
- E18 machinery: trial-block stages, balanced shuffles, probe
  byte-identity, divergence metric D (1 - pairwise-mean cos),
  E-window settling transient — divergence must be read in settled
  windows (L), not E.
- E19/E20 machinery: world layer (vote/consequence/disruption
  trains run-seeded), world log, schedule audit, gates — all
  reusable verbatim.

# EXPERIMENT A (E21) — TEMPORAL CAPACITY / GAP SWEEP

## A.1 Research question

What antecedent->probe separation can the frozen E6/E12 organism
still resolve (internal antecedent information measurable in the
probe response)?

## A.2 Design

- Paradigm: E18 verbatim — antecedent A|C (500 ms) -> GAP -> probe
  B (500 ms, all-SEQ) -> ITI; 2000 ms cadence (gap + ITI
  compensated: ITI = 1000 - GAP); 200 trials; balanced 20/20 per
  40-trial window; seed 20260912; S0 silence 5000.
- ARMS: six runs, gap in {0, 50, 100, 200, 400, 800} ms. ONLY the
  gap differs across arms (and exp_id). Gap 0 = probe starts at
  antecedent offset: the ZERO-RETENTION REFERENCE — if D(0) shows
  no divergence the paradigm itself is broken (protocol failure,
  not a capacity result).
- No output interface (no world): E21 is measurement-only. No
  growth, no mechanisms, no parameter changes.

## A.3 Measurement

- Internal divergence D_L(g) = 1 - pairwise-mean cos(probe-after-A
  vs probe-after-C) in the L window (trials 161-200; settled — the
  E transient is a known settling artifact). E/M/L reported.
- NOISE FLOOR (registered, per arm): NF(g) = split-half
  within-condition divergence = 1 - mean cos(B-after-A[first
  half of L] vs B-after-A[second half of L]) (same for C; mean of
  the two). This measures how divergent SAME-condition responses
  are by chance/drift — the principled floor for retention.
- A-C sanity per arm (< 0.60); gates per arm.

## A.4 Retention criterion + cliff definition (pre-registered)

- RETAINED(g) := D_L(g) - NF(g) > 0.05 (margin carried from the
  E-series 0.05 convention; now anchored to a measured floor).
- TEMPORAL CAPACITY (the cliff) := the largest g in {0, 50, 100,
  200, 400, 800} with RETAINED(g) true AND RETAINED(g') true for
  all g' < g in the set (monotone prefix; a non-monotone break is
  reported as an anomaly, not re-selected).
- If only RETAINED(0): capacity is at-offset-only (< 50 ms). If all
  RETAINED: capacity >= 800 ms (extension only by a new approved
  release). No interpolation between sampled gaps (resolution =
  the sampled grid, stated).

## A.5 Leakage/shortcut audit

Probe byte-identical after A/C (E18 machinery); timing template
constant per arm (only the registered gap differs); balance
windows; seeded determinism; no labels; A/C rate/duration
identical (both 20 Hz/500 ms — no frequency cue); gap-0 arm has
zero silence (the probe physically overlaps the antecedent's decay
— expected, it IS the reference). Shortcut: none available — no
behavioral readout exists in E21.

## A.6 Verdict logic (pre-registered)

- MEASURED: the cliff (per A.4) + the full D/NF table.
- PROTOCOL FAILURE: gap-0 reference shows no divergence; any
  leakage test fails; determinism/gates fail in an arm (that arm
  excluded, cliff computed on the remaining prefix only if
  contiguous; else inconclusive).
- E21 does NOT test behavior, consequences, or mechanisms.

# EXPERIMENT B (E22) — CLOSED-LOOP PRESSURE WITHIN CAPACITY

## B.1 Research question

Can environmental consequences shape behavior when the frozen
organism is operated inside its empirically measured temporal
capacity?

## B.2 Preregistered gap-selection rule (defined NOW; no tuning)

g_B := the largest g in {50, 100, 200, 400, 800} with RETAINED(g)
per E21. 0 is EXCLUDED (no-gap makes the antecedent physically
overlap the probe — the world would read a mixed stimulus, an
information leak by construction). If NO g >= 50 is RETAINED,
E22 is NOT EXECUTED and the registered outcome is: "temporal
capacity insufficient for a closed-loop retention task in the
frozen substrate" — a valid E21-derived result, not an E22
failure. g_B is chosen from E21's table alone, before any E22
run; never revisited after behavioral results.

## B.3 Design (E19/E20 verbatim except the registered dependencies)

- Trial: antecedent 500 -> gap g_B -> probe B 500 -> consequence
  500 -> ITI (cadence 3000 with gap compensation ITI = 1000 - g_B;
  epoch durations identical to E19/E20).
- WORLD: E19/E20 law verbatim — vote = majority of output groups
  64-69 vs 70-75; A-context correct = group 1, C-context = group
  2; match -> silence, mismatch/no-action -> disruption 16-23 @
  80 Hz x 500 ms; trains run-seeded in trial order; 200 trials;
  balance 20/20 per 40; seed 20260912.
- VOTE WINDOW (the one interface decision — D1 principle, probe-
  ANCHORED): [probe_onset, probe_onset + 150) — the first 150 ms
  of the probe, where the measured envelope shows the strongest
  output activity (11441+11698 in the first two 100 ms bins vs 0
  after 300 ms). Rationale: E20 proved probe-OFFSET anchoring
  reads post-response silence (the echo does not exist); reading
  the probe response itself is the only activity-bearing readout
  at the decision point, and it is exactly the signal whose
  antecedent-dependence E21 certifies at g_B. FLAGGED as decision
  1 below (it deviates from "D1 verbatim" by anchoring, not by
  the 150 ms length).
- Consequence epoch starts at probe end (decision point closed at
  probe_onset + 150; the remaining probe is registered as part of
  the sensory epoch — the vote cannot be influenced by the
  consequence, which is strictly later).

## B.4 Measurements (E/M/L = 1-40 / 81-120 / 161-200)

- Behavioral: BD(window) = |P(vote=g1|A) - P(vote=g1|C)| (no-
  action excluded, reported); BR; no-action rate; vote spike
  counts per group.
- Internal: D on probe epochs (E18 metric, L window primary); A-C
  sanity; bucket/weight organization at block boundaries (E15
  machinery); output-neuron rate maps (which group-neurons change
  responsiveness across windows).
- Gates: failures 0, max rate <= 250, permanence > 0; schedule
  audit; determinism.

## B.5 Criteria + verdict grid (pre-registered)

- Behavioral criterion: BD_L - BD_E > 0.30 (carried binomial-power
  margin). Fixed-action ceiling: 0.5 (balanced antecedents).
- Internal criterion: D_L - D_E > 0.05.
- VERDICT GRID: (A) both criteria met = consequence-driven
  behavioral adaptation with internal development; (B) behavioral
  only; (C) internal only; (D) neither; (E) apparent success with
  a detected shortcut/leak (protocol failure); (F) degenerate
  (no-action > 0.5 in L — now a genuine policy outcome, not
  structural silence, because the vote window overlaps driven
  activity; if F occurs with vote-window output activity ~0 in
  the telemetry, reclassify as interface failure).
- CONTROLS: open-loop control (identical schedule, consequence
  always silence) ONLY IF the canonical meets the behavioral
  criterion (E19 trigger rule — attributes development to
  consequences vs exposure). Shuffle audit per E18 rule.
- Contamination checks: consequence trains never enter the vote
  window (epoch order: probe/vote -> consequence); disruption
  channels 16-23 virgin to A/B/C machinery; no reward/teacher
  channel exists (world injects only channel spikes).

## B.6 Dependency registration

E22 MUST NOT be frozen/executed until E21's table exists and g_B
is derived by B.2 and recorded. E22's protocol freeze will cite
E21's commit and the derived g_B.

# EXPERIMENT C (E23) — REFLEX-SHAPING CONTROL

## C.1 Research question

Can environmental consequences shape any output behavior when NO
temporal retention is required — i.e., can consequences shape a
purely stimulus-driven response routing under the frozen
architecture?

## C.2 Design (smallest closed-loop reflex)

- Stimulus: the committed antecedent A {0-7} or C {8-15}, 20 Hz,
  500 ms — the stimulus IS the cue; balanced 20/20 per 40-trial
  window; seed 20260912; 200 trials.
- VOTE WINDOW: [100, 500) of the stimulus epoch (first 100 ms
  excluded — the response ramp; envelope shows sustained output
  from the first 100 ms bin onward: 13967..8405). Majority vote,
  groups 64-69 vs 70-75, same world law: A -> group 1 correct,
  C -> group 2; match -> silence, else -> disruption (identical
  parameters).
- Consequence: [500, 1000) after stimulus offset. ITI to 2000 ms
  cadence (1000 ms ITI). No probe, no gap, no hidden state, no
  retention demanded. Nothing to remember: the appropriate action
  is fully determined by the CURRENT stimulus.
- NOT supervised: the organism receives only the stimulus and its
  consequences; no error signal, no target, no label enters the
  organism. The A->g1/C->g2 mapping exists only in the world.

## C.3 Control (scientifically necessary — co-equal arm, not
## conditional)

OPEN-LOOP arm: identical stimulus schedule, consequence always
silence. Rationale: outputs ALREADY differentiate A vs C under
passive exposure (the envelope differs by stimulus); without the
open-loop comparison, any BD could be pure exposure effect. Two
arms total (closed, open), same seed, 200 trials each.

## C.4 Measurements + criteria (pre-registered)

- BD(window) per arm; the EFFECT of consequences := BD_closed(L) -
  BD_open(L) with criterion > 0.30 (carried margin). Internal:
  antecedent-response divergence D per arm (L), bucket
  organization, output-neuron rate maps. Gates/schedule audit as
  B.4.
- Verdict grid: (A) consequence effect positive with internal
  divergence gain = reflex shaping with representational change;
  (B) behavioral effect only; (C) internal only; (D) no effect
  (consequences do not shape even reflexes — strong negative);
  (E) shortcut/leak (protocol failure); (F) degenerate.
- Shortcut audit: the vote reads the stimulus response — the
  registered, honest property (this experiment deliberately tests
  reflex shaping); fixed-action ceiling 0.5; alternation fails
  (balanced random); quiescence detected (no-action > 0.5 in L —
  again now a genuine outcome since activity exists in-window);
  no timing cue (template constant); no schedule structure
  (balanced shuffles); determinism.

## C.5 What C is NOT

Not a memory experiment (nothing to retain), not supervised
classification (no labels/reward — only consequences), not a
substrate test. It isolates the CONSEQUENCE-SHAPING primitive
itself.

# CROSS-EXPERIMENT ISOLATION

| | E21 | E22 | E23 |
|---|---|---|---|
| question | temporal capacity of the substrate | consequences shape behavior within capacity | consequences shape reflexes at all |
| frozen | e12 substrate; E18 paradigm; no world | e12 substrate; E19/E20 world+law; g from E21 | e12 substrate; E19/E20 world+law; no gap/probe |
| changed | GAP only (6 arms) | gap = g_B (rule B.2); vote probe-anchored | stimulus-epoch vote; no probe/gap |
| establishes | measured retention cliff | whether pressure works when reachable | whether pressure works at all (reflex) |
| cannot establish | anything behavioral | substrate capacity (E21's job); reflex shaping without retention (E23's job) | anything about memory/retention |
| contaminations guarded | no world/no vote | g_B fixed pre-run from E21 only | no retention demand; open-loop co-arm |

E21 cannot become a behavior experiment (no interface exists);
E22 cannot become a substrate experiment (organism frozen; g_B
rule-locked); E23 cannot become a memory experiment (no gap).

# REPRODUCIBILITY (all three)

Same-seed byte-identical telemetry/snapshots/world logs; config
hashes recorded pre-run; isolation tests (config JSON equality vs
e12/e18/e19 as applicable); env-grid tests per experiment; world
unit tests (vote gating, decision mapping, train determinism);
determinism integration; delivery-tick +1 convention retained;
analysis instruments telemetry-only (no anima_core), byte-
deterministic on rerun. Artifacts: run dirs + protocol records +
registry entries; separate commits per experiment.

# CONDITIONAL DEPENDENCIES

- E22 depends on E21's table (rule B.2; freeze cites the commit).
- E22's open-loop control is conditional on behavioral success.
- E21 needs no other experiment; E23 needs none (can run in
  parallel with both).

# UNRESOLVED DECISIONS REQUIRING APPROVAL

1. E22 vote-window ANCHORING: probe-anchored [onset, onset+150)
   (recommended — measured envelope; E20 proved offset-anchoring
   reads silence) vs literal-D1 offset-anchored (guaranteed
   structural zero). The 150 ms length is unchanged.
2. E21 arm set {0,50,100,200,400,800} and 200 trials/arm
   (6 runs); alternative: drop 400 (5 runs) — resolution loss.
3. E21 noise floor via split-half (recommended) vs carried 0.05
   absolute only (weaker).
4. E23 open-loop arm as co-equal (2 runs) vs conditional-on-
   success (cheaper; weaker attribution) — recommend co-equal.
5. E23 vote window [100,500) of stimulus (recommended) vs full
   [0,500).
6. Seeds: 20260912 primary for all three (cross-seed only by
   later approved release, per house convention).

No implementation, freezing, or execution; no fourth experiment;
no substrate change proposed (any future substrate question is
Category C and requires its own approval path).