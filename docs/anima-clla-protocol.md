# ANIMA CLLA protocol — frozen bundle-level test (design freeze, NOT executed)

Status: FROZEN PROTOCOL, 2026-09-20. Approved for freezing from
the K-review of docs/x-assembly-arch.md. NOT implemented, NOT
executed, NOT tuned. No E-number until this protocol passes
integrity review and the implementation identity gate.
Architecture reference: docs/x-assembly-arch.md (as amended).

## 0. Scope and claim cap

SCOPE: Does CLLA as a COMPLETE ARCHITECTURE prevent synaptic
interference while preserving independently learned
representations?

NOT tested here: individual causal attribution of consolidation,
protection, or partition. No ablation arms. Report language is
restricted to "CLLA bundle supported / not supported" per
falsifier; no "consolidation caused X" style claims.

## 1. Frozen configuration

All arms at the e24 cell, everything identical to the committed
e24 configs EXCEPT the CLLA flag and the D-arm stimulus:

| setting | value |
|---|---|
| cell | β = 0.0046875, slow_state_tau_ms = 5000 (e24 cell) |
| seeds | 20260912, 424242, 9001 (frozen e24 seeds with committed single-pattern references) |
| organism | exactly configs/e24-s20260912-{a,c,il,bac,bca}.toml minus pattern lists |
| plasticity | a_plus=0.005, a_minus=0.0053, tau ± 20 ms, decay 1e-6 |
| M2 | t_e = 0.8, m2_buckets = 1 (baseline path) |
| M3 | c_slots=6, w_c_init=0.01, theta_permanent=0.05, w_c_permanent=0.02 |
| M5/M6/E6 | unchanged from e24 configs |
| CLLA flag | `assembly_protect = true` for test arms; false for identity arm |
| p_max_frac | **0.75** (frozen; protected cap = 0.75 × t_e = 0.60/neuron). Justification from committed references: single-pattern protected-mass needs at drive end are ~0.29 (A) + ~0.24 (C) = 0.53 total; 0.60 gives 0.07 margin for both to coexist, leaves ≥0.20 working budget; derived arithmetically from committed reference masses — NOT tuned. |
| w_consolidate_min | **0.05** = theta_permanent (existing constant, reused; no new number) |
| config invariants (asserted at run start) | w_c_permanent (0.02) ≥ silence_w (0.02); w_consolidate_min (0.05) ≥ theta_prune (0.005) |

## 2. Arm set (minimal — the e24 diagnostic, no expansion)

| arm | S1 curriculum | reps | CLLA | purpose |
|---|---|---|---|---|
| il | A/C interleaved | 40 each | true | F1/F2/F3/F4/F7 primary |
| bac | 20 A then 20 C | 20+20 | true | F1 blocked-overwrite, F7 |
| bca | 20 C then 20 A | 20+20 | true | F1 blocked-overwrite, F7 |
| d | D = channels 0–15 (A∪C), 20 Hz, 500 ms | 40 | true | F6 composition |
| ident | il curriculum (20260912 only) | 40+40 | **false** | identity gate |

Schedules: S0 5 s silence; S1 as above, cadence 2000 ms (500 ms
presentation + 1500 ms off); S2 20 s silence. D-arm stimulus:
pattern D with channel_ids = [0..15], rate 20 Hz, duration 500 ms,
jitter 2 ms — the ONLY new stimulus; no new organism mechanism.

Runs: 4 test arms × 3 seeds = 12 runs + 1 identity run = 13
executions. Nothing else.

## 3. Measurements (all read-only instruments on committed/recorded artifacts)

All metrics use the established N=52 internal/output neuron set
(ids 24..75), channel groups A = 0–7, C = 8–15 (committed config
fact). Drive-end = snapshot at t = 84,000 ms (post-40th-presentation
snapshot; the e24 convention from x-synmem). Snapshot cadence
1000 ticks (1 s). Telemetry: spike rows + presentation windows per
committed decoder.

### 3.1 Coexistence — EXACT DEFINITION

MEASURED VECTOR: per-neuron raw A-channel mass and C-channel mass:
  m_A[i]  = Σ w over live, excitatory, non-inhibitory synapses
            with pre ∈ {0..7} onto neuron i
  m_C[i]  = Σ w over live, excitatory, non-inhibitory synapses
            with pre ∈ {8..15} onto neuron i
at the drive-end snapshot (t = 84,000).

RAW mass (all live exc afferents, regardless of consolidated
flag) — because the single-pattern references are CLLA=false runs
and comparability requires the identical quantity; protected-mass
specifics are covered by F3/resource checks.

AGGREGATION: M_A = mean over 52 neurons of m_A[i]; M_C likewise.
Per seed, per arm.

REFERENCE (frozen, committed, never regenerated): for each seed,
the single-pattern references are the committed e24-s<seed>-a and
e24-s<seed>-c runs' drive-end masses, measured by this same
instrument (byte-identical to the committed analysis, x-synmem G1
values for 20260912: M_A_ref = 0.290, M_C_ref = 0.239).

COEXISTENCE CRITERION (F1 fail ⇒ bundle not supported):
  M_A(arm) ≥ 0.5 × M_A_ref(seed)  AND  M_C(arm) ≥ 0.5 × M_C_ref(seed)
applied to arms il, bac, bca. If the criterion fails in ANY of
the 3 arms × 3 seeds → F1 FAILS.

### 3.2 Response separation — EXACT DEFINITION

RESPONSE VECTOR: v_P(k) = 52-dim vector of spike counts during the
k-th presentation of pattern P, window = [t_pres, t_pres + 500)
(during-window; the established rateDrn representation).

COMPARISON: cross-class mean pairwise cosine over late
presentations:
  X = mean over k,k' ∈ {21..40} of cos(v_A(k), v_C(k'))
WITHIN-class means:
  W = mean over k≠k' of cos(v_A(k), v_A(k')) and same for C,
  then averaged over the two patterns.

F2 CRITERION (F2 fail ⇒ bundle not supported):
  X < 0.90 AND (mean within-class − X) ≥ 0.05
by presentation 20 (reps 21–40 define the measurement window).

EXCLUSIONS (frozen): any presentation window with all-zero vector
is excluded from the pair means; if excluded pairs exceed 25% of
all pairs in the arm, F2 is judged FAILED (uninformative). This
rule is registered, not arbitrary.

### 3.3 Protected mass cap — EXACT DEFINITION

P[i] = Σ w over live, excitatory, non-inhibitory, CONSOLIDATED
synapses onto neuron i. Measured at EVERY snapshot (1 s cadence).

F3 CRITERION (fail ⇒ bundle not supported):
  ∀ i, ∀ snapshot: P[i] ≤ 0.75 × t_e + ε,  ε = 0.04
(ε = 0.05 × t_e = the tolerance for post-consolidation LTP
overshoot documented in failure mode 8 of the architecture doc;
frozen, not tuned).

EQUALITY/EXHAUSTION handling: P[i] == cap is allowed (≤).
When P[i] is at cap, working budget W[i] target = t_e − P[i];
M2 normalizes working synapses only (factor = (t_e − P[i])/W[i]
when W[i] > 0); if W[i] = 0 the neuron's working set is left
alone. Capacity exhaustion is a REPORTED outcome (time series of
median headroom per arm), not by itself a falsifier — UNLESS it
manifests as F3b below.

F3b (premature exhaustion; fail ⇒ bundle not supported):
  median(P)/cap ≥ 0.99 AND median(W) ≤ 1e-6 at drive end
i.e. the organism has locked all capacity and can no longer learn
anything new at all.

### 3.4 Novel D composition — EXACT DEFINITION

RESPONSE BASES (frozen): v̄_A, v̄_C = mean during-window response
vectors over reps 21..40 of the SAME-SEED il ARM (CLLA=true,
same organism state as the d arm). This is the load-bearing
basis: protection can shift response dynamics, and comparing a
CLLA=true D response to CLLA=false committed vectors would fail
the cosine for reasons unrelated to composition. The committed
single-pattern (CLLA=false) vectors are reported as SECONDARY
diagnostics only, never as the F6 basis.

D RESPONSE: v̄_D = mean during-window response vector over reps
21..40 of the d arm.

COMPOSITION COEFFICIENTS (frozen, NOT fitted post-hoc): α = β = 1
in the vector-sum space — D presents channels 0–15 = A∪C at the
same 20 Hz/channel rate as A and C individually, so the expected
retained-structure response is the SUM of the A and C responses.
(This is the scale argument from existing state: D has exactly
twice A's input channels, and the retained structures are sums of
per-channel weights.)

F6 CRITERION (fail ⇒ bundle not supported):
  cos(v̄_D, v̄_A + v̄_C) ≥ 0.90
AND secondary diagnostics reported (not thresholds):
  cos(v̄_D, v̄_A), cos(v̄_D, v̄_C), residual norm
INTERPRETATION (frozen): composition-consistent D SUPPORTS
independently retained A/C structure. A D-specific response
unsupported by the A/C components is NOT automatically proof of
hidden labels — it is reported as evidence requiring further
investigation (E-gated), per the K-review mandate.

### 3.5 Blocked-order overwrite — EXACT (retained)

F1 already covers bac/bca coexistence. ADDITIONAL registered
overwrite diagnostic (reported, not a falsifier): M_A at bac
drive end vs 0.208 (the blocked A-collapse measured in the
unmodified substrate) — the bundle predicts M_A(bac) ≥ 0.145
(= 0.5 × 0.290, identical to F1's threshold when seed-matched;
the 20260912 committed collapse value is reported as context).

### 3.6 Protected-mass erosion (LTD leak) — F7

Definition: P_A[i] = Σ w over consolidated synapses with pre ∈
{0..7} onto neuron i; peak P_A over snapshots vs drive-end value.

F7 CRITERION: **F7 fails iff** drive-end total protected A-mass
  < 0.5 × peak protected A-mass
  (total = sum over all neurons)
i.e. only a greater-than-half within-pattern erosion of the
protected structure trips the falsifier. On F7 failure, the
pre-registered remedy is an A-series amendment proposing the
consolidated-LTD exemption (architecture doc failure mode 8); the
F7 result is reported WITH the leak, not excused.

## 4. Identity gate

Before any CLLA=true run: the ident arm (CLLA=false, il
curriculum, seed 20260912) must reproduce the committed baseline
EXACTLY:
- event-stream FNV-1a hash: 135,293-row identity anchor
  d452d028ffaec973 (committed E24/v23gate identity);
- snapshot artifact bytes / SHA-256 7e3ef343…24b6 (committed);
- RNG draw count (no extra draws when flag off);
- same-seed byte-identical telemetry (single-run determinism is
  a substrate guarantee).
CLLA=false must differ from committed baseline by ZERO bytes
(flag-off code path: consolidated flag never serialized, M2 path
untouched). If the identity gate fails → BLOCKED, report, do not
interpret any CLLA=true run causally.

## 5. Resource accounting (explicit checks, every run)

R1. P[i] ≤ cap + ε at every snapshot (F3).
R2. W[i] after each M2 window: W[i] ≤ (t_e − P[i]) + 1e-6
    (relative tolerance; "working mass correctly normalized
    after protected mass removed").
R3. Consolidated synapse count per neuron ≤ b_e (40) (M5
    invariant extended to the class).
R4. No hidden memory: Σ consolidated-synapse weights per neuron
    == P[i] (single-flag provenance check); count of live exc
    == count(unconsolidated) + count(consolidated).
R5. Config invariants asserted at run start (w_c_permanent ≥
    silence_w; w_consolidate_min ≥ theta_prune).
R6. RNG draw parity via identity gate (identity-run draws
    counted).

## 6. Falsifier summary (frozen)

NOTE ON CONVENTION: every row below states a PASS condition; a
falsifier FAILS iff its stated inequality/relation is violated
(e.g. F7 fails iff drive-end protected A-mass < 0.5 × peak;
F6 passes only while cos(v̄_D, v̄_A + v̄_C) ≥ 0.90).

| id | criterion | triggers |
|---|---|---|
| F1 | M_A ≥ 0.5·M_A_ref AND M_C ≥ 0.5·M_C_ref, arms {il,bac,bca} × 3 seeds | coexistence fails despite CLLA protection; blocked overwrite persists |
| F2 | X < 0.90 and within−X ≥ 0.05 over reps 21–40 (exclusions <25%) | response separation collapses by presentation 20 |
| F3 | P[i] ≤ 0.75·t_e + 0.04 at every snapshot, all neurons | protected mass exceeds capacity / resource violation |
| F3b | median(P)/cap ≥ 0.99 and median(W) ≤ 1e-6 at drive end | premature capacity exhaustion |
| F4 | P2 runaway abort (50 Hz/5 s) or any nonzero failure count | CLLA produces runaway/unstable dynamics |
| F5 | static design-conformance: consolidation/partition decision paths read no pattern-id, no channel-member test, no presentation count, no global state (verified by code review at implementation; recorded in the protocol run record) | CLLA becomes a hidden contextual address/label mechanism |
| F6 | cos(v̄_D, v̄_A + v̄_C) ≥ 0.90 | D not explained as composition of independently retained A/C |
| F7 | drive-end total protected A-mass ≥ 0.5 × peak | within-pattern LTD erodes protected structure (the registered leak) |

## 7. Verdict rule (frozen, binary)

CLLA bundle SUPPORTED ⇔ identity gate passes AND F1, F2, F3, F3b,
F4, F5, F6, F7 all pass in ALL 3 seeds × applicable arms.
Anything else ⇒ NOT SUPPORTED, with the failing falsifier(s) and
raw distributions reported (no post-hoc threshold changes, no
parameter sweep, no added mechanism). Secondary diagnostics
(coexistence time courses, headroom, blocked-context collapse,
D residuals) reported as context, never as verdict.

## 8. Execution order (frozen)

1. Register protocol (this document) + commit.
2. Implement CLLA (flag-off identity path first).
3. Identity gate: ident arm vs committed anchors (FAIL ⇒ BLOCKED).
4. Code-review F5 conformance (no hidden context) → record.
5. 12 CLLA=true runs, seeds/arms as frozen.
6. Read-only measurements per §3 (instruments to be committed;
   twin-verified where pairs exist).
7. Verdict per §7; report with the frozen claim language.

## 9. No E-number

E-number assignment happens only after this protocol passes
integrity review (phantom-reference regression, execution-record
integrity, identity gate evidence), per standing discipline.

STOP — protocol frozen. Do not implement, do not execute, do not
tune until the integrity review and your approval.