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


---

# V2.2 STAGE-1 EXECUTION RECORD + SPEC DEFECT (2026-09-19)

Implementation: per frozen spec (network.rs u/z dynamics, G2
LogNormal draws appended post-wiring + at birth, config surface,
z_latch telemetry field). All suites green (core 70, exp 55).

## Stage 1 — identity gate (C-S5)

- v22gate-v21replica (latch_enable ABSENT => false): 511,333
  non-marker event rows, FNV fingerprint e4b018992c97d1a9 —
  IDENTICAL to committed runs/v21probe-b0.00625-t5000-*
  (functional-identity standard, V2.1 Stage A precedent).
  **C-S5 PASS.**
- Diagnostic arm v22gate-latchid (enable=true, sd=0): diverges
  from V2.1 by design (gate active). PRESERVED as run.

## SPEC DEFECT FOUND (blocking Stage 2)

The frozen u_reg formula is dimensionally wrong. Spec §1.3:
u_reg = beta / (1 - exp(-1000/tau_s)) = 0.00625/0.1813 = 0.0345
— this is the equilibrium at ONE spike per second. Actual u
margins accumulate at firing-rate f to u* = beta·f·tau_s/1000
(f=100 Hz => 3.125). With theta = theta_rel_mean·u_reg = 2·0.0345
= 0.069, every driven neuron latches within ~1 s of drive onset
(observed in v22gate-latchid: immediate all-latch, divergence
from V2.1). Stage 2 executed with this constant would test a
degenerate all-latch gate — its F1 failure would be an artifact
of the constant, not evidence about the latch hypothesis. The
spec's own rationale ("theta sits inside the X3-measured band",
margins 0.5–15) is inconsistent with its formula; the formula is
the bug, the rationale is the intent.

## Proposed amendment A-1 (NOT applied; requires user approval)

Replace u_reg := beta / (1 - exp(-1000/tau_s)) with
u_reg := beta · f_ref · tau_s / 1000, f_ref = 100 Hz (the
pre-existing E3 rate-calibration band [100,200] Hz midpoint —
chosen from E1–E3 historical calibration, NOT tuned to X3/E24
results). All frozen multipliers unchanged (theta_rel_mean 2.0,
U_rel 0.9, phi_rel 0.5, sds 0.35). At the Stage-2 cell:
u_reg = 3.125, theta = 6.25, phi = 3.125, plateau = 2.81.
Threshold then sits at the sparse-core/pacemaker boundary of the
X3 map, matching the spec's stated rationale.

Per the standing mandate ("do not alter thresholds... after
execution begins"), Stage 2 is HALTED pending A-1 decision.
Nothing else changed; no E-number; V2.1/history untouched.


---

# A-1 FREEZE + STAGE-2 INTEGRITY CHECK (2026-09-20, pre-execution)

1. AMENDED FORMULA RECORDED: u_reg = beta * f_ref * tau_s / 1000,
   f_ref = 100 Hz. Units: [u] = [u/spike]*[spikes/s]*[s]. Applied in
   network.rs (const F_REF_HZ = 100.0) with citation comment.
2. SOURCE: f_ref = 100 Hz is the midpoint of the pre-registered
   E3 rate-calibration band [100,200] Hz (docs/e3-protocol.md
   lines 38, 86-89; A2 calibration logged pre-execution there;
   measured EMA ~148 Hz at gain 0.05 inside band). Pre-V2.2
   historical record; not tuned to X3/E24/V2.2 data.
3. RESULTING STAGE-2 VALUES + DISCREPANCY SURFACED: the approval
   message quotes theta=6.25 / phi=3.125 / plateau=2.81, which
   are the values at beta=0.00625 (wedge cell). The FROZEN
   Stage-2 cell is the E24 cell beta=0.0046875, tau=5000, where
   the amended formula gives EXACTLY:
     u_reg = 0.0046875*100*5 = 2.34375
     theta = 2.0 * u_reg = 4.6875
     phi   = 0.5 * theta  = 2.34375
     plateau = 0.9 * u_reg = 2.109375
   "Execute the original 120-run Stage 2" with "all other
   parameters exactly as frozen" is arithmetically satisfiable
   ONLY at the frozen cell; executing at beta=0.00625 would
   relocate the design cell (an alteration). Decision: formula at
   frozen cell; discrepancy recorded here rather than silently
   resolved either way. theta=4.6875 sits at the sparse-core/
   pacemaker boundary of the X3 margin map (sparse 1.2-8.1,
   silent <=0.92, pacemaker >=6) per the spec's placement
   rationale.
4. NO STAGE-2 RUN PRE-EXECUTED: runs/ contains only v22gate-*
   (2 gate runs). Verified by listing.
5. STAGE-1 UNCHANGED: v22gate-v21replica fingerprint re-verified
   post-A-1 code change: 511333 rows, fnv e4b018992c97d1a9
   (A-1 edits only the latch_enable=true branch; null arm
   untouched). The halted diagnostic arm v22gate-latchid is
   preserved as a FAILED SPECIFICATION DIAGNOSTIC (pre-A-1
   constant), NOT Stage-2 data; excluded from all Stage-2
   analyses.
6. AMENDMENT FROZEN PRE-OBSERVATION: no Stage-2 result exists at
   freeze time (this commit precedes the first v22s2-* run).
   Y1-ON VALUE PRE-DECLARED: the frozen spec left eta_rel's
   on-value unspecified; fixed at eta_rel = 1.0 (identity
   relation: each spike subtracts exactly its own increment; the
   only value introducing no free constant), declared here
   before any Stage-2 run.
7. ARMS: a0 = null (latch absent => V2.1 exactly; C-S5-proven
   path), a1 = G1 (enable, sds 0), a2 = G1+G2 (+theta/U sd 0.35,
   tau-het off), a3 = G1+G2+Y1 (+eta_rel 1.0). 6 seeds x 5
   curricula x 4 arms = 120 fresh runs under one binary.


---

# V2.2 STAGE-2 EXECUTION RECORD (2026-09-20, A-1 amended, commit 8e63f5e freeze)

120 runs (runs/v22s2-{a0,a1,a2,a3}-s{seed}-{cur}-*), arms:
a0 null (V2.1-exact; M values byte-match the E24 table — second
independent identity confirmation), a1 = G1 (enable, sd=0),
a2 = G1+G2 (+theta/U sd 0.35), a3 = G1+G2+Y1 (+eta_rel 1.0,
pre-declared). All runs preserved incl. aborts.

## Criteria results

- C-S5 identity: PASS (Stage 1 + a0-arm byte-match of E24 Ms).
- C-S1 decoding (LOSO nearest-centroid on drive-end latch-set,
  exact binomial vs 20% chance): a1 6/30 = 0.200, p=0.572;
  a2 7/30 = 0.233, p=0.393; a3 6/30 = 0.200, p=0.572.
  **ALL AT CHANCE — FAIL.**
- C-S2 stability (Jaccard >= 0.8 over 2 s): a1 30/30, a2 29/29,
  a3 26/26 — trivially PASS (see forensics: sets are tiny/empty,
  stability of an empty/near-empty set is vacuous).
- C-S3 dynamic range (<300 Hz in >= 5/6 seeds): a0 22/30, a1
  20/30, a2 18/29, a3 26/26 — **FAIL for a1/a2 (a3 vacuous:
  nothing latches, nothing fires)**. Latching did NOT fix the
  refractory-clock regime; a3's Y1 kills latching entirely.
- C-S4 no new instability: a1 0/30 aborts, a2 1/30, a3 4/30 —
  marginal pass on aborts (a3's aborts are drive-period
  runaway, same class as X3's).

**VERDICT (frozen rule C-S1..C-S5 conjunctive): V2.2 as
specified is NOT a successful carrier.** F1 fired (latch-set
carries no curriculum information — all arms at chance) and F3's
antecedent holds (a1/a2 keep ceiling-rate winners).

## Forensics (observations, mechanistic)

- FO-1 Latch sets are TINY: a1 sizes {0:4, 1:9, 2:14, 3:2, 5:1};
  a2 {0:14, 1:6, 2:6, 3:3, 4:1}. theta=4.6875 sits above typical
  drive-end u for most neurons (26/30 a1 runs have exactly the
  single top-u neuron crossing; a2's LogNormal spread RAISES many
  thresholds, halving latching: 14/29). The latch code as placed
  is ~1-2 bits per run, seed-determined.
- FO-2 Within-seed Jaccard across curricula 0.174 (a1): sets
  shared across curricula at the same seed swamp the
  curriculum-specific part; cross-seed same-curriculum 0.022.
  The seed dominates the latch-set — the same failure E24's
  magnitude endpoint had, now at the bit level.
- FO-3 Y1 at eta_rel=1.0 exactly cancels the per-spike increment:
  a3 latches NOTHING in 30/30 runs (u strictly non-increasing
  under firing). The identity choice was faithful to "no free
  constant" but makes eta=beta a degenerate off-state — the
  reset rule as frozen is non-viable at its own default.
- FO-4 a0's C-S3 22/30 shows the <300 Hz criterion was already
  marginal for V2.1 itself at this cell (not a regression from
  V2.2).

## Falsifier mapping (frozen)

- F1 CONFIRMED (all arms chance decoding).
- F2 not reached (C-S1 failed first).
- F3 CONFIRMED-antecedent for a1/a2 (ceiling rates persist).
- F4 not applicable (a1 vs a2 both chance; homogeneity vs
  heterogeneity indistinguishable at these set sizes).
- F5 CONFIRMED (Y1-on strictly worse: no latching at all).
- F6 n/a (identity held throughout).

## Conclusion (within frozen scope)

The bundled G1+G2+Y1 at the frozen constants is falsified as a
carrier: the SET threshold placement (theta at the sparse/
pacemaker boundary) yields 1-2-bit, seed-dominated latch codes,
and the frozen reset default is degenerate. Per the frozen
falsifier semantics this is a NEGATIVE RESULT, retained in full.
The bundling decision itself (§0 of the spec) partially collapses:
F4 could not discriminate G2's contribution because the code
capacity was ~0 regardless. Stage 3 NOT executed (success gate
failed). No E-number; V2.1 and history untouched.
