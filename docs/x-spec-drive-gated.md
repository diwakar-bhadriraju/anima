# X-series: Drive-Gated Slow-State Write — Frozen Exploratory Spec

Status: FROZEN SPEC, 2026-09-20. Implemented; identity gate
PASSED; diagnostic NOT executed (ambiguity STOP, see §8).

PROTOCOL CORRECTION (2026-09-20, recorded before execution, user-
approved resolution of §8): the earlier statement 'β unchanged
(0.00625)' was erroneous. The corrected baseline is the actual
V2.3 gate baseline — beta = 0.0046875, m2_buckets = 1, decay =
1e-6, a_minus = 0.0053 — NOT the A3 partition arm (m2_buckets=2,
decay=0). A3 remains a historical retention anchor only. Seeds:
{20260912, 424242, 9001, 123456}. Curricula: BOTH blocked orders
{bac, bca}. This is a protocol correction, not a post-hoc
experimental adjustment; no parameter was retuned.
Exploratory (no E-number, no promotion). V2/V2.1/V2.2/V2.3 and
all E-numbered records untouched.

## 1. Problem statement (evidence)

The organism forms sensory-specific organization during
stimulation (during-window cohort cosines 0.30–0.89 across
seeds, E24-era runs) but the persistent state u carries no
sensory information off-window (off-window cosines 0.999+;
V2.1 endogenous activity = deterministic pacemaking, Fano≈0).
Cause hypothesis: the per-spike write `u += β` is cause-
agnostic — a pacemaking spike deposits exactly as much u as a
sensory-driven spike. The slow state therefore integrates
"firing history", not "sensory history".

## 2. Intervention (entire mechanism)

Gate the per-spike slow-state write by a strictly local fast
afferent-drive trace: u accumulates only in proportion to
concurrent sensory drive reaching the neuron.

Scope fence (implemented, nothing more): no consolidation
synapses, no recurrent attractors, no structural growth; V2/V2.1/
V2.2/V2.3 untouched; no parameter tuning.

## 3. Frozen mechanism (exact)

### 3.1 Afferent drive input

```
I_aff_i(t) = Σ amplitude·w_{j→i} over excitatory synapses where
             presynaptic j is an INPUT CHANNEL and j fired at t
```

- Input-channel → this-neuron excitatory synapses only.
- Recurrent (internal→internal) and inhibitory excluded.
- Input-vs-internal = architectural boundary (D8), not a label.
- **SPEC CORRECTION (2026-09-20, user-approved, pre-execution):
  the membrane receives `amplitude * w` per spike
  (network.rs deposit: `current = s.amplitude * s.w`); the
  original spec (and the failed 16-run diagnostic, commit
  2923a90) summed bare `w`, a 52x scale error documented in
  docs/x-mechanism-review.md. Corrected definition uses the
  frozen substrate constants amplitude = 52.0 and v_th = 1.0
  (LIF threshold).


### 3.2 Drive trace (per neuron, fast, deterministic)

```
x_i(t) = min( I_aff_i(t) / v_th, 1.0 )      # v_th = 1.0 (LIF threshold,
                                            #   existing frozen constant;
                                            #   SPEC CORRECTION: t_e was a
                                            #   dimensionless M2 budget, not
                                            #   a membrane-current scale)
λ_g    = exp( -dt / τ_g ), dt = 1 ms, τ_g = 20 ms (= STDP τ)
g_i(t+1) = λ_g · g_i(t) + (1 - λ_g) · x_i(t)   # g_i ∈ [0,1], g_i(0)=0
```

- EMA bound [0,1] by construction; steady state = sustained
  input fraction; no scale parameter.
- `g` is organism state: serialized into snapshots (only when
  gate on); deterministic replay.

### 3.3 Gated write (the experimental change)

flag off (default): `on spike:  u_i += β`   — exactly as V2.1
flag on:              `on spike:  u_i += β · g_i(t_spike)`

- REPLACES the ungated write; no additive term.
- t_spike semantics per the recurrence: the write uses g_i held
  DURING tick t (before that tick's x_i is folded in), i.e.
  g_i(t) as defined by g(t+1) = λg + (1−λ)x.
- Scope: exactly the neuron population that receives the
  per-spike write today (non-input, non-retired spikers).
- Pacemaking spikes (g≈0) consolidate ~nothing; sensory-driven
  spikes consolidate in proportion to drive.
- β, τ_s, decay, dv injection: unchanged. Latch (V2.2) fields
  untouched; if latch_enable additionally set, gate applies to
  the same write.
- `g` is READ-ONLY w.r.t. synapses: it influences only the u
  write; never STDP/M2/M3/M4/M5/M6/E6; excluded from M2 and
  resource accounting (not a weight).
- V2.1 β=0 identity preserved: gate on with β=0 ⇒ no writes at
  all ⇒ V2 identity.

## 4. Parameters (zero new free parameters)

| param | value | source |
|---|---|---|
| t_e | — | removed from the drive definition (spec correction) |
| v_th | 1.0 | existing LIF threshold (frozen) |
| τ_g | 20 ms | existing STDP tau (tau_plus) |
| β | 0.0046875 (protocol-corrected V2.3 gate baseline; the 0.00625 statement was erroneous) | existing slow_state_beta |
| τ_s | unchanged | existing slow_state_tau_ms |
| dt | 1 ms | substrate tick |
| g(0) | 0.0 | frozen init |

No new config parameter besides the boolean
`slow_state_beta_drive` (default false ⇒ byte-identical path).

## 5. Charter justification (for the record)

`g_i` is computed from the neuron's own afferent excitatory
current. Source-type (input channel vs internal neuron) is an
architectural boundary fixed by the substrate (D8), not
curriculum knowledge, not a label, not reward, not error, not
global population activity. The write law is a local
consolidation rule: spikes consolidate into slow state in
proportion to coincident sensory drive. No global variables;
deterministic; no hidden supervision.

## 6. Identity gate (executed, PASSED)

Mandatory, before any execution of the diagnostic.

- Suite test `x_drive_identity_gate` (crates/anima-exp/src/
  env.rs): runs the committed V2.3 gate config
  (configs/v23gate-s20260912-bac.toml, m2_buckets=1) with
  slow_state_beta_drive=false; asserts:
  1. event-stream FNV-1a == 0xd452d028ffaec973 and row count
     == 135,293 (byte identity with V2.3 baseline),
  2. snapshots.bin.zst SHA-256 == 7e3ef343…24b6 (the committed
     V2.3 gate run's file hash),
  3. g_drive omitted from snapshot serialization when flag off
     (skip_serializing_if = None yields byte-identical files).
  Result: PASS (test green).
- Suite test `x_drive_trace_inert_when_off`: mechanism-level —
  flag off ⇒ g_drive ≡ 0.0 for all neurons; flag on ⇒ g
  accumulates; gated u ≤ ungated u elementwise. PASS.
- Suites: anima-core 70 + anima-exp 55+2 green, 0 failed.

## 7. Minimal diagnostic protocol (frozen, NOT executed)

- Arms: (A) baseline config, flag OFF; (B) identical config,
  flag ON. One variable differs.
- Curriculum: one A/C pair — reuse from V2.3 A3/A2 runs
  (blocked: 20×first pattern then 20×second, 2 s cadence).
- Seeds: 4 existing seeds (see §8 ambiguity).
- Timing: E18/E20/E21-style window/off-window logic. Off-window
  read = max(existing off-window definition, stimulus offset +
  200 ms). At τ_g=20 ms, 200 ms ⇒ g decayed by e^−10, so
  off-window effects are provably in u, not the drive trace.
  (Check reported in instrumentation.)

### Pre-registered verdict rules (per seed; exact)

- Definitions: c_dur = during-window cosine(A,C) same-run;
  c_off_exp = off-window cosine(A,C) experimental arm;
  ‖u‖_off = off-window persistent-state norm.
- 1 NORM GUARD: `‖u_exp‖_off < 0.1 × ‖u_base‖_off` ⇒
  NO_PERSISTENT_STATE; no cosine reported as a result.
- 2 STRONG: c_off_exp ≤ c_dur (full sensory separation in
  persistent state).
- 3 PARTIAL: c_dur < c_off_exp ≤ 0.90.
- 4 NULL: c_off_exp > 0.90.
- 5 RUNAWAY: any abort; endogenous rate at refractory ceiling;
  or ‖u‖ growing through the off-window.
- GO: ≥ 2/4 seeds at PARTIAL or better AND zero RUNAWAY. A
  single strong seed is NOT a go (E24 seed-dominance lesson).
- Anything else: STOP and report.

### Instrumentation (report all)

- u vectors at stimulus offset and at off-window read (both
  arms); g at offset and at read (proves trace decayed);
  during-window cosine(A,C); off-window cosine(A,C) — primary
  endpoint (scale-invariant); off-window distance(A,C);
  off-window u norms per arm; norm ratio A/C;
  endogenous off-window firing rate + Fano;
  top-u neuron overlap Jaccard (top-10) between A/C;
  >200 ms information survival check; decoder SECONDARY/optional
  (no decoder claim on n=8 samples).

### Falsifier

NOT "more spikes". The question: does the persistent state
carry sensory-specific information after the stimulus is gone?
More endogenous activity with cosine ≈ 1.0 = NULL. Discrimin-
ability via trivial global amplitude difference does not count
(cosine scale-invariance + norm ratio handle this).

## 8. AMBIGUITY STOP — RESOLVED (2026-09-20, user ruling; spec corrected above)

The fence requires: "If any parameter below is ambiguous
despite this spec, STOP and report the ambiguity before
executing." Found during implementation verification:

**Conflict 1 — β.** The spec text freezes "β unchanged
(0.00625)". The V2.3 A3/A2 runs (the defined baseline and
curriculum source, §7) were executed at **β = 0.0046875**
(E24 band): configs/v23a3-*.toml and sded-am53-d0-*.toml all
carry slow_state_beta = 0.0046875 (verified in-repo). The
V2.3-era diagnostic base also differs in decay (0.0 in
v23a3/sded-am53-d0 vs 1e-6 in the v23gate identity config),
a_minus (0.0053 both), and m2_buckets (2 in v23a3 vs 1 in
v23gate). "β unchanged (0.00625)" vs "identical config to the
V2.3 A3/A2 runs" cannot both hold.

**Conflict 2 — seeds.** "4 existing seeds" is not a subset
specification. The frozen set is six {20260912, 424242, 9001,
123456, 777, 31337}; the A3/A2 anchors use all six. Which 4?

**Conflict 3 — curriculum.** "one A/C pair, reuse from the
V2.3 A3/A2 runs": those runs are pairs {bac, bca} (blocked
both orders). "One pair" = bac? bca? both (8 arms/seed pair)?
Not specified.

**RESOLVED (user ruling, verbatim intent):** baseline = truthful
V2.3 gate config (β = 0.0046875, m2_buckets = 1, decay = 1e-6);
A3 (m2_buckets=2, decay=0) NOT combined with this test (A3 =
orthogonal retention intervention; historical anchor only);
seeds exactly {20260912, 424242, 9001, 123456}; curricula both
blocked orders {bac, bca}; arms: slow_state_beta_drive false vs
true; 4 × 2 × 2 = 16 runs. Rationale (user): isolates the
write-path intervention without confounding with the tested
retention mechanisms.

## 9. Scope guard

- Implemented mechanism is exactly the gated write; nothing
  else changed (diff: network.rs step+fields, structural.rs
  birth init, recorder neuron field, harness snapshot + config
  wiring + provenance, config.rs field, env tests).
- No tuning anywhere; no post-hoc thresholds; all verdict rules
  frozen above.

STOP at diagnostic execution boundary pending user resolution
of §8.

---

# X-DIAGNOSTIC EXECUTION RECORD (2026-09-20)

16 runs (4 seeds x 2 curricula x 2 arms), V2.3 gate baseline
(beta=0.0046875, m2_buckets=1, decay=1e-6), configs
configs/xdg-{base,exp}-{bac,bca}-s{seed}.toml, instrument
examples/xdg_an.rs. Two run batches exist on disk: the FIRST
batch (15:19Z) had the flag appended AFTER [e6] (TOMl table
mismatch -> flag ignored -> arms identical) and is preserved as
a config-generation defect, NOT data. The corrected second batch
(15:22-24Z) below is the diagnostic.

## 1. Run IDs / status (corrected batch)

base: xdg-base-{bac,bca}-s{20260912,424242,9001,123456}-20260920T15225x-335Z — all
curriculum-complete.
exp: xdg-exp-bac-* all complete;
     xdg-exp-bca-s20260912-20260920T152405Z — **failure:runaway-activity**;
     remaining exp-bca complete.

## 2. Identity gate: PASS (committed suite tests x_drive_identity_gate
FNV d452d028ffaec973 / 135293 rows / snapshot SHA 7e3ef343...)

## 3-4. Cosines — NOT REPORTED as results (norm guard rule; see note)
During and off-window cosines computed but per pre-registered rule
NO_PERSISTENT_STATE forbids reporting any cosine as a finding. For
the record, computed values: baseline c_dur 0.85-0.94, c_off
0.22-0.99; experimental c_dur 0.77-0.97, c_off 0.50-0.99 with
u_norm ~0.01-0.02. Descriptive only.

## 5-8. Key measurements

| cell | base ||u||_off | exp ||u||_off | exp rate/Fano | exp g@read |
|---|---|---|---|---|---|
| bac 20260912 | 3.97 | 0.01 | 0.0 sp/s, Fano 0.00 | 0.00000 |
| bac 424242 | 3.47 | 0.02 | 0.0, 0.00 | 0.00000 |
| bac 9001 | 5.14 | 0.01 | 0.0, 0.00 | 0.00000 |
| bac 123456 | 9.16 | 0.01 | 0.0, 0.00 | 0.00000 |
| bca 20260912 | 14.60 | 0.01 | (abort) | 0.00000 |
| bca 424242 | 3.05 | 0.02 | 0.0, 0.00 | 0.00000 |
| bca 9001 | 2.18 | 0.02 | 0.0, 0.00 | 0.00000 |
| bca 123456 | 14.15 | 0.02 | 0.0, 0.00 | 0.00000 |

- g at offset and read: 0.00000 in all exp runs at snapshot
  resolution (1 s) — the 20 ms trace is dead at any snapshot
  time (provably beyond 10*tau_g); off-window effects cannot be
  attributed to g.
- top-u overlap: n/a per norm-guard rule (no persistent state).
- Endogenous firing collapsed in all exp runs (rate 0.0,
  Fano 0.00): NO pacemaking survived; the gate erased the
  persistent regime entirely.

## 9-10. Verdicts per cell and GO

ALL 8 exp cells: NO_PERSISTENT_STATE (||u_exp||_off < 0.1 x
||u_base||_off in every cell); 1 RUNAWAY (bca 20260912).
GO rule: >=2/4 seeds PARTIAL+ AND zero RUNAWAY — **FAILED
(0/4 seeds; 1 RUNAWAY)**.

## 11. EXACT VERDICT

**NO_PERSISTENT_STATE — the drive-gated write, at the frozen
parameters (t_e normalization x = min(I_aff/t_e, 1) with
weights ~0.02-0.06 and 20 Hz/500 ms drive => x ~ 10^-3), gates
u to ~0.4% of the ungated level: the persistent state does not
exist at this operating point; therefore the question of whether
it carries sensory information is UNANSWERABLE under this
parameterization and the diagnostic stops per its own rules.**

Mechanism note (observation, not tuned): with beta normalized by
t_e=0.8 and real afferent sums ~0.001-0.01, x_i(t) is typically
~10^-3 during drive, g ~ 10^-3, so u accumulates ~0.1% per
spike. The gating is functionally an OFF switch at this
curriculum's drive strength. Whether a different normalization
(no t_e division; or drive-per-spike weighting) would restore a
persistent-but-selective u is a MECHANISM QUESTION — requires
user decision; NOT executed under frozen rules (no tuning).

Preserved: all runs incl. the 1 abort and the 8 defective-batch
runs (marked in this record as non-data). No E-number. No
progression to Alternative A/B. STOP.
