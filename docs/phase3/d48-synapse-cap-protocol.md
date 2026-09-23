# D-48 PRE-REGISTRATION: is the ~229 wall a synapse-budget artifact or a structural ceiling?

Date: 2026-09-23. Registered BEFORE the cap-raised run. Now deterministic
(BTreeMap fix, 2c44d7b): single-run size claims are reproducible.

## Provenance finding (why this test)
The 20000 synapse cap in evolve.rs is an INCIDENTAL monitor-wiring
constant (added e06bbb3, `rcfg.max_neurons=600; rcfg.max_synapses=20_000`)
- never registered, never calibrated. resources.rs default is 2000.
The size-push 'wall' (bg_13: collapse at 244; bg_7: exhaustion at
267) may be THIS budget biting, not a structural/property ceiling. The
whole 'neurogenesis capped at ~229' narrative rests on an unregistered
constant.

## Test (single clean deterministic run)
`evolve 424242 16 <SYN_CAP>` with SYN_CAP=40000 (2x the incidental
20000). Same seed, same config, only the synapse budget differs.
Instrumented (D-47): died_at + fail kind per org -> distinguishes
resource-exhaustion (cap) from recognition collapse (structural).

## Pre-registered PASS / FAIL criteria
P1 (budget-artifact, PASS for 'wall was my budget'): organism crosses
  ~267 with recognition intact -> max size significantly > 267, best
  fit >= 0.6 sustained >= 2 gens, NO resource-exhaustion failure before
  the (raised) cap. => neurogenesis is NOT capped at ~229; the prior
  'structural wall' verdicts (22b8950, 8710b7d) were an unregistered
  harness budget, CORRECTED.
P2 (structural ceiling, FAIL for budget-theory): raised cap merely MOVES
  the death point (e.g. 40k exhaustion at a larger size) OR recognition
  collapses independently of the cap (dead=None fail=None but fit->0,
  i.e. the D-45 attractor / representational limit). => the wall is
  structural; D-48 produces the clean recognition-wall evidence the
  earlier docs only assumed.

## Inconclusive bound
If the run hits 40k cap AND collapses at exhaustion -> that's P2
(raised cap only moves exhaustion). If it passes 267 clean but the run
times out before 16 gens -> INCONCLUSIVE, re-run longer.

## Both outcomes are valuable (registered)
- P1: growth is budget-limited -> next lever = size-scaled synapse
  budget or synapse-growth regulation (the real E4d angle).
- P2: growth is structurally recognition-limited -> the D-45 attractor
  IS real above 267 -> compartmentalized pools is the target.
Either way this run decides the fork on evidence, not assumption.

## Guard
Flag-off identity: SYN_CAP default 20000 = current behavior, byte-
identical monitor semantics. Cap-raised runs are a NEW measurement,
registered here.
STOP.
