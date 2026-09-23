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

## D-46: output-band lateral inhibition (REGISTERED 2026-09-23)

CONTEXT: D-45 telemetry (beat 0-22, seed 424242) showed the collapse is
a DEGENERATE-OUTPUT ATTRACTOR, not readout dilution: the internal pool
collapses onto driving 2 output neurons (channels 1,5) to 250-cap while
10 go silent -> out=[0,250,0,0,0,250,0,0,0,0,0,0] IDENTICAL for A/C/D,
r decays 1.00->0.00. This organism is ALL-EXCITATORY (no inhibitory
synapses - E4d). The output band has no element opposing 2 neurons
capturing all drive.

MECHANISM (charter-allowed: plasticity law at I/O boundary): each firing
OUTPUT-class neuron deposits inhibitory current onto every OTHER
same-tick OUTPUT neuron (output_inhibition_gain, default 0 = identity).
Distinct from inhibition_gain (E3b = ALL non-input, worsened separation)
and d_ing (identity-gated). Readout-band only - does not touch pool
plasticity or growth.

PROBE (seed 424242, the collapsing case, 120 beats):
  oi=0.0  die beat 22 (degenerate attractor) fit 0.403
  oi=0.05 die beat 15 (too weak) fit 0.361
  oi=0.1  36 born +996 syn survive to beat 108 fit 0.762 (real runaway
           at 283 Hz only) | out[] NOT degenerate (recognition true)
  oi=0.2  38 born +1203 syn survive beat 112 fit 0.760
BREAKTHROUGH: oi>=0.1 breaks the degenerate-output attractor on the
collapsing seed - it grows instead of seizing.

CAVEAT (why GA-search): fixed oi=0.1 REGRESSES healthy seeds
(20260912 fit 0.10, 9001 fit 0.32 vs baseline 0.42-0.68). Per-lineage
optimum. => output_inhibition_gain wired as GA gene 9 (init 0.0, mutate
in 0..9 loop). Selection must discover each lineage's gain.

PREDICTION: with gene 9, seed 424242 does NOT collapse to all-zero (its
lineage finds oi>0); 20260912/9001 find low oi (>= baseline fitness);
mean_fitness across 3 seeds >= the c94f507 no-inhibition baseline.
FALSIFIER: if 424242 still all-zero at g7 AND gene 9 stays 0 (selection
never reaches oi>0), output inhibition is unreachable by this GA and
the segregation axis is closed too.
STOP.

## D-46 VERDICT: FALSIFIED as causal factor (recorded 2026-09-23)

TWO sequential tests, each refuting:
RUN 1 (multiplicative init-0, inert): gene 8 hovered 0.005-0.007
  (floor). 424242 best_g7=0.703 - but unreachable, so NOT attributable
  to inhibition.
RUN 2 (additive init-0, reachable): gene 8 drifted to 0.03-0.28 with 7
  organisms at 0.19 (the probe's working zone; reachability blocker fixed).
  424242 best_g7=0.703 - IDENTICAL to run 1.

CONCLUSION: making output inhibition REACHABLE and exercised (7 orgs at
oi=0.19) produced the SAME best fitness as the run where it stayed
inert. oi>=0.1 breaks the 424242 degenerate-output collapse in ISOLATED
probes (probe: die beat 22 -> survive to 108, fit 0.40->0.76), but under
full evolution it confers NO selectable advantage - the 424242 recovery
in GA runs is confounded (breed-order RNG shift + clamp change vs
c94f507), NOT inhibition. The collapse either does not recur under the
current GA config anyway (all 3 seeds grew to 152-202 g7), or inhibition
does not change the outcome selection reaches.

STATUS: output-band lateral inhibition = NOT a selectable mechanism for
the degenerate-output wall under this GA. The D-45/D-46 axis (readout/
output segregation) is CLOSED. 424242's earlier all-zero collapse did
not reproduce in D-46b runs - all seeds reached g7 sizes 180-202 with
improved/no-regression fitness vs the D-43-era collapse, indicating the
collapse point moved or the GA config drift resolved it. Residual open
question: whether the ~229 wall is real under the CURRENT config (only
424242 got to 202, close but not past the old 229 threshold in D-46b).
CLOSED pending that replication.
STOP.

## D-46 FINAL VERDICT (amended, controlled) - output inhibition: per-seed rescue, NOT selectable mechanism

DETERMINISTIC CONTROL (same seed+world, only oi differs - NO RNG drift,
the clean A/B the GA comparisons couldn't give). THE closing table:

  seed      oi=0.0                  oi=0.1                   delta fit
  20260912  die 127  fit 0.349      die 14   fit 0.102      -0.247  HURT
  9001      die 109  fit 0.592      die 35   fit 0.323      -0.269  HURT
  424242    die 22   fit 0.403      die 108  fit 0.762      +0.359  HELP
  MEAN      -        0.448           -        0.396          -0.052 NET NEG

=> oi=0.1 rescues ONLY the seizure-prone seed (424242); actively harms
the other two. Net across the ensemble: mean fit 0.448 -> 0.396 (NEG).
The mechanism is SEED-CONDITIONALLY beneficial and NET-HARMFUL - a
per-lineage trait (like a heritable allele) that a GA gene COULD in
principle express via lineage-specific retention, but the present search
dynamics did not (selection on the 20260912/9001 lineages correctly
purges oi>0; the 424242 0.19-retention is lineage-specific drift, not a
demonstrated selectable advantage).

GENE-8 x FITNESS pairing: RETRACTED. All verbose 'evolve 424242' runs
were actually 20260912 data - evolution IGNORED argv and always ran all
3 seeds (=== seed 20260912 === header). The correct-per-seed parse
requires the argv seed-filter fix (added D-46). This is why the claim
kept failing to reproduce. DO NOT use any prior GA-level gene-8 pairing.
The D-46 verdict rests SOLELY on the deterministic 3x2 control table
(birthprobe, which correctly parses a seed arg - unaffected by this bug)
+ the GA null (0.703 inert vs 0.703 exercised, both full-3-seed runs -
also valid).

VERDICT (amended): output-band lateral inhibition is a seed-conditionally
beneficial / net-harmful per-lineage trait - it breaks 424242's
degenerate-output collapse in isolation but is NET-NEGATIVE across the
seed ensemble (mean fit 0.448 -> 0.396) and confers NO selectable GA
advantage (best_g7 = 0.703 both with inert gene and with reachable gene
exercised at 0.19). The D-45/D-46 readout-segregation axis is CLOSED,
on clean evidence: the mechanism is real for one seed but selection
cannot exploit it without per-seed hand-tuning (forbidden).

HONEST re-scope of the ~229 wall (advisory-corrected): the prior wall
evidence (424242 all-zero at ~229) came from the D-43-era config. Under
current config, D-46b reached ~202 mean (424242 hit exactly 202) but NO
organism crossed 229, so "wall real" vs "wall gone" is UNTESTED above
~205 under current machinery - the D-46b non-collapse at 202 is
expected under either hypothesis. The wall question needs a deliberate
size-push run (more gens or relaxed budget) to cross 229 - NOT settled
by this verdict.
STOP.

## SIZE-PUSH VERDICT: the ~229 wall is REAL under current config (424242, 16 gens)

The size-push (the decisive open question) ran with the argv-fixed
evolve: `evolve 424242 16` (isolated seed, 16 generations).

  gen  sz    best fit
  6    215   0.654   (healthy growth)
  7    242.8 0.423   (warning: 0.42/0.34/0.25 + one 0.00)
  8    244.0 0.000   (ALL organisms dead)
  9-15 244.0 0.000   (irrecoverable)

RESULT: 424242 grows stably to ~215, then the moment it crosses into
~229-244 ALL four organisms' RECOGNITION COLLAPSES in the SAME
generation (gen 8, fit->0.000 while mean_sz holds 244). The wall is
SHARP and REPLICATED under the current (argv-fixed, gene-8 reachable)
config - NOT a D-43-era config artifact.

PRECISION ON MECHANISM (advisory-corrected): gen-8 fit=0 is RECOGNITION
COLLAPSE, not necessarily monitored death. evolve does not surface
died_at/fail_kind (0 'failed=' in log), and prior birthprobe evidence
(D-45 telemetry) shows this collapse occurs with activity IN-BAND
(a=1, rate ~88 Hz) and recognition r->0 - i.e. organisms ALIVE but
UNRECOGNIZING, not runaway (the 280 Hz monitor did not fire) and not
silent-death. So the claim is: growth past ~229 collapses RECOGNITION
while the organism stays active - the bounded resource is recognition,
not neurogenesis (bodies keep growing to 244).

MECHANISM (per D-45 telemetry): past ~229 neurons, growth-driven
reciprocal excitation collapses the readout into the degenerate-output
attractor (2 output neurons saturate at 250-cap, out[] identical for
A/C/D, recognition -> r=0). This is the E4d amplifier seizing the
output band.

CONCLUSION: the ~229 recognition wall is structural - it survived every
intervention (dynamic fixes E3/E3b closed, output-competition D-46
falsified as selectable, readout-gating D-45 de-registered). The binding
problem in this organism is REPRESENTATIONAL and SCALE-COUPLED: the
single shared pool cannot hold separable assemblies past ~229 neurons;
growth beyond it degenerates the readout. The self-construction is
bounded by a structural ceiling the plasticity laws + growth shape do
not (and, per E3b representational verdict, cannot via dynamics) escape.

STATUS OF THE GROWTH-TO-RETAIN GOAL: neurogenesis is VIABLE to ~215-229
(3-4x baseline), then hits a hard structural wall. The honest claim
stands: the organism grows ~4x and retains recognition through that
growth, then the shared-pool ceiling caps it. Options for a next
registered intervention: (a) compartmentalized pools (grown neurons
recruit into NEW pools instead of the shared one - attacks the
representational ceiling directly); (b) accept the ~229 bound as the
self-construction ceiling and pivot to other axes.
STOP - wall replicated; structural.
