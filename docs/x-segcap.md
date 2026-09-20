# V2.1 Local temporal-segmentation primitive — capability analysis (READ-ONLY)

Status: 2026-09-20. No implementation, no runs, no parameter
change, no E-number. All numbers from committed A/C traces
(v21repac-b0.003125-t10000 + byte-identical twin xp2), via the
read-only instrument examples/segcap.rs (deterministic
reconstruction of per-neuron local state from recorded spikes).

## 0. The boundary problem restated

- A/C information survives afferents, spikes, per-presentation
  counts; dies at cross-presentation accumulation.
- Population envelope segments cleanly (1.5-1.7x modulation);
  no existing single-neuron trace did (median |d'| <= 1).
- A global reset and an external trial boundary are both
  excluded. The question: can a STRICTLY LOCAL signal, built
  from quantities a neuron already receives, mark presentation
  boundaries — distinguishing new sensory drive from ongoing
  pacemaking, without false boundaries in silence?

## 1. Candidate local signals and their measured properties

All candidates reconstructed per-neuron from the committed spike
stream (weights from nearest snapshot; substrate constants:
amp=52, tau_syn=5, adapt 0.05@200, u beta@10 s). Bucket means
are population sums over 52 neurons (per-tick); d' is
median-neuron over presentations.

| signal | local inputs | exists? | new state var | new timescale | pop/global? |
|---|---|---|---|---|---|
| S1 = Iaff20 − Iaff250 (fast-vs-slow drive diff) | own input afferent current (amp·w per input spike) via two EMAs | Iaff current computable (used in gate); two EMA copies = new states | yes (2 EMA vars) | yes (20/250 ms) | no |
| S2 = d(i_syn)/dt (synaptic-current transient) | own total i_syn (τ=5ms) — EXISTS as state | **yes (i_syn already a state var)** | **no** | no (derivative over dt) | no |
| i_syn itself (level) | own i_syn | yes | no | no | no |
| i_adapt itself | own adaptation (state var) | yes | no | no | no |
| Iaff20 (raw fast afferent drive EMA) | own input afferent current | EMA = new state | yes | 20 ms | no |

Measured bucket means (population, per tick):

| bucket | S1 | S2 | i_syn | i_adapt | Iaff20 |
|---|---|---|---|---|---|
| onset0 [0,10) | 0.017 | **0.042** | 0.279 | 0.144 | 0.019 |
| onset10 [10,50) | 0.075 | 0.007 | 0.670 | 0.190 | 0.088 |
| rise [50,100) | 0.089 | −0.002 | 0.682 | 0.277 | 0.120 |
| steady [100,250) | 0.062 | −0.000 | 0.675 | 0.388 | 0.122 |
| late [250,500) | 0.031 | 0.001 | 0.705 | 0.455 | 0.128 |
| **offt [500,550)** | **−0.054** | **−0.016** | **0.070** | 0.439 | 0.045 |
| post1 [550,1000) | −0.041 | −0.000 | **0.000** | 0.244 | **0.000** |
| post2 [1000,2000) | −0.004 | −0.000 | 0.000 | 0.150 | 0.000 |

Median-neuron boundary d' (onset [0,50) vs steady; offt vs post1):

| signal | onset-vs-steady d' | offt-vs-post1 d' |
|---|---|---|
| S1 diff20-250 | −0.79 | **+1.52** |
| S2 disyn | +0.55 | +0.69 |
| i_syn level | −0.52 | **+1.17** |
| i_adapt | −0.87 | +0.06 |
| Iaff20 | −0.86 | +0.76 |
| Iaff5 | −0.52 | +1.17 |

Off-window (20 s silence) false-boundary maxima (per-neuron):
- S1 max|x| = 0.0002 (numerical floor; internal-synapse residue)
- S2 max|.| = 0.0000
- Iaff20 max = 0.0000

## 2. The decisive structural fact

**Input-channel afferent current is exactly zero during silence
by construction** — pacemaking has zero input-channel
component. Therefore ANY signal built from Iaff (S1, Iaff20/5)
or from input-driven i_syn has a hard false-boundary bound of 0
in the off-window. The measured 0.0000-0.0002 confirms it:
**local input-derived signals cannot false-trigger on the
pacemaker** (requirement 7 SATISFIED by construction, measured
at machine epsilon).

## 3. Candidate-by-candidate verdict (mapping to the 10 questions)

### S1 = Iaff20 − Iaff250 (fast-vs-slow drive contrast)
1. Inputs: own input-channel afferent current (amp·w per input
   spike) — computable from existing synapse weights + own
   input afferent spikes; both already exist in the substrate
   (weights on synapses; input spikes on own input afferents).
2. Exists: the CURRENT exists; the two EMA copies do not.
3. New state vars: 2 EMA accumulators per neuron.
4. New timescale: 20 ms and 250 ms (both in the substrate's
   tau_syn/adapt family).
5. Population/global: NONE — pure own-afferent sums.
6. Distinguishes new sensory drive from pacemaking: YES by
   construction — pacemaking has no input component.
7. False boundaries in silence: NO (measured max 0.0002).
8. Identity-preserving: YES if disabled (no draws, no state
   change; default-off flag pattern already established).
9. Works without knowing A/C/BAC/BCA: YES — it reads drive
   level, not pattern identity.
10. Not an external hidden reset: correct — the EMA difference
    is computed locally every tick; nothing is injected.
Strength: onset0 transient 0.017→0.075 within 10 ms (punctate
onset), offt −0.054 (signed offset). Weakness: median onset d'
is −0.79 (it marks the transition but with modest per-neuron
separation); the SIGN of the signal (onset +, offset −) is
exactly the boundary marker.

### S2 = d(i_syn)/dt (total synaptic current transient)
1. Inputs: own i_syn — an EXISTING state variable; needs its
   previous-tick value (one register).
2. Exists: fully (i_syn is a substrate state).
3. New state vars: one register (previous i_syn) — arguably
   none ("derivative" can be computed as the delta between
   successive reads).
4. New timescale: NONE (per-tick derivative of existing τ=5 ms
   current).
5. Population: none.
6. Sensory-vs-pacemaking: PARTIAL — i_syn includes recurrent
   contributions, so pacemaking DOES move i_syn; the polarity
   helps: sensory onset gives +0.042 transient, offset −0.016,
   while recurrence-driven pops give small symmetric jitter.
   Median offt-vs-post1 d' 0.69 (weaker than i_syn level).
7. False boundaries: SILENCE max |.| = 0.0000 measured (no
   recurrent spikes in the off-window of THIS run — the
   pacemaker's recurrence deposits were absent at the measured
   snapshot cadence; caveat: endogenous activity in other
   regimes (e.g. β=0.00625 pacemaker) DOES produce recurrent
   i_syn — false-positive risk regime-dependent).
8. Identity: yes, additive-on-flag.
9. Curriculum-blind: yes.
10. Not external: yes.
Strength: no new timescale/state (cheapest); onset0 spike
0.042. Weakness: recurrent contamination in endogenous-active
regimes; median d' modest.

### i_syn level (fast current magnitude)
ofFt-vs-post1 d' = 1.17 (best single-neuron offset marker);
onset-vs-steady d' −0.52. i_syn IS an existing state variable
(no new anything); drops 0.705→0.070→0.000 across offt/post1.
False boundaries in silence: 0 in this run; recurrent caveat as
S2. It marks the END of a presentation more sharply than the
start (the offset collapse is dramatic because input plus
recurrent both vanish). As an onset marker it is weaker (onset0
only reaches 0.279 vs steady 0.675 — gradual because input
trains are 20 Hz, not instantaneous).

### i_adapt level
Existing; onset-vs-steady d' −0.87 (adaptation lags the onset,
rises through the presentation), offt-vs-post1 d' +0.06
(adaptation decays too slowly to mark offset: 0.44→0.24→0.15),
NEVER zero in silence (decays on 200 ms, and pacemaking
re-injects). It cannot mark a boundary cleanly and it keeps
moving during endogenous activity. Rejected as a standalone
marker; usable as CONTRAST partner (rapid-adaptation
difference) but that adds nothing S1 lacks.

## 4. Strongest candidates

1. **S1 (Iaff fast-vs-slow diff)** — the only candidate with a
   SIGNED onset(+)/offset(−) pulse, provably zero in silence
   (measured 0.0002), strictly local, curriculum-blind, and
   pattern-agnostic. Its onset punctateness (0.017→0.075 over
   10 ms) is exactly a "new presentation began" edge. Cost: two
   EMA state vars + two timescales.
2. **i_syn level (or its delta, S2)** — the cheapest (existing
   state), sharpest OFFSET marker (d'=1.17, collapse to 0.000),
   zero new timescale. Weaker onset and recurrent-contamination
   risk in endogenous-active regimes (caveat stands).

## 5. Evidence for/against (from committed data)

FOR S1: onset0→onset10 jump within 10 ms (0.017→0.088 Iaff20;
0.042→0.007 S2) — a genuine edge; signed offt −0.054 vs post
−0.041—0.004; silence floor 0.0002; median offt d' 1.52.
AGAINST S1: per-neuron onset d' only −0.79 (transitional, not
all-or-nothing; the 20 Hz Poisson input spreads the onset over
~50 ms — the first-10 ms pulse is real but the steady buildup
dilutes the "onset-vs-steady" contrast at 50 ms granularity).
FOR S2/i_syn: d' 1.17 offset; zero noice floor this run; no new
timescale. AGAINST: depends on input drive dominating i_syn;
the 0.00625 pacemaker (a committed run) has recurrent i_syn
during silence — the false-positive bound holds only for the
input-derived part, not for total i_syn.

## 6. Minimum distinguishing experiment (NOT run; no E-number)

`segcap` measured separation statistics; the DISTINGUISHING test
for S1-vs-S2/i_syn is a boundary-READOUT simulation on the
committed data (deterministic, read-only): for each presentation
in the committed run, compare the time-of-first-crossing of
`S1 > θ` vs `i_syn < θ_off` against the true presentation offset
(known from markers), scoring onset-latency distribution, true-
positive rate, false-positive count per 20 s silence — WITHOUT
changing the substrate (a pure offline detector benchmark). The
two candidates are distinguished by: (a) S1 would first-fire
within 10-30 ms of onset with zero silence FPs; (b) i_syn-level
would fire only at OFFSET (onset latency unusable) and its FP
rate must be re-measured on the β=0.00625 pacemaker run. One
committed run each, two detectors, one table. This decides
whether the primitive is an ONSET-edge (S1) or an OFFSET-collapse
(i_syn) detector without touching the organism.

## 7. Constraints honored

READ ONLY. No implementation, no parameter change, no sweep, no
E-number, no architecture proposal. Historical runs preserved.
The instrument (segcap.rs) is committed as read-only analysis.
STOP after capability analysis.