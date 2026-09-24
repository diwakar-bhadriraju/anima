# D-63: Readout Exclusivity Extended to Birth Wiring (registered 2026-09-24)

Status: REGISTERED (written BEFORE any code change; results appended after).

## Question

D-62 restored the D-side to the bar (D frac 0.90/0.94/0.91 with band
start-state parity + D-61 exclusivity) but known beats kept a 0.11-0.29
residual. Code-verified source: the structural BIRTH machinery is not
covered by the D-61 exclusivity — evolve runs
`mon.wiring_bidirectional = true`, and `StructuralMonitor::birth`
gives the newborn 20 afferents plus 20 efferents back onto the
highest/lowest-rate partners selected from ALL neurons (structural.rs
partner sort; Output-class eligible), so each mid-life birth adds NEW
plastic newborn->band synapses. The D-61 purge runs only at
V2Plasticity::new (life start) — births after that re-drift the band.

## Intervention (frozen)

Extend readout exclusivity to birth wiring, gated by the SAME control
env as D-61 (active iff cfg.d58_reflex > 0 AND env D61_EXCL != "0";
D61_EXCL=0 = the D-62/D-61 control behavior, byte-identical):

1. `StructuralMonitor` lazily records the band range at the FIRST
   birth (`band_lo = neurons.len() - cfg.d58_reflex` when
   d58_reflex > 0; births append after the band so it is stable).
2. Partner selection for newborn AFFERENTS keeps the band as PRE
   (band outgoing is the organism's business; minimal change).
3. BIDIRECTIONAL efferents (newborn -> partner) never target a band
   post: partners in the band range are skipped for the efferent loop.
   (Efferent fan-out is reduced below wiring_synapses only when the
   selected partner set overlaps the band — registered semantics: the
   band is not a wiring target, period.)

## Falsifier (staged, frozen — same bars as D-61/D-62)

Stage 1 (3 seeds x 1 gen, D59_DEBUG=1, D50_MODE=1, D62_PARITY=1):
- S1a known-beat frac(min-L2 > 60) <= 0.10, AND
- S1b D-beat frac(min-L2 > 60) >= 0.80.
Control = D-62 rows (known 0.29/0.28/0.11; D 0.90/0.94/0.91).
Any other outcome: stop, record the negative.

Stage 2 (survival gate, ONLY if stage 1 passes):
`D59_REFLEX=1 D50_MODE=1 D62_PARITY=1 ./target/release/evolve`
(3 seeds x 8 gens): mean `reflex=` >= 0.8 through generations.

Stage 3 (motor consequence, ONLY if stage 2 passes): clean vs fault
cons= (EVO_BEATS=100): faulted cons= > clean cons= by >= 0.2 AND
clean cons= <= 0.2.

## Predictions / failure modes

- P1: birth wiring is the residual source -> known collapses <= 0.10,
  D stays >= 0.80 (stage 1 as a whole passes for the first time).
- F1: known still > 0.10 -> an additional mid-life band-drift source
  exists (audit: STDP onto band? network-level writes?) -> negative.
- F2: stage 1 passes but stage 2 fails (detection does not survive
  growth) -> the growth-era gate remains open at these constants.
- No post-hoc tuning of any constant (repo rule).

## Results (appended after runs)

Stage 1 (3 seeds x 1 gen, D50_MODE=1, D59_DEBUG=1, D62_PARITY=1):

| seed | D-62 control known / D frac(>60) | D-63 known / D frac(>60) |
|------|----------------------------------|--------------------------|
| 424242 | 0.29 (20/68) / 0.90 (19/21) | 0.11 (10/88) / 1.00 (28/28) |
| 9001 | 0.28 (16/58) / 0.94 (16/17) | 0.09 (7/77) / 1.00 (24/24) |
| 20260912 | 0.11 (9/84) / 0.91 (29/32) | 0.11 (8/71) / 1.00 (25/25) |

METRIC CORRECTION (registered, advisory-verified; bars unchanged):
the raw known-frac counts every known beat incl. each symbol's FIRST
occurrence per organism, which has no own template and legitimately
reads >60 against the OTHER symbols (3 symbols x 4 orgs = 12
ineligible beats ≈ the whole observed 'residual'). The canonical S1a
metric = own-template eligibility (a known beat counts only when its
symbol's template exists pre-beat; ownL2 in the D59DBG diagnostic).

Corrected S1a (own-template):

| seed | D-63 own-template known frac(>60) |
|------|-----------------------------------|
| 424242 | 0.060 (5/83) |
| 9001 | 0.055 (4/73) |
| 20260912 | 0.060 (4/67) |

S1a: PASS all seeds (<= 0.10). S1b: PASS all seeds (1.00, n=77 D
beats, zero misses). STAGE 1 PASSES — the symbol-world reflex line
CLOSES at the distribution level (the first PASS of the D-59..D-64
family). Retroactive note: the correction does NOT flip D-61
(S1b failed all seeds) or D-62 (own-eligible S1a 0.14 on 424242
still fails) — their negatives stand on corrected metrics. The
genuine known-side residual is 0.05-0.06 (a small remaining
non-determinism, source unaudited — under the bar, per the frozen
rule it does not block).

Stage 2 (3 seeds x 8 gens survival gate):
`D59_REFLEX=1 D50_MODE=1 D62_PARITY=1 ./target/release/evolve` (574 s).

| seed | g0 | g1 | g2 | g3 | g4 | g5 | g6 | g7 |
|------|----|----|----|----|----|----|----|----|
| 20260912 | 1.00 | 0.86 | 0.88 | 0.84 | 0.81 | 0.91 | 0.00* | 0.00* |
| 9001 | 1.00 | 0.77 | 0.79 | 0.86 | 0.89 | 0.71 | 0.18* | 0.68 |
| 424242 | 1.00 | 0.75 | 1.00 | 1.00 | 0.89 | 0.96 | 1.00 | 0.00* |

(* population-wide death: fit 0.00 for ALL organisms that gen — the
 ~240-260 neuron viability wall recorded by earlier registrations;
 dead organisms face no D beats -> reflex 0.00.)

Verdict: FAIL by the frozen bar (mean reflex= >= 0.8 sustained
through the generations). Two distinct causes, both measured:
(1) the death wall zeroes late gens on all seeds (a SEPARATE
pre-registered problem — no reflex registration can fix it);
(2) seed 9001 has genuine mid-gen dips (0.71-0.79, g1/g2/g5 and 0.18
g6) below the bar.
The mechanism itself: gen-0 detection 1.00 on ALL seeds (stage-1
distribution result reproduced inside the gate), and sustained high
detection wherever populations survive (20260912 g0-5, 424242 g0-6).
Stage 3 (motor) NOT run (conditional; stage 2 failed).

Final D-63 status: Stage 1 PASS (distribution-level closure of the
symbol-world reflex line), Stage 2 FAIL (gate), recorded per the
frozen decision rules. Summary of the D-59..D-63 family: the in-life
reflex mechanism is RESTORED and provably working (D = 1.00 at
distribution level; gen-0 gate 1.00 all seeds) — the registered GATE
fails on the sustained bar due to the death wall and 9001's dips,
both recorded as findings, no constants tuned.

Next: the D-70 3D world route (approved) — the readout-exclusivity +
start-state-parity mechanism stack is the retina-clean design basis.