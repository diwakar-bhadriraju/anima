# CLLA F4 causal stability audit — what exactly causes the instability

Status: READ-ONLY, 2026-09-20. Uses only committed CLLA telemetry/
snapshots/event streams (runs/clla-*). No new runs, no parameters,
no amendments. Instrument: examples/clla_traj.rs (per-snapshot
state reconstruction + first-event markers), examples/evictscan.rs.

CORRECTION first: the earlier record prose said "6/12 aborted";
the RunEnded telemetry shows **7/12** (il 2/3, bac 2/3, bca 0/3,
d 3/3). Verdict (NOT SUPPORTED) is unaffected; the number is fixed
here.

## 1. Run-by-run causal timeline (F4 runs)

First-event ordering per run (ms):

| run | arm | abort | 1stPerm | 1stP>0 | 1st rate>50 | gap perm→>50 | nPerm | fin nC |
|---|---|---|---|---|---|---|---|---|
| s20260912-bac | bac | 34015 | 7100 | 8000 | 26000 | +18.9 s | 266 | 216 |
| s20260912-d | d | 22092 | 5400 | 6000 | 6000 | +0.6 s | 498 | 335 |
| s424242-il | il | 70011 | 7400 | 8000 | 6000 | **−1.4 s** | 520 | 322 |
| s424242-d | d | 22093 | 5400 | 6000 | 6000 | +0.6 s | 661 | 491 |
| s9001-il | il | 48083 | 5500 | 6000 | 42000 | +36.5 s | 550 | 398 |
| s9001-bac | bac | 44036 | 7100 | 8000 | 30000 | +22.9 s | 337 | 266 |
| s9001-d | d | 26017 | 5500 | 6000 | 6000 | +0.5 s | 674 | 465 |

KEY: in the d arm the population rate crosses 50 Hz within the
FIRST presentation window (t=6000), with protected mass P ≈ 0.3
and nC ≈ 13–27 — i.e., at the moment of crossing, consolidation
has contributed essentially NOTHING (P ≈ w_c_permanent·13 ≈ 0.26).
In s424242-il the rate crossed 50 Hz at t=6000 BEFORE the first
P>0 snapshot — rate escalation precedes consolidation outright in
3 of 7 F4 runs.

First-presentation comparison (t=6000, same seed/arm-d vs arm-il):

| run | t=6000 rate | spk1k | P | nC | t=10000 rate | P | nC |
|---|---|---|---|---|---|---|---|
| s20260912-d | 62.4 | 7176 | 0.64 | 27 | 22.2 | 24.1 | 280 |
| s20260912-il | 47.6 | 5097 | 0.00 | 0 | 14.7 | 3.4 | 82 |
| s424242-d | 67.8 | 7397 | 0.58 | 24 | 15.1 | 14.0 | 417 |
| s424242-il | 56.7 | 5634 | 0.00 | 0 | 16.5 | 1.4 | 49 |
| s9001-d | 138.8 | 10885 | 0.26 | 13 | 32.1 | 18.6 | 387 |
| s9001-il | 41.1 | 4682 | 0.04 | 2 | 10.0 | 5.0 | 133 |

The d stimulus = channels 0–15 at 20 Hz (A∪C) = exactly 2× the
320 events/s... (D drives 16 channels = 2× A/C's 8 channels at the
same per-channel rate ⇒ 2× input event rate). The rate excess at
t=6000 is present at P≈0.3 — INPUT-LOAD driven, not
consolidation-driven.

## 2. Stable vs F4 at matched times

| run | arm | t15 r/P/nC | t20 r/P/nC | t30 r/P/nC | t40 r/P/nC |
|---|---|---|---|---|---|
| s20260912-bac **F4** | bac | 15/18/134 | 37/38/163 | 58/59/198 | aborted (34 s) |
| s424242-il **F4** | il | 5/7/181 | 13/17/252 | 19/46/288 | 32/92/308 |
| s9001-il **F4** | il | 3/7/224 | 9/17/308 | 27/48/367 | 46/99/389 |
| s9001-bac **F4** | bac | 9/23/191 | 36/46/217 | 51/78/256 | 66/108/261 |
| s20260912-il ok | il | 4/10/170 | 15/21/236 | 20/41/330 | 32/77/360 |
| s424242-bac ok | bac | 6/13/156 | 20/35/225 | 27/60/254 | 38/108/279 |
| s9001-bca ok | bca | 3/4/82 | 12/12/118 | 24/38/164 | 32/66/182 |
| s20260912-bca ok | bca | 8/8/82 | 16/18/103 | 26/35/132 | 38/49/146 |
| s424242-bca ok | bca | 4/7/69 | 14/14/103 | 24/29/143 | 43/41/165 |

DECISIVE: s424242-il (F4) and s20260912-il (ok) — same arm, same
curriculum — track at t=20 (13/17 vs 15/21), t=30 (19/46 vs
20/41), t=40 (32/92 vs 32/77) almost identically in rate AND
protected mass, yet one trips at 70 s and the other completes
105 s. s424242-bac (ok) and s9001-bac (F4): same P at t=40
(108/279 vs 108/261), tripped vs not. CONCLUSION: protected-mass
magnitude does NOT separate F4 from ok runs. Every surviving run
accumulates the same unbounded growth (fin P 57–225, nC 217–407)
without tripping.

## 3. First-event ordering (frozen question 2)

d arm (3/3): excitation escalation precedes/appears simultaneously
with consolidation (rate >50 at P≈0.3). Consolidation does NOT
precede excitation — the input load is the driver.
il-bac F4 (4): consolidation precedes sustained escalation by
19–37 s — but the SAME sequence, with the SAME P trajectories,
occurs in ok runs. Temporal precedence exists but is not causal:
matched-P ok runs prove consolidation + unbounded P are
compatible with stability.
s424242-il even crosses 50 Hz at t=6000 (before first P>0) and
completes until 70 s — momentary crossings are benign; the P2
detector requires 5 s sustained.

## 4. Correlates (frozen question 4) — measured, not asserted

- nC, P, Pmax (per-neuron protected mass): NOT discriminative
  (overlap fully between F4 and ok; §2).
- M2 working-mass reduction (dist metric, W-target distortion):
  ≈ 0.00 in ALL runs at all times measured — because once
  per-neuron P ≥ t_e (0.8), the CLLA M2 branch computes
  target = t_e − P ≤ 0 and SKIPS THE NEURON ENTIRELY (normalize
  "continue"). The working pool is then NEVER normalized — W
  grows alongside P (F4 W reaches 76–195 vs t_e·52/4 ≈ 10.4
  mean). The "M2 redistribution" is not a redistribution: it is
  an M2 ABANDONMENT of over-protected neurons.
- Neuron concentration: Pmax 7.9–14.8 at abort vs 3.9–7.9 in ok
  fin — overlap; not discriminative.
- M6 inhibition: inh mass 7.6–16.0 at abort-2k vs 6.6–17.5
  earlier in the same runs — M6 does NOT collapse; it is
  OVERWHELMED (exc mass grows 5–10× while inh stays flat). M6
  is overcome, not failing first.
- Recurrent excitation: not directly separable in snapshots;
  the rec-matrix behavior was stable in the unmodified audit
  (x-synmem G6); the runaway here is driven by the afferent
  mass growth.
- M5: EXCLUDED from the immediate pathway by events: zero
  budget-evictions in all surviving runs (evictscan), and the
  F4 runs' failure events show no eviction precursor.

## 5. Event-local analysis around first consolidation (question 5)

State at t(first_P>0)+{0,1k,5k,10k} and abort−2k (all F4):

| run | +0 rate/P | +1k rate/P | +5k rate/P | +10k rate/P | abort−2k rate/P |
|---|---|---|---|---|---|
| s20260912-bac | 25/2.4 | 10/2.4 | 9/10 | 31/34 | 61/65 |
| s20260912-d | 62/0.6 | 23/0.6 | 13/24 | 50/80 | 62/109 |
| s424242-il | 22/0.3 | 8/0.3 | 5/4 | 11/10 | 69/167 |
| s424242-d | 68/0.6 | 25/0.6 | 6/14 | 44/73 | 86/142 |
| s9001-il | 41/0.0 | 15/0.0 | 4/5 | 8/10 | 74/143 |
| s9001-bac | 23/2.5 | 9/2.5 | 7/15 | 27/37 | 69/111 |
| s9001-d | 139/0.3 | 69/0.3 | 20/19 | 55/98 | 89/218 |

Temporal evidence: (a) the rate right after first consolidation is
LOW (5–25 Hz in 5/7 runs) and drops FURTHER at +1k–+5k; (b)
escalation begins between +5k and +10k as P passes roughly the
10–50 range; (c) at abort−2k every run is >60 Hz. So the local
sequence is: first consolidation → modest P growth → passive
period → exponential drive growth → trip. This is consistent with
the M2-abandonment mechanism: once per-neuron P crosses t_e=0.8,
normalization of that neuron ceases and its afferent mass grows
combatively (LTP + no brake). P crosses 0.8/neuron around t≈10k
for most runs (P total 34–98 by +10k / 52 neurons ≈ 0.7–1.9).

## 6. Does any stable run reach comparable protected mass? (question 6)

YES — s20260912-il (ok) reaches fin P=224.72, nC=407; s424242-bac
(ok) fin P=129.72, nC=312; s424242-bca (ok) fin P=56.9. Protected
mass alone is NOT the instability; every ok run carries the same
unbounded growth. The distinguishing factor at matched times is
the RATE trajectory (driven by seed/arm-specific recurrent
dynamics + input load), not the protected mass.

## 7. Hypothesis test (question 7)

H1 "consolidation → permanent excitation locking → working-pool
distortion → instability":
- Working-pool distortion: FALSIFIED as the mechanism in its
  literal form — the dist metric stays 0.00; the working pool is
  not "distorted," it is ABANDONED (normalize skips neurons with
  P ≥ t_e). The unprotected working pool then grows, not
  shrinks (W 39→130–195).
H2 "instability begins independently; consolidation merely
correlates": SUPPORTED for the d arm (rate >50 at P≈0.3) and for
s424242-il (rate >50 before first P>0); NOT supported as the full
story for the il/bac F4 runs, where consolidation precedes
sustained escalation by 19–37 s.
Strongest evidence-supported mechanism (both hypotheses partially
true):
    consolidation removes M2 + decay from the protected synapse
    (by design) → LTP on protected synapses unopposed → per-neuron
    P crosses t_e=0.8 → CLLA M2 branch computes target = t_e − P
    ≤ 0 → the neuron is SKIPPED by normalize entirely → working
    synapses ALSO lose the M2 brake → total afferent mass grows
    without bounds (P+W 5–10× t_e budget) → population drive
    escalates → P2 trip when the mean stays >50 Hz for 5 s.
Whether the trip actually fires is a threshold lottery set by
seed/arm-specific recurrent dynamics: matched-P ok runs don't
trip, F4 runs do. The mechanism is a PROTECTION × M2-INTERACTION
failure: the protection is total (no brake at all), and the M2
interaction degenerates to abandonment instead of normalization.

## 8. M5 exclusion (question 8)

EXCLUDED: zero budget-evictions in every F4 run's event stream
and in all surviving runs (evictscan). M5 cannot be in the
immediate F4 pathway. (Frozen protocol recorded this too.)

## 9. Memory concept vs stability failure (question 9)

SEPARATED. Nothing in this audit contradicts the memory
hypothesis. Measured: afferent channel structure developed under
CLLA (survivor F1 passes), consolidation occurred (hundreds of
synapses per run), P grew. What failed is the DYNAMICAL BRAKE
contract: protection-without-replacement removed the substrate's
two stabilizing forces (M2 normalization, passive decay) from a
growing set of synapses in an all-excitatory organism — the same
class of failure E3/E4 growth arms showed. A memory capability
failure (e.g., F1 failing with stable dynamics) was NOT observed
in any surviving cell; F2/F6 remain unmeasured due to F4, not due
to memory evidence.

## 10. Unresolved because F2/F6 unmeasurable

- Whether coexistence (F1 continuing) and response separation
  (F2) actually hold under stability — 5/9 F1-relevant arms
  aborted; d composition (F6) never measurable in any seed.
- Whether the d arm is intrinsically over-driven: D = 2× input
  load; rate >50 at P≈0.3 suggests the protocol's D stimulus
  may be outside the stable drive range of the organism EVEN
  FLAG-OFF. No flag-off D run exists (none was in the frozen
  arm set); cannot be resolved from committed artifacts.

## 11. Likely failure location

- Consolidation (the act): not the failure — ok runs consolidate
  equally.
- Protection (total exemption): NECESSARY contributor — removes
  the only two brakes measured to hold the all-excitatory
  organism down (E3/E4 precedent).
- M2 interaction: NECESSARY contributor and the amplifier — the
  target ≤ 0 → skip behavior abandons normalization of the whole
  neuron instead of bounding anything.
- M6 interaction: secondary — M6 is overcome (flat inh vs 5–10×
  exc growth), it does not initiate.
- Combination: protection × M2-interaction is the failure; the
  concrete defect is the "skip on target ≤ 0" degenerate branch
  plus absence of any upper bound on protected growth (the
  entry-only cap documented as mode-3 "bounded by w_max" was
  falsified: many protected synapses each near w_max ⇒ P is not
  bounded).

## 12. The single smallest architectural question to resolve before any amendment

    Is an UPPER BOUND on protected mass/normalization — enforced
    DURING growth (e.g., normalize working mass to t_e − P with
    P capped, and either cap consolidated weight or restore a
    braked growth rule for protected synapses) — sufficient to
    restore stability while preserving coexistence, or does any
    protected-class growth in the all-excitatory organism
    necessarily outrun the P2 detector under the current
    drive regime?

Equivalently: "protection without a growth bound" was falsified
(F3 13×, F4 7/12). The question is whether "protection WITH a
growth bound" (bound form TBD — P ceiling at LTP, or normalized
class budget) is stable AND still F1/F2-capable. That is the
smallest falsifiable fork: until it is answered, no amendment
(cedure/ceiling/M2-branch fix) is justified — any of them might
merely move the failure.

Nothing proposed beyond this question. No parameters, no
mechanism changes, no runs. STOP.