# Phase II-A frozen registration — D-core: experience-derived context tracks

Status: FROZEN PROTOCOL, 2026-09-22. Architectural authority:
docs/x-phase2-architecture.md (commit c52a7cb), candidates §8, resource
models §12, smallest distinguishing experiment §15, ladder §16.
NOT implemented, NOT executed, NOT tuned. No E-number assigned by this
registration; standing rule: E-number assignment happens only after this
protocol passes integrity review and the implementation identity gate.

## 0. Scope and claim cap

SCOPE: Does D-core (learned, local context tracks; K = 2) allow a
BLOCKED-order second pattern to form persistent protected synaptic
structure at the Phase I S1 bar, while preserving alternating
coexistence, under byte-identical identity discipline?

CLAIM CAP (frozen): this registration makes NO claim about retrieval or
re-expression. A passing S1 is interpreted ONLY as: "D-core can form
multiple persistent representational structures". Phase II-B (re-expression)
is explicitly NOT designed here and NOT executed; the II-B metric (§11) is
DECLARED as the handoff contract and nothing more.

PROHIBITED in the implementation: BAC/BCA/A/C labels as data; epoch or
phase parity as a key; trial counters; global similarity engines; reward;
external episode boundaries; any mechanism beyond D-core as specified
below (no extra gains, gates, or ablations).

## 1. The three arms

| arm | key policy | runs needed |
|---|---|---|
| 0: control (committed baseline) | none (plain CLLA-fe substrate) | NOTHING NEW — the 9 committed clla-fe runs (15:33 batch) ARE the arm; endpoints computed with the same instruments |
| 1: D-core (learned tracks) | per-neuron learned co-activity prototypes, K = 2 (§6–§7) | 9 runs: {bac, bca, il} × seeds {20260912, 424242, 9001} |
| 2: epoch-key active control | THE ARCHIVED V2.3-STYLE EPOCH PARITY KEY (m2_buckets=2, epoch 40 windows) — metadata partition, the study's rejected key policy; isolates "partition" from "learned key" | 9 runs, same matrix |

Total NEW runs: 18 experimental + 3 identity/control-path runs (below).
The matrix is NOT expandable by this registration.

## 2. Curricula (exact, cloned from the committed Phase I schedules)

Per-arm config = committed configs/clla-fe-s{seed}-{order}.toml with:
- exp_id renamed (clla-d2a-s{seed}-{order} / clla-d2b-s{seed}-{order});
- the D-core flag (arm 1) or the m2_buckets=2 partition fields (arm 2,
  exactly as the archived v23a3 configs);
- NOTHING else changed (stimulus, cadence, plasticity, M3/M4/M5/M6/E6,
  V2.1 cell β=0.0046875/τ_s=5000, seeds).

Schedules (committed facts): S0 5 s silence; S1 at 2 s cadence (500 ms
presentation @ 20 Hz on the pattern's 8 channels + 1500 ms off):
- bac: 20 × A then 20 × C;
- bca: 20 × C then 20 × A;
- il: 20 × A and 20 × C interleaved;
then S2 20 s silence.
Seeds: 20260912, 424242, 9001 (frozen Phase I seeds).

## 3. Identity / control path

- New config flag `d_core: bool` in V2Params (default false,
  skip-serialize-when-false). Flag OFF ⇒ every D-core branch skipped ⇒
  byte-identical committed behavior.
- New serialized state, all skip-serialized when flag-off/empty so
  flag-off artifacts stay byte-identical:
  - per-synapse `track: u8` (0/1; serde skip when 0);
  - per-neuron prototype vectors `ctx_protos: Vec<f32>` (2 × 24 =
    48 f32; serde skip when empty) — snapshots carry them in D-core
    runs (audited via committed read-only instruments).
- Identity gate (arm-1 AND arm-2 implementations, and the flag-off
  binary): the ident run must reproduce the committed anchor
  byte-for-byte — event-stream FNV-1a 9647ea8a0ca4dbd2 (152,254 rows,
  clla-ident-il lineage) and 105/105 snapshot frames identical to the
  committed flag-off baseline (arbitrated against the documented
  z_latch serialization artifact only). FAIL ⇒ BLOCKED, no arm-1/arm-2
  run interpreted.
- Identity RUNS: 3 executions — (i) ident-il with d_core OFF;
  (ii) ident-il with d_core ON at zero exposure (no input) ⇒ proves
  the machinery is inert without drive; (iii) ident-il with
  m2_buckets=2 partition fields set ⇒ the archived partition isolation
  control (v23 identity precedent).
- Same-seed determinism is a substrate guarantee (E1-era hash-equal
  duplicates); any divergence between identical re-runs blocks the
  batch.

## 4. D-core parameters (frozen; every value cited, NONE tunable)

| parameter | value | derivation (not tuning) |
|---|---|---|
| K (tracks/neuron) | 2 | architectural floor: K=2 is the only Phase-I-scale setting with writable sub-budgets (per-track cap 0.30 ≥ single-pattern need 0.24–0.29; K=4 ⇒ cap 0.15 < need). Study §12/§14 |
| θ_sim (similarity threshold) | 0.50 (cosine) | sits inside the measured separation band: within-presentation cosine 0.85–0.89 vs cross-presentation 0.15–0.40 (CLLA capability adjacent F2); θ at 0.5 classifies same/different context with margin on both sides |
| α_p (prototype soft-update rate) | 0.10 / window | p += α_p (x − p); ~88% convergence in 20 windows: >1 block of the 100 ms structural window cadence — converges within a 2 s presentation; NOT a new timescale (structural window used) |
| similarity metric | cosine on x_i(w) | standard, bounded, deterministic |
| per-track protected cap | p_max × t_e / K = 0.75 × 0.8 / 2 = 0.30 (+ε 0.04, the Phase I F3 tolerance) | protocol arithmetic; per-track clip at headroom (925146c machinery, per-track) |
| per-track working target | T_t = (t_e − P_tot) / n_populated, n_populated = # tracks with live working weight > 0 (min 1) | V2.3 capacity-matched formula family (`total ≤ t_e` ALWAYS); single-track behavior reduces to Phase I baseline |
| recruitment gain | rg8c form: i_boost = min(k_g·(1−R_t)·I_W_t, max(0,(v_th−v)·τ_m/dt)), k_g = 8.0 code-frozen; active ONLY on track t's working afferents when (1−R_t) > 0 | the two registered rg8/rg8c lessons embedded: input-proportional, threshold-completion capped, unexplained-track-only (per-track (1−R) = the validated CLLA rule signal); NOT a new mechanism — specified D-core component, study §8(3); unchanged mechanics, gate re-keyed per track |
| track-usage tie rule | lower track index wins | deterministic; pre-registered |
| w_consolidate_min, w_c_permanent, θ_perm, c_slots, decay, silence/mPrune | UNCHANGED Phase I constants | no new numbers |

## 5. Initialization

- Prototypes: all zeros at network construction (and at neuron
  creation — the growth machinery is unchanged but Phase II-A runs
  birth_trigger="none" like every Phase I-era run).
- Track tags: 0 for initial wiring (no context exists); the initial
  wiring is NOT exempted from its track-0 budget (M2 per-track
  normalization applies from the first window).
- Track budgets: P_t = 0, working shares computed by the §4 target
  formula from the first window.

## 6. Learned-prototype construction (exact)

For neuron i (internal/output, ids 24..75), every structural window w
(100 ms, the existing cadence):

1. During the window, the EXISTING input-current accumulation pass
   (structural_v2 accumulate_input_current, no-op flag-off) additionally
   accumulates, per input channel c with a live excitatory afferent to
   i, the delivered current x_i[c](w) = Σ (amp × w) over that channel's
   synapses onto i during w. Channels without afferents to i
   contribute 0. x_i is the neuron's own locally observed usage vector
   (24 dims). Zeroed at window start with the res accumulators.
2. At window end (inside window(), before M2):
   a. if total working input current delivered to i in w is 0 ⇒ no
      update (inert — covers silence and fully-protected neurons).
   b. s_t = cosine(x_i(w), p_i[t]) for t ∈ {0,1} (0 if ‖p_t‖ = 0).
   c. if max_t s_t ≥ θ_sim: context c* = argmax (tie ⇒ §4 rule);
      soft update p_i[c*] += α_p (x_i(w) − p_i[c*]); ALL of this
      window's plasticity-relevant events for neuron i are tagged
      track c* (synapses created in M3 and the LTP clip pass).
   d. else (novel usage vector): context c* = track with smaller
      protected mass P_i,c* (tie ⇒ §4 rule); HARD SET p_i[c*] :=
      x_i(w); tag the window's events track c*.
3. Prototype update consumes NO RNG (deterministic); no new
   timescale (structural window); x_i is curriculum statistics
   (co-occurrence), never group membership.

This is online k-means with soft updates, per neuron, on the neuron's
own delivered-current history — fully local, parameter-complete,
reproducible.

## 7. Track assignment / update semantics (exact)

- A synapse carrying track tag τ means: it belongs to context τ's
  budget. Tag assignment points (all inside the d_core flag):
  1. M3 candidate → permanence: newly created synapse gets the
     neuron's CURRENT window context tag (c* of §6 at that window).
  2. Initial wiring: tag 0.
  3. STDP: a synapse's tag NEVER changes after permanence; working
     synapses keep their creation tag (their M2 bucket is their tag).
- The tag is per-synapse state (u8), written at synapse creation for
  M3-born synapses only; dormant-candidate survival, eviction,
  M4 pruning operate on the synapse regardless of tag (unchanged
  machinery).
- LTP clip: per POST-NEURON (n, τ) track: headroom τ =
  cap_τ − P_{n,τ}, applied in the existing consolidated-LTP path
  (925146c clip re-keyed by the post-synapse tag; flag-off = today).
- M2 normalization: per (neuron, track): scaling the track's live
  working weight sum toward T_t (§4). Implemented as the V2.3
  per-bucket pass with bucket := track tag.
- M6/E6/STDP equations: unchanged.

## 8. Track budget / resource accounting (checkable invariants, every snapshot)

- R-track-1: per (neuron, track): P_{n,τ} ≤ 0.30 + 0.04 at every
  snapshot (F3 per track).
- R-track-2: per neuron: Σ_τ (working + protected) ≤ t_e + 1e-6 after
  every M2 window (V2.3 invariant, per-track targets sum to t_e − P_tot).
- R-track-3: Σ_τ P_{n,τ} ≤ 0.60 (the Phase I cap) — implied by
  R-track-1 but checked explicitly.
- R-track-4: prototype storage = 2 × 24 f32 per neuron (fixed),
  track tags = 1 byte per synapse, track budgets = the existing
  weight budget (no hidden memory; total weight units ≤ 0.8 × 52 =
  41.6 + P-total ≤ cap).
- R-track-5: Σ consolidated weight per neuron == Σ_τ P_{n,τ}
  (single-flag provenance, track-resolved).
- Exhaustion (reportable, NOT by itself a falsifier): a track at cap
  with θ_sim-unexplained input ⇒ §6(d) routes to the other track; both
  tracks at cap ⇒ no writable track (F3b-style state reported with the
  time series; if median min_τ headroom ≥ 0.99·cap AND working ≤ 1e-6
  at drive end ⇒ F3b-equivalent FAIL, per CLLA precedent §3.3).

## 9. Consolidation / protection semantics

- M3 permanence, w_c_permanent, θ_perm: UNCHANGED (per-neuron
  candidate machinery is not per-track — candidates are pre-protected
  substrate and remain context-free until permanence assigns the tag).
- Protected class: consolidated + tag τ ⇒ exempt from M2 (per the
  tag) and M4 and decay (Phase I semantics, per track).
- Per-track LTP clip (§7) replaces the per-neuron clip when d_core on;
  flag-off = the 925146c path byte-exact.
- P stability: the Phase I capability F7 (drive-end ≥ 0.5 × peak per
  track) is REPORTED, not a falsifier in this registration (no
  expectation set pre-hoc; the register keeps Phase I bars only where
  the question transfers).

## 10. Primary endpoint and criteria (frozen; Phase I bars verbatim)

PRIMARY (arm 1 only): the Phase I S1 criterion, exactly as frozen in
the rule/reserve/fe/fec registrations:
  second-block total protected mass ≥ 0.5 × first-block total
  protected mass, per cell; S1 PASS requires 6/6 cells
  (bac+bca × 3 seeds) in arm 1.
(Both blocks measured at their block ends: first at the S1 midpoint
snapshot, second at drive end t = 84,000 — the e24 convention;
instruments: the committed clla_traj-family mass counts, track-resolved
versions of the same quantities, twin-checked.)

SECONDARY (report; PASS/FAIL bars where stated):
- S-a coexistence preservation: arm-1 il raw drive-end A/C masses
  ≥ 0.5 × per-seed committed references (20260912: 0.290/0.239;
  424242: 0.249/0.221; 9001: 0.284/0.156) — the Phase I F1-il bar;
- S-b stability: failures = [] in every run; all curriculum-complete;
- S-c per-track cap invariants R-track-1..5 at every snapshot;
- S-d total track resource ≤ 41.6 + cap (R-track-4/5);
- S-e no hidden metadata (static conformance review, F5-style:
  routing/update/clip paths read only x_i, prototypes, P_τ, tags,
  caps — no channel-group test, no order/count/parity, no global
  state); recorded in the run record;
- S-f identity parity (§3) — mandatory before any arm-1/arm-2
  interpretation;
- S-g arm-2 (epoch key) reported with the SAME endpoints, as the
  key-policy control. Arm-2 outcomes do not vote on the primary.

CRITICAL NEGATIVE CONTROL (registered, not retrofitted): the
historical adjacent-state cosine 0.972→0.990 (x-synmem G3) is the
recorded ρ ≈ 0 baseline for the CURRENT substrate. It is NOT a target,
NOT a tuning anchor, and NOT part of arm-1 endpoints; it exists so the
eventual Phase II-B result is read against a fixed, pre-existing datum.

## 11. Phase II-B handoff (contract only; NOT designed, NOT executed)

If (and only if) S1 passes 6/6 in arm 1, with S-a..S-f satisfied:
- STOP the Phase II-A result as the frozen antecedent (preserve runs,
  commit verdict);
- the II-B registration, to be frozen separately, will satisfy AT
  LEAST: ρ(A) = cos(v_A_late, v_A_ref) − cos(v_A_late, v_C_ref) ≥ δ
  with δ = 0.10 (justification: measured discrimination band, study
  §15), v over the 52-dim spike-count vectors of the last 10
  presentations of a re-exposure block, all-zero-vector exclusions
  (< 25%, CLLA precedent), plus a C-retention check after the A
  block (ρ(C) ≥ 0.10 against learning-phase references).
- II-A itself does NOT run re-exposure blocks.

If S1 fails: STOP; report the exact failure mechanism from telemetry —
per cell: prototype trajectories (cosine between the two prototypes
and between each prototype and the pattern-usage vectors — the
BLUR metric), per-track protected mass curves, per-track working
shares (n_populated history, sub-budget-floor check §12), first-block
churn of the second cohort (prune counts, as in the rule record).
No reparameterization in this registration.

## 12. Failure modes registered a priori (all named, all local)

1. PROTO-BLUR: prototypes converge (cos(p_0, p_1) → 1) ⇒ routing
   becomes random-tie ⇒ per-track budgets matter as one budget
   (S1 fails with P ratio ≈ plain CLLA). Reportable via the blur
   metric.
2. KEY-SWAP on novelty: §6(d) hard-set repeatedly flips the novel
   pattern between tracks (both tracks' P tie at low values) ⇒
   fragmentation (measured: track-tag entropy of the second block's
   protected cohort).
3. SUB-BUDGET FLOOR: with both tracks populated, per-track working
   target (0.8−P)/2 < the second pattern's writable need ⇒
   recruitment starvation inside the track (fec 0.054 symptom at
   track level).
4. FIRST-BLOCK CAPTURE: the first block protects to 0.30 on both
   tracks of its neurons (n_populated=1 ⇒ unbounded working share
   until the second pattern arrives) ⇒ the second pattern finds no
   headroom (E24 primacy signature at track level).
5. GAIN INTERACTION: the rg8c-capped gain fires inside an
   unexplained track during the second block; if the track's working
   substrate is already dead (churn), I_W_t = 0 ⇒ boost = 0
   (the rg8 lesson re-enters — expected and reportable, not a new
   mechanism).
6. Operating-point overdrive if a track's first-exposure drive is
   strong (rg8 4b lesson): the threshold-completion cap is the
   registered bound; violations of the 4b-style band are reported
   with the pres-1..5 burst means (E3-band reference, not a
   falsifier here).

## 13. Execution order (frozen)

1. THIS protocol committed (this commit). STOP for integrity review.
2. Implement D-core behind `d_core` flag (arm 2 reuses archived v23
   machinery). Implementation commit; unit tests: routing determinism,
   prototype soft/hard update math, per-track clip, per-track M2
   target formula, tie rules, flag-off identity branch coverage.
3. Full test suite + identity gate (3 runs per §3) — separate commit
   with evidence.
4. Static conformance review (S-e) — recorded.
5. Experimental matrix: 18 runs (arm 1 + arm 2 × 3 orders × 3 seeds)
   — execution commit; all runs preserved incl. any abort.
6. Read-only measurement (§10, track-resolved instruments) — twin
   where pairs exist — measurement commit.
7. Verdict per §10/§11 and STOP.

No post-hoc endpoint changes; no parameter changes after results; no
expansion of the matrix; Phase I records immutable; tree clean at every
commit boundary.

STOP — protocol frozen. Do not implement, do not execute, do not tune
until the integrity review and your approval.