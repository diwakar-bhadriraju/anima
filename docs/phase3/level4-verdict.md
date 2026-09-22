# Phase III Level-4 verdict — temporal prediction via eligibility-trace LTP

Status: FALSIFIED (reject; no patch per protocol-immutability),
2026-09-22. Instrument: examples/predict.rs + examples/gapstate.rs.

## Result (measured, 3 seeds × alternating il)

| run dir (d_elig config) | late-gap internal spikes | mean Π |
|---|---|---|
| clla-arex-s20260912-il-20260922T165455Z (clla-elig-s20260912-il) | 0 / 37 gaps | +0.0000 |
| clla-arex-s9001-il-20260922T165502Z (clla-elig-s9001-il)      | 0 / 34 gaps | +0.0000 |
| clla-arex-s424242-il-20260922T165509Z (clla-elig-s424242-il)   | 0 / 33 gaps | +0.0000 |

NOTE - run dirs inherit `exp_id` from the base (clla-arex-*); the config
passed was configs/clla-elig-s{seed}-il.toml with d_elig=true (verified
in-file line 153). d_elig activity confirmed behaviorally: per-pattern
rates diverge massively from the no-elig E-nogain baseline for the same
seed (C S1 159.2 -> 59.4 Hz, A S1 50.6 -> 21.0 Hz, seed 20260912). All
three d_elig runs completed (curriculum-complete; no runaway).

FROZEN falsifier: "PI ~ 0 across seeds after training => reject."
Achieved: PI = 0 exactly in all three seeds; more strongly, the LATe-GAP
INTERNAL SUBSTRATE IS EMPTY (0 spikes in 37/34/33 late-gap windows; the
presentation refs are non-trivial refmag 1781-2082, so the probe is
sound - presentations fire, gaps are silent).

## Mechanism diagnosis (why; evidence-tagged, not tuned)

The eligibility trace switched the STDP LTP pre-trace to the slow trace
(elg, tau 1500 ms), which reweighted the recurrent pool so that the
persistent plateau firing that sustained the E-nogain cross-gap state
(gapstate feasibility probe: 300-1200 spikes/1500 ms in the no-elig
baseline) no longer survives the gap. d_elig => LTP consolidation
shifted the rate regime (C dominated 59 Hz vs A 21 Hz; imbalanced), so
presentations fire but the gap is silent. The mechanism that was to USE
the bridge destroyed it. This reproduces the E4/E4c-family finding: in
this all-excitatory E-nogain regime, plasticity reweighting destabilizes
the very firing regime the mechanism depends on. The gap bridge is a
FRAGILE attractor of the base dynamics, not a stable substrate.

## Boundary

- Rejected: eligibility-trace-modulated LTP as the Level-4 temporal
  association lever. No parameter tuning (protocol 8).
- Preserved evidence: predict.rs + gapstate.rs committed; runs under
  runs/ (disk); no-elg E-nogain gap substrate reproducible.
- Next hypothesis (NEW registration, not a patch): make prediction a
  PASSIVE readout that does not feed back into LTP - a per-neuron trace
  read by the organism as anticipation of the next event, with the
  gap state LEFT in the untouched E-nogain dynamics (where it exists)
  and no plasticity lever. This is a different locus (readout, not
  plasticity) and preserves the fragile bridge. Requires selecting the
  external-response reference from the base dynamics and a new freeze.

STOP - falsified; evidence preserved; next branch is a new registration.