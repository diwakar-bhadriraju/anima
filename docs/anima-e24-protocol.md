# ANIMA E24 — Regime-selection capacity (FROZEN PROTOCOL)

Promoted from draft v2 (docs/draft-e24-regime-capacity.md, commit
4f54a2b) unchanged in design. Frozen 2026-09-19 BEFORE execution.
Category: measurement experiment on committed V2.1 substrate
(only registered V2.1 config fields + curriculum vary). No
mechanism change. No detector change. Not an amendment of Stage C
(that record stays retracted/not-executed and untouched).

## Question

Do endogenous dynamics retain information about prior drive
composition after drive offset (memory, not reflex)?

## Design (frozen)

- Substrate: V2.1, β=0.0046875, τ_s=5000 (X3 transition band;
  0/15 X3 aborts). v21probe schedule otherwise: S0 5 s silence |
  S1 drive: 40 presentations, 500 ms each, 2 s cadence | S2 20 s
  silence. Seed frozen per run.
- 6 seeds (pre-declared): 20260912, 424242, 9001, 123456, 777,
  31337.
- 5 arms (composition only; total presentations 40 in all arms):
  a   = 40×A interleaved
  c   = 40×C interleaved
  il  = 20×A + 20×C interleaved (seeded shuffle)
  bac = 20×A then 20×C (blocked)
  bca = 20×C then 20×A (blocked)
- 30 runs total, deterministic. Configs: configs/e24-s{seed}-{arm}
  .toml (scripts/gen_e24.py). No retries, no censoring, no
  mid-experiment changes of any kind. Aborts preserved.

## Primary endpoint (frozen; instrument e24_endpoint.rs)

M per run:
- run aborts on runaway-activity  ⇒ M = +∞ (top rank)
- otherwise                        ⇒ M = log10(1 + S10)
  S10 = endogenous (non-input, id≥24) spikes in
  [silence_onset, silence_onset + 10 s), silence_onset = last
  StimulusPresented t + 500 ms (relative window; delivery-tick
  convention arm-symmetric).

## Hypotheses (frozen; mutually exclusive patterns)

- H0 pure count: a = c = il = bac = bca (in M).
- H1 composition: a ≠ c (channel-identity effect).
- H2 recency/order: bac ≈ c and bca ≈ a; |bac − bca| ≈ |a − c|.
- H3 interference: il < max(a, c) (interleaving suppresses
  consolidation).

## Statistical procedure (frozen)

Within-seed paired contrasts on M; the seed is the statistical
unit (n=6). Exact two-sided sign test on paired differences
(min attainable p = 2/26 = 0.031) with median within-seed Δlog10
and exact CI. Aborts enter as +∞ ranks (all arithmetic on ranks;
for medians of Δ involving an abort the Δ is +∞-dominated and is
reported as such). Contrasts: C1 a vs c; C2 bac vs bca; C3 il vs
mean(bac, bca) (pair each seed's il against that seed's mean of
the two blocked arms). Secondary contrasts reported but
non-primary: bac vs c, bca vs a (H2's convergence form).

Verdict rules (frozen):
- A hypothesis Hk is SUPPORTED if its full pattern holds across
  contrasts and no simpler hypothesis explains the pattern; H0 is
  the null of "no curriculum information retained".
- Primary claim "endogenous dynamics retain drive-composition
  information" requires: C1 or C2 significant (p ≤ 0.05 exact
  sign) AND the winning pattern consistent (H1 or H2 shapes).
- All other outcomes = NOT SUPPORTED (retained as negative
  results).

## Secondary readouts (frozen list, descriptive/mechanistic)

Regime class (silent/sparse-core/pacemaker/irregular; thresholds
as in v21phase), drive-end top-u (neuron id + magnitude) and
top-3 u sum, winner-identity overlap across arms within seed,
silence trajectory (2 s bins, 20 s), output-class participation
in silence (ids + spike counts). Prediction checks (u-margin
hypothesis, falsifiable): P1 Spearman(M, top-u); P2 direction of
bac/bca matches second block's pure arm; P3 no per-trial A/C u
separation (optional tertiary).

## Freeze integrity

- Config generator scripts/gen_e24.py (committed pre-exec).
- Endpoint instrument crates/anima-exp/examples/e24_endpoint.rs
  (committed pre-exec; sanity-checked against X3 cells).
- This document is the protocol of record; execution record
  appended below after runs complete.
