# ANIMA E23 causal audit (read-only; committed artifacts only)

Status: AUDIT (2026-09-19). Sole evidence: the two committed E23
runs (e23-closed/e23-open-20260918T1817{38,40}Z) + repository
mechanism code. Instrument `e23audit` (telemetry+snapshots only;
byte-deterministic). No implementation of organism/world; no
reruns; no new experiments.

## 1. What changed during E23 (closed vs open, snapshots at trial
## 40 vs 160, plastic synapses alive in both)

Output-neuron afferent weight sums (the decision substrate):

| quantity | CLOSED E→L | OPEN E→L |
|---|---|---|
| outputs' A-side (0-7) afferents | 1.788→1.024 (**−0.763**) | 3.407→5.042 (**+1.636**) |
| outputs' C-side (8-15) afferents | 1.783→3.744 (**+1.960**) | 0.703→1.398 (+0.696) |
| g1 A-side | 1.095→0.151 (destroyed) | 0.997→2.042 |
| g1 C-side | 0.857→1.677 | 0.237→1.083 |
| g2 A-side | 0.693→0.873 | 2.410→3.000 |
| g2 C-side | 0.926→2.067 | 0.466→0.315 |

The arms moved in OPPOSITE directions: closed lost A-afferents and
gained C-afferents on output neurons; open gained A-afferents (and
P(g1|A)→1.0 — the open arm's late mapping is pro-A, roughly
correct by drift). Pathway dW (alive-both) is positive everywhere
except no negative totals — the redistribution is competitive
(via M2's fixed t_e=0.8 budget per neuron: gains on C-afferents
displace A-afferents).

Behavioral readout consequence (measured, late window):
closed output spikes/trial: g1 fires on C (264.1) not A (18.8);
g2 also fires on C (179.6) not A (35.5). So on A-trials BOTH
groups are quiet (near-tie; g2 wins margins); on C-trials BOTH
fire (g1 wins margins) — producing the observed inverted votes
A→g2, C→g1 as a pure C-dominance readout of both groups.

## 2. Temporal causality (finest committed resolution: per-trial
## votes, 10-trial blocks, and ms-timestamped events)

The inversion is GRADUAL and trial-locked to consequence exposure:
blocks 1-80 wander around chance (benign 3-8/10); blocks 81-100
collapse (benign 0-2/10); blocks 101-120 PARTIALLY RECOVER
(P(g1|A)=0.8, benign 4/10 — the mapping loosens); blocks 131-200
lock into the inverted mapping (benign 0/10, P(g1|A)=0, P(g1|C)=1).
The world's mapping never changes; the organism's routing passes
through a near-correct phase (trials 41-70: P(g1|A)=1.0 with
benign 7-8/10) before inverting. Weight changes follow the trial
cadence: STDP event timing relative to epochs shows the
consequence epoch [500,1000) carries only ~7% of total LTP
(+201.6 vs +2694.0 stimulus epoch) — the bulk of plasticity is
stimulus-driven; the consequence-epoch share is where the arms
differ qualitatively (below).

## 3. STDP audit (sign/timing convention vs the inversion)

Plasticity is pairwise STDP: pre-before-post within τ=20 ms ⇒ LTP;
post-before-pre ⇒ LTD (a_plus=0.005/a_minus=0.0053 — near-
symmetric, slightly LTD-dominant). Consequence-epoch LTP by
pathway (the discriminating measurement):

| pathway | CLOSED | OPEN |
|---|---|---|
| OUT→INT | **50.00** | 9.90 |
| IN→INT (16-23 drive) | **5.43** | 0.18 |
| INT→OUT | 2.00 | 34.17 |
| INT→INT | 141.76 | 83.95 |

In the closed arm, disruption (channels 16-23 → internals) makes
INTERNALS spike ~immediately after the vote-window OUTPUT spikes.
STDP's pre-before-post window (20 ms) mostly CANNOT bridge
vote-offset (t=500) to consequence onset (t=500) — they are
contiguous: the last vote-window output spikes and the first
disruption-driven internal spikes can fall within 20 ms of each
other at the boundary, potentiating OUT→INT and IN(16-23)→INT
paths. Measured: OUT→INT LTP is 5.05x the open arm's; 16-23→INT
LTP is 30x. This is ordinary Hebbian correlation at the action/
consequence boundary — NOT "reinforcement" (no eligibility, no
credit assignment, no valence exists in the rule; the rule cannot
represent "consequence" at all — it sees only spike timing).
Crucially the sign is not action-selective: the disruption follows
BOTH actions (match→silence, mismatch→disruption), so any boundary
potentiation hits whichever output pathway was active, and
disruption is 10x more frequent as the mapping worsens — a
positive feedback onto the punished disposition.

## 4. Pathway localization of the inverted mapping

The inversion is expressed in SENSORY→OUTPUT AFFERENT COMPETITION,
not in a rerouted motor map: both output groups came to be driven
by C-side afferents (g1 C 1.677 / g2 C 2.067 vs A 0.151 / 0.873).
The vote inversion is a side effect of C-dominance + asymmetric
group thresholds, not a learned A→g2 association per se. Recurrent
(INT→INT) and inhibitory pathways show large activity but no
arm-specific sign pattern beyond the consequence-epoch OUT→INT
asymmetry. INT→OUT LTP favors the OPEN arm (34.2 vs 2.0) — the
closed arm's outputs fire less overall late (both groups
C-dominated, A-trials quiet).

## 5. M2 audit

M2 pins each neuron's total excitatory afferent weight to t_e=0.8
(E15/E16-era invariant, verified in-run during earlier audits).
Therefore the C-gain/A-loss on output neurons is a ZERO-SUM
redistribution: whatever drives C-afferent potentiation AUTOMATIC-
ALLY erodes A-afferents. M2 is the amplifier/expressor of the
inversion, not an independent cause: it converts any slight
activity-statistics asymmetry into a categorical routing flip.
Contribution magnitude not decomposable from committed data
(no M2-off arm exists in E23).

## 6. M6 audit

Inhibitory weights moved in both arms (E-series-typical net-
negative drift); no arm-specific inhibitory signature was
measured by the committed instruments beyond endpoint drift.
M6's role cannot be excluded or established — UNRESOLVED from
committed data.

## 7. Structural audit (M3/M4/M5)

Maturations: closed 7,644 (6,014 stimulus-epoch / 1,630
consequence-epoch / 0 ITI) vs open 6,270 — a 22% excess in the
closed arm, concentrated in the stimulus epoch. Prunes: closed
6,855 vs open 5,450 (ITI-heavy in both). The consequence epoch
carries 21% of closed-arm maturations vs 26% open (proportionally
similar); the EXCESS is activity-driven, consistent with higher
overall firing (disruption response). No evidence of
consequence-timed structural specificity. M5: 0 evictions (both
arms, verified earlier pattern holds). Candidate-pool internals:
not reconstructable (established limitation).

## 8. Temporal credit window

Weight-event timing: the emitted STDP events cluster inside
epochs; the discriminating OUT→INT/IN(16-23)→INT LTP sits at the
vote/consequence boundary (contiguous at t=500; τ=20 ms reach).
The plasticity rule has NO representation of "consequence" — it is
spike-timing only. Any credit-like effect is an emergent property
of the boundary contiguity, not a mechanism.

## 9. Seed robustness

LIMITATION: only seed 20260912 exists (registered; cross-seed not
approved). All findings are single-seed. No generalization claim
is made.

## 10. Alternative explanations vs evidence

| hypothesis | verdict from committed data |
|---|---|
| (H1) consequence potentiates the preceding action pathway | PARTIALLY SUPPORTED at the boundary: OUT→INT and IN(16-23)→INT LTP excess is real, contiguous, and closed-specific — ordinary STDP correlation. But the inverted ROUTING is carried by C-vs-A afferent competition on outputs, which H1 alone does not explain. |
| (H2) output routing changed indirectly via sensory correlations | STRONGLY SUPPORTED as the proximate cause: the disruption drive (16-23) co-activates and potentiates internal→output paths favoring C-side readout; outputs' C-afferents +1.96/A −0.76 exactly anti-track the open arm. The inversion is a sensory-redistribution effect of the consequence CHANNEL, not of the action. |
| (H3) M2 redistribution | SUPPORTED as amplifier (zero-sum displacement; §5). |
| (H4) M6 redistribution | UNRESOLVED (no discriminating measurement). |
| (H5) ordinary STDP from altered activity (no credit) | SUPPORTED and sufficient to explain H1's signature: disruption raises post-vote internal firing; pre-before-post pairs arise; no valence needed. |
| (H6) structural turnover | PRESENT but non-specific (§7); cannot carry the categorical flip alone. |
| (H7) readout/measurement artifact | REFUTED: vote margins are large (spike counts 18.8 vs 264.1); world logic unit-tested; deterministic; open arm uses identical readout and does not invert. |
| (H8) schedule/order artifact | REFUTED: identical seeded schedules across arms; lag-1 -0.565 both; only the loop differs. |

## 11. Necessity/sufficiency (strict)

ESTABLISHED: (a) the closed loop caused the behavioral divergence
(only difference between arms; determinism verified); (b) the
inversion is expressed as C-dominance of output afferents
(snapshots); (c) consequence-epoch STDP contains closed-specific
boundary LTP (OUT→INT 5x, 16-23→INT 30x); (d) the rule has no
consequence representation — any credit is emergent timing.
MERELY SUGGESTED: the boundary LTP participates in the loop that
destabilizes the correct mapping (the trajectory passes THROUGH
near-correct at trials 41-70 before inverting — consistent with
feedback, not proven). NOT ESTABLISHED: any mechanism's necessity
or sufficiency (no within-E23 ablations; M2/M6/M3 contributions
indecomposable; single seed).

## 12. Learning trajectory (finest committed resolution)

10-trial blocks (§2): chance (1-40) → near-correct excursions
(41-80, benign up to 8/10) → collapse (81-100) → transient partial
recovery (101-130) → locked inversion (131-200). The mapping was
never "learned then inverted" in one step; it drifted through
correct, and the consequence statistics (disruption increasingly
dominating as benign fell) correlated with the lock-in of the
inverted state.

## 13. Closed vs open — why only the closed arm inverted

The open arm's consequence epoch is SILENT: no 16-23 drive, no
post-vote internal surge, no boundary LTP asymmetry (OUT→INT 9.9;
16-23→INT 0.18). Its output afferents drifted pro-A (+1.64) —
consistent with ordinary exposure statistics (A/C balanced; slight
A-favoring drift, seed-specific). The closed arm's disruption
provided a systematically ASYMMETRIC extra drive (only present
when votes missed, increasingly often) that (i) added 16-23→INT
and OUT→INT LTP at the boundary, (ii) raised overall firing
(+22% maturations), and (iii) via M2's zero-sum budget, displaced
A-afferents on outputs. The loop's feedback sign was effectively
INVERTED relative to the world's benefit mapping — a punishment-
channel artifact in a valence-free substrate, expressed through
sensory competition, not through action selection learning.

## 14. Final causal verdict on the stated hypothesis

"The inverted mapping is caused by consequence-driven strengthening
of the response that immediately preceded the consequence under
the existing Hebbian/STDP rules."

**PARTIALLY SUPPORTED, and materially incomplete.** The committed
data support the existence of consequence-bounded Hebbian
strengthening (boundary LTP, closed-specific) consistent with the
hypothesis's mechanism. However, the mapping inversion itself is
carried by C-vs-A afferent redistribution on output neurons — a
sensory-channel effect of the disruption drive, amplified by M2 —
not by strengthening of the punished ACTION pathway as such (both
groups, whatever they voted, were C-dominated in the end). The
hypothesis as stated (action-pathway strengthening) is NOT
supported as the sufficient cause; a revised account — consequence-
channel sensory redistribution with boundary STDP as a reinforcing
contiguity — fits all committed measurements. Necessity of any
component: unestablished.

## 15. Minimum unresolved causal questions (no experiment proposed)

1. Whether the boundary STDP component (OUT→INT/16-23→INT) is
   necessary for the inversion, or whether the sensory
   redistribution alone suffices (would require an arm with the
   consequence decoupled in time from the vote — not designed).
2. Whether the inversion is seed-robust (single-seed limitation).
3. Whether M2's zero-sum amplification is necessary (would
   require an M2-off reflex arm — gate-risky per v2 history).
4. Whether a benign-channel consequence (instead of disruptive
   16-23 drive) preserves shaping without the inversion — the
   valence-free-substrate question.

Nothing implemented, modified, or rerun; audit instrument committed
for reproducibility. STOP.