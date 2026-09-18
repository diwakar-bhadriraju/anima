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