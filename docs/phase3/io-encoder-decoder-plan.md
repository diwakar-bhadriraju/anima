# Phase III I/O layer — encoder + decoder codebooks (frozen plan, D-25)

Status: FROZEN PLAN (design only, no implementation yet). 2026-09-22.
Purpose: the human-readable interface to the organism. TWO fixed,
deterministic, pre-registered codebooks — one on the input side (we send
it symbols), one on the output side (it tells us its state). Both are
instrumentation at the boundary (charter: I/O and environment are ours
to define; internal architecture stays untouched). This is implemented
BEFORE the survival loop so the loop reuses the same machinery.

## 0. Principle (honest, in one line)

The brain is spike-based; it does not understand symbols, text, or
language. We (humans) define the encoding of symbols onto its 24 input
channels, and the decoding of its 12 output neurons into symbols we can
read. The brain only learns distinctions; the codebooks are our
language around it. All codebooks are deterministic, fixed once frozen,
identity-gated (flag-off = exact existing path).

## 1. INPUT ENCODER CODEBOOK (we send things IN)

Symbols are encoded as channel patterns (exclusive channel subsets +
rate + duration), the same mechanism the committed configs already use
(deterministic seeded Poisson trains). This formalizes the existing
[[pattern]] machinery into a public "alphabet":

  FROZEN input alphabet (v1, all mutually exclusive channel ranges):
    A      -> channels 0..7    20 Hz / beat
    C      -> channels 8..15   20 Hz / beat
    D(novel)-> channels 16..23  20 Hz / beat
    QUIET  -> no channels (silence beat)
  (reserved for later alphabets: text/audio encoders - DETERMINISTIC
   rules, no pretrained models, per charter E9/E10 direction).

A "LIVE-INPUT path" (new harness capability, identity-gated): instead of
only consuming a pre-generated static schedule, the harness can accept
an input symbol at runtime (index or name: "A" / "C" / "D" / "quiet"),
expand it through the exact same deterministic train generator, and
present it for one beat. Flag-off: schedule-driven path unchanged.

Verification (frozen): (a) same symbol + same seed => byte-identical
trains (determinism); (b) distinct symbols => disjoint channels (no
cross-talk); (c) flag-off run FNV-identical to the committed baseline.

## 2. OUTPUT DECODER CODEBOOK (it tells us its state)

Per 500 ms beat, we decode the 12-neuron output vector v (ids 64..75)
through a PUBLIC fixed rule into a small human-readable symbol set.
References: A_ref, C_ref = mean 12-dim output vector over S1 A/C
training (the measured 100%-separable code). cos = centered cosine.

  codebook (v, one beat):
    amp  = mean |v| ;  rhoA = cos(v, A_ref) ; rhoC = cos(v, C_ref)
    amp < q_floor                       -> QUIET
    rhoA > th_known and rhoA > rhoC     -> "A"   (organism says: I know A)
    rhoC > th_known and rhoC > rhoA     -> "C"   (organism says: I know C)
    max(rhoA,rhoC) <= th_known          -> NOVEL (never learned this)
    otherwise                           -> UNSURE
  plus, for humans/debugging, the RAW 12-bit pattern printed as hex
  (e.g. "A7") every beat - the full observable signal, decoded symbol
  is the human-friendly projection.
  Frozen constants: q_floor, th_known (calibrated once from committed
  S1 measurement, before any use).

Growth policy: the codebook GROWS only when the organism's DISTINCT
internal states grow (new patterns learned -> add their refs/symbols).
It never fabricates symbols the organism cannot reliably produce.

Verification (frozen): (a) held-out decoder accuracy = the measured
100% A-vs-C separation, so "A"/"C"/"NOVEL" decode is honest; (b) NOVEL
(never-seen D) must NOT decode as A or C (>= 2/3 seeds); (c) flag-off
identity.

## 3. IMPLEMENTATION WORK ITEMS (after this freeze is approved)

W1. Input alphabet config + LIVE-INPUT harness mode (flag-gated;
    flag-off static path byte-identical). Unit: determinism, disjointness.
W2. Output decoder (codebook fn: v -> symbol + raw hex) with frozen
    refs/thresholds; unit: 100% A/C on held-out, NOVEL not A/C.
W3. Two-way DEMO harness: "you type a symbol, it senses, it replies"
    printed as decoded symbol + raw hex, live. Reuses all committed
    substrate (E-nogain, no new mechanism).
W4. Identity gate: flag-off FNV 9647ea8a0ca4dbd2 unchanged.
W5. (later, separate) deterministic text encoder (char -> channel
    pattern) and audio encoder (bands) - documented, not this slice.

## 4. SEQUENCING (agreed)

I/O layer (this plan) FIRST -> survival loop (D-22..D-24, reuses the
same decoder codebook machinery and LIVE-INPUT path for its closed-loop
env) -> reproduction/selection/growth (later registrations).

STOP - frozen plan D-25; awaiting approval to implement W1-W4.
## D-26 — ref provenance + NOVEL honesty anchors (frozen)

(1) REF PROVENANCE (W2): A_ref / C_ref are NOT taken from an ad-hoc
training run. They are computed from a FROZEN, COMMITTED source: the
E-nogain S1 training window of the very same runs used as identity
controls (clla-arex-s{seed}-il 1327Z), S1 presentations only, mean
12-dim output vector per pattern, as measured by the committed probes
(outselect/retent convention). If W3 uses a fresh live run, the refs are
captured from ITS OWN S1 formation stage (frozen at S1-end, per D-24) -
never mixed between runs. The decoder doc must record WHICH ref source
was used, so the honesty claim is auditable.

(2) NOVEL HONESTY (v1): the input alphabet's D is pinned as NEVER-
TRAINED in the v1 demo (W3). D is presented only as a NOVEL probe; the
decoder has only {A_ref, C_ref} and D must decode as NOVEL. If a later
slice trains D in the same run (D-21 style), the growth policy (D-25
section 2) makes the decoder GAIN a D_ref (a new registered alphabet
symbol) - the decoder never mis-decodes a trained pattern as NOVEL-by-
definition. W2 test (b) asserts exactly this: never-trained D -> NOVEL
in >= 2/3 seeds, and the D_ref-gain path is a separate registration.
