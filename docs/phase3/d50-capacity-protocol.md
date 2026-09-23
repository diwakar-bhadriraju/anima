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

## D-50 SECONDARY FINDING (2026-09-23): the 12-dim output-band encodes refs near-degenerate even at 8ch - a codebook limitation, not pool capacity

Measured live (this codebook, seed 424242, legible ref vectors):
  LEGACY 8ch:  A=[500,0,0,0,254,0,0,0,500,0,0,0]
                C=[500,0,0,0,137,0,0,0,500,0,0,0]  cos(A-C)=0.988
                (org1: 0.993)
  d50-2 (6ch):  A=[500,0,0,0,0,500,...] C=[500,0,0,0,0,362,...] cos=0.92-1.00
A and C collapse onto the SAME 3-4 tonic output channels (0,4,8),
differing only in ONE channel's magnitude (254 vs 137). cos ~0.99.
=> decode works (in legacy) only via argmax over tiny magnitude diffs;
   6ch squeezes that difference under th_known -> chance separation.

IMPLICATION (overturns prior assumption): the earlier 'legacy A/C
separated, cross-cos 0.6-0.75' (from memory registry E3b) was a
DIFFERENT pipeline (pool-neuron chunk telemetry), NOT the 12-dim output
band. In the actual output codebook, A/C refs are ~0.99 cosines at ANY
channel width. The shared-pool -> 12-neuron-output encoding produces
nearly-identical templates for distinct inputs. This is a fundamental
12-neuron OUTPUT-BAND ENCODING limit, present even in the working
2-symbol case.

CONSEQUENCE for D-50: the capacity question (can the pool store N
symbols?) is UNANSWERABLE via this output codebook because the codebook
itself cannot represent distinct symbols (refs degenerate). D-50's
'did adding E break A-C' measured codebook collapse, not pool capacity.
AND: the 2-symbol survival 'success' (sep 1.00 in D-46-era) ran on
magnitude-threshold argmax over near-identical refs - fragile, not the
robust separation the narrative implied.

REAL NEXT LEVER (evidence-backed): the OUTPUT CODEC is the bottleneck,
not the pool. Options: (a) richer output encoding (more/heterogeneous
output neurons, or non-spike-count readout) so distinct inputs produce
distinct output templates; (b) compartmentalized pools if even a good
codec can't hold N>2. This reframes the goal: 'stores more as it
grows' first needs an output codec that can represent more distinct
symbols at all.
STOP - codebook degeneracy is the primary blocker, upstream of both
capacity and pool-organization questions.

## D-50 MECHANISM COMPLETE (2026-09-23): tonic-neuron seizure is SAME-per-symbol; codebook always degenerate

Full legacy 8ch refs (seed 424242, 4 orgs):
  org0 cos .988: A,C BOTH seize channels {0,8}; differ only ch4 254v137
  org1 cos .993: both seize {0,1,2,9}; differ ch10 497v500, ch11 134v0
  org2 cos .956: both seize {7,10}; differ ch8 391v469, ch9 0v249
  org3 cos 1.000: A,C IDENTICAL on {5,9,11} (497v500 clips same)
REFUTES the '8ch worked because tonic neuron differed per symbol'
hypothesis: the SAME tonic neurons seize for both A and C; separation
was ONLY the magnitude of 1-2 SHARED channels (254 vs 137). At org3 the
difference vanishes entirely (cos 1.000).
CONCLUSION (unchanged, now airtight): the 12-neuron output codebook
encodes distinct inputs as near-identical templates (cos 0.96-1.00)
even in the working 2-symbol case. 6ch just removes the last magnitude
margin. Codebook degeneracy = primary blocker, upstream of capacity,
pool organization, and decode. 2-symbol 'separation' was magnitude-
argmax over a razor-thin margin, never robust structure.
STOP - mechanism closed; output codec is the next registered lever.

## D-50 STRAIN BASELINE (2026-09-23, 424242 8-gen verbose) - strain is FLUCTUATION, not seed-cap

Corrected per-org/per-gen join (SEPSPLIT vs dead/fit/n lines):
  org-gens: 23 measured; post-survival sep >= 0.7 in 16 (70%)
  distribution: 1.0 x13, 0.67 x4, 0.33 x2, 0.42/0.75/0.83/0.92 x1 each
  - NO org is consistently strained: org0 0.67->1.0->1.0->1.0 (recovers),
    org3 0.33->1.0->0.67 (fluctuates). Every org hits 1.0 sometimes.
  - strain does NOT correlate with size (0.33 at n=91 AND n=251; 1.0 at
    both), with fit (0.11-0.23 uniformly), or with org index.
  - 1-org dies per gen (dead=Some(29/22/17), fit=0.000) but the survivors
    still fluctuate on sep.
=> the earlier '424242 seed is strained (0.42-0.67)' was 1-gen sampling
noise; at 8 gens the strain is GEN/ORG-RANDOM fluctuation around a
mostly-1.0 mean. Corrected: 3-symbol post-survival separation is
usually-clean with intermittent degradation - an INSTABILITY (which
gen/org dips is world/growth-dependent), not a capacity ceiling or a
seed cap.

The instability (0.33<->1.0 between gens for the same lineages, with
fitness stable) is the real target: WHY does separation swing between
gens when the organism's survival/fitness is stable? Candidate: refs
refreshed per-gen against a net whose output encoding swings with
growth (the post-survival state at gen k+1 vs k differs enough that
fresh refs re-measure different separation). THAT is consistent with
D-55: refs must match the tested state, and at this scale the state
changes every generation - so separation measurement is itself
state-sensitive. The honest question: is the swing REAL (the organism's
discrimination genuinely degrades/recovers with growth) or an artifact
of refs-vs-state timing (capture_refs at 338 measures the pre-survival
state, so the post-refs-measure is ALSO stale by one beat-window)?
This is the next measurement design question (refs and responses must
be sampled at the SAME instant, or refs must be continuously refreshed).
STOP - strain = instability; measurement state-timing is the open
question.

## D-50 STRAIN RESOLVED (2026-09-23): org 0 IS the entire strain - a synapse-starved lineage

Positional join (sep vs syn_growth, 8 gens, all 32 org-slots):
  org 0: STRAIN in ALL 8 gens (post-sep 0.33-0.67)
  org 1/2/3: PASS in ALL 8 gens (post-sep 0.83-1.00)
  syn_growth: strain mean -1308 (NEGATIVE - de-growing) vs pass +30
  org 0's synapses get PRUNED AWAY every gen (-5221, -6323 late) while
  the others build (+2129..+14978 late).
=> NOT stochastic fluctuation, NOT seed-cap, NOT birth-timing noise.
   424242's population contains ONE persistent low-synapse lineage
   (org 0) that fails 3-symbol separation; three healthy ones. The
   'strain' is a SYNASPE-STARVATION phenotype: M4 pruning/M3 failure
   eats its connectivity (de-growth) while the others build.

REFINES the prior 'fluctuation / instability' claim (4817faa): the
per-gen dips I read as instability were actually org-0-locked (my
positional join was wrong earlier - it assumed org index repeats; the
correct positional pairing shows it is org-locked). The fluctuation
was org-0's synapse starvation varying in DEGREE (0.33-0.67), not
separation swinging between healthy orgs.

CAVEAT: org 0 also showed STRAIN at d50-2 (2-symbol, 6ch) in the
earlier control - so org 0's lineage is persistent-degenerate even at
2 symbols: consistent with the D-53 pre-survival-degenerate lineage
that survival dynamics DON'T fix (its connectivity is being pruned
away, so maturity never converts it). The D-53 self-confirmation
mechanism applies to THIS lineage's survival-loop numbers.

CORRECTED CAPABILITY: 3/4 of 424242's lineages hold 3 symbols at
0.83-1.00 (robust). The starved lineage (org 0) cannot - synapse
starvation (de-growth) is ITS limit, not pool capacity. The goal-level
'stores more as it grows' holds for the 75% healthy lineages; org 0's
lineage needs synapse-growth regulation (the E4d angle: its M3/M4
balance is net-pruning), not pool reorganization.
STOP - strain = synapse-starved lineage; lever = growth-balance
regulation for THAT lineage, not pool restructuring.
