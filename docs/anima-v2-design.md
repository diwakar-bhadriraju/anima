# ANIMA v2 — Minimum Structural-Plasticity Architecture (design, not implementation)

**Status: DESIGN ONLY. No code changed, no parameters tuned, no runs
executed.** Decision framing: this design seeds from the E1–E4f
diagnosis; values for every named parameter are left for a future
pre-registration, not chosen here.

---

## 1. Capabilities ANIMA currently has

| Capability | Where | Verdict |
|---|---|---|
| Deterministic environment, seeded Poisson input, exact attribution | env.rs (D9) | solid |
| Weight-scale LTP (post-fires ⇒ strengthen all active afferents) | plasticity.rs `stdp_tick` | works, saturates |
| LTD (sparse; crippled for discrimination — coincident-pair skip + post fires in every context) | same | effectively inert |
| Passive weight decay + silence pruning (w < 0.02 for 60 s) | same + structural.rs | too slow to be structural |
| Adaptation current (rate stability) | network.rs (E3) | works — stability solved here |
| Neuron-birth machinery (trigger, wiring, dormancy lifecycle) | structural.rs (E4) | exists; every registered trigger/shape combination was inert or regressive |
| Post-hoc analysis instruments (cosines, selectivity, retention) | analyzer.rs / examples | external, no feedback to organism |
| Runaway detection (external guard) | resources.rs | unchanged, keeps working |

What the organism still **cannot** do: change its receptive structure,
compete at the synapse level, suppress shared drive, or learn from
anything except co-activation strength.

## 2. Capabilities missing (derived from the diagnosis)

1. **Synapse creation** — candidates that can become real afferents
   after experience, at all (only the fixed birth draw ever creates
   synapses, and E4's births were neuron-level and unstable).
2. **Synapse removal at learning timescale** — pruning at 60 s of
   silence is irrelevant to a 500 ms × 120-presentation learning
   epoch; nothing removes *defeated competitors*.
3. **Heterosynaptic competition** — no negative interaction between a
   neuron's afferents, so LTP amplifies *all* co-active inputs and
   saturates (E1's 73%-at-ceiling endpoint). This is the single most
   consequential absence.
4. **Input-specific receptive acquisition** — nothing lets a neuron
   develop an afferent set concentrated on one input group.
5. **Structural stability law** — E4's growth accumulated unbounded
   drive (sinks → runaway); no conserved quantity exists in the
   growth machinery (E4c: failure tracks birth count, not cadence).
6. **Differential suppression of shared/recurrent drive** — E3b's
   same-tick global inhibition compressed the differential instead;
   there is no *learned* negative sign.

## 3. Proposed new mechanisms (plasticity laws, not wiring plans)

All are local, curriculum-agnostic rules. None references pattern ids,
channel groups, stages, or rewards. They are the minimum set to make
the question "can an initially under-structured nervous system develop
distinct internal representations from experience?" answerable.

### M1 — Dense-weak initial sampling (substrate variance)
- Rule: each internal neuron starts with afferents drawn uniformly
  from a *large* random subset of input channels + recurrent neurons,
  at *weak* weights (well below saturation).
- Rationale: the diagnosis showed the binding bottleneck is the
  initial draw — at p=0.038, in-degree ≈ 2.85 and ~0.4 pattern-private
  neurons exist in the whole pool. Dense-weak sampling gives the
  population *variance to select from*; selection then does the work,
  not the draw. This is a change to the initial-condition law, not a
  hand-wired pathway (every neuron still samples randomly).

### M2 — Per-neuron synaptic normalization (heterosynaptic competition)
- Rule: periodically, each neuron's incoming weights are rescaled so
  total incoming weight (or total expected drive) is conserved at a
  per-neuron budget.
- Local information: the neuron's own sum of weights. Zero global
  state.
- Effect: LTP on one afferent forces LTD on every other afferent of
  that neuron — the "amplify everything" behavior becomes "amplify
  whoever wins". This is the missing counterpart to STDP that E1's
  saturation endpoint demanded.

### M3 — Candidate-synapse stabilization (synaptogenesis)
- Rule: each neuron maintains a small pool of *candidate afferents*
  — contact sites drawn at birth/list-time from unconnected eligible
  partners (input channels first, recurrent neurons second). A
  candidate has a tiny weight and obeys the same trace/timing rule
  as a real synapse; if it crosses a *permanence* threshold via
  repeated pre→post reinforcement it becomes a real synapse
  (consuming M5's budget); if it decays below a *die* threshold it
  is withdrawn and replaced by a fresh draw.
- Local information: pre/post traces (already exist) + candidate age.
- Effect: new input-specific afferents can be *tried* and *kept*
  based on experience — genuine structural search, the thing E4's
  neuron-birth machinery never delivered.

### M4 — Competitive pruning at learning timescale
- Rule: replace the 60 s silence rule with a structural-timescale
  rule: any live synapse whose weight (post-normalization) stays
  below a floor for a short sustained window is removed, freeing an
  M5 slot. Defeated competitors are *recycled* into M3 candidates.
- Effect: losers of the per-neuron competition exit the structure;
  their slots probe new partners. Iterating (M2/M3/M4) across S1
  turns the initial random bias into convergent receptive fields.

### M5 — Per-neuron synapse budget (structural conservation law)
- Rule: each neuron has a fixed number of live synapses
  (receptor-site model). A successful M3 candidate requires a free
  slot or evicts the lowest-weight live synapse.
- Effect: total network drive is bounded *by construction* — this is
  the structural stability law E4 lacked. With adaptation (existing)
  bounding rates and M2 bounding per-afferent drive, runaway of the
  E4/E4c class is structurally impossible rather than detector-guarded.

### M6 — Anti-Hebbian inhibitory synapses (learned differential
### suppression)
- Rule: a second synapse class with negative sign; inhibitory
  weights update anti-Hebbian (strengthen on co-activity of pre and
  post), bounded, and normalized like M2. Targets and sources are
  the same neurons — nothing hand-wired about which populations
  inhibit which.
- Effect: decorrelation of co-active populations (the covariance-
  whitening mechanism). E3b's global same-tick inhibition failed
  because it was structureless; M6 is *learned* structure.

### M7 — (Reserved, NOT in the minimum) — prediction-error-gated
### structural plasticity
- If PE earns a role, it is as a *rate modulator of structural
  learning* (learn faster when input is surprising), with the U3b
  lesson hard-coded: any adaptive bound must have horizon ≪
  sustain. It is excluded from v2 minimum to keep the organism's
  self-organization driven purely by its own activity, not by the
  external instrumentation the environment happens to expose.

## 4. Mechanisms that are conveniences for this A/B/C task (excluded)

- Topographic pre-wiring by channel group (A→left block, B→middle,
  C→right block).
- Fixed inhibitory web over the A/B/C populations (E3b-style, static).
- Reward/label signal telling the organism which stimulus it saw
  (that is E6 territory and changes the question).
- Curriculum manipulation to make separation easier (e.g., sparse
  markers, larger channel groups, presentation of A alone for long
  epochs).
- Neuron-level growth tuned to produce a convenient birth count.
- Any rule that reads `pattern_id`, channel group labels, or stage id.

The design must be validated on the *existing* curriculum; the only
curriculum role in v2 is as a probe of architecture, never as a
design input.

## 5. Level of growth: synapse, not neuron, not population

- **Neuron level**: excluded in the minimum — E4a–f showed neuron
  birth under every trigger/shape combination is either inert or
  destabilizing, and it injects large drive per event (need to bound
  structure before adding cells).
- **Population level**: no global scheduler; population effects
  emerge from local laws (this is the point of self-organization).
- **Synapse/receptor-sites**: yes — M3/M5 model a neuron as a set of
  receptor sites with candidates, which is the smallest unit that
  carries *input-specific* information.

## 6. Interaction with existing STDP

Two timescales, same primitives (traces, spikes, weights):

1. **Fast (per tick)**: existing trace-STDP runs unchanged (LTP/LTD
   on real synapses; M3 candidates use the identical traces for their
   own update).
2. **Slow (e.g., per 50–100-tick window)**: M2 normalization
   (rescale per-neuron totals), M3 candidate update/permanence/die,
   M4 pruning, M5 slot reallocation, M6 inhibitory update.

STDP's endpoint behavior changes meaning: hitting w_max now *squeezes*
competitors via M2 instead of freezing the system (E1's dead end) —
saturation becomes a win signal, and defeated afferents exit via M4.
Silence-prune (existing) can remain as a backstop but is superseded
for learning-timescale dynamics.

## 7. Is inhibition fundamentally required?

Two honest answers:

- **For stability: no.** E3 proved rate-bounding via adaptation; M2+M5
  bound drive structurally. The runaway detector stays as a guard but
  is not the design's load-bearing element.
- **For separation: probably yes.** Every separation-negative
  (E1/E3/E3b/E4-series) is consistent with one reading: in a purely
  excitatory, recurrent, overlapping-drive network, no local
  positive-sign rule can create a *preferred* input — it can only
  amplify shared drive. The only local mechanism that creates
  differential suppression from shared co-activation is a learned
  negative sign (M6). Biology agrees: cortex is E/I.

Therefore the minimum v2 includes M6, formulated as a plasticity law,
not a wiring plan. M6's *necessity* is itself a testable claim:
running v2-minimum without M6 is a legitimate control arm (prediction:
cross-cosine stays ≥ ~0.6).

## 8. Role of prediction error / novelty

**None in the minimum.** The U3b series showed PE-as-trigger is
fragile (timescale coupling) and load-coupled to the environment the
organism doesn't control. Novelty stays an *external* instrument.
This keeps v2's question pure: does structure self-organize from
activity alone? If a future arm wants neuromodulation, M7 (modulator,
not trigger; horizon ≪ sustain) is the designated shape.

## 9. How a neuron develops input-specific receptive structure
(narrative of the mechanism, end to end)

1. **Birth**: dense-weak random sampling (M1) gives every neuron a
   slightly different input mix — a lottery, not a design.
2. **Selection**: during S1, STDP amplifies afferents that predict
   the neuron's firing; M2 normalization forces every gain to be paid
   for by losses elsewhere. A neuron with a slight A-bias acquires
   strong A-afferents and its B/C-afferents fall (heterosynaptic
   depression).
3. **Replacement**: M4 recycles the fallen afferents; their slots
   become M3 candidates that try new partners and are kept only if
   reinforced. Over hundreds of presentations the receptive field
   converges toward the neuron's consistent drive source.
4. **Decorrelation**: M6 anti-Hebbian inhibition suppresses the
   co-active residual — populations that fire together learn to
   suppress each other, sharpening the A-response against the shared
   B/C component.
5. **Stability**: the per-neuron budget (M5) + normalization (M2) +
   adaptation cap both the wiring and the rates — the E4 runaway
   class is excluded by construction, not by detector.

End state (predicted): some neurons become quasi-private to A, some
to B, some to C, some remain mixed — the *distribution* over neurons
of pattern-preference is the self-organized representation, and it is
measured by receptive-field statistics, not by hand-assigned labels.

## 10. The smallest v2 architecture (consolidated)

- **I/O**: unchanged (24 Poisson channels, existing curriculum as
  probe only).
- **Initial law**: M1 (dense-weak uniform sampling).
- **Fast rule**: existing trace-STDP, unchanged.
- **Slow rules**: M2 normalization + M5 budget (conservation laws);
  M3 candidates + M4 pruning (structural search); M6 anti-Hebbian
  inhibition (learned negative sign).
- **Removed/disabled for v2-minimum**: neuron birth (E4 machinery
  parked); silence-prune as primary (kept as backstop); PE/novelty in
  any structural role.

Falsifiable predictions (to be pre-registered with values):
- P1: after S1, ≥ some neurons per pattern reach
  quasi-private receptive fields (pattern-preference entropy above a
  threshold) — if not, structural selection failed.
- P2: stability guard passes (no runaway — confirming M2/M5 as the
  stability law).
- P3: late-S1 cross-cosine < 0.60 with selectivity > 0.50 (the same
  endpoints as E3b, keeping comparability).
- P4: different seeds → different private sets (self-organization:
  the outcome is experience-selected variance, not a fixed design).
- Control arm (legitimate pre-registered question): v2-minus-M6 —
  prediction: separation endpoints fail ⇒ M6 necessity confirmed.

---

## Notes on honesty

This design does **not** guarantee separation. Its claim is narrower
and testable: it is the smallest set of local, curriculum-agnostic,
conservation-respecting plasticity laws that gives the organism the
*capability* the review identified as missing (receptive-structure
change with learned competition). Whether A/B/C separate is then an
empirical question about self-organization — which is exactly the
research question, rather than a wiring exercise.