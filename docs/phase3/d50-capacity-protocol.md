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

## D-50 VERDICT: FAIL - shared pool caps at 2 known; 3rd symbol breaks pairwise separation irreversibly

Measurement (decode-against-refs falsifier, corrected per advisory):
present each known symbol FRESH, decode via frozen io codebook vs refs.
Isolation control first (confound removed):
  legacy  8ch A/C   sep 1.0     (baseline)
  d50-2   6ch A/C   fit = baseline (.511/.532)  <- channel squeeze ZERO effect
  d50-3   6ch A/C/E sep 0.0-0.33               <- 3rd symbol E breaks A-C
So the collapse is CAUSED by the 3rd symbol, not the 6-channel layout.

Full 3-seed x 8-gen D50-3 (deterministic, D50_MODE=1):
  sep distribution (83 orgs): 64 at 0.333 (chance!), 10 at 0.667, 5 at
  0.000, 1 at 0.75, 1 at 0.50, 1 at 0.25, 1 at 0.083. NO organism ever
  reaches the 0.7 PASS bar.
  fitness collapses: best 0.205/0.219/0.123 across the 3 seeds - selection
  CANNOT restore separation over 8 generations.
  sizes still grow (223/241/189) - growth works (D-49), but recognition
  of 3 knowns stays at chance.

CONCLUSION: FAIL branch confirmed in 2/3+ seeds (actually 3/3).
The single shared pool CANNOT cleanly hold 3 known patterns - adding E
irreversibly mixes A and C (pairwise separation -> chance), and
evolution cannot recover it. This is a REPRESENTATIONAL CONTENT limit
(not a capacity/size limit - sizes grew fine). The 'store more as it
grows' goal requires COMPARTMENTALIZED POOLS: grown neurons recruit
into SEPARATE pools per pattern so A/C/E don't mix. That structural
step is now the evidenced, mandatory next investment for the memory
goal.
STOP - capacity caps at 2 in a shared pool; compartmentalize next.

## RETRACTION (2026-09-23): prior D-50 verdict 863bd71 INVALIDATED by wiring bug

The 863bd71 FAIL verdict was built on BROKEN trains: every train call in
the d50 path used the legacy `io::symbol_trains()` wrapper (hardcoded
mode=""), so `E` resolved to EMPTY input (not in the legacy alphabet)
and A/C were presented with LEGACY 8-CHANNEL trains - the 6-channel d50
layout and the 3rd symbol were NEVER actually exercised. The run tested
the legacy 2-symbol organism with a phantom silence-E. All separation
numbers in 863bd71 are confounded.
FIX (committed): threaded mode through form_s1/capture_refs/
capture_separation (symbol_trains_mode(sym, d50_mode(), seed)) and
survival's world presentations; legacy mode="" verified byte-identical
(gen0 .511/.642, gen1 .532/.663 unchanged).
CORRECTED PRELIMINARY (trains now real): d50-2 (6ch A/C) A-C sep ~0.5
(chance) AND D probes decode as known (contamination) - so the 6-CHANNEL
SQUEEZE alone breaks separation for 2 symbols, and the novel probe is
compromised. The earlier 'squeeze has zero effect' (fit-match) claim was
measured with wrong trains and is ALSO retracted. The capacity question
is NOT yet answered - d50-3 (6ch A/C/E) must be re-run with correct
trains. Status: D-50 reopened, verdict pending corrected measurement.
STOP - verdict retracted; re-running with correct trains.
