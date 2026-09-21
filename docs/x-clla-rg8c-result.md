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
| 4b | block-1 pres 1–5 mean burst ∈ [100,200] Hz | **175.6 / 184.6 / 183.1 / 189.1 / 184.5 / 184.3** (pres-1 378–440 still; pres 2–5 decay 68–217) | 185.9–201.0 (1 edge fail) | **PASS** |
| 4c | pres-20 posts ≥ 0.9 × base | 1.00–1.04 | 1.00–1.04 | PASS |
| 4d | maxP ≤ 0.6 | 0.6000 (baseline pin) | 0.6000 | PASS |
| 1 | novel end-mean ≥ 0.09 | 0.092 / 0.080 / 0.069 / 0.000 / 0.000 / 0.000 (n = 5–15 syn!) | 0.000–0.058 | **FAIL (1/6)** |
| 2 | first-novel posts ≥ 0.5 | 0.13 / 0.08 / 0.19 / 0.00 / 0.00 / 0.00 (base 0.12–0.46) | 0.00–0.33 | **FAIL (0/6)** |
| 3 | novel permanence b2 ≥ 150 | 5 / 15 / 5 / 0 / 0 / 0 (base 25–65) | 0–22 | **FAIL (0/6)** |
| 5 | IL preservation | C posts 1.08–1.21× ✓; C perms 97 / 74 / 105 (s9001 = 74 fails [78,118]); C end-mean 0.100–0.141 (base 0.094–0.114) | posts ✓, perms 93/61/96 | MIXED (1/3) |
| 6 | gap ±0.5 Hz | 2.3–27.0 vs 0.45–5.94 | 1.9–6.3 / IL 10–24 | FAIL |

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

## Verdict (per frozen outcome rules, §4 of the protocol)

4b PASSES but endpoints 1–3 FAIL → **the weak-pattern recruitment
hypothesis is FALSIFIED at this substrate under the corrected operating
point**: a membrane-side, input-proportional recruitment gain cannot
compensate substrate churn that precedes exposure in the blocked
paradigm. The mathematical reason is the mechanism's own bound
(i_boost ≤ k_g·I_W): block-1 churn removes 72–100 % of the late pattern's
working afferents before its first presentation, so no membrane-side gain
has anything to amplify. This converges with the drive audit's
curriculum-inherent conclusion (IL, which never churns the late pattern,
recruits symmetrically without any gain).

- 4a / 4c / 4d / 4b: PASS — the SAME k_g = 8 is safe at the strong-drive
  operating point once capped; existing-pattern integrity fully preserved.
- Registration A's operating-point diagnosis is CONFIRMED (capping removed
  the 4b failure); the mechanism's recruitment target is not reachable at
  this substrate.
- Registration B closed. NO reparameterization, NO bracketing, NO E-number.

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