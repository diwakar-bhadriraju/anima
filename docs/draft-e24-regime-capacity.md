# DRAFT — regime-selection capacity experiment (NOT executed, NOT preregistered)

Status: DRAFT protocol proposal derived from X1–X3 exploratory
findings (docs/xplor-log.md §2–§6). Not frozen, not committed as
a protocol, not executed. Prepared for user review only. If
approved, it would become a numbered E-series experiment with
frozen endpoints BEFORE implementation.

## Motivation (evidence so far, all exploratory)

- u produces persistent endogenous activity (Stage B, valid).
- Per-trial vector readouts show NO antecedent code (X2: u-vector
  A/C gap +0.003, p=0.12; off-window spike vectors cos≈0.999).
- But the REGIME the network settles into (silent / decaying /
  self-sustained pacemaker) depends on the drive it received, at
  fixed (β, τ, seed): X2 O2.1 (C-only→pm, A-only→decay,
  interleaved→death at 20260912/0.0046875) and X3 O3.3 (drive
  shifts regime within the transition band at 2/4 clean seeds).
- The mechanism candidate is a per-neuron u margin over a
  self-regeneration threshold: drive-end top-u > ~2 predicts
  persistence (0.85 acc; silent runs never exceed 0.92,
  pacemakers never below 6) — X3 O3.4.

Question (draft): **Can the organism's endogenous regime encode
which CURRICULUM it experienced, and does that encoding survive
beyond the drive (i.e., is it memory rather than reflex)?**

This is a capacity question about the slow state u as a
regime-selector — distinct from E21's per-trial D_L metric (which
X2 shows is dead at these operating points) and from the retracted
Stage C (per-trial gap sweep at the rule-selected point, which
X2 O2.3 shows would abort anyway).

## Draft design sketch (for critique, not execution)

Arms (curricula), all at the transition band cell (β=0.0046875,
τ=5000) unless the band placement is revised:
  1. A-only drive (40 reps)
  2. C-only drive
  3. A/C interleaved
  4. blocked: 20×A then 20×C
  5. blocked: 20×C then 20×A
(4 vs 5 isolate ORDER at matched composition — the cheapest
sequential-history signature.)

Readouts (all from existing telemetry/snapshots, no new
mechanisms):
  - R1 regime class of the post-drive silence (existing v21phase
    classifier, thresholds frozen at commit time of the protocol).
  - R2 drive-end top-u (neuron id + magnitude) and top-3 sum.
  - R3 silence trajectory: spikes/2s bins, active set, half-life.
  - R4 (secondary) output-neuron participation: does any output
    join the persistent core (E19-relevant: endogenous OUTPUT
    activity is the closed-loop prerequisite).

Seeds: ≥5 (X3's five, pre-declared). Primary endpoint (draft):
regime-separation — for each pair of curricula (i, j), the
fraction of seeds on which the silence regime class (or, better,
a frozen scalar summary like first-10s silence spike count, which
X3 shows spans 0 to ~10^4) differs by more than a pre-registered
margin. Secondary: is the blocked-A→C vs C→A pair separated at
matched composition (order sensitivity).

Predictions of the u-margin hypothesis (falsifiable):
  - P1 regime differences track drive-end top-u margins
    monotonically.
  - P2 order matters where the SECOND block's antecedent recruits
    a different winner neuron (else blocked arms collapse).
  - P3 no per-trial separation at any point (X2 replicated).

Failure modes / risks to design around:
  - Abort fragility in the band (X3 O3.5: 2 anomalous mid-drive
    aborts at 9001). May need the E2b-style detector-recalibration
    amendment pattern, or band-center re-placement declared BEFORE
    freezing.
  - Seed-inconsistency of the drive effect (2/4 seeds in X3):
    n=5 may underpower; the draft should specify how abstaining
    seeds are counted (as "no signal", not dropped).
  - Regime class is coarse (3 values); the frozen scalar
    (first-10s spike count) is continuous and safer for
    pre-registration.

Scientific-constraint check (draft): local mechanisms only (u is
unchanged); no labels/reward; curricula differ only in stimulus
schedule (environment-side, Category-A-like interface usage); no
hidden supervision; deterministic seeds; negative results
retained. No E-number assigned until user approves promotion.

## Explicitly NOT done

- This draft was not executed.
- The old frozen Stage C was not executed (its rule remains
  preserved in docs/v2_1-spec.md; X2 O2.3 records that its E21-
  paradigm drive at the selected point aborts on P2 — any
  decision about superseding it belongs to the user).
- V2.1 was not modified (all X3 runs vary only the two registered
  V2.1 config fields and drive composition).
