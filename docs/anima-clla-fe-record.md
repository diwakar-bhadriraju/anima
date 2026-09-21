# First-exposure allocation experiment — execution record & verdict

Status: EXECUTION RECORD, 2026-09-21. Protocol: docs/x-clla-
first-exposure-audit.md (frozen fb11b67; approved for execution).
Implementation: commit b4d9ba8 (fired-channel draw bias +
pressure-eviction reuse; identity PASS). 13 runs executed and
preserved. No tuning, no post-hoc changes, no E-number.

## 0. Run inventory (13 executions; telemetry-authoritative)

| run id | arm | seed | ended | failures |
|---|---|---|---|---|
| clla-ident-il-20260921T082718Z (flag OFF) | ident | 20260912 | curriculum-complete | 0 |
| clla-fe-s20260912-bac-20260921T082750Z | bac | 20260912 | curriculum-complete | 0 |
| clla-fe-s20260912-bca-20260921T082755Z | bca | 20260912 | curriculum-complete | 0 |
| clla-fe-s20260912-il-20260921T082800Z | il | 20260912 | curriculum-complete | 0 |
| clla-fe-s20260912-d-20260921T082839Z | d (info) | 20260912 | curriculum-complete | 0 |
| clla-fe-s424242-bac-20260921T082805Z | bac | 424242 | curriculum-complete | 0 |
| clla-fe-s424242-bca-20260921T082811Z | bca | 424242 | curriculum-complete | 0 |
| clla-fe-s424242-il-20260921T082816Z | il | 424242 | curriculum-complete | 0 |
| clla-fe-s424242-d-20260921T082844Z | d (info) | 424242 | curriculum-complete | 0 |
| clla-fe-s9001-bac-20260921T082821Z | bac | 9001 | curriculum-complete | 0 |
| clla-fe-s9001-bca-20260921T082826Z | bca | 9001 | curriculum-complete | 0 |
| clla-fe-s9001-il-20260921T082830Z | il | 9001 | curriculum-complete | 0 |
| clla-fe-s9001-d-20260921T082849Z | d (info) | 9001 | curriculum-complete | 0 |

All RunEnded = curriculum-complete @105001; failures = [].

## 1. Identity / integrity — PASS

- Ident (flag OFF): FNV 9647ea8a0ca4dbd2 (152,254 rows); 105/105
  frames byte-identical — fe bias + eviction-reuse invisible.
- Suites: core 84 (fe pressure test), exp 58, telemetry 13,
  viz 5 — green.

## 2. S1 PRIMARY — second-block protected mass ≥ 0.5 × first-block

| run | arm | first | second | ratio | verdict |
|---|---|---|---|---|---|
| s20260912-bac | bac | 20.30 | 3.41 | 0.168 | FAIL |
| s424242-bac | bac | 21.68 | 3.56 | 0.164 | FAIL |
| s9001-bac | bac | 23.58 | 1.71 | 0.072 | FAIL |
| s20260912-bca | bca | 17.78 | 3.83 | 0.216 | FAIL |
| s424242-bca | bca | 24.14 | 2.25 | 0.093 | FAIL |
| s9001-bca | bca | 21.39 | 2.96 | 0.139 | FAIL |

S1: FAIL 6/6 (ratios 0.072–0.216).

## 3. Causal path (instrumented; bac s20260912)

1. ACTIVE INPUT AT FIRST EXPOSURE: visible C channels 37/52 =
   0.71/neuron (uC at t=44k) — the surviving-afferent visibility
   cap, lower than the audit's 1.25 estimate (whose onset sample
   was t=41k; churn continued 41k→44k).
2. CANDIDATE BOUND: via PRESSURE-EVICTION only (pool full of
   other-channel candidates + fired channel absent → bind).
   Implementation fact verified in unit test: the REDRAW-BIAS path
   is inert — fired_channels is populated only from live outgoing
   synapses, and draw_candidate's connected() skip excludes
   exactly those channels, so the bias falls through to random.
3. CANDIDATE WEIGHT TRAJECTORY: bound at w_c_init 0.01; at g=1.0
   (R=0 in C block) accumulates +0.01/co-active window → θ=0.05
   in ~4-5 windows ≈ 1 presentation (with off-decay 0.99^15).
4. PERMANENCE RATE: 41 C-cohort events over 20 C presentations =
   2.05/pres (vs alloc-only 35 = 1.75/pres; reserve 38). The
   counterfactual's ~65-first-round supply did NOT materialize —
   binding is limited to full-pool moments.
5. TIME TO PERMANENCE: first C permanence at t=47,400 — 2.4 s
   after C-block onset (45,001) — 10× faster than baseline (which
   had no C permanence until mid-block). The fe mechanism WORKS;
   its VOLUME is the failure.
6. SECOND-BLOCK PROTECTED MASS: 3.41 (s20260912-bac).
7. FIRST-BLOCK PROTECTED MASS: 20.30 — HIGHER than baseline
   (16.85): the fe bias benefits the ACTIVE pattern (block 1's
   fired channels are bound aggressively; more permanence 248 vs
   170). This is the mechanism's real effect: it allocates to
   whatever is currently firing (including block 1), not
   specifically to the future second block.
8. RELEVANT CANDIDATES SURVIVING TO SWITCH: pool C-cohort at
   onset 108–124 at w 0.007–0.016 (reserve-run boundary data — fe
   runs lack CandidatePool telemetry; see §5 S6 note).
9. VISIBILITY CAP OR FLUX BINDING: VISIBILITY CAP + INERT REDRAW
   PATH. Flux (permanence accumulation rate) is NOT the limiter:
   bound candidates reach θ within ~1 presentation. The binding
   SUPPLY is capped: (a) visible channels 0.71/neuron at onset;
   (b) redraw-bias inertness means binding only under full-pool
   pressure, which is rare at the transition.

## 4. Secondary/sanity

S2 ALTERNATING IL PRESERVATION: pA/pC — s20260912: 11.41/11.43
(ratio 0.500); s424242: 10.73/13.46 (0.444); s9001: 8.54/12.64
(0.403). Balanced-coexist (0.40–0.50; slightly C-leaning vs
baseline 0.50–0.62 — the bias handles both cohorts once each is
active). PASS (no degradation).
S3 CAP: max per-neuron P = 0.6000 (P ≤ 0.6+1e-6) — PASS.
S4 M2 TARGET: P ≤ 0.6 < t_e ⇒ t_e − P > 0 — PASS (by invariant +
no degenerate-skip signature).
S5 STABILITY: 13/13 curriculum-complete, zero failures, no new
class — PASS.
S6 FINITE POOL + CONFORMANCE: pool code path unchanged (c_slots=6,
no resize); fe bias reads only the neuron's fired set + pool; no
labels/identity/global — static PASS. NOTE: CandidatePool telemetry
emits only under dormant_reserve, so fe runs have no pool rows; the
pool invariant was verified in the reserve execution (312 at every
row) and the code path is identical.

## 5. Verdict (frozen binary)

BUNDLE: NOT SUPPORTED — S1 fails 6/6.
The frozen question ("Does first-exposure rebinding prevent
second-block starvation by ensuring currently active novel inputs
can acquire candidate substrate before M2/M4 eliminate it?") is
answered NEGATIVELY with a precise mechanism:
  - rebinding WORKS (time-to-permanence 2.4 s; block-1 allocation
    increased 248 vs 170 events);
  - but it binds only channels VISIBLE through surviving live
    afferents (~0.71/neuron at onset), and only when the pool is
    full (the redraw-bias branch is structurally inert because
    visible ⇒ connected ⇒ skipped);
  - supply therefore remains ~ baseline (41 vs 35 events), far
    below the counterfactual's ~65 first-round estimate (which
    assumed redraw-path binding that cannot occur as implemented);
  - second-block protected mass unchanged (~1.7–3.8 vs 0.75–4.5
    prior experiments).
The mechanism's true effect: allocation follows the CURRENTLY
ACTIVE pattern — it strengthens block 1 (20.3–24.1 vs 16–21) and
would help ANY pattern once visible, but it cannot create
substrate for a pattern whose afferents already died.

## 6. Classification (per mandate a–e)

a) INSUFFICIENT FIRST-ROUND VISIBILITY: YES — 0.71–1.25 visible
   channels/neuron; invisible channels cannot be bound.
b) CANDIDATE CAPACITY: NO — 6 slots, 154+ free at onset.
c) PERMANENCE FLUX: NO — bound candidates reach θ in ~1
   presentation; flux is not the bottleneck.
d) PROTECTED-CAP INTERACTION: NO — cap respected, headroom 0.28.
e) OTHER: YES — the implementation fact: redraw-bias inertness
   (visible ⇒ connected ⇒ skipped) means binding only under
   full-pool pressure; the audit's counterfactual implicitly
   assumed the bias engages at redraw, which the connected()
   guard prevents. THIS is the binding gap to fix if the idea
   were pursued (e.g., relax connected() for fe draws or bind on
   every redraw when a visible unpooled channel exists), NOT the
   visibility cap per se.

## 7. What is established / not claimed

- Established: fe allocation accelerates first contact (2.4 s to
  permanence); strengthens the active pattern; alternating
  preserved; all sanity pass.
- NOT claimed: memory success (S1 failed); that the visibility
  cap is fatal (classification e identifies the inert-redraw as
  the fixable gap); D-arm evidence (informative only, complete
  3/3).
- The correct NEXT question (for review, NOT executed): does
  relaxing the fe-redraw inertness (bind a visible unpooled fired
  channel on every redraw, not only under full-pool pressure)
  lift the supply, or does the visibility cap (~0.7–1.25/neuron at
  onset) bind regardless? That is an amended-mechanism question;
  no amendment proposed without approval.

F2/F6 11–20 correction retained; historical verdicts untouched.
All 13 runs preserved. STOP — experiment complete.