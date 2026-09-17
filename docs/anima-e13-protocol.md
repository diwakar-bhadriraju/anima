# ANIMA E13 — Reversal dynamics / hysteresis probe

Status: **FROZEN** (2026-09-17). No edits after this point except
registered amendments (A-series, user-approved, logged in the appendix).

## 1. Question

E12 (Outcome C) showed that 60 REV-B presentations transfer B's
association from the C-side to the A-side, but only the pre/post
states were measured. E13 asks: **how does the learned representation
reorganize during the contradictory REV experience?** — a
measurement-resolution extension of E12, not a mechanism experiment.

## 2. Discipline (unchanged)

No organism or mechanism changes of any kind. No parameter changes, no
new plasticity rules, no normalization, no inhibition, no phase
duration, firing-rate, repetition-count, or schedule changes. No new
simulation runs: E13 analyzes the committed E12 run artifacts
(`0b049f2` execution) with a new read-only instrument. A/C streams,
B phases, M3 windows, E6 balancing, RNG, analyzer, telemetry: as E12.

**No post-hoc checkpoint selection, no new endpoint thresholds, no
mechanism claims.** The only registered thresholds are the frozen E12
quantities (B-independence 0.60 both raw and L1; alignment = argmin).

## 3. Checkpoints (predetermined, seed-independent grid)

REV round k (1..60) = global round 60+k, start S(k) = 5000 + 6000(59+k).
Checkpoint Tk = end of REV round k = 5000 + 6000(60+k):

| checkpoint | sim-ms | meaning |
|---|---|---|
| T0 | — | after 0 REV: window = rounds 51-60 (last 10 SEQ) |
| T10 | 425 000 | after 10 REV-B |
| T20 | 485 000 | after 20 REV-B |
| T30 | 545 000 | after 30 REV-B |
| T40 | 605 000 | after 40 REV-B |
| T50 | 665 000 | after 50 REV-B |
| T60 | 725 000 | after 60 REV-B (= registered snapshot point) |

Window Wk = the 10 S1 rounds ending at REV round k: for k = 10..60,
Wk = [5000 + 6000(50+k), 5000 + 6000(60+k)) — 30 presentations
(10 A + 10 B + 10 C), all-REV for B. T0 window = [305000, 365000)
(rounds 51-60, all-SEQ, like-for-like local measurement). E12's
whole-block T0/T1 values remain citation anchors only, not verdict
inputs.

## 4. Instrument (observation-only proof)

`crates/anima-exp/examples/e13_trajectory.rs`:

- imports `anima_telemetry` (and std) **only**; zero `anima_core`
  imports — checked statically at freeze (`grep anima_core` on the
  instrument = no hits) — it cannot construct an Environment or run
  simulation;
- opens the three E12 run dirs read-only;
- per presentation in Wk builds internal spike-count vectors from
  telemetry Spike rows (neuron id = row.n, filtered to n >= N_IN);
  per checkpoint reports, with the frozen E12 formulas:
  A-B / B-C / A-C pairwise-mean cosines (raw and L1-normalized);
  B-independence boolean (all of raw A-B, raw B-C, L1 A-B, L1 B-C
  < 0.60); B-alignment argmin(A-B, B-C); windowed selectivity
  (analyzer's per-neuron (best-2nd)/best, median over Wk);
  candidate-permanence events in Wk; in-window Failure events;
  internal-rate stats (mean/max over snapshots in Wk);
- additionally reports the per-seed in-round position of the first
  REV-B (round 61) and of the last REV-B (round 120) from the
  telemetry (E12 record requirement).
- deterministic: pure function of the pinned artifacts (verified by
  re-run equality on a test run).

## 5. Frozen inputs

E12 run dirs (analysis only; **never rerun**):

| seed | run dir | telemetry sha256 (aggregate) | snapshots sha256 |
|---|---|---|---|
| 20260912 | `runs/e12-20260916T202445Z` | `695bd87ae6ee7477` | `005acea9a7f671a4` |
| 9001 | `runs/e12-seed9001-20260916T202822Z` | `0e248586f59cfaaf` | `8ed50dd7ef3a4f90` |
| 424242 | `runs/e12-seed424242-20260916T202822Z` | `65d47f5fbfd87f4e` | `9b83082187f08532` |

(aggregate = sha256 over the sorted per-file hashes; full per-file
hashes recorded in the appendix at freeze time.)

## 6. Classification (Strict frozen machinery; user-approved)

Per seed, from the 7-checkpoint series of (ab, bc, align, indep):

- deltas: d_ab[k] = ab[k] - ab[k-1] for k = 10..60 (6 intervals);
  F_k = |d_ab[k]| / sum_j |d_ab[j]| — majority-movement fraction
  (only intervals that move toward A-side count: sign(d_ab) must be
  negative; the same applied to bc moved toward C, i.e. d_bc > 0;
  both must agree per interval for the interval to be counted —
  directionality is part of "movement").
- **A — continuous reorganization**: alignment flips C -> A by T60,
  direction consistent at every interior checkpoint (ab strictly
  decreasing, bc strictly increasing, all 6 intervals), and
  F_k <= 0.5 for all k (no interval carries the majority of the
  movement).
- **B — threshold-like**: alignment holds C through >= 2 consecutive
  checkpoints after T0 (i.e., no flip at T10 and T20... actually at
  least the T10/T20 checkpoints), then flips, and exactly one
  interval carries F_k > 0.5 (the change completes within a short
  subsequent interval).
- **C — temporary coexistence**: exists a checkpoint k with
  indep[k] = true whose neighbors (k-1 and k+1, where they exist)
  are false — an intermediate state detected by the frozen 0.60 rule
  alone.
- **D — seed-dependent dynamics**: per-seed shape class (A/B/C)
  differs materially across seeds (no unanimous class; if one seed
  differs, the outcome is D with the minority shape reported).
- **E — non-reproduction**: alignment fails to flip C -> A in >= 2
  seeds by T60.
- **F — gates**: any P2 failure (in-window failures > 0), telemetry
  hash mismatch vs the pinned values, determinism failure, or
  structural-engagement failure (zero candidate-permanence events in
  every checkpoint window).

Precedence: **F > E > D > shape (A/B/C)**. If no classification
applies cleanly the outcome is reported as D with the raw tables.

## 7. Verdict discipline

Outcome reported with the full per-seed tables (all checkpoints, all
quantities — never just the final state). No mechanism claims. No
claims beyond the tested quantities (60 REV presentations, three
seeds, one frozen organism).

## 8. Deliverables

1. This protocol (frozen commit).
2. `e13_trajectory.rs` + tests (grid math, window composition,
   B-once-per-round, positions, instrument determinism).
3. Full test suite green (124 + new), 0 warnings.
4. Analysis of the three E12 run dirs; per-seed class; verdict.
5. Execution record appended here; registry updated; commit;
   report hash/tests/warnings/tree status. No E14 proposal until E13
   analyzed and committed.

## Appendix: amendments

- **A-1** (none yet).

## Appendix: per-file telemetry hashes (recorded at freeze)

(recorded on the next line block; see execution record for the
aggregate table above — full sha256 list appended at freeze commit)| seed | file | sha256 |
|---|---|---|
| 20260912 | runs/e12-20260916T202445Z/telemetry/chunk-00000.parquet | 4383123862cedbd273966881a68214204663d15338c84a4fde27646390dfdfa3 |
| 20260912 | runs/e12-20260916T202445Z/telemetry/chunk-00001.parquet | 646f58bf6ea06da05728594ad923a2cb39e735628747671cbe61f54ff81461bf |
| 20260912 | runs/e12-20260916T202445Z/telemetry/chunk-00002.parquet | c79e0a756ba334b8164916032e122e970e91e269e8baf3dccded5e52633ba876 |
| 20260912 | runs/e12-20260916T202445Z/telemetry/chunk-00003.parquet | fbaa6a8cf29582525d390c0915b80c92307e2fe6c289b5fb761c8453d6f5eaeb |
| 20260912 | runs/e12-20260916T202445Z/telemetry/chunk-00004.parquet | 68a6289990abcb509cdcdca69fa3f3d533fae58184206f6e81e0fe4fa9266db3 |
| 20260912 | runs/e12-20260916T202445Z/telemetry/chunk-00005.parquet | 26dadb862538665322033128150165ef6f2ae9fb61ef486ef6d3d234c4e51c8e |
| 20260912 | runs/e12-20260916T202445Z/telemetry/chunk-00006.parquet | 011df2dcbc87711e8a87d09c6933aa7a18eb3f6246487c2a6a1dddefc0fc380d |
| 20260912 | runs/e12-20260916T202445Z/telemetry/chunk-00007.parquet | 11c95e44036f7f601ab958d16c07bea785133cda6aab8340cf38a4f5e915c2a8 |
| 20260912 | runs/e12-20260916T202445Z/telemetry/chunk-00008.parquet | d014b854656b7656829f8524bb63783668a53b4a1450021b969f189884ac972a |
| 20260912 | runs/e12-20260916T202445Z/telemetry/chunk-00009.parquet | b0e1d815933ed28420317189955c5d195dbcf784c7b21752762633131606c922 |
| 20260912 | runs/e12-20260916T202445Z/telemetry/chunk-00010.parquet | 3cb82015c05b3239137f17d8439dfe4ee0ab89742892326f347a35333ab5551f |
| 20260912 | runs/e12-20260916T202445Z/telemetry/chunk-00011.parquet | 02f3c475db39e4374e611c7cb2c3b9bd59b06322bbd2528dad374cdbbafa7a31 |
| 20260912 | runs/e12-20260916T202445Z/telemetry/chunk-00012.parquet | eade53015865133044b7d3a396b9b2650218fd98ec498404ec0e5cf3a91e1b93 |
| 20260912 | runs/e12-20260916T202445Z/telemetry/chunk-00013.parquet | 7f111ac6c563aee7471666b3d1ecf106781c5f1945324b9e2cc561c8d45ab056 |
| 20260912 | runs/e12-20260916T202445Z/telemetry/index.json | 82550a37598a47153dbf86e56f9e35155ea9fd67f16b6c7804c0ba1855e1c140 |
| seed9001 | runs/e12-seed9001-20260916T202822Z/telemetry/chunk-00000.parquet | 0d31ef98726688acb8e9d5482ba9b994aaaaea60926e987d3c2b62185994cd83 |
| seed9001 | runs/e12-seed9001-20260916T202822Z/telemetry/chunk-00001.parquet | e1c8ec0d5ec5b9b3b2552eb1ecd476e5074e532ab90510c3949bbfbf3f9781c7 |
| seed9001 | runs/e12-seed9001-20260916T202822Z/telemetry/chunk-00002.parquet | 052ed77a35cf2ada4f3802595bf320eca4cfe756b1e99164262c0f9f59f88c14 |
| seed9001 | runs/e12-seed9001-20260916T202822Z/telemetry/chunk-00003.parquet | 3f38b037d1063d702ba439209ae164ac1fa511c3392762ae10adb71ee0dc4109 |
| seed9001 | runs/e12-seed9001-20260916T202822Z/telemetry/chunk-00004.parquet | 11f03d6ce6e786ead30ad9ebaa654d37d872360c9eb8a891861a798a68cf4518 |
| seed9001 | runs/e12-seed9001-20260916T202822Z/telemetry/chunk-00005.parquet | 53cb282d40d98d5da522a1fd578d09faa4869ac040422a06bedab6437199853c |
| seed9001 | runs/e12-seed9001-20260916T202822Z/telemetry/chunk-00006.parquet | 9d9881ab4bfb4550380ce5aabb18edf58755b5b220c28da71e1c10d734f17968 |
| seed9001 | runs/e12-seed9001-20260916T202822Z/telemetry/chunk-00007.parquet | 60eb4e8586da2b7e0f7d9ee0e04a66733b5c101d4247bd3bc657817f6e72ec8b |
| seed9001 | runs/e12-seed9001-20260916T202822Z/telemetry/chunk-00008.parquet | 0a19520621da92c5d1e115e568481a3ea9cb54ba513d30ec4d7523bbfb2cfe89 |
| seed9001 | runs/e12-seed9001-20260916T202822Z/telemetry/chunk-00009.parquet | eeed11931ac54d7e96271e2076869c8e47d4aebbb22ab2e359b3c5b39fadae0e |
| seed9001 | runs/e12-seed9001-20260916T202822Z/telemetry/chunk-00010.parquet | fba4545691760ab116628a77783cba05a6738fb87a5781c4c77161580219e8cc |
| seed9001 | runs/e12-seed9001-20260916T202822Z/telemetry/chunk-00011.parquet | bb46f0fa8d2dd9783b9f4cad0cb3323bef1f0f0ed2f2afb6b7532a2fce6f1b33 |
| seed9001 | runs/e12-seed9001-20260916T202822Z/telemetry/chunk-00012.parquet | 12852e518e0da0f811b4b435fe5365ebf3a6a97cce9a329e322a94a1c8a271f5 |
| seed9001 | runs/e12-seed9001-20260916T202822Z/telemetry/chunk-00013.parquet | 920d7e3dc57c89b0b014e906ac56a230302f9238ebaccbfe11536667542bf94a |
| seed9001 | runs/e12-seed9001-20260916T202822Z/telemetry/index.json | 17c81b6186f75c751835e468788db45f77fd6e8bfdc112ff59edf1ef030def4e |
| seed424242 | runs/e12-seed424242-20260916T202822Z/telemetry/chunk-00000.parquet | 709bcdb3b2fc73fe93066171e0e928f430309950e47fb4e37f0dde35655f76f0 |
| seed424242 | runs/e12-seed424242-20260916T202822Z/telemetry/chunk-00001.parquet | 49997faea9df4d7c2f84d59ebd2fb9f49057251458947eab0b21636c9e886222 |
| seed424242 | runs/e12-seed424242-20260916T202822Z/telemetry/chunk-00002.parquet | 4a4fb9ef4e6618740f7d41e2175158c102c54e89adf6b25a7cb2cc70d33aaa3c |
| seed424242 | runs/e12-seed424242-20260916T202822Z/telemetry/chunk-00003.parquet | 7eadc52e0da34b5151de4f22051a4e6c43758c40f6661e91191cec3d5d7f8a4c |
| seed424242 | runs/e12-seed424242-20260916T202822Z/telemetry/chunk-00004.parquet | dae03da197f33e2d9919ebe5a754f0101f80b27890d88b1fb019c44fabfd7029 |
| seed424242 | runs/e12-seed424242-20260916T202822Z/telemetry/chunk-00005.parquet | 71c642aa465809ff90171c5ecb8b252e0c2db611d4e17eb47b9c4ff733c9c362 |
| seed424242 | runs/e12-seed424242-20260916T202822Z/telemetry/chunk-00006.parquet | 4c2fc24785f3f2a8cc5d85aa10533e0507c3f6259a9d894962c30da08583aad0 |
| seed424242 | runs/e12-seed424242-20260916T202822Z/telemetry/chunk-00007.parquet | 7b735344f10d26041f4c069f409b6c2205596abf6218273c99fa2c61e21dfab7 |
| seed424242 | runs/e12-seed424242-20260916T202822Z/telemetry/chunk-00008.parquet | 0ca91d41e9987177ecb7829fe1aca20c1de12fa82a6ac78c859b406ce868f1b0 |
| seed424242 | runs/e12-seed424242-20260916T202822Z/telemetry/chunk-00009.parquet | a3464d0cff689445bdefd0559750f71d385a22ca7387ceab3909b0298f6430e4 |
| seed424242 | runs/e12-seed424242-20260916T202822Z/telemetry/chunk-00010.parquet | a507fcea6d9852fd4d5885a0cf592dbca77aaaf5c9b7e201e5053c1f23928ac2 |
| seed424242 | runs/e12-seed424242-20260916T202822Z/telemetry/chunk-00011.parquet | d87c55281f2bec14a1de32427ddc523afc5e3a21eef32ed285a42ea5e1c7d863 |
| seed424242 | runs/e12-seed424242-20260916T202822Z/telemetry/chunk-00012.parquet | ffbe8d5b55929a59e991d2866c3a35f41b1e3aba5a9c7ba310e2e49fdd85bd30 |
| seed424242 | runs/e12-seed424242-20260916T202822Z/telemetry/chunk-00013.parquet | 5524daf8f03271df9b771ca4efab20b90d7b5d4b17e854cb60b3e966c50d3457 |
| seed424242 | runs/e12-seed424242-20260916T202822Z/telemetry/index.json | 9e1a2a725c4b425d0995623385d3e5dad1647bc85a53ef494899af7590c1fd84 |

---

## E13 execution record (2026-09-17)

Instrument `431fdcc` (e13_trajectory, telemetry-only, deterministic);
127 tests green, 0 warnings. Analysis-only over the pinned E12 run
dirs (telemetry + snapshot hashes re-verified byte-identical to the
freeze table; failures 0 everywhere; permanence per window 421-692).

### Full trajectory per seed (Wk = 10 rounds; L1 == raw everywhere;
### selectivity = windowed median; perm = candidate-permanence events)

seed 20260912 (REV-B positions round 61:0, round 120:1):

| cp | A-B | B-C | A-C | indep | align | sel | perm |
|---|---|---|---|---|---|---|---|
| T0 | 0.680 | 0.075 | 0.000 | F | C | 0.590 | 669 |
| T10 | 0.242 | 0.918 | 0.000 | F | A | 0.869 | 585 |
| T20 | 0.218 | 0.914 | 0.000 | F | A | 0.840 | 499 |
| T30 | 0.263 | 0.885 | 0.000 | F | A | 0.863 | 564 |
| T40 | 0.259 | 0.819 | 0.000 | F | A | 0.852 | 526 |
| T50 | 0.316 | 0.773 | 0.000 | F | A | 0.825 | 571 |
| T60 | 0.316 | 0.721 | 0.001 | F | A | 0.852 | 586 |

seed 9001 (positions round 61:2, round 120:2):

| cp | A-B | B-C | A-C | indep | align | sel | perm |
|---|---|---|---|---|---|---|---|
| T0 | 0.760 | 0.097 | 0.000 | F | C | 0.831 | 692 |
| T10 | 0.211 | 0.637 | 0.000 | F | A | 0.836 | 601 |
| T20 | 0.189 | 0.707 | 0.000 | F | A | 0.857 | 507 |
| T30 | 0.232 | 0.648 | 0.000 | F | A | 0.855 | 528 |
| T40 | 0.225 | 0.625 | 0.000 | F | A | 0.866 | 525 |
| T50 | 0.219 | 0.653 | 0.000 | F | A | 0.851 | 640 |
| T60 | 0.217 | 0.622 | 0.001 | F | A | 0.830 | 455 |

seed 424242 (positions round 61:2, round 120:2):

| cp | A-B | B-C | A-C | indep | align | sel | perm |
|---|---|---|---|---|---|---|---|
| T0 | 0.554 | 0.065 | 0.000 | **T** | C | 0.962 | 560 |
| T10 | 0.148 | 0.712 | 0.000 | F | A | 0.907 | 488 |
| T20 | 0.177 | 0.622 | 0.000 | F | A | 0.897 | 441 |
| T30 | 0.168 | 0.532 | 0.000 | **T** | A | 0.961 | 415 |
| T40 | 0.183 | 0.613 | 0.002 | F | A | 0.936 | 498 |
| T50 | 0.199 | 0.544 | 0.000 | **T** | A | 0.934 | 474 |
| T60 | 0.188 | 0.595 | 0.000 | **T** | A | 0.918 | 421 |

Rates: no failures in any window in any seed; internal mean 7.4-10.8
Hz, max 73-102 Hz (all below the 250 Hz gate).

### Frozen classification (letter rule)

Per-seed classes: 20260912 = **none**; 9001 = **none**; 424242 = **C**
(T30 independence with non-independent neighbors T20/T40; T50 fails
the strict rule because T60 is also independent; T60 nominal
independence 0.595 < 0.60 is reported, is not part of the C
signature). Not unanimous => **OUTCOME D — seed-dependent dynamics**
(frozen rule), minority shape C. E absent (alignment flipped in all
three seeds by T60). F absent (gates clean: hashes pinned,
determinism verified, zero failures, engagement non-zero at every
checkpoint).

### Registered facts (no mechanism claims)

1. **The C->A transfer completes within the first 10 REV
   presentations in every seed.** Interval F1 carries 77.1% (seed
   20260912), 87.3% (9001), 83.6% (424242) of the total A-B movement;
   every seed's first post-T0 checkpoint already shows the final
   alignment. The registered grid therefore bounds the transfer
   interval at <= 10 presentations (60,000 sim-ms) — a measured upper
   bound, not an instantaneous-transfer claim.
2. After the flip the A-side association persists with mild drift:
   A-B stays 0.19-0.32 in all seeds; B-C softens monotonically in
   seed 20260912 (0.918 -> 0.721) and drifts 0.62-0.71 in 9001;
   424242 alone passes through an intermediate coexistence region
   (T30/T50) and ends nominally independent.
3. Selectivity jumps at the flip (20260912: 0.590 -> 0.869) and
   stays high — the reorganized representation is sharper, matching
   E12's T0->T1 selectivity rise.
4. T0 local-window anchors are consistent with the E12 whole-block
   values for A-B (0.680 vs 0.682; 0.760 vs 0.655; 0.554 vs 0.562);
   B-C local windows are more side-absorbed than the block means
   (0.075/0.097/0.065 vs 0.228/0.339/0.193) — the last-10-round state
   is the most absorbed, consistent with progressive absorption
   through the SEQ block (direction: toward C).
5. No new thresholds or endpoints were introduced; verdict uses only
   the frozen 0.60 independence rule, argmin alignment, and the
   registered Fk majority-movement rule.
