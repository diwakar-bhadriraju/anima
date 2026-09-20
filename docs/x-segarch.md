# V2.1 "episode segmentation → persistent write" — mechanism-design analysis

Status: READ-ONLY DESIGN, 2026-09-20. No implementation, no runs,
no tuning, no E-number. Built on the committed S1 benchmark
(x-mechanism-review.md appendix; threshold ±0.0020 predeclared).

## 0. The architectural question

How can S1 prevent cross-presentation accumulation in u without
resetting u (persistence must survive episode end)?

Critical numerical input (from the committed benchmark): the
66%/96% TP figures are PER-NEURON-PAIR. Per-presentation edge
coverage — P(at least one of 52 neurons detects the edge) —
is ~1.0 in BOTH regimes (P(all miss) = 3.7e-25 weak / 8.5e-76
wedge). This reframes everything: a local mechanism keyed on
"any neuron's edge" has effectively zero missed-episode rate;
per-neuron misses only affect that neuron's own accounting.

## 1. Candidate mechanisms (A/B/C)

### A. Episode gate (write-enable latch)
S1-rising opens a per-neuron write latch; S1-falling closes it;
u += beta per spike only while latch open.
- State: 1 latched bit/neuron.
- Missed offset (per-neuron ~34% weak cell): that neuron's
  latch stays open through the silence → its pacemaking writes
  β per spike → re-creates the exact cross-accumulation the
  mechanism exists to prevent, for that neuron, until the next
  onset re-opens... i.e. ~34%×1.5 s gap leakage. Missed onset
  merely delays opening (benign).
- False offset in silence: S1 floor 0.0002 « ±0.0020 → none
  (measured).
- Verdict: latching a per-neuron BOOLEAN inherits the per-neuron
  TP. Weak cell → ~1/3 of each gap still leaks. FAILS the
  "prevent cross-presentation accumulation" requirement in the
  weak cell.

### B. Episode buffer + commit (RECOMMENDED)
Per-neuron episode accumulator b (u-units, i.e. adds beta per
own spike), CLEARED at onset-edge, COMMITTED (u += b) then
cleared again at offset-edge.
- The per-neuron miss problem disappears: if neuron i misses its
  own edges, its b just accumulates across the gap — but the
  COMMIT is keyed on ANY neuron's offset edge, and since edge
  coverage ≈ 1.0, b gets committed at every true boundary anyway
  (~never missed globally). After commit, ALL neurons clear b —
  including the neuron that missed its own edge. The miss is
  thereby corrected by the population edge, WITHOUT any global
  mechanism: each neuron clears WHENEVER the local commit signal
  arrives, and the commit signal is just "some S1 fell" — which
  each neuron can know only through... see §3 for the strictly
  local commit trigger.
- u is never zeroed: at offset, u += b (additive, persistent).
  Persistence survives episode end by construction.
- Next episode: onset clears b; u untouched → old memory
  persists until the next commit.
- Verdict: separates WRITE-EPOCH (b) from PERSISTENT-STATE (u)
  with exactly one new state var; per-neuron TP shortcomings are
  laundered by the ≈1.0 population edge coverage.

### C. Existing-state reuse — examined and REJECTED as primary
- i_syn: rejected as a segmenter by the benchmark (offset-only,
  recurrent-contaminated, mid-presentation dips).
- g_drive (V2.1 X-series EMA, Iaff20-equivalent): exists as a
  state var from the drive-gated experiment and IS the S1 fast
  leg — but reusing it as a WRITE GAIN (the X-series design)
  failed for scale reasons (u collapsed; NOT its role here). As
  an EDGE SOURCE its information is identical to S1, and it
  lacks the slow leg → no signed offset. No simpler local state
  suggests itself; C does not beat B.

## 2. The one genuinely hard design point: the commit trigger

B as stated needs "some neuron detected the offset" — a
population fact. The mandate forbids population/global activity.
Resolution options:

1. Per-neuron b with SLOW LEAK, no explicit commit signal:
   b decays (tau_b ~ 250 ms = S1's slow leg) and u += b·rho
   continuously (rho small). Then "commit" is the integral of
   the leak; boundary is marked by b's PEAK (end of sensory
   drive), not by a detected edge. MISSED edges are irrelevant:
   b peaks when drive ends regardless of edge detection. FALSE
   edges in silence: b=0 in silence (no input) so nothing to
   commit. This needs NO edge detection at all — the boundary
   comes from the input ending, which S1's offset edge merely
   mirrors.
   → This is the strictly-local completion: b is a leaky
     episode integrator; u += rho·b per tick (or b → u on a
     local b-downcrossing); every quantity is per-neuron.

2. Local b-peak commit: commit when b starts to decline after
   reaching a local maximum (per-neuron derivative of b). All
   local; no edges needed.

Option 1 (leaky b, hill-climb-into-u) is the minimal honest
form: it uses ONLY "my input stopped" — which is exactly the
information S1's offset edge carries, but expressed as b's own
decay rather than a threshold. The S1 edge study is then the
VALIDATION that input-offset is locally detectable (measured),
not a required component of the mechanism.

Recommended mechanism shape:

```
state:  u  (persistent, unchanged identity when off)
        b  (episode buffer, NEW, per neuron; zero when off)
params: rho (commit rate, 0 < rho <= 1 per tick; see §4)
        tau_b = 250 ms (SHARED with S1 slow leg: no new timescale)

per tick, flag ON:
  on own spike:  b += beta            # episode-local write (was: u += beta)
  b *= exp(-dt/tau_b)                  # leak: b bounded even in long episodes
  u += rho * b                         # continuous commit (persistent write)
identity (flag OFF): u unchanged; b unused; exact V2.1 path
```

Boundary semantics fall out naturally:
- onset: b starts accumulating; u keeps whatever it had.
- offset: drive ends → b stops growing and decays → commits
  taper. b peaks approximately at drive end.
- silence: b → 0; u frozen (rho·0); NO false writes (basis:
  input absent, measured b-floors zero).
- next onset: b re-accumulates; u preserved (old memory)
- overwrite/coexist: u is a running integral of leaky-b
  episodes — new episodes ADD to u; old content is not
  destroyed, it is weighted by rho and the leak's memory. An
  episode's contribution to u at late times decays? NO: once
  committed (u += rho·b while b>0), u keeps it; u never decays.
  Coexistence = additive superposition of episode traces, each
  scaled by its b peak (≈ episode size). Sequential episodes
  thereby COEXIST in u with relative weights = their sizes.

## 3. The 12 questions, answered for the recommended design

1. State vars: u (existing) + b (one new per-neuron scalar) +
   the leak/commit params. rho and tau_b are the only new
   constants.
2. Genuinely new: b; reuses: u, spike stream, tau_b 250ms
   (S1-slow-leg value, existing experimental timescale), beta.
3. Onset semantics: b begins integrating own spikes; no explicit
   event, no reset of u.
4. Offset semantics: b's leak dominates; commits taper with b.
   No explicit event, no u reset.
5. Silence: b → 0 exponentially; u += 0 → persistent state
   FROZEN during pacemaking — the cross-accumulation enemy is
   structurally gone (pacemaker spikes still write b, but b
   leaks to ~0 in silence and commits nothing by offset
   semantics? CAREFUL: pacemaker spikes DO add beta to b in
   silence, and u += rho·b would then commit pacemaker
   activity! See §4 — the identity/boundedness fix).
6. Next onset: fresh b accumulation on top of existing u.
7. Old persistent info survives: YES — u never reset.
8. Overwrite vs coexist: COEXIST additively (weighted by
   episode size and rho).
9. External trial boundary? NO — no count, no duration, no
   marker; boundaries emerge from local input statistics.
10. S1 used how? Validational only (proved offset detectability
    and zero silence FPs); the mechanism itself is edge-free.
    This is the cleanest reading of the mandate: S1 is the
    EVIDENCE that segmentation is possible, not a component.
11. Curriculum-blind, reward-free: yes — b, rho, tau_b are
    pattern-agnostic; no classifiers.
12. Identity when disabled: flag off → b never touched, u
    follows exact V2.1 path → C-S5-style byte identity
    (FNV/snapshot hash), same as X-series gates.

## 4. The silence-write problem and its resolution

Pacemaker spikes in silence would write b and hence u — 
reintroducing exactly what we removed, since pacemaking HAS
spikes. Resolution options, in order of principle:

(a) b only accumulates from spikes while b's INPUT-DRIVE gate is
    high — i.e. b's write is ALSO drive-gated (b += beta only
    when Iaff20 > 0). Then silence pacemaker spikes never enter
    b; no new state (Iaff20 exists). This is the union of the
    two X-series ideas, now with correct roles: gating for
    write-eligibility (b), leak-commit for persistence (u).
    Predictable cost: b drops to ~0 within 20ms of input end →
    commits taper fast after offset; in-episode b ≈ sensory
    spikes only. This is the RECOMMENDED form.
(b) rho very small: pacemaker noise in u suppressed by
    averaging — leaves slow leak of background into u; weaker.

With (a): silence b ≡ 0 (no input → Iaff20=0 → no b writes;
leak keeps b=0), so u += 0 in silence even though pacemaking
continues. Cross-presentation accumulation is eliminated exactly
in the sense required: u only integrates spikes that occurred
while that neuron received input drive.

## 5. Failure modes

- F1 Missed onset (per-neuron): b simply starts later; u
  unaffected; nothing corrupts. Global coverage ~1.0 anyway.
- F2 Missed offset: with (a) the DRIVE END is the real event —
  b stops growing when input stops regardless of edge detection.
  Edges are not load-bearing. No state corruption.
- F3 False onset/offset (silence): impossible by construction
  (input absent ⇒ Iaff20=0 ⇒ b=0 ⇒ no commit).
- F4 Long episodes: b bounded by leak (b_max ≈ beta·rate·tau_b);
  u grows linearly in episode count but each contribution is
  size-bounded → no runaway in u.
- F5 Episodes with ZERO sensory drive (pure endogenous epochs):
  b=0; u unchanged; silence semantics = idempotent. Good.
- F6 rho too large → u ≈ running mean of b (over-commits,
  blurs episodes); too small → episodes under-represented.
  This is a genuine free parameter! Mitigation: rho must be
  FIXED at design (not tuned post-hoc) — pre-register e.g.
  rho = 0.05 (commit over ~20 ms of drive), justified as
  "commit within the fast-leg timescale"; identity unaffected.
- F7 b and u readout ambiguity: telemetry must carry b
  (additive Option field, like g_drive/z_latch precedents).

## 6. Smallest distinguishing experiment (NOT run; read-only first)

Two-tier, both on committed traces (no new runs):

T0 (required, identity): flag-off simulation must reproduce the
committed u trajectory bit-exactly (validates the simulator:
u += beta per spike, exact decay). Do this on v21repac +
v21rep runs.

T1 (distinguishes A vs B semantics WITHOUT implementing):
deterministic simulation of the recommended mechanism (b leaky,
Iaff20-gated, u += rho·b) over the committed A/C spike stream
using measured S1/Iaff20 traces as drive gate; endpoints:
 - u_new(A-window) vs u_new(C-window) cosine INCREASES vs the
   committed u during-match (0.9995) → barrier to demonstrate
   the mechanism restores A/C separation, at least in the
   per-presentation windowed regime;
 - u silence growth: |u(end-of-S1) - u(end-of-silence)| ≈ 0
   (vs contemporary u which grows in silence);
 - b boundedness: max b < beta·rate·tau_b across all episodes.
This simulates the ARCHITECTURE ex ante; it does not modify the
organism. If T1 shows u-new retaining A/C separation while
frozen in silence, the next step (a frozen mechanism
experiment, with user approval) is justified.

## 7. Identity-gate requirements (for the eventual frozen experiment)

- New config: `episode_commit_enable: bool = false` (identity)
  + `episode_commit_rho` + `episode_buffer_tau_ms` (250).
- Flag-off: code path untouched → committed X-series identity
  standard: C-S5 FNV + snapshot SHA-256 equal to the closest
  committed run; no RNG draws; g_drive/Iaff machinery either
  off (no state change) or read-only.
- Exact identity anchors: runs/v21rep-b0.00625-t5000-... and
  runs/v21repac-b0.003125-t10000-... (FNVs already recorded).
- Suite tests (like x_drive_identity_gate): full-run event-FNV
  + snapshot hash + flag-on scale sanity (b bounded, u-gain per
  episode in [0, rho·b_pk]).

STOP after design analysis; nothing implemented.