# CLLA local allocation rule — frozen design (no implementation yet)

Status: DESIGN FREEZE, 2026-09-21. Read-only analysis complete
(docs/x-clla-allocation-audit.md + resratio measurements).
No implementation, no runs, no tuning, no E-number.

---

## 1. Exact local allocation variable

R_i(t) = protected-input-current fraction of neuron i during an M3
window:

    R_i = Ip_i / (Ip_i + Iw_i)

where, per post neuron i, summed over the current structural window
(during presentations):

    Ip_i = Σ amplitude · w_s       over fired input-channel afferents s
                                   with consolidated(s) = true
    Iw_i = the same over fired afferents with consolidated(s) = false

- Units: dimensionless fraction in [0, 1] (zero-input windows: R
  defined as 1 — fully explained by absence, no allocation).
- Boundedness: Ip, Iw ≥ 0 ⇒ R ∈ [0, 1] by construction.
- State: per-neuron running window sums of delivered input current
  split by the existing `consolidated` flag at delivery time
  (the delivery loop already sums total input current; the split
  is a second accumulator, one float per neuron, updated at the
  same site).
- Parameter: NONE. R needs no threshold, no gain, no constant.
- Distinguishes new vs stronger-old: STRONGER-OLD scales both Ip
  and Iw of the same cohort proportionally (same channels, same
  weights) ⇒ R invariant to presentation rate. NEW pattern: Ip
  for the neuron's own protected cohort drops to ~0 while Iw of
  the arriving cohort > 0 ⇒ R → 0. Measured: bac switch
  R-med 0.959 → 0.000; il late 1.000.
- R = 0 behavior: maximal unexplained drive ⇒ maximal allocation
  priority (subject to headroom).
- H = 0 behavior (H = cap − P): the rule multiplies by a headroom
  factor; H = 0 ⇒ no allocation, as today (cap entry gate).
- Endogenous silence behavior: no input current ⇒ R := 1
  (explained-by-absence) ⇒ no allocation during silence.
- Per-neuron independent: Ip_i/Iw_i are purely local sums over the
  neuron's own incoming; no cross-neuron, global, or population
  quantity.

Existing substrate already delivers this: the membrane accumulates
per-tick input current; only the protected/unprotected split is
new (a second local accumulator, not a mechanism).

## 2. Exact decision rule (acts ONLY on candidate permanence)

At each M3 structural window, for each neuron i, let

    H_i = cap − P_i          (existing headroom, cap = 0.75·t_e)
    R_i = Ip_i/(Ip_i+Iw_i)   as above

Allocation priority for THIS window's candidate accumulation:

    g_i = (1 − R_i) · sgn⁺(H_i)

where sgn⁺(H_i) = 1 if H_i > 0 else 0 (H_i ≤ 0 → g_i = 0).

M3 permanence accumulation per active candidate is then scaled:

    Δperm_eff = Δperm · g_i

applied at the existing co-active accumulation site
(structural_v2.rs candidate_pass):
    pool[i].w += delta_perm · beta · g_i   (co-active branch)
    pool[i].w *= decay_c                   (else-branch: unchanged)

ALL OTHER M3 mechanics unchanged: θ_permanent, θ_die, w_c_permanent,
w_consolidate_min, draw/redraw policies, eviction order.

## 3. Exact interaction with M3 permanence

- The gate operates BEFORE the permanence threshold: candidates
  accumulate slower when g_i < 1, so fewer reach θ_permanent per
  window; zero when g_i = 0 (fully-explained or no-headroom).
- It does NOT gate the permanence event itself at threshold-time;
  it slows the rate of approach (smooth, no hard boundary).
- The cap-entry check (P + w ≤ cap) at permanence time is
  UNCHANGED — the rule makes it reachable/unreachable by the rate,
  not by a new rejection path.
- Consequence for blocked BAC: first block drives R from 0 → ~1,
  decelerating its own accumulation as it explains the neuron
  (before consuming all headroom); when C arrives, R → 0 on that
  neuron giving C's candidates near-full Δperm priority into the
  remaining H, instead of starving at residual headroom.

## 4. Proof of no label/address introduction

The rule reads only: per-neuron delivered input current split by
the existing boolean flag `consolidated`; per-neuron P, cap, H —
all present in the neuron's own incoming synapse list. No pattern
id, no channel membership test, no presentation counter, no order
statistic, no global/population average, no similarity, no
context. The rule cannot name A/C/BAC/BCA — it reacts to
"protected synapses explain this input or not", which is the
audit's local mismatch signal. No new state beyond the split
accumulator (one f32/neuron, not serialized; flag-off path
unchanged). No new parameter.

## 5. F2/F6 protocol-window amendment (recorded; NOT executed)

Problem: frozen capability windows (per-pattern reps 21–40) are
empty — the il curriculum carries only 20 A + 20 C presentations
(verified by prescnt scan). The protocol never executed these
windows formally; the capability record marked F2/F6 unmeasurable.

Minimal amendment (frozen, replaces reps 21–40 with reps 11–20):
- F2 (§3.2): measurement window becomes reps {11..20} of each
  pattern, same inequality (cross < 0.90 AND within−cross ≥ 0.05);
  exclusions rule unchanged.
- F6 (§3.4): same-seed CLLA=true bases use the il-arm reps
  {11..20}; D-arm basis stays reps {11..20} (D has 40 reps; the
  amendment uses the same 11–20 window for symmetry and to keep
  both bases at matched repetition depth — recorded as the frozen
  choice).
- Scope note: the amendment applies ONLY to future executions of
  this capability protocol. It does not retract the capability
  record's F2/F6-unmeasurable status; it does not alter the old
  CLLA verdict (NOT SUPPORTED stands on F1).

## 6. Minimal execution matrix (frozen)

Config: e24 cell exactly as the bounded-protection CLLA runs
(beta 0.0046875, tau 5000, m2_buckets=1, seeds {20260912, 424242,
9001}, same timings), one new config flag enabling the rule
(default off; flag-off byte-identical, identity gate first).

RUN MATRIX (frozen; 13 executions total — count corrected 2026-09-21):

  1  identity (clla-ident-il, rule OFF) — byte-identical FNV
     9647ea8a0ca4dbd2 (152,254 rows) + 105/105 snapshot frames
  3  bac × seeds {20260912, 424242, 9001} (rule ON) — PRIMARY
     allocation endpoint
  3  bca × seeds {20260912, 424242, 9001} (rule ON) — PRIMARY
     (order mirror)
  3  il × seeds {20260912, 424242, 9001} (rule ON) — coexistence
     intact check
  3  d × seeds {20260912, 424242, 9001} (rule ON) — INFORMATIVE
     ONLY: reported separately; never evidence for or against the
     allocator; no effect on any endpoint or the verdict; same
     informative-only status as the previous CLLA protocol (§0 of
     anima-clla-protocol.md). The d-arm first-presentation >50 Hz
     crossing occurs before the cap engages and no flag-off D
     reference exists — D-completion/stability must not be cited
     for or against the allocation rule.
  --
 13  total (1 + 3 + 3 + 3 + 3)

All runs preserved incl. failures. No other arms, no other changes.

## 7. Exact success/failure criteria (frozen, no post-hoc edits)

Primary allocation endpoint (per seed, arms bac/bca, rule ON):
    second-block protected mass ≥ 0.5 × first-block protected mass
    at drive end (protected mass = Σ w over live consolidated
    synapses whose pre ∈ second-block channels; measured from the
    committed snapshot at t = 84,000, same instrument as before).
    PASS = 6/6 blocked arms.

Alternating intactness (arms il, rule ON):
    F1-il: A-mass ≥ 0.5·A_ref AND C-mass ≥ 0.5·C_ref per seed
    (raw drive-end channel masses vs committed e24 single-pattern
    references; same frozen criterion) — 3/3.
    F7: drive-end protected A-mass ≥ 0.5 × peak — 3/3.
    F3: P ≤ 0.75·t_e + 1e-6 every snapshot — 3/3.

Safety (all rule-ON runs):
    F4: 0 runaway/failure events (RunEnded telemetry).
    F5: static conformance — rule reads only local quantities
    (reviewed; split-accumulator code inspection).
    No new resource/failure class; no new serialized state
    (accumulator is transient, flag-off unaffected).

BUNDLE (this experiment only): supported iff identity gate AND
6/6 primary AND 3/3 il-intact AND safety all pass.

## 8. Expected failure modes

- First-block under-allocation: R reaches ~1 before the first
  block saturates → blocked first block ends below the 0.5
  reference; primary endpoint compares WITHIN-arm (second ≥ 0.5 ×
  first in the SAME run), so this masks only if both end low —
  still passes the endpoint but both are weaker; alternation F1
  then binds against the committed references. Reported, not
  tuned.
- R measurement lag: R uses the previous window's current sums at
  the presentation start (snapshot at t0); first window of a new
  pattern has R computed from a mixed window. The measured 0.959
  → 0.000 switch is already at 1-presentation resolution; no
  mechanism change needed.
- g_i = 0 hysteresis: a fully-explained neuron (R=1) with H>0
  (e.g., il late, H=0.09) allocates nothing — desired; but a
  neuron oscillating near R=1 may briefly reject a genuinely new
  cohort that arrives with residual old-cohort current. The
  blocked regime's exact-zero cross-cohort current (IA=0/IC=0)
  prevents this in the tested curricula; flagged.
- Zero-input windows redefine R=1 → no allocation in silence
  (desired, matches audit expectations).
- The split accumulator's window-sum semantics must match the M3
  window cadence exactly (window_ticks=100); off-by-one windows
  would phase-shift R by ≤1 window — harmless to the endpoint
  (established by unit test at implementation).

## 9. What this experiment answers (and does not)

Answers ONLY: can a local allocator prevent first-arrival
starvation of a later distinct configuration? No memory-capability
claim from this experiment alone; no E-number; no implementation
unless the freeze is approved.

STOP — design frozen; no code written, no runs made.