# Telemetry Storage v2 — Columnar Chunks (T-CHUNK)

**Status**: designed 2026-09-13, implemented before E3. Supersedes the
JSONL telemetry file for new runs. E1/E2 records remain on disk as-is.

## Problem

E2 arm B (a pathological high-rate run) produced 8.3 GB of JSONL:
48.8 M weight events serialized as text, one JSON object per line. The
analyzer's `read_telemetry()` loads the whole file into RAM, so report
generation after such a run costs minutes of parse + multi-GB RSS, and
the sim thread pays per-event `serde_json::to_string` on the hot path.

## Requirements

- Retain everything (lossless for all discrete events; no silent drops).
- Bounded memory: never buffer the whole run.
- No per-event text serialization on the sim thread.
- Replay/seek/scrub, analyzer metrics, timeline, inspector keep working.
- Backpressure: a pathological run cannot allocate without bound.
- Determinism: same seed ⇒ identical stored data.

## Design

**One run dir contains N Parquet row-group chunks + a small index.**

```
runs/<exp>-<ts>/
  telemetry/            # chunked columnar store
    chunk-00000.parquet
    chunk-00001.parquet
    ...
    index.json          # chunk table + per-chunk [t_min, t_max, n_rows]
  snapshots.bin.zst     # unchanged
  report.md, metrics.json
```

- **Chunk**: one Parquet file with a single row group, ZSTD(3). Rolled
  over at 250k rows OR 60 s of sim time, whichever first. 250k rows
  ≈ 8–25 MB compressed for our event mix — bounded, seekable.
- **Columns** (typed, one row per envelope):
  - `seq: u64` (delta-encoded by Parquet automatically)
  - `t: u64` sim-ms (sorted ⇒ dictionary/delta friendly)
  - `kind: u16` small-enum dictionary
  - Payload columns are unioned across kinds with per-kind fields as
    nullable columns: `n: u32`, `syn: u32`, `pre: u32`, `post: u32`,
    `w: f32`, `delta: f32`, `value: f32`, `neurons: u64`,
    `synapses: u64`, `spikes_window: u64`, `mean_rate_hz: f32`,
    `active_neurons: u64`, `spikes: u64`, `reason: utf8`,
    `detail: utf8`, `pattern: utf8`, `stage: utf8`, `str1: utf8`,
    `seed: u64`, `params_json: utf8`.
  - `reason_json: utf8` for ReasonPayload (rare events only).
  Sparse null columns compress to near-nothing; no text enum repetition.
- **High-rate kinds stay rows** (spikes, weight deltas): at 48 M events
  the row count is high but each row is ~10 typed bytes + ZSTD —
  measured compression beats JSONL ~10–20×. Spike *batching* (multiple
  neurons at one tick) is kept as separate rows to preserve strict
  per-event seq ordering and lossless reconstruction.
- **index.json**: `[{file, t_min, t_max, rows}]` written on rollover.
  Replay seek = binary-search the index, open one chunk, iterate.
  Analyzer = iterate chunks sequentially, aggregate incrementally.
- **Backpressure**: writer buffers up to 250k rows (few MB); rollover
  is synchronous on the sim thread at chunk boundaries (amortized
  ~1/60 s of sim time). A pathological burst cannot allocate beyond
  the current chunk's row budget.

## Compatibility

- `Recorder::write(&Envelope)` signature unchanged; internally routes
  to the columnar buffers.
- `read_telemetry()` keeps its signature (`→ Vec<Envelope>`) for
  small/legacy files, but new-callers use `TelemetryReader::iter()`
  which streams chunk by chunk and reconstructs `Envelope`s.
- JSONL remains readable (legacy runs); format detection by directory
  (`telemetry/` dir present ⇒ v2, else JSONL).
- Lossless roundtrip is a hard test: Envelope → columns → Envelope
  must be identity for every Payload variant (f32 NaN→null preserved
  via f32_json contract; params JSON kept verbatim).

## What we do NOT do

- No Arrow IPC files, no external DB — Parquet files + index.json are
  the whole store.
- No background writer thread (determinism/simplicity; rollover cost
  is amortized and bounded).
- No schema evolution machinery (v1 accepts v1; a future format change
  bumps a format version in index.json).
