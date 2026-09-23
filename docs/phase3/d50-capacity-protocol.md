# D-50 PRE-REGISTRATION: multi-pattern capacity under growth (does it store >2 symbols?)

Date: 2026-09-23. Registered BEFORE implementation. Deterministic
(BTreeMap). Streaming guard: falsifier text commits before the run.

## Question (DECIDED: option B = FIXED-BANDWIDTH capacity)
The organism holds A/C + NEVER-TRAINED D (NOVEL) in ONE shared pool. As
it grows, does it HOLD MORE known patterns (capacity scales) or CAP at
~2 (shared pool mixes, E3b)?

SCOPING DECISION (advisory): input channels are positional 1:1 to input
neurons (0-23), and the output band 64-76 is hardcoded. Expanding to
8-channel symbols (option A: 32 channels) would SHIFT the output band
and invalidate every constant - invasive, risks committed baselines.
DECISION: option B = FIXED-BANDWIDTH capacity. Keep 24 total channels,
re-slice to 6-channel EXCLUSIVE blocks (A:[0-5] C:[6-11] E:[12-17]
D-novel:[18-23]). Subsets remain exclusive (no shared input neurons) -
this tests 3 known symbols in the SAME total input bandwidth, which is
a legitimate and arguably MORE honest memory question (fixed context,
more tokens compete). It is NOT the pure 8-channel-per-symbol question
(option A, invasive, defers to a follow-up if B passes).

## Only variable changed
KNOWN_SYMS count: [A,C] (baseline identity) -> [A,C,E] (3 known). Same
growth budget, same size-scaled cap (k=200), same world_seed cadence.
Channel layout: 24 channels / 4 symbols = 6 channels each:
  A: [0..6)  C: [6..12)  E: [12..18)  D-novel: [18..24)
(D moves from 8-channel block to remaining 6 - novel probe retained.
Flag-off default = legacy A/C 8-channel layout, byte-identical.)

## Pre-registered falsifier (the advisory guard)
PAIRWISE SEPARATION, not single-vs-pair: for all N(N-1)/2 = 3 pairs
(A-C, A-E, C-E), compute per-presentation cross-cosine vs the correct
ref. A symbol is SEPARATED if its within-symbol cosine beats every
cross-symbol cosine (argmax over N refs = correct symbol) for a
measured fraction of its presentations. CAP BAR:
  PASS (capacity scales): >= 2/3 seeds show pairwise separation acc
  >= 0.7 across all pairs at MATCHED size to the 2-symbol baseline.
  FAIL (capacity caps): separation drops to <= 0.5 on any pair, or
  < 2/3 seeds reach the 0.7 bar - shared pool mixes at 3 known.

## Decode threshold
th_known frozen at 0.20 (survival spec). N-way argmax over refs (already
generic - decode iterates refs). Novel D must decode to NOVEL/UNSURE
(< th_known), NOT to a known symbol.

## Scope (the real work = plumbing, not compute)
- io.rs: ALPHABET + RATE_HZ per-symbol (add E; D gets 6 channels)
- form_s1/capture_refs: train refurbish KNOWN_SYMS
- survival.rs world: generalise known-set (world transitions, is_known,
  novelty/prediction-error pressure, approach/withdraw)
- decode: already N-way generic (verify)

## Honest outcomes (both decisive)
PASS -> shared pool stores 3+ known as it grows; capacity scales;
  strengthen to 4/8 later; the 'representational ceiling at 2' weakens.
FAIL -> shared pool caps at ~2; COMPARTMENTALIZED POOLS becomes the
  mandatory next structural step for the memory goal. Either way the
  'store more as it grows' question is answered on evidence.
STOP.
