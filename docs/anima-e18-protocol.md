# ANIMA E18 — Developmental capability: antecedent-dependent probe
# response (temporal state)

Status: **FROZEN** (2026-09-18). Design basis `1bf214e`;
user-approved decisions recorded in the appendix. Nothing under this
protocol is implemented or executed until a separate implementation
release. E17 closed, not reopened.

## 1. Question

Can the frozen ANIMA organism develop a *temporal state* — an
internal response to the same probe pattern B that encodes which
antecedent (A vs C) preceded it across a silent gap — with the
encoding *growing over developmental experience*?

Capability (single): antecedent-dependent probe response.

## 2. Approved decisions (user, 2026-09-18)

1. Probe: **B** (committed E12 pattern; phases 4-7 -> 8-11; all
   presentations SEQ by variant_block = 60 with < 60 reps).
2. Primary gap **G0 = 800 ms**; persistence gap **G1 = 1600 ms**.
3. **S4 contingency filter: EXCLUDED** from E18.
4. Shuffled/silence-probe control: **TRIGGERED ONLY IF** the
   canonical run shows a positive result (L - E > 0.05 p.D.) AND the
   drawn antecedent sequence audit fails (|lag-1 autocorrelation|
   > 0.3 or any 40-trial window imbalance worse than 60/40). The
   control arm then: same curriculum except antecedents placed in
   separated epochs (probe trials preceded only by silence);
   same seed; single run.
5. Endpoint thresholds: **pre-registered below** (section 7);
   never derived from observed results.
6. Canonical substrate: exact committed E6/E12 organism (configs/
   e12.toml organism, plasticity, structural, resources, v2, e6
   sections byte-identical; only the run id, block schedule, and
   variant_blocks differ as registered). Zero new mechanisms.
7. **Early -> mid -> late developmental divergence windows**
   registered (section 5) to distinguish acquired history dependence
   from a fixed/transient trace.

## 3. Environment, interfaces

- Environment: the committed deterministic schedule harness; new
  trial curriculum; no feedback; no labels; no reward.
- Sensory: 24 channels; antecedents A {0-7} and C {8-15} (500 ms,
  20 Hz, jitter 2.0 — committed definitions); probe B (500 ms,
  40 Hz two-phase, jitter 2.0 — committed definition); channels
  16-23 never active; silence elsewhere.
- Action/output: none — measurement-only (the "behavior" is the
  internal probe response; output neurons passive observables).
- Trial template (frozen, cadence = the E-series 2000 ms):
  antecedent 500 ms -> gap G -> probe B 500 ms -> ITI (2000 - 500 -
  G - 500); G0 block: gap 800, ITI 200; G1 block: gap 1600, ITI
  0. Trial length 2000 ms constant across all blocks (nothing about
  trial type or block is encoded in timing).

## 4. Curriculum

- Initial: fresh canonical organism, seed 20260912 (registered;
  cross-seed only by a later approved release).
- S1 development: 120 trials, G0, antecedent order = seeded
  balanced shuffle (exact 60 A / 60 C per S1; 20/20 per 40-trial
  window), Bern(0.5) draws from the environment's seeded RNG in
  registered order (per-trial draw, no replacement within windows).
- S2 measurement: 40 trials, G0, balanced (20/20).
- S3 persistence: 40 trials, G1, balanced (20/20).
- Total 200 trials; blocks contiguous; baseline silence 5000 ms
  before S1.
- Windows (frozen): early E = S1 trials 1-20; mid M = S1 trials
  51-70; late L = S1 trials 101-120; S2 = trials 121-160; S3 =
  161-200.
- Held constant: everything except accumulated experience and the
  registered gap in S3.

## 5. Measurements (existing machinery; per-presentation vectors
## from telemetry spike rows; snapshots; analyzer)

- Behavioral: per-presentation internal vectors of the probe epoch
  (500 ms window); pairwise-mean cosines: D(block) = 1 - mean over
  (B-after-A x B-after-C) trial pairs in the window (raw == L1
  expected); B-after-A/B-after-C alignment vs A and C (frozen
  argmin); A-C separation sanity (frozen 0.60 rule, A and C epochs
  of S2).
- Developmental: E, M, L, S2, S3 divergence values (section 7 for
  the registered criteria).
- Internal organization: E15 bucket totals by side at block
  boundaries (T0 = before trial 1; S1/S2/S3 boundaries); per-neuron
  selectivity (analyzer); permanence events per window.
- Stability: per-window mean + spread (raw); persistence: S3 vs S2.
- Resource: failures; snapshot internal-rate max; engagement
  (candidate-permanence events > 0 in S1); the exact frozen gate
  bars: zero failures, max rate <= 250 Hz, permanence events > 0.
- Activity description: B-after-A vs B-after-C spike-timing
  profiles (quartiles) — descriptive only.

## 6. Information-leakage verification (at execution)

- Config carries no labels; the antecedent identity reaches the
  organism ONLY as the antecedent's own channel activity.
- Timing is constant across trial types and (per-trial) blocks;
  balance is exact so frequency/order statistics cannot label
  trials.
- Analyzer trial-type labels are analysis-side; verified by a
  no-label assert on the schedule representation (environment
  schedules contain no trial-type fields).

## 7. Endpoint thresholds (PRE-REGISTERED; not derived)

Let D(X) denote the divergence in window X (section 5).

- Developed component: L - E.
- **Development criterion (positive)**: L - E > **0.05**.
- **Trace-baseline separation**: the developed component is the
  divergence above the early-window baseline; a claim of acquired
  history dependence requires L - E > 0.05 (and M between E and L
  reported; monotonicity claimed only if M - E > 0 and L - M > 0).
- **Persistence criterion**: S3 - E > **0.05** (the developed
  component survives G1); otherwise persistence absent (S3 - S2
  decay reported descriptively).
- A-C sanity: mean cos(A, C) < **0.60** in S2 (frozen 0.60 rule).
- All margins (0.05) are arbitrary-but-registered; 0.05 is one
  order of magnitude below the E-series effect sizes observed for
  such cosine differences (0.3-0.7), chosen to avoid noise claims
  without being derived from E18 data.

## 8. Success / partial / failure / inconclusive (qualitative +
## registered bars)

- SUCCESS: L - E > 0.05 AND S3 - E > 0.05 AND A-C < 0.60 AND gates
  clean; the probe response encodes the antecedent beyond the trace
  baseline and persists across the longer gap.
- PARTIAL: L - E > 0.05 but S3 - E <= 0.05 (developed but
  non-persistent); or divergence flat (L - E <= 0.05) with S2 vs S1
  differences unexplained; reported with the raw windows.
- FAILURE: no divergence at any window (L - E <= 0.05, S2 ~ E):
  the capability does not emerge under the canonical organism; a
  registered negative; no mechanism modification.
- INCONCLUSIVE: gates fail (failures, rates > 250 Hz, zero
  permanence engagement), telemetry/determinism faults, or window
  sizes insufficient (any window with < 15 trials of each
  antecedent after quality filtering).

## 9. Relation to E1-E17 (freeze summary)

Depends on: E9/E12 antecedent-order sensitivity, E14 per-presentation
machinery, E15 bucket instruments, E17's instant re-expression (the
trivial alternative the early-window baseline defeats). Provisional:
E6 phi and M6 residual trace hypotheses (mechanism-agnostic here).
Does not rely on: E13 verdicts, E16/E17 ablation outcomes as
mechanism claims. Progression: the endpoint is a developed
capability (behavior + persistence + growth), not a representation
property.

## 10. Verification plan (execution time)

1. Config hash pins (e18.toml) recorded pre-run.
2. Environment tests: trial grid (200 trials, 2000 ms each),
   balance (60/60 S1, 20/20 windows), gap/ITI arithmetic, timeline
   total, no trial-type labels in the schedule, deterministic
   antecedent sequence (same seed -> same sequence).
3. Determinism: same-seed rerun -> byte-identical telemetry.
4. Gates + isolation of measurement instruments (telemetry-only
   imports, static test).
5. Full suite green; 0 warnings; tree clean at freeze.

## Appendix: user decisions (2026-09-18, approved)

Probe B; G0 800 ms / G1 1600 ms; S4 excluded; shuffled control
triggered only on (positive result AND schedule-structure audit
failure); thresholds pre-registered here; canonical E6/E12
substrate; early/mid/late windows registered. Implementation and
execution require a separate release.

## Appendix: amendments

- (none yet)
---

## PRE-EXECUTION AMENDMENT A-1 (user-approved, 2026-09-18)

- ORIGINAL CONSTRAINT (protocol sections 3-4): "Trial cadence:
  2000 ms throughout"; trial template ITI = 2000 - 500 - G - 500.
- CONFLICT: G1 = 1600 ms forces ITI = -600 ms with a 2000 ms
  cadence; the approved G1 and the cadence constraint cannot both
  hold.
- SELECTED AMENDMENT: G1 stays 1600 ms. S1/S2 trials remain exactly
  2000 ms (antecedent 500 + gap 800 + probe 500 + ITI 200). S3
  trials are 2600 ms (antecedent 500 + gap 1600 + probe 500 + ITI
  0). No other protocol variable changes.
- RATIONALE: G1 = 1600 ms is the essential persistence manipulation
  (reducing it would change the approved question); the cadence
  constraint exists to prevent trial-type timing cues WITHIN blocks,
  which still holds (all trial types share one template per block).
- TIMING (exact): S1/S2 trial k: antecedent at T_k, probe B at
  T_k + 1300, next trial at T_k + 2000, T_0 = 5000, T_k = 5000 +
  2000k. S3 trial k (k = 161..200): antecedent at T_k, probe B at
  T_k + 2100, next trial at T_k + 2600, T_161 = 5000 + 320000.
  Timeline: 5000 + 120*2000 + 40*2000 + 40*2600 = 429,000 ms.
- RECORDED (not a protocol-variable change): B's registered
  variant_block is set to 100000 so all 200 probe presentations are
  the SEQ variant (byte-identical probe content after A and after
  C); the protocol's approved decision 6 authorizes variant_blocks
  divergence as registered.
