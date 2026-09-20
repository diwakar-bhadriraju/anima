# CLLA capability experiment — execution record & bundle verdict

Status: EXECUTION RECORD, 2026-09-21. Protocol: docs/anima-clla-
protocol.md (frozen, amended c9d7a88 incl. corrected F6 basis, F7
wording, convention note). Architecture: bounded-protection CLLA
(viability PASS 5c28d49; implementation 925146c). All runs are the
frozen-protocol runs (same configs, bounded binary, determinism
byte-guarantee) — preserved in runs/; nothing rerun for measurement.

QUESTION: Does CLLA (bounded) preserve, coexist, and re-express
distinct sensory-specific synaptic memories?

## 0. Run inventory (12 experimental + identity)

| run id | arm | seed | ended | failures |
|---|---|---|---|---|
| clla-ident-il-20260920T181711Z | ident | 20260912 | curriculum-complete | 0 |
| clla-s20260912-il-20260920T181753Z | il | 20260912 | curriculum-complete | 0 |
| clla-s20260912-bac-20260920T181759Z | bac | 20260912 | curriculum-complete | 0 |
| clla-s20260912-bca-20260920T181805Z | bca | 20260912 | curriculum-complete | 0 |
| clla-s20260912-d-20260920T181847Z | d | 20260912 | curriculum-complete | 0 |
| clla-s424242-il-20260920T181810Z | il | 424242 | curriculum-complete | 0 |
| clla-s424242-bac-20260920T181816Z | bac | 424242 | curriculum-complete | 0 |
| clla-s424242-bca-20260920T181821Z | bca | 424242 | curriculum-complete | 0 |
| clla-s424242-d-20260920T181854Z | d | 424242 | curriculum-complete | 0 |
| clla-s9001-il-20260920T181827Z | il | 9001 | curriculum-complete | 0 |
| clla-s9001-bac-20260920T181833Z | bac | 9001 | curriculum-complete | 0 |
| clla-s9001-bca-20260920T181838Z | bca | 9001 | curriculum-complete | 0 |
| clla-s9001-d-20260920T181900Z | d | 9001 | curriculum-complete | 0 |

All end reasons from RunEnded telemetry; failures = [] everywhere.

## 1. Identity / integrity — PASS

- FNV 9647ea8a0ca4dbd2 (152,254 rows) byte-identical across ident
  reruns and the committed e24-il baseline (modulo documented
  V2.2-era z_latch artifact drift).
- 105/105 snapshot frames identical pre/post amendment.
- Suites: core 78, exp 58, telemetry 13, viz 5 — green.

## 2. PROTOCOL-DEFECT: F2/F6 frozen windows unexecutable

Frozen F2: reps {21..40} of each pattern; frozen F6 basis:
same-seed il-arm reps {21..40}. The frozen il curriculum
(present=["A","C"], reps=20 interleaved) delivers **20 A + 20 C
presentations** (verified by presentation-count scan on every il
run). The windows {21..40} are EMPTY. This is a metric/curriculum
mismatch in the frozen text — NOT a result-driven change. Per the
mandate (do not amend the protocol on intermediate results), F2
and the F6 basis are reported UNMEASURABLE under the frozen
definition; the closest frozen-adjacent windows (il reps 11-20,
the LAST 10 of each pattern) are reported as NON-DECISIVE
diagnostics. D arm has 40 D reps (v̄_D measurable).

## 3. Falsifier results

### F1 — A/C coexistence (frozen: raw drive-end channel mass ≥ 0.5×
### per-seed single-pattern reference, arms il/bac/bca)

| arm | s20260912 A/C | s424242 A/C | s9001 A/C | verdict |
|---|---|---|---|---|
| il | 0.370 / 0.291 | 0.365 / 0.258 | 0.322 / 0.305 | **PASS 3/3** |
| bac | 0.492 / 0.101 | 0.466 / 0.057 | 0.478 / 0.037 | FAIL 3/3 (C side) |
| bca | 0.116 / 0.479 | 0.056 / 0.428 | 0.055 / 0.461 | FAIL 3/3 (A side) |

**F1: FAIL (6/9 cells; alternating coexists, blocked does not).**
Refs (committed e24 single-pattern): 20260912 A 0.290/C 0.239;
424242 A 0.249/C 0.221; 9001 A 0.284/C 0.156.

Mechanistic depth: protected-mass ratio A/(A+C) at drive end —
il 0.526–0.589 (balanced), bac 0.916–0.983 (A-dominant), bca
0.049–0.106 (C-dominant). The blocked-order second block captures
both the shared working budget and — via M3 permanence + the
entry-gate — the per-neuron protected slots; the cap constrains
TOTAL protected mass per neuron but does not allocate it across
patterns. Exactly the recency-dominance signature already measured
in the unbounded audit (x-clla-f4-audit §2) and in the unmodified
substrate (x-synmem G3): blocked order overwrites; alternation
coexists. The amendment preserved that behavior; it did not (and
was not designed to) fix allocation.

### F2 — response separation (frozen window; UNMEASURABLE — see §2)

Non-decisive diagnostic on the last available 10 reps of each
pattern (il arms):

| run | early X/within/gap | late X/within/gap |
|---|---|---|
| s20260912-il | 0.527/0.741/+0.215 | 0.153/0.875/+0.723 |
| s424242-il | 0.403/0.675/+0.272 | 0.268/0.846/+0.579 |
| s9001-il | 0.441/0.683/+0.243 | 0.395/0.891/+0.496 |

All three late-window separations satisfy the frozen inequality
(X < 0.90, gap ≥ 0.05) on the closest available window — evidence
consistent with response separation, NOT a frozen-verdict pass.

### F3 — protected cap (P ≤ 0.75·t_e + 0.04 = 0.64 everywhere)

PASS 12/12: max per-neuron P = 0.6000000 (f32-exact) in every run.

### F3b — premature exhaustion (median P/cap ≥ 0.99 AND W ≤ 1e-6)

NOT EXHAUSTED 12/12: median P/cap 0.931–1.000; total working mass
11.85–16.45 at drive end (≫ 0). Note: median P near cap in several
runs — per-neuron protected slots are nearly full by drive end,
but working mass remains functional (capacity bounded, not dead).

### F4 — runaway/stability

PASS 12/12 (zero failures; covered by viability verdict 5c28d49).

### F5 — no hidden context

PASS (static conformance; unchanged from 47b1ecc + bounded clip).

### F6 — D composition (frozen: cos(v̄_D, v̄_A+v̄_C) ≥ 0.90; basis
### same-seed il reps 21-40 — UNMEASURABLE per §2)

Non-decisive diagnostic with the closest same-seed late basis
(il reps 11-20):

| run | cos(vD, vA+vC) | vsA | vsC | frozen criterion |
|---|---|---|---|---|
| s20260912-d | 0.822 | 0.682 | 0.578 | FAIL (< 0.90) |
| s424242-d | 0.909 | 0.792 | 0.668 | PASS_adjacent |
| s9001-d | 0.826 | 0.776 | 0.634 | FAIL (< 0.90) |

1/3 adjacent-pass, 2/3 adjacent-fail. D-response is substantially
A/C-correlated but does not meet the 0.90 composition bar in 2/3
seeds at this basis. D is NOT treated as a hidden-label falsifier
(per frozen interpretation).

### F7 — protected-mass erosion (frozen: drive-end ≥ 0.5 × peak)

PASS 12/12: protected A-mass ratio 0.802–1.000 (only s9001-d dips
to 0.80 via D-block crowding; all well above 0.5). Consolidated
structure is retained through the full 40-rep curriculum — the
bounded protection does not erode.

## 4. Primary-question answers

1. COEXISTENCE: **Partial.** Alternating (il): YES (balanced
   protected A/C 0.53-0.59, F1 pass). Blocked (bac/bca): NO —
   second block dominates protected and raw mass (F1 fail 6/6).
2. RESPONSE SEPARATION: **Evidence consistent** — late-epoch
   cross-class cosine 0.15-0.40 vs within 0.85-0.89 in all 3 il
   arms (diagnostic window; frozen window unmeasurable).
3. PROTECTION: **Yes** — consolidated synapses retained (F7),
   working pool continued adapting (F3b not exhausted).
4. RETRIEVAL: **Consistent with re-expression** — the late-window
   response separation indicates after 20 alternations the A/B
   (A/C) responses are pattern-distinct, i.e. forward stimulation
   reactivates the retained configuration.
5. D COMPOSITION: INCONCLUSIVE (adjacent 1/3 pass; frozen bar
   unmeasurable).
6. CAPACITY: **One structure per neuron group demonstrated
   (bac/bca); two under alternation (il).** No pathological
   collapse (F3/F3b/F7 pass); but blocked-order allocation shows
   the budget is captured by the later block — capacity supports
   coexistence only when patterns interleave.

## 5. Operating-point confound

The 3-7 Hz primary operating point (viability) is LOW but non-zero:
within-window counts are substantial (responses separate with
norm ~10² spikes/vector), so the response-separation measurements
are not noise-limited (X 0.15-0.40 with W 0.85-0.89 is far from
the 0.5 noise floor). No measurement in this record is confounded
by quiescence; d-arm s9001 peak 138.8 Hz is the known
first-presentation overdrive (confounded by stimulus, not by
operating point — reported, not evidence).

## 6. Bundle verdict (frozen §7, binary)

Falsifiers under the FROZEN definitions:
- F1 FAIL (blocked-order coexistence, 6/9 cells)
- F2 UNMEASURABLE (frozen window empty — protocol defect),
  adjacent diagnostic passes
- F3 PASS; F3b PASS; F4 PASS; F5 PASS
- F6 UNMEASURABLE (frozen basis empty — same defect),
  adjacent 1/3 pass
- F7 PASS

**BUNDLE: NOT SUPPORTED under the frozen criterion set** — F1
fails regardless of the F2/F6 measurement defect. The failure is
specific and architectural: blocked-order recency capture of the
shared budget/protected slots (allocation), not protection,
stability, retention, or drift. Alternation — the regime the
capability question names ("repeated alternating exposure") —
PASSES coexistence and shows separation; the blocked arms fail
it.

## 7. What this does and does not claim

- Establishes: bounded CLLA retains structure (F7), protects
  (F3/F3b), separates responses under alternation (adjacent F2),
  and fails coexistence under blocked order (F1) — matching the
  unmodified substrate's order-dependent behavior. The cap fixed
  the runaway, not the allocation.
- Does NOT claim: D hidden-label evidence; F2/F6 verdict
  decisions (unmeasurable); that blocked-order is "wrong" — the
  frozen criterion simply demands coexistence there and CLLA
  (bounded) does not provide it.
- No E-number. No mechanism amendment. All runs preserved.

STOP — capability experiment complete; results as reported.