# ANIMA E20 — Consequence shaping of the post-probe response
# disposition (D1 interface)

Status: **FROZEN** (2026-09-19). Basis: action-interface audit
`f3bf699` (approved); E19 `2ce7072` stands as the failed-interface
predecessor (its FAILURE verdict is unchanged and valid).

## 1. Scientific question

Can environmental consequences shape the organism's response
disposition when the action interface reads its existing post-probe
activity (the echo — the only self-generated activity at the
decision point)?

## 2. Category-A interface correction (registered)

E19's vote window was a 500 ms SILENT window [1800,2300) of the
trial template; the committed organism produces zero spikes in
sustained silence (audit f3bf699, measured), so pressure could not
reach any actionable output. E20 changes exactly ONE thing: the
vote reads the first 150 ms after probe offset — [1800,1950) — the
network's own echo. This is an experimental-interface correction
(category A): no organism change, no mechanism, no tuning; the
honest prediction (from E18: the probe response is antecedent-
independent, cos >= 0.96 after settling) is FAILURE unless
consequences create the dependence that exposure did not — which is
the pressure question, now reachable.

## 3. Frozen variables (E19 verbatim)

Organism/config sections (JSON-isolated vs e12), patterns A/C/B
(B variant_block 100000, all-SEQ probes), 200 trials, seed 20260912,
balanced 20/20 per 40-trial window, 3000 ms trial template with
epochs ante [0,500), gap [500,1300), probe [1300,1800), consequence
[2300,2800), ITI to 3000; consequence law: benign = silence,
disruption = channels 16-23 @ 80 Hz for 500 ms on mismatch or
no-action; mapping A-context -> group 1 (64-69) correct, C-context
-> group 2 (70-75) correct; vote = majority of accumulated output
spikes, tie/zero = no-action; disruption trains run-seeded in trial
order; delivery-tick +1 convention; telemetry/snapshots/world-log;
E/M/L = trials 1-40 / 81-120 / 161-200.

## 4. The ONE change

Vote window: [1800,1950) (150 ms post-probe echo; audit-measured
echo duration ~100-150 ms; fixed BEFORE any run; not to be
re-optimized after results). Decision closes at 1950. Consequence
epoch unchanged [2300,2800). Everything else byte-identical.

## 5. Endpoints and criteria (E19 frozen verbatim)

BD(window) = |P(vote=g1|A) - P(vote=g1|C)| (no-action excluded,
reported); BR; no-action rate; internal D on probe epochs (E18
metric); A-C sanity < 0.60; gates (failures 0, max rate <= 250,
permanence > 0).

- SUPPORT: BD_L - BD_E > 0.30 AND D_L - D_E > 0.05.
- PARTIAL-B: BD criterion met, D flat. PARTIAL-C: D criterion met,
  BD flat. FAILURE: both flat (<= thresholds). INCONCLUSIVE/
  PROTOCOL FAILURE: leakage/gate/determinism failures.
- Shortcut detectors (frozen): fixed-action (BD ~ 0 with high
  one-sided vote rate — now a real risk since the echo votes);
  quiescence (no-action > 0.5 in L); schedule audit (E18 rule).

## 6. Leakage/shortcut audit (design-level)

Unchanged items carry over from E19 (verified there): probe byte-
identity, template constancy, balance, determinism, virgin
consequence channels, no reward/teacher. New items for D1:
- The vote reads output spikes during [1800,1950) — input channels
  are silent then (probe ended; disruption only at 2300+), so the
  vote is the organism's own echo, not world input. Test: zero
  input-channel spikes in [1800,1950).
- The echo is stimulus-driven (probe) — registered as the honest
  property of D1: the interface reads response disposition, not
  deliberation. The antecedent-leak question reduces to E18's
  measured antecedent-independence of the probe response (no leak;
  the shortcut risk is the opposite: a stimulus-reflex vote).
- Consequence timing unchanged; vote close at 1950 does not alter
  any epoch.

## 7. Verification gates (before interpretation)

Config isolation: e20 == e19 except exp_id. World: 150 ms window
unit-verified; E19 world behavior byte-identical when exp_id is e19
(regression). Determinism: antecedents, trains, world, telemetry.
Leakage: input-silence in the vote window; probe byte-identity;
balance; schedule audit. Suite green.

## Appendix: amendments

- (none yet)

## Appendix: config hash (pre-run)

- e20.toml: recorded at implementation (before any run).