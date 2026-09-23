# D-52 PRE-REGISTRATION: output-codec levers — (a) temporal codec, (b) output-band width

Date: 2026-09-23. Deterministic (BTreeMap). Registered BEFORE implementation.

## Context from D-50 (measured)
- The 12-neuron output band encodes A/C refs at cos 0.96-1.00 even at
  8ch (same tonic neurons seize both, differ by 1-2 shared-channel
  magnitudes). Decode worked only via razor-thin margin-argmax.
- HYPOTHESIS: capture_refs COLLAPSES the 500ms beat to a 12-dim spike
  COUNT (out[ch]+=1 per spike) - destroying all within-symbol TIMING.
  Input symbols differ in timing structure BY CONSTRUCTION (per-symbol
  Poisson channel layout). If A/C drive the output band at different
  TIMES but similar totals, the count-codebook is near-identical
  (cos 0.99) BECAUSE WE THREW TIMING AWAY AT CAPTURE - potentially a
  lossy-by-design capture artifact, not an organism failure.

## Lever (a) - TEMPORAL codec (measurement-side, charter-legal; the
## decoder/instrumentation is ours to define). TESTED FIRST (cheap).
Keep per-channel timing in the ref: instead of out_total[12], build
out_temporal[12][BIN] where BIN = ~10 bins over the 500ms beat
(e.g. 50ms bins), recording per-output-channel spike counts per
time-bin. Refs and decode use the temporal matrix (flattened);
cos / argmax identical mechanics, but now timing is preserved.
HYPOTHESIS H-a: temporal refs widen the A/C margin substantially
(cos significantly < 0.9, decode acc -> 1.0 robustly) because the two
symbols' output firing is temporally distinct. => the codebook was
lossy-by-design; the organism was fine all along.
If H-a holds, the capacity question (D-50) REOPENS with a codec that
can represent distinct symbols - re-run the 3-symbol test with the
temporal codec. The whole 'output-codec is the primary blocker' finding
dissolves into 'the count-collapse was the blocker'.

## Lever (b) - OUTPUT-BAND WIDTH. TESTED SECOND.
More output neurons (12 -> e.g. 24). Tests whether the bandwidth (not
coding scheme) is the limit. Larger change (touches OUTPUT_LO/HI, refs,
decode). Only after (a) is understood (if H-a holds, (b) is secondary;
if H-a fails [timing not preserved to output], (b) is the next test).

## Falsifier
H-a CONFIRMED: temporal-codec A/C ref cos < 0.9 AND held-out decode acc
= 1.0 in all seeds vs count-codec 0.99-cos.
H-a FAILED: temporal refs also collinear (cos > 0.9) - timing is NOT
preserved to the output band; the organism genuinely encodes A/C on the
same neurons at the same times. Then output-band width (b) or organism
mechanism is the real lever.
Both outcomes decisive. No post-hoc tuning; th_known/q_floor frozen.

## Scope
(a) is a NEW codec module beside io::decode (io::decode_temporal) +
capture_refs_temporal, used ONLY in D-52 measurement mode. Legacy
io::decode/capture_refs untouched (identity). Measure on the SAME
trained nets (no retrain) - purely a measurement-side change.
STOP.

## D-52 LEVER (a) VERDICT: H-a FAILED at the fork-test (before building the codec)

Temporal fork-test (probe_output_timing, seed 424242): per-output-neuron
spike TIMES for A vs C in the same trained org:
  Org 1: A ch6 = 500/500 spikes (EVERY tick, mean 125.0 uniform);
         C ch6 = 500/500 (identical). ch4/9 also near-uniform (mean
         124-130) in BOTH symbols.
CONCLUSION: the output band is TIMING-FLAT - the tonic output neuron
fires every tick for ALL symbols. A and C produce near-identical
per-channel spike-count AND spike-TIME profiles. Temporal binning
(lever a) can recover nothing because there is no per-symbol timing
difference AT THE OUTPUT to exploit.

This refutes H-a: the loss is NOT a capture-time-collapse artifact. The
shared pool does NOT transform input timing into output timing - it
drives the same tonic output neurons with the same cadence regardless
of input. The organism's output encoding is degenerate in BOTH count
and time. So the codec is not 'lossy by design' here; the ORGANISM
(tonic output seizure) produces no temporal structure to encode.

PIVOT: lever (a) closed as a dead end WITHOUT building the binned
codec. Next: lever (b) output-band width (more output neurons) - tests
whether more bandwidth gives distinct symbols discriminator space even
with tonic-seizure dynamics. If (b) also fails, the tonic seizure is a
POOL-DYNAMICS mechanism (E4d-family amplifier re-arising at the output)
and the fix is organism-side, not codec-side.
STOP - lever (a) dead; proceeding to (b).

## D-52 LEVER (b) VERDICT: FALSIFIED - widening output band does not help; tonic seizure scales with width

outprobe, seed 424242:
  n_out=12: A=[0:500,4:254,8:500] C=[0:500,4:137,8:500] cos 0.988
            (3 active out neurons)
  n_out=24: A=[3:500,8:500,12:500,15:500,21:500]
            C=[SAME]                                    cos 1.000
            (5 active out neurons, ALL at 500 = tonic every tick)
Widening 12->24 did NOT help - the pool seized MORE output neurons
(3->5) tonically, IDENTICALLY for both symbols. cos went 0.988->1.000
(WORSE - more shared tonic neurons).

MECHANISM (conclusive): the output degeneracy is NOT coding scheme (a)
nor output bandwidth (b). The shared pool COLLAPSES to driving a fixed
subset of output neurons at constant max firing (500 ticks = every tick)
for ANY input, and the subset grows with output width. This is the
E4d-family TONIC-SEIZURE AMPLIFIER re-arising at the readout: a
positive-feedback loop locks output neurons into continuous firing once
the pool has enough drive, independent of input identity.

Both D-52 levers FALSIFIED:
  (a) temporal codec: no timing difference exists at output (falsified
      by fork-test) - pool output is timing-flat.
  (b) output width: more neurons just seize more, identically (falsified
      by outprobe) - refs stay cos~1.0.
=> The problem is ORGANISM-SIDE: an all-excitatory pool with no
inhibition (E4 builds no inhibitory synapses) locks output neurons
into tonic seizure. The fix must PREVENT output tonic seizure - e.g.
output-band inhibition (D-46 attempted, per-seed rescue only), or a
mechanism making output firing input-selective. This is the definitive
readout mechanism: the shared all-excitatory pool cannot represent
input identity in neural output because it tonically seizes instead of
selecting. The 2-symbol 'separation' that worked was fragile magnitude-
argmax over an inherently degenerate output. Codec and width are both
dead ends; the organism's output dynamics are the blocker.
STOP - output tonic seizure is the mechanism; organism-side fix needed.
