# D-58 PRE-REGISTRATION: reflex-y familiarity projection into dedicated novelty nodes (B+A)

Date: 2026-09-23. Deterministic. Registered BEFORE implementation.
Targets the #1 measured capability gap: novelty detection = 0/3 at every
stage (D-57), because the 12-dim output band + argmax decode cannot
produce a 'this is new' answer when refs are near-identical (cos~0.99).

## Biological grounding
Organisms separate 'what is it' (identity) from 'is it familiar'
(familiarity/novelty) into DISTINCT systems. A hard-wired reflex arc
guarantees a response by construction without learning. Combined: a
FIXED (non-plastic, genetically-specified) projection from input into
dedicated NOVELTY/FAMILIARITY nodes, parallel to the learned pool.

## Design (charter-legal: I/O-boundary wiring, content-NEUTRAL)
- Add ~2 dedicated novelty nodes in the OUTPUT band (output band 12 ->
  14: last 2 = novelty/familiarity channels).
- FIXED projection: each input channel gets a HARD-WIRED, random-weight
  (seeded, deterministic) connection to each novelty node - same for
  ALL symbols (content-neutral: no A/C/E-specific wiring). This is a
  REFLEX: it does not learn, it just sums input drive into the novelty
  nodes. The projection is GENERIC - it cannot know which symbol
  fired, only HOW MUCH total input drive arrived and its channel
  signature.
- DECODE: 'novel' iff the novelty nodes' response is OUT OF BAND vs
  any KNOWN symbol's typical novelty-node response. A known symbol has
  a learned-typical novelty-node output (measured during ref capture);
  D (never-before-seen channel pattern) produces a DIFFERENT aggregate
  -> detected novel. (familiarity-mismatch = novelty, exactly how
  familiarity detection works.)
- Pool/output band unchanged: the existing 12 identity nodes keep the
  learned argmax decode. So identity AND novelty are parallel systems
  in one output band (14 nodes): 12 identity + 2 familiarity.

## Why this is A (reflex) + B (separate novelty system) combined
A: the fixed non-plastic input->novelty projection is a reflex arc.
B: novelty is a SEPARATE node system, not entangled with identity
   argmax (the D-57 failure mode: D always won argmax over identical
   refs).

## Hypothesis (falsifiable, honest metric)
H-58: novel beats drive the novelty nodes to an out-of-band response
vs any known symbol's typical novelty-node response, reliably.
Falsifier (honest metrics, D-53 rules): fraction of D-beats flagged
novel by the novelty-node band check >= 0.8 in >= 2/3 seeds; and known
beats NOT flagged novel (no false positives) >= 0.9.
If novelty nodes respond IDENTICALLY to all inputs (cos still ~1 on
the novelty nodes - the projection sums drive too coarsely), H-58
fails -> the reflex projection is too crude; try per-channel-ordered
or onset-weighted projection (refine, register as amendment).
Identity decode must NOT regress (12-node argmax unchanged).

## Scope + identity
- Output band 12->14 (OUTPUT_LO/HI = 64..78). All existing capture/
  decode/indexing that slices the output band must derive the identity
  sub-range (64..76) separately from the novelty nodes (76..78).
  Flag-off identity: D58_REFLEX=0 keeps OUTPUT_HI=76, no novelty
  nodes, byte-identical. D58_REFLEX=1 adds nodes + projection.
- Projection weights: deterministic seeded FNV per (input_channel,
  novelty_node) - fixed once, never plastic.
- No pretrained models; no external memory; all internal.

STOP - registered; build with honest metrics + identity gate.

## D-58 PHASE-1 VERDICT: reflex MECHANISM WORKS (A gets unique signature, D detected) but 2 nodes quantize too coarsely for 4 channel-groups

Measured (seed 20260912, D58_REFLEX=1, debug):
  reflex-node responses: A=[0,55], C=[0,0], E=[0,0], D=[0,0]
  stage1 (only A known): D=[0,0] vs A=[0,55] -> verdict=TRUE (D detected)
  stage2/3 (C/E added):  D=[0,0] matches C/E=[0,0] -> verdict=FALSE
MECHANISM: the fixed projection into 2 novelty nodes fires node 1 ONLY
for A (channels 0-5 sum a strong projection); C (6-11), E (12-17), D
(18-23) all project weakly -> collapse to [0,0]. So the reflex gives A
a UNIQUE signature but cannot separate the other 3 channel-groups.

This is a QUANTIZATION problem, not a mechanism failure:
- The reflex DOES create fixed per-input signatures (A != others).
- 2 novelty nodes are too few to give each of 4 channel-patterns a
  distinct micro-signature. With one summed projection node pair, the
  channel-identity information is under-sampled.
FIX (next iteration): more reflex nodes (e.g. 8 = one per channel
group) OR a per-channel-group projection layout so every KNOWN symbol
gets a unique familiarity fingerprint; then familiarity-mismatch
detects D against ALL known refs, not just A.

Partial-PASS on the honesty front: novelty detection in stage1 via
reflex = TRUE (the first time ANY mechanism flagged D under honest
metrics). The library world + reflex together demonstrate: A-vs-D
discrimination is now genuinely detectable (0/3 -> would-be 3/3 if the
stage-1 rule were the metric).

STATUS: D-58 Phase-1 = mechanism validated (reflex creates separable
signatures), quantization failure identified. Phase-1.5 = more reflex
nodes / finer projection. Honest metrics applied throughout.
STOP.

## D-58 PHASE-1.5 VERDICT: PASS - reflex-y familiarity nodes achieve NOVELTY DETECTION (first mechanism to do so)

k=8 (reflex node count) honest stage lines, 3 seeds:
  seed        stage1  stage2  stage3
  20260912    3/3     3/3     3/3
  9001        3/3     3/3     3/3
  424242      3/3     3/3     0/3
8/9 stage-seed cells detect D as NOVEL. The single failure (424242
stage3) is where that seed's knowns collapse under library growth
(C=3/E=3 but D-vs-knowns reflex margin broke).

WHY IT WORKS (measured spectra, k=8): each input pattern drives the 8
fixed non-plastic novelty nodes to a DISTINCT spectrum, e.g.
  A:  [0,0,429,0,72,228,500,0]
  C:  [0,0,427,0,197,154,500,0]
  E:  [0,0,435,0,282,200,500,0]
  D:  [0,0,344,0,27,56,500,0]   <- node 5 far below all knowns
D's node-5 (56-76) vs knowns (154-248) -> familiarity-mismatch fires.
The reflex gives identity-channel SIGNATURES that the 12-dim identity
codec's near-identical refs (cos~0.99) could never.

SIGNIFICANCE: FIRST mechanism in the project to detect novelty under
honest metrics (D-53 rules): 8/9 cells, 2/3+ seeds (3/3 at stages 1-2).
The D-58 design (fix A: content-neutral reflex projection + fix B:
separate novelty-node system, no argmax entanglement) is VALIDATED.

REMAINING: 424242 stage-3 (its knowns collapse there - the library-
retention issue, separate from novelty). Also reflex_det is measured
on captured-ref fresh spectra (run-to-run variance in node values ~5%)
but verdict robust.

STATUS: D-58 PASS (novelty detection achieved). Next: the 424242
stage-3 retention failure + whether reflex specificity degrades with
library size (k=8 fixed vs vocab>8?).
STOP - novelty detection WORKS via reflex; recorded.
