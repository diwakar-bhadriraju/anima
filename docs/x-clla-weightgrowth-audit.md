# Second-block synaptic weight growth — causal audit (READ-ONLY)

Status: 2026-09-21. Uses only committed source-correction runs
(clla-fe-*20260921T1533*) + telemetry/snapshots. No implementation,
no runs, no tuning, no E-number. Instrument: examples/wgrowth.rs
(per-synapse weight trajectories, STDP event net, M4 prunes by
cohort, M2 target trajectory, per-presentation post/pre counts).

## 1. LTP budget decomposition (bac s20260912, corrected-fe)

Per newly consolidated C synapse (61 events during block 2):
- STARTING WEIGHT AT PERMANENCE: w_c_permanent = 0.02 committed
  (snapshot-verified: first-5 rows all 0.019–0.021).
- PRE-SPIKES: abundant — C channels fire 56–87 input spikes per
  500 ms presentation (measured); the input side is NOT starved.
- POST-SPIKES: 17× collapsed at the switch. Last A presentation
  (pres 20): 1,697 network posts; first C (pres 21): 101; C's
  presentations sustain only 101–222 posts (vs A's 1,300–1,700).
- RAW LTP OPPORTUNITY: post-coincident pre windows at 8–13% of the
  A-block level (post counts ratio ~150/1600) — majors deficit.
- APPLIED LTP: STDP net on C-cohort in C-block = **−21.63**
  (1,961 coalesced events, net NEGATIVE). Posts so sparse that the
  dominant pre→post pairing is LTD (post-trace ≈ 0 at most pre
  spikes). Per-synapse: where posts DID coincide, growth factors
  2.5–3.8× (0.02→0.185, 0.02→0.205, median 2.5×) — the LTP
  machinery itself is intact; it just rarely fires.
- M2 SCALING FACTOR: not the limiter — target t_e−P stays
  0.360→0.303 across block 2 (P/neu 0.440→0.497, cap 0.6).
- WORKING MASS BEFORE/AFTER: mild squeeze (target 0.36→0.30);
  working C afferents 29→11 but the LTP deficit appears before
  this and dominates it.
- PROTECTED MASS P: 0.0→0.10/neuron over block 2 — tiny; no
  cap interaction (reserve 0.1 of 0.6).
- AVAILABLE M2 TARGET: ≥ 0.303 always — ample.
- SURVIVING WORKING C AFFERENTS: 29→11 across block 2
  (secondary; see §4).
- M4 PRUNES OF C AFFERENTS: C-block only 23 (vs 161 in A-block) —
  M4 is QUIET during the second block, not destructive.
- WEIGHT LOST VIA M2 REDISTRIBUTION: at most the 0.36→0.30 target
  drift on working mass; PROTECTED C weight is excluded from M2
  (by CLLA design) — M2 caused ≤ 0.06/neuron working-mass loss.
- WEIGHT GAINED VIA STDP: net NEGATIVE (−21.63 on C-cohort in
  C-block) — STDP is the net LOSS channel, not the gain channel,
  because post spikes are absent.

## 2. M2 scaling trajectory (measured)

| t | P/neu | M2 target | uC (working C aff) | cC | cmC |
|---|---|---|---|---|---|
| 44000 | 0.440 | 0.360 | 29 | 0 | 0.00 |
| 51000 | 0.442 | 0.358 | 17 | 5 | 0.10 |
| 61000 | 0.468 | 0.332 | 12 | 28 | 1.48 |
| 71000 | 0.488 | 0.312 | 10 | 46 | 2.51 |
| 84000 | 0.497 | 0.303 | 11 | 55 | 2.98 |

M2 target never below 0.30; cap (0.6) never approached. M2 is
NOT the applied-growth suppressor.

## 3. C working-afferent survival (measured)

uC: 29 → 17 → 12 → 10 → 11 across block 2. Losses are mild M2
target drift + working→protected consolidation (each permanence
converts a working C afferent to protected). M4 C-prunes in
C-block = 23 — negligible. Churn is NOT the C-block limiter.

## 4. M4 contribution (measured)

A-block pruned C-afferents heavily (161) — block-1 destructive
mass — but C-block prunes only 23. M4 cannot explain the second
block's weight stall; it is nearly inactive there.

## 5. Temporal causal ordering

EVENT SEQUENCE (all timestamps measured):
  pres 20 (A): 1,697 posts, cmC = 0
  pres 21 (first C): 101 posts          ← POST-COLLAPSE
  t = 47,500: first C permanence         (2.5 s AFTER the collapse)
  t = 47.5k–84k: C protected mass grows 0→2.98 SLOWLY (0.10→0.10
  per 10s) with C posts ~150              ← LTP STARVED
  M2 target 0.36→0.30; uC 29→11         ← SEQUENTIAL, not causal

ORDERING: post-activity collapse (pres 21) precedes EVERY C
consolidation and all C weight growth. The LTP deficit is present
from the first C presentation — it is NOT caused by the later
protected-mass accumulation or working-churn. This is event
ordering, not correlation.

## 6. Strongest evidence-supported bottleneck

EVENT-LIMITED (A): C receives too few RELEVANT post-spike LTP
opportunities. The convergence of evidence:
- post counts 101–222 vs A-block 1,300–1,700 (17× at switch);
- C pre-spikes abundant (56–87) — input fine;
- STDP net on C-cohort in C-block = −21.63 (sparse posts make the
  dominant pairing LTD);
- per-synapse growth works where posts present (0.02→0.185);
- M2 target ≥ 0.30 (not suppressing);
- M4 C-prunes in C-block = 23 (not destroying);
- ordering: collapse precedes consolidation.

The fe source correction delivered candidates and permanence
exactly as designed; it cannot create POST-SYNAPTIC ACTIVATION for
a pattern the network no longer supports. The deep cause: the
network after block 1 is A-conditioned — A-afferents dominate the
membrane, the recurrent pool is A-excited; C's 29 surviving
working afferents deliver weak drive, so C presentations barely
fire the population. The 0.054 vs 0.096 final-weight gap is the
symptom; the cause is the 17× post-activity deficit at the switch.

## 7. Are e1 or e2 justified?

- e1 (protect a fraction of second-block working afferents at
  consolidation time): NOT justified by this evidence. Churn is
  mild (M4 = 23 prunes; M2 target ≥ 0.30) and postdates the LTP
  deficit. Protecting working C afferents would not raise post
  activation — there are no posts to protect gain for.
- e2 (protected-synapse LTP growth path): NOT the binding
  constraint either — where posts exist, protected C synapses DO
  grow (0.02→0.185). The growth path is intact.
- The evidence points to the PRE-CONSOLIDATION input-drive level:
  the C block cannot excite the network because the A-conditioned
  substrate (working A afferents 52 survivors + A-dominant
  recurrent pool) suppresses C's effective drive. This is a NEW
  candidate mechanism question (drive/activation asymmetry), NOT
  e1 or e2. Neither e1 nor e2 is justified by this audit.

## 8. Smallest next experiment (if justified; NOT executed)

The falsifiable question: is the 17× post-collapse intrinsic to
the blocked curriculum's drive asymmetry, or removable by a
mechanism that raises C's effective post activation (e.g., a
transient working-gain asymmetry during the first exposures of a
novel pattern — explicitly NOT M2/M4/protection changes)? The
smallest test that distinguishes EVENT-LIMITED from
MECHANISM-REMOVABLE without parameter sweeps:
  compare the corrected-fe blocked runs against IL runs at matched
  presentation counts: in IL, C has been present from pres 1, so
  its working substrate never collapsed; if IL's C posts stay
  high (≥ A's) and its C protected weight reaches ≥ A's, the
  deficit is purely blocked-order drive history (event-limited,
  curriculum-inherent); if IL's C posts are ALSO low, the deficit
  is a per-neuron drive property independent of history.
This is a read-only cross-arm analysis of ALREADY-COMMITTED runs
(IL arms exist) — no new runs, no mechanism, no tuning.

## 8b. Cross-arm localization (committed IL runs, read-only)

The same instrumentation on the committed IL arm (clla-fe-s20260912-
il, C present from pres 1, no collapsed substrate):
- C post activity: mean 1,491 (range 745–5,045) vs A 1,574 —
  C/A ratio 0.947 (vs 0.08 in the blocked C block).
- C-cohort permanence 98 ≈ A-cohort 95 — BALANCED coexistence.
- First5/last5 C posts 1,785→1,568 — stable, never collapsed.

CONCLUSION: the 17× post-collapse is BLOCKED-ORDER DRIVE HISTORY,
not a per-neuron drive property. When C shares the network's
history (IL), its posts and permanence are symmetric to A's. The
C weight-growth deficit in blocked runs is therefore
CURRICULUM-INHERENT: block 1 teaches the network to ignore C's
input channels (A-afferents dominate; recurrent pool is A-
conditioned; C's surviving working afferents ~29 are too weak to
excite the population). No substrate mechanism (M2/M4/protection/
consolidation) produced it — the blocking itself did.

## 9. Claims / limits

- Established: second-block weight growth is post-spike-event-
  limited; M2/M4/churn excluded at measured magnitudes; causal
  ordering puts the collapse before consolidation.
- NOT established: the upstream cause of the post-collapse (drive
  asymmetry mechanism) — Section 8's cross-arm analysis is the
  read-only step that would localize it further.
- No e1/e2 justification from this data. No implementation, no
  runs, no tuning, no E-number.

STOP — audit complete.