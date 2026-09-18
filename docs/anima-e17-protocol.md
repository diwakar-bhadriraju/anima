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

- e17-m3m4off.toml `6000aa7c04ba805f` (recorded at implementation,
  before the run).

## Appendix: amendments

- (none yet)

## Appendix: cross-seed replication configs (registered pre-run,
## raw-file hashes; = committed stdpoff base + disable_m3_m4 + exp_id)

- e17s9001-m3m4off.toml `44204d8d5ef091e9`
- e17s424242-m3m4off.toml `1a8103aeb52b84ea`
---

## E17 execution record (2026-09-18)

Implementation `10faf45` (config e17-m3m4off.toml, hash
`6000aa7c04ba805f` recorded pre-run; isolation test). Run
`runs/e17-m3m4off-20260918T135900Z` (seed 20260912). 133 tests
green at execution, 0 warnings.

### Ablation isolation check — ALL PASS

- STDP kind 8/9 events in interval: 0 (strengthened=0, weakened=0).
- M3 maturations: 0 (kind 6 candidate-permanence, whole run: 0
  permanence alive at any snapshot; permanence column 0).
- M4 competitive-prunes in interval: 0 (0 churn: symmetric-diff 0
  neurons / 0 synapses pre vs post).
- M5 budget-evictions: 0.
- M2 active: per-neuron exc sum invariant mean|sum-0.8| <= 4e-6 at
  all measured instants (52 neurons).
- M6 active: 431/514 inhibitory synapses endpoint-changed, net dW
  -1.389.
- E6/M5/adaptation semantics unchanged (config isolation test).
- Config diff vs control (e16-stdpoff) = exp_id + disable_m3_m4
  only (suite test e17_arm_isolated_topology_ablation).

### Primary endpoint (frozen E14 machinery)

| cp | A-B | B-C | align | indep | sel | perm | rates |
|---|---|---|---|---|---|---|---|
| T0 | 0.628 | 0.207 | C | F | — | 0 | — |
| REV1 | 0.167 | 0.689 | A | F | 0.779 | 0 | 3.6/28.3 |
| REV2 | 0.152 | 0.664 | A | F | 0.673 | 0 | 3.6/28.3 |
| REV3 | 0.173 | 0.628 | A | F | 0.645 | 0 | 3.6/30.8 |
| REV4 | 0.194 | 0.646 | A | F | 0.638 | 0 | 3.7/30.8 |
| REV5 | 0.170 | 0.683 | A | F | 0.632 | 0 | 3.7/30.8 |
| REV6 | 0.145 | 0.633 | A | F | 0.623 | 0 | 3.7/30.8 |
| REV7 | 0.184 | 0.666 | A | F | 0.610 | 0 | 3.7/30.8 |
| REV8 | 0.216 | 0.615 | A | F | 0.625 | 0 | 3.7/30.8 |
| REV9 | 0.177 | 0.668 | A | F | 0.603 | 0 | 3.8/30.8 |
| REV10 | 0.168 | 0.670 | A | F | 0.859 | 0 | 3.9/30.8 |
| REV20 | 0.138 | 0.649 | A | F | 0.882 | 0 | 4.0/31.2 |
| REV30 | 0.182 | 0.665 | A | F | 0.889 | 0 | 3.9/32.8 |
| REV40 | 0.168 | 0.643 | A | F | 0.889 | 0 | 3.9/31.9 |
| REV50 | 0.195 | 0.582 | A | T | 0.890 | 0 | 3.8/33.0 |
| REV60 | 0.177 | 0.651 | A | F | 0.920 | 0 | 4.0/34.4 |

First A-alignment REV1; sustained k* = 1 through REV10 and through
REV60; A-B/B-C raw == L1; A-C separated throughout; failures 0;
rates 3.6-4.0 mean / 28.3-34.4 max Hz (in band); engagement by the
registered arm semantics (live viability: 1,108 live exc / 514 inh
at all instants, no collapse, selectivity rising to 0.920).

### Frozen decision outcome

The test arm ACHIEVES the frozen REV1 re-anchoring criterion
(k* = 1, trajectory near-identical to the committed STDP-off
control: control T0 0.638/0.199 -> REV1 0.162/0.686 -> REV60
0.171/0.651; arm T0 0.628/0.207 -> REV1 0.167/0.689 -> REV60
0.177/0.651).

=> **M3/M4 topology dynamics are NOT NECESSARY for the STDP-free
REV1 flip at this tested scale/seed (20260912).**

Permitted interpretation (per approval): the flip survives removal
of both Hebbian STDP and excitatory topology dynamics; pre-existing
afferent structure plus the remaining active non-topological
processes (M2 re-pinning, M6, passive decay, adaptation, network
dynamics; E6 beta active but M3-absent) constitute the operative
system. NOT claimed: passive re-expression alone is sufficient.

### Secondary observations (descriptive; no causal inference)

- Bucket pools at T0 (arm): A-side 14.12 (A-only 8.033 + lower-B
  6.091), C-side 14.35 (7.138 + 7.214), inactive 13.124, recurrent
  n=491 w=0.000 (initial recurrent wiring persists at the weight
  floor — with no LTP and no M3/M4 the prunes that collapsed
  recurrence in the control never occur; control had n=1).
- Endpoint movement: 38% of alive-both synapses (vs 89% control) —
  only decay + M2 re-pin + M6 move weights; per-bucket |dW| <= 0.03.
- M6: net -1.389 (inhibitory, activity-mediated).
- Activity: 471 internal spikes in the REV1 window (q1 250, q2 196,
  q3 25, q4 0) — phase-reversal-driven, matches control profile.
- The flip occurs with ZERO topology events, ZERO STDP events, and
  ~0.03 total bucket weight drift.

### Stop condition

Cross-seed gate: STOP after the 20260912 arm. Requesting approval
before extension to 9001/424242 (isolation passed, gates passed,
outcome interpretable).

---

## E17 cross-seed replication record (2026-09-18)

Configs `44204d8d5ef091e9` (9001) / `1a8103aeb52b84ea` (424242)
registered pre-run; isolation test `e17_replication_arms_isolated`
(green); runs `runs/e17s9001-m3m4off-20260918T155612Z`,
`runs/e17s424242-m3m4off-20260918T155612Z`. 134 tests green, 0
warnings. (Note: an initial commit shipped a red test — missing
.toml in v3_config names — fixed in `944eeee` before interpretation;
no effect on runs.)

### Isolation check (both seeds) — ALL PASS

STDP events 0; M3 maturations 0; M4 prunes 0; M5 evictions 0; M2
invariant (mean|sum-0.8| <= 4e-6 at every instant, 52 neurons); M6
active (9001: 410/517, net -1.049; 424242: 451/516, net -1.293);
0 failures; rates 24.5-33.4 Hz max; config diff vs the same-seed
committed stdpoff control = exp_id + disable_m3_m4 only.

### Primary endpoint (frozen E14 machinery)

- seed 9001: T0 0.671/0.239 (C) -> REV1 0.188/0.807 (A), first
  A-alignment REV1, sustained k* = 1 (REV10 0.236/0.715; REV60
  0.210/0.734 all A); selectivity 0.662 -> 0.822; A-C separated;
  failures 0; rates 3.8-4.4/29.9-32.7 Hz.
- seed 424242: T0 0.742/0.242 (C) -> REV1 0.230/0.785 (A), k* = 1
  (REV10 0.218/0.770; REV60 0.201/0.696 all A); selectivity 0.549
  -> 0.806; A-C separated; failures 0; rates 3.5-4.5/24.5-33.4 Hz.

Control comparisons (committed stdpoff arms, never rerun): 9001
T0 0.671/0.231 -> REV1 0.179/0.801 -> REV60 0.182/0.725 vs arm
0.671/0.239 -> 0.188/0.807 -> 0.210/0.734; 424242 0.744/0.228 ->
0.215/0.772 -> 0.190/0.694 vs arm 0.742/0.242 -> 0.230/0.785 ->
0.201/0.696. Trajectories near-identical; B-independence episodes
unchanged in pattern (never at REV1).

### Replication decision (per seed, frozen rule)

- 9001: k* = 1, sustained A-alignment -> **REPRODUCED**.
- 424242: k* = 1, sustained A-alignment -> **REPRODUCED**.
- (20260912: REPRODUCED, prior record.)

**Three-seed replication: 3/3 — M3/M4 topology dynamics are NOT
NECESSARY for the STDP-free REV1 flip under the tested conditions.**
Seed-dependent boundary: all three flip at REV1 with k* = 1; final
A-B values differ slightly per seed (0.177 / 0.210 / 0.201) — not
collapsed into a universal claim.

### Secondary observations (descriptive only)

9001: recurrent n=560 at w=0 persists (control collapsed to 1);
A-side 13.41 / C-side 14.20 at T0; per-bucket drift <= 0.03.
424242: recurrent n=519 at w=0; A-side 13.08 / C-side 15.04; drift
<= 0.02. Spike profiles match controls (q1 149/190, q2 265/210,
q3 45/26, q4 0). Endpoint movement on 38-63% of recurrent-bucket
synapses (decay + M2 re-pin + M6 paths).

### Stop condition

STOPPED as registered. No further ablation proposed. Limitations
held: M3/M4 non-necessity refers to the COMBINED topology subsystem
in the STDP-free regime; NOT claimed — re-expression sufficiency,
individual M3 or M4 non-necessity, M6 necessity/sufficiency,
universal mechanism identity.
