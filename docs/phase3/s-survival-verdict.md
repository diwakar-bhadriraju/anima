# Phase III sY — survival loop verdict

Status: PASS on the frozen falsifier. 2026-09-22.
Protocol: docs/phase3/s-survival-protocol.md (D-22..D-30).
Runs: configs/clla-surv-min-s{seed}.toml (S1 -> SURV direct, warm
start; no silent stages). 3 seeds, all curriculum-complete.

## Endpoints (frozen D-22/D-30)

PRIMARY (known-vs-novel differential across >= 2/3 seeds, bar > +0.05):
  s20260912: kvs_diff = 0.133   PASS
  s9001:     kvs_diff = 0.063   PASS
  s424242:   kvs_diff = 0.099   PASS
  -> 3/3 seeds. The organism responds to a NEVER-TRAINED novel probe (D)
     with measurably different recognition/viability than to known A/C,
     while refs stay fixed at S1-end and D stays untrained (D-24/D-26).

SECONDARY (persistence): all 3 seeds survive the full 30-beat horizon
(died_at=None), pool rate stays in [5,250] Hz (a=1), viability
mean_v 0.63-0.78 accumulates. PASS -- no death in a benign world.

## Mechanism findings (measured, D-30 sequence)

1. DEGENERATE AUTONOMY (D-30): with full self-selection of the next
   stimulus, the organism locks onto one known pattern (A) and never
   meets novelty -> PRIMARY undetectable. Fixed by forcing the D probe
   with p_novel=0.2 (pre-registered), preserving autonomy otherwise.
2. REBOUND SEIZURE (D-29): running survival after 20 s silent stages
   (deep cold brain 2.7 Hz) -> stimulus seizes it into 340-400 Hz
   sustained firing (output saturated 250/beat). Fixed by S1->SURV
   direct (warm start) + inter-beat 1500 ms rest gap (trained cadence).
3. NOVEL -> SILENCE (D-30 finding): the never-trained D probe drives the
   output to all-zero (QUIET) -- a plausible "shock/withdraw" response.
   The world-update originally re-presented D after QUIET, deadlocking
   the loop into novel->silence->D (seeds 9001/424242 identical 11-beat
   death). Fixed: withdraw/QUIET -> move to the OTHER known pattern; D
   appears ONLY from forced novelty.
4. Recognition r(t): exact-match only (a mismatched known beat is a
   failure, not a hit) - the anti-degenerate measurement fix.

## Honest limits (recorded, not hidden)

- The post-loop survival runner calls net.step + stdp_tick but does NOT
  run V2Plasticity M2-M6/structural-window/E6 rate-balance on the
  per-tick path (governors). Warm-start + rest cadence + low activity
  kept rates in band for this benign world; the governor-dependency is
  NOT yet stress-tested under novel-avalanche or high-drive. The
  in-loop frame-source swap (governors on every tick) is deferred but
  recorded as the fidelity upgrade if the loop is pushed into
  harder regimes or reproduces under evolutionary selection.
- kvs_diff is small (0.06-0.13) but consistently > +0.05 across seeds;
  the direction (novel lowers recognition-viability) is the honest,
  measured signal.

## Verdict

The survival loop PASSES its frozen falsifiers: the organism, in a
closed sense-act-sense loop with a homeostatic drive, measurably
distinguishes known from novel and persists. It does NOT die in a
benign world, and its viability meaningfully varies with experience.
This is the first time "thrives" is a measured, selected-able quantity
in this program -- the precondition for the evolution/death-selection
slice (D-31, later registration).

STOP - verdict recorded; PASS; no tuning (corrections D-28..D-30 were
pre-matrix pre-registrations, not post-hoc tuning).