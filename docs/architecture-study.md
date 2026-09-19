# ANIMA Architecture Study — persistence substrate boundaries
# (design study; NOT an experiment, NOT an implementation)

Status: STUDY (2026-09-20). No repository modification, no
experiment, no E-number, no parameter choices. Grounding: E1–E23
record + repository dynamics code as cited.

## 0. The exact gap (from measured evidence)

The frozen substrate (network.rs:553–563) integrates:

    dv = (−(v−v_rest) + i_syn + i_ext − i_adapt) · dt/τ_m
    i_syn *= exp(−dt/τ_syn)          (fast current kernel)
    i_adapt *= exp(−dt/τ_adapt)      (HYPERPOLARIZING, τ≈200 ms)
    i_ext ≡ 0; no noise term; v_rest = 0

Every slow variable the organism already has is either
hyperpolarizing (adaptation suppresses firing), read-only
(rate_hz feeds telemetry/homeostasis, not the membrane), or
plasticity-gain-only (E6's φ, τ≈2.5 s, scales a⁺/Δ_perm — never
firing). Consequences, measured: temporal capacity < 50 ms (E21),
zero activity in any silent window (E19/E20), recurrence starved by
M2's zero-sum t_e budget under input-dominated statistics (E17).
So the missing degree of freedom is singular and precise:

**A slow, LOCAL, DEPOLARIZING state that feeds the membrane.**
Nothing in the current physics carries positive drive across
silence; every candidate must supply exactly that, and nothing
else, to count as minimal.

Category discipline: this is category 1 (physical substrate
capability). It must not smuggle in category 5 (task machinery).

## 1. Candidate mechanisms (analyzed, none assumed)

For each: fixes / enables / assumes / touches / local? / implicit
memory? / prescription risk / smallest validation.

### C1. Slow subthreshold depolarizing intrinsic state
One new per-neuron state u, generic and stimulus-blind:

    u += β·χ(spike) − (dt/τ_s)·u   (τ_s ≈ 1–10 s)
    dv = (−(v−v_rest) + i_syn + i_ext − i_adapt + u)·dt/τ_m

- FIXES: the persistence gap directly — u integrates the neuron's
  own firing history and injects positive drive for seconds.
- ENABLES: (i) state that survives gaps ≫ 50 ms and influences
  future dynamics (u biases v toward threshold); (ii) endogenous
  activity in silence if u can hold v above threshold — bistable
  intrinsic persistence; (iii) graded, self-organized "attention"
  without any assigned role.
- ASSUMES: one new physical fact — neurons have a slow
  activity-dependent depolarizing process (neuroscience analog:
  calcium-activated cation current; this is physics, not task).
- TOUCHES: neuron dynamics only (one state var, two params β, τ_s).
- LOCAL: strictly (per-neuron, own spikes only).
- IMPLICIT MEMORY: yes — and honestly so: it is a physical
  integrator, exactly the category-1 capability we lack. It carries
  no labels, no task structure, no assigned cells.
- PRESCRIPTION RISK: low. Nothing about u knows A/B/C, votes,
  gaps, or consequences. Whether any neuron's u becomes
  load-bearing is up to development.
- CONTAINMENT: the existing architecture already owns the brakes —
  M6 (anti-Hebbian inhibition), E6 (rate balancing), M2 (excitatory
  budget), adaptation. A positive-feedback intrinsic state inside
  a homeostatically regulated loop is the classic biological
  pattern (persistent firing + homeostasis); runaway is the
  predicted failure mode the existing machinery must contain.
- SMALLEST TEST: zero-gate identity (β=0 ⇒ byte-identical to V2,
  preserving E1–E23 as baselines) + a persistence probe: does
  stimulus-locked activity now extend past 50 ms / survive a
  500 ms silence in ANY neuron (not a chosen one)?

### C2. Tonic endogenous drive (i_ext > 0 or lowered v_th)
- FIXES: silence-activity only. ENABLES: spontaneous firing — but
  NO state: constant drive carries zero information about history.
- ASSUMES: cells are tonically active. TOUCHES: one parameter.
- LOCAL: yes. IMPLICIT MEMORY: no — which is exactly the problem:
  it gives noise, not persistence.
- PRESCRIPTION RISK: low; scientific value: low alone (E19-style
  readouts would read an unmodulated oscillator).
- SMALLEST TEST: silence-firing rate distribution. Verdict: useful
  only as a companion (see C4), never as the answer.

### C3. Stochastic spontaneous activity (membrane noise)
- ENABLES: intermittent endogenous spikes — random, stateless.
  With STDP, noise alone drives weight diffusion (M2 renormalizes
  it away); it cannot bridge gaps with information.
- ASSUMES: channel noise. LOCAL: yes. MEMORY: no.
- PRESCRIPTION RISK: low. FAILURE: turns readouts into coin flips.
- Verdict: same as C2 — a jitter source, not a persistence
  substrate; only interesting combined with C1 (noise + bistability
  = stochastic state transitions).

### C4. Slow synaptic current (NMDA-like second kernel)
A second per-SYNAPSE current with τ ≈ 1 s gated by recent
pre/post co-activity.
- FIXES: persistence in the CONNECTION, not the cell.
- ENABLES: seconds-long reverberation through existing synapses —
  but M2's t_e budget actively starves exactly these paths (E17
  evidence), so C4 fights the substrate's own economics unless M2
  is also changed — making it non-minimal.
- TOUCHES: synapse dynamics (every synapse gains a state var +
  2–3 params — ~10^3 new DOF vs ~76 for C1).
- LOCAL: yes. MEMORY: yes (synaptic, equally honest).
- PRESCRIPTION RISK: medium (dual-kernel synapses smell like
  designed memory); CONTAINMENT: harder to reason about.
- SMALLEST TEST: same probe as C1. Verdict: legitimate second step
  (V2.3), not the minimal one.

### C5. Persistent recurrent excitation (raise p_rec / lower M2 cap)
- FIXES: nothing new structurally — this is a PARAMETER of the
  existing architecture, already explored: v2 ablations showed M2
  is load-bearing for separation, and E17 showed recurrence
  collapses without dedicated support. Raising recurrence without
  a slow carrier still dies in ~100 ms (τ_syn).
- Verdict: not an architectural change; a tuning of one — excluded
  by the no-tuning rule and by evidence.

### C6. Intrinsic plasticity (slow homeostatic v_th / gain drift)
- ENABLES: neurons adjust own excitability over minutes —
  stabilizes endogenous activity once it exists; creates slow
  state as a side effect.
- Does NOT by itself create positive drive from silence (it
  regulates, not drives). Verdict: excellent V2.4 stabilizer /
  category-3 development mechanism; not the minimal persistence
  carrier.

### C7. Disinhibition (activity-dependent inhibitory depression)
Slow depression of M6 synapses would let recent activity
disinhibit future activity — persistence via released brake.
- LOCAL: yes. Enables: state + endogenous activity.
- RISK: high — it unbalances the one global stabilizer the loops
  rely on; interacts with E6/M2 unpredictably; and it makes the
  CONTAINMENT story worse rather than better. Non-minimal in
  consequence.

## 2. THE MINIMAL CANDIDATE

**C1 — one generic slow depolarizing intrinsic state per neuron.**
It is the only candidate that (a) supplies exactly the measured
missing capability and nothing else, (b) adds ~2 parameters × 76
non-input neurons rather than ~3 × every synapse, (c) leaves every
existing law untouched in form (M2 normalizes synapses, not
intrinsic currents; M6/E6/adaptation remain the brakes), (d)
degrades gracefully (β=0 ⇒ bit-identical V2, all of E1–E23 remain
valid baselines), and (e) carries zero task information by
construction. The neuroscientific precedent (intrinsic persistent
firing via calcium-activated currents) argues it is physics an
organism could plausibly have, not intelligence we are smuggling
in. "What is the minimum new physical degree of freedom required?"
— one scalar per neuron with a slow time constant and positive
sign.

## 3. THE MAXIMAL CHANGE WE CAN STILL CALL ANIMA

The charter: specify boundaries, local laws, resources,
environment; the organism determines its own organization. The
boundary is therefore not a component count but a SIGNAL-SEMANTICS
invariant:

**Permitted:** any local state variables and local laws (arbitrary
timescales), resource budgets, stochastic physics, structural
growth/death under budgets, homeostatic loops — provided no
variable's DYNAMICS encode task identity, valence, credit, or
global supervisory control.

**Excluded (category 5):** reward/error signals, eligibility traces
that reference actions-outcomes (vs plain timing), attention as a
top-down controller, neuromodulation that gates learning by
behavioral outcome, any assigned memory/decision cells, any readout
feeding back into dynamics.

By this invariant the maximum defensible architecture (V3-max) is:
multi-timescale intrinsic dynamics (fast LIF + slow depolarizing
state + intrinsic plasticity), dual-timescale synapses with
metaplastic consolidation, dormant-but-budgeted structural
growth/death active in closed loop, and strictly LOCAL diffusive
modulation (a volume-limited, activity-derived chemical that
changes local excitability but never carries valence or task
context). Everything remains "specify the laws, not the circuit".
What breaks ANIMA at the next step: any GLOBAL signal whose
semantics are defined by the experimenter's task (a reward), any
mechanism whose function presupposes the experiment's categories
(a "working-memory cell type"), or any supervision path — at that
point the system is a designed cognitive architecture and the
developmental question is void.

## 4. ARCHITECTURE CONTINUUM

| step | new DOF | capabilities unlocked | assumptions added | exp. cost | interp. cost | prescription risk | still learnable |
|---|---|---|---|---|---|---|---|
| **V2.0** (current) | — | transient filtering, representation separation, consequence-shaped routing | — | — | — | — | E1–E23 corpus |
| **V2.1** = C1 | u per neuron (β, τ_s) | persistent state ≫ 50 ms; possible endogenous activity; gap bridging | slow local depolarizing physics exists | low (1 identity test + capacity re-run) | low (one transparent scalar) | low | does capacity extend? does endogenous structure self-organize? do E18/E19 become satisfiable-in-principle? |
| **V2.2** = V2.1 + C2/C3 noise | noise amplitude | stochastic state transitions; silence is no longer information-free | membrane noise | low | low–med | low | state reliability vs noise; spontaneous activity statistics |
| **V2.3** = V2.2 + C4 slow synapse | synapse slow kernel (~10³ DOF) | synaptic persistence, seconds-scale association | synapses have slow component | med | med | med | synapse- vs intrinsic-carried state; interaction with M2 budget |
| **V2.4** = V2.3 + C6 intrinsic plasticity | v_th/gain drift | self-stabilized endogenous regimes; activity setpoints emerge | homeostatic excitability | med | med | low | stability/autonomy trade-offs; developmental epochs |
| **V3.0** = V2.4 + growth-on (existing dormant machinery under budgets, closed loop) | neuron/synapse counts | architecture grows with experience | development uses persistence | high | med–high | med | real developmental capability experiments (E18/E19-class questions finally reachable) |
| **V3-max** = V3.0 + local diffusive modulation (no valence) | local modulator field | arousal-like local gates; richer regimes | local chemistry | high | high | high (approaches boundary) | whether self-organized gating emerges without task semantics |

Beyond V3-max: reward/credit/attention/task state — NOT ANIMA
(designed cognition; developmental claim void).

## 5. COMPARATIVE ANALYSIS (condensed, per step beyond V2.1)

- **Locality:** every step preserves strict locality; V3-max's
  diffusion is volume-limited (local neighborhood), never global.
- **Parameter counts:** V2.1 +2; V2.2 +1; V2.3 +(2–3)×10³;
  V2.4 +2; V3.0 +0 (machinery exists, dormant); V3-max +3.
- **M2/M6 modification needs:** NONE through V2.2 (i_slow is
  outside M2's synaptic budget; M6 remains the brake). V2.3
  requires an M2 ruling (slow kernel inside or outside t_e?) —
  a real protocol decision. V3.0 unchanged (budgets already
  exist). 
- **Existing plasticity meaningfulness:** preserved at every step
  (STDP/M3–M6 operate on spikes/weights exactly as now; u only
  changes WHEN neurons spike).
- **Existing experiments as baselines:** preserved by the zero-gate
  identity property at V2.1 and every subsequent step (each new
  mechanism defaults to off ⇒ bit-identical V2).
- **Energy/resources:** u adds no synapses; growth steps multiply
  synapse counts (bounded by existing budgets).
- **Expected failure modes:** V2.1 runaway bistability (contained
  by M6/E6/adaptation — testable); V2.2 noise-driven weight
  diffusion (M2 contains); V2.3 budget conflict with M2; V2.4
  oscillatory homeostat; V3.0 cancerous growth (budgets contain);
  V3-max semantic creep (charter vigilance).
- **Implementation complexity:** V2.1 ≈ tens of lines in
  network.rs + config gate; each further step roughly doubles.
- **Scientific risk:** V2.1 lowest (physics-flavored, reversible);
  rises monotonically; V3-max highest short of the boundary.

## 6. VERDICTS REQUESTED

**MINIMUM — "the smallest change I would permit":**
V2.1 (C1): one per-neuron slow depolarizing state u, gated by
β=0 ⇒ exact V2 identity, no task semantics, containment left to
the existing homeostatic laws. Anything smaller (tonic drive,
noise) supplies activity without state and does not address the
measured gap; anything larger is not minimal.

**MAXIMUM — "the largest change I would permit":**
V3-max: multi-timescale local dynamics (intrinsic + synaptic),
intrinsic plasticity, budgeted structural growth, and strictly
local, valence-free diffusive modulation — all under the
signal-semantics invariant (no task identity, no valence, no
credit, no supervision, no assigned roles). The first component
whose function references the experiment's categories (reward,
eligibility-for-actions, attention-as-control, memory-cell types)
crosses out of ANIMA.

**RECOMMENDED — "what I would actually investigate first":**
V2.1 alone, in three stages and nothing more:
1. identity gate + persistence probe (does ANY self-organized
   state survive 500 ms silence?);
2. E21-style capacity re-measurement under V2.1 (does the < 50 ms
   cliff move, by how much, and is it tunable by τ_s without
   touching anything else?);
3. only then, an E18/E19-class closed-loop re-run on the enlarged
   capacity — the same questions that failed for honest physical
   reasons, re-asked once the physics exists.
Why: it is the single change that converts every negative result
E18–E21 from "architecture cannot" into a real test of
"development under consequences"; it is cheap, reversible,
baseline-preserving, and it adds no semantics. If persistence
emerges but bridging still fails, THAT would be the first evidence
justifying V2.2/V2.3 — not before.

## 7. What this study deliberately does NOT do

No implementation, no parameter values (β, τ_s ranges are stated
as physics scales, not choices), no experiment design, no E-number,
no optimization against E18/E19/E20/E21/E23 outcomes, no repository
modification. The ladder is a boundary map, not a plan.

STOP after this study.