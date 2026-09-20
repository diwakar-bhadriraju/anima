# Plasticity-as-memory audit — can the existing synaptic state serve as persistent, sensory-specific memory?

Status: READ-ONLY AUDIT, 2026-09-20. No runs, no organism changes,
no tuning, no E-number. New read-only instrument:
`crates/anima-exp/examples/synmem.rs` (snapshot-only analysis of
committed runs). Channel groups (committed configs): A = ch 0–7,
C = ch 8–15; B = ch 4–11 phase variants (never presented in any
analyzed run); ch 16–23 idle. Internal/output ids 24..75 (52, the
established instrument convention). Inhibitory = plastic==false
(M6 creation sets plastic=false, core network.rs:534).
Snapshots every 1000 ticks; u read directly from NeuronState.u_slow.

## 0. Runs analyzed (all committed; nothing rerun)

| run | drive | cell (β, τ) | seed |
|---|---|---|---|
| e24-s20260912-a | A-only ×40 | 0.0046875, 5000 | 20260912 |
| e24-s20260912-c | C-only ×40 | 0.0046875, 5000 | 20260912 |
| xp1-b0.0046875-t5000 | A-only ×40 | 0.0046875, 5000 | 20260912 |
| xp2c-b0.0046875-t5000 | C-only ×40 | 0.0046875, 5000 | 20260912 |
| xp2-b0.0046875-t5000 | A/C interleaved 40/40 | 0.0046875, 5000 | 20260912 |
| v21repac-b0.003125-t10000 | A/C interleaved 40/40 | 0.003125, 10000 | 20260912 |

e24-a ≡ xp1 and e24-c ≡ xp2c are the SAME config+seed (separately
committed runs): all derived numbers agree at print precision
(e.g. A/C masses 0.290/0.026; full-matrix cosine 0.1191 both
pairs) — an accidental determinism re-validation, noted.

## 1. G1 — cross-run persistence: A-trained vs C-trained states

Per-neuron 24-dim afferent weight vectors at drive end
(t = 84,000 ≈ 500 ms after 40th presentation):

| metric | value |
|---|---|
| A-trained: A-mass / C-mass / O-mass (mean/neuron) | 0.290 / 0.026 / 0.033 |
| C-trained: A-mass / C-mass / O-mass | 0.031 / 0.239 / 0.038 |
| full-matrix cosine A-trained vs C-trained (drive end) | **0.119** |
| full-matrix cosine (final, after 20 s silence) | 0.111 |
| per-neuron cosine: mean / % < 0.9 | 0.076 / 52/52 |
| selectivity (A−C)/(A+C): A-trained | **+0.906** |
| selectivity: C-trained | **−0.758** |
| neurons A-favoring: A-trained / C-trained | 49/52 vs 2/52 |
| top-5 channel overlap per neuron | 0.119 |
| LOPO cosine-centroid run-identity decode | **95/104 (91%)** |

VERDICT G1: **YES — training on A vs training on C leaves
distinct persistent synaptic configurations.** Same seed, same
initial weights; divergence is entirely training-driven. The
trained states occupy distinguishable regions (91% decode;
cosine 0.12; selectivity ±0.8–0.9), and the distinction survives
20 s of silence with no input (final cosine 0.111 — unchanged).

Raw MAT rows verified against the committed file (nid 24 drive
end: A-ch 0.3072, C-ch 0.0615); the earlier 0.887→0.052 "affw"
result is superseded in meaning by this direct cross-run measure.

## 2. G2 — internal representation: development and post-offset persistence

A-trained run, per-snapshot:

| tick | sel mean | aff cos vs t₀ | rec cos vs t₀ | aff cos consecutive | live aff |
|---|---|---|---|---|---|
| 1000 (S0) | +0.040 | 1.000 | 1.000 | 1.000 | 617 |
| 11000 (S1 early) | +0.544 | 0.676 | 0.872 | 0.9998 | 417 |
| 41000 (S1 mid) | +0.652 | 0.428 | 0.903 | 0.9998 | 336 |
| 81000 (S1 late) | +0.903 | 0.327 | 0.909 | 0.9998 | 309 |
| 101000 (S2, 16 s silence) | +0.914 | 0.287 | 0.913 | 0.9999 | 216 |

- Selectivity develops monotonically through S1 and KEEPS
  developing in silence (+0.903 → +0.914; driven by M4 pruning of
  remaining weak non-A mass: live aff 309 → 216).
- aff drive-end→final (20 s silence) cosine: A-run **0.986**,
  C-run **0.993** — structure persists through silent offset;
  per-tick structural change is tiny (consecutive 0.9998).
- Post-offset change: aff cos vs t₀ drifts 0.327 → 0.287 during
  silence (1.4% matrix change), mostly pruning — drift, not
  collapse.
- Selectivity gain is front-loaded: ~60% of the total gain in the
  first 5 presentations (+0.04 → +0.54), then slower.

VERDICT G2: **YES — organization develops during stimulation and
survives stimulus offset (structural persistence, distinct from
persistent firing; see G6).**

## 3. G3 — multiple-memory coexistence / interference under alternation

v21repac (A/C alternating, true order C,A,C,A… at 2 s cadence),
state after each presentation (mean over 52 neurons):

| k | after C_k: A-mass / C-mass | after A_k: A-mass / C-mass |
|---|---|---|
| 1 | 0.130 / 0.156 | 0.167 / 0.127 |
| 11 | 0.197 / 0.145 | 0.246 / 0.134 |
| 21 | 0.223 / 0.153 | 0.271 / 0.141 |
| 31 | 0.228 / 0.183 | 0.243 / 0.171 |
| 40 | 0.232 / 0.197 | 0.258 / 0.179 |

- **No overwrite:** both channel groups retain large mass under
  alternation (A ~0.23–0.26, C ~0.15–0.20 at k=40) — versus
  single-pattern runs where the un-driven group is pruned to
  ~0.026–0.031. M4 pruning cannot select while both groups are
  always driven.
- **But no recent-episode discrimination either:** cosine of the
  per-neuron A-mass vector after C_k vs after A_k rises
  0.972 → 0.990 (k=1 → 40); per-presentation state change
  converges (consecutive aff cosine late: 0.988–0.996). The
  state after an A presentation ≈ state after the adjacent C
  presentation.
- **The alternating state is NOT a mixture of the trained
  endpoints:** cos(alt, A-only) = 0.277, cos(alt, C-only) =
  0.235, cos(A-only, C-only) = 0.119 (xp cell). Best linear fit
  (α·A-only + (1−α)·C-only) leaves residual 96% of the alt norm.
  Under alternation the store goes to a THIRD configuration:
  both groups wired, selectivity each group 0.46 vs +0.91/−0.76
  singly (mass gap ±0.03 vs ±0.26 when isolated).

VERDICT G3: **traces COEXIST in the synaptic store (no
overwrite) but the resulting state is a shared, near-symmetric
superposition: neither a mixture of the isolated endpoints nor
a state that discriminates which episode occurred most recently.
This is the same class of failure previously measured in u
(alternating u-cos → 1.0), now demonstrated at the synaptic
level — the superposition problem is NOT specific to u.**

## 4. G4 — plasticity mechanism audit (from committed causal records)

Sources: E12–E14 (passive re-expression era), E15 audit, E16
protocol + results, E17 protocol + results + cross-seed
(unknowns-registry U9g), SDE-B/C2/D, V2.3.

CREATE (structure differentiation):
- M2 is REQUIRED for the separated regime: M2-off → T0 A-B
  ≈0.99, selectivity 0.476–0.51, permanence starved (E16 3×3).
- STDP is NOT individually necessary (STDP-off reproduces the
  canonical trajectory near-verbatim, E16).
- M3/M4 topology dynamics NOT necessary (STDP-off+M3M4-off still
  flips; k*=1 all 3 seeds, E17 + U9g).
- Recurrent structure collapses under STDP-off (~1 live rec
  synapse vs 45/54/35 canonical) — STDP is the recurrent
  afferent maintainer specifically (E17).

MAINTAIN:
- M2 normalization (bidirectional, pins exc sum per neuron to
  t_e=0.8, structural_v2.rs:300-315) is active at every instant;
  combined with M4 pruning (theta 0.005, 10 windows) it is what
  SELECTS the driven channel group in single-pattern runs (G2
  live-aff 617→309; selectivity rise).
- Passive decay 1e-6/tick is the dominant eraser (SDE-C2: ~70–80%
  of block-1 cohort removed by decay alone).
- M6 inhibitory drift observed active (endpoint Δ −0.98..−1.37,
  E16/E17); M5 budget eviction inactive in the probed regime
  (occupancy 9–12/40, 0 evictions, E16).

RETRIEVE/use:
- The REV1 flip uses PRE-EXISTING weights with zero required
  plasticity events (E17: 0 STDP, 0 M3, 0 M4, 0 M5; flip
  completes with input-statistics reversal as the only event-
  scale change) — retrieval is a DYNAMICS function (gain routing
  by input phase order), not a plasticity function.

VERDICT G4: creation = M2(+E6/STDP-modulated) selection;
maintenance = M2 cadence + M4 pruning; retrieval = forward
dynamics on stored weights. M2 is the single mechanism appearing
in BOTH create and maintain; deleting it breaks the separation
regime (E16 3×3), which is the closest the record comes to
"necessary for persistent structure."

## 5. G5 — STDP-off REV1 re-expression reconciliation

The record resolves this precisely (E17 audit §3–§5):
- The information WAS ALREADY PRESENT in synaptic state before
  REV1: T0 A-side bucket totals 16.46/11.05/19.59 with 194/139/
  203 live A-side synapses (E15/E17); per-bucket net weight
  change ≤ 0.16 while readout cosine moved 0.47–0.62 (20–50×).
- Re-expression was driven by the REVERSED INPUT STATISTICS
  acting on those weights (phase-order reversal 8-11↔4-7), not by
  M3/M4/M6: maturations occur 200–300 ms INTO the window (after
  the flip's onset); M6 drift is small; E17 zeroes all three and
  the flip survives (k*=1, 3/3 seeds).
- Boundary of permitted reading (per E17 approval): NOT
  "passive re-expression alone is sufficient" — M2, M6, E6,
  decay, adaptation remain active in the STDP-free arms.

Meanings: (a) synaptic state is the storage medium; (b) the
flip is retrieval, not re-learning; (c) creation and retrieval
use different mechanisms (plasticity vs dynamics) — consistent
with G4's create/maintain/retrieve split.

## 6. G6 — information vs activity persistence

A-trained run, drive end → after 20 s silence:

| quantity | drive end (84k) | silence end (105k) | persistence |
|---|---|---|---|
| aff structure cos (drive end → final) | — | 0.986 | structural |
| rec structure cos (drive end → final) | — | 0.993 | structural |
| selectivity | +0.903 | +0.914 | structural (rises) |
| u norm | 7.999 | 2.282 | decays 3.5× |
| internal spikes/s | 479 | 71 | decays 6.7× |

Structural quantities change ~1%; activity quantities decay
3.5–7×. Persistent structural information and persistent
neural activity are cleanly separable: structure persists,
activity decays. (u cos vs t₀ is unavailable — u begins at 0 —
so the audit used norm/spike decay together with the committed
O2.6 top-u→spikes monotonicity.)

VERDICT G6: **weights = persistent structural information;
u/spikes = decaying activity. The substrate stores
information in weights, not in activity.**

## 7. G7 — memory capacity

- Raw DOF: 52 × 24 afferent sites + recurrent, continuous
  weights (0..1) — large in principle. Real constraints:
  - M2 pins total exc weight per neuron to t_e=0.8 (measured:
    sums 0.79999 in G1 runs) — a hard per-neuron budget shared
    by all groups.
  - Single-pattern training concentrates the budget on one
    group (11:1 mass ratio), pruning the other — the budget +
    pruning = winner-selection.
  - Under alternation both groups stay driven, so neither is
    pruned; the store holds both at ~2:1.6 — but undiscriminated
    (G3).
- Overwrite: YES at block level — second blocked block erases
  first via M2 reallocation (SDE-B: balance fraction 0.079→0.054
  at t_e 1.6; SDE-C2: passive decay removes 70–80%), the
  documented basis for V2.3's partition rescue (bca 6/6).
- Capacity limit: the M2 budget per neuron IS the limit — V2.3
  partition (t_e split into n_pop buckets, total never > t_e)
  increased usable coexistence at measured cost of per-bucket
  ceiling (sum ≤ t_e preserved; F-P-series passed).
- Interference: yes — competition for the same synaptic budget
  (same-neuron sums), demonstrated by SDE-B (capacity-
  independent: rule-level competition) and SDE-D (skew-invariant
  residual ≈0.07, M2 reallocation inferred).
- BUT: the partition was only ever tested BLOCKED (V2.3 bca).
  Alternating + partition is untested.

VERDICT G7: two memories can coexist (alternation holds both;
partition holds blocked cohorts); a new experience overwrites an
old one only when the old group goes quiet (pruning window) —
otherwise superposition; interference is budget competition;
capacity scales with M2's per-neuron budget structure, and the
budget's partition is the demonstrated (blocked-order) capacity
lever.

## 8. G8 — retrieval

- ESTABLISHED (E12–E17): a later presentation (REV1) with
  reversed phase statistics re-expresses an EARLIER-established
  A-side configuration, with zero plasticity events; the
  endpoint weight change is ≤3% while readout flips 0.47–0.62.
  This is reinstatement of a stored configuration by a sensory
  input — retrieval as forward dynamics through stored weights.
- NOT established: per-episode retrieval under ALTERNATION. G3:
  state after A_k ≈ state after C_k (cos 0.99) — a later
  A-presentation does not reinstate an A-specific configuration
  distinguishable from the adjacent C-configuration. E18's
  design note applies: at the tested scale, "present from trial
  1 / fixed trace" is the standing trivial alternative for the
  association-level flip; nothing in the record shows episodic
  (episode-specific) reinstatement.

VERDICT G8: retrieval exists at the association/trace level
(re-expression), NOT at the episodic level under alternation.

## 9. Contradictions / limitations

- v21repac is at the β=0.003125 cell; its single-pattern twins
  do not exist (xp1/xp2c are 0.0046875) — mixture-vs-endpoint
  tests done only at the 0.0046875 cell (xp triple).
- G3's after-A_k/after-C_k cosine compares states 2 s apart with
  exactly one intervening presentation — a conservative,
  presentation-local difference measure.
- u-cos-vs-t₀ undefined (u starts at 0); G6 uses norms/spikes +
  committed O2.6.
- All analysis is seed 20260912, one α (and one second cell for
  alternation). No cross-seed claim beyond the committed
  records.
- LOPO 91% (not 100%): 9/104 neuron vectors are nearer the
  wrong centroid — separation is strong, not total.

## 10. FINAL ARCHITECTURAL DIAGNOSIS

Classification: **B — synapses persist and are sensory-
specific, but coexistence/retrieval (at the required level) is
unestablished.** Specifically:
- Persistence: established (G1, G2, G6 — cross-run distinct
  states stable through silence).
- Sensory-specificity: established (G1 selectivity ±0.8–0.9,
  LOPO 91%).
- Coexistence: PARTIAL — blocked-order coexistence requires the
  V2.3 partition (established, bca rescue); alternating
  coexistence exists in mass (no overwrite) but the store
  superposes the two traces into a third, undiscriminating
  configuration (G3).
- Retrieval: established only as trace/association re-expression
  (E12–E17), NOT episode-specific under alternation (G3).

The most important question — need for separate persistent u?

**No, based on the evidence.** (1) The information lives in the
weights; u/spikes are decaying activity (G6) with u's documented
role being the interplay that sustains endogenous regime
activity (X1/O2.6) — a rate/energy role, not a store. (2) The
alternation failure that killed u (tsegsim cos→1.0) recurs at
the SYNAPTIC level with the same signature (G3 cos 0.97→0.99,
superposition state) — the bottleneck is superposition in a
shared additive store, not the substrate identity. Adding a
parallel u-like store would reproduce the same mixing with new
machinery. (3) The only intervention ever measured to improve
coexistence in the weight store is the M2 budget partition
(V2.3) — the evidence points at making plasticity more
memory-capable, not at a second scalar substrate.

## 11. Single smallest capability a future mechanism experiment must add

**Budget partition along the locally available channel-group
axis, tested under ALTERNATION** (per-pre-channel-group M2
buckets; the partition is locally determinable at each synapse —
its pre-synaptic partner's channel — and the mechanism is
V2.3's proven partition machinery, re-keyed from write-epoch to
pre-channel identity; total ≤ t_e unchanged, zero new
timescales/parameters beyond the bucket key).

Why this is the minimal increment: G3 isolated the failure —
under alternation the shared M2 budget cannot select, so the
store superposes; the only measured countermeasure (partition)
was tested blocked-only. Deciding whether partitioned budgets
make the alternating store discriminate recent A vs C (G3's
after-A_k/after-C_k cosine < mixed baseline) would directly
distinguish "budget superposition is the memory limit" (partition
helps) from "superposition is intrinsic to Hebbian storage at
this overlap" (partition doesn't) — and it reuses only
committed, locally-specified machinery.

NOT frozen, NOT designed in detail, NOT implemented. The
distinguishing experiment, if pursued, must be pre-registered
per standing discipline, gated by identity (flag off =
byte-identical V2.3 path), and evaluated only on the G3 metric
family plus the V2.3 F-checks.

STOP. Audit complete; nothing modified beyond the read-only
instrument and this record.