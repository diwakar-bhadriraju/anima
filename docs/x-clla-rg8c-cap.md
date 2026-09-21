# k_g = 8.0 + threshold-completion cap — frozen protocol (2026-09-22)

Registration B of the local recruitment gain. Authority: docs/x-clla-rg8-
result.md §6 (recorded-but-not-adopted alternative, now adopted by user
decision). Registration A (uncapped, 272e0e5) closed: operating-point FAIL.

## 1. Mechanism (amended equation — only change)

```
i_boost_i(t) = min( k_g · max(0, 1 − R_live,i(t)) · I_W,i(t),
                    max(0, (v_th − v_i(t)) · τ_m / dt) )
```

- k_g = 8.0 UNCHANGED (frozen mechanism constant, `recruit_gain_k()`).
- The cap uses ONLY existing constants (v_th = 1.0, τ_m = 20, dt = 1) and
  the neuron's own membrane potential — parameter-free.
- Semantics: the boost can never supply more current than what takes THIS
  neuron from its current v to threshold in one step. At block-1 strong
  drive (I_W ≈ 12, R = 0, raw boost ≈ 96) the cap binds: across the
  pres-1 fluctuation v ∈ [0, 1], cap ≈ 0–20 current, so total drive
  ≈ 38 + ≤20 instead of 108 — the ~3× overshoot is removed. At the weak
  first-exposure point (I_W ≈ 1.565, raw boost ≈ 12.5, cap = 20 when v =
  0) the cap does NOT bind — the mechanism's target case is untouched.
- Everything else byte-identical to registration A: gate (R_live live
  accumulators, silence convention R := 1), window cadence, flag
  (`recruit_gain`, V2Params), k_g constant, no config knob for the cap.

## 2. Matrix (identical to registration A)

9 gain arms (`clla-rg8c-s{seed}-{order}` = committed clla-fe config +
`recruit_gain = true`; exp_id renamed for provenance) vs the 9 committed
baselines. Orders (verified): bac = blocked [A,C], bca = blocked [C,A],
il = interleaved. Seeds s20260912, s9001, s424242.

## 3. Predeclared endpoints — IDENTICAL bars to registration A

4a no runaway trips incl. block-1 pres 1–5; 4b block-1 pres 1–5 mean
burst rate ∈ [100, 200] Hz (E3 band); 4c first-pattern posts at pres 20
≥ 0.9 × basline; 4d maxP ≤ 0.6; 1 end-of-block-2 novel protected mean
≥ 0.09; 2 first-novel posts/n ≥ 0.5; 3 novel permanence block 2 ≥ 150;
5 IL preservation (C posts ≥ 0.8 × base, C permanence within ±20 % of
98); 6 gap activity within ±0.5 Hz of baseline.

## 4. Outcome rules — identical to registration A

- 4a/4b fail → operating-point failure classification FIRST; weak-pattern
  hypothesis verdict only via endpoints 1–3.
- Primary question unchanged: does the (capped) gain let first-exposure
  novel patterns recruit enough posts to establish protected structure?
- PASS → report + separate magnitude discussion. FAIL → STOP, report the
  failure mechanism; no reparameterization in this registration; no
  E-number assigned.

## 5. Interpretation notes

- The cap changes ONLY the strong-drive limb. If 4b passes and 1–3 pass,
  registration A's operating-point diagnosis is confirmed and the
  recruitment concept is vindicated at k_g = 8.
- If 4b passes but 1–3 still fail, the weak-pattern hypothesis is
  falsified at this substrate (blocked churn precedes exposure — the
  IL-vs-blocked asymmetry).
- If 4b still fails with the cap, the overshoot is not availability-
  driven but recurrence-driven; report measured burst decomposition.

STOP — protocol frozen.