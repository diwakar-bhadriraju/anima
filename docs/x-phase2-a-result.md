# Phase II-A execution record — D-core verdict: S1 FAIL 6/6, mechanism reported, STOP

Status: EXECUTION + VERDICT RECORD, 2026-09-22. Protocol:
docs/x-phase2-a-protocol.md (frozen cf768bc). Implementation + identity
gate: commit 607f43f. All runs preserved in runs/ (including the
abort). No post-hoc tuning, no parameter changes, no reparameterization,
no E-number.

## 0. Identity / integrity (all four verification items recorded)

1. M1 SYNAPSE TRACK SEMANTICS — VERIFIED: every pre-existing M1
   synapse carries the documented default track 0 (unit
   dcore_m1_synapses_have_documented_default_track; flag-off never
   writes track). Documented consequence: the never-presented
   pattern's pre-existing working substrate is assigned to track 0's
   budget from birth (see §3 — this is the measured failure site).
2. EMPTY-TRACK BOOTSTRAP — VERIFIED: deterministic, RNG-free
   (ctx_update consumes no RNG; verified by unit
   dcore_bootstrap_deterministic_lower_track across seeds; first
   novel bind = track 0 by the tie rule).
3. PROTOTYPE/TRACK CONSISTENCY — VERIFIED: the single `track`
   identity keys prototype routing, M3 tag, M2 bucket, protection
   clip, and recruitment gate (units
   dcore_track_identity_consistent_at_permanence +
   dcore_per_track_m2_target_invariant; per-track targets
   capacity-matched, total ≤ t_e).
4. IDENTITY GATE — PASS:
   (i) flag-off ident rerun (runs/clla-ident-il-20260922T084155Z)
       byte-identical to the 153305Z anchor (telemetry chunks,
       snapshots, metrics);
   (ii) zero-exposure d_core ON vs OFF (runs/clla-d2a-ident-sil-
        on/off-20260922T0842..252Z): event-stream FNV identical
        (2567529e41283d1f, 2,532 rows), snapshots byte-identical;
        only byte-difference = the config-echo run-params envelope
        carrying the flags (metadata, documented);
   (iii) arm-2 partition ident run curriculum-complete (the archived
         V2.3 normalize path; see §4 for the CLLA-precedence finding);
   suites: anima-core 98, workspace 98+47+11+13+5 green; static
   conformance S-e scan clean (no channel-group/order/parity/trial
   reads in d_core paths).

## 1. Run matrix (all 18 preserved)

Arm 1 (D-core; clla-d2a-s{seed}-{order}, d_core + recruit_gain):
- s20260912: bac 084433Z, bca 084439Z, il 084445Z — curriculum-complete
- s424242:  bac 084452Z, bca 084458Z, il 084504Z — curriculum-complete
- s9001:    bac 084510Z — **failure:runaway-activity t=24,017**
            (mean 63.2 Hz > 50 Hz for 5,000 ms; S1 pop mean 164.2 Hz),
            bca 084512Z, il 084518Z — complete
Arm 2 (epoch-key control; clla-d2b-s{seed}-{order}): all 9
curriculum-complete (084523Z–084604Z).

## 2. Endpoints (frozen bars)

PRIMARY S1 (arm 1; second-block protected ≥ 0.5 × first-block):

| cell | first | second | ratio | verdict |
|---|---|---|---|---|
| s20260912-bac | 22.44 | 0.86 | 0.038 | FAIL |
| s20260912-bca | 24.76 | 1.15 | 0.046 | FAIL |
| s424242-bac | 20.71 | 0.00 | 0.000 | FAIL |
| s424242-bca | 25.59 | 0.00 | 0.000 | FAIL |
| s9001-bac | abort | — | — | FAIL (unmeasurable) |
| s9001-bca | 23.50 | 0.00 | 0.000 | FAIL |

**S1: FAIL 6/6.**

SECONDARY:
- S-a (F1-il, raw drive-end masses/neuron): s20260912 A 0.103 FAIL /
  C 0.176 PASS; s424242 A 0.134 PASS / C 0.219 PASS; s9001 A 0.097
  FAIL / C 0.192 PASS ⇒ **2/3 FAIL (A-side below 0.5× reference)**.
- S-b stability: **FAIL** (1 runaway; all others failures = []).
- S-c per-track caps: PASS 9/9 (T0 max 0.300–0.340 ≤ 0.34; T1 max
  0.300–0.313; PER-NEURON TOTAL PROTECTED = 0.600 exactly in every
  run — the per-track caps compose to the Phase I cap, R-track-1..3
  hold).
- S-d total track resource: PASS (protected totals 24–28 < 41.6+cap;
  prototype storage 48 f32/neuron fixed; no hidden memory).
- S-e no hidden metadata: PASS (static scan + code review).
- S-f identity parity: PASS (§0).
- S-g arm-2: **INERT — see §4**.

## 3. Failure mechanism (measured, per cell)

1. THE LEARNED KEY WORKS. Prototype-pair cosine at drive end
   (BLUR): 0.24–0.43 across all nine arm-1 runs — well below
   θ_sim = 0.5, far below the within-class band (0.85–0.99); no
   prototype collapse (failure mode #1 NOT observed); no key-swap
   fragmentation (#2 not observed: per-track max protected 0.30–0.34
   on both tracks, consistent occupancy).
2. THE BLOCKED-ORDER FAILURE PERSISTS AT THE SUBSTRATE, at the exact
   site Phase I identified: the late pattern's pre-existing M1
   working afferents carry the DOCUMENTED default track 0 (§0.1), so
   they live in track 0's budget and are churned by track-0
   normalization and M4 during block 1 exactly as measured in Phase I
   (rule record wC 197→77; fec 29→11). Track 1 — where second-block
   permanence binds (via cur_ctx, verified) — never receives the
   working substrate that would feed its LTP: second-block protected
   mass 0.00–1.15 vs first-block 20.7–25.6. This is failure-mode #4
   (first-block capture) in its exact structural form: the M1
   default-tag assignment places the never-presented pattern's
   substrate inside the first context's budget, and no local signal
   can know in advance that it belongs elsewhere.
3. SUB-BUDGET FLOOR at track level (#3 observed): when both tracks
   populate, per-track working targets (t_e − P_tot)/2 ≈ 0.4 compress
   the first pattern's raw mass below the Phase I F1-il bar in 2/3
   seeds (A-side 0.097–0.103 vs bars 0.142–0.145) — capacity-matched
   sharing halves the working pool the il regime previously used.
4. OPERATING-POINT family (#6 observed): s9001-bac runaway at
   t = 24,017 under the tracked gain/per-track interplay (S1 pop
   mean 164.2 Hz; detector 63.2 Hz/5 s) — the rg8 4b lesson
   re-entering at track level; preserved, not re-run.

Per protocol §12: failure modes #3, #4, #6 registered a priori and
observed; the learned-key machinery itself (blur, tags, caps, targets)
performed exactly as frozen.

## 4. Arm-2 finding (protocol-configuration, reported not revised)

Arm-2 (epoch-key partition on the clla-fe substrate) produced numbers
EXACTLY equal to the committed fec baseline (s20260912-bac first/second
22.87/2.98 — identical to anima-clla-fec-record §2). Cause: normalize()
applies CLLA precedence whenever assembly_protect is set (documented
code comment, v2.3 era), so m2_buckets=2 is a no-op on the CLLA
substrate. Per mandate, the protocol is NOT revised; arm-2 is recorded
as an inert control that doubles as a reproducibility check of the fec
baseline (exact match). The intended partition-vs-learned-key isolation
is not delivered by this arm as frozen.

## 5. Verdict (frozen §10/§11)

**S1 FAIL 6/6 ⇒ Phase II-A closes at the formation step.**
- D-core's learned context tracks are mechanically sound (identity
  parity; prototypes separated; caps exact; budgets invariant) but do
  not deliver sequential formation: the pre-existing substrate of a
  never-presented pattern is budgeted to the first context by the M1
  default tag, and is churned before the pattern's first exposure —
  the Phase I allocation bottleneck reproduced at track level.
- S-a..S-g per §2; no II-B executed; **STOP.**
- No reparameterization, no post-hoc changes, no E-number; Phase I
  records untouched; tree clean.

## 6. What this establishes / leaves open (claim language per protocol)

- ESTABLISHED: experience-derived per-neuron budget dimensions
  (learned prototypes) function as specified — routing, caps,
  invariants, determinism — and are NOT sufficient for blocked-order
  formation on this substrate because the substrate they would
  protect is pre-assigned to the wrong track by the documented M1
  default.
- NOT ESTABLISHED: selective re-expression (II-B, never tested);
  whether a substrate-survival amendment (e1-family: protecting
  never-yet-exposed working afferents from churn) would make the
  track structure effective — a NEW registration question, per §7.

## 7. Registered next questions (NOT executed; require approval)

1. e1-family amendment inside D-core: exemption of content-free
   (never-yet-coactivated) working afferents from M2/M4 churn, gated
   by the same experience-derived context signal family (a track-0
   exemption for afferents whose channels have never co-fired with
   the neuron) — directly targets the measured failure site (§3.2).
2. K-vs-floor tradeoff re-examination (K=1 control exists as arm-0;
   asymmetric budgets) — sub-budget compression (§3.3) is the cost
   of K=2 and must be priced against formation gains if (1) works.
3. s9001-bac operating point: report-only at this registration; any
   gain-interaction amendment belongs to a new registration.

STOP.