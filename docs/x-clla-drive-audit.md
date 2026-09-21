# First-exposure drive-recruitment audit (READ-ONLY)

Status: 2026-09-21. Uses only committed corrected-fe BAC/BCA/IL
runs + telemetry/snapshots. Instrument: examples/driveaudit.rs
(per-neuron per-presentation current decomposition: protected/
working A+C afferent, recurrent, inhibitory, posts, membrane).
No implementation, no runs, no tuning, no E-number.

## 1. Exact localization of the C recruitment failure

The failure is at THE VERY FIRST LINK: **afferent drive at first
contact**, upstream of everything else.

Blocked (bac s20260912), first C presentation (t=45,001):
- Iaff (total afferent current) = 1.565 per neuron — vs first-A
  (same run, pres 1) = 11.982 → **7.7× lower**.
- Of that 1.565: 100% is WORKING (IwC=1.565; IpC=0 — zero
  protected C, expected), delivered through the ~29 surviving
  working C afferents (of 197 original: block-1 M2/M4 churn
  destroyed 85%).
- Recurrent current = 0.050 vs A's first-pres 6.90 (and 21.6 by
  pres 2) → **a 100-400× recurrent deficit**; the all-excitatory
  recurrent amplifier never engages for C because too few neurons
  fire.
- Posts/n = 0.12 vs A's 0.98 → **8.2× fewer posts** (the measured
  17× network-total drop, ~8× on a per-neuron basis, consistent).

So: C's first contact delivers 1.6 drive/neuron; A's first contact
delivered 12. The neuron is ~7.7× underdriven, largely
subthreshold, produces few posts, no recurrent bootstrap, no LTP
opportunities. Everything later (weight stall) is downstream.

## 2. A vs C drive/recruitment decomposition (measured)

| quantity (per neuron) | block-1 A, pres 1 | block-2 C, pres 21 | ratio |
|---|---|---|---|
| IpA (protected A aff) | 0.000 | 0.000 | — |
| IwA (working A aff) | 11.982 | 0.000 | — |
| IpC | 0.000 | 0.000 | — |
| IwC (working C aff) | 0.000 | 1.565 | — |
| Iaff total | 11.982 | 1.565 | 7.7× |
| Irec (recurrent) | 6.901 | 0.050 | 138× |
| Iinh (inhibitory) | −2.191 | −0.812 | 2.7× |
| posts/n | 0.98 | 0.12 | 8.2× |
| vmed (context, coarse) | 0.00 | 0.25 | — |

Recurrent is THE amplifier that makes A's first block robust
(Irec climbs 6.9 → 21.6 by pres 2 as posts feed back); C has no
posts to feed back, so Irec stays ≤ 0.13. The 0.12 posts/n is
entirely C-afferent-driven (subthreshold); A's 0.98 was
afferent(12)+recurrent(7) driven.

## 3. IL vs blocked — the crucial variable

IL (C from pres 1): first C = **IwC 11.130, Irec 7.05, posts 1.00
— numerically indistinguishable from A's first contact** (blocked
pres1-A: 11.982/6.90/0.98). IL C posts stay 0.54–1.00 across the
whole run; C permanence 98 ≈ A 95.

CONCLUSION (measured, not inferred): the variable is PRIOR
REPRESENTATION / SYNAPTIC COMPETITION — i.e., the block-1 churn
that destroys C's working afferents before C ever presents. IL
never destroys them, so C's first contact has full drive. The C
stimulus is intrinsically fine (IL proves it recruits as well as
A); the blocked paradigm starves it of substrate pre-exposure.

## 4. M6 contribution

Measured: Iinh at first C = −0.812 vs first-A −2.19. Inhibition is
QUIET in the C block because the population is quiet (M6 is
activity-driven). It is not suppressing C (its magnitude at first
C is 2.7× LOWER than at first A). M6 is excluded as a causal
factor in the first-exposure failure. (It could contribute a
secondary floor later, but nothing in the data shows inhibition
exceeding excitation at the C switch.)

## 5. Local signals that could support recruitment (existing state)

Audited per neuron:
- (1−R) = unexplained-input fraction: EXISTS (alloc rule) — at
  first C it is 1.0 (R=0) — but it cannot add current; it only
  gates allocation. Measured 1.0 throughout.
- afferent-to-recurrent ratio: exists (both computable locally);
  at first C = 1.565/0.05 = 31 (A = 12/6.9 = 1.7) — the ratio is
  HIGH for C, not low; it flags "input not amplified" but is a
  consequence, not an exploitable cause.
- membrane underdrive: vmed at first C (0.25) ≈ A's (0.18–0.32) —
  the snapshot cadence (1 s) cannot resolve the 500 ms window;
  not usable at this resolution (reported for context only).
- excitation/inhibition balance: measurable, but M6 is quiet (yes).
- adaptation: i_adapt is NOT in snapshots/telemetry — unmeasurable
  from committed artifacts. The audit cannot exclude a
  contribution, but there is no positive evidence for one: A's
  first contact had the same adaptation dynamics and recruited
  fine.
- working-afferent availability: THE measured signal — 29/197 =
  15% of C's original working afferents survive at first contact
  (IL: ~100%). This is the only quantity whose blocked-vs-IL
  difference explains the full response gap, and it is local
  (per-neuron incoming-synapse count × weight).

## 6. Is a small transient recruitment change sufficient in principle?

Retrospective arithmetic (no dynamics altered):
- A's first contact: drive 12+7 = 19 (aff+rec) → posts 0.98.
- C's first contact: drive 1.565+0.05 = 1.6 → posts 0.12.
- Linearity suggests posts ≈ drive/16 (0.98/19 ≈ 0.052; 1.6×0.052
  ≈ 0.083 ≈ observed 0.12 within noise). A ~10× afferent boost at
  first C (to ~16 ⊍ A's level) would put C at drive 16+7 = 23 →
  posts ≈ 1.0 → recurrent amplification (the measured A pres-2
  Irec 21.6 feedback loop) → LTP opportunities → weight growth.
- Therefore a transient ~10× recruitment-side gain at first
  exposure is ARITHMETICALLY SUFFICIENT to cross the recruitment
  threshold and bootstrap the recurrent loop, IF it acts on
  response (since the afferents themselves are gone; no
  synapse-count/weight mechanism could help — §8).
- Restriction: this is arithmetic, not dynamics; the experiment
  would test it. But it bounds the required magnitude: the boost
  must compensate a 7.7× afferent deficit and does not need to be
  large relative to A's normal drive.

## 7. The three-arm summary (Section 8 candidates)

The candidate mechanisms are evaluated against the measured
failure chain (afferent-substrate deficit at first contact →
underdrive → no posts → no recurrence → no LTP → low weight):

A. local novelty-dependent recruitment gain (neuron boosts its
   response when input is unexplained and drive is low): CAN act
   at the exact link (response side), does not need afferent
   repair, local, needs only (1−R) + input-current (both exist).
   Directly compensates the 7.7× deficit. SMALLEST that matches
   the failure site.
B. local disinhibition when protected poorly explains input:
   targets M6 — but M6 is already quiet (−0.81 vs −2.19); there
   is nothing to disinhibit at first C. NOT supported by data.
C. transient afferent competition relief (slow M2/M4 during
   first exposure): targets substrate preservation — but the
   destruction ALREADY happened in block 1; at first C the
   afferents are gone. Would need to act DURING block 1 on a
   pattern not yet presented (impossible, would require future
   knowledge). NOT supported.
D. intrinsic excitability/adaptation response to underrepresented
   input: targets adaptation — unmeasurable in artifacts; no
   positive evidence; A's same dynamics recruited fine. Speculative.
E. another mechanism: none in the data — the failure is
   substrate-driven at first contact; response-side gain is the
   only lever that (a) exists locally, (b) compensates 7.7×, (c)
   does not require future knowledge.

SELECTED: A — local novelty-dependent recruitment gain, scoped to
the measured gap (compensates sub-A first-contact drive when
(1−R) high AND posts low). It is the only candidate that acts at
the demonstrated failure link.

## 8. Safety against cheating (for any future mechanism)

Any recruitment gain MUST:
- use only (1−R), input current, posts, headroom — all per-neuron
  existing quantities; a gain g' = f((1−R), drive, posts) with
  no channel/cohort/pattern identity (it cannot know A/C);
- NOT use population/global activity (per-neuron only);
- NOT label input as novel (it uses "unexplained + underrecruited",
  both computable locally — this is the audit's §5 signal set);
- NOT be permanent (the gain must decay once the pattern is
  explained or consolidated — else it is a permanent boost);
- respect resource constraints (transient response gain does not
  add synapses/weights; stays within membrane dynamics).

The mechanism must explain "a neuron becomes recruitable when its
protected configuration doesn't explain incoming activity": via a
local response gain proportional to working-input proceeds and
inversely to protected-explained fraction — i.e., the missing
recruitment is supplied while the representation forms, then
withdraws as R → 1.

## 9. Claims / limits

- Established: the failure is at the afferent-drive level at
  FIRST CONTACT (7.7× underdrive); recurrent amplification never
  engages; IL proves the stimulus is fine (C 11.13 current,
  posts 1.0); M6 excluded; the deficit is 85% destroyed C
  working substrate from block 1.
- NOT established: the exact dynamics of the bootstrap crossing
  (arithmetic only); adaptation's role (unmeasurable); whether
  any response-side gain is dynamically stable (needs the
  experiment).
- No implementation, no runs, no tuning, no E-number. The single
  smallest capability worth designing is A (local
  novelty-dependent recruitment gain), justified by the measured
  drive deficit and the IL control.

STOP — audit complete.