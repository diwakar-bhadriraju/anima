# ANIMA autonomy log

Status: maintained under the autonomous research charter (2026-09-22).
Structure per decision: observation → hypothesis → evidence → decision →
rejected alternatives → experiment → result → interpretation → open q.
Distinction kept: MEASURED vs INTERPRETATION vs HYPOTHESIS vs DECISION.

## D-2026-09-22-01 — First-contact claim (candidate E) as the next formal step

- OBSERVATION (measured): Phase II-A S1 FAIL 6/6. M1 audit split the
  failure into (a) preserved-but-MISADDRESSED cells (s20260912 ×2,
  s9001-bac: surviving second-cohort working mass 0.289–0.543, 100%
  tag-0, first-C track-1 current exactly 0 — the recruitment gain muted
  by the wrong tag) and (b) UNPRESERVED cells (s424242 ×2: surviving
  mass 0.000 — churn killed the substrate before exposure).
- HYPOTHESIS (from the audit counterfactual): local re-keying at first
  exposure lifts first-exposure drive to A-parity (~9.5/neuron) and
  unmutes the capped gain, crossing the bootstrap in the preserved
  cells — PROVIDED the surviving mass is normalized under the per-track
  regime (capacity-matched T_1), not the Phase-I single budget.
- DECISION: implement candidate E behind `d_core`: M1 synapses start
  UNCLAIMED (tag 2); first-contact claim rule; churn-exempt dormancy
  (floor θ_prune, M4-exempt) for unclaimed; two-regime M2
  (per-track targets for claimed work, budget-capped floor for
  unclaimed); all existing D-core mechanisms retained; a pre-registered
  S3 re-expression probe added (amendment). Freeze as
  docs/x-phase2-ar-protocol.md.
- REJECTED AT THIS STEP: D-only (no floor — leaves s424242 dead);
  C (duplication reintroduces superposition); A (static — falsified);
  a "defer per-track while unclaimed" M2 (would kill the predicted
  upscale); separate II-B registration (in-run S3 probe is the
  efficient path under the charter).
- EXPERIMENT: frozen x-phase2-ar. 9 new runs (3 orders × 3 seeds) +
  identity; endpoints S1 (6/6), S-a..g, S3 ρ(A)/ρ(C).
- RESULT: PENDING.
- OPEN: does E cross S1; does ρ>0 under either blocked order or
  alternation; do gain-regime aborts persist.
## D-2026-09-22-02 — E-nogain milestone + generalization; blocked-ρ gap prioritized

- OBSERVATION: E (with rg8c gain) forms but aborts 4 cells; E-nogain
  9/9 complete, S1 (blocked) 6/6, alternating re-expression ρ 0.25-0.77.
- OBSERVATION: clean-PN generalization (novel 16-23 group @ A statistics)
  forms S1 6/6 and alternates ρ 0.36-0.91 — NOT A/C-overfit. The earlier
  "AB" probe was confounded (config "B" = 40 Hz phase-variant over 4-11,
  overlapping A's channels); discarded, not evidence.
- H3 (probe re-learning) RULED OUT: blocked ρ flat across 5 re-exposure
  windows.
- DECISION: blocked-order re-expression is the highest-evidence gap
  (shared-pool recency signature: C/Pn re-exposure lights up both
  assemblies). Test H2 (recurrent-claim capture) next via an
  afferent-only-claim gated variant; then H1 if unchanged.

## D-2026-09-22-03 — blocked-ρ = expression-layer recency; checkpoint D-04

- OBSERVATION: per-track totals BALANCED at drive end in blocked runs
  (T0/T1 ≈ 8.97/9.22 for s20260912-bac E-nogain); second-block AFF mass
  ~2x first; blocked ρ contamination = both assemblies respond to one
  pattern (C re-exposure cosA~0.9 AND cosC~0.95) — shared
  membrane/recurrent pool recency (G3 superposition localized to the
  EXPRESSION layer). Budgets+addressing fix persistence/formation; the
  read is still pool-mixed when one memory dominates.
- DECISION: this exponential-family milestone (formation S1 6/6 × 2
  channel pairs; alternation ρ 0.25-0.91 general) is consolidated as a
  checkpoint; the next loop (D-04) targets blocked-order re-expression
  via an expression-layer intervention (per-track recurrent protection
  or a read gate), then capacity (K≥3), then closed-loop.

## D-2026-09-22-04 — sparse-commit fork (Phase III-A) frozen

- OBSERVATION (measured, respoverlap): blocked-order re-exposure
  responder sets are the ENTIRE pool, Jaccard 0.98-1.00, zero
  recurrent-only neurons -> dense p_in=0.5 pool has no structural
  separation; A/C expressions differ only by rate modulation. The
  alternation ρ win is rate-modulation too, not membership.
- HYPOTHESIS: selective re-expression under recency REQUIRES disjoint
  (sparse-committed) assemblies; budgets+addressing fix persistence/
  formation but not expression membership.
- DECISION: fork to sparse-commit (selective afferent dropout under a
  local commitment threshold R_t >= theta_commit=0.5), a redesign not a
  patch; the diagnostic uniquely demands it. Freeze
  docs/phase3/sparse-commit-protocol.md; implement next.
- REJECTED: further E-nogain patching for blocked-ρ (patch-spiral
  guard); moving up the ladder before closing this Level-3 gap (a
  shaky retrieval base).
- OPEN: does sparse-commit give Jaccard < 0.5 and blocked ρ >= 0.10
  without starving the second block (over-commit risk #1)?

## D-2026-09-22-05 — sparse-commit closed: measured blocked-order tradeoff; alternation base is the platform

- OBSERVATION (sparse matrix, 9 runs): responder separation PERFECT
  (Jaccard ~0.000, rho_A 0.85-0.94, cosC=0.000) but S1 gating FAIL 9/9
  (second block 0.0-3.4 vs first 9-15) - over-commit/first-past-the-post
  dropout destroyed the later memory's capacity. Alternation also broke
  (responders collapsed).
- INTERPRETATION (measured tradeoff): single shared pool + local first-
  contact commitment makes blocked-order ACCESS and SEPARATION
  structurally incompatible (dense: access no separation; dropout:
  separation no access). Alternation bypasses the asymmetry (balanced
  simultaneous commitment).
- DECISION: close sparse-commit as configured (S1 gate), no rescue
  chain (patch-spiral guard), no reparameterization. E-nogain
  alternation (formation + rho 0.25-0.91, general) is the demonstrated
  platform. Next: probe (b) sparse-wiring topology OR advance to Level
  4 (temporal) on the alternation base; pending the milestone record.

## D-07 — Level-4 temporal branch selected + feasibility probe POSITIVE

- OBSERVATION (gapstate on E-nogain): cross-gap internal activity is
  substantial (~300-1200 spikes per 1500ms gap, ramping to plateau) and
  PREDECESSOR-DISTINCT in the late gap for 2/3 seeds (cross A-C cosine
  0.625/0.759 vs within ~0.99) - a real endogenous temporal bridge that
  the V2.1-era G6 decay regime lacked.
- DECISION: selected Level-4 temporal/prediction over the sparse-wiring
  fork (which only refines the well-understood blocked-order gap; the
  mission ladder and organism objective demand temporal->prediction->
  action). Recorded docs/phase3/level4-decision.md + frozen minimal
  design (paired-associate prediction, eligibility trace, prediction
  index PI).
- OPEN: can the gap state be SHAPED into anticipation (the transition-
  learning mechanism) - falsifier PI ~ 0 after training.

## D-09 — Level-4 eligibility-trace FALSIFIED (clean rejection)

- d_elig (slow-trace LTP) implemented, identity gate PASS (flag-off
  byte-identical: FNV 9647ea8a0ca4dbd2, 105/105 snapshots), 183 tests
  green. 3-seed alternating run completed without runaway.
- FALSIFIER hit: PI = 0 exactly in all 3 seeds. Cause (measured): the
  eligibility LTP eliminated the cross-gap substrate (0 late-gap spikes
  in 37/34/33 gaps; presentations still fire, refmag ~1800-2080). The
  mechanism meant to USE the bridge destroyed it - reproduces the
  E4-family theme (plasticity reweighting destabilizes the firing
  regime it depends on in this all-excitatory E-nogain substrate).
- Verdict docs/phase3/level4-verdict.md. NO tuning (protocol 8).
- NOTE: clla-elig runs land in runs/clla-arex-* dirs (exp_id inherited);
  dir mapping in the verdict. d_elig configs clla-elig-s{seed}-il.
- Next (NEW registration, not patch): passive prediction READOUT that
  does not feed back into LTP - leave the base E-nogain gap state
  untouched, read a per-neuron trace as anticipation. Different locus.

## D-10 — d_elig_ro (partitioned B) FALSIFIED; Level-4-via-LTP CLOSED

- Wiring bug found: first d_elig_ro build never consumed the flag in
  plasticity.rs (patch AssertionError aborted it; runs were base-
  identical). Fixed both LTP sites; identity re-passed; rerun.
- With the mechanism genuinely active: pool PI ~0 (-0.033/-0.010/
  +0.002), output PI = 0 with output late-gap firing ZEROED (0 in all 3
  seeds vs base 162.9/gap). Pool bridge weakened, not preserved.
- Two-mechanism theme: all-edges d_elig AND readout-only d_elig_ro both
  fail - LTP consolidation collapses the persistent cross-gap state it
  would bridge. Ranked cause: no inhibition -> consolidation unopposed.
- Level-4 temporal prediction via spike-timing LTP: CLOSED (2
  falsifications, no tuning). Next families (await approval): inhibitory
  gating, structural growth, overlapping-stimulus curriculum.

## D-11 - d_ing (branch 1, inhibitory gating) FALSIFIED on association; protective premise CONFIRMED

- Implemented (identity PASS, FNV unchanged; 3 unit tests; 110 core /
  183 total green). 3-seed il runs, no runaway.
- CONFIRMED: pool gap substrate PRESERVED under inhibition (334.8/187.3/
  141.4 pool spikes/gap; output 23-288/gap) - unlike d_elig which ZEROED
  it. The per-cohort gate closes on drive / opens in tail as designed.
- FALSIFIED: output PI ~ 0 across seeds (-0.1204, +0.0139, +0.0085 vs
  base +0.0153/-0.0176/-0.0088); 0/3 reach the +0.05 PRIMARY bar.
  The gated readout slow-LTP did not turn preserved predecessor state
  into next-event anticipation.
- Ladder status Level-4 temporal prediction: 3 independent mechanisms
  falsified (d_elig, d_elig_ro, d_ing). The gap bridge is now PROTECTABLE
  (a real mechanism asset) but NO route converts it to anticipation.
  Next candidates recorded (overlap curriculum, readout coerced post,
  structural-growth-targeted) - approval required, no tuning.

## D-15 - adjacent-gap probe: Level-4 temporal prediction CLOSED (representational + dynamic)

- Shrank S1 gap to 40/100 ms (STDP reach) on base E-nogain, 3 seeds x2
  gaps. Out PI 0/3 reach +0.05 at either gap; post-X state is decaying-X
  memory, never Y-tuned even adjacent. 3/6 runs RUNAWAY (fast-STDP
  bridging over-amplifies the all-excitatory pool).
- Verdict: NOT a bridging-mechanism limit - REPRESENTATIONAL + DYNAMIC.
  Concordant with E3b (single shared pool mixes readouts) and the whole
  E4/d_elig/d_ing family (plasticity consolidation unopposed -> runaway).
  Level-4 temporal prediction via STDP transition learning CLOSED across
  gap scale and mechanism family. Confirmed assets: alternation
  retrieval (L3), protectable gap bridge (d_ing).
- Next (approval): (r1) substrate representation change, (r2) accept L3
  bound + move ladder to another rung, (r3) inhibition-with-gate-risk.

## D-18 - r2 Slice-1 (closed-loop action) FALSIFIED as-registered, with map-conditioning confound

- Slice-0 gate (base telemetry): output stimulus-selectivity cross-cos
  0.728/0.467/0.805 - action channel EXISTS but weak.
- Slice-1 (closedloop.rs): iterated the trained output-response map
  through the frozen tag split. Attractors: s9001 = 2-cycle A<->C (self-
  sustaining alternation - genuine single-seed agency); s20260912 &
  s424242 = fixed {A} (collapse).
- D-17 criterion (attractor bias matches C-dominance, >=2/3): NOT met
  (2/3 mismatch) => falsified as-registered.
- CONFOUND: the 2 collapses are a map-conditioning artifact (tag split
  output-half 64-69 is unconditionally more active for BOTH stimuli), so
  the falsification does NOT show organism failure - it shows the frozen
  contiguous-tag map is uninformative on 2/3 seeds; s9001 shows the loop
  CAN self-sustain when the split separates the responses.
- Preserved: single-seed positive; next = a NEW registration (contrastive
  / well-conditioned output readout before the loop), needs approval.

## D-19 - r2 hardened measurement CORRECTS D-18 (two claims were wrong)

(a) Action surface STRONG not weak: S3 per-trial decode = 100% (10/10)
all 3 seeds (pooled cosine understated per-trial separability). Level-5
action ENCODING confirmed. Slice-0 "weak" retracted.
(b) s9001 2-cycle FRAGILE (from-C per-presentation 9/20, margin 2.0) -
pooled 423.5/406.5 was averaging ~50/50; "genuine agency" retracted.
(c) collapse-to-A is organism structure (s20260912 robust across all tag
splits; from-A/from-C C=0/20), not purely map artifact.
Outcome: action encoding PASSES; closed-loop self-sustenance FALSIFIED
per D-17; blocker is the loop readout (continuous-tag), not the action
surface -> a contrastive/well-conditioned tag is the well-motivated next
registration (approval required).

## D-20 - MISSION CORRECTION (user directive, 2026-09-22)

Goal restated by the user: this is NOT a prediction system, and Level-4
temporal prediction was the wrong target for this organism class.
The mission: an alternative to LLMs - a PLASTIC, SELF-BUILDING neural
network that:
  1. learns INCREMENTALLY/PROGRESSIVELY by experience (no re-training;
     unlike an LLM which requires retraining);
  2. remembers persistently and retrieves (memory, not prediction);
  3. self-builds (constructs its own structure, like multi-cellular
     organisms);
  4. does ELIMINATION/SELECTION among a few possibilities under current
     circumstances (pick the most probable tendency), NOT generative
     prediction.
"Predict" is replaced by "select/eliminate". Multi-cellular organisms do
not predict; they discriminate and choose.

Implications for the program:
- Level-4 temporal prediction line stays CLOSED (D-09..D-16) but is
  demoted from "the objective's direction" to "one attempted, failed,
  unnecessary-for-mission capability". Corrected in
  docs/phase3/level4-decision.md rationale going forward.
- The confirmed assets ARE the goal's core: robust retrieval (Level-3 =
  persistent memory), 100% per-trial action encoding (the "select among
  alternatives" raw material), online STDP (experience learning, no
  retrain by construction), committed M3/M4/d-core candidate-permanence-
  prune machinery (synapse-level self-construction runs in every run).
- The genuinely un-met capability versus this goal: demonstrations of
  (a) progressive acquisition (+new pattern, old retained, no retrain)
  as a clean recorded result, (b) behavioral SELECTION between
  alternatives (contrastive-tag closed loop previously flagged), and the
  deep open question (c) self-building NEW NEURONS (structural growth,
  E4-family, historically regression-prone at the gate).
- Next slice should be chosen against THIS goal, not the temporal
  ladder: highest-value cheapest is (a) progressive acquisition demo on
  the existing substrate (curriculum change only), then (b) selection
  loop, then (c) structural growth when gated.

## D-21 - progressive-acquisition demo: PARTIAL (acquisition PASS, retention FAIL)

Curriculum-only run (S1 A/C -> S1D NEW pattern D 30 reps -> S3A/S3C/S3D
probes; no A/C exposure during S1D). 3 seeds, no runaway.
- ACQUISITION: D formed online+retained (S3D cos-self 0.80/0.91/0.92 vs
  its S1D ref; rho>0 vs A/C in 5/6). NEW pattern learned by experience,
  no retrain: CONFIRMED - core LLM-alternative claim evidenced.
- RETENTION: A/C re-expression degrades after D (S3 cos-self 0.9->0.1-0.4
  subs); C selectivity sign flips in 2/3 seeds; S3 rates -45..-76% vs
  no-D control. Catastrophic-forgetting half of the LLM problem IS
  present, in miniature. d_claim/d_core protection did NOT prevent it.
- Root: single shared all-excitatory pool reuses neurons for new
  patterns, reorganizing old assemblies (same ceiling as D-09..D-19).
- Next (approval required): selective memory protection during
  acquisition - pattern-gated plasticity or structured allocation; now
  concretely motivated by this measured forgetting.

## D-27 - direction: survival loop first; demo paused; evolution-to-selection is the goal; network SIZE is the open research question

User decision (2026-09-22): pause the I/O interactive demo (A decodes,
C/D misdecode on the cold end-of-run brain - load fixed by key-match,
remaining issue may be network too small). Proceed in order:
1. SURVIVAL LOOP first (D-22..D-24) - the environment + homeostatic drive,
   the thing evolution selects on.
2. Then HARNESS-INTEGRATED LIVE MODE (the faithful working demo; needs
   the live warm network, not snapshot reconstruction).
3. Then POPULATION + DEATH-SELECTION loop: spawn variants, each dies if
   it fails to thrive, only the fittest survives - "leave it on a loop,
   each thing dies, one ultimate organism remains."
KEY QUESTION (user, evidence-backed): is the 40-neuron shared pool big
enough to process? Every prior ceiling (temporal, closed-loop, forced
forgetting) traces to #neurons/shared-pool capacity. The survival loop +
death-selection is the design that makes size answer itself: if bigger
brains process better AND survive better, selection finds them; growth
becomes the fitness target. Recorded as the core hypothesis going forward.

## D-31 - Survival loop PASSES (frozen falsifier): closed-loop thrive measurable

3 seeds, S1->SURV direct (warm start), 30 beats each, no deaths.
KNOWN-VS-NOVEL (PRIMARY, bar>+0.05 in >=2/3): 0.133/0.063/0.099 =
3/3 PASS. The never-trained D probe drives measurably different
recognition/viability than known A/C (novel -> output silence/QUIET).
PERSISTENCE (SECONDARY): all 3 seeds survive full horizon, pool rate in
band, mean_v 0.63-0.78.
Pre-matrix corrections (pre-registered, not tuning): D-28 a_bounds +
sustained death; D-29 no-cold-silence + rest cadence (rebound seizure
was 340-400Hz from deep-quiet start); D-30 forced-novelty p_novel=0.2
(full autonomy degenerate-locks onto one known pattern) + withdraw->other-
known (novel->silence->D deadlock) + exact-match r(t).
HONEST LIMIT: post-loop runner skips V2Plasticity M2/E6 governors per
tick; benign-world OK, governor-dependency untested under harder
regimes (in-loop frame swap = deferred fidelity upgrade).
"Thrives" is now a measured, selected-able quantity - the precondition
for the evolution/death-selection slice.

## D-33 - evolution/death-selection verdict: SELECTION FINDS NO IMPROVEMENT (founders at local optimum)

Determinism check PASSED (best-org rebuild = scored, bit-identical, all
seeds) - pipeline valid, rankings trustworthy. 8 gens x 3 seeds, N=4,
elitism, size band [32,56]:
- SIZE (PRIMARY): 40.0 -> 40.0 / 41.3 / 37.3 - no consistent direction;
  size is NOT selected in this world.
- FITNESS (SECONDARY): mean declined in all 3 seeds (0.72->0.41,
  0.52->0.05, 0.61->0.26); with elitism carrying the winner verbatim,
  the decline is entirely mutated offspring: variation is destructive.
CONCLUSION: the formed 40-neuron E-nogain brain is at a LOCAL FITNESS
OPTIMUM - random variation degrades it. Combined with D-21 (forgetting
cost) + Level-4 closure: the shared-pool ceiling is architecture-level,
NOT reachable by parameter/size/growth variation from inside. The
demonstrated core that survived everything: experience learning without
retrain + perfect known/novel recognition + closed-loop persistence.
Next: substrate-level registration (structured populations / inhibition
opposing consolidation / true neurogenesis) OR accept the bound and
document. User decision pending.
