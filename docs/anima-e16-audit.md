# ANIMA E16 — Causal decomposition audit (protocol-design stage)

Status: AUDIT (2026-09-18). Not a frozen protocol; no simulation, no
reruns, no organism/parameter/protocol change, no new mechanisms, no
thresholds, no causal claims. Basis: committed E12 artifacts read-only
+ committed E15 analysis + read-only probe `e16_activity.rs`
(telemetry-only imports, deterministic, no anima_core).

## 1. REV1-window process-activity audit (per seed)

Interval (REV1-pre, REV1-post] = (364000, 366000] (20260912) /
(368000, 370000] (9001, 424242); REV1-B presentation windows
[365000, 365500) / [369000, 369500).

| Process | 20260912 | 9001 | 424242 | Status |
|---|---|---|---|---|
| STDP (emitted, |dW|>0.01/tick) | 365 up / 287 down, net +8.18-9.46 = -1.28; 29+42 distinct ids (8.6% of 825) | 412/383, net +10.58-13.48 = -2.90; 34+68 of 889 (11.5%) | 297/181, net +6.00-5.48 = +0.52; 15+36 of 854 (6.0%) | **OBSERVED ACTIVE** (all seeds; depression-dominated in 2/3) |
| E6 rate-balancing beta | no direct trace | no direct trace | no direct trace | **ACTIVITY NOT RESOLVABLE** (effects embedded in STDP magnitudes + M3 accumulation; phi/beta never stored) |
| M2 normalization | per-neuron exc sum invariant: mean\|sum-0.8\| <= 7e-6, max <= 1.2e-5 at every instant | same (<=7e-6 / <=1.2e-5) | same (<=6e-6 / <=1.0e-5) | **OBSERVED ACTIVE** (post-condition holds at every window boundary measured; 52 exc-bearing neurons) |
| M3 candidate pass | 6 maturations (all during) + silent accumulation/decay | 7 (all during) + silent | 1 (during) + silent | **OBSERVED ACTIVE** (maturation boundary); accumulation kinetics (candidate weights, deaths, redraws) NOT RESOLVABLE |
| M4 pruning | 8 competitive-prunes (3 before, 3 during, 2 after) | 57 (53 before, 2 during, 2 after) | 22 (21 before, 1 after) | **OBSERVED ACTIVE** (all seeds; 9001 wave precedes the window) |
| M5 budget eviction | 0 evictions; occupancy <= 10/40 exc (B_e), 10/10 inh (B_i) | 0; <= 12/40 | 0; <= 10/40 | **OBSERVED INACTIVE** (capacity not binding; 0 budget-eviction events) |
| M6 inhibitory | 436/514 inhibitory synapses endpoint-changed, net dW -1.27 | 402/517, net -1.07 | 368/516, net -1.99 | **OBSERVED ACTIVE** (endpoint; 71-85% reweighted, net negative in all seeds); intra-interval steps NOT RESOLVABLE |
| Network dynamics / adaptation | 756 internal spikes in window (q1 293 q2 208 q3 255 q4 0) | 1095 (346/493/256/0) | 574 (201/236/137/0) | **OBSERVED ACTIVE** (dense spiking during the presentation, all seeds); adaptation's isolated contribution NOT RESOLVABLE |

## 2. Cross-seed common active mechanisms

STDP (emitted), M2 (invariant pinned at every instant), M3
(maturations during the presentation window in every seed), M4
(competitive pruning; timing differs), M6 (broad inhibitory weight
movement), network dynamics. Common directionalities: inhibitory net
dW negative everywhere; strengthened/weakened events never touch the
created or pruned synapses (0 intersections, all seeds); 10-28 ids
saw both potentiation and depression within the interval.

## 3. Demonstrably inactive during REV1

- M5 budget eviction: 0 events in any seed; live occupancy 9-12 of
  B_e=40 (and B_i fully subscribed at 10/10) — eviction cannot fire.
- v1 silence-prune path: no prune events with any other reason.

## 4. Activity not resolvable from telemetry

- E6 beta (no stored phi/beta trajectory; effects embedded in event
  magnitudes).
- M3 accumulation kinetics (candidate births/deaths/redraws/weights —
  established in the E15 audit).
- Intra-interval timing of M2 rescaling, M6 updates, passive decay,
  |dW| <= 0.01 STDP (endpoint deltas exact; silent component only as
  aggregate residuals -0.044 / -0.018 / -0.020).
- Adaptation's isolated contribution to the spiking pattern.

## 5. Observed processes -> E15 structural observations

- E15 endpoint weight movement (89.6/80.8/80.0% of alive-both): a
  COMPOSITE — emitted STDP on 6-12% of synapses (net negative in
  20260912/9001, near-zero in 424242), silent STDP/decay everywhere,
  M2 rescaling (per-neuron sums pinned at t_e=0.8 every window —
  weight redistribution without total change), M6 on 71-85% of
  inhibitory synapses (net negative). Only the STDP-emitted and
  topology components carry timestamps (D4).
- Lower-B (4-7) endpoint gains (+0.786/+0.498/+0.170): M3 creations
  (w=0.02 each, 6/7/1, all during the presentation) contribute
  +0.12/+0.14/+0.02 of it; the remainder is weight movement on
  existing lower-B afferents (STDP-emitted + silent + M2 re-share).
- Prunes (8/57/22): all competitive-prune; in the pos-2 seeds the
  wave precedes the presentation (t=368100-368400); pruned ids were
  never STDP-touched in the interval.
- The behavioral flip (E14 k*=1) co-occurs with: dense spiking,
  maturations inside the window, and window-boundary-wide
  renormalization — no causality inferred.

## 6. Causal candidate set (criteria 1-3 applied)

Criterion gate: (1) demonstrably operates during REV1; (2) plausible
path to the T0->REV1 change under the frozen architecture; (3)
disabling distinguishes >= 2 plausible explanations.

| Mechanism | (1) | (2) | (3) | Verdict |
|---|---|---|---|---|
| **M2 normalization** | YES (invariant at every instant) | weight redistribution across afferents on a 100 ms window cadence = a fast re-expression engine | distinguishes normalization-re-expression (B) vs raw weight dynamics | **PRIMARY** |
| **STDP** | YES (emitted events, all seeds) | direct per-spike weight change on existing structure | distinguishes Hebbian weight dynamics vs everything-else | **PRIMARY** |
| M3 (M3+M4 flag) | YES (maturations) | new A-side synapses; but creation direction varies (lower-B x2, upper-B x1 in 424242) while the flip is uniform | distinguishes structural-additions-necessary vs not | SECOND-order (conditional arm) |
| M4 pruning | YES (all seeds; wave precedes flip in 2/3) | removes low-weight afferents | only via shared M3+M4 flag | SECOND-order (with M3) |
| M6 | YES (endpoint) | inhibitory activity shaping only (category C) | arm excluded: v2-M6 gate probe FAILED the frozen rate gate (301.7 Hz) — an M6-off arm is not viable without compensation (tuning, prohibited) | **EXCLUDED with evidence** |
| M5 | NO (0 evictions) | — | — | **NOT A CANDIDATE** |
| E6 beta | NOT RESOLVABLE | modulates STDP/M3 magnitudes | only becomes distinguishable via an E6-off arm | CONDITIONAL resolver |
| Network/adaptation | active but isolated contribution unresolvable | category D context | no clean knife | context only |

## 7. Candidate counterfactuals (existing frozen endpoints only:
## E14 k*-rule, frozen alignment/independence machinery, gates)

Run-start ablation (established convention: v2 ablations, E6 arms).
Canonical predictor: k* = 1, alignment A by T10-T60, B-independence
booleans unchanged, gates pass.

- **If M2 is necessary for rapid re-anchoring**: M2-off => no C->A
  flip at REV1: alignment at T10 still C (fails the sustained-k*
  rule); possibly no flip by T60; representation stays C-absorbed or
  degrades; gates must still pass (0 failures, >= 3 established,
  rates 20-250 Hz) for the outcome to be readable.
- **If M2 is not necessary**: M2-off => k* = 1 reproduced (flip at
  REV1 under the registered rule), A-alignment by T10.
- **If STDP is necessary**: STDP-off => no flip by T60 (alignment C
  sustained), or the flip is absent at every registered checkpoint.
- **If STDP is not necessary**: STDP-off => k* = 1 reproduced;
  endpoint weight movement still pervasive.

## 8. Proposed minimal E16 comparison

**Primary two-condition comparison (frozen curriculum E12:
60 SEQ / 60 REV, seeds, analyzer, telemetry, gates unchanged):**

1. canonical (E12 organism — the committed e12.toml; NEVER rerun;
   its E13/E14/E15 results are the canonical row),
2. **M2-off** arm (existing flag `disable_m2 = true`),
3. **STDP-off** arm (registered parameter-zero: `a_plus = 0.0`,
   `a_minus = 0.0` — no code change; E6's beta x 0 = 0 keeps the
   arm clean).

One arm set on the canonical seed (20260912) first; cross-seed
(9001, 424242) only if the canonical outcomes are informative and
the sole gates pass (P2 + engagement + the carried gate bars) —
registered policy, no factorial sweep.

Conditional third arm (only if the primary pair does not
distinguish): **M3+M4-off** (`disable_m3_m4 = true` — the existing
shared flag; creations-necessary question). Conditional resolver for
E6's role: **E6-off** (`[e6] enable = false`) only if beta
attribution becomes load-bearing in the primary outcomes. No
parameter sweeps, no tuning, no new flags/code.

## 9. What E16 would and would not establish

WOULD establish: necessity of M2 and/or STDP for the REV1 re-
anchoring in THIS organism at THESE seeds (per-arm flip/no-flip at
the frozen checkpoints); whether normalization-mediated re-expression
(B-class) or Hebbian weight dynamics (A/B-class) is required; with
the conditional arms, whether structural additions are necessary and
whether E6 modulation matters.

WOULD NOT establish: sufficiency of any mechanism; the interaction
(simultaneous necessity of M2 AND STDP) without a joint arm (NOT
proposed — the audit shows the pair is only discriminative, not
complete); any claim of generality beyond the frozen organism,
curriculum, 3 seeds; any intra-interval timing claims beyond emitted
events (D4); causality of accompaniment facts.

## 10. Protocol ambiguities requiring user approval

1. **Ablation semantics**: run-start ablation (full trajectory) is
   assumed, matching every prior arm (v2 ablations, E6 arms,
   nom3m4, m1only). Approve as E16's registered semantics.
2. **STDP-off form**: parameter-zero (a_plus = a_minus = 0.0) rather
   than a new flag — no code change; approve.
3. **Primary arm set**: exactly {M2-off, STDP-off} + canonical
   (never rerun); conditional {M3M4-off, E6-off}; approve.
4. **Cross-seed policy**: canonical seed first; cross-seed gated on
   gates + informative canonical outcomes; approve.
5. **Endpoint reuse**: E14 k*-rule + frozen alignment/independence
   machinery as the arm endpoints; no new thresholds; approve.
6. **Arm count/scale**: 1 seed x 2 arms (2 new runs) initially;
   nothing else without approval.

Stop — awaiting approval before any E16 freeze or execution.