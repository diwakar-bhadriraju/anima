# ANIMA CLLA run record — bundle verdict NOT SUPPORTED

Status: PROTOCOL EXECUTION RECORD, 2026-09-20. Protocol:
docs/anima-clla-protocol.md (frozen). Implementation:
commits 47b1ecc + 061b?? (identity/integrity). All 13 runs
preserved in runs/ (including all failures). Read-only verdict
per frozen criteria; no tuning, no post-hoc threshold changes.

## 0. Identity gate (frozen §4) — PASS

- Ident arm run: runs/clla-ident-il-20260920T171250Z
  (config clla-ident-il.toml = committed e24-s20260912-il.toml
  with exp_id renamed only; CLLA fields absent).
- Event-stream FNV-1a: rows=152254, fnv=9647ea8a0ca4dbd2 —
  IDENTICAL to committed runs/e24-s20260912-il-20260919T132037Z.
  (Protocol's cited d452d028ffaec973 belongs to the V2.3-gate
  family, 135,293 rows; this curriculum's committed baseline is
  the e24-il run itself — anchor corrected in this record, no
  threshold reinterpreted.)
- Snapshot frames: fresh flag-off reproduction
  (runs/e24-s20260912-il-20260920T171348Z) vs ident arm:
  105/105 frames byte-identical (snapdiff 0 differing frames).
  Both vs the 2026-09-19 committed artifact differ ONLY in the
  V2.2-era `z_latch` serialization (null vs 0; field added in
  c6e0f0b AFTER those runs were recorded) — documented
  pre-existing artifact drift, no dynamics difference (event FNV
  identical).
- RNG: event-stream equality implies identical draw sequence.
- Integrity suite: core 76 (incl. 6 new CLLA tests), exp 47+11,
  telemetry 13, viz 5 — 0 failed.
- F5 static conformance: PASS (consolidation/normalize/prune/
  decay decisions read only w, consolidated flag, P, frozen
  caps; no pattern-id, channel-membership test, presentation
  count, or global state; M5 eviction is identity-free and
  inactive at measured occupancy, per frozen "M5 unchanged").

## 1. Run matrix (all 13, preserved)

| run | arm | seed | ended | P2 abort tick / mean rate |
|---|---|---|---|---|
| clla-ident-il-20260920T171250Z | ident | 20260912 | curriculum-complete | — |
| clla-s20260912-il-20260920T171638Z | il | 20260912 | curriculum-complete (S1 pop rate 140 Hz — runaway-adjacent, under trigger) | — |
| clla-s20260912-bac-20260920T171645Z | bac | 20260912 | **failure:runaway-activity** | t=34015, 62.8 Hz |
| clla-s20260912-bca-20260920T171648Z | bca | 20260912 | curriculum-complete | — |
| clla-s20260912-d-20260920T171653Z | d | 20260912 | **failure:runaway-activity** | t=22092 |
| clla-s424242-il-20260920T171658Z | il | 424242 | **failure:runaway-activity** | t=70011, 77.3 Hz |
| clla-s424242-bac-20260920T171703Z | bac | 424242 | curriculum-complete | — |
| clla-s424242-bca-20260920T171708Z | bca | 424242 | curriculum-complete | — |
| clla-s424242-d-20260920T171714Z | d | 424242 | **failure:runaway-activity** | t=22093 |
| clla-s9001-il-20260920T171715Z | il | 9001 | **failure:runaway-activity** | t=48083 |
| clla-s9001-bac-20260920T171718Z | bac | 9001 | **failure:runaway-activity** | t=44036 |
| clla-s9001-bca-20260920T171720Z | bca | 9001 | curriculum-complete | — |
| clla-s9001-d-20260920T171726Z | d | 9001 | **failure:runaway-activity** | t=26017 |

6 of 12 CLLA=true runs aborted on the frozen P2 runaway detector
(50 Hz/5 s; arm completion: bca 3/3, bac 2/3, il 1/3, d 0/3). All
preserved with telemetry + snapshots. (End reasons verified from
RunEnded telemetry, not CLI echo; one CLI echo was misleading —
telemetry is authoritative.)

## 2. Falsifier verdicts (frozen §6)

| id | criterion | result |
|---|---|---|
| F1 | coexistence ≥ 0.5× refs, arms {il,bac,bca}×3 | **NOT EVALUABLE** — 5/9 applicable runs aborted (all 3 il); survivors PASS (bca×3 + bac×424242: A-mass 0.46–2.48 vs thresholds 0.12–0.15) but the survivors' "coexistence" is the artifact of unbounded weight growth, not protected coexistence |
| F2 | response separation < 0.9 by rep 20 | **FAIL** — all 3 il runs aborted before measurement; no F2 measurement exists (protocol: uninformative ⇒ FAIL) |
| F3 | P ≤ 0.75·t_e + 0.04 at every snapshot | **FAIL** — survivors only: max P / (0.75·t_e) = 13.23, 12.57, 6.66, 13.30; cap violations 2284–3408 per run |
| F3b | median(P)/cap ≥ 0.99 AND W ≤ 1e-6 | **FAIL (triggered as F3)** — P exceeds cap by ≥ 6.6× |
| F4 | P2 runaway abort or any failure | **FAIL — 6/12 runs** |
| F5 | no hidden context (static) | PASS |
| F6 | cos(v̄_D, v̄_A+v̄_C) ≥ 0.90 | **FAIL** — all 3 d runs aborted; unmeasurable ⇒ FAIL |
| F7 | drive-end protected A-mass ≥ 0.5 × peak | pass numerically (1.000) but VERDICT-NULL: peak ≡ drive-end because consolidated mass grows monotonically to ceiling (LTP, unbounded) — no erosion is measured because there is no stable state to erode; the registered leak is masked by the F3/F4 failure |

## 3. Mechanism root cause (from artifacts, not tuning)

Consolidated synapses are exempt from M2 normalization and passive
decay (per protocol §3.5/§3.7) but STDP still applies (per §3.5).
Result: within-pattern LTP on protected synapses is unimpeded —
no down-scaling, no decay — and weights grow toward w_max
(measured max P ≈ 7.9/neuron vs 0.60 cap; raw A/C masses 0.46–2.48
vs references 0.24–0.29). This is the architecture doc's §5 mode 3
("post-consolidation LTP can push P above p_max_frac·t_e… bounded
by w_max") — the "bound" was wrong: with many protected synapses
per neuron each near ceiling, P is not bounded by w_max, and the
resulting drive exceeds the P2 detector. The protection removed
the very mechanisms (M2 rescale, decay) that previously held the
all-excitatory organism's runaways in check (E3/E4 history: growth
arms without these brakes regress).

## 4. Verdict (frozen §7, binary)

**CLLA bundle NOT SUPPORTED.**

Identity gate passed; F5 passed; F1 not evalueable (5/9 aborts);
F2, F3, F3b, F4, F6 FAIL; F7 verdict-null. Per the frozen rule,
any falsifier failing in any seed/arm ⇒ NOT SUPPORTED, reported
here with raw distributions, no threshold changes, no tuning.

## 5. Registered forward options (NOT executed; require your approval)

1. A-series amendment: consolidated-LTD exemption PLUS a
   consolidated-LTP ceiling (new cap constant) — addresses the
   F3/F4 mechanism directly while keeping the protection
   semantics; this is a PROTOCOL AMENDMENT, not a re-run of the
   frozen protocol.
2. Recalibrate P2 for CLLA arms — NOT recommended; the runaway is
   an organism-level instability, not an instrument setting.
3. Reconsider the consolidation-without-brakes design premise
   (protection must not be total: a protected class needs its own
   homeostatic bound by construction, cf. u's failed
   unbounded-accumulation history).
No E-number. Protocol closed per §7; nothing re-run, nothing
tuned, all artifacts preserved.