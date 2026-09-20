# CLLA amendment design review — bounded protected class (READ-ONLY)

Status: AMENDMENT DESIGN REVIEW, 2026-09-20. No implementation, no
runs, no parameters, no E-number. Builds on docs/x-clla-f4-audit.md
(root cause: unbounded protected-class growth + M2 skip-on-target≤0).

## 0. Question

Can a bounded protected synaptic class restore stability while
preserving CLLA's memory/allocation concept — using ONLY the frozen
p_max_frac = 0.75 (no new parameter)?

The amendment adds ONE thing: enforcement of the already-declared
cap P ≤ p_max_frac × t_e DURING protected-synapse growth.

## 1. Where the growth enters the code (exact site)

stdp_tick, LTP pass (crates/anima-core/src/plasticity.rs:307-350):
for each post neuron that fired, every plastic, non-inhibitory
incoming synapse with pre-trace > 0 gets

    s.w = (s.w + a_plus * pre_t * gate * beta).min(w_max)

Consolidated synapses are NOT special-cased here (per frozen §3.5:
"STDP still applies"). This is the ONLY site where a protected
synapse's weight increases (M2 excludes them; decay skips them;
M4/silence skip them; M3 only creates at w_c_permanent). LTD +
post-consolidation LTP are the only other weight movers for
protected synapses (LTD shrinks, never violates the cap upward).

## 2. Candidate semantics (A/B/C/D)

### A. Clip the LTP increment at the remaining protected-mass headroom

In the LTP pass, per (post neuron) compute the local protected mass
P (one sum over live consolidated incoming excitatory weights —
already computed by consolidated_mass(), a purely local read of the
neuron's own incoming set). For each consolidated incoming synapse:

    headroom = p_max_frac * t_e - P
    if headroom <= 0: skip this synapse's increment (no change)
    else: dw_eff = min(a_plus * pre_t * gate * beta, headroom)
          s.w += dw_eff;  P += dw_eff

Unconsolidated synapses: unchanged (normal LTP).

Properties vs the ten questions:

1. **P mathematically bounded at every tick?** YES. P is updated
   with the same increments applied to weights; the guard headroom
   ≤ 0 → skip means P can never exceed p_max_frac·t_e at the end of
   the LTP pass (last increment that would cross is clipped exactly
   to headroom; f32 rounding bounded by 1 ulp — cap check in the
   protocol uses ≤ cap + 1e-6 as before).
2. **Preserves consolidated memory?** YES. Nothing removes or
   decays protected weight; growth to the cap is still allowed.
   Squeezing continues until the neuron's protected class fills.
3. **Working synapses keep learning?** YES. The working pool is
   untouched by the clip; only consolidated synapses' increments
   are limited, and only when the neuron's P is near cap.
4. **M2 working target strictly positive?** YES, by invariant:
   P ≤ 0.75·t_e ⇒ W-target = t_e − P ≥ 0.25·t_e = 0.2 (frozen
   p_max_frac = 0.75). The "skip on target ≤ 0" branch of the CLLA
   M2 becomes UNREACHABLE BY CONSTRUCTION — but as a consequence
   of the bounded class, not by changing M2 itself.
5. **M2 itself unchanged?** YES — zero edits to normalize(); the
   branch is unreachable, not modified. (The mandate's constraint
   met literally: "Do not solve by changing if target<=0: skip".)
6. **New state or parameter?** NONE. P is computed on the fly from
   incoming synapses (existing consolidated flag); p_max_frac is
   frozen at 0.75; w_consolidate_min unchanged. Zero new RNG.
7. **Hidden capacity?** NONE. Total protected mass per neuron is
   exactly ≤ 0.75·t_e = 0.6, sum over 52 neurons = 31.2 max —
   explicit, measurable at every snapshot (already the F3 check).
8. **Neuron at capacity?** All further LTP on its consolidated
   synapses is stopped by the headroom guard; working pool keeps
   its 0.2 budget; the neuron is a full, stable memory slot.
9. **Repeated presentations of a consolidated pattern?** Synapses
   are at/near cap; increments clipped to ~0; no further growth,
   no decay, no pruning (protected) — the pattern persists
   unchanged. Mild recoil possible if LTD temporarily dips P (see
   failure-modes discussion below) — a small bounded ripple.
10. **Novel pattern after exhaustion?** Its candidates can still
    become permanent (M3) and be protected (cap check at
    permanence still admits if P + w ≤ cap — saturated neurons
    reject), and its working synapses still learn within the 0.2
    budget. But with the neuron at P = cap, its LTP on protected
    structure is clipped: the new pattern cannot displace old.
    Capacity exhaustion = graceful new-pattern non-persistence —
    exactly the architecture's declared semantics (§3.3 of the
    design doc), now actually enforceable.

### B. Reject the LTP increment when no headroom remains

Same guard, except the increment is all-or-nothing: if
P + dw > cap, apply 0. 
1. Bounded: yes.
2-4, 7-10: same as A except:
- Wastes residual headroom: with P at 0.59 and dw = 0.02, B applies
  0 instead of 0.01 — the synapse holds at 0.59 forever, and every
  future attempt also rejected → the class silently wedges below
  its own cap. A uses headroom fully.
- Simpler (no per-step P update needed if using "reject if P+dw
  > cap"), but behaviorally a cliff: at the boundary, increments
  flip from full to zero with no partial region. Less faithful to
  "LTP still applies" (the frozen §3.5 wording).

### C. Prevent further consolidation at cap

This is ALREADY implemented (the permanence gate P + w ≤ cap).
It does NOT bound post-consolidation LTP — which is exactly the
measured failure (F3 excess built through LTP on protected
synapses, not via new permanence events: nC grows 13→330 after
first cap breach). C alone = status quo. Not an amendment.

### D. Other local implementations

- Per-synapse ceiling (w ≤ w_cons): does not bound P (many synapses
  each near ceiling ⇒ P unbounded; this was the F4 failure mode 3
  in the design doc — "bounded by w_max" was falsified).
- Normalize the protected class itself: adds a second M2-like
  process = new mechanism + new parameter-like behavior; violates
  "smallest change" and the "do not change M2" constraint in spirit.
- Decay reintroduction for protected synapses (partial): re-opens
  the SDE-C2 eraser path we deliberately closed; changes protection
  semantics beyond a growth bound (mandate: "protection exemption
  semantics unchanged except the growth bound itself"). Rejected.

### Decision: A — clip at headroom.

It is the smallest change (one guard + one min in the LTP pass,
local reads only), keeps every frozen semantic intact, enforces the
already-frozen cap, and it is genuinely "STDP still applies" —
the increment is scaled to what the class headroom permits, the
same way (a_plus · pre_t) already scales with the trace.

## 3. Why this is the minimum change

- One site (stdp_tick LTP), one local variable (P per post), one
  min(). No new params, state, events, RNG.
- Does not touch: M2, M6, M3/M4/M5, STDP params, thresholds,
  curriculum, allocation rule, retrieval, decay exemption.
- The F4 causal chain is broken at its FIRST link: unopposed LTP
  on protected synapses. Clipping the increment at the class cap
  keeps P ≤ cap always, so:
    1. M2 target ≤ 0 branch becomes unreachable (target ≥ 0.2);
    2. working pool is never abandoned;
    3. total drive cannot grow 5-10× the budget via protected mass;
    4. P2 runaway source removed.
- Counterfactual support (from the F4 audit's own trajectories):
  the cap engages at t = 8-14k in ALL 12 runs — i.e., well BEFORE
  the rates diverge between F4 and ok runs (matched-time table:
  F4/ok track together through t=40k with P 77-108; the divergence
  is late). Clipping from t≈10k would cap total protected mass at
  ≤ 31.2 (52 × 0.6) vs observed 57-225 — a 2-7× reduction at the
  exact stage where rate escalation begins.
- Working-mass sufficiency at the frozen p_max_frac: W-target =
  t_e − P ≥ 0.25 × 0.8 = 0.2/neuron. Is 0.2 "enough"? It equals
  25% of the full budget that the unmodified organism runs stably
  on (e24 baselines complete at t_e = 0.8 with healthy activity).
  The working pool also includes unconsolidated afferents from all
  present patterns plus the survivors' capacity for new learning —
  0.2 per neuron is a positive, functional normalization target;
  the question of "enough for WHAT" (new learning) is exactly what
  the memory-capability test (later, after viability) will answer.

## 4. Expected effect on the F4 causal chain (predicted, to be tested)

Chain (from x-clla-f4-audit §7):
  consolidation → unopposed LTP → P > t_e → M2 skip → W unbound →
  drive 5-10× → P2 trip.
With A:
  consolidation → LTP clipped at P = 0.6 → P never exceeds cap →
  W-target ≥ 0.2 always → M2 always normalizes working → total
  mass bounded ≈ 31.2 + per-neuron working ≤ 0.2 → drive stays in
  the historical stable band → P2 not tripped by this pathway.

The d-arm and rate-lottery aspects remain: the amendment does NOT
address seed-specific recurrent dynamics nor the D stimulus's 2×
input load. Both are separated in §5/§6.

## 5. D-arm confound treatment

The d arm crosses 50 Hz in its FIRST presentation at P ≈ 0.16/neuron
max — far below the cap; the cap does not even engage before the
crossing. The amendment therefore cannot be expected to fix the
d arm, and d-arm stability must NOT be evidence for/against the
bounded class. Two possibilities:
- D (16 channels at 20 Hz) is intrinsically over-driving the
  organism (no flag-off D run exists — unanswerable from current
  artifacts).
- or the first-presentation burst is transient and the sustained
  tipping requires the mass-growth pathway (s9001-d: 139 Hz spike,
  then 32 Hz at t=10k — a burst-then-decay shape; the runoff is
  what trips sustained >50 Hz).
Resolution requires either a flag-off D run (new run, not allowed
in this review) or treating the d arm as INFORMATIVE ONLY in the
amendment protocol (viability endpoints on il/bac/bca arms; d
reported separately, never as sole evidence).

## 6. Minimal falsifiable stability protocol (design; NOT executed)

Goal: does CLLA + bounded protected growth remain dynamically
viable? Memory capability NOT tested here.

Config: identical to the frozen CLLA arm set, ONLY the LTP clip
active. Everything else frozen: e24 cell (β=0.0046875, τ=5000),
m2_buckets=1, seeds {20260912, 424242, 9001}, arms il/bac/bca
(primary) + d (informative-only), curricula as committed.

Endpoints (pre-registered, no post-hoc thresholds):
  V1. P2 stability: 0 failures (runaway) across all primary arms ×
      seeds (9 runs). FAIL ⇒ amendment not viable.
  V2. Cap invariant: P ≤ 0.75·t_e + 1e-6 at every snapshot every
      neuron (F3, unchanged).
  V3. M2 working target positive: W-target = t_e − P > 0 everywhere
      (automatic given V2; reported as the invariant check).
  V4. Anti-modal: no new failure class introduced (failure count
      zero; P2 detector untouched; no resource events beyond
      historical baseline).
  Primary verdict: viable iff V1-V4 in all 9 primary runs. The d
  arm's outcome is reported separately and does NOT enter the
  viability verdict (confound documented).

Identity gate: unchanged — flag off = byte-identical CLLA-off
path (the clip is inside the assembly_protect-guarded branch? NO:
the clip must be gated on assembly_protect so flag-off runs take
the exact stdp_tick LTP path — verify the guard is: if
assembly_protect && s.consolidated { clip } else { normal LTP }.
Flag-off → consolidated never set → identical.)

Counterfactual-before-running: described in §7 — the amendment is
small enough that a deterministic offline replay of the committed
trajectories (weights + spikes) can simulate the clipped LTP path
on the SAME network state sequence and directly show P ≤ cap and
W-target > 0 as invariants hold during replay. This validates the
BOUNDEDNESS logic without running the organism; it cannot validate
the dynamical effects (spikes change under the clip), which is what
the 9-run protocol then tests.

## 7. Read-only counterfactual feasibility (answer to the mandate)

Possible IN PART:
- YES for invariant-boundedness: simulate the LTP clip on the
  committed snapshot-to-snapshot weight trajectories (each
  snapshot is a full network state; apply the clip rule at each
  step, verify P ≤ cap and W-target ≥ 0.2). This is a faithful
  deterministic check of the AMENDMENT'S BOUNDEDNESS PROPERTY.
- NO for dynamical consequences: clipping changes weights → changes
  spikes → changes everything downstream; that cannot be
  counterfactually replayed without simulating the organism, which
  is a run. So the viability question (does bounding RESTORE
  stability) is not answerable offline — the 9/12-run protocol
  (§6) is the minimal truth test, per the mandate's alternative.

## 8. Risks / failure modes of the amendment

- Clipping may starve the LTP that the survivor runs used to reach
  their (unstable) equilibria — no evidence this path is needed for
  viability; the ok runs are stable well before their P grows past
  cap.
- A neuron whose protected class fills early is closed to that
  pattern's further strengthening; if the pattern's synapses
  need headroom to grow past w_c_permanent (0.02 → useful? no:
  consolidation already happens on permanence), no issue.
- LTD dipping P slightly below cap reopens a tiny LTP window
  (ripple); bounded, mirrored by F7's existing erosion check.
- Interaction with E6 β-scaling in the LTP increment: the clip is
  applied AFTER the β-scaled increment would be computed — no
  change to E6's role; unconsolidated path bit-exact.
- The d arm is NOT expected to be fixed (see §5) — a d-arm
  failure in the protocol must not, by itself, refute the bounded
  class; conversely a d-arm pass must not be claimed as evidence
  for it.

## 9. What must NOT change (frozen constraints, repeated)

beta, M2 (code), M6, STDP rule/params, curriculum, thresholds,
p_max_frac = 0.75, w_consolidate_min = 0.05, allocation rule,
retrieval, decay/prune protection exemptions (except the added
growth bound at LTP). Identity: flag-off byte-identical.

STOP. Design review only; nothing implemented, nothing run.