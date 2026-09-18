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