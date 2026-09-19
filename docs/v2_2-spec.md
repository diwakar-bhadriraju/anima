# ANIMA V2.2 — FROZEN ARCHITECTURAL SPECIFICATION (candidate latch substrate)

Status: FROZEN SPEC, 2026-09-20, pre-implementation. Promoted from
the approved candidate direction (G1+G2+Y1 slate). V2.1 remains
implemented and unchanged; all historical results (E1–E24, X1–X3,
V2.1 A/B) retained verbatim. Nothing in this document is executed.

---

## 0. Bundling decision and justification

**Chosen intervention: G1+G2+Y1 (bistable latch + per-neuron
heterogeneity + local reset). Minimal smaller bundles are NOT
scientifically useful; larger ones are confounded. Justification:**

- **G1 alone is untestable as a carrier.** With homogeneous
  neurons, the latch threshold sits at one value for all 52
  neurons; the X3/E24 evidence says the population's u margins
  under a fixed curriculum span the threshold *stochastically by
  seed* (0-to-9,961 spread at identical curriculum). A
  homogeneous-latch experiment would measure threshold-crossing
  lottery again, with a different nonlinearity. Heterogeneity
  (G2) is what converts the single knife-edge into a *band of
  crossings* distributed across the population — the code becomes
  "which subset crossed", which is the quantity the whole E24
  analysis says must carry the information. Without G2, G1's
  falsification test cannot distinguish "latch fails" from
  "threshold placement failed".
- **G1 without Y1 is unwritable.** A bistable element with no
  reset has exactly one transition available (unlatched→latched)
  and no return path; curricula differing in *later* blocks
  (bac/bca arms) could not be distinguished even in principle,
  and entrenchment is permanent. Y1 (local, self-spike-triggered
  decay boost) is the minimal return path that keeps the rule
  strictly local (no global variables, no input from other
  neurons' states beyond what synapses already carry).
- **Y1 without G1 is meaningless** (nothing to reset) and G2
  without G1 changes only time constants of a linear integrator
  (X3 already maps that regime; no new capability).
- Therefore the smallest intervention whose success/failure is
  *attributable* is the triple, WITH the discrimination plan
  (§8) that internally separates components via null sub-settings
  (β=0 identity, heterogeneity-only arm, latch-off arm) rather
  than via further bundling.

Charter note: this is a Category-C mechanism change, approved by
the user as a candidate direction on 2026-09-20 (this exchange).
The binding rule — evidence-tagged candidates with predictions +
smallest distinguishing experiment BEFORE implementation — is
satisfied by the approved slate message and this frozen spec.

## 1. Exact local dynamics (per neuron i, per tick)

V2.1 code path (network.rs:575-592, unchanged semantics):

```
i_syn  *= exp(-dt/tau_syn)
i_adapt *= exp(-dt/tau_adapt)
u_slow *= exp(-dt/tau_s)          # decay-then-read
v      += (-(v - v_rest) + i_syn + i_ext - i_adapt + u_slow) * dt / tau_m
if v >= v_th and past refractory:
    v = v_reset; i_adapt += adaptation_gain
    u_slow += beta                                  # V2.1 increment-after-spike
```

V2.2 replaces the u_slow line and the increment with a
two-variable local state (u_i, z_i) per neuron:

```
# decay (identical to V2.1):
u_i *= exp(-dt/tau_s_i)            # tau_s_i heterogenous (G2)

# G1 latch gating (per-tick, BEFORE the membrane read):
if z_i == 0 and u_i >= theta_i:  z_i = 1            # SET: crossing latches
if z_i == 1 and u_i < phi_i:     z_i = 0            # RESET: decay drops below release

# membrane read (u_slow position now reads the EFFECTIVE drive):
u_eff_i = u_i + z_i * U_plateau_i                   # plateau adds to u, not replaces

... membrane update identical, using u_eff_i in place of u_slow ...

if spiked:
    v = v_reset; i_adapt += adaptation_gain
    u_i += beta                                       # V2.1 increment unchanged
    # Y1: reset semantics — self-spike-driven decay boost:
    u_i -= eta                                        # local subtraction on spike
```

### 1.1 State variables

- `u_slow: f32` (existing field; now "sub-threshold
  integrator").
- `z: u8` (new; 0 = unlatched, 1 = latched). One byte per
  neuron; serialized in snapshots as `z_latch: Option<u8>`
  (serde default None = pre-V2.2 files).

### 1.2 Update order (frozen; one tick)

1. decay i_syn, i_adapt, u_slow (per-neuron tau where enabled)
2. latch gate evaluation (SET uses decayed u; RESET uses decayed u)
3. compute u_eff = u_slow + z·U_plateau
4. membrane update + spike decision (uses u_eff)
5. on spike: v/i_adapt updates exactly as V2.1; then
   u_slow += beta; then u_slow −= eta (Y1), floored at 0
6. structural-birth site (if a birth occurs this tick) draws
   heterogeneity params for the new neuron from the same seeded
   distributions (G2 extends to born neurons; RNG stream
   documented in §5)

### 1.3 Parameters (config, all default = V2.1 identity)

| field | default | meaning |
|---|---|---|
| `latch_enable` | false | master switch (false ⇒ V2.1 exactly) |
| `theta_rel_mean`, `theta_rel_sd` | 2.0, 0.0 | SET threshold θ_i = u_reg · θ_rel_i, u_reg = beta/(1−exp(−1000/τ_s)) per-neuron regeneration equilibrium; θ_rel_i ~ LogNormal(mean, sd) (G2) |
| `u_plateau_rel_mean`, `u_plateau_rel_sd` | 0.9, 0.0 | plateau amplitude U_i = u_reg · U_rel_i, U_rel_i ~ LogNormal (G2) |
| `tau_het_rel_sd` | 0.0 (off) | τ_s_i = τ_s · T_i, T_i ~ LogNormal(1, sd) (G2, default off to limit first-stage moving parts) |
| `phi_rel` | 0.5 | RESET release threshold φ_i = θ_i · φ_rel (latch releases when u decays below half θ) |
| `eta_rel` | 0.0 (off) | Y1 strength η = β · η_rel; per-spike subtraction |
| `latch_tau_ms` | 0 (off) | optional plateau self-decay (reserved; default off — plateau persistence is u-maintenance, not a timer) |

Defaults are the identity nulls; a V2.2 run sets them
explicitly. LogNormal draws are per-neuron, at construction, in
neuron-id order, from the network's existing Xoshiro stream
AFTER all V2/V2.1 construction draws — recorded so β=0 runs are
byte-identical to V2.1 (draws skipped entirely when
heterogeneity sd=0; no RNG consumption at identity ⇒ identity is
bit-exact, not merely distributionally equal).

## 2. Parameter distributions (G2)

- θ_rel_i, U_rel_i, T_i ~ LogNormal(μ, σ) with μ = ln(mean)−σ²/2
  (median-preserving parameterization so "mean" is the stated
  multiplicative mean).
- Drawn once per neuron at construction (and at structural
  birth, §1.2 step 6), deterministic given seed.
- First-stage defaults: θ_rel (2.0, 0.35), U_rel (0.9, 0.35),
  τ-het off. Rationale: θ mean 2× regeneration equilibrium sits
  inside the X3-measured band structure (silent margins ≤0.92,
  pacemaker ≥6 in units of u_reg≈β·τ/1000·... exact normalization
  per §1.3); sd 0.35 spans the band without exploding it.

## 3. Reset semantics (Y1) — exact

- On each spike of neuron i: u_i ← max(0, u_i − η), η = β·η_rel.
- SET/RESET gate unchanged by spiking (gate reads decayed u only).
- No global variables, no neighbor state, no input-identity
  terms: strictly local. (Charter: no hidden supervision.)
- Purpose: writable memory — strong self-firing erodes its own
  integrator, bounding plateau tenure and preventing permanent
  entrenchment. At η_rel=0 Y1 is off (pure G1+G2).

## 4. Resource/budget implications

- State: +1 byte/neuron (z). No new synapses; M2/M5 budgets
  untouched; M3/M4/M6 untouched.
- Compute: per-tick adds two comparisons + one FMA per neuron
  (negligible vs synapse loop).
- Structural births: heterogeneity draws per birth add no budget
  pressure (≤4 births/window unchanged).
- Telemetry: `u_slow` continues to serialize; `z_latch` added
  (schema-additive; readers default None). Snapshot size +~52 B.

## 5. Determinism/provenance

- All draws from the existing per-network Xoshiro stream,
  appended strictly after V2/V2.1 construction draws, in neuron
  order (θ, then U, then T per neuron). Draw order recorded in
  this spec = implementation contract; RNG state is part of the
  determinism guarantee (same seed ⇒ identical draws).
- Identity (all sd=0 / enable=false): no draws consumed; runs
  byte-identical to V2.1 at equal config (verified in Stage-A-
  style gate, §7).

## 6. Identity behavior at the null setting

- `latch_enable=false` ⇒ code path short-circuits to V2.1
  exactly (u_eff ≡ u_slow, no gate, no η). Zero new draws.
- `latch_enable=true, sd=0` ⇒ homogeneous latch (G1 only
  arm).
- Full V2.2 = enable + sd>0 (+η_rel>0 for Y1).
- The four regimes (V2.1 / G1 / G1+G2 / G1+G2+Y1) are all
  reachable from config alone — this is the discrimination
  instrument (§8).

## 7. Stability/runaway criteria

- Existing P2 runaway detector unchanged (50 Hz mean internal,
  5 s) — aborts remain outcomes, never suppressed.
- Latch-specific recorded instabilities (observed, not
  corrected): (a) mass-latching cascade (z=1 fraction > 0.8
  sustained 5 s) — recorded as `latch-saturation` observation in
  metrics, non-fatal; (b) plateau-driven refractory locking (a
  latched neuron at ceiling) — visible as pacemaker class; (c)
  η runaway (η ≥ β·(1+θ_rel) makes u monotonically decreasing
  under firing — self-extinguishing, benign).
- Success requires the carrier to live INSIDE the existing gate
  structure (zero P2 failures at the validated operating point).

## 8. What constitutes a successful carrier (frozen criteria)

V2.2 (full setting) at the E24 operating point (β=0.0046875,
τ=5000) and curriculum set, 6 seeds:

- C-S1 CAPACITY: cross-validated curriculum decoding from the
  drive-end latch-set vector {z_i} beats exact chance (5 arms ⇒
  20%; pre-registered test: exact binomial on 30 runs, one-sided
  α=0.05, leave-one-seed-out pooling).
- C-S2 STABILITY: the latch-set at drive end persists ≥ 2 s into
  silence with ≥ 80% identity overlap (set intersection/union).
- C-S3 DYNAMIC RANGE: silence max per-neuron rate < 300 Hz in
  ≥ 5/6 seeds (no refractory-clock winners).
- C-S4 NO NEW INSTABILITY: zero P2 aborts and latch-saturation
  observations in ≤ 1/6 seeds.
- C-S5 IDENTITY: the §6 null reproduces V2.1 byte-identically
  (100k-tick telemetry hash match, as Stage A).

All five must hold for "V2.2 is a successful carrier".
Partial results are reported as partial (e.g., capacity without
stability = writable but volatile code).

## 9. Falsifiers of the latch hypothesis (what would kill it)

- F1 Latch-set carries no curriculum information (C-S1 fails at
  chance exactly): the distributed-threshold code is not written
  by drive — falsifies the G2 premise that drive recruits
  distinguishable crossing subsets.
- F2 Latch-set is curriculum-informative but volatile (C-S1
  passes, C-S2 fails): the code exists but decays within the
  readout window — falsifies "u-maintained plateau" persistence;
  points to needing the reserved latch_tau (timer) instead.
- F3 Stable but saturated (C-S2 passes, C-S3 fails at ceiling
  rates): the latch trades the u-race for a clock again —
  falsifies the "bounded plateau preserves dynamic range"
  claim specifically (the info-audit's deficit 2 stands).
- F4 Homogeneous arm ≥ heterogeneous arm on C-S1: G2 was the
  wrong premise; heterogeneity hurts (fragmented code) — the
  bundling justification collapses and G1 alone is the candidate.
- F5 Y1-on worse than Y1-off on C-S1 AND C-S2: reset erases more
  signal than entrenchment costs — Y1 rejected, G1+G2 retained.
- F6 Identity failure: implementation bug, not science (blocker,
  fix before any run counts).

## 10. Experimental discrimination plan (structure, not executed)

Stage 1 — identity gate (C-S5) on a V2.1 replica config.
Stage 2 — 4-arm substrate discrimination at the E24 cell
(β=0.0046875, τ=5000), 6 seeds × 5 curricula × 4 substrate arms
{V2.1-null, G1, G1+G2, G1+G2+Y1} = 120 runs, endpoints C-S1–C-S4
computed per substrate arm; the BETWEEN-arm comparisons ARE the
component attribution (V2.1 arm = E24's own data re-used as
baseline where configs match; G1 vs G1+G2 isolates G2; +Y1 vs
G1+G2 isolates Y1). This is exploratory-disciplined (no
E-number) until a discriminating outcome emerges; promotion to
E25+ follows the standing rule (sufficiently interesting,
discriminating, reproducible ⇒ preregister with frozen
endpoints).
Stage 3 — only on Stage-2 success: E21 gap-sweep paradigm and
E24 curriculum paradigm re-run under V2.2 (capacity remeasured
with the carrier present; frozen before execution).

Attribution logic (frozen): each component's causal claim is the
signed difference between its arm and its parent arm on the
pre-specified criterion set (C-S1..C-S4), same seeds, paired.
Ambiguous/attribution-null outcomes are recorded as such — no
post-hoc decomposition.

## 11. Scope guards

- V2.1 code path, config surface, and historical runs: untouched
  (V2.2 is additive fields + additive branch).
- Historical Stage-C record: untouched.
- No labels, no reward, no global state, no input-identity
  variables anywhere in the mechanism (charter audit of §1–§3:
  every term is a function of the neuron's own state and its own
  spikes only).
- Implementation may not alter M1–M6, U1, E6, or the detector.
