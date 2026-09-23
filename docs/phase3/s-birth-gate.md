# Phase III sZ — birth-trigger gate probe (D-38)

Status: GATE FAILED (with a measurable mechanism). 2026-09-22.
Probe: birthprobe.rs — homeostatic-saturation birth trigger wired into the
survival life (organism's own novel/unrecognized beats -> prediction error
-> birth), V2 M3/M4 self-construction also active. Fixed the E4-fragility
bug: V2Plasticity::on_neuron_appended resizes internal bookkeeping when a
structural birth appends a neuron (previously OOB-panicked at window).

RESULT (seed 20260912, 100-400 beats, trigger rate {15,30,60,90}):
  rate  born  syn_growth  outcome
  15    3     -104        died beat 9, fit 0.000
  30    3     -104        died beat 9, fit 0.000
  60    3     -104        died beat 9, fit 0.000
  90    1     -144        died beat 9, fit 0.000

GATE VERDICT: neurogenesis FIRES (1-3 neurons born) but every birth is
followed by the organism's DEATH at beat 9 with fitness collapse and
heavy pruning (-104..-144 synapses) - at EVERY trigger rate. The
homeostatic-saturation newborn destabilizes the survival organism
(returns the organism to the E4-family death pattern). Birth is
mechanically possible but the default trigger's newborns are actively
harmful in the survival world.

IMPLICATION: under this trigger, evolution (D-38) would hard-select
AGAINST birth (neuron count stays 40; fit drives to 0 on any birth).
Whether a DIFFERENT birth rule (persistent-error trigger, other wiring,
gated birth conditions) survives is an open question - but the evidence
says the current trigger produces non-viable newborns at every rate.

DECISION (user, per prior agreement: "do b first, if something works out
we go for a"): gate FAILED -> the GA build (D-38) is NOT yet justified
for the homeostatic trigger. The choice is (1) try the OTHER trigger
(persistent-error) as a second cheap gate, or (2) accept that this
substrate's birth machinery is irreparably destabilizing and stop the
neurogenesis line per the E4 family, preserving the stable survival loop.
STOP - gate recorded; machinery + fix committed; evolution over this
trigger not justified by evidence yet.
