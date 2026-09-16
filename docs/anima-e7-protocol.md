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
  run; A-2 revision, 2026-09-16): e7-pos `04b6dfd50c3c2b8397edad31fa39b51989891cef1e500f64be7dab109e8e3799`;
  e7-abs `df13806966e88175ef38af4708d76a3aadec954f3e15cb4cbe95b9a583554a1a`;
  e7-bridge `069b4a845555837f68d40c1543caa8ad2b13661876d3421cf37c13920add1143`;
  e7-abs-seed9001 `89d68483fbdcfbbdb9a25f2300ddaa6cc7bf83fb2b7e023ca7388463e7137a88`;
  e7-abs-seed424242 `81152337f80648c8298031bf784a1378606d044a118d30432815025e3488fe88`.
- Amendment A-1 (2026-09-16, user-approved): §10.3 P-condition E6
  property — P has ACTIVE E6 (common {0-3} at 2:1 duty), β_common < 1
  < β_exclusive in [0.5, 2]; test asserts the 2:1 structure and the
  mechanism β range.
- Amendment A-2 (2026-09-16, user-approved): E7 pattern ids X→A, Y→B
  in all five configs. Reason: the frozen anima-telemetry analyzer
  hard-codes selectivity/retention to pattern ids A/B/C, so X/Y-
  labeled curricula yield EMPTY selectivity (0 entries) — the M3
  endpoint would be undefined. Ids are labels + seed-derivation
  inputs only; condition content (channel sets, activity, residuals,
  paired streams) unchanged; streams re-derived; hashes above
  supersede the X/Y versions; the pre-A-2 X/Y runs are superseded
  and rerun.
- Commit (implementation): `(recorded at run time)`.
---

## E7 execution record (2026-09-16)

Implementation `5254369`; amendments A-1 (P active-E6 property) and
A-2 (pattern ids A/B) committed `49fc8ef`; 104 tests green, 0
warnings. Run dirs: P `runs/e7-pos-20260916T083635Z`, A
`runs/e7-abs-20260916T083635Z`, B `runs/e7-bridge-20260916T083635Z`,
A-9001/A-424242 `runs/e7-abs-{9001,424242}-20260916T084235Z` — all
curriculum-complete, telemetry intact, zero failures. Analyzer:
v3_analysis with the registered E7 window args
(245000 485000 0 0 485000 530000 485000); cross_cosine and
e7_attribution verbatim.

### Per-condition results (frozen M1–M3 + attribution)

| | P (presence) | A (absence) | B (bridge) |
|---|---|---|---|
| established by S1 end | 3,074 ✓ | 7,046 ✓ | 5,365 ✓ |
| P2 (failures; rates) | 0; 9.7/81.4 Hz ✓ | 0; 6.9/70.8 Hz ✓ | 0; 11.8/103.3 Hz ✓ |
| late-S1 cross A-B | **0.033** ✓ | **0.413** ✓ | **0.000** ✓ |
| L1-normalized cross | 0.033 ✓ | **0.413** ✓ | 0.000 ✓ |
| within-pattern mean | 0.950 | 0.544 | 0.952 |
| selectivity median (S1) | 0.987 ✓ | **0.972** ✓ | 1.000 ✓ |
| **condition verdict** | **PASS** | **PASS** | **PASS** |

P's E6-regime as amended (A-1): community channels at 2:1 duty,
β_common < 1 < β_exclusive — the active regime, verified in tests.

### Cross-seed A (§8.4): **SUPPORTED**

| seed | mean H | established | cross | L1-norm |
|---|---|---|---|---|
| 20260912 | 2.513 | 7,046 | 0.413 | 0.413 |
| 9001 | 2.580 | 6,904 | 0.331 | 0.309 |
| 424242 | 2.492 | 7,130 | 0.520 | 0.520 |

|ΔH| ≤ 0.067 < 0.20; |Δestablished|/est ≤ 0.02 < 0.50; the A
separation verdict (raw AND normalized < 0.60) reproduces in all
three seeds.

### Frozen verdict: **OUTCOME A** (§7 tree)

P PASS ∧ A PASS (with attribution) ∧ B PASS ⇒ positive AND absence
discriminations both work ⇒ the absence/disconfirmation hypothesis
as registered is **NOT supported at the minimal scale**.

Interpretation (registered; no over-claim):

1. The frozen E6 organism demonstrably separates subset-from-
   superset categories whose only selective structure is the
   absence of channels {8-15}: cross 0.413 with a clean L1
   attribution (structure, not drive), selectivity 0.972, stable,
   cross-seed reproducible.
2. A is markedly noisier than the presence controls (0.413 vs
   0.033/0.000; within-pattern 0.544 vs 0.950) — absence-structured
   discrimination works, but via a coarser complement readout.
3. Scope boundary (intentional): a 2-category presence-vs-absence
   task is solvable by a P-presence detector plus default
   classification. E7 therefore establishes "absence-structured
   discrimination succeeds at the 2-way level"; it does NOT test
   the 3-way overlapping ambiguity where E6's outcome C arose
   (B ⊆ A∪C, disambiguation requires absence-veto/context, not
   complement classification). E7's outcome A weakens the strong
   form of H7 without contradicting E6's outcome C.
4. No mechanism is implied or implemented; absence-gating remains
   an open question for any future user-approved experiment.
