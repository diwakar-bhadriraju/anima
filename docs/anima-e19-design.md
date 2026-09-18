# ANIMA E19 — Closed-loop developmental pressure test
# (design + audit; NOT a frozen protocol)

Status: RESEARCH DESIGN / AUDIT (2026-09-19). Nothing implemented,
frozen, or executed. Authority: repository state at `0ee81f4`
(E18 closed). E1–E17 substrate; E18 negative (no persistent temporal
state from passive exposure).

## 0. Repository audit (verified facts this design rests on)

- Organism I/O: 24 input channels (ids 0–23), 40 internal (24–63),
  12 output (64–75). Output neurons are ordinary LIF internal-class
  cells whose spikes are ADDITIONALLY emitted as `OutputActivity`
  (kind 4) — network.rs:48–49, harness.rs:503. The organism's action
  surface already exists and is already logged; no organism change
  is needed to observe it.
- Env/harness split: `Environment::step() -> (InputFrame, marker)`
  per tick; `Network::step(frame) -> {spikes, output_spikes}`. The
  harness currently feeds env→net with NO coupling back. Closing the
  loop (output_spikes → world → future InputFrame) is a HARNESS/
  ENVIRONMENT-layer change. The organism, its config, and all
  plasticity/structural mechanisms remain byte-frozen.
- Frozen substrate (verified from configs/e12.toml + E16/E18
  isolation tests): 24/40/12; organism/plasticity/structural/
  resources/v2/e6 sections; E6 on (alpha 0.04); M1–M6 on; STDP on;
  adaptation on; birth_trigger "none" (no growth — per the E19
  constraint); deterministic Xoshiro scheduling; snapshots every
  1000 ticks; delivery-tick convention (telemetry = scheduled + 1).
- E18 machinery reusable verbatim: trial-block stages (balanced
  seeded antecedent shuffles), byte-identical probe guarantee,
  marker completeness (contiguous-boundary fix), E/M/L windows,
  determinism tests, no-label schedule representation.
- Known organism constraint entering E19 (E12–E18 evidence): no
  identified persistence carrier beyond ~200 ms traces (adaptation)
  and E6 φ (~2.5 s, plasticity-gain-only); recurrence collapses
  without LTP. E19's honest prior is therefore a likely negative —
  which is the informative outcome it is designed to produce.

## 1. Research question

Can closed-loop consequences alone — with the immediately available
sensory input held ambiguous — create pressure for the frozen
E6/E12 nervous system to develop or use a temporally relevant
internal distinction (antecedent-conditioned action) that passive
exposure (E18) did not produce?

## 2. Hypothesis (mechanism-agnostic)

If the organism's actions change its sensory future in a way that
depends on a hidden antecedent (available only before an ambiguous
probe and a silent gap), then interaction with this consequence-
bearing world will develop antecedent-conditioned action (and/or
antecedent-conditioned internal probe organization) beyond the
early-window baseline — without labels, reward, teacher, or any
architectural change. Null: no developmental growth of either the
behavioral or internal antecedent-conditioning; or apparent success
explainable by a shortcut/leak (Section 6–7). No mechanism (memory,
recurrence, STDP, traces) is assumed; those are post-hoc
investigations.

## 3. Frozen substrate (exact; verified)

configs/e12.toml sections run/organism/plasticity/structural/
resources/v2/e6 — byte-identical (the E18 isolation-test method:
JSON equality of every section). E6 exactly as-is. No new neuron
model, plasticity, memory, consolidation, growth, or parameter
change. The ONLY conceptual change: harness closes the loop
(outputs affect subsequent input). Changed variables table in
Section 17.

## 4. World definition (minimal)

- HIDDEN STATE: one bit per trial s ∈ {A-context, C-context}, set
  by the antecedent epoch, invisible except through consequences.
  No cross-trial world state; reset = next trial's antecedent.
- SENSORY OBSERVATIONS: the committed patterns A {0–7} (20 Hz) and
  C {8–15} (20 Hz), 500 ms; the committed probe B {4–7→8–11} phases
  (40 Hz, 500 ms, all-SEQ by variant_block 100000); consequence
  epochs: benign = silence; disruptive = channels 16–23 @ 80 Hz for
  500 ms (virgin channels — never pattern-active in E12/E18 — so
  consequence input cannot confound the A/B/C representation
  machinery; magnitude 4×80×0.5 = 320 events, ~2× the probe's
  drive, homeostatically salient but inside the [20,250] Hz gate
  band by construction).
- ACTIONS: binary, read from the organism's own output spikes
  during a fixed 500 ms action window (silence): action = majority
  of output-group-1 (neurons 64–69) vs output-group-2 (70–75);
  no winner (tie or total quiescence) = no-action. The readout
  convention is part of the WORLD (I/O boundary), not a prescribed
  internal organization — which neurons actually differentiate, and
  how, is left entirely to the organism.
- ACTION CONSEQUENCES: delivered in the fixed 500 ms consequence
  epoch: action matches the hidden state (A-context wants group-1,
  C-context wants group-2) → silence; mismatch or no-action →
  disruption. The mapping (which group is "correct" under which
  antecedent) is fixed and arbitrary-registered; it is NOT signaled
  to the organism in any way other than through consequences.
- EPISODE STRUCTURE (one trial, 3000 ms): antecedent 500 → gap 800
  (silence) → probe B 500 → action window 500 (silence) →
  consequence 500 (silence or disruption) → ITI 200. Trial template
  identical for both trial types and all trials.
- GAPS/PROBE: gap 800 ms (E18 continuity — the tested timescale);
  probe = the E18 probe, byte-identical after A and C.
- RESET: none beyond the next antecedent (world is memoryless
  across trials).
- ENVIRONMENT STATE: {hidden bit, phase-in-trial, pending vote
  counts} — all internal to the world; never sensory.

## 5. I/O boundary

- ENTERS ANIMA: channel spikes (A/C antecedents, B probe,
  disruption bursts), silence otherwise. Nothing else — no reward,
  no error, no teacher, no label, no "correct/wrong" signal, no
  world-state channel.
- LEAVES ANIMA: the per-tick spike set of the 12 output neurons
  (already emitted as OutputActivity).
- ENVIRONMENT KNOWS: trial schedule (seeded), hidden antecedent,
  action-window vote, consequence due.
- ANIMA DOES NOT RECEIVE: the hidden antecedent (except as the
  antecedent's own sensory epoch before the gap), the correctness
  rule, the action readout, trial boundaries (identical template),
  or any post-hoc analysis quantity.

## 6. Information-leakage audit (design-level; to be test-verified
## at implementation)

1. Probe identity: B_after_A == B_after_C byte-identical — same
   committed pattern, same variant selection, antecedent has no
   influence on the train tuple (E18-verified machinery; test
   re-registered).
2. Timing: one trial template; both trial types identical in every
   interval; only the CONTENT of the consequence epoch varies, and
   only as a function of action × hidden state — the legitimate
   feedback channel (post-action by construction).
3. Spike statistics: antecedent epochs differ by design (they ARE
   the information); probe and pre-action epochs are statistically
   identical across trial types; disruption content is fixed
   (channels/rate constant; only presence varies).
4. No channel identifies the antecedent at probe/action time:
   channels 0–15 silent post-antecedent; channels 16–23 carry only
   consequences (post-action).
5. No episode-position artifact: template constant; consequence
   epoch exists in every trial (content may be silence).
6. No deterministic-schedule memorization: antecedents = seeded
   balanced shuffles (20/20 per 40-trial window; E18 machinery).
7. No action-history leak: consequences of trial k are separated
   from trial k+1's antecedent by the ITI; antecedent draws are
   independent of history.
8. Environment state not otherwise exposed: the world's only
   outputs are silence/spikes on registered channels.
9. Reset timing: no reset event exists (template continuity).
10. No reward/teacher channel: verified — the harness adds no new
    event type to the organism; consequences are ordinary input
    frames; OutputActivity was already logged.

Formal task-necessity argument (why antecedent information is
required): at action time the input history since trial start is
identical across trial types except for the antecedent epoch;
correct(action, s) depends on s; antecedents are balanced; hence
any policy with success strictly above the fixed-action maximum
(0.5) must retain antecedent information from before the gap to
the action window. The task genuinely requires the distinction.

## 7. Shortcut audit (each with detection)

| shortcut | how it would work | detection |
|---|---|---|
| always one action | fixed vote every trial | success pinned at ~0.5; behavioral divergence BD ≈ 0 |
| alternating actions | vote flips by trial parity | same (antecedents random) |
| quiescence | no vote → no-action | no-action rate high; success ~0 (no-action → disruption by design) |
| episode timing | none available (template constant) | structural — cannot occur |
| presentation order | memorize seeded order | order audit (lag-1 autocorr, balance windows — E18 trigger rule) |
| action-independent sensory differences | none at probe/action time | leakage tests 1–4 |
| deterministic RNG | organism cannot read RNG | none reachable |
| reset artifacts | no reset event | structural |

## 8. Curriculum (smallest pressure-bearing)

- Trials: 200 (E18 scale; power below), seed 20260912 primary.
- A/C balance: exact 20/20 per 40-trial window (E18 machinery),
  100/100 overall; seeded shuffles, no other structure.
- Windows: E = trials 1–40, M = 81–120, L = 161–200 (40-trial
  behavioral windows for binomial power; internal-divergence
  windows computed on the same spans).
- Timing: the Section-4 template (3000 ms/trial; total 605,000 ms).
- Durations justification: gap 800 ms = E18's tested timescale
  (continuity of the question); antecedent/probe 500 ms = committed
  pattern durations; action window 500 ms = readout power (12
  outputs at ~10 Hz ⇒ ~30 spikes/side; design choice, flagged);
  consequence 500 ms = one pattern-epoch duration (design choice);
  disruption 80 Hz × 4 virgin channels = ~2× probe drive (design
  choice, registered).
- No labels, no reward, no curriculum shaping, no adaptation of the
  world to the organism. The world is stationary.

## 9. Developmental measure (E→M→L, E18-style)

Per window (E/M/L) and per antecedent:
- BEHAVIORAL: P(vote=group-1 | A) vs P(vote=group-1 | C) →
  BD(window) = |difference|; benign-rate BR (fraction correct);
  no-action rate; output spike rates.
- INTERNAL: probe-epoch internal response vectors after A vs after
  C → divergence D (E18's 1 − pairwise-mean cosine; raw == L1);
  A-B/C-B alignment means; antecedent-epoch A-C separation (sanity).
- ORGANIZATION: E15-style bucket totals by side at block boundaries
  (snapshots); per-neuron selectivity; permanence events.
- STABILITY: window-to-window BD/D trajectories; BR trend.
- Resource: failures, snapshot rate max, permanence (existing
  gates).

## 10. Causal interpretation grid

A. Behavioral adaptation + internal divergence → consequences
   produced antecedent-conditioned organization (strongest).
B. Behavioral adaptation without measurable internal divergence →
   action differentiation without probe-representation change
   (output-level solution; still antecedent-conditioned behavior).
C. Internal divergence without behavioral adaptation →
   representation without use.
D. Neither → consequences at this scale/timescale do not create
   the pressure (or the carrier is absent) — clean negative
   isolating the architectural boundary.
E. Apparent success via shortcut/leak → protocol failure (Section
   6/7 detections).
F. Degenerate/quiescent policy → pressure insufficient or
   readout mismatch; register and interpret without tuning.
No result is called "intelligence".

## 11. Falsification criteria (pre-registered now)

- PRIMARY behavioral criterion: BD_L − BD_E > 0.30. Justification:
  binomial SE of BD at 20/20 trials ≈ 0.158; 0.30 ≈ 2 SE — an a
  priori power calculation, not data-derived. (If you prefer
  deferring, see Section 21.)
- Behavioral support additionally requires: BR_L > 0.5 + no-action
  rate low in L, and BD_E ≈ chance (early baseline ~0).
- Internal divergence: D reported with the E18 margin (0.05) as
  the descriptive secondary.
- SUPPORT: A-grid (primary criterion met). PARTIAL: B or C grids
  (one axis only). FAILURE: D-grid (BD_L − BD_E ≤ 0.30 and
  D flat ≤ 0.05). INCONCLUSIVE: gate failures (rates > 250 Hz,
  failures, zero engagement), leakage/shortcut detection firing,
  determinism faults, or E-window contamination.
- PROTOCOL FAILURE: any Section 6 test fails at implementation.

## 12. Controls (minimal, conditional)

- OPEN-LOOP control (pressure attribution): identical schedule and
  action windows, but the consequence epoch is always silence
  (world ignores actions). TRIGGER: run ONLY IF canonical meets the
  primary criterion (BD_L − BD_E > 0.30) — to attribute the
  development to consequences rather than exposure. If canonical
  fails, the control is uninformative and is not run.
- ANTICEDENT-SHUFFLE audit: reuse E18's trigger rule (|lag-1
  autocorr| > 0.3 or window imbalance) — analysis-time check on
  the drawn sequence, control run only if it fires.
- No other controls. No factorial arms.

## 13. Reproducibility

Seeds: 20260912 primary (cross-seed only by later approved
release). Deterministic world: consequence trains pre-generated
per trial from the run-seeded env RNG in registered order; action
votes accumulate deterministically from logged output spikes.
Byte-identical on same-seed rerun (telemetry + snapshots + world
log). Config hash recorded pre-run; isolation tests (E18 method)
prove substrate equality vs e12. Delivery-tick +1 convention
retained.

## 14. Observability (world log — analysis-side only)

Per trial: antecedent identity, scheduled times, per-group output
spike counts in the action window, vote/no-action, consequence
delivered (silence/disruption), cumulative BR. From existing
telemetry: OutputActivity (kind 4), StimulusPresented markers,
spikes, snapshots. Nothing new enters the organism.

## 15. Scale

200 trials × 3000 ms ≈ 10 min/run at E18 pace — the same scale
that produced E18's clean negative, with binomial-powered windows.
No large-scale training: the first pressure test must be
interpretable, not optimized.

## 16. Failure modes (considered)

Reflex shortcut (fixed probe→vote wiring; BD ≈ 0 — detected);
quiescence (detected; punished by design); saturation/runaway
(gates; disruption magnitude chosen inside the band); output
collapse (no-action rate metric); world oscillation (none —
stationary world); schedule exploitation (audit); hidden-state
leakage (Section 6 tests); insufficient pressure (D-grid outcome —
valid); pressure too strong (rates/failures — gate); task
impossible-by-construction (excluded by the Section 6 formal
argument: the information exists pre-gap; only retention is
required).

## 17. Experimental isolation table

| variable | frozen from E18 | changed in E19 | reason |
|---|---|---|---|
| organism, all mechanisms, params | ✓ | — | isolation constraint |
| patterns A/B/C, gap 800, probe byte-identity | ✓ | — | E18 continuity |
| trial count 200, balance machinery, seeds | ✓ | — | comparability |
| trial template | 2000/2600 ms | 3000 ms | action + consequence epochs added |
| sensory stream | ante/probe/silence | + consequence epoch (silence or disruption on 16–23) | the ONE conceptual change |
| output path | observed only | observed AND read by the world (vote → consequence) | closed loop |
| world | none (open) | hidden bit + consequence rule | the ONE conceptual change |

Exactly one conceptual change: closed-loop consequences.

## 18. Decision gate (before implementation)

The protocol is valid and implementation may begin only if:
1. Every Section 6 leakage test is implementable as a passing
   automated test (probe byte-identity, template constancy, channel
   silence windows, balance, determinism).
2. Every Section 7 shortcut has a concrete detection metric.
3. The formal task-necessity argument holds (it does — Section 6).
4. The world log reconstructs every trial (Section 14).
5. User approves the unresolved items in Section 21.

## 19. Research interpretation

E18 asked: does passive exposure produce persistent temporal
state? (No.) E19 asks: does consequence-bearing interaction create
pressure for useful temporal organization? If E19 succeeds where
E18 failed, the missing ingredient was pressure, not architecture.
If E19 fails the same way E18 did, the evidence converges on the
architecture boundary (no persistence carrier) — the minimal
justification for a future, separately-approved substrate
candidate (out of scope here). E19 CANNOT establish: mechanism
identity (which process carries the distinction), sufficiency of
consequences in general, universality beyond this world/scale, or
any claim about growth/recurrence value (excluded by design).

## 20. Next-experiment boundary

No E20 proposal. The only conditional follow-up WITHIN E19 is the
triggered open-loop control (Section 12). New memory mechanisms,
growth, intrinsic plasticity, reward, RL, attention, or
neuromodulation are separate hypotheses and are excluded from E19.

## 21. Unresolved questions requiring user approval

1. ACTION READOUT: majority vote of fixed output groups (64–69 vs
   70–75) over a 500 ms silent window; no-action on tie/quiescence
   → disruption. (The readout is world-side; alternative:
   first-spike latency — rejected: fragile at ~10 Hz.)
2. CONSEQUENCE PARAMETERS: benign = silence; disruption = channels
   16–23 @ 80 Hz, 500 ms; no-action punished. (Design choices,
   registered; not tuned post-hoc.)
3. GAP: 800 ms (E18 continuity) — accepting the honest prior of
   failure if no carrier exists. Alternative 400 ms would raise
   success odds via usable traces but changes the tested
   timescale; recommend 800.
4. PRIMARY THRESHOLD: BD_L − BD_E > 0.30 (binomial-power
   justification above). Alternative: defer thresholds to a
   pre-execution registration step once E-window variance is
   observable — NOT recommended (post-hoc risk).
5. OPEN-LOOP CONTROL trigger (Section 12) — approve.
6. TRIAL TEMPLATE 3000 ms with silent action window — approve
   (silence during voting prevents the world from cueing the
   action).

STOP — awaiting approval. No implementation, no execution.