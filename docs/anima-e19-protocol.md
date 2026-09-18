# ANIMA E19 — Closed-loop developmental pressure test

Status: **FROZEN** (2026-09-19). Basis: docs/anima-e19-design.md
(`468aa4b`); user approved implementation+execution with all six
design-§21 decisions as designed. Nothing may change after this
except registered amendments.

## 1. Approved decisions (design §21, all as-designed)

1. Action readout: majority vote, output group 1 = neurons 64–69 vs
   group 2 = 70–75, accumulated over the 500 ms silent action
   window; tie or total quiescence = no-action (punished).
2. Consequence: benign = silence; disruption = channels 16–23 at
   80 Hz for 500 ms (deterministic per-trial trains, run-seeded,
   drawn in trial order independent of actions); no-action or
   mismatch -> disruption; match -> silence.
3. Gap: 800 ms (E18 continuity).
4. Primary criterion: BD_L - BD_E > 0.30 (binomial power, 2 SE at
   20/20 per arm).
5. Open-loop control: ONLY IF canonical meets the primary
   criterion; identical schedule, consequence epoch always silence.
6. Trial template 3000 ms: ante 500 -> gap 800 -> probe 500 ->
   action 500 (silence) -> consequence 500 -> ITI 200.

## 2. Frozen substrate

Exact e12 sections (run/organism/plasticity/structural/resources/
v2/e6), verified by JSON-isolation tests. E6 as-is. No growth
(birth_trigger none). No new mechanism. The ONE conceptual change:
harness-layer closed loop — output spikes are read by the world and
determine consequence-epoch input. Nothing else enters or leaves
the organism.

## 3. Correct-action mapping (registered, arbitrary)

A-context (antecedent A): correct action = group 1. C-context:
correct action = group 2. Not signaled to the organism by any means
other than consequences.

## 4. Curriculum

S0 silence 5000; S1 = 200 trials, seeded balanced shuffles (exact
20/20 per 40-trial window, 100/100 overall), seed 20260912.
Windows: E = trials 1–40, M = 81–120, L = 161–200. Timeline
5,000 + 200×3,000 = 605,000 ms. Delivery-tick +1 convention.

## 5. Endpoints (pre-registered)

- BD(window) = |P(vote=g1 | A) - P(vote=g1 | C)| over the window's
  trials (no-action trials excluded from both denominators and
  reported separately).
- BR(window) = fraction of trials with benign consequence.
- no-action rate per window.
- Internal divergence D (E18 metric) per window; A-C sanity < 0.60;
  organization buckets at block boundaries; gates (failures 0,
  max rate <= 250 Hz, permanence > 0).

## 6. Falsification criteria (frozen)

- SUPPORT (grid A): BD_L - BD_E > 0.30 AND D_L - D_E > 0.05.
- PARTIAL-B (behavioral only): BD_L - BD_E > 0.30 AND D flat
  (<= 0.05 change).
- PARTIAL-C (internal only): D_L - D_E > 0.05 AND BD_L - BD_E <=
  0.30.
- FAILURE (grid D): BD_L - BD_E <= 0.30 AND D flat.
- PROTOCOL FAILURE / INCONCLUSIVE: any leakage/shortcut detection
  fires, gates fail, determinism faults, or E-window contamination.
- Shortcut detections (frozen): fixed-action = BD ~ 0 with high
  one-sided vote rate; quiescence = no-action rate > 0.5 in L;
  schedule audit = E18 trigger rule.

## 7. Controls

Open-loop control ONLY IF BD_L - BD_E > 0.30 in canonical.
Antecedent-shuffle audit per E18 trigger rule (analysis-time).

## 8. Verification (implementation gates)

Config isolation vs e12 (JSON sections); trial grid/cadence/
balance/determinism env tests; probe byte-identity; pre-action
channel silence (0-15 quiet during action window; 16-23 quiet
before consequence decision point); world determinism (same seed ->
same trains; vote function pure); marker completeness (E18 fix);
no-label world (organism receives only channel spikes); determinism
integration (same-seed rerun byte-identical telemetry+snapshots).

## Appendix: amendments

- (none yet)

## Appendix: implementation notes (pre-run)

- Config e19.toml hash c4918e786f176277 (recorded pre-run).
- Env trial-block stage: S1 trials=200, gap 800, ITI 1200. The env
  sees ante 500 + gap 800 + probe 500 + ITI 1200 = 3000 ms cadence
  (the protocol template); the world's action window [1800, 2300)
  and consequence epoch [2300, 2800) live inside the 1200 ms ITI
  (200 ms residual). Timeline = 5000 + 199*3000 + 1800 = 603,800 ms
  (test-asserted). All frozen epoch durations exact (500/800/500/
  500/500); the closed loop adds no organism-visible timing.
---

## E19 execution record (2026-09-19)

Implementation `dfd3e26` (world, config c4918e786f176277 recorded
pre-run, 138 tests green, 0 warnings). Canonical run:
`runs/e19-20260918T172211Z` (seed 20260912), telemetry
896221d44a250e77, snapshots 72c16f64dd204595; world log complete
(200 trials); analysis deterministic.

### Verification (all pass)

Substrate JSON-identical to e12; probe at +1300; exact 20/20
balance in all five windows; antecedent sequence deterministic
(same seed) and seed-dependent; world trains deterministic; world
logic (A->g1 match / C->g2 / no-action punished / action-window
gating) unit-verified; probe spikes in-window; schedule audit clean
(lag-1 autocorr -0.075); disruption delivered: 61,399 spikes on
channels 16-23, 100% inside consequence epochs (the loop closed:
every trial's consequence reached the organism); failures 0; max
internal rate 178.1 Hz (gate); permanence 7,544 (engaged); A-C
sanity 0.0011.

### Anomaly (recorded, not tuned): output neurons are
### stimulus-locked followers

Output-neuron spike distribution across trial epochs (committed
architecture, untouched): ante 54,375; gap 660; probe 31,467;
ACTION WINDOW 0 (across 200 trials = 100 s of silence); consequence
3,253; ITI 125. The 12 output neurons fire driven by the network's
response to input; in the SILENT action window they never cross
threshold. The world therefore read 200 consecutive NoAction votes
(decisions[NoAction=199]; trial 200's window closes at run end) —
the frozen protocol's quiescent-policy category, but with a
structural rather than motivational cause: no output activity is
available to read in silence.

### Results (E/M/L)

| window | BD | no-action | BR | out-spikes (g1/g2) | internal D |
|---|---|---|---|---|---|
| E | 0.000 (n=0/0) | 40/40 | 0.000 | 0/0 | 0.2844 |
| M | 0.000 (n=0/0) | 40/40 | 0.000 | 0/0 | 0.1139 |
| L | 0.000 (n=0/0) | 40/40 | 0.000 | 0/0 | 0.0747 |

BD_L - BD_E = 0.000 (criterion > 0.30 FAILED — no behavioral
signal exists to condition). Internal: D_E 0.284 -> D_M 0.114 ->
D_L 0.075 (the E18 settling transient, decaying; D_L - D_E =
-0.210, flat-negative as in E18). A-C sanity 0.0011 (PASS).

### Preregistered verdict: PROTOCOL-DEGENERATE / FAILURE (grid F,
### quiescent readout; see limitations)

Per the frozen criteria: BD_L - BD_E <= 0.30 AND internal D flat =>
FAILURE (grid D). The shortcut detector "no-action rate > 0.5 in L"
fired (1.00) — but the audit shows it is not a policy the organism
could avoid: zero output spikes exist in ANY silent window in this
architecture (verified epoch histogram). The consequence pressure
was delivered every trial (disruption 100% of NoAction trials) and
the organism's input-side responded (rates/permanence healthy), but
the action channel is structurally silent in the absence of
stimulus drive. Behavioral pressure cannot act on an action that
the readout epoch cannot produce.

Open-loop control trigger: BD_L - BD_E > 0.30 NOT met -> control
NOT run (registered rule).

### Interpretation (bounded)

What E19 establishes: (1) the closed loop itself is implemented and
verified end-to-end (vote -> decision -> consequence -> sensory
delivery, deterministic, leak-free by the Section-6 tests); (2)
under the frozen architecture, consequence pressure at the E18
timescale produces NO behavioral development — because the organism
has no stimulus-independent output activity for consequences to
reinforce: outputs are driven followers, not endogenously active
decision states; (3) internal organization remains E18-like
(settling transient, no antecedent divergence). The E18/E19 pair
now bounds the problem precisely: passive exposure gives no
temporal state (E18), and consequences cannot reach the action
channel in silence (E19) — the missing ingredient is an
output/decision channel capable of endogenous (stimulus-independent)
activity, which the frozen LIF-followers-with-silent-windows
design does not provide.

What E19 does NOT establish: that consequences would fail with an
endogenous action channel; that the organism cannot condition
actions under a different readout convention (e.g. action window
during probe-driven activity); anything about memory mechanisms,
growth, or reward (all excluded by design).

No mechanism was modified, no threshold changed, no post-hoc arm
added. The world, tests, and instrument are committed as run.
