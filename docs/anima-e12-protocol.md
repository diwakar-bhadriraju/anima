# ANIMA E12 — Pre-registered Protocol (frozen before implementation)

**Status: PRE-REGISTERED.** Written before any E12 code, config, or
run. E12 is a **path-dependence / recovery probe**: after
leading-side lock-in is established by asymmetric experience (60
SEQ-B presentations), does subsequent contradictory temporal
experience (60 REV-B presentations) reorganize B toward
independence? Curriculum/history experiment only — the frozen E6
organism and all mechanisms are untouched. The existing experiments
are the baselines (E9 = 60-rep all-SEQ end-state reference for the
midpoint; E10 = persistent-asymmetry baseline; E11 = balanced-
order baseline). E10/E11 are NEVER rerun. Frozen values change ONLY
via a logged, user-approved amendment.

---

## 0. Question and framing (frozen wording)

> **H12: If leading-side lock-in is first established by asymmetric
> experience (60 SEQ-B), can equal subsequent contradictory
> experience (60 REV-B) reorganize B's representation toward
> independence?**

- E10 (120 SEQ) tests formation under persistent asymmetry → lock-in.
- E11 (60 SEQ / 60 REV interleaved) tests learning under balanced
  order → independence (3/3).
- **E12 (60 SEQ / 60 REV BLOCKED: SEQ first, REV second) tests
  recovery/reorganization after asymmetric formation.**

The critical quantity: does E12 move from the E9/E10-like state
(measured at the midpoint T0) toward the E11-like state (measured
at the final T1) during the REV block?

## 1. Freeze (verbatim E10/E11 creature + conditions)

Frozen E6 organism (byte-identical organism/plasticity/structural/
resources/v2/e6 sections); E6 rate balancing enabled; A {0-7} and
C {8-15} static 20 Hz × 500 ms; B phases {4-7} @ 40 Hz × [0,250) →
{8-11} @ 40 Hz × [250,500) (variant 0) and the REV mirror (variant
1); same marginal exposure; **same 120 total B presentations in
S1**; same S1 timeline/presentation structure (500/1500 ms cadence,
seeded Fisher-Yates rounds); same M3 windows (100 ms); same
analyzer/telemetry; no S2/D; seeds 20260912 / 9001 / 424242;
analyzer args `365000 725000 0 0 725000 815000 725000` for the
final endpoints. No mechanism, parameter, phase-duration, rate, M3,
repetition-count, or schedule-shuffle changes.

## 2. The ONLY curriculum change (frozen): blocked temporal history

- **BLOCK 1: B presentations 1–60 = SEQ-B** ({4-7} → {8-11}).
- **BLOCK 2: B presentations 61–120 = REV-B** ({8-11} → {4-7}).

Implemented as a registered environment-layer generalization of the
E11 variant-selection rule:

> `variant_index(rep) = (rep / variant_block) % variants.len()`,
> where `variant_block` defaults to **1** (⇒ the E11 rule `rep % 2`
> exactly — byte-identical for E11 and all existing configs), and
> E12 registers `variant_block = 60` (⇒ reps 0–59 = variant 0 (SEQ),
> reps 60–119 = variant 1 (REV), S3 reps 120–134 = (120/60)%2 = 0
> ⇒ all-SEQ retention; S3 is retention-only, no bar).

Deterministic, consumes no RNG, environment-only. The variant-
selection change is the ONLY implementation difference vs E11
besides identity fields.

### The 60th → 61st boundary (registered)

B presentation rep 59 (the 60th SEQ-B) ends during the 180th S1
presentation. **T0 = 365,000 sim-ms** = the start of the 181st S1
presentation (after exactly 180 presentations = 60 round-trips,
deterministic and seed-independent — S1 slots are a fixed 2000 ms
grid). B rep 60 (the first REV-B) occurs in round 61, whose exact
position within the round is the unchanged seed-dependent shuffle
(reported per seed at implementation time).

## 3. Pre-data proofs (frozen; each asserted by test)

1. **Total B exposure identical to E10/E11**: 120 S1 B
   presentations × 10 expected spikes per channel; totals
   80 ± 9 per presentation.
2. **Per-channel marginal exposure identical**: each B channel
   fires exactly one 250 ms @ 40 Hz phase per presentation
   (variant-independent) ⇒ 10 ± √10 per presentation per channel.
3. **Each phase order exactly 60 times**: reps 0–59 SEQ (4-7
   leads), reps 60–119 REV (8-11 leads) — test-enforced from the
   schedule.
4. **No organism/mechanism/parameter changes**: freeze test vs
   e6-full sections; E12 config == e10 config except B's variant
   structure, `variant_block`, and [run] identity.
5. **The only intervention is temporal history** (SEQ-block then
   REV-block): A/C streams bit-identical to E10 (same seeds,
   slots); B marginal identical; the sole difference is the
   intra-B order sequence across presentations.
6. **Boundary registered** (above).
7. **φ/β window-placement differences documented pre-data**
   (A-5-class): within BLOCK 1 the per-channel window profiles are
   E10-like (SEQ placement); within BLOCK 2 they are the REV
   mirror (windows 2–4 bursts). The full-run φ vs E10 therefore
   sits between; registered audit band for B channels ±0.15 vs
   E10 (the E11 A-5 band), actuals reported; A/C 0% by
   construction; per-block placement documented, not corrected.
8. **No post-hoc selection** of block size (60, origami of the
   design), schedule pairing (blocked-first, registered), or
   endpoints. No reruns "because a seed is inconvenient".

## 4. Midpoint observation (frozen; measurement only, registered)

A NEW measurement-only example `e12_transition.rs` computes the
same representation metrics at BOTH observation points from the
recorded telemetry (no mechanism, no new telemetry):

| Point | Window | Meaning |
|---|---|---|
| **T0 (midpoint)** | S1 presentations in [5000, 365000) | end of the 60 SEQ-B block |
| **T1 (final)** | S1 presentations in [365000, 723500) | end of the 60 REV-B block |

Per point, with the SAME definitions as the existing instruments:
- A-B, B-C, A-C mean cross-cosines (pairwise-mean, cross_cosine
  method) + the L1-normalized attribution pair;
- B-independence booleans (A-B < 0.60 AND B-C < 0.60) — L1-
  attributed;
- B-alignment = argmin(B-A, B-C);
- windowed selectivity (the analyzer's per-neuron
  (best − 2nd)/best formula over that window's presentations,
  median);
- structural engagement: candidate-permanence event count in the
  window (created in-window);
- P2 window status: failures in the window (events), internal
  rate stats from snapshots in the window (mean/max), budget
  invariant (asserted during the run — no per-window budget
  telemetry exists; the run-wide invariant holds by construction,
  reported once).

The final-vs-midpoint transition is reported as the movement of
A-B and B-C (especially whether B moves away from its original
leading-side association during the REV block: the paired
difference of B-alignment and of the A-B/B-C cosines, T0 → T1).

## 5. Endpoints (frozen; E9/E10/E11 definitions verbatim)

- **Primary (final) B-independence**: A-B < 0.60 AND B-C < 0.60 at
  T1, per seed, L1-attributed.
- M1 engagement at T1: established ≥ 1 by S1 end (v2 §11 floor).
- M2 P2 at T1 verbatim.
- M3 pairwise + L1 at T1.
- Transition readout (§4): T0 vs T1 per seed, exact values.
- M5 reported: selectivity (T0/T1), mean H + participation
  (snapshot 725,000), retention, E6 φ readout.

## 6. Verdict tree (frozen; every outcome valid)

Preconditions: Failure ⇒ REGRESSION; 0 established by S1 end ⇒
INCONCLUSIVE; P2/telemetry/determinism ⇒ D. Any gate failure ⇒ D
(never interpreted as path-dependence evidence).

| Outcome | Definition | Interpretation |
|---|---|---|
| **A — RECOVERY** | B-independent at T1 in **all three seeds** | contradictory temporal experience reorganizes the previously established leading-side association — recoverability/path-dependent plasticity at the tested scale; does NOT establish a mechanism |
| **B — PERSISTENT LOCK-IN** | B still associated with the original leading side at T1 in **all three seeds** (T1 B-alignment unchanged AND A-B/B-C verdict unchanged, per seed) | the initial asymmetric experience produces a representation not readily reversed by equal subsequent contradictory experience at this scale; NOT called irreversible/permanent — the test covers only 60 REV presentations |
| **C — REORGANIZATION WITHOUT FULL INDEPENDENCE** | B moves substantially away from the E10-leading-side state at T1 (e.g., the originally-high pair drops by ≥ 0.10) but B-independence fails | partial recovery/reorganization — preserved as a DISTINCT outcome, requiring a separate follow-up registration, never forced into A or B |
| **D — GATES** | any precondition fails | inconclusive |

**Per-seed discipline**: verdicts are evaluated per seed and
reported exactly; if seeds disagree in categorical outcome, the
registered modal rule applies (the outcome class held by ≥ 2
seeds, with the dissenting seed's exact values reported) — a
disagreement itself is reported as seed-dependence, never
averaged away. The same categorical interpretation across all
three seeds is required for the strongest claim.

## 7. Comparison frame (frozen; not a ranking)

- E10 = persistent asymmetry → lock-in (end-state; never rerun).
- E11 = balanced exposure from the start → independence (3/3;
  never rerun).
- E12 = asymmetry first → contradictory experience afterward.
- **T0 reference for the midpoint: E9's all-SEQ 60-rep end-state**
  (the learning-curve state at 60 SEQ presentations — the release's
  "E10-like state" is approximate; the correct reference is E9:
  canonical 0.630/0.083/0.000, sel 0.702). T1 reference: E11's
  independence (0.462/0.488/0.000 canonical).
- The scientific claim is the T0 → T1 movement (toward or away
  from the balanced-order outcome), not a three-way ranking.

## 8. Implementation scope (frozen, after release)

- Environment layer only: `variant_block: u64` on PatternSpec
  (serde default 1 ⇒ the E11 `rep % len` rule byte-identically);
  selection `variants[(rep / variant_block) % len]`; validation:
  variant_block ≥ 1.
- Configs `e12.toml` (e12, 20260912), `e12-seed9001.toml`
  (e12-seed9001, 9001), `e12-seed424242.toml` (e12-seed424242,
  424242) — from e10.toml: B gets phase_variants [SEQ, REV] +
  `variant_block = 60`; nothing else differs.
- `e12_transition.rs` (measurement-only; §4 definitions). No
  anima-core/anima-telemetry/analyzer-definition changes.

## 9. Tests (registered, before data)

1. Freeze: e12 == e10 except B variant structure + variant_block +
   [run] identity; organism sections == e6-full; E6 on.
2. Blocked selection: reps 0–59 SEQ leading group (4-7), reps
   60–119 REV (8-11); 60/60 counts; boundary rep 60 verified;
   S3 all-SEQ per the rule (retention-only).
3. Equivalence: A/C streams bit-identical to e10 (all seeds);
   per-channel B marginals 10 ± √10; totals 80 ± 9; M3 co-activity
   1/5 windows per B presentation (both variants).
4. φ/β audit vs e10: B channels within ±0.15 (A-5 band), A/C <
   0.01; actuals reported.
5. e12_transition windowing: T0 = 365,000 boundary; window
   definitions; determinism on a short-run integration test.
6. Timeline unchanged (813,500 ms actual); analyzer args sane.
7. Existing suites green (variant_block default preserves E11
   behavior byte-identically: E11 tests rerun unchanged).

## 10. Experiment order (frozen)

1. Freeze (this document). 2. Implement configs + tests. 3. Run
   20260912. 4. Gate (M2 + M1) ⇒ 9001. 5. Gate ⇒ 424242.
6. e12_transition (T0/T1) + final endpoints + P2 + engagement.
7. Per-seed verdicts + transition report + comparison vs
   E9/E10/E11. 8. Verdict + interpretation + registry.

No post-hoc selection of block size, schedule, or endpoints; no
reruns.

## 11. Scientific discipline (frozen)

- Outcome C is preserved as a distinct result, never collapsed
  into A or B.
- Outcome B is NOT called irreversible — only "not readily
  reversed by 60 subsequent REV presentations at this scale".
- Outcome A establishes path-dependent recoverability at the
  tested scale — NOT a mechanism.
- No E13 is proposed until E12 is fully analyzed and committed.

---

## Appendix (filled at execution time)

- Config hashes: e12 `…`; e12-seed9001 `…`; e12-seed424242 `…`
  (recorded at creation, before runs).
- Baseline references (verbatim): E9 canonical (0.630/0.083/0.000);
  E10 (0.697/0.081/0.003, 0.760/0.080/0.000, 0.624/0.080/0.000,
  F/F/F); E11 (0.462/0.488/0.000, 0.535/0.511/0.000,
  0.398/0.529/0.232, T/T/T). B-alignment: E9 A; E10 A/C/A; E11
  A/C/A.
- Commit: `…` (recorded at run time).