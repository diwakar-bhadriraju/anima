# CLLA bounded-protection amendment — execution record & viability verdict

Status: PROTOCOL EXECUTION RECORD, 2026-09-20. Amendment frozen in
docs/x-clla-amendment-review.md (§2A clip at headroom; §6 protocol).
Implementation: commit 925146c (protected-LTP clip in stdp_tick,
plasticity.rs LTP pass; gated on assembly_protect && s.consolidated).
All runs preserved in runs/ (12 experimental + 1 identity rerun),
including the informative D arm. No tuning, no post-hoc thresholds.

## 0. Scope (frozen)

This experiment answers EXACTLY ONE question:

    Does bounded protected growth make CLLA dynamically viable?

F2/F6/memory capability NOT measured or interpreted here.

## 1. Identity & integrity gates — PASS

- Ident arm (clla-ident-il-20260920T181711Z, flag off, clip present):
  event FNV-1a 9647ea8a0ca4dbd2, rows=152254 — byte-identical to the
  pre-amendment ident run and to the committed e24-il baseline
  (modulo the documented V2.2-era z_latch serialization drift).
- Snapshot frames: 105/105 identical vs pre-amendment ident run.
- Suites: core 78 (incl. 2 new clip tests), exp 58, telemetry 13,
  viz 5 — all green.
- The clip is unreachable flag-off (gated on assembly_protect), so
  the baseline LTP path is byte-exact by construction + verified.

## 2. Runs (12/12 curriculum-complete; telemetry-authoritative)

| run | arm | ended | failures |
|---|---|---|---|
| clla-s20260912-il-20260920T181753Z | il | curriculum-complete @105001 | 0 |
| clla-s20260912-bac-20260920T181759Z | bac | curriculum-complete @105001 | 0 |
| clla-s20260912-bca-20260920T181805Z | bca | curriculum-complete @105001 | 0 |
| clla-s20260912-d-20260920T181847Z | d (informative) | curriculum-complete @105001 | 0 |
| clla-s424242-il-20260920T181810Z | il | curriculum-complete @105001 | 0 |
| clla-s424242-bac-20260920T181816Z | bac | curriculum-complete @105001 | 0 |
| clla-s424242-bca-20260920T181821Z | bca | curriculum-complete @105001 | 0 |
| clla-s424242-d-20260920T181854Z | d (informative) | curriculum-complete @105001 | 0 |
| clla-s9001-il-20260920T181827Z | il | curriculum-complete @105001 | 0 |
| clla-s9001-bac-20260920T181833Z | bac | curriculum-complete @105001 | 0 |
| clla-s9001-bca-20260920T181838Z | bca | curriculum-complete @105001 | 0 |
| clla-s9001-d-20260920T181900Z | d (informative) | curriculum-complete @105001 | 0 |

All end reasons read from RunEnded telemetry events (not CLI echo).
Identity rerun (clla-ident-il-20260920T181711Z) also complete.

## 3. Viability criteria

### V1: zero P2/runaway failures — PASS (9/9 primary, 12/12 total)

All 12 runs: RunEnded reason curriculum-complete at t=105001;
metrics.json failures = [] everywhere. Peak mean internal rates:
primary arms 33.6–56.7 Hz (max 1 consecutive >50 Hz snapshot —
transient, far below the 5 s sustained P2 window); the frozen
unbounded-CLLA F4 runs aborted at 22–70 s with 62–139 Hz sustained.

### V2: P ≤ 0.6 + 1e-6 at every applicable snapshot — PASS (12/12)

Per-neuron protected max over ALL snapshots, every run:
maxPmax = 0.6000000 exactly (over-cap = +0.00e+00 at f32
precision). The clip holds the invariant exactly — protected mass
fills to cap and never exceeds it, in every seed/arm including the
previously-F4 cells. Witness trajectory (previously-F4 s424242-il):
Pmax climbs 0 → 0.6000 by t≈20 k and stays pinned at 0.6000 for
the remaining 85 s while total P grows 12.9 → 28.95 (more neurons
filling to cap, no neuron ever over).

### V3: M2 working target strictly > 0 at every applicable update — PASS

P ≤ 0.6 < t_e = 0.8 at every snapshot ⇒ W-target = t_e − P ≥ 0.2 >
0 by construction. Empirically: working-sum W tracks the budget
cleanly (s424242-il: W 41.6 → 12.5 as P fills; dist metric 0.000
everywhere — M2 normalizes working mass without the degenerate
target≤0 skip; the previously-F4 "W grows to 130–195" signature is
gone; max W any run = 41.6 = 52 × 0.8 with P at drive-start ≈ 0).

### V4: no new failure class — PASS

failures=[] in every run; the only failure kind in the protocol's
entire execution history remains runaway-activity (P2), which
never fired here. No resource, structural, or recorder failures.

## 4. Verdict

    CLLA + bounded protected growth is DYNAMICALLY VIABLE
    (V1–V4 all pass in all 9 primary runs; 12/12 total).

The single question is answered: yes, bounded protection makes the
architecture stable. The previously-F4 cells (s424242-il,
s9001-il, s20260912-bac, s9001-bac) now complete with rates
3.5–6.5 Hz and pinned protected mass.

## 5. D-arm (informative only, per frozen scope)

All 3 D runs now complete (previously 3/3 aborted). BUT: the
D-arm crosses 50 Hz in its FIRST presentation (P ≈ 0.16/neuron ≪
cap) — the cap does not engage before the crossing — so D-arm
coexistence with the cap does NOT attribute the improvement to the
bounded class; equally, D-arm instability would not have refuted
it. s9001-d still reaches 138.8 Hz transient (max 3 consecutive
snapshots > 50 Hz — under the 5 s sustain window), matching the
first-presentation over-drive shape. The flag-off D reference does
not exist; D remains a reported confound, not evidence.

## 6. What this does NOT claim

- NOT memory capability (F1/F2/F6/G1–G8 intentionally unmeasured —
  that is the next authorized experiment, if any).
- NOT that the cap is the only possible bound.
- NOT that p_max_frac = 0.75 is "enough learning room" — the
  working-pool lifetime (W shrinking as P fills) is reported, not
  interpreted; capability tests will decide sufficiency.
- NOT an E-number. No E-number exists for this execution.

## 7. Artifacts

Instrument: examples/clla_traj.rs (unchanged from the F4 audit),
examples/endreason.rs, examples/snapdiff.rs, examples/v22ident.rs.
All runs preserved under runs/clla-*20260920T1817* (+1818*).
Implementation: commit 925146c; this record: (this file).

STOP — viability established; awaiting review before any memory
capability experiment.