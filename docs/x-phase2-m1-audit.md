# Phase II-A M1 track-addressing audit — how pre-existing afferents acquire learned context identity

Status: READ-ONLY ARCHITECTURE AUDIT, 2026-09-22. Evidence: Phase II-A
runs (runs/clla-d2a-*20260922T0844*Z), Phase I records, protocol
cf768bc, result d0c349b. No implementation, no new runs (only read-only
instruments over preserved artifacts: examples/m1audit.rs, rg8eval),
no tuning, no E-number.

## 1. The exact M1 initialization problem

M1 wiring is created at network construction — BEFORE any track
identity exists. Phase II-A assigned these synapses the documented
default tag 0 (§7 of the protocol; verified unit). The problem is not
the mechanics of the default; it is the SEMANTICS: a track tag means
"membership in a learned context". Assigning membership at birth, for
the ~85% of virgin afferents that will never co-activate with the first
presented pattern, grants context-0 membership WITHOUT learning. The
consequences, all measured in Phase II-A:

1. The never-presented pattern's surviving substrate lives in track-0's
   budget from birth: at first C exposure, surviving C working mass is
   100% tag-0 (m1audit: s20260912-bac C_tag0 = 0.289, C_tag1 = 0.000),
   so its current is invisible to track-1's novelty gate: the
   recruitment gain (gated on (1−R_1) with I_W_t1 ≈ 0) never fires
   (measured: first-C I_c_tag1 = 0.000 even while 317 spikes fire).
2. The same substrate is churn-exposed exactly as Phase I measured —
   each per-track budget collapses to the Phase I shared-budget
   behavior while its population is 1.
3. Measured split across cells: s20260912 (both orders) PRESERVED but
   MISADDRESSED (surviving mass 0.289/0.543 all tag-0; first-exposure
   posts 0.13/0.04); s424242 (both orders) UNPRESERVED (surviving mass
   0.000); s9001-bac preserved-but-misaddressed (0.364 tag-0) then
   aborted.

## 2. Could a churn exemption (preservation) alone solve it? — NO

The fec-era records already answer this: with preservation-like
conditions (the corrected allocator, surviving substrate 1.565/neuron
at first C), first-exposure posts stayed at 0.12 — far below the 0.5
bootstrap bar. Phase II-A reproduces the state: s20260912-bac has MORE
first-C single-spike drive than the fec baseline (0.29/neuron... vs
fec 1.57 measured by the SAME single-spike convention — wait: reconcile
in §3) and still posts 0.13. Preservation gives substrate; it does not
address it, and ADDRESSING is what decides whether the surviving
current (a) reaches the unexplained track's gain, and (b) sits in a
track whose M2 target upscales it. The Phase II-A counterfactual (§3)
shows the misaddressed state is the one that blocks the crossing.

## 3. Read-only counterfactual: local re-keying at first exposure

Method: arithmetic replay on preserved snapshots/telemetry only; the
organism's recorded dynamics are untouched. Convention: single-spike
current per neuron (the audit convention: 1.565/neuron at fec first-C
means amp × mean survival mass; ratios are convention-invariant).
Retag rule tested: at the first exposure window of the second pattern,
all live working afferents of the second cohort switch tag 0 → 1;
then the frozen per-track M2 target applies to track 1:
T_1 = (t_e − P_tot)/2 per neuron. The M2 fixed point is linear:
one-shot factor = T_1/(second-cohort working mass per neuron).

Per cell (data: m1audit M-line at t = 44,000; drive = rg8eval
single-spike convention; posts measured at pres 21):

| cell | surviving 2nd-cohort mass/neu | T_1/neu | retag factor | drive after retag/neu | gain (capped) | projected posts | verdict |
|---|---|---|---|---|---|---|---|
| s20260912-bac | 0.00556 | 0.184 | 33× | 0.29 → 9.5 | +≤20 | ≥ 0.5 (map: ≥19 saturating; IL 19→0.6) | CROSSES |
| s20260912-bca | 0.01044 | 0.184 | 18× | 0.54 → 9.6 | +≤20 | ≥ 0.5 | CROSSES |
| s9001-bac | 0.00700 | 0.184 | 26× | 0.36 → 9.4 | +≤20 | ≥ 0.5 | CROSSES (modulo abort cell) |
| s424242-bac | 0.000 | 0.184 | — | 0 → 0 | 0 | 0.04 (measured) | VACUOUS |
| s424242-bca | 0.000 | 0.184 | — | 0 → 0 | 0 | — | VACUOUS |

Notes: factors never clamp (scaled weights 0.098–0.35 < w_max 1);
track-0 loses the mass (benign scale-up of its residual working set);
per-window re-normalization reaches the same fixed point (linear);
the boost estimate uses the rg8c cap (≤ ~20 current) on the NOW
unmuted track-1 gate.

CONCLUSION: in the three preserved-but-misaddressed cells, local
re-keying at first exposure lifts the first-exposure drive to A-parity
(~9.5/neuron vs A's 11.98) and unmutes the capped gain — crossing the
measured bootstrap region. In the two unpreserved cells, re-keying is
vacuous: nothing survives to re-key. THE TWO CONDITIONS ARE
COMPLEMENTARY OVER THE SEED SET: preservation is required first
(s424242), addressing is required for preservation to matter
(s20260912). Neither alone suffices for all seeds.

## 4. Candidate M1 track-addressing semantics (12 questions each)

Questions (abbrev.): 1 first-pattern fate; 2 second-pattern fate;
3 same synapse in two memories?; 4 retag overwrites memory?;
5 extra explicit resource?; 6 label-free?; 7 implementable from
firing input + own prototypes?; 8 new state?; 9 new timescale?;
10 all-K-occupied?; 11 A→C and C→A symmetric?; 12 IL stable?

### A. STATIC INITIAL ASSIGNMENT (current; default 0 forever)
1 stays 0; 2 stays 0 (measured: S1 6/6 fail, misaddressed survival in
3 cells); 3 no (single tag); 4 n/a; 5 no; 6 yes; 7 yes (trivial); 8 no;
9 no; 10 measured exhaustion → second block starves; 11 symmetric
failure measured (0.038/0.046 and 0.000/0.000); 12 IL passes 3/3.
VERDICT: cannot solve sequential formation (Phase II-A measured);
only option compatible with today's code.

### B. LOCAL RETAGGING (M1 afferent changes track on first relevant
### co-activation with the post's current learned context)
1 first pattern claims its co-firing afferents (tag → first context);
2 second pattern claims its survivors at first exposure;
3 no (single tag; a synapse belongs to one track);
4 NO if the rule fires only for afferents that have NEVER fired in
any earlier context (first-contact rule): established memories'
afferents already fired → immune. If retag-on-any-reactivation were
used instead, YES — must be rejected (this is why the rule must be
first-fire-only);
5 no new storage (tag rewrite only);
6 yes: the evidence is "this channel fired while context c* was the
neuron's learned context" — no channel-group/patter identity;
7 YES — precisely the signals named; the prototype state is the
local evidence;
8 no (existing tag field; no extra bits);
9 no (structural-window cadence; no new timer);
10 exhausted tracks: retagging moves mass between existing tracks —
if all K tracks hold protected memories, first-fire retagging still
functions (the afferent joins an existing track; no new track is
created). Capacity is unchanged: no new memory can be created beyond
K — exhaustion semantics as frozen;
11 YES by construction (order-blind first-contact rule: C channels
fire first in bca, A channels in bac; the rule reads only "fired +
context match");
12 IL: A-afferents claim their context at first A windows, C at
first C windows, then both are stable (no re-firing ever retags);
alternation unchanged — supported by the measured rule-intactness.
CON: nothing protects the substrate from churn BEFORE first contact
(the s424242 half: retag is vacuous on dead substrate) — needs
exemption as a separate capability (§6).

### C. DUPLICATION (one M1 synapse in K tracks)
1 the synapse joins every track's budget; 2 same;
3 YES (multiplicitous participation); 4 no (no writes, only shared
accounting); 5 YES — K× accounting, and each track's caps tighten
per shared weight; total per-neuron effective capacity shrinks
(weight counted K times against budgets); 6 yes; 7 yes; 8 yes (tag
set/bitmask); 9 no; 10 occupied tracks: the shared synapse's substrate
is already reserved — no allocation freedom added; 11 symmetric;
12 stable.
VERDICT: gives nothing that B gives, costs K× accounting and tighter
caps; AND it re-introduces exactly the superposition semantics Phase I
measured as non-selective (a shared physical synapse contributes to
both memories' expression by construction). Not recommended.

### D. DEFERRED TRACK ASSIGNMENT (untracked until first exposure)
1 first pattern CLAIMS its co-firing afferents (untracked → first
context at first co-activation), exactly B's rule but with an
explicit third state before contact;
2 second pattern claims its survivors at first exposure (same rule);
3 no (single tag after claim; untracked before);
4 no (first-contact-only, as B);
5 NO NEW per-synapse state beyond one tag VALUE (tag 2 = untracked
vs today's implicit-0); the untracked class must be normalized — the
frozen rule: untracked working mass shares ONE pre-track budget
normalized to (t_e − P_tot) exactly as Phase I (tag-untracked is a
temporary state, not a track);
6 yes (fired + context evidence only);
7 yes;
8 one new state NEEDED: the explicit untracked value — this is the
semantic core (default-0 was the implicit unlearned assignment);
9 no;
10 exhausted tracks: untracked mass claims into an existing track at
first contact (never creates capacity); tracks full of protected
memory ⇒ claim still works into the least-loaded track (no new
memory possible beyond K — frozen semantics);
11 YES (order-blind, as B);
12 YES (as B; post-claim stability identical).
CON: same as B — churn precedes first contact in the unpreserved
cells; D alone does not stop block-1 death of virgin substrate.

### E. ANOTHER local rule justified by the evidence: FIRST-CONTACT CLAIM
### WITH CHURN-EXEMPT DORMANCY (D + a survival floor)
While untracked (never-co-activated), an afferent contributes to the
shared pre-track budget BUT is exempt from M4 pruning and is decay-
protected at a floor (θ_prune is the floor, not death); on first
co-activation it is claimed by the post's current context and enters
normal track semantics. Justified by: measured split (§1) — s424242
lost its substrate to churn before contact; SDE-C2 (decay is the
dominant eraser); the fec (e) classification (working-afferent churn
is the residual). This is the ONLY candidate that addresses both
measured conditions; it is A+B/D plus a floor on the unclaimed class
— no new learnable machinery, one new state value, the existing
theta_prune as the floor constant (no new parameter).

## 5. Preserved substrate vs correctly addressed substrate

Measured, not assumed:
- s20260912 (both orders): preserved-and-misaddressed ⇒ retag alone
  crosses (§3). Correct addressing is the binding condition HERE.
- s424242 (both orders): unpreserved ⇒ retag vacuous. Preservation
  is the binding condition HERE.
- "Preserve the dormant afferent" alone was also measured (fec-era:
  survival without addressing left posts at 0.12).
Resolution: the two conditions bind at different seeds; the smallest
capability that repairs the measured split is E (first-contact claim
with churn-exempt dormancy).

## 6. Is default track-0 an architectural violation?

YES, as a design inconsistency: the protocol's learned-key principle
(§6: identity is learned from the neuron's own usage history) is
violated for the virgin majority — default-0 grants context-0
membership without learning, embedding a primacy bias (the first
context owns all unencountered substrate) that was never derived from
co-occurrence. It is documented (not a hidden bug) and it is the
measured root of the misaddressing half of the failure. The fix is the
explicit untracked state (D/E).

## 7. Resource/capacity consequences

- B/D: zero new storage beyond the tag VALUE; capacity unchanged
  (K tracks, sum ≤ t_e; exhaustion semantics frozen). Retagging
  redistributes weight from the old track's budget to the new one —
  the old track's working target re-normalizes (its residual working
  mass scales up mildly; protected mass untouched).
- C: K× accounting and tighter effective caps; rejected.
- E: the untracked floor adds NO weight (theta_prune floor is the
  existing constant); the pre-track shared budget is Phase I's budget
  — same total.

## 8. Symmetry implications

First-contact rules are order-blind: for bac, C-channels fire first
at pres 21; for bca, A-channels fire first — the rule reads only
"channel fired while context c* active". Measured counterfactual:
both s20260912 orders cross identically (33×/18× → same drive
endpoint). IL stability: claims happen in the first few presentations
of each pattern and never re-fire-retag — the measured rule-intact
IL arms (3/3 pass) are the projection surface.

## 9. The SINGLE smallest architectural capability that must change

Change the M1 initialization semantic from "assigned at birth
(default 0)" to "UNCLAIMED until first co-activation" — i.e.,
candidate E's core: an explicit untracked state (one tag value),
first-contact claim into the post's current learned context (B's
rule, label-free, using only fired channels + the neuron's own
prototypes), and churn-exempt dormancy at the existing θ_prune floor
for the unclaimed class (the preservation half, needed by the
s424242 cells). One state value; one rule; zero new parameters
(theta_prune reused); no new timescale; flags-off byte-identity
preserved. This is D + B's mechanics + the floor — and the audit's
counterfactual shows it would have crossed the three preserved cells
while the floor is what the two unpreserved cells require before any
addressing can matter.

STOP — audit complete. No implementation, no new runs, no tuning,
no E-number.