# Phase III Level-4 probes — adjacent-gap protocol (D-14)

Purpose: separate "the substrate cannot bridge a silent 1500 ms gap via
plasticity" (mechanism limit, 3 falsified routes) from "the substrate
cannot learn a forward X->Y transition at all" (representational limit).
If the E-nogain BASE learns anticipation when the gap is within STDP
reach (~2-3 tau_plus), the ladder's binding constraint is the bridging,
not representation. If it fails even adjacent, the restriction is
representational (consistent with E3b "binding is representational").

Method: PURE CURRICULUM CHANGE on the committed E-nogain platform —
no new mechanism, no substrate change. Reduce the il S1 inter-presentation
gap (off_ms) from 1500 ms (all prior runs) to a value within STDP reach.
Standard STDP (tau_plus 20 ms) then bridges the adjacency naturally.

Arms (frozen; 3 seeds x gap, base E-nogain, il alternation A/C):
  arm 1500 ms: the committed clla-arex-s{seed}-il (control; prior runs)
  arm  100 ms: clla-adj100-s{seed}-il
  arm   40 ms: clla-adj40-s{seed}-il
Everything else identical (patterns A=C=20 Hz, duration 500 ms, S1
present=[A,C] reps=20 interleaved, S3A/S3C probes retained).

Endpoint (frozen), measured by predict.rs:
  PI(group) = cos(gap_after_X, Y_ref) - cos(gap_after_X, X_ref),
  positive = the post-X state anticipates the imminent successor Y.
  For adjacent arms the measurement window is the actual inter-
  presentation gap [t0+dur, t1), not "late 500 ms" (which overlaps the
  neighbouring presentation when the gap < 500 ms).

Falsifier / discriminator:
  - If PI > 0 (>= 2/3 seeds) in a STDP-reachable arm (40 or 100 ms):
    the substrate DOES learn forward transitions; the binding constraint
    is the long-gap bridging -> next Level-4 registration targets a
    persistence/anticipation primitive that preserves the bridge (the
    protectable half from d_ing) AND converts it.
  - If PI ~ 0 in BOTH adjacent arms AND the 1500 ms control: the
    restriction is representational (no added mechanism rescues it);
    temporal prediction is closed off via readout/STDP in this substrate
    and the ladder moves to a different capability.

No post-hoc tuning; arms preserved; protocol immutable.
STOP — protocol frozen (D-14).
## Amendment (D-16): immediate-adjacency training -> LONG-gap transfer

D-15 measured PI inside the short training gap (confounded by X-tail
decay). The actual ladder question is TRANSFER: after adjacent STDP
training (off_ms=0, A ends and C starts next ms, well inside tau_plus),
does a LONE A re-exposure at the 1500 ms gap (S3A probe, unchanged)
evoke C-assembly anticipation? If yes, adjacency training gives
gap-crossing anticipation (a Level-4 result despite direct long-gap
learning failing). If no, the representational limit fully closes
temporal prediction (nothing to transfer).

ARM (frozen): base E-nogain (all d_* false), 3 seeds, S1 off_ms=0
immediate A/C alternation reps=20; S3A/S3C probes unchanged (A/C alone,
off_ms=1500). New exp_id clla-adj0-s{seed}-il (distinct dirs).

ENDPOINT (frozen): during S3A (A alone, 1500 ms spacing), late-gap-after-A
state vs C_ref MINUS vs A_ref (same PI definition as D-15 but restricted
to the S3A re-exposure windows):
  PI_s3a = cos(gap_after_A, C_ref) - cos(gap_after_A, A_ref)
Falsifier / discriminator:
  - PI_s3a > +0.05 in >= 2/3 seeds: adjacency training transferred to
    long-gap anticipation -> Level-4 temporal prediction ACHIEVED via
    adjacency (bridge crossing ok, direct long-gap not needed).
  - PI_s3a ~ 0 (or negative): no forward transition was acquired to
    transfer -> representational limit confirmed; Level-4 closed.
No post-hoc tuning; arms preserved.
