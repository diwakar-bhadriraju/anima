# D-53: the survival-loop 1.00 recognition was a closed-loop SELF-CONFIRMATION artifact

Date: 2026-09-23. Reconciliation of a contradiction that falsifies a
headline claim. This is the most important finding of the E4..D-52 arc.

## The contradiction
- D-46-era survival verdict (docs/phase3/s-survival-verdict.md): known_
  recognized_frac 1.00/0.91/1.00 for s20260912/s9001/s424242 - "known
  patterns recognized ~100%".
- D-52 outprobe (identical formation + decoder, seed-forced BOTH A and C
  as held-out): s20260912/s9001 decode A vs C at CHANCE (0.50, refs
  cos=1.000); only s424242 keeps a fragile magnitude margin (0.988).
Same seeds, same formation (verified: form_s1 and outprobe form both do
20 reps, per-sym beat + 1500ms gap, same STDP/v2), same decode. Both
can't be true of equivalent organisms.

## Root cause (measured): the survival CLOSED LOOP never tests C
survival.rs cur-cycle (world-update):
  let act = io::decode(&out, refs, q_floor=1.0, th_known=0.20);
  cur = if force_novel { "D" } else if act is a known ref { act }
        else { some OTHER known != cur };
mechanism: a TONIC-SEIZED org (the dominant state, see D-52) always
outputs high amp -> decodes to its NEAREST ref = "A" (argmax over
degenerate cos~1.0 refs) -> act=="A" -> cur stays "A". C is NEVER
presented by the closed loop (act is always known=A, so the
withdraw->other-known fallback that would feed C never triggers).
=> known_n counts ONLY A-beats; A always 'recognized' (act==cur==A).
known_recognized_frac = 22/22 = 1.00 while the org CANNOT distinguish
A from C (outprobe: chance, cos 1.0). The 1.00/0.91/1.00 was
SELF-CONFIRMATION, not recognition.

## Rank-ordered evidence
1. outprobe forces A AND C held-out -> honest chance for 2/3 seeds.
2. survival's world only presents what the org 'decodes to' (approach-
   known) -> tonic org starves C.
3. amp-floor 1.0 passed trivially by tonic org (500/12 = 41.7) -> never
   QUIET -> never triggers the other-known fallback.

## Consequences (this falsifies a headline goal claim)
- The 'known_recognized 1.00 / novel 0.00 - perfect known-vs-novel
  separation' (the strength the goal rested on) OVERSTATES: it measured
  'org keeps presenting the one symbol it can name' (self-confirmation),
  not 'org can tell A from C'.
- The real organism (2/3 seeds) CANNOT discriminate 2 trained symbols in
  neural output - the tonic-seizure degeneracy (D-52) is structural.
- outprobe (forced both symbols) is the HONEST recognition metric going
  forward. The survival ready-loop number is a validity hazard.

## Corrected capability statement
'Recognizes known vs novel' is FALSE as a robust claim. Correct:
- 2/3 seeds (20260912/9001) cannot separate A from C (cos 1.0, chance
  decode). 1/3 (424242) keeps a fragile magnitude margin.
- known-vs-novel (A/D) still separates (the ONLY real axis) because D is
  genuinely never-trained; but A-vs-C (the two 'knowns') does not.
- The survival-loop 1.00 is an artifact of closed-loop self-confirmation.

## Action
HOUSE RULE: any recognition claim must measure against BOTH symbols
held-out (outprobe-style), never a closed loop that feeds the org only
what it decodes-to. The closed-loop survival metric is invalid for
discrimination claims until this is fixed.
STOP - headline recognition claim corrected; the tonic-seizure
degeneracy is the real, structural limit.
