# X-series drive-gated write — READ-ONLY MECHANISM REVIEW

Status: MECHANISM REVIEW, 2026-09-20. Read-only; no new runs, no
implementation, no tuning, no E-number. Failed diagnostic
preserved exactly as-is (corrected batch, commit 2923a90).

## 0. Verdict up front

**The failure is fundamentally caused by the drive normalization
— specifically a dimensional/scale error in the I_aff definition,
not by the gating concept.** The gate computed afferent drive as
bare synaptic weight flux (Σ w), while the membrane actually
receives **amplitude × w** per spike (network.rs:840). The
config amplitude is 52.0, so the gate underestimated the true
drive by ~52×; dividing by t_e=0.8 (a dimensionless weight
budget, not a current scale) compressed it further (~0.6‰ duty).
Net: g ≈ 0.7% of the scale that would have produced persistent
state, u collapsed to ~0.01, endogenous activity died, and the
information question became unanswerable — exactly what the
norm guard then recorded.

A principled normalization exists and uses no new gain: the
per-neuron EPSP flux from input-channel afferents,
Σ(amplitude·w)/v_th, with v_th = 1.0 the existing LIF
threshold — "threshold units of afferent input". Estimates
below show it would operate at g ≈ 0.28 during drive (not
0.004) and sustain nonzero selective u.

## 1. Units and scales (Q1)

Empirical, from the committed runs (drive-end snapshot +
telemetry input spikes, xdg-exp-bac-s20260912):

- **I_aff (as implemented)**: per-tick sum of bare w over
  spiking input→internal afferents. Units: weight-flux
  [w/ms]. Whole-window sums: median 2.67, range 0.76–7.19
  (50 internal neurons); per-tick median ≈ 0.005.
- **Membrane afferent current**: amplitude·w per spike, 52×
  larger. Per-tick median ≈ 0.28 (range 39.7–373.9 over the
  full 500 ms window).
- **t_e = 0.8**: dimensionless M2 excitatory weight budget
  (sum-of-w target per neuron). It is a total, not a flux;
  not a current scale.
- **β = 0.0046875**: per-spike u increment, in u units.
- **u**: same units as membrane voltage (added to dv as +u;
  baseline ‖u‖ norms 2.18–14.60).

So: I_aff has [w/time] units, t_e [w] — the ratio is [1/time]
(a fractional activity per tick), which is a legitimate *activity
fraction* but not the *drive strength* the neuron integrates.

## 2. Empirical I_aff distribution during sensory drive (Q2)

Measured (instrument examples/iaff.rs, read-only):
500 ms A-presentation, 50 active internal neurons:
- gate quantity Σ w per window: min 0.76 / med 2.67 / max 7.19
  (per tick ≈ 1.5e-3–1.4e-2 — matches the stated ~1e-3–1e-2)
- membrane quantity Σ(amplitude·w): min 39.7 / med 139 / max
  373.9 (per tick ≈ 0.08–0.75)

The stated "I_aff ≈ 1e-3–1e-2" is correct for the gate's
definition and wrong for the neuron's actual afferent drive by
the amplitude factor.

## 3. Is I_aff/t_e meaningful? (Q3)

Two defects, separable:
1. **Missing amplitude (52×)**: the gate summed w; the
   membrane's driving term is amplitude·w (network.rs:840:
   `current = s.amplitude * s.w`). The gate measured 1/52 of
   the relevant quantity.
2. **t_e as denominator**: dimensionally [w]/[w] = 1/time
   fraction — meaningful as "fraction of the weight budget
   firing per tick", but t_e is a *budget ceiling* that
   normalizes weights in M2; nothing in the membrane reads
   "w/t_e". Conflating the M2 bookkeeping scale with the
   physiological (EPSP) scale compressed the signal a further
   ~1.25× and, more importantly, tied the write gain to a
   mechanism (M2) that is itself under test for unwanted
   reallocation.

A meaningful normalization must be in the neuron's own
integrator units: EPSP (amplitude·w) over threshold (v_th).

## 4. Principled normalization from existing quantities (Q4)

**x_i(t) = Σ_{j∈input channels firing at t} (amplitude·w_{j→i}) / v_th**

- v_th = 1.0 (existing LIFParams, frozen), amplitude = 52.0
  (existing config, frozen). **Zero new gains.**
- Units: input charge expressed in threshold units — the
  classic "EPSP/threshold = excitability" ratio, dimensionless,
  strictly local, curriculum-blind, D8-boundary-scoped.
- Completeness note: the per-tick x is the *flux*; the EMA
  (τ_g=20 ms) then represents the recent afferent drive in
  threshold units. Empirically during drive: Σ(amp·w)/v_th per
  tick ≈ 0.08–0.75 ⇒ g ≈ 0.28 median — a *graded* drive, not
  the ~0.004 of the implemented gate.

(Conceptual cousin: normalize by the single-synapse EPSP
amplitude × typical w — same scale family; dividing by v_th is
the cleaner universal.)

## 5. Candidate comparison (Q5 + Q6)

| candidate | locality | curriculum-blind | no recurrent contamination | sustains nonzero u | pacemaking risk |
|---|---|---|---|---|---|
| raw Σ w (implemented numerator) | ✓ | ✓ | ✓ (input-only) | ✗ (~52× shortfall → collapse, measured) | none (u dead) |
| Σ w / t_e (implemented) | ✓ | ✓ | ✓ | ✗ (collapse, measured) | none (u dead) |
| **Σ(amplitude·w) / v_th** | ✓ | ✓ | ✓ | **✓ (~0.28 drive; u ~0.5–4 est.)** | low: off-window g→0 (τ_g=20ms); no off-window consolidation |
| spike/event-weighted drive (input-spike count EMA, no w) | ✓ | ✓ | ✓ | ✓ (scale by counts) | low (same τ_g) |
| total i_syn (all classes) | ✓ | ✓ | **✗ recurrent-contaminated** | ✓ | higher (recurrent feed) |

- Raw spike count is dimensional and simple but weight-agnostic
  — it would treat a 0.01-w and 0.2-w afferent identically,
  discarding the exact structure (weight-based drive) the
  network encodes; weaker discriminator.
- Total i_syn fails the no-recurrent-contamination test — the
  D8 boundary is load-bearing here; must stay input-only.

## 6. The BCA runaway explained (Q7)

exp-bca-s20260912 aborted at t=34,492 ms, `mean rate 77.6 Hz >
50 Hz for 5 s`. It is NOT a contradiction of the collapse: it is
a secondary trajectory effect of the gate.

- With u≈0, the network has no background pacemaker; all firing
  concentrates into the 500 ms presentation windows instead of
  spreading across the full 2 s cadence (baseline bca runs
  showed off-window rates 12–726 Hz *population*, i.e. the
  5 s moving-window mean stayed under detector in baseline
  because firing was spread out).
- Under the gate, drive-window summit density rises; the P2
  detector's 5 s sustained window (which in this curriculum's
  duty cycle ~0.25 sees presentations) crossed 50 Hz at seed
  20260912-bca specifically. The gate also changed STDP history
  (fewer off-window spikes → different weight evolution),
  which re-tunes the drive-window response per seed.
- So: a *concentration artifact* of zero-mean background firing,
  at one seed — consistent with the E2b lesson that this
  organism's stability depends on having a background firing
  regime; deleting it moves instability into the drive windows.
  (Also explains why only 1/8 cells aborted: seed-specific.)

## 7. What must change for a scientifically justified re-test (Q8)

The mechanism spec's I_aff definition only. Concretely,
the frozen diagnostic spec would need a correction amendment
(like any pre-frozen record — recorded, not silently changed):
- **I_aff_i(t) := Σ_{j∈input channels, j fired at t} amplitude · w_{j→i}**
  (the actual membrane-delivered current, network.rs:840
  semantics), and
- **x_i(t) := min( I_aff_i(t) / v_th , 1.0 )** with v_th = 1.0
  (existing LIF constant), replacing the t_e denominator.
- Nothing else changes: τ_g = 20 ms, EMA form, g bounds,
  gated write u += β·g, flag semantics, seeds/curricula/arms,
  verdict rules, identity gate.
- Rationale for justification bar: this is a *correction of the
  wire definition* to match the substrate's own per-spike
  current semantics (amplitude·w is literally the code's
  deposit expression), not a new gain and not a parameter tune.
  The prediction to preregister: g_drive during drive ≈ 0.2–0.3
  (vs 0.004), u norm ≈ 0.5–4 (vs 0.01), off-window consolidation
  ≈ 0 (τ_g decays e^-10 by 200 ms), frictionless run of the same
  16-run design; verdict rules unchanged.
- The t_e-denominator choice is abandoned on principle: t_e is
  M2's weight-budget bookkeeping unit, dimensionally incompatible
  with the per-spike current delivered to the membrane.

## 8. Preserved records

- Failed diagnostic: corrected batch (runs/xdg-*), 8 defective-
  batch runs (non-data, marked in spec record), 1 abort, all
  metrics/snapshots; commit 2923a90. Untouched.
- Read-only instrument added: examples/iaff.rs (empirical
  I_aff scale audit).
- No runs executed for this review; no code changed.

STOP after mechanism review.

---

# APPENDIX: boundary-detector benchmark (READ-ONLY, commit fee5c96 x-segcap §6)

Thresholds DECLARED before inspection (x-segcap §1 recorded scale:
S1 silence floor 0.0002, i_syn zero-floor):
  S1 onset   : S1 > +0.0020  within [t, t+500)     (10x S1 floor)
  S1 offset  : S1 < -0.0020  within [t+500, t+2000) (symmetric)
  i_syn off  : i_syn < 0.0010 within [t+450, t+2000) (10x zero-floor)
No post-hoc optimization; single fixed rule.
Instrument: examples/benchmark.rs (read-only; committed).
DP: detector-capable = >=1 input afferent w>0 (52/52; note the 3
non-plastic-only neurons counted by benchmark but not by wdiv's
plastic-only 49/52 — documented).

## A/C cell (v21repac-b0.003125-t10000, 80 pres)
S1 onset   : TP 66.1% (neuron-pres pairs), lat median 12 ms, p90 40
             ms; FP in 20 s silence = 0 (0/52 neurons).
S1 offset  : TP 65.8%, lat median 0 ms, p90 17 ms; FP = 0.
i_syn off  : TP 64.9%, lat median 77 ms, p90 92 ms; FP = 0;
             mid-presentation dips 5,381 (103/neuron) = the
             i_syn offset detector false-falls ~1.3x per
             presentation DURING the stimulus at the 0.001 level.
FULL i_syn (recurrent-inclusive upper bound; M6 inhibitory not
recoverable from snapshots => overestimate): FP in silence 875
(36/52 neurons) — i_syn as a TOTAL-current detector false-
triggers during pacemaking when recurrent deposits are included
(upper bound; true inhibitory contribution unknown).

## beta=0.00625 pacemaker cell (v21rep-b0.00625-t5000, 40 pres, A-only)
S1 onset   : TP 96.4%, lat median 7 ms, p90 43 ms; FP = 0.
S1 offset  : TP 95.8%, lat median 0 ms, p90 16 ms; FP = 0.
i_syn off  : TP 95.2%, lat median 78 ms, p90 90 ms; FP = 0 for
             INPUT-ONLY i_syn; FULL upper bound FP = 40
             (40/52 neurons) — one fall per neuron across the
             20 s silence. Mid-presentation dips 3,168 (61/neuron).

## RAW VERDICT (no claim of 'best')
- S1 (input-current fast-vs-slow diff) is a candidate EPISODE-
  BOUNDARY detector: detects onset (lat 7-12 ms) AND offset
  (lat 0 ms) with ZERO false positives in silence in BOTH cells,
  including the pacemaker run. TP per neuron-pres ~66% (weak
  cell) / ~96% (wedge), improving with drive strength; per-
  presentation coverage is higher than per-pair TP (any of 52
  neurons may fire). FP=0 holds BY CONSTRUCTION (input current
  exactly zero with no input spikes) — the 0.0002 measured floor
  is below the 10x threshold.
- i_syn (input-only) is an OFFSET-ONLY detector (onset not
  detectable; offset lat 77-92 ms = the 5 ms-tau drain through
  the 0.001 level), FP=0 input-only but mid-presentation
  self-dips are frequent (103/neuron over 80 pres); as a FULL
  total-current detector (recurrent-inclusive upper bound) it
  false-triggers during silence in both cells (36-40/52
  neurons). i_syn therefore FAILS the local-boundary
  requirement whenever recurrent contribution is real (the
  pacemaker run demonstrates this).
- Neither per-pair TP is near 100% at the weak cell, but S1
  does not false-trigger in the pacemaker; i_syn does (upper
  bound) and false-dips mid-stimulus.

## Minimum-experiment outcome
S1 is the only candidate satisfying all local-boundary
requirements measured: onset+offset, zero silence FP in the
worst (pacemaker) case, strictly local, pattern-agnostic. i_syn
is excluded by (a) offset-only function, (b) mid-presentation
dips, (c) recurrent-induced silence FPs (upper bound). Whether
S1's 66-96% TP at fixed threshold suffices for a mechanism is a
threshold/architecture decision NOT made here (mandate: no
architecture). Benchmark STOP.
