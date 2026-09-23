# D-54 PRE-REGISTRATION: output-band RATE-DEPENDENT winner-take-most competition (input-selectivity)

Date: 2026-09-23. Deterministic. Registered BEFORE implementation.

## Problem (from D-52/D-53, measured)
The all-excitatory shared pool TONIC-SEIZES the output band: 2/3 seeds
produce near-identical output (cos~1.0) for distinct trained symbols
A/C -> cannot distinguish them (chance decode). D-53: the survival-loop
"100% recognition" was closed-loop self-confirmation (never tested C).
D-46's output-band inhibition was pairwise-SAME-TICK and SYMMETRIC:
a tonic neuron firing every tick suppresses all co-firing outputs
EQUALLY -> no winner emerges -> no selectivity. It rescued only 1/3
seeds (per-seed, net-negative).

## Mechanism (D-54) - rate-dependent competition, winner-take-most
Each OUTPUT neuron accumulates its own firing rate (use the existing
per-neuron rate_hz EMA). At each tick, every output's CURRENT RATE
depresses the RESTING POTENTIAL (v_rest, via a threshold elevation) of
ALL OTHER output neurons, by an amount proportional to that output's
rate:
   for output o (rate r_o): for each other output b: v_rest[b] += comp * r_o
This is ASYMMETRIC and RATE-PROPORTIONAL: the most-driven output raises
its rate -> suppresses rivals more -> consolidates as winner. A
different input drives a different output to win -> INPUT-SELECTIVITY.
Flag: output_competition_gain (0 = identity, byte-identical baseline;
distinct from D-46's same-tick pairwise output_inhibition_gain).

## Hypothesis (falsifiable, honest metric = held-out BOTH symbols)
H-54: rate-dependent competition makes the output band INPUT-SELECTIVE.
Falsifier (outprobe-style, forced A and C held-out):
   PASS: A/C decode acc -> >= 0.9 in >= 2/3 seeds (vs 0.5 chance now),
         refs cos(A-C) < 0.9.
   FAIL: still chance / cos~1.0 in >= 2/3 seeds -> the all-excitatory
         pool cannot be made output-selective by this competition law
         either; platform representationally capped.
The metric is outprobe's forced-both-symbols decode (D-53 house rule),
NOT the survival-loop self-confirmation number. th_known/q_floor frozen.

## Scope
- New cfg field output_competition_gain (default 0).
- Implemented in Network::step (output band only, plasticity-law at I/O
  boundary). No change to pool growth/plasticity/survival logic.
- outprobe extended to take the gain; sweep it and report held-out
  A/C decode acc across the 3 seeds.
- Legacy flag-off byte-identical.

## Both outcomes valuable
PASS: the organism genuinely discriminates A vs C - the first robust
trained-symbol recognition. Capacity/codebook questions reopen on a
non-degenerate output.
FAIL: tonic-seizure is structurally unbeatable by output-side
competition -> the all-excitatory single-pool platform CANNOT represent
distinct trained identities; the honest end-state is trained-vs-novel
only. Moves the goal to a different architecture decision.
STOP.
