# ANIMA E11 — Pre-registered Protocol (frozen before implementation)

**Status: PRE-REGISTERED.** Written before any E11 code, config, or
run. E11 is the **minimal distinguishing experiment** for the E10
result: the leading-side lock-in (A-B 0.697 / 0.760 / 0.624; B-C
0.081 / 0.080 / 0.080; B-independence 0/3) was observed under a
**persistently asymmetric** temporal curriculum (all 120 B
presentations SEQ: {4-7} → {8-11}). E11 counterbalances the order —
exactly 60 SEQ-B and 60 REV-B presentations — with everything else
frozen from E10. Question: does the lock-in depend on the persistent
order asymmetry of experience, or is it intrinsic to the frozen
learning dynamics? No mechanism is changed, proposed, or implied.
Frozen values change ONLY via a logged, user-approved amendment.

---

## 0. Primary question (frozen wording)

> **H11: When neither shared group (4-7 vs 8-11) systematically
> occupies the leading temporal position over the full
> experiment — 60 SEQ-B and 60 REV-B presentations, balanced per
> channel, marginal exposure identical to E10 — does B become
> independently represented (A-B < 0.60 AND B-C < 0.60)?**

Outcome A shows the lock-in is attributable to persistent
temporal-order asymmetry in **experience**; it does NOT establish a
mechanism. Outcome B shows an intrinsic asymmetry or another
interaction in the frozen dynamics remains implicated — no
mechanism is invented either way.

## 1. Freeze (verbatim E10 creature + curriculum)

Complete E10 configuration (`configs/e10.toml`, commit `6700090`):
organism byte-identical to E6; E6 rate balancing enabled; A {0-7}
and C {8-15} static 20 Hz × 500 ms; B phase contents/rates/
durations exactly unchanged (40 Hz, 250 ms phases, ±2 ms jitter);
S0/S1/S3 with **120 S1 repetitions**; no S2/D; timeline
[5000, 723500) presentations / nominal S1 window 725000, S3
[725000, 815000), snapshot 725,000, analyzer args
`365000 725000 0 0 725000 815000 725000`; seeds 20260912 / 9001 /
424242; telemetry, replay, analyzer. **No third repetition point;
no mechanism, parameter, threshold, plasticity, adaptation, or
balancing-rule changes.**

## 2. The ONLY experimental change (frozen)

B's within-presentation temporal order is **counterbalanced**:

- **SEQ-B** = {4-7} @ 40 Hz × [0,250) → {8-11} @ 40 Hz × [250,500)
  (identical to E10's B).
- **REV-B** = {8-11} @ 40 Hz × [0,250) → {4-7} @ 40 Hz × [250,500).
- Exactly **60 SEQ-B and 60 REV-B** across the 120 S1 B
  presentations.

### Schedule decision D1 (frozen): deterministic alternating variant

**Variant = B-repetition-index parity: rep % 2 == 0 ⇒ SEQ-B;
rep % 2 == 1 ⇒ REV-B.** The rep index is the existing per-pattern
counter (0-based, spanning S1 then S3 — S3's 15 B presentations
follow the same parity rule: 8 SEQ / 7 REV; S3 is retention-only,
no bar). The interleaved round order is the unchanged E10 seeded
Fisher-Yates (identical shuffle: same seed, same stage structure
⇒ the A/C presentation sequence is bit-identical to E10). The
variant selection consumes NO RNG and adds no random variable.

### Schedule decision D2 (frozen): pairing audit (computed BEFORE
### freeze, from the E10 schedules — these numbers are in the
### protocol):

| seed | B position in round (SEQ) | (REV) | preceding pattern (SEQ) | (REV) |
|---|---|---|---|---|
| 20260912 | [21, 17, 22] | [19, 19, 22] | A=31, C=29 | A=31, C=29 |
| 9001 | [22, 14, 24] | [24, 16, 20] | A=35, C=25 | A=32, C=28 |
| 424242 | [22, 20, 18] | [24, 17, 19] | A=33, C=27 | A=31, C=29 |

No systematic position or preceding-pattern bias by variant (all
splits ≈ 20 per position; preceding-pattern differences ≤ 4/60);
B never directly follows B (rounds of three). Registered residual:
per-seed asymmetries ≤ 4/60 in preceding-pattern counts —
reported, not corrected (a covariate, not a confounder).

## 3. Pre-data equivalence proofs (frozen; each asserted by test)

1. **Total B channel exposure identical to E10**: 120 S1 B
   presentations × 10 expected spikes per channel = same totals
   (SEQ and REV each deliver 10 ± √10 per channel per
   presentation; variant mix cancels exactly at the marginal
   level). Totals 80 ± 9 per presentation.
2. **Each B channel marginal exposure identical**: every B channel
   fires exactly one 250 ms @ 40 Hz phase per presentation in
   both variants (lead or trail) ⇒ 10 ± √10 per presentation per
   channel, identical to E10.
3. **No systematic leading/trailing advantage**: exactly 60
   leading presentations per group (SEQ: {4-7} leads; REV: {8-11}
   leads) ⇒ the leading-position count is perfectly balanced by
   construction; the D2 audit covers higher-order schedule
   pairings.
4. **No new mechanism**: variant selection is environment-layer
   only (`phase_variants`), consumes no RNG, and the organism
   sees only spike trains. Freeze tests assert organism sections
   byte-identical to e6-full and the E11 config differing from
   e10.toml ONLY in B's phase structure and [run] identity.
5. **E10 isolation**: A and C streams are bit-identical between
   E10 and E11 (same pattern ids/reps/slots/seeds); B's
   per-channel marginal identical; the ONLY difference is the
   leading-order mix (all-SEQ vs balanced). E10 run dirs are the
   untouchable baseline — never rerun.

**E6 φ/β audit (registered):** per-channel window-count marginals
are identically distributed between E10 and E11 (each B channel
fires 40 Hz × 250 ms per presentation in both) ⇒ φ and β match
E10's within the registered drift band (|φ_E11 − φ_E10|/φ_E10 <
0.03, per channel; actuals reported). A/C φ identical by
construction.

## 4. Endpoints (frozen; E9/E10 definitions verbatim)

- M1 engagement: established ≥ 1 by S1 end (v2 §11 floor).
- M2 P2: 0 failures; S1 rates in the [20, 250] Hz reading; budget
  invariant.
- M3 pairwise late-S1 cosines A-B, B-C, A-C (< 0.60 bar) with the
  L1 attribution per pair.
- **Primary B-independence (categorical)**: A-B < 0.60 AND B-C <
  0.60, per seed, L1-attributed.
- B-alignment: argmin(cosine(B,A), cosine(B,C)).
- M5 reported: selectivity median, mean H + participation,
  retention, E6 φ readout.

## 5. Verdict tree (frozen)

Preconditions: Failure ⇒ REGRESSION; 0 established ⇒ INCONCLUSIVE;
P2/telemetry/determinism failure ⇒ D.

| Outcome | Definition | Interpretation |
|---|---|---|
| **A** | B-independent in **all three seeds** (categorical reproduction) | the E10 leading-side lock-in is dependent on persistent temporal-order asymmetry in experience — curriculum-level, NOT a mechanism discovery |
| **B** | B remains absorbed in ≥ 1 seed (A-B ≥ 0.60 or B-C ≥ 0.60) | persistent order asymmetry alone does not explain the lock-in; an intrinsic asymmetry/other interaction in the frozen dynamics remains implicated — no mechanism invented |
| **C** | generic collapse: A-C ≥ 0.60 or instability | interpreted separately; NOT evidence for or against the leading-order hypothesis |
| **D** | gates | INCONCLUSIVE |

Cross-seed: the categorical B-independence verdict must reproduce
across seeds for a strong result (the N/3 split is reported exactly
either way, per E10's rule: 3/3 strong, 2/3 or 1/3 stochastic, 0/3
no evidence). Exact pairwise cosines and B-alignment reported for
every seed.

## 6. Implementation scope (frozen, after release)

Environment layer only: optional `phase_variants: Vec<Vec<PhaseSpec>>`
on PatternSpec; when present, the pattern's phases for B
presentation with rep index r are `variants[r % variants.len()]`
(registered semantics; consumed for the E11 two-variant case).
Mutually exclusive with `phases`/`channels`/`channel_ids`; every
variant must tile [0, duration_ms) exactly (validation). Absent ⇒
byte-identical behavior for every existing config. Seed derivation
unchanged (phase index inside the variant's stream tuple as in E9,
per presentation-slot and phase index; variant selection adds no
seed material). No anima-core / anima-telemetry / analyzer changes.

Configs: `e11.toml` (e11, 20260912), `e11-seed9001.toml`
(e11-seed9001, 9001), `e11-seed424242.toml` (e11-seed424242,
424242) — from e10.toml; B's `phases` replaced by
`phase_variants` = [SEQ, REV]; nothing else differs.

## 7. Tests (registered, before data)

1. Freeze: e11 == e10 except B phase structure + [run] identity;
   organism sections == e6-full; E6 on.
2. Variant semantics: presentations with even B-rep index produce
   SEQ phase maps; odd → REV (verified from trains: leading group
   matches parity); S1 count 60 SEQ / 60 REV; S3 8/7.
3. Marginal exposure: per-channel 10 ± √10 per presentation in
   both variants; totals 80 ± 9; within-φ audit band vs E10 < 3%.
4. A/C streams bit-identical to E10 (same seed).
5. M3 co-activity 1/5 windows per B presentation (both variants).
6. Determinism: repeated generation + short-run telemetry
   identity (e11 layout).
7. Timeline/analyzer args unchanged from E10.

## 8. Experiment order (frozen)

1. Freeze (this document). 2. Implement + tests. 3. Run 20260912.
4. Gate (M2 + M1) ⇒ 9001. 5. Gate ⇒ 424242. 6. Pairwise tables +
   B-alignment + L1 + scale comparison vs E10. 7. Verdict +
   interpretation + registry.

No post-hoc choice of schedule, no endpoint-threshold changes, no
reruns "because a seed is inconvenient".

## 9. Interpretation boundaries (frozen)

- Outcome A: "the lock-in is sensitive to temporal-order balance
  in experience" — nothing about how the dynamics implement it.
- Outcome B: "order asymmetry is not sufficient; an intrinsic
  asymmetry or another interaction remains implicated" — no
  mechanism invented, no organism change.
- E10 is the single baseline; differences are attributeable to the
  counterbalancing variable by the §3 proofs.
- If any ambiguity had existed in the schedule construction or
  statistical interpretation, it was resolved BEFORE data and
  recorded here (D1/D2); post-data reinterpretation is
  prohibited.

---

## Appendix (filled at execution time)

- Config hashes: e11 `…`; e11-seed9001 `…`; e11-seed424242 `…`
  (recorded at creation, before runs).
- E10 baseline (verbatim): 20260912 (0.697/0.081/0.003), 9001
  (0.760/0.080/0.000), 424242 (0.624/0.080/0.000); B-independence
  F / F / F; selectivity 0.662 / 0.802 / 0.902.
- Commit: `…` (recorded at run time).