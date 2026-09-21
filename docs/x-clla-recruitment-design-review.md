# Local novelty-dependent recruitment gain — mechanism design review (READ-ONLY)

Status: 2026-09-21. Grounded in committed corrected-fe runs +
telemetry/snapshots + the frozen membrane/allocation code
(crates/anima-core/src/network.rs, structural_v2.rs, plasticity.rs).
No implementation, no runs, no tuning, no E-number.

## 0. Grounded substrate (measured/verified)

- Membrane (network.rs step): dv =
  (−(v−v_rest) + i_syn + i_ext − i_adapt + u_eff)·dt/τ_m, dt = 1,
  τ_m = 20 ms, v_th = 1.0 (frozen), v_rest = 0.0, refractory = 2.
- CLLA residual signal (structural_v2.rs): per-neuron per-tick
  accumulators res_ip (delivered current from consolidated input-
  channel afferents), res_iw (unconsolidated); R = ip/(ip+iw);
  SILENCE CONVENTION: R := 1.0 when total = 0. Gate =
  max(0, 1−R) × (headroom > 0), computed from just-finished
  windows; window_ticks = 100.
- Measured first-C drive: I_aff = 1.565, 100% working (R = 0),
  I_rec = 0.05, posts/n = 0.12; A first-block: 11.98/6.90/0.98.
- Measured drive→posts map (all presentations, bac run):
  flat plateau posts ≈ 0.10–0.12 over drive 1.6–4.2 (pres 21–40);
  saturation 0.98 at drive 38 (pres 20). Linear interpolation:
  posts ≈ 0.0246·drive + 0.034 (mid-band [4, 19] UNMEASURED —
  no presentation ever operated there).
- Measured R trajectories: C-block (1−R): 1.0 (pres 21) → 0.91
  (pres 25) → 0.43 (pres 30) → 0.28 (pres 40). A-block (1−R):
  1.0 (pres 1) → 0.38 (pres 6) → 0.17–0.26 (pres 11–20).
- Existing dimensionless constants: all ≤ 1 (adaptation_gain
  0.05, inhibition_gain 0.5, a_plus/a_minus < 1, phi_rel 0.5).
  g_drive ≡ 0 in committed clla-fe runs (slow_state_beta_drive
  default false — only xdg-exp enables it). The identity
  coefficient (×1) is the LARGEST parameter-free multiplier.

## 1. Exact proposed equation

```
i_boost_i(t) = k_g · g_i(t) · I_W,i(t)

g_i(t)      = max(0, 1 − R_i(t))                 # allocation-rule residual
R_i(t)      = res_ip_i(t) / (res_ip_i(t) + res_iw_i(t))
              if res_ip+res_iw > 0, else 1.0     # EXISTING silence convention
I_W,i(t)    = Σ_{fired input channels c, live synapses c→i,
              unconsolidated} amplitude·w         # EXISTING per-tick term
              (the same value accumulate_input_current adds to res_iw)
```

Integrated as: dv = (−(v−v_rest) + i_syn + i_ext + i_boost − i_adapt + u_eff)·dt/τ_m.

Reading RAW weight into cured form: the gain acts ONLY on the
neuron's own unexplained input-channel current I_W, at the
allocation rule's own residual fraction (1−R). It is the CLLA
"unexplained-drive" signal (existing), re-issued as membrane
current. k_g is dimensionless and UNAVOIDABLY NEW (see §9) — per
the mandate we do NOT set it; the frozen probe (§10) measures it.

## 2. Exact local variables (all existing)

res_ip, res_iw (per-tick accumulators, already maintained),
their silence convention R := 1.0, the per-tick delivered
current value (computed in accumulate_input_current's deposit
scan), v, v_th, τ_m. No new state: the rule reads the existing
accumulators mid-window (they are cleared at window boundaries —
same cadence the allocation rule already uses).

## 3. Units and bounds

- i_boost: current units (same as i_syn; added inside dv).
- g_i ∈ [0,1] (existing gate, parameter-free, headroom-frozen:
  the allocation rule ALREADY multiplies by headroom>0 — the
  boost inherits that accounting gate verbatim).
- 0 ≤ i_boost ≤ k_g·I_W ≤ k_g·(per-tick afferent current) —
  bounded by the neuron's OWN input availability, never by
  population state.
- Bounded: for the measured first-C (I_W ≤ 1.565) and the
  counterfactual bracket k_g ∈ [6,10]: i_boost ≤ 15.7 — the
  SAME order as A's steady protected+working drive (≈26.7) —
  no regime outside what the organism already survived.
- The boost is transient: I_W = 0 between/after presentations
  (no input spikes → no current → boost 0).

## 4. Temporal semantics

- Turn-on: within 1 tick of the first unexplained spike of a
  presentation (the accumulators receive that tick's delivered
  current; the boost reads them live). This is the existing
  1-tick synaptic-delay semantics — no new timer.
- Before the first spike: nothing happens (no current yet).
- After the first few posts: the boost PERSISTS while R is low
  (measured: (1−R) ≈ 0.91 still at pres 25 — the whole
  bootstrap phase is boosted), then DECAYS as consolidation
  converts working → protected: measured (1−R) 0.43 → 0.28
  (pres 30 → 40).
- Disappearance: (a) immediately when I_W → 0 (between
  presentations); (b) progressively as R → 1 (existing M3
  permanence timescale, ~30 presentations — measured decay);
  (c) hard-zero when the P headroom closes (existing gate).
- No new timescale: window_ticks (100 ms), 1-tick lag, and M3's
  consolidation dynamics — all existing.

## 5. Proof of zero endogenous-triggering (pacemaker test)

A neuron firing endogenously has no input-channel spikes.
Two independent zeros, both from EXISTING quantities:

1. I_W,i(t) = 0: the deposit scan only fires on input-channel
   afferents; an endogenous spike deposits only recurrent/
   inhibitory current, which is NOT part of I_W. ⇒ i_boost = 0.
2. res_ip = res_iw = 0 over any input-silent period ⇒ the
   existing silence convention sets R := 1.0 ⇒ g_i = 0. ⇒
   i_boost = 0.

Either alone suffices; both hold. A pacemaker can never
self-amplify: its own spikes feed neither the gate nor I_W, and
u_eff (V2.1 slow state) is outside the gate's read set. ∎

## 6. Positive-feedback safety analysis (self-limiting)

Loop: boost → posts → (M3 coactivity) → permanence → res_ip ↑ →
R ↑ → g ↓ → boost ↓. STRICTLY NEGATIVE feedback via the existing
consolidation machinery, monotone, and it is THE loop the
organism already uses to settle allocations. Secondary negative
loops (existing): i_adapt +0.05/spike (hyperpolarizing, opposes
sustained firing), M6 inhibition (anti-Hebbian, rises with
co-firing), refractory (2 ticks), and the hard 50 Hz/5 s runaway
detector unchanged.

The one intended positive loop is the RECURRENT bootstrap
(weak drive → first posts → recurrence → stronger response).
Its open-loop gain is bounded: (a) boost is capped at k_g·I_W
(own-input-bounded, k_g ≤ 10 ⇒ ≤ 15.7 current ≈ A's normal
drive); (b) recurrence is NOT boosted (I_rec ∉ I_W — the boost
is input-channel-only by construction), so the loop gain
carries no multiplier beyond the bare recurrent factor the A
block already demonstrates without instability; (c) posts/n
saturates at 1.0 (refractory/adaptation), so the bootstrap's
amplifier is gain-limited at the same operating point A already
lives at. No runaway fixed point is reachable that does not
already exist in the A-block operating regime (which survived
875 s with the same guards).

## 7. CLLA / M2 / M6 interaction

- R ≈ 1 (protected explains drive): g = 0 → no boost. Measured:
  established A (pres 20) would see (1−R) = 0.16 → boost ≤
  0.16·k_g·4.2 ≈ 0.7–6.7 vs its 26.7 drive — ≤ 25%, declining
  to 0 as R → 1. No amplification of established memory.
- R low (unexplained): g positive → boost possible. Measured:
  first C R = 0 → full gate.
- Protected cap: UNCHANGED (p_max_frac·t_e); the boost adds
  CURRENT only — no synapse is created or strengthened by it.
  M3 permanence still governs consolidation; the existing
  headroom gate multiplies the boost, so when P is full the
  boost is denied exactly as allocation is.
- M2: untouched (no mass is created; the boost is not a
  synapse-count or weight operation). M6: untouched (the rule
  neither reads nor writes inhibitory synapses; M6's existing
  activity-driven dynamics are the only inhibition path).
- Resource accounting: NOT bypassed — the boost is a transient
  membrane current, invisible to synapse budgets; consolidation
  pressure it creates flows through the existing M3/P-cap gates.

## 8. Retrospective counterfactual

Method: trace-level arithmetic on the measured per-presentation
rows of the committed bac run — afferent term scaled by the
proposed boost at the MEASURED gate, posts read off the fitted
measured map, recurrence coupling bounded from A-block data.
NO organism dynamics altered.

Fitted map (measured endpooints): posts(1.6–4.2) ≈ 0.10–0.12
(FLAT PLATEAU), posts(38) = 0.98. Linear interpolation across
the unmeasured band [4.2, 19]: posts ≈ 0.0246·drive + 0.034.
A-block recurrence: at posts ≈ 0.5–1.0, the next presentation's
recurrent drive is 5–22 (I_rec grew 6.9 → 21.6 in the first
two presentations); C-block recurrence stays ≤ 0.13 while
posts ≤ 0.12 (no posts to feed it).

Counterfactual with boost (first-C gate g = 1, I_W = 1.565):

| k_g | drive pres 1 | posts pres 1 | +recurrence pres 2 | verdict |
|---|---|---|---|---|
| 1 (identity, parameter-free) | 3.1 | 0.11 | 3.2 | PLATEAU — no bootstrap |
| 2 (max parameter-free) | 4.7 | 0.15 | 4.8 | plateau edge — no bootstrap |
| 5 | 7.8 | 0.23 | 9–11 → 0.28–0.30 | stalls (posts < 0.4) |
| 8 | 12.5 | 0.34 | 16–19 → 0.44–0.50 | borderline |
| 10 | 15.7 | 0.42 | 20–24 → 0.55–0.62 | LIKELY bootstraps |

Bootstrap regime (posts ≥ 0.5/n, where A's recurrence demonstrably
engages) requires sustained drive ≥ ~19 → k_g ≥ ~8–10 under
A-type recurrence, ≥ 12 without it. k_g ≤ 5 stalls on the
plateau+sigmoid; the identity/parameter-free forms (1–2×) are
PROVABLY insufficient: they cannot lift drive above the measured
plateau's upper edge (4.2), where posts stayed 0.10–0.12 for 20
presentations.

Same rule on the controls:
- Established A: gate ≤ 0.16, I_W(A remnant) ≤ 4.2 → boost ≤
  ~7 at k_g=10 vs A drive 26.7 — ≤ 25%, posts already saturated
  at 1.0 — no observable change; declines to 0 as R → 1.
- Endogenous: §5 — zero by construction.
- Ordinary strong drive (first-block A): pres 1–2 get the full
  gate (R = 0) → boost ≈ k_g·I_W. POSTS SATURATE at 1.0
  (cannot exceed), and R decays 1.0 → 0.38 by pres 6 — the
  boost is self-terminating within a few presentations. The
  measurable risk is RATE, not posts (see §10 — the probe must
  measure rate; the 50 Hz/5 s guard + adaptation bound it).
- IL: each pattern gets the boost only during its pre-
  consolidation windows (measured IL C posts already 0.86–1.0
  pre-consolidation → saturation-limited; late IL R ≈ 0.9 →
  gate ≈ 0.1 → no effect). Coexistence preserved — the boost
  does not "amplify every unexplained transient", it decays
  with the SAME R the consolidation machinery computes.

Honest limits: the map's middle band [4.2, 19] is UNMEASURED
(no committed presentation ever operated there); the recurrence
coupling fits are unstable (A-block posts saturate while I_rec
fluctuates); the counterfactual brackets the crossing but cannot
resolve it. This is precisely why the frozen experiment (§10)
MEASURES the map instead of trusting the interpolation.

## 9. Is a new free parameter necessary? — YES; STOP and report

Every existing attainable magnitude is ≤ 1× on the unexplained
component (identity; all frozen dimensionless constants ≤ 1;
dividing by window_ticks, τ_m or v_th yields ≤ 1×; re-injecting
the previous window's average unexplained current is identity by
definition). The counterfactual requires ~8–10×. Therefore:

  A membrane-side recruitment gain with the required magnitude
  is NOT expressible from existing quantities and existing
  constants. A new dimensionless gain parameter k_g ∈ [6, 10]
  is UNAVOIDABLE.

Per the mandate we STOP here rather than invent one. k_g is the
ONLY new quantity; the rule's form, gate, domain, bounds,
temporal semantics and self-limiting structure are all fully
determined by existing local quantities (sections 1–8).

## 10. The SINGLE smallest frozen experiment (protocol only, NOT run)

PURPOSE: measure the unmeasured drive→posts map band [4.2, ~19]
so that k_g becomes a MEASURED value (k_g := I_cross / 1.565),
never a chosen one. This removes the only free parameter from
the design — by construction.

- Config: committed clla-fe-s20260912-bac seed/config, organism
  dynamics UNCHANGED — the recruitment rule is NOT implemented.
- Instrument: the existing per-neuron external current probe
  mechanism (network.rs i_ext, "used by probes/tests");
  telemetry as in the audited runs.
- Design: during seconds-block C presentations (pres 21–30),
  inject i_ext = L on every internal neuron that received
  working C current in the preceding C presentation (a set
  defined by the run's own telemetry, no labels), L ∈ {0, 2, 4,
  6, 8} current units — one presentation per level, fixed
  ascending order, L = 0 first (control re-measurement of the
  plateau).
- Measures: posts/n AND internal spike RATE per presentation
  (rate matters: posts saturate at 1.0 — the rate curve
  determines how close to the 50 Hz guard the crossing sits).
- Pre-registered outcomes:
  - PASS: monotone increase, posts ≥ 0.5/n at some L ≤ 8, no
    runaway, and an L = 0 A re-test (pres 31) retains A posts ≥
    0.9 × baseline (no damage to the established representation).
  - FAIL: runaway (existing detector) or posts stay < 0.5 at
    L = 8 → the map band extends higher than the pre-registered
    bracket; a pre-registered stage-2 probe at L ∈ {10, 12, 16}
    then resolves the crossing (two-stage titration, not tuning).
- Output: I_cross (smallest L with posts ≥ 0.5) → k_g :=
  I_cross / 1.565, a measured constant with the form already
  frozen by this review. The rule (equation of §1) may then be
  implemented ONLY as a follow-up experiment with that single
  measured k_g and the §9 safety criteria as gates.

This is the single smallest experiment: it changes NOTHING about
the organism (probe-only), measures the one missing datum, and
converts the unavoidable parameter into a measurement.

## 11. Architectural status

(C) — a LOCAL SYNAPTIC-CURRENT-TO-EXCITABILITY COUPLING: the
neuron's own, already-computed unexplained input current is
re-issued into its own membrane equation at the residual
fraction, transiently, until the CLLA machinery explains the
input. It is not (A) a new intrinsic recruitability state (no
new state variable exists — the accumulators are already
maintained), and not (B) a naked modulation of membrane
parameters (τ_m/v_th untouched — the coupling is input-selective
and vanishes with the input). It adds no memory subsystem, no
population signal, no pattern identity, no novelty label, no
trial count. It cannot know A/C/BAC/BCA; it cannot permanently
boost a channel (gate → 0 as R → 1); it cannot bypass resource
accounting (headroom gate inherited, no synapse operations).

## 12. Summary

- Form: fully determined by existing local quantities.
- Magnitude: k_g ∈ [6, 10] REQUIRED, NOT derivable from
  existing constants — the single unavoidable new parameter;
  per mandate: STOP and report instead of inventing.
- Pacemaker: zero by double construction (§5).
- Positive feedback: bounded, self-limiting via consolidation,
  adaptation, inhibition, refractory, guard (§6).
- CLLA/M2/M6: untouched; accounting intact (§7).
- Counterfactual: brackets the crossing but cannot resolve it —
  the map's middle band is unmeasured (§8).
- Next: the frozen map probe (§10) turns k_g into a measurement.

No implementation, no runs, no tuning, no E-number.

STOP — review complete.