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