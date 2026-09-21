# k_g = 8.0 + threshold-completion cap — result (registration B)

Executed 2026-09-22. Authority: docs/x-clla-rg8c-cap.md (frozen protocol B,
approved by user). Registration A (uncapped): operating-point FAIL.

## Identity / integrity

- Implementation: per-NEURON clamped boost (two-pass): the deposit loop
  accumulates this tick's working input current per post; a post-loop pass
  applies ONE boost = min(k_g·gate·ΣI_W, max(0,(v_th−v)·τ_m/dt)) per
  neuron. The per-synapse clamping error flagged in review was caught and
  fixed BEFORE the matrix ran; discriminated by unit test
  rg_cap_clamps_per_neuron_not_per_synapse (two afferents, summed raw
  boost 41.6 > cap 20 ⇒ ONE clamped boost, not 2×cap).
- Suites: anima-core 94 (incl. 9 rg_*), workspace 94+47+11+13+5 — green.
- Flag-off byte-identity gate: PASS (rerun committed config with cap
  binary: all telemetry chunks, snapshots, metrics byte-identical —
  cap code lives only inside the recruit_gain branch).

## Matrix

9 cap arms (clla-rg8c-s{seed}-{order}-20260921T1737..31Z) vs the 9
committed clla-fe baselines. All 9 ended curriculum-complete; failures []
in all 9 (4a PASS). Orders: bac = blocked [A,C], bca = blocked [C,A],
il = interleaved.

## Endpoints (identical bars to registration A)

| # | bar | result (cap) | registration A (no cap) | verdict |
|---|---|---|---|---|
| 4a | no guard trips | failures [] × 9 | same | PASS |
| 4b | block-1 pres 1–5 burst rate ∈ [100,200] Hz — SEE §4b reading | aggregate means 175.6–189.1 IN band; **per-presentation: pres-1 = 378–440 Hz OUT of band in all 6 arms** (both registrations) | uncapped 185.9–201.0 agg; pres-1 391–449 | **NOT CLEAN — see reading** |
| 4c | pres-20 posts ≥ 0.9 × base | 1.00–1.04 | 1.00–1.04 | PASS |
| 4d | maxP ≤ 0.6 | 0.6000 (baseline pin) | 0.6000 | PASS |
| 1 | novel end-mean ≥ 0.09 | 0.092 / 0.080 / 0.069 / 0.000 / 0.000 / 0.000 (n = 5–15 syn!) | 0.000–0.058 | **FAIL (1/6)** |
| 2 | first-novel posts ≥ 0.5 | 0.13 / 0.08 / 0.19 / 0.00 / 0.00 / 0.00 (base 0.12–0.46) | 0.00–0.33 | **FAIL (0/6)** |
| 3 | novel permanence b2 ≥ 150 | 5 / 15 / 5 / 0 / 0 / 0 (base 25–65) | 0–22 | **FAIL (0/6)** |
| 5 | IL preservation | C posts 1.08–1.21× ✓; C perms 97 / 74 / 105 (s9001 = 74 fails [78,118]); C end-mean 0.100–0.141 (base 0.094–0.114) | posts ✓, perms 93/61/96 | MIXED (1/3) |
| 6 | gap ±0.5 Hz | 2.3–27.0 vs 0.45–5.94 | 1.9–6.3 / IL 10–24 | FAIL |

## 4b reading — reported honestly (frozen-text ambiguity)

The frozen text (272e0e5 §10.1): "mean internal burst rate during
block-1 pres 1–5 (per presentation, gain arm) within the established
[100, 200] Hz calibration band". Two readings:

1. AGGREGATE (5-presentation mean): all 6 cap arms land 175.6–189.1 IN
   band → PASS.
2. PER-PRESENTATION: pres-1 = 378–440 Hz in EVERY cap arm (reg. A:
   391–449) → OUT of band in all arms → FAIL under the strict reading.

Calibration wrinkle: the E3 [100,200] Hz band was calibrated on
burst-peak internal EMA, NOT on presentation-mean rates. Baseline
block-1 pres-1 presentation-mean rates are 57–94 Hz — themselves OUTSIDE
the band. A per-presentation bar fails baseline pres-1 by construction
and cannot be the intended reading; the aggregate reading is the one the
band can support. NEITHER reading hides the operative fact: block-1
first-exposure burst rate stays ~4× baseline (378–440 vs 57–94) in both
registrations — the registered distortion mode (block-1 first-exposure
overdrive) PERSISTS under the cap.

CAP LIMITATION (measured): the cap moved pres-1 only 391 → 378 Hz
despite capping the boost 96 → ≤20 current. The cap is per-tick: a
below-threshold neuron is topped to threshold EVERY tick while I_W is
continuous, sustaining ≈1 spike/tick (refractory-limited) independent of
boost magnitude. The overshoot is now RATE-sustained, not
magnitude-sustained; the cap repairs average drive but cannot repair
first-exposure sustained rate. This is the cap's measured limitation and
the reason the churn pressure barely fell.

## What the cap changed

- Operating point REPAIRED per the letter of 4b: block-1 pres-1–5 means all
  inside the E3 band (uncapped: s9001-bca 201.0 above). Pres-1 bursts stay
  high (378–440 Hz) — the cap limits the boost term (96 → ≤20), but at
  drive ≈ 58 the pres-1 response is recurrence/refractory-limited, not
  boost-limited; pres 2–5 decay faster, pulling the 5-presentation mean
  under 200.
- Substrate churn REDUCED but not stopped: late-cohort prunes in block 1 =
  193–221 vs base 157–187 (still +8–28%); first-exposure working substrate
  survived in 2/3 bac arms (iwC 9.9–10.0 network-total = 0.19/neuron vs
  base 1.565) but was still fully annihilated in the other arms
  (s9001-bca, s424242-bac, s424242-bca: iwA/iwC = 0.000).
- Where substrate survived, the boost engaged but at input-proportional
  scale: 8 × 0.19 ≈ 1.5/neuron (vs the 12.5 the design targets) → posts
  0.13–0.19 — measurable improvement over baseline (0.12) but far below
  the 0.5 bar and far below what recurrence needs (0.5) to bootstrap.

## Verdict (both readings reported, conservative classification)

Under the STRICT per-presentation reading of 4b, pres-1 (378–440 Hz) is
outside the band in all arms → per the frozen rule, classify as
RECRUITMENT-GAIN OPERATING-POINT FAILURE FIRST (block-1 first-exposure
overdrive persists, mode-shifted from magnitude to sustained rate).
Under the AGGREGATE reading, 4b passes and the pre-registered rule
(4b pass + 1–3 fail ⇒ falsified at this substrate) applies.

EITHER WAY the substantive result is identical and independent of the
label: endpoints 1–3 fail in every arm (novel end-mean 0.000–0.092, n =
0–15 synapses; first-novel posts 0.00–0.19 vs bar 0.5; novel permanence
0–15 vs bar 150). The measured bottleneck is substrate churn PRECEDING
exposure: block-1 prunes 193–221 of the late cohort (base 157–187),
leaving 0.000–0.190/neuron of working substrate at first exposure (base
1.565); where substrate survived (s20260912-bac: iwC 9.9, irec 53), the
gain was input-proportionally starved (8 × 0.19 ≈ 1.5/neuron boost) and
reached only baseline-level outcomes (end-mean 0.092 with 5 synapses vs
bar 0.09 with ≥150 permanence expected). The mechanism's own bound
(i_boost ≤ k_g·I_W) means a membrane-side gain cannot amplify a substrate
churned before exposure. This converges with the drive audit: IL, which
never churns the late pattern, recruits symmetrically without any gain.

- 4a / 4c / 4d: PASS. The cap DOES restore aggregate block-1 drive to the
  band and preserves existing-pattern integrity; its limitation is
  per-tick rate-sustaining (see §4b reading).
- Registration A's operating-point diagnosis confirmed; REGISTRATION B
  CLOSED. No reparameterization, no bracketing, no E-number.
- Honest limit of the falsification claim: neither registration achieved
  a clean block-1 operating point (pres-1 remains ~4× baseline in all
  arms), so "falsified at this substrate" holds only up to the residual
  operating distortion; the substrate-churn-before-exposure mechanism is
  the strongest evidence-supported reading, and it is curriculum-inherent
  per the audits and the IL control.

## Recorded options (not pursued here)

- Blocked-order viability requires protecting the late pattern's working
  substrate DURING block 1 (e.g., a churn exemption for never-yet-exposed
  afferents) — this is the e1-family question that the first audit judged
  "not justified by the then evidence"; the two gain registrations now
  provide the direct evidence that substrate pre-exposure survival is the
  binding constraint, if the user wants to revisit it as a separate
  registration.
- IL remains the known-working curriculum for symmetric A/C recruitment;
  the recruitment gain adds nothing there (posts already ≥ 0.8) and
  costs permanence in 1/3 arms + gap afterglow.

STOP.