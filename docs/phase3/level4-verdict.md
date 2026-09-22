# Phase III Level-4 verdict (branch 1) — inhibitory gating (d_ing)

Status: FALSIFIED on the association endpoint; protective premise
CONFIRMED. 2026-09-22. Instrument: predict.rs (pool/out PI + gap
density), outspike.rs. Design: docs/phase3/level4-inhib-design.md.

## Result (measured, 3 seeds x il, d_ing on; configs clla-ing-s{seed}-il)

Run dirs (exp_id inherited -> runs/clla-arex-*; config WAS
clla-ing-s{seed}-il with d_ing=true):
  s20260912 -> runs/clla-arex-s20260912-il-20260922T175020Z
  s9001     -> runs/clla-arex-s9001-il-20260922T175029Z
  s424242   -> runs/clla-arex-s424242-il-20260922T175041Z
All completed curriculum-complete (no runaway).

OUTPUT prediction index (frozen PRIMARY: must exceed +0.05 in >=2/3):
  s20260912: -0.1204   (base control +0.0153)
  s9001:     +0.0139   (base +0 ... )   [compare earlier: base -0.0176]
  s424242:   +0.0085   (base -0.0088)
  => 0/3 seeds reach +0.05. POOLED output PI ~ 0. ANTICIPATION NOT LEARNED.

POOL prediction index (secondary):
  s20260912: -0.0046,  s9001: +0.0180,  s424242: +0.0081  (all ~ 0)

## What the mechanism DID and did NOT do (measured)

CONFIRMED (the protective premise of the research question):
- pool late-gap firing PRESERVED: 334.8 / 187.3 / 141.4 spikes/gap
  (non-empty, base order of magnitude 40-239). Output gap firing 23-288
  /gap. Unlike the falsified eligibility family (which ZEROED the gap
  substrate, d_elig -> 0 spikes), d_ing does NOT destroy the bridge:
  the local inhibitory gate keeps the recurrent pool off the phasic-lock
  regime while leaving its fast STDP and persistent tail intact.
  "Can local inhibitory gating preserve the predecessor-specific
  cross-gap state?"  -> YES (substrate survives).

FALSIFIED (the association endpoint):
- output PI ~ 0 across all 3 seeds (only s9001 mildly positive, far
  below the +0.05 bar; s20260912 NEGATIVE). The gated additive readout
  slow-LTP did not turn the preserved predecessor state into
  anticipation of the next event (Y-selective late-gap output firing).
  "while allowing the next-event association to be learned?" -> NO.
  Adding a_ing=0.005 x gate(open in tail) x elg produced no measurable
  re-weighting of the readout toward anticipating the successor.

## Mechanism-truth observations (why; recorded, not tuned)

1. The per-cohort gate works as designed: driven cohort closes
   (ing_gate rises, gate -> ~0.04 at saturating drive), quiet cohort
   re-opens (gate -> ~1 after tau_g 300 ms). Unit-verified separately.
2. Pool fast STDP is untouched (identity gate PASS, FNV unchanged),
   and the pool bridge survives end-to-end — the design's central claim
   about substrate protection is empirically confirmed.
3. The association did not accumulate: pooled output PI stayed at the
   base "memory-not-anticipation" level (the +-0.02/-0.12 scatter is the
   same rate-asymmetry artifact seen in the untrained base, not a learned
   signal). Either the a_ing term is too weak / too rarely active to
   reweight the readout within S1, or the output's gap firing is not
   tractably pulled toward Y by the pool-tail->output path. No tuning.

## Falsification (frozen endpoint)

PRIMARY falsifier "output PI > +0.05 in >= 2/3 seeds" -> NOT met
(0/3). FALSIFIED. Pool preservation (the one positive half) does not
rescue the mechanism: the research question was whether inhibition
preserves the state WHILE the association is learned; the state lives,
the association does not.

## Next (from this negative; no patch / no tuning)

The substrate-protection half is now a CONFIRMED mechanism asset; the
learning half is the persistent failure across ALL three Level-4 routes
so far (d_elig LTP, d_elig_ro readout LTP, d_ing gated readout). The
common cause: in this E-nogain all-excitatory environment, no
spike-timing readout plasticity has turned the (now-preserved)
predecessor state into next-event anticipation — the state is memory,
not prediction. Candidates for the NEXT registration (approval required,
not patching this one):
- (a) measure output-anticipation against the ORGANISM's own next-event
  response coerced from the readout (readout responses as the post may
  be too sparse/weak; use dwell-time-consistent presentation overlap);
- (b) a curriculum where predecessor and successor presentations
  OVERLAP so association does not depend on bridging a silent gap;
- (c) revisit structural-growth-targeted wiring (E4-family, gate-on
  regression history) now that the gap bridge is protectable.

STOP - falsified; protective premise confirmed; evidence preserved.