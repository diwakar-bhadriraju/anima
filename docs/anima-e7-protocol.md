# ANIMA E7 — Pre-registered Protocol (frozen before implementation)

**Status: PRE-REGISTERED.** Written before any E7 code, config, or
run. Freezes the complete E6 organism (v2 M1–M6 + E6 rate
balancing) wholesale. The ONLY deliberate variable is the sensory
curriculum. No absence-gating, no anti-correlation rule, no new
mechanism of any kind is implemented or proposed here — E7 is a
**minimal distinguishing experiment** for the absence/disconfirmation
hypothesis. Frozen values change ONLY via a logged, user-approved
amendment; negative results are recorded, never tuned away.

---

## 0. Hypothesis under test (frozen wording)

> **H7 (absence/disconfirmation)**: the frozen organism can learn
> discriminations driven by positively-correlated evidence, but it
> has no local mechanism that requires a neuron to represent
> "feature X is absent" — so a discrimination whose only selective
> structure is the ABSENCE of a feature will fail even when a
> matched-presence discrimination of equal difficulty succeeds.

E7 does NOT test an absence mechanism. It tests whether the
capability is actually missing. Outcome B (below) is the hypothesis-
supporting result and does NOT by itself justify implementing
absence-gating.

## 1. Freeze (verbatim E6 organism)

Complete E6 organism as executed (commits `563bc10`, `56399f2`):
M1–M6, E6 rate balancing (φ EMA, β, both application sites), STDP
pairwise additive + LTD, adaptation, neuron dynamics, thresholds,
structural budgets, window order (E6-φ → M4 → M3 → M2 → M6 →
budget), telemetry v2, snapshots, replay, viz, resource limits,
seeds, RNG consumption order. The E6 run `runs/e6-20260914T204947Z`
is the reference. No mechanism code is touched. `[e6]` remains
enabled in every E7 config. Analyzer: all anima-telemetry metrics
are event-driven and used verbatim (selectivity, assembly,
retention). The single registered analyzer change is the
v3_analysis stage-window parameterization (§6, measurement-only).

## 2. Derivation of the minimal construction (registered reasoning)

The task must isolate "absence" from three confounds:

1. **Identity confound**: if the low-feature category has ANY
   category-exclusive channels, the discrimination is presence-based
   (identify those channels) — no absence needed.
2. **Activity confound**: if "absent" simply means "fewer spikes",
   a drive-threshold/global-rate readout separates the categories
   without any feature structure.
3. **Difficulty scale**: the control must be comparable in shared-
   evidence magnitude and presentation statistics.

**Impossibility result (registered)**: exact total-activity matching
is impossible for a pure single-feature absence discrimination.
The absent feature's spikes must be compensated by SOMETHING; any
category-specific compensation is positive evidence (confound 1),
any shared compensation is common evidence (the subset structure
remains, activity still differs when the feature is missing from one
category). Therefore the absence condition necessarily carries a
registered activity residual; the design removes confound 1 by
construction (subset categories have no exclusive channels) and
isolates confound 2 with a bridge condition (below) that has the
IDENTICAL residual but presence structure.

**Minimum distinguishing pair (frozen)**: one category is a strict
superset of the other — Y = X ∖ P where P is the designated absent
feature group. Y has no channels that X lacks; the only selective
structure is P's absence. With P = 8 channels, X = 16, Y = 8; the
registered residual is: Y delivers exactly half of X's total input
spikes (160 ± 13 vs 80 ± 9 per presentation).

## 3. Conditions (frozen; 24 input channels throughout)

All channels fire independent Poisson @ 20 Hz, ±2 ms jitter,
500 ms, 1500 ms off — identical spike statistics to v2/v3/E6.

### P — positive control (matched activity, presence-based)

| Pattern | Channels | Active ch | Spikes/presentation |
|---|---|---|---|
| X | {0-3} ∪ {8-11} | 8 | 80 ± 9 |
| Y | {0-3} ∪ {12-15} | 8 | 80 ± 9 |

EXACT activity match. Common evidence {0-3} (4 ch) in both;
discriminative evidence = channel identity of the 4-channel groups
{8-11} (P) vs {12-15} (Q). Presence-based, matched difficulty.

### A — absence/disconfirmation TEST

| Pattern | Channels | Active ch | Spikes/presentation |
|---|---|---|---|
| X | {0-15} | 16 | 160 ± 13 |
| Y | {0-7} | 8 | 80 ± 9 |

Y ⊂ X; Y has NO exclusive channels. The only selective structure is
the absence of {8-15}. **Registered residual**: 2:1 total activity
(X:Y). Acceptance bound: separation is accepted as absence-based
ONLY if the L1-normalized attribution check passes (§5); raw
separation carried by the residual alone is recorded as drive-
threshold (A′) and does NOT count as absence learning.

### B — bridge condition (presence-based, IDENTICAL residual)

| Pattern | Channels | Active ch | Spikes/presentation |
|---|---|---|---|
| X | {0-15} | 16 | 160 ± 13 |
| Y | {16-23} | 8 | 80 ± 9 |

Same 2:1 activity residual as A, same X side, but Y owns exclusive
channels {16-23} → presence-based structure available. B isolates
the residual: if a drive-threshold strategy sufficed, A would pass
like B. P passes + B passes + A fails ⇒ the failure is specific to
the absence structure (confound 2 controlled).

**Paired-stream property (registered)**: A and B share pattern ids
("X", "Y"), stage structure, and the canonical seed ⇒ the X-side
channel streams are bit-identical between A and B configs (seed
derivation keyed on pattern_id, rep, channel, schedule index). The
X side is held exactly constant; only Y's structure varies.

## 4. Curriculum and timeline (frozen; per condition)

- S0: 5000 ms silence (unchanged).
- S1: X and Y interleaved (seeded Fisher-Yates rounds), **120 reps
  each** = 240 presentations × 2000 ms = 480,000 ms → [5000,
  485000). Late-S1 = presentations in the second half (event-
  defined, as in v2).
- S3: X and Y interleaved, **15 reps each** = 45,000 ms →
  [485000, 530000) — retention read (event-driven analyzer).
- **No S2/D** (registered): the novelty condition is not part of
  E7's question; the analyzer tolerates its absence (no "novelty"
  fields), and v3_analysis prints a registered "no S2" line.
- Same seeds: canonical 20260912 for P/A/B; cross-seed 9001 and
  424242 for A only (§8).

## 5. Measurements (frozen; existing instruments verbatim)

Per condition (P, A, B), all computed by the SAME code paths as
v2/v3/E6:

- **M1 engagement**: established structural changes (candidate-
  permanence alive at t+10 s) by S1 end; zero ⇒ INCONCLUSIVE (v2
  §11 verbatim). Turnover reported (permanence/prune counts).
- **M2 stability — P2 verbatim**: zero Failure events; S1 internal
  mean rates within the established [20, 250] Hz reading; budget
  invariant every window.
- **M3 separation — P3 bars verbatim (2-pattern case)**: late-S1
  X–Y mean cross-cosine < **0.60** AND median selectivity (S1,
  analyzer definition) > **0.50** ⇒ condition PASS.
- **M4 attribution (registered, pre-hoc)**: additionally compute
  the late-S1 X–Y cosine on **L1-normalized** response vectors
  (each per-presentation vector divided by its own L1 spike sum
  before cosine). For A and B, structural separation REQUIRES
  normalized cosine < 0.60; raw < 0.60 with normalized ≥ 0.60 is
  recorded as drive-threshold (A′) and is not absence learning.
- **M5 reported, no bar**: mean H and per-channel participation at
  the RF snapshot (nearest t = 485,000 for P/A/B — registered E7
  snapshot time, replaces 715,000; the E6 φ readout (recomputed
  replay) as in E6; retention from metrics.json (S3-vs-S1).
  D-condition section: absent (registered).

## 6. Analyzer parameterization (registered, measurement-only)

`v3_analysis` stage-window constants [5000,725000)/S2/S3 are
v2-v3-specific. E7 registers optional trailing CLI args
(`late_s1_lo late_s1_hi s2_lo s2_hi s3_lo s3_hi`) defaulting to
the current constants — absent ⇒ byte-identical outputs for all
existing runs. When a run has no S2 presentations, the D-section
prints a fixed "no S2" line instead of computing a degenerate ratio.
No metric definition changes; no anima-telemetry changes.

## 7. Verdict tree (frozen; every outcome valid)

Preconditions (any failure ⇒ that outcome, ahead of the tree):
Failure event ⇒ REGRESSION (stop, diagnose); 0 established by S1
end ⇒ INCONCLUSIVE; telemetry/determinism/config-hash violation ⇒
INCONCLUSIVE; P2 fail ⇒ D in the affected condition.

| Outcome | Definition | Meaning |
|---|---|---|
| **A** | P PASS ∧ A PASS (with attribution) ∧ B PASS | positive AND absence discriminations both work — absence is NOT missing (hypothesis not supported) |
| **B** | P PASS ∧ B PASS ∧ A FAIL | positive works, absence-specific failure — **absence/disconfirmation hypothesis SUPPORTED** |
| **C** | P FAIL | neither works at matched scale (organism-level limitation) |
| **D** | engagement/activity/determinism/measurement gates fail | inconclusive |
| A′ (report flag) | A raw-PASS, attribution FAIL (normalized ≥ 0.60) | separation is drive-magnitude, not absence structure — recorded under B-reporting with the flag |
| Edge | A PASS ∧ B FAIL | opposite asymmetry — recorded as observation, no pre-registered meaning |

The scientifically important outcome is **B**. B does NOT
automatically justify implementing absence-gating — that decision
is a separate, future, user-approved registration.

## 8. Seeds and run order (frozen)

1. P (canonical 20260912) — positive control.
2. A (canonical 20260912) — absence test.
3. B (canonical 20260912) — bridge (residual control).
4. Cross-seed A (9001, 424242) — **gated**: only if A's M2 (P2)
   and M1 engagement pass. Reduced P4 endpoints: A-condition
   raw/normalized verdict reproducibility across seeds AND
   |Δ mean H| < 0.20 (canonical vs each seed) AND |Δ established|/
   established < 0.50.

Concurrent runs at nice 10. No sweeps, no extra arms, no
reordering.

## 9. Configs (frozen at creation; hashes recorded in appendix)

`configs/e7-pos.toml` (exp_id e7-pos), `configs/e7-abs.toml`
(e7-abs), `configs/e7-bridge.toml` (e7-bridge),
`configs/e7-abs-seed9001.toml`, `configs/e7-abs-seed424242.toml`.
Each = `e6-full.toml` with patterns/stage replaced by §3/§4
(channel_ids form), [e6] enabled, organism/plasticity/structural/
resources/v2 sections byte-identical (enforced by test).

## 10. Tests (registered, before any run)

1. Config conformance: channel sets, stage structure, reps, seeds,
   E6-on, organism sections == e6-full (existing comparison
   idiom).
2. Paired-stream identity: at seed 20260912, A's X presentations
   are bit-identical to B's X presentations (environment-level).
3. P-condition E6 regime (AMENDMENT A-1, 2026-09-16, user-approved;
   protocol §3 has shared common evidence {0-3} firing in BOTH
   categories, so the original "every channel fires in exactly one
   pattern ⇒ β ≡ 1" premise was internally inconsistent): P has an
   ACTIVE E6 — φ_common = 2 × φ_exclusive by design (deterministic),
   β_common < 1 < β_exclusive with both in [0.5, 2] (neuron-local
   φ̄). Test asserts: (a) per-channel event totals follow the exact
   2:1 common:exclusive structure, (b) mechanism β for the common
   channels is < 1, for exclusive channels > 1, both within
   [0.5, 2]. P remains E6-ON like every E7 condition; its scientific
   role (matched-activity presence control) is unchanged.
4. Deterministic repeated execution (existing integration idiom,
   e7-abs config).

## 11. Observability and reproducibility (unchanged)

Live viz, replayable telemetry, structural events, resource usage,
RF stats, deterministic seeds, commit + config hash recording
(RunStarted), reproducible run dirs. Any observability failure
stops the run and is diagnosed; no silent repair.

---

## Appendix (filled at execution time)

- Config hashes (sha256 of raw file, recorded at creation before any
  run): e7-pos `e4385792fc999cb04abaf3a760183d33b8b73a6e96bb9a9fc85d208d0a1b2eba`;
  e7-abs `6ac461de82d084b577778a4e8fc422308de28bea28fccdbc10562fd2e1aa0107`;
  e7-bridge `bf3d3c2652eaafe55e2351f3176e4ab3f293c98e1c24dec13cee02752a135742`;
  e7-abs-seed9001 `bf831485214099436f33ae8f093df91e069eb1d070a16ab2894fc2fe6c834a66`;
  e7-abs-seed424242 `17db26e9be8ba23f4c07810302089f25568194ca8a396e2890fae1ea5d4fe3b8`.
- Amendment A-1 (2026-09-16, user-approved): §10.3 P-condition E6
  property — P has ACTIVE E6 (common {0-3} at 2:1 duty), β_common < 1
  < β_exclusive in [0.5, 2]; test asserts the 2:1 structure and the
  mechanism β range.
- Commit (implementation): `(recorded at run time)`.