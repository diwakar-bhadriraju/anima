# ANIMA E16 — Causal necessity of M2 and STDP for rapid re-anchoring

Status: **FROZEN** (2026-09-18). User-approved: run-start ablation
semantics + 3-arm minimal comparison (audit `b435a55`). No edits
after this point except registered amendments.

## 1. Question

Does disabling M2 (excitatory normalization) or STDP prevent the
REV1 C->A re-anchoring that occurs under the frozen canonical system?
Necessity experiment only — no sufficiency, joint-necessity,
interaction, or universal claims beyond the tested scale/seeds.

## 2. Frozen baseline

Exact frozen E10/E12 mechanism stack and REV1 experiment
(60 SEQ / 60 REV, variant_block=60). Preserved unchanged: organism,
E6 β, M3/M4/M5/M6, adaptation, network dynamics, neuron parameters,
STDP parameters except the registered zeroing, curriculum, timing,
rates, phase durations, presentation counts, seed derivation,
schedules, analyzer definitions, E14 transition rule, P2/rate/
engagement gates, telemetry, snapshot machinery, endpoints.

## 3. Arms (run-start ablation; deterministic initial condition =
## canonical config, seed; no compensation, no replacement rules)

| arm | exp_id | config | change |
|---|---|---|---|
| A canonical | e16 | e16.toml | none (e12.toml copied; exp_id only) |
| B M2-off | e16-m2off | e16-m2off.toml | `[v2] disable_m2 = true` only |
| C STDP-off | e16-stdpoff | e16-stdpoff.toml | `[plasticity] a_plus = 0.0, a_minus = 0.0` only |

Ablation represented explicitly in the config and in run-start
provenance (config hash in RunStarted). No M2×STDP-off arm, no other
ablations.

## 4. Canonical reproduction gate

The canonical arm reruns e12.toml (same seed). Gate = telemetry
byte-identity with the committed E12 run (runs/e12-20260916T202445Z,
hash pins below) AND the E14 frozen endpoints reproduce (T0 anchor
0.680/0.075 reference-anchored; k* = 1; REV1 A-B 0.190/B-C 0.929).
If the canonical arm fails materially (hash mismatch or endpoint
divergence) or any frozen gate fails: mark the initial comparison
INCONCLUSIVE/BLOCKED, report, do not interpret ablations causally,
do not modify thresholds.

## 5. Endpoints (frozen machinery, no new thresholds)

Per arm (from the new runs' telemetry, read-only instruments):
- E14 sustained A-alignment k* (first REV presentation whose
  reference-anchored alignment is A, sustained through REV10);
  REV1 checkpoint trajectories; T0 B-alignment; first A-alignment;
  sustained-A achieved by REV60;
- A-B / B-C / A-C (raw == L1 frozen method); B-independence (0.60
  frozen); selectivity; P2 status; engagement (candidate-permanence
  events); failures.
- T0 = reference window rounds 51-60 (E14 estimator).

## 6. Secondary mechanistic observations (per ablation)

M2 arm: normalization invariant (per-neuron exc sum distribution at
window boundaries — enabled arms only), excitatory weight
redistribution persistence, endpoint weight movement, structural
events, lower-B/upper-B/A/C bucket mapping (E15 instrument).
STDP arm: emitted STDP event counts (enabled), absence of STDP
events (disabled arm), endpoint weight movement, structural events,
bucket mapping. Direct-weight vs topology vs activity-mediated
effects kept distinct; no causality from endpoint correlation.

## 7. Decision logic (per mechanism, per seed)

- canonical flips AND arm does not flip => mechanism INDIVIDUALLY
  NECESSARY at this tested scale/seed.
- canonical flips AND arm flips => NOT individually necessary.
- canonical or arm fails gates => mark arm; no forced causal
  interpretation.
No sufficiency/joint-necessity/interaction/universal claims.

## 8. Cross-seed gate

After the 20260912 three-arm comparison is clean (gates pass,
canonical reproduction passes): run the same three arms for seeds
9001 and 424242 (e12-seed9001.toml / e12-seed424242.toml bases).
No changes except seed. Per-seed results preserved; no collapsing
seed-dependent results into universal claims.

## 9. Stop conditions

Stop and report if: canonical reproduction fails; any implementation
change alters frozen machinery outside the intended ablation;
telemetry/determinism fails; P2 fails in a way that prevents
interpretation; ablation semantics cannot be proven isolated.
No repair by tuning; no parameter sweeps; no E17 proposal.

## 10. Config hashes (sha256, raw files, recorded pre-run)

- e16.toml `47a4ff96aed72cfa`; e16-m2off.toml `20479de8c2e8eefc`;
  e16-stdpoff.toml `965bc7776f6ef82d` (recorded at implementation,
  before any run).

## Appendix: amendments

- (none yet)

## Appendix: cross-seed configs (registered pre-run, hashes raw files)

- e16s9001.toml `a3a56556586b62db`; e16s9001-m2off.toml
  `9797ea8ed36740a3`; e16s9001-stdpoff.toml `4e4413633d9fffe1`;
  e16s424242.toml `5829f68bdef584d6`; e16s424242-m2off.toml
  `669a2a2645376921`; e16s424242-stdpoff.toml `68050b1f6caa1cce`.
  (Note: initial generation reused the base exp_id across arms of a
  seed — run-dir collision; corrected with per-arm exp_ids and
  re-recorded before any arm ran.)
---

## E16 execution record (2026-09-18)

Implementation `f133ab1` (arm configs + isolation tests); cross-seed
configs `19d44be` (+ correction for per-arm exp_ids, hashes
re-recorded pre-run). 132 tests green at execution, 0 warnings.

### Canonical reproduction gate — PASSED (all three seeds)

Snapshot frame hash of the canonical rerun == committed E12 run
(`005acea9a7f671a4`, 20260912); telemetry differs ONLY in the
registered exp_id provenance (per-chunk meta + RunStarted; per-file
sizes identical, diff clusters = the `e12`->`e16` character).
Endpoints identical to the committed E14 record for every canonical
arm (20260912 REV1 0.190/0.929 ... 0.230/0.905; 9001 0.127/0.672;
424242 0.112/0.746; k* = 1 all).

### Per-arm endpoint matrix (E14 frozen machinery; k*: sustained
### A-alignment; all arms k* = 1, bound (T0, REV1])

| seed | arm | T0 A-B / B-C | REV1 A-B / B-C | REV60 A-B / B-C | R10 sel / perm / rates | failures |
|---|---|---|---|---|---|---|
| 20260912 | canon | 0.680 / 0.075 | 0.190 / 0.929 | 0.270 / 0.837 | 0.869 / 585 / 9.3-77.1 | 0 |
| 20260912 | m2off | 0.992 / 0.766 | 0.741 / 0.992 | 0.779 / 0.942 | 0.476 / 94 / 30.9-163.2 | 0 |
| 20260912 | stdpoff | 0.638 / 0.199 | 0.162 / 0.686 | 0.171 / 0.651 | 0.875 / 530 / 3.9-31.1 | 0 |
| 9001 | canon | 0.760 / 0.097 | 0.127 / 0.672 | 0.158 / 0.610 | 0.836 / 601 / 9.6-94.2 | 0 |
| 9001 | m2off | 0.989 / 0.766 | 0.740 / 0.987 | 0.868 / 0.884 | 0.507 / 93 / 27.8-162.7 | 0 |
| 9001 | stdpoff | 0.671 / 0.231 | 0.179 / 0.801 | 0.182 / 0.725 | 0.724 / 572 / 4.0-32.5 | 0 |
| 424242 | canon | 0.554 / 0.065 | 0.112 / 0.746 | 0.225 / 0.516 | 0.907 / 488 / 8.7-81.8 | 0 |
| 424242 | m2off | 0.991 / 0.781 | 0.798 / 0.982 | 0.842 / 0.932 | 0.487 / 115 / 30.5-154.1 | 0 |
| 424242 | stdpoff | 0.744 / 0.228 | 0.215 / 0.772 | 0.190 / 0.694 | 0.603 / 544 / 3.8-28.5 | 0 |

Gates: 0 failures everywhere; rate maxima 28.5-163.2 Hz (in the
frozen [20,250] band); engagement (candidate-permanence per 10-round
window) nonzero in every arm (93-601). Analysis deterministic
(byte-identical reruns).

### Decision logic (frozen section 7) — per mechanism, per seed

k* = 1 in EVERY arm of EVERY seed: canonical flips AND both
ablation arms flip => **neither M2 nor STDP is individually
necessary for the REV1 re-anchoring at the tested scale/seeds
(9/9 runs)**.

Mechanistic observations (registered, no causal claims):
- STDP-off reproduces the canonical regime almost verbatim: T0
  side-absorption present (weaker: 0.638/0.199, 0.671/0.231,
  0.744/0.228), REV1 A-absorption near-canonical, selectivity and
  permanence healthy (0.60-0.88; 469-921/window), activity low
  (3.6-4.0 mean / 28-32 max Hz) but engaged. The flip therefore does
  NOT require Hebbian weight updates: with STDP zeroed the only
  weight-moving processes are M3 maturations, M2 re-pinning, M6,
  and passive decay — the A-side readout re-expresses pre-existing
  weights under the reversed input statistics.
- M2-off does NOT prevent the alignment reversal, but it destroys
  the canonical SEPARATION regime: T0 A-B ~0.99 / B-C ~0.77-0.78 in
  all seeds (B near-identical to both sides — the canonical
  C-absorbed state never develops), selectivity drops (0.48-0.51),
  permanence starves (93-170/window vs 488-601), rates rise
  (28-31/154-163 Hz). The "flip" in this arm is a C->A reversal of a
  degenerate representation (REV60 A-B 0.78-0.87 — still entangled).
  M2 is not necessary for the reversal but is demonstrably required
  for the canonical separated regime (consistent with the v2-M2
  collapse knowledge, now at E12 scale).
- Common to all arms: A-C separated throughout; B-independence never
  achieved at REV1 in any arm (frozen 0.60 rule).

### Stop-condition check

No implementation change outside the registered ablations (isolation
tests green); telemetry/determinism clean; P2 clean; ablation
isolation proven by config-diff tests and the deterministic initial
conditions. No tuning, no thresholds, no new mechanisms.

Interpretation boundaries: this establishes NON-NECESSITY (of M2 and
STDP individually) at the tested scale/seeds — not sufficiency, not
joint structure, not interaction, not universal claims. The
flip's survival under both ablations leaves the remaining machinery
(M3 maturations + M6 + M2-invariant-free dynamics + activity
statistics on existing weights) as the operative set; no mechanism
identity is claimed beyond non-necessity.
