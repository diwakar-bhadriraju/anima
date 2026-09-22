# Phase III sX — progressive acquisition demo (D-21, frozen)

Goal (mission correction D-20): evidence the LLM-alternative claim —
learn NEW knowledge incrementally by experience on top of existing
knowledge, with NO re-training, and WITHOUT forgetting what was already
learned. Not prediction; not a new mechanism; curriculum-only on the
committed E-nogain platform.

## What is NOT yet demonstrated (honest gap)
Novel-group generalization and S1 retention exist as partial evidence
(Phase II records), but a clean registered run — form A/C, THEN introduce
a genuinely new pattern D, show (a) D acquired online, (b) A/C retrieval
intact after D — has never been run.

## Curriculum (frozen; configs clla-prog-s{seed}-il.toml unless
## contraindicated by identity — see Execution note)

- base: identical to clla-arex-s{seed}-il (E-nogain, d_core/d_claim) for
  patterns A (ch 0-7) and C (ch 8-15), S1 20 reps interleaved off 1500.
- NEW pattern D: channels 16-23, 20 Hz, 500 ms, no phase variants
  (channels 16-23 unused by A/C in the base config).
- S2 (new, after S1): present=[D], reps=30, order=blocked, off_ms=1500.
  Pure online acquisition of never-seen-exposed pattern; no A/C exposure
  during S2 (so any A/C retention measured at S3 cannot come from
  re-training during S2).
- S3A/S3C probes: UNCHANGED from base (A alone, C alone, off 1500) =
  the standard retention convention.
- S3D (new probe): D alone, 5 reps, off 1500, after S3C.

## Endpoints (frozen)

1. PROGRESSIVE ACQUISITION: D is formed. D S1..S2 mean_hz rises; D
   selectivity (S2 late) present; S3D re-expression > S2-early baseline.
2. NO CATASTROPHIC FORGETTING: A/C retention at S3A/S3C with D trained
   in between is NOT degraded vs the committed no-D control runs
   (clla-arex-s{seed}-il 1327Z): per-pattern S3A/S3C mean_hz and
   selectivity within run-to-run variance (S3 A/C mean_hz delta < 40%
   vs control; retain selectivity sign).
3. ONLINE / NO RETRAIN: by construction (single continuous run, STDP
   live throughout; no reset, no external weights).

Weighing the S3-re-exposure convention: S3A/S3C themselves re-expose the
pattern 5 times; the measure is re-expression of the S1-formed assembly,
the same convention every prior run used — comparisons are against the
1327Z controls, so forgetting is quantified by control-relative change.

## Falsifiers / outcome classes

- PASS: D formed (endpoint 1) AND A/C retention within control variance
  (endpoint 2). => progressive acquisition demonstrated; this is the
  LLM-alternative core claim, recorded.
- PARTIAL: D formed but A/C degraded > 40% => progressive acquisition
  happens at a memory cost; quantify and record the tradeoff (no tuning).
- FAIL: D not formed => the fixed-pool substrate cannot acquire new
  patterns after formation (would bound the whole mission); record.

No post-hoc tuning; 3 seeds; runs preserved. STOP - frozen D-21.