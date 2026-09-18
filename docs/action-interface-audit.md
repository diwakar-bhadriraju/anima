# ANIMA — Action-interface audit (post-E19; design only)

Status: AUDIT (2026-09-19). No implementation, no protocol, no E20.
Question: what should count as a legitimate action interface for
ANIMA before we ask environmental pressure to shape behavior?
Authority: repository code/config/telemetry as cited.

## 1. Repository findings

### 1.1 How outputs 64-75 are generated and connected

- Output neurons are ordinary LIF internal-class cells created last
  in `Network::new` (network.rs:331-345); their ONLY distinguishing
  property is `class = Output`, which mirrors their spikes into
  `output_spikes`/`OutputActivity` (network.rs:48-49, 610-616). No
  distinct dynamics, no distinct plasticity, no distinct drive.
- Wiring (v2 M1, network.rs:362-427): every non-input neuron,
  INCLUDING outputs, receives (1) input afferents (p_in=0.5, w∈
  U(0.02,0.06)), (2) recurrent afferents from ALL non-input ids
  (p_rec=0.2, w∈U(0.005,0.02)), (3) M6 inhibitory afferents
  (p_inh=0.3, capped B_i=10). So outputs DO receive recurrent
  input from internals — they are not anatomically cut off.
- Drive equation (network.rs:555): dv = (−(v−v_rest) + i_syn +
  i_ext − i_adapt)/τ_m with v_rest = 0, i_ext = 0 everywhere
  (only a test injects i_ext, :657). With no input, v decays to 0
  and NOTHING fires. There is no noise term, no tonic drive, no
  self-sustaining oscillator in the committed architecture.

### 1.2 Stimulus-driven vs endogenous — measured, not assumed

E19 run `runs/e19-20260918T172211Z`, internal+output spikes by
trial-epoch (per-100 ms histogram, whole run):
- antecedent [0,500): ~100k spikes; first 100 ms after antecedent
  offset [500,600): 40,796 (32,377 internal + 8,419 output) — a
  network echo tail; by [600,700) it is 16 spikes: effectively
  zero. The gap [600,1300) is silent.
- probe [1300,1800): 10,169 internal + 3,250 output; first 100 ms
  after probe end [1800,1900): 478 (350 internal + 128 output);
  the remaining 400 ms of the action window [1900,2300): 0 spikes.
- Disruption drives ~280k spikes in the consequence epoch (the
  loop demonstrably reaches the organism).

Conclusion: NO neuron class in the committed organism produces
activity in sustained silence. All firing is stimulus-echo with a
decay time constant of ~100 ms (i_syn decay, τ_syn; adaptation
τ=200 ms only suppresses further). Endogenous persistent activity
DOES NOT EXIST in this architecture — not for outputs, not for
internals (E17 corroborates: with STDP+M3+M4 off, recurrence
collapses to 1 live synapse; the recurrent path is too weak to
sustain any state).

### 1.3 Candidate action-capable populations

- Outputs 64-75: connected as above; fire only under stimulus.
- Internals 24-63: same story (gap = 16 spikes / 100 s).
- No other population exists. There is no "motor" population, no
  oscillator, no intrinsic-bursting cell in the committed config.

### 1.4 Information available during the ambiguous probe

During the probe epoch the network state carries: the probe-driven
response PLUS the residue of the antecedent response (i_syn tail ≤
~100 ms, i_adapt ≤ 200 ms, E6 φ ~2.5 s plasticity-gain-only, M6
inhibitory weights ~1 s). At the 800 ms gap the residue is gone
(E18: internal divergence flat). If a vote window overlapped the
PROBE epoch, the readout would see the probe response itself —
which E18 proved is antecedent-INDEPENDENT (cos ≥ 0.96 after
settling). If it overlapped the first ~100 ms after the probe, it
would see the probe echo — same property.

## 2. Causal diagnosis of E19

E19's action interface demanded activity in a 500 ms SILENT window
800 ms after sensory offset. In this architecture that demand is
unsatisfiable a priori: (a) no endogenous activity exists
(§1.1-1.2); (b) at the vote time the network state is
antecedent-independent anyway (E18). The interface was therefore
not testing "pressure shapes behavior" — it was testing whether
silence can produce spikes, which the membrane equation answers
analytically (v→v_rest=0; no firing). Classification: **A — an
experimental interface defect** (a readout convention incompatible
with the organism's actual dynamics), not an organism property to
repair and not a missing mechanism that E19 was entitled to add.

## 3. Candidate interface designs + what each tests

| design | what it reads | what it scientifically tests | leakage/shortcut risk |
|---|---|---|---|
| D1 post-stimulus echo vote (window = first ~100-150 ms after probe offset; E19 template otherwise unchanged) | the network's response to the probe + its decay tail — with NO antecedent dependence (E18: flat divergence) | whether consequences can shape a STIMULUS-CONDITIONED readout — a genuine conditioning question, but NOT temporal-state: the vote is a reflex of the probe | none new; but it silently changes the question from E19's |
| D2 probe-overlapping vote (window inside probe epoch) | the probe response itself | same as D1, worse: the world reads a pure sensory echo; "action" = stimulus classification | the action is fully stimulus-driven — begs "is this an action at all?" (§4) |
| D3 continuous-effector interface (no window: the world reads output activity whenever it occurs; consequences gated by activity in registered intervals) | whatever activity exists, whenever it exists — including echo tails | whether consequences can reshape the organism's entire output disposition over development | weakest pre-registration (must define consequence law on continuous time); closest to "behavior = ongoing dynamics" |
| D4 endogenous-activity requirement (keep silent window) | nothing — impossible in committed architecture | impossible; would require tonic drive/noise/recurrent support = organism change (C) | n/a — not available as A |
| D5 antecedent-overlapping vote (window inside the ANTECEDENT epoch) | the antecedent response itself | trivial: reads the sensory stimulus directly; equivalent to a classifier on the antecedent | direct leak — fails the E19 question by construction |

## 4. Is "action" itself an artifact of the environment abstraction?

Partly. The current world imposes a discrete decision (binary vote
in a fixed window) — a psychological-task shape imported from RL.
ANIMA's philosophy ("behavior from its own dynamics") does not
require discrete votes; it requires that the organism's OWN
activity produce effects. In a nervous system with no endogenous
activity, the only self-generated events available are stimulus-
triggered response patterns (the echo). A legitimate interface must
read those — but must also be honest that what is being shaped is
the organism's RESPONSE DISPOSITION, not a deliberative act. D3 is
the least presumptuous form; D1 is the minimal registered variant.

## 5. Recommendation (minimal change, category A)

**D1** — keep everything from E19 (world, consequence law, balance,
leakage tests, E/M/L, criteria) EXACTLY as frozen, and move only
the readout convention: the vote window becomes the first 150 ms
after probe offset ([1800,1950) of the trial template), i.e., the
network's own echo to the probe — the only self-generated activity
that exists at the decision point. One template constant changes;
no organism contact; no new mechanism.

- Scientific question it tests: can closed-loop consequences shape
  the organism's probe-response disposition in an antecedent-
  dependent manner? NOTE: since E18 showed the internal probe
  response is antecedent-independent at this timescale, the honest
  prediction is failure unless consequences create the dependence
  that exposure did not — which is precisely the pressure question,
  now actually reachable.
- D3 (continuous effector) is the philosophically purer long-term
  interface but requires a new consequence law design (bigger
  change than A); flagged as the direction if D1-class experiments
  mature.

## 6. What is explicitly NOT being proposed

No organism change (C), no new mechanism/capability, no reward/RL,
no growth, no memory, no moving-the-window-to-succeed: D1 is
justified by §1.2's measurement (the echo is the only activity
available), registered BEFORE any E19-style rerun, with the
prediction on record (likely failure — informative either way).

## 7. Unresolved decisions requiring approval

1. D1 vs D3 as the next interface (recommend D1: minimal, category
   A, immediate; D3 as later redesign).
2. Vote window length for D1: 150 ms (covers the measured echo;
   100-200 ms range defensible) — approve or amend.
3. Whether a D1-based rerun is a NEW experiment (recommended: new
   id, new protocol; E19 stands as run) vs an E19 amendment
   (discouraged: E19's outcome is recorded and valid).
4. Accept the reframed scientific question (pressure on response
   disposition, not on silent-window deliberation) as the target.