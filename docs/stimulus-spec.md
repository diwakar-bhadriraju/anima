# ANIMA Stimulus Specification — E1 / E3 / E3b / E4 (and any future arm
# derived from `configs/e1.toml`)

**Traced from**: `crates/anima-exp/src/env.rs`, `crates/anima-exp/src/harness.rs`,
`configs/e1.toml` (also `configs/e2-armA.toml`, `configs/e2-armB.toml`,
`configs/e3-armB.toml`, `configs/e3b-inh.toml`, `configs/e3-armB.toml`
family — all use the same patterns and stages as e1.toml).

**No simulation behavior changed.** This document records what the
network already receives.

---

## 1. Architecture

- **24 input neurons** (D8: pure spike sources — they spike iff and
  exactly when the environment's `InputFrame` says so; no membrane
  state, no drift, no plasticity on the input neuron itself; the
  input→internal synapses are plastic).
- **Channel-to-group assignment** (`env.rs:62–67` and `harness.rs:114`,
  identical mapping in both files):
  `group_of(ch) = letters[ch / group_size]` with `group_size = 8` and
  `n_input_channels = 24`. Concretely:

  | Channel IDs | Group | Count |
  |---|---|---|
  | 0 – 7  | A | 8 |
  | 8 – 15 | B | 8 |
  | 16 – 23 | C | 8 |

- **Input → internal wiring**: 3.8 % random sparse (seeded). Each
  input neuron drives a small, distinct subset of internal neurons.

## 2. Stimulus table

`configs/e1.toml` defines four patterns; channels are matched by
group label (`env.rs::channels_for`), and spike trains are
per-(pattern × rep × channel) deterministic streams.

| Pattern | Active input neurons | Channels (group) | Spike pattern | Duration | Repetitions | Phase(s) | Purpose |
|---|---|---|---|---|---|---|---|
| **A** | ch 0–7 | "A" (8 ch) | independent Poisson @ 20 Hz per ch, ±2 ms jitter | 500 ms | 120 (S1) + 15 (S3) | S1, S3 | baseline pattern 1 (training + retention) |
| **B** | ch 8–15 | "B" (8 ch) | independent Poisson @ 20 Hz per ch, ±2 ms jitter | 500 ms | 120 (S1) + 15 (S3) | S1, S3 | baseline pattern 2 (training + retention) |
| **C** | ch 16–23 | "C" (8 ch) | independent Poisson @ 20 Hz per ch, ±2 ms jitter | 500 ms | 120 (S1) + 15 (S3) | S1, S3 | baseline pattern 3 (training + retention) |
| **D** | ch 0–7 **and** ch 16–23 | "A" ∪ "C" (16 ch) | independent Poisson @ 20 Hz per ch, ±2 ms jitter on all 16 | 500 ms | 30 | S2 | **novelty** — A+C co-activation; never presented in S1 |

**Spike statistics per presentation** (Poisson, λ = 0.020 / ms,
500 ms duration):

- A, B, C: 8 channels × 20 Hz × 0.5 s = **80 spikes ± √80 ≈ 9** per
  presentation.
- D: 16 channels × 20 Hz × 0.5 s = **160 spikes ± √160 ≈ 13**.

Jitter is applied per spike independently; in total a presentation's
spike times are integer milliseconds, each within `[0, duration_ms − 1]`.

## 3. Curriculum (env.rs:70–103 — exact expansion)

| Stage | Patterns | Order | Off between pres. | Reps | Presentations | Sim-time span | What changes |
|---|---|---|---|---|---|---|---|
| **S0** | none (silence) | — | — | 0 | 1 silence block | **5,000 ms** (5 s) | nothing; the network runs empty |
| **S1** | A, B, C | **interleaved** (seeded Fisher-Yates per round, 120 rounds of 3) | **1,500 ms** | 120 each → 360 | **360** | 360 × (500 + 1500) = **720,000 ms** (12 min) | training; weights mutate; assemblies form |
| **S2** | D | **blocked** (A∪C co-activation, all 30 in a row) | 1,500 ms | 30 | 30 | 30 × 2000 = **60,000 ms** (1 min) | **novelty probe** — D has not been seen; novel response measured |
| **S3** | A, B, C | interleaved (separate salt) | 1,500 ms | 15 each → 45 | 45 | 45 × 2000 = **90,000 ms** (1.5 min) | **retention test** — same patterns as S1, no plasticity changes intended |
| **Total curriculum** | | | | | 436 presentations | **875,000 ms** ≈ 14.6 min sim-time | |

**Interleaved randomness**: `presentation_order(seed, stage, salt)` runs
Fisher-Yates over `[0, n_pat)` once per round, extended round-by-round
(`env.rs:218–227`). Salt = `schedule.len()` at the time of expansion,
so S1 and S3 use different orderings even with the same seed.

**Blocked**: in `presentation_order`, `order == "blocked"` short-circuits
to `[0, 0, …, 1, 1, …, n_pat-1, …]` (each pattern repeated fully before
the next — S2 is 30 D presentations back-to-back).

## 4. Seed determinism

Per-spike stream: `derive_seed(master_seed, [hash(pattern_id), rep,
channel_id, schedule_index])` feeds an independent `Xoshiro256PlusPlus`
RNG (`env.rs:115–120`). The same `(master_seed, config)` pair therefore
produces the **same exact spike train for the same rep**. Across runs
with the same seed the environment is bit-identical (test:
`trains_deliver_deterministic_poisson`).

## 5. What the network receives vs analysis labels

| Item | Network input? | Source |
|---|---|---|
| Spike times per channel | **YES** — emitted by `InputFrame.spikes: Vec<InputChannelId>` each tick | `env.rs::Environment::step` |
| `pattern_id`, `stage` strings | **NO** — they only label `StimulusPresented` telemetry events | `events.rs::Payload::StimulusPresented` |
| `RunStarted.params` (n_input_channels, etc.) | **NO** — runs once at start, observation-only | telemetry |
| Silences (S0) | **NO** — empty frame | `env.rs::Schedule::pattern == "silence"` ⇒ empty `trains` |
| Group letters ("A" / "B" / "C" / "D") | **NO** — they are config-side labels for `channels_for` matching; the network sees only which channel fired | `env.rs:62–67` |

## 6. Noise / overlap / randomization summary

- **Per-spike jitter**: ±2 ms uniform (`jitter_ms = 2.0`). Applied per
  spike. Jitter is **independent** across channels and reps.
- **Poisson independence**: per channel per rep — the 8 channels of A
  in rep 0 are 8 independent streams. No temporal alignment of spikes
  across channels.
- **Inter-trial overlap**: none — between presentations there is
  1500 ms of silence (only refractory decay + adaptation decay). No
  presentation starts while the previous presentation is still active.
- **Between-pattern randomization**: presentation order is shuffled per
  round (seeded). Trials within a stage have a different order each
  round, but every round contains all three patterns exactly once.
- **Across-stage**: S3 has the same patterns as S1 (same per-(rep,ch)
  seed formula), but the salt differs, so presentation *ordering*
  differs from S1 even at the same seed.
- **Baseline vs novelty**:
  - **Baseline** = A, B, C (presented in S1 and S3; the network
    treats these as "known" by the time S2 fires).
  - **Novelty** = D = A ∪ C co-activation presented only in S2.
  - There is **no separate baseline hold-out**: D is novel because it
    is never presented during S1, not because it has unique statistics.

## 7. What changes between training (S1) → test (S2) → retention (S3)

- **Inputs**: pattern set shrinks S1↔S2 (A,B,C → D) then back (A,B,C).
- **Plasticity**: STDP / structural rules fire on every tick regardless
  of stage (the rule is config-selected, not stage-gated). The S3
  retention metric is therefore not a perfect no-plasticity replay —
  S3 presentation continues to mutate weights. The retention read is
  "after seeing D in S2, do A/B/C responses still recover?" rather
  than "freeze-weights retrieval".
- **Resource / adaptation**: adaptation current, runaway detector, and
  resource caps are always on; E3 / E3b configs additionally keep them
  active across all stages.
- **Telemetry**: every StimulusPresented carries the pattern_id and
  stage label. The timeline bin (timeline.rs) records one pattern per
  bin start, which is what the viz's stage pill reads.

## 8. What is NOT in the stimulus

- No images, audio, text, or external media of any kind (D9).
- No task labels, no reward signal, no "correct response" cue.
- No read-out target; "output neurons firing" is observed but not
  shaped (E6+ is where reward modulation enters).
- No noise injected into the network beyond spike jitter and Poisson
  randomness. The dynamics themselves are deterministic given inputs.
- No learning gates (U4 / E5), per the user's exclusion for the E4
  experiment.

---

## Verification commands (no run needed)

```bash
# The channel-group mapping is in env.rs:62-67:
sed -n '62,67p' crates/anima-exp/src/env.rs

# Pattern definitions and durations:
grep -A4 '^\[\[pattern\]\]\|^\[\[stage\]\]' configs/e1.toml

# Total curriculum duration:
nice -n 10 ./target/release/anima-run run --config configs/e1.toml
# (prints "run dir: runs/<exp_id>-<UTC>/", then the env sim-time end.)
```
