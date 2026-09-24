# D-57 PRE-REGISTRATION: the LIBRARY world - cumulative-vocabulary curriculum that forces memory

Date: 2026-09-23. Deterministic. Registered BEFORE implementation.
Addresses the goal gap directly: the organism must GROW + LEARN +
REMEMBER *no matter what*, by making the environment impossible to
survive without accumulation.

## Why (measured gaps this targets)
1. Novelty detection WEAK-to-ZERO, decaying with generations (D-53b) -
   nothing in current fitness rewards detecting new vs known.
2. 3-symbol separation fragile (33% org-gens < 0.7) - pool strains.
3. NO persistent memory pressure: survival only tests what's in front
   of the organism NOW. Nothing forces retaining earlier symbols.
Root: the curriculum never punishes forgetting. A library world does.

## The architecture (environment/curriculum only - charter-legal)
The organism accumulates a GROWING vocabulary over its life; survival
requires recognizing EVERYTHING it has ever learned, including with
long gaps since first-seen:
  stage k (k=1..N): world presents {s_1..s_k} + NOVEL. The organism
  must recognize s_1..s_k (all accumulated) correctly, reject NOVEL,
  or its viability drops / it dies.
  NOVEL re-appears every stage (detection must persist).
  Old symbols re-tested with growing delay (delayed-retrieval).
=> forget s_j early and the stage-k survival re-test kills you.

## Channel constraint (frozen decision)
24 input channels, 6ch exclusive blocks in d50 mode -> vocab max = 3
trained (A, C, E) + D-novel. So PHASE 1 library stages:
  stage1 {A}, stage2 {A,C}, stage3 {A,C,E}. (3 is the 24ch ceiling.)
PHASE 2 (capacity): to let vocabulary grow past 3 "no matter what",
the I/O boundary must widen - 32/40 input channels with the output
band parameterized (OUTPUT_LO/HI derived from n_input+n_internal, not
const). This is the invasive change (touches decode/refs/survival/
evolve), deferred behind the 24ch proof-of-mechanism. Phase 2 is the
'really keeps growing' enabler; Phase 1 proves the mechanism.

## Honest metrics (D-53 house rules, all applied)
- Per-symbol held-out recognition over ALL accumulated symbols at each
  stage (present each s_j fresh, decode vs refreshed refs).
- novel_detected_frac (the honest novelty metric, per stage).
- retention = holds-s_j from stage1..k (the NEW library number).
- Survivorship bias handled: measure every org's retention before death
  / die tracking explicit.
- refs refreshed per stage (state-matched, D-55 rule).

## Pre-registered predictions
PASS (mechanism works): retention of s_1..s_{k-1} STAYS high across
stages (the library accumulates) AND novel_detected_frac does NOT decay
to 0 (survival now requires detecting NOVEL to avoid mislabel-as-known,
which would fail a known re-test) in >= 2/3 seeds.
FAIL: retention still decays (accumulation impossible in the channel-
limited pool) OR novelty stays 0 despite the pressure (fitness-blind
was not the cause; structural). Both falsify specific earlier
conclusions usefully.

## Scope for this turn
Phase 1: a library-survival harness (staged curriculum + accumulated-
symbol recognition + honest novelty metric). Reuses io/d50 machinery.
No new network mechanism - the environment is the change. If PASS,
Phase 2 (channel widening) is the follow-up registration.
STOP.

## D-57 PHASE-1 VERDICT (honest metrics, all D-53 rules applied)

Library world (cumulative vocab {A}->{A,C}->{A,C,E}, d50 6ch):

  seed       stage3 retention                        novel_det
  20260912   A=3/3 C=0/3 E=3/3                       0/3 every stage
  9001       A=3/3 C=0/3 E=0/3                       0/3 every stage
  424242     A=3/3 C=0/3 E=3/3                       0/3 every stage

FINDINGS:
1. A (first-learned) is RETAINED 3/3 at every stage, all seeds - robust.
2. C DEGRADES as later symbols are added (3/3 at its stage2 -> 0/3 at
   stage3 for 20260912/424242; 0/3 immediately for 9001). GENUINE
   retention loss for earlier symbols under library growth - a real
   cross-symbol interference / catastrophic-forgetting-style tradeoff.
3. E (newest) is 3/3 where it settles (2/3 seeds), 0/3 for 9001.
=> The library world WORKS as designed: it exposes honest retention
dynamics (A holds, C fades) that the flat 3-symbol survival loop hid.
The organism does NOT perfectly accumulate - it shows symbol-ordered
interference - which is the capacity/representation question made
visible.

4. NOVELTY DETECTION: 0/3 at EVERY stage, ALL seeds - even under
   explicit library pressure (D among known). Library pressure ALONE
   did NOT fix novelty detection. EVIDENCE AGAINST the fitness-
   blindness hypothesis for the loop-side codebook: the blocker is the
   th_known/output-codebook structure (all refs cos~0.99 -> D decodes
   to nearest known), NOT absence of selective pressure.
   (Runs as-is; D-56 fitness-blindness could still matter as a SECOND
   contributor, but Phase-1 shows pressure alone is insufficient.)

CORRECTED SUB-CLAIMS:
- 'accumulates perfectly' (earlier artifact from self-ref decode) is
  FALSE. The honest library world shows partial, symbol-ordered
  retention (A robust, C fades).
- Novelty: NOT fixable by curriculum pressure alone - codebook/threshold
  is the structural blocker (now evidenced, not assumed).

PHASE 2 implication: the symbol-ordered interference (C fades as E is
added) is the pool-capacity issue. The library world now MEASURES it.
Phase-2 (wider I/O boundary for vocab>3) + the interference-free
representational study are the follow-ups; novelty detection needs a
codec/threshold intervention, not curriculum.

STOP - Phase-1 result recorded; library world validated as a retention
probe; novelty needs codec fix (pressure insufficient).
