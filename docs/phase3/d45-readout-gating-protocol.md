# D-45 PRE-REGISTRATION: gated/decoded readout — readout limit vs storage limit

Date: 2026-09-23. FREEZE PROTOCOL (no implementation until user signs).
Instrumentation-side measurement change — does NOT touch the organism, its
network, growth, or plasticity. Bound by charter: organism output is
separate from human-readable decoder/instrumentation; the decoder is
allowed to be hand-designed.

## Background (evidence)
- E4/E-4-fix chain produced VIABLE neurogenesis: 2/3 seeds grow 3-4x
  (40 -> 195/230 neurons) with improving fitness under selection.
- BUT: seed 424242 collapses at ~229 neurons via RECOGNITION collapse
  (r=0). D-44 showed it's not a fitness-gradient artifact (flat fitness
  floor could not rescue it).
- E3b registry verdict: assembly separation is REPRESENTATIONAL - every
  readout mixes all patterns via random 3.8% connectivity (shared pool).
- Current decode (io.rs): plain spike-count over FIXED output neurons
  64..76, accumulated per beat; refs = plain-cos template; decode =
  plain-cos argmax + amp floor. All 40+ grown neurons' activity must
  funnel through this 12-neuron slice.
- HYPOTHESIS under test: as the pool grows past ~229 neurons, the 12-
  neuron output readout MIXES multiple patterns (too few output channels
  to separate a growing population's assemblies) -> recognition collapses
  even though the assemblies themselves are intact. This predicts the
  wall is a READOUT capacity limit, not a storage/connectivity limit.

## Instrumentation change (measurement only)
Replace the plain spike-count + plain-cos decode with a GATED decoder:
each output neuron's identity is not 'A' or 'C' by fixed slot; instead
the decoder computes the readout by MAX-GATING: for each pattern ref,
score = max over output neurons of (contribution to that ref), with
winner-take-all over the 12 output channels. This is a pure
decoder/prediction change - the organism still emits the same 12 output
spike counts; only the interpretation (which pattern) changes.

CONCRETE (to be detailed at implementation, after sign-off):
  decode_gated(out, refs, q_floor, th_known):
    - same amp floor (QUIET)
    - for each ref p: score(p) = max_j ( out[j] * ref_p[j] )  [max-gate]
    - argmax over p; > th_known -> p; else NOVEL
  This is the MINIMAL readout change: it does not build a new decoder,
  it changes how the EXISTING 12 output channels are read.

## Alternative arm (falsifier control, same instrumentation boundary)
  decode_pc (plain-cos, current): run in parallel on the same runs for a
  within-run A/B - so both readouts see IDENTICAL spikes and the only
  difference is decoder. If gated fixes the wall but plain-cos does not,
  that CONFIRMS readout limit (same data, different decode). If BOTH
  fail at the same ~229 neurons, that FALSIFIES readout limit and points
  at the afferent/connectivity or storage side (E3b's claim).

## Registered predictions (falsifiable)
P1 (readout-limit): gated-decode organism survives past ~229 neurons
   (seed 424242 no longer collapses at recognition; r_known stays high;
   known_recognized_frac ~ 1.0 through 300+ beats), while plain-cos same
   runs still collapse ~229. => readout limited.
P2 (storage-limit, falsifier): gated does NOT change the collapse point;
   both return r=0 at ~229. => wall is upstream of readout (afferent
   connectivity / pool capacity). This would CONFIRM E3b's representational
   verdict and close the 'readout' axis too, leaving a storage-bound.

## Success criteria
GATED survives 300 beats at seeds {20260912, 9001, 424242} with
known_recognized_frac >= 0.9 AND no runaway AND no death, while
PLAIN-COS same adds fall to ~229 on 424242. >=2/3 seeds must reach the
higher size sustained for PASS/readout-limit.

## Clean start / no tuning
- Same world_seed, same S1 schedule, same growth genes, same GA init as
  c94f507 baseline. Only the decoder fn differs (instrumentation flag).
- Registered as FLAG (e.g. READOUT_GATED=1 env or config), default off
  preserves existing plain-cos behaviour (bit-identical baseline).
- Post-hoc changes to th_known/q_floor are NOT allowed; freeze them at
  baseline values (th_known, q_floor unchanged from survival spec).

## Honest failure mode
If P1 fails (both collapse), this does NOT mean growth is dead - it means
the recognition wall is storage/connectivity, and the readout axis is
closed the same way E3b closed the dynamics axis. That is still progress:
it would bound the ALL axes (dynamics E3/E3b, growth E4, readout D-45)
and point the program at the true blocker (afferent structure / storage
capacity), which would be the next pre-registered target.
STOP.

## PRE-IMPLEMENTATION FALSIFICATION (telemetry, beat 0-22 of seed 424242)

Added BEFORE implementing the decoder, per advisor concern: ground the
failure mechanism first. DBG_SURV per-beat output captured for a
growth-to-collapse run (424242, 400 beats, w_scale 0.1, monitor on):

  beats 0-5:  out distributed across ~6-8 channels (real structure),
              recognized=true, r 1.00 -> 0.83
  beats 6-13: channels 1 and 5 ramp toward cap
  beats 14+:  out = [0,250,0,0,0,250,0,0,0,0,0,0] IDENTICAL for A, C, D
              recognized=false, r decays 0.50 -> 0.00 by beat 22

FINDING: the collapse is NOT readout dilution. The 12-neuron output
band is a dedicated readout; instead the ORGANISM degenerates into a
tonic-saturation attractor - the internal pool collapses onto driving 2
output neurons (channels 1,5) to permanent 250-cap firing while all 10
others go silent. Every input produces the same degenerate vector, so
decode returns NOVEL for A, C, AND D regardless of decoder.

=> A gated decoder CANNOT address this: there is no signal left in out[]
to decode (identical 250-250 for all classes). The wall is a DEGENERATE-
OUTPUT ATTRACTOR in the organism's own dynamics - the E4-family
reciprocal-excitation amplifier (E4d, now seen seizing the output band).

VERDICT ON D-45: readout-limit hypothesis FALSIFIED at step 1 by
telemetry, before any decoder code. The readout is not the bottleneck;
the organism's output degenerates. D-45's gated-decoder arm is
DE-REGISTERED (would not change the identical-output regime).

NEW HYPOTHESIS (target for next registration): the recognizer needs an
output-segregating mechanism so the internal pool CANNOT collapse all
its drive onto 1-2 output neurons. Candidates (each to be registered
separately): output-channel homeostatic competition at the organism
level, or inhibitory gating on the output band (not the identity-gated
d_ing, which failed; plain pool-level inhibition on output projection).
STOP - do not implement gated decoder; re-scope to degenerate-output.
