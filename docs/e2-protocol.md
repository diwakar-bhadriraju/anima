# E2 Protocol — STDP Bound Variants: Additive vs Multiplicative

**Status**: pre-registered before execution. Thresholds below are the ONLY
rules the auto-comparison uses; verdicts are labeled `auto-generated —
review` and humans own final interpretation.

## Hypothesis

The near-threshold assembly-formation and novelty results in E1 are
caused by the **plasticity rule's** hard additive bounds: with
Δw independent of w, synapses saturate at w_max, stop adapting, and
co-active patterns pool onto the same internal neurons instead of
specializing. A multiplicative rule (Δw ∝ w for depression, Δw ∝ (1−w)
for potentiation) keeps synapses in an adaptable range, sustaining
competition — predicted to yield **higher assembly separation (higher
assembly score, higher selectivity) and stronger novelty discrimination
than Arm A, with retention still ≥ 80%**.

**The only experimental variable is the plasticity rule.** Organism
morphology, connectivity, curriculum, pattern structure/overlap, seed,
caps, and all other parameters are identical to E1 and identical between
arms.

## Design

Clean 2-arm A/B, identical initial conditions (same seed 20260912,
same config hash base, same E1 curriculum):

| | Arm A (control) | Arm B (treatment) |
|---|---|---|
| Rule | pairwise additive STDP (E1, D7) | pairwise multiplicative STDP |
| LTP | Δw = +a_plus · pre_trace | Δw = +a_plus · pre_trace · **(1 − w)** |
| LTD | Δw = −a_minus · post_trace | Δw = −a_minus · post_trace · **w** |
| Bounds | hard clip [w_min, w_max] | soft (the w(w−1) factors are the bound) |

All other parameters = `configs/e1.toml` exactly: 24 input / 40 internal /
12 output, p=0.038, amplitude 52.0, a± = 0.005/0.0053, tau± = 20 ms,
decay 1e-6/tick, silence/prune thresholds unchanged, seed 20260912,
curriculum S0/S1/S2/S3 unchanged (patterns A, B, C; novel D = A+C).

Configs: `configs/e2-armA.toml` (rule = "stdp-pairwise") and
`configs/e2-armB.toml` (rule = "stdp-multiplicative"). The configs differ
in `exp_id` (e2a / e2b) and `rule` — nothing else.

## Metrics (identical machinery as E1)

Per arm, from full telemetry + snapshots:
- **Assembly score** (within − cross pattern cosine, early vs late S1)
- **Selectivity** (median (r_best − r_2nd)/r_best over internal neurons)
- **Novelty** (D response-profile distance to nearest learned pattern vs
  learned pairwise max)
- **Retention** (S3 vs late-S1 response ratio per pattern)
- **Saturation fraction** (share of synapses with w > 0.9 over time —
  the diagnostic this experiment targets)

## Verdict thresholds (pre-registered, unchanged from E1)

Per arm:
- **Assembly formation supported**: late-S1 assembly score ≥ 2× early-S1
  AND median selectivity > 0.5. One of the two ⇒ weakly supported; else
  inconclusive.
- **Novelty discrimination supported**: D-to-nearest-learned distance
  ≥ 2× learned pairwise max; else inconclusive.
- **Retention supported**: S3 ≥ 80% of late-S1 for all of A/B/C;
  50–80% weakly supported; < 50% inconsistent.

Secondary analysis (between-arm, explicitly NOT a pre-registered pass/fail
rule — descriptive comparison):
- Arm B vs Arm A on assembly score, selectivity median, novelty ratio
  (D-distance / learned pairwise max), saturation fraction trajectory.
- Interpretation guidance: if Arm B shows higher separation metrics with
  retention ≥ 80%, the E1 inconclusives are attributable to the additive
  bound shape. If both arms saturate or both fail identically, the binding
  problem lies elsewhere (e.g. morphology/inhibition absence) — feed E3+.

## Scope guards

- No neuron-count, curriculum, pattern-overlap, threshold, or cap changes.
- No analyzer changes (same code path for both arms; the only new code is
  the multiplicative rule itself, unit-tested against hand-computed Δw).
- Determinism: each arm's telemetry must be byte-identical across re-runs
  of the same seed.

## Execution record — Arm B failure and amendment A1

**Arm B first run (`runs/e2b-20260913T082332Z`) failed at t=10.2 s with
`failure:runaway-activity`** (internal mean rate 399 Hz sustained 5 s,
vs the pre-registered 50 Hz runaway detector). Trajectory: S0 silence
fine; at first S1 onset (t=6 s), ~32 of 40 internal neurons locked into
a self-sustaining burst; weights were still mid-range (mean 0.33, max
0.53) — the multiplicative rule was working as designed (no saturation)
but had **no braking force**: once the whole population fires together,
LTD ∝ w is too weak to break the loop, and the runaway detector (correctly)
aborts the run.

**Amendment A1 (logged before re-run)**: the runaway detector is a safety
mechanism calibrated for the *additive* rule's operating point. Arm B's
hypothesis concerns assembly formation, not seizure resistance. For Arm B
only, the detector parameters are raised to `runaway_rate_hz = 399 × 1.5
= 600 Hz`, `runaway_sustained_ms` unchanged (5000). This is a *detector*
change, not an organism change — the rule and all learning parameters are
untouched. Logged as A1 to keep the run interpretable rather than
silently tuning until the verdict flips.

## Execution record — final (2026-09-13)

**Arm A** (`runs/e2a-20260913T082247Z`): completed, metrics in
`report.md` — serves as the additive-rule replication of E1.

**Arm B** (`runs/e2b-20260913T082426Z`): completed the curriculum but is
**scientifically invalid as an assembly test**. After amendment A1 raised
the runaway detector to 600 Hz, arm B entered a permanent population-wide
seizure at the first S1 onset: internal mean rate ≈ 400 Hz sustained for
the entire run, every internal neuron firing at every opportunity.
Snapshot analysis: cross-pattern cosine = 1.000 for all pairs (no response
profiles exist to separate), selectivity median = 0.0, retention ratio
1.00 (trivially — the network never stops firing). The multiplicative
LTD term (Δw⁻ ∝ a⁻·w) provides too little inhibition at low weights to
break the recurrent loop this organism's wiring produces.

**Verdict**: hypothesis NOT answered by this arm — the experiment is
dominated by network stability, not assembly specificity. Per the
protocol's scope guards, no further detector tuning was applied to force
a verdict. The finding itself is recorded: classic multiplicative STDP
without an inhibition/homeostasis mechanism is unstable in this organism
regime, which is itself evidence relevant to U2 and future experiment
design (an inhibition term or homeostatic mechanism becomes a
prerequisite for bound-variant comparisons).

**Status**: E2 closes as *inconclusive-by-instability* for Arm B; Arm A
replicates E1. The next experiment targeting the binding problem must
either introduce a stability mechanism first (E3 adaptation, U1 — the
pre-registered roadmap already places it next) or compare bounds only
after a stability mechanism exists.

## Resource note (logged)

Arm B telemetry reached 8.3 GB (vs arm A's ~0.6 GB): without saturation,
every synapse emits continuous weight-delta events for the full run — the
coalescing threshold (|Δw| > 0.01) fires constantly. Two consequences
logged for future experiments:
1. Report/metric regeneration must stream, not load, telemetry (the
   snapshot stream alone suffices for all scientific metrics — used here).
2. E3+ should reconsider the wire/telemetry emission budget for
   non-saturating rules (larger coalesce threshold or time-bucketed
   emission), or disk usage becomes the experiment's bottleneck.

## Storage stress result (v2 chunks)

Arm B seizure re-run on the v2 columnar store: 72,412,724 rows / 290
chunks → **159 MB** total (vs **8.3 GB** JSONL for the same run: ~52×
smaller). The pathological-run problem is solved: bounded 250k-row
chunks, no unbounded memory growth, and the 8.3 GB JSONL is never
written.

Note: report regeneration from an 8.3 GB legacy JSONL is what timed out
(>5 min); the v2 chunk path must stream — the current report
implementation materializes all envelopes, which is fine for ≤ a few
hundred MB but needs incremental aggregation for seizure-scale runs.
Tracked as a follow-up; the storage itself is fixed.

**Stress-run regeneration completed** (~25 min single-threaded,
nice 19): metrics.json + report.md produced from the 72M-row chunk
store. Arm B metrics confirm the seizure state: assembly score ≈ 0.002
(no separation), selectivity median 0.0 (n=98), retention 1.00
(trivial — the network never stops firing), zero Failure events (the
A1-recalibrated detector stayed quiet). These are the expected
degenerate values for a seizure state and validate the analysis path
end-to-end on pathological input.

**Follow-up logged for E3+**: incremental (non-materializing) report
aggregation, and reconsidering the emission budget for weight-delta
events under non-saturating rules — 48M weight events is the dominant
cost even in columnar form.
