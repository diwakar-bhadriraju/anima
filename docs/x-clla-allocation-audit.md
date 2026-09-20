# CLLA allocation audit — is there a local signal for allocating unused capacity?

Status: READ-ONLY AUDIT, 2026-09-21. Uses only committed CLLA runs
(bounded-protection architecture, 925146c) + telemetry/snapshots.
No new runs, no implementation, no tuning, no protocol amendment.
Instrument: examples/alloc_audit.rs (per-presentation protected-
current IA/IC by channel cohort, response, permanence-event
bucketing, per-snapshot headroom + protected/working mass by group).

CORRECTION to the capability record (e7c0605 §3, F1 mechanism):
the recorded interpretation "the second block captures…" is
inconsistent with its own table. In the bounded runs, bac ends
A-mass 0.478–0.492 (FIRST block) / C-mass 0.037–0.101 (second);
bca mirrors. The FIRST block dominates; the second block fails to
establish. The failure is ALLOCATION DENIAL, not recency capture.
This audit supersedes that sentence; the capability verdict
(NOT SUPPORTED on F1) is unaffected.

## 1. The mechanism, precisely (blocked vs alternating)

### 1a. What protected synapses experience under re-presentation (Q1)
Protected A-synapses are NEVER modified by a C presentation:
- pA (total protected A-mass) is constant 21.20 from t≈46k through
  the end of bac (SNAP rows: pA=21.20 at t=46000..101000).
- IA (A-cohort current during C presentations) = 0.00 EXACTLY at
  every C presentation — C-channels never drive A-cohort synapses
  (pre-gating; D8 channel wiring is static).
- F7 (drive-end ≥ 0.5×peak) passed 12/12 in the capability run.
Re-presenting the SAME pattern: IA rises with consolidation
(il: A 10.7→18.7 across 20 reps) — protected structure grows
toward the cap, then pins. No re-presentation modifies the
protected cohort destructively.

### 1b. What happens when a different pattern arrives (Q2)
The arrived pattern's OWN cohort delivers low protected current at
first contact and grows slowly:
- bac: first-C IC=2.81 (vs A-block's settled 27.16); rises to 5.09
  by pres 40 — 5× increase across the 20-rep block, still ~5×
  below the first block's level.
- bca: first-A IA=1.56 (vs settled C 25.22); rises to 5.96.
- Permanence DOES occur: bac cumC 0→199; bca cumA 0→265. But
  protected mass acquires slowly (bac pC 0→1.95 vs pA 21.20;
  bca pA 0→~2 vs pC ~20): arrival allocations are capped by the
  ~0.16–0.19 headroom left after the first block.

### 1c. Why blocked differs from alternating — the exact site (Q6)
Not: recruitment of the same neurons (both patterns synaptically
reach all 52 neurons; the wiring is static and identical).
Not: modification of existing protected synapses (1a).
Not: M2 routing new activity into one population (M2 normalizes
per-neuron working mass to t_e−P; it does not choose neurons).
The failure is: **M3 permanence + cap-entry serialization.** The
first block:
    A presentations → strong IA → A-candidates co-fire per window
    → permanence events accumulate (cumA reaches 364 by pres 20)
    → entry gate admits them while headroom (0.6→0.19 at pres 20)
    → protected A mass fills the per-neuron budget.
The second block arrives with headroom already consumed: its
candidates still reach permanence (M3 runs regardless) but the
entry gate (P + w ≤ cap with P≈cap) admits nothing; protected C
accretes only through the residual headroom, 6× slower. In
alternation, BOTH patterns' candidates co-fire from pres 1, so
both claim headroom concurrently and end balanced (pA 15.85 /
pC 11.34; ratio 0.58).

### 1d. First-C-after-A vs later-C; first-A-after-C vs later-A (Q7/Q8)
Measured at the cohort boundary (bac pres 20→21; bca pres 20→21):
- pattern switch: protected current for the arriving pattern drops
  ~10× in one presentation (bac: 27.16→2.81; bca: 25.22→1.56).
- response norm ALSO drops (bac 27.6→5.5; bca 34.0→19.6) but
  recovers only partially with training (response is activity-
  driven, not identity-driven).
- permanence events for the arriving pattern CONTINUE to accrue
  through the second block at a bounded rate; protected mass
  growth is budget-capped, not stopped (bac pC 1.95 = 9% of pA).
- There is therefore a DETECTABLE local event at the boundary
  (protected-current step-down ~10×), but no allocation response
  to it: the organism registers the novelty (low protected
  activation) and DOES consolidate, but the per-neuron budget is
  already spent.

## 2. Candidate local novelty/mismatch signals (Q3/Q4/Q5)

| candidate | measure | separates? | evidence |
|---|---|---|---|
| (a) protected-cohort current at presentation | IA, IC per neuron | YES for identity (A-only: IC=0.00; C-only: IA=0.00; exact) | every presentation, every run |
| (b) pattern-exclusive cohort activation | IA·IC ≡ 0 | separates NEW-NEW vs OLD-NEW: new pattern shows old-cohort≈0 AND new-cohort>0 | exact by construction |
| (c) response norm | spikes/window | NO between patterns (A 22–24, C 34→21 il; recruitment overlap confounds) | measured |
| (d) per-neuron headroom | cap − P | YES as a bounded 'unused config' quantity; NOT identity-specific | 0.6→0.09–0.16 by end |
| (e) protected vs working activation ratio | IA/(IA+WA) | identity-correlated (A pres: 27 vs WA≈1) but magnitude rises with training — cannot separate 'new' from 'stronger old' | bac C-block IC 2.81→5.09; il C 5.3→14.9 |
| (f) M3 permanence accumulation | candidate→permanent events | responds to BOTH new and repeated old (co-activity) | cumA/cumC both accrue |
| (g) failure of protected-assembly activation | IA < threshold on arrival | THE signal: 10× step-down at every pattern switch | bac pres 20→21, bca 20→21 |

CRITICAL DISTINCTION (Q5): Protected-current MAGNITUDE cannot
separate 'new pattern' from 'stronger/repeated old' — both rise
with consolidation (bac C 2.81→5.09; il C 5.32→14.88). The
separable signal is STRUCTURAL, not magnitude: a presentation is
'novel wrt this neuron' iff its channel-cohort activation hits
this neuron's protected cohorts at ~0 while hitting some
working/unprotected cohort >0 — i.e. a NEURON-LOCAL mismatch
between 'whose channels are firing' and 'which cohorts I have
protected'. This is (b)+(g): it exists, is local, deterministic,
and requires NO global operation or context label (Q10 satisfied).

## 3. Allocation failure location (exact)

M3 candidate→permanent accumulates on co-activity with NO
reference to what the neuron already protects. The per-neuron
entry gate (P + w ≤ cap) then serializes acquisitions by
first-come-first-served arrival: whichever pattern's candidates
mature earliest takes the headroom; later arrivals are admitted
only at the residual. There is no mechanism that CONSUMES the
novelty signal (2g) to RESERVE headroom, PRIORITIZE an
under-protected cohort, or CAP the first pattern's claim.

Headroom therefore does NOT serve as allocatable capacity in the
blocked regime — it serves as a first-pattern reservoir (Q-final).
In the alternating regime it works by luck of concurrent arrival,
not by design: headroom 0.6→0.15 with both patterns sharing it
pA+pC ≈ 27.2 < 31.2 cap total.

## 4. Evidence summary per the mandate's list

1. blocked-vs-alternating allocation: blocked = first-come cap
   serialization (allocation denial for the second block);
   alternating = concurrent headroom sharing (balanced outcome).
2. candidate local signals: (a)–(g) above; (g) is the operative
   novelty signal, (d) the operative capacity quantity.
3. evidence for/against each: table in §2; magnitude-signals
   (c,e) fail the new-vs-stronger-old test; structural signals
   (a,b,g) pass; headroom is real but identity-blind.
4. exact failure site: M3 permanence accumulation + cap-entry
   gate — the gate serializes by arrival time, not by protecting
   the under-represented cohort.
5. CAN allocation be derived from existing local state? PARTIALLY:
   the novelty signal (g) and the capacity quantity (d) both exist
   per-neuron, locally, deterministically. What is MISSING is a
   connection: nothing reads (g) to reserve (d) for the novel
   cohort. So: the SIGNAL exists; the DECISION does not.
6. minimum new capability: a LOCAL allocation rule using the
   existing quantities — on strong novel-cohort activation with
   available headroom, the neuron reserves/prioritizes headroom
   for that cohort's candidates (e.g., an arrival-ordered per-
   cohort claim on cap, or a headroom-reservation triggered by the
   (g) step-down). This is a decision on existing state, not a
   new signal. (No implementation proposed in this audit.)
7. smallest falsifiable experiment: blocked-order (bac/bca,
   all 3 seeds) with the allocation rule (whatever form it takes)
   added; pass = second block's protected mass ≥ 0.5× first
   block's (the F1 blocked criterion) AND alternating behavior
   unchanged (F1 il + F7 + F3). Identity: flag-off byte-identical;
   the rule reads only local cohort activation + headroom.
8. F2/F6 window repair (NOT executed, required): the frozen
   protocol's per-pattern reps {21..40} windows are empty under
   the frozen il curriculum (20 A + 20 C). Required repair:
   amend the protocol text to use reps {11..20} (the last 10 of
   each pattern — the 'late' window that actually exists) for
   both F2 separation and the F6 same-seed basis, OR extend the
   il curriculum to 40/40 (config change, new runs). Either is a
   protocol amendment requiring user approval, not an analysis
   substitution.

## 5. Does a local novelty detector exist? — answer

YES, in the narrow, correct sense: per-neuron protected-cohort
activation is a local, deterministic, pattern-locked signal that
detects mismatch against already-protected structure (the 10×
step-down at every pattern switch; exact zero-cross-excitation
between A/C cohorts). It is NOT a global similarity detector and
NOT a label.

This audit found NO evidence that the organism USES it: blocked-
order runs show the step-down, then allocation proceeds into the
residual budget regardless. The signal is present but uncoupled
from the allocation decision.

## 6. Unmeasurable / unresolved

- F2/F6 formal windows (per-protocol) — unexecutable; repair
  required (§4.8).
- Whether the 'stronger old pattern' case (same channels, higher
  rate) would confuse the structural signal: the curriculum has
  no such condition; the structural signal predicts it would NOT
  confuse (identity is channel-set-based, rate-independent), but
  that is [INFERENCE], untested.
- Whether per-cohort reservation degrades single-cohort stability
  (the cap currently pins P per neuron regardless of cohort) —
  needs the allocation experiment, not analysis.

STOP. Audit complete; nothing implemented, nothing run, no
protocol change made.