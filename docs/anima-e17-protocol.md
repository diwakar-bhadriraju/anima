# ANIMA E17 — STDP-free topology necessity test (M3/M4)

Status: **FROZEN** (2026-09-18). User-approved (audit `6e9bd67`;
approval includes the causal-language correction: the arm cannot
prove "passive re-expression alone is sufficient" — M2, M6, E6,
decay, adaptation remain active; it tests only M3/M4 necessity in
the STDP-free regime).

## 1. Question

Are M3/M4 topology dynamics (candidate accumulation/permanence/
creation + pruning) NECESSARY for the rapid REV1 re-anchoring
observed when STDP is disabled?

## 2. Frozen substrate

Control = committed E16 STDP-off condition (`e16-stdpoff.toml`,
run `runs/e16-stdpoff-20260918T075855Z`, **reused, never rerun** —
config equivalence proven by the isolation test; only experiment
provenance may differ). Preserved in the test arm: STDP disabled,
M2 enabled, M6 enabled, E6 enabled, M5 enabled, adaptation,
neuron/network dynamics, all weights/parameters, curriculum,
SEQ->REV schedule, phase timing, presentation counts, seed
derivation, analyzers, telemetry, snapshots, E14 transition rule,
P2/engagement gates. No parameter changes, no compensation, no new
thresholds, no new mechanisms.

## 3. Conditions

- CONTROL: `e16-stdpoff.toml` (committed; k* = 1, bound (T0, REV1],
  T0 0.638/0.199 -> REV1 0.162/0.686 on 20260912).
- TEST: `e17-m3m4off.toml` = e16-stdpoff base + `[v2]
  disable_m3_m4 = true` + exp_id `e17-m3m4off`. disable_m3_m4 must
  disable M3 (accumulation/permanence/creation) AND M4 (pruning) and
  nothing else.
- Initial seed: 20260912 only. Stop after this arm; request approval
  before extending seeds. No factorial experiments.

## 4. Ablation isolation check (must pass before interpretation)

From the test arm's telemetry/snapshots (frozen instruments):
- emitted STDP events (kind 8/9) = 0
- M3 permanence/creation events (kind 6, candidate-permanence) = 0
- M4 prune events (kind 7, competitive-prune) = 0
- M2 remains active (per-neuron exc sum invariant at window
  boundaries; M2-on by config)
- M6 remains active (inhibitory endpoint movement present)
- E6 remains enabled (config; beta applies to M3-only under STDP-off
  — recorded as a config fact)
- M5 semantics unchanged (0 evictions expected at occupancy <= 12/40;
  budget events unchanged)
- config diff vs control = ONLY exp_id + disable_m3_m4 (isolation
  test in the suite)
If any isolation check fails: STOP, mark INCONCLUSIVE, report.

## 5. Primary endpoint (frozen E14 machinery, no new endpoints)

T0 A-B/B-C; REV1 A-B/B-C; first A-alignment; sustained k*; REV10;
REV60; selectivity; A-C; rates; engagement (candidate-permanence
events — registered as 0 by design under the arm; engagement gate
for this arm = live-state viability, failures, rates, and the
absence of pathological collapse); failures.

## 6. Frozen decision logic

- Test arm achieves the frozen REV1 re-anchoring criterion (sustained
  A-alignment k* <= horizon, canonical-comparable trajectory):
  **M3/M4 topology dynamics are NOT NECESSARY for the STDP-free REV1
  flip at this tested scale/seed.** Permitted interpretation: the
  flip survives removal of both Hebbian STDP and excitatory topology
  dynamics, leaving pre-existing afferent structure plus the
  remaining active non-topological processes as the operative system.
  NOT permitted: "passive re-expression alone is sufficient".
- Test arm does NOT re-anchor and retains the prior C-side
  association through the frozen decision horizon (alignment C at
  the registered checkpoints / k* fails):
  **the combined M3/M4 topology subsystem is NECESSARY in this
  frozen STDP-free condition.** No M3-vs-M4 individual inference.
- P2/engagement/measurement/isolation gates fail: **INCONCLUSIVE**.
  No tuning to recover the arm.

## 7. Secondary analysis (descriptive localization only; no causal
## inference from secondary differences)

Control vs test: input-afferent pool weights (bucket totals);
lower-B/upper-B/A-side/C-side readout; inhibitory weight movement
(M6 endpoint); M2 normalization behavior (invariant); recurrent
connectivity; passive excitatory weight movement (endpoint deltas);
activity trajectory across REV1 (spike counts by window quartile).

## 8. Cross-seed gate

Stop after the 20260912 arm and report. Extension to 9001/424242
only after explicit approval, and only if isolation passes, frozen
gates pass, and the outcome is interpretable.

## 9. Claim boundaries (approved)

E17 may establish at most: necessity or non-necessity of the COMBINED
M3/M4 topology subsystem in the STDP-free regime at the tested
seed/scale. It does NOT establish: passive re-expression sufficiency,
M3 or M4 individual necessity, M6 necessity/sufficiency, STDP-ON
mechanism identity, universal claims, interaction.

## 10. Config hash (raw file, recorded pre-run)

- e17-m3m4off.toml: appended at the implementation commit (before
  run).

## Appendix: amendments

- (none yet)