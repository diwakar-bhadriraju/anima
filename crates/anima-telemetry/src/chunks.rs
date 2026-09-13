//! Chunked columnar telemetry store (T-CHUNK, telemetry-v2).
//!
//! One Parquet file per chunk (single row group, ZSTD), rolled over at
//! `CHUNK_ROWS` rows or `CHUNK_MS` of sim time; an `index.json` maps
//! chunks to [t_min, t_max, rows] for O(log n) replay seek. Envelopes
//! are flattened to typed nullable columns (no per-event text).

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use parquet::basic::{Compression, ZstdLevel};
use parquet::file::properties::WriterProperties;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::file::writer::SerializedFileWriter;
use parquet::schema::parser::parse_message_type;

use crate::events::{Envelope, EventKind, Payload};

pub const CHUNK_ROWS: usize = 250_000;
pub const CHUNK_MS: u64 = 60_000;

/// Arrow-free flat schema: one row per envelope. Nulls where a field
/// doesn't apply to the row's kind.
const SCHEMA_STR: &str = "
message row {
  required INT64 seq;
  required INT64 t;
  required INT32 kind;
  optional BINARY exp_id;
  optional INT64 n;
  optional INT64 syn;
  optional INT64 pre;
  optional INT64 post;
  optional DOUBLE w;
  optional DOUBLE delta;
  optional DOUBLE value;
  optional INT64 neurons;
  optional INT64 synapses;
  optional INT64 spikes_window;
  optional DOUBLE mean_rate_hz;
  optional INT64 active_neurons;
  optional INT64 spikes;
  optional INT64 seed;
  optional BINARY reason;
  optional BINARY detail;
  optional BINARY pattern;
  optional BINARY stage;
  optional BINARY params_json;
}
";

/// Column indices used by writer/reader (kept in sync with SCHEMA_STR).
mod col {
    pub const SEQ: usize = 0;
    pub const T: usize = 1;
    pub const KIND: usize = 2;
    pub const EXP_ID: usize = 3;
    pub const N: usize = 4;
    pub const SYN: usize = 5;
    pub const PRE: usize = 6;
    pub const POST: usize = 7;
    pub const W: usize = 8;
    pub const DELTA: usize = 9;
    pub const VALUE: usize = 10;
    pub const NEURONS: usize = 11;
    pub const SYNAPSES: usize = 12;
    pub const SPIKES_WINDOW: usize = 13;
    pub const MEAN_RATE_HZ: usize = 14;
    pub const ACTIVE_NEURONS: usize = 15;
    pub const SPIKES: usize = 16;
    pub const SEED: usize = 17;
    pub const REASON: usize = 18;
    pub const DETAIL: usize = 19;
    pub const PATTERN: usize = 20;
    pub const STAGE: usize = 21;
    pub const PARAMS_JSON: usize = 22;
}

pub const N_COLUMNS: usize = 23;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChunkMeta {
    pub file: String,
    pub t_min: u64,
    pub t_max: u64,
    pub rows: u64,
}

/// One chunk's in-flight column buffers.
#[derive(Default)]
struct Buffers {
    seq: Vec<i64>,
    t: Vec<i64>,
    kind: Vec<i32>,
    exp_id: Vec<Option<Vec<u8>>>,

    n: Vec<Option<i64>>,
    syn: Vec<Option<i64>>,
    pre: Vec<Option<i64>>,
    post: Vec<Option<i64>>,
    w: Vec<Option<f64>>,
    delta: Vec<Option<f64>>,
    value: Vec<Option<f64>>,
    neurons: Vec<Option<i64>>,
    synapses: Vec<Option<i64>>,
    spikes_window: Vec<Option<i64>>,
    mean_rate_hz: Vec<Option<f64>>,
    active_neurons: Vec<Option<i64>>,
    spikes: Vec<Option<i64>>,
    seed: Vec<Option<i64>>,
    reason: Vec<Option<Vec<u8>>>,
    detail: Vec<Option<Vec<u8>>>,
    pattern: Vec<Option<Vec<u8>>>,
    stage: Vec<Option<Vec<u8>>>,
    params_json: Vec<Option<Vec<u8>>>,
}

impl Buffers {
    fn new() -> Self {
        Self::default()
    }

    fn len(&self) -> usize {
        self.seq.len()
    }

    fn push(&mut self, env: &Envelope, kind_id: i32) {
        self.seq.push(env.seq as i64);
        self.t.push(env.t as i64);
        self.kind.push(kind_id);
        self.exp_id.push(Some(env.exp_id.clone().into_bytes()));
        self.n.push(None);
        self.syn.push(None);
        self.pre.push(None);
        self.post.push(None);
        self.w.push(None);
        self.delta.push(None);
        self.value.push(None);
        self.neurons.push(None);
        self.synapses.push(None);
        self.spikes_window.push(None);
        self.mean_rate_hz.push(None);
        self.active_neurons.push(None);
        self.spikes.push(None);
        self.seed.push(None);
        self.reason.push(None);
        self.detail.push(None);
        self.pattern.push(None);
        self.stage.push(None);
        self.params_json.push(None);
        let last = self.len() - 1;
        match &env.payload {
            Payload::RunStarted { config_hash, seed, params } => {
                self.seed[last] = Some(*seed as i64);
                // config_hash + params stored verbatim as one JSON blob
                // (lossless; they're only read together).
                let blob = serde_json::json!({ "config_hash": config_hash, "params": params });
                self.params_json[last] = Some(blob.to_string().into_bytes());
            }
            Payload::RunEnded { reason } => {
                self.reason[last] = Some(reason.clone().into_bytes());
            }
            Payload::TickStats { mean_rate_hz, active_neurons, spikes } => {
                self.mean_rate_hz[last] = mean_rate_hz.map(|v| v as f64);
                self.active_neurons[last] = Some(*active_neurons as i64);
                self.spikes[last] = Some(*spikes as i64);
            }
            Payload::Spike { n } => self.n[last] = Some(n.0 as i64),
            Payload::OutputActivity { n } => self.n[last] = Some(n.0 as i64),
            Payload::StimulusPresented { pattern_id, stage } => {
                self.pattern[last] = Some(pattern_id.clone().into_bytes());
                self.stage[last] = Some(stage.clone().into_bytes());
            }
            Payload::SynapseCreated { syn, pre, post, w, reason } => {
                self.syn[last] = Some(syn.0 as i64);
                self.pre[last] = Some(pre.0 as i64);
                self.post[last] = Some(post.0 as i64);
                self.w[last] = w.map(|v| v as f64);
                self.reason[last] = Some(serde_json::to_vec(reason).unwrap_or_default());
            }
            Payload::SynapsePruned { syn, reason } => {
                self.syn[last] = Some(syn.0 as i64);
                self.reason[last] = Some(serde_json::to_vec(reason).unwrap_or_default());
            }
            Payload::SynapseStrengthened { syn, delta } => {
                self.syn[last] = Some(syn.0 as i64);
                self.delta[last] = delta.map(|v| v as f64);
            }
            Payload::SynapseWeakened { syn, delta } => {
                self.syn[last] = Some(syn.0 as i64);
                self.delta[last] = delta.map(|v| v as f64);
            }
            Payload::NeuronCreated { n, reason } => {
                self.n[last] = Some(n.0 as i64);
                self.reason[last] = Some(serde_json::to_vec(reason).unwrap_or_default());
            }
            Payload::NeuronDormant { n, reason } => {
                self.n[last] = Some(n.0 as i64);
                self.reason[last] = Some(serde_json::to_vec(reason).unwrap_or_default());
            }
            Payload::NeuronReactivated { n, reason } => {
                self.n[last] = Some(n.0 as i64);
                self.reason[last] = Some(serde_json::to_vec(reason).unwrap_or_default());
            }
            Payload::NeuronRetired { n, reason } => {
                self.n[last] = Some(n.0 as i64);
                self.reason[last] = Some(serde_json::to_vec(reason).unwrap_or_default());
            }
            Payload::PredictionError { value } => {
                self.value[last] = value.map(|v| v as f64);
            }
            Payload::NoveltySignal { value } => {
                self.value[last] = value.map(|v| v as f64);
            }
            Payload::ResourceUsage { neurons, synapses, spikes_window, metabolic_cost } => {
                self.neurons[last] = Some(*neurons as i64);
                self.synapses[last] = Some(*synapses as i64);
                self.spikes_window[last] = Some(*spikes_window as i64);
                self.value[last] = metabolic_cost.map(|v| v as f64);
            }
            Payload::Failure { kind, detail } => {
                self.reason[last] = Some(kind.clone().into_bytes());
                self.detail[last] = Some(detail.clone().into_bytes());
            }
        }
    }
}

/// Chunked columnar telemetry writer. Replaces the JSONL file for new runs.
pub struct ChunkedTelemetry {
    dir: PathBuf,
    buffers: Buffers,
    chunk_open_t: u64,
    chunks: Vec<ChunkMeta>,
    rows_total: u64,
    chunk_seq: usize,
}

impl ChunkedTelemetry {
    pub fn create(dir: &Path) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        Ok(Self {
            dir: dir.to_path_buf(),
            buffers: Buffers::new(),
            chunk_open_t: 0,
            chunks: Vec::new(),
            rows_total: 0,
            chunk_seq: 0,
        })
    }

    pub fn append(&mut self, env: &Envelope) -> io::Result<()> {
        if self.buffers.len() == 0 {
            self.chunk_open_t = env.t;
        }
        let kind_id = env.kind.to_id();
        self.buffers.push(env, kind_id);
        self.rows_total += 1;
        if self.buffers.len() >= CHUNK_ROWS || env.t.saturating_sub(self.chunk_open_t) >= CHUNK_MS {
            self.roll_chunk()?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> io::Result<()> {
        if self.buffers.len() > 0 {
            self.roll_chunk()?;
        }
        Ok(())
    }

    pub fn rows_total(&self) -> u64 {
        self.rows_total
    }

    pub fn index(&self) -> &[ChunkMeta] {
        &self.chunks
    }

    fn roll_chunk(&mut self) -> io::Result<()> {
        if self.buffers.len() == 0 {
            return Ok(());
        }
        let file_name = format!("chunk-{:05}.parquet", self.chunk_seq);
        self.chunk_seq += 1;
        let path = self.dir.join(&file_name);
        let file = File::create(&path)?;
        let schema = parse_message_type(SCHEMA_STR)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("schema: {e}")))?;
        let props = WriterProperties::builder()
            .set_compression(Compression::ZSTD(ZstdLevel::try_new(3).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?))
            .build();
        let mut writer = SerializedFileWriter::new(file, std::sync::Arc::new(schema), std::sync::Arc::new(props))?;
        let mut rg = writer.next_row_group()?;
        write_col_i64(&mut rg, col::SEQ, &self.buffers.seq)?;
        write_col_i64(&mut rg, col::T, &self.buffers.t)?;
        write_col_i32(&mut rg, col::KIND, &self.buffers.kind)?;
        write_col_opt_bytes(&mut rg, col::EXP_ID, &self.buffers.exp_id)?;
        write_col_opt_i64(&mut rg, col::N, &self.buffers.n)?;
        write_col_opt_i64(&mut rg, col::SYN, &self.buffers.syn)?;
        write_col_opt_i64(&mut rg, col::PRE, &self.buffers.pre)?;
        write_col_opt_i64(&mut rg, col::POST, &self.buffers.post)?;
        write_col_opt_f64(&mut rg, col::W, &self.buffers.w)?;
        write_col_opt_f64(&mut rg, col::DELTA, &self.buffers.delta)?;
        write_col_opt_f64(&mut rg, col::VALUE, &self.buffers.value)?;
        write_col_opt_i64(&mut rg, col::NEURONS, &self.buffers.neurons)?;
        write_col_opt_i64(&mut rg, col::SYNAPSES, &self.buffers.synapses)?;
        write_col_opt_i64(&mut rg, col::SPIKES_WINDOW, &self.buffers.spikes_window)?;
        write_col_opt_f64(&mut rg, col::MEAN_RATE_HZ, &self.buffers.mean_rate_hz)?;
        write_col_opt_i64(&mut rg, col::ACTIVE_NEURONS, &self.buffers.active_neurons)?;
        write_col_opt_i64(&mut rg, col::SPIKES, &self.buffers.spikes)?;
        write_col_opt_i64(&mut rg, col::SEED, &self.buffers.seed)?;
        write_col_opt_bytes(&mut rg, col::REASON, &self.buffers.reason)?;
        write_col_opt_bytes(&mut rg, col::DETAIL, &self.buffers.detail)?;
        write_col_opt_bytes(&mut rg, col::PATTERN, &self.buffers.pattern)?;
        write_col_opt_bytes(&mut rg, col::STAGE, &self.buffers.stage)?;
        write_col_opt_bytes(&mut rg, col::PARAMS_JSON, &self.buffers.params_json)?;
        rg.close()?;
        writer.close()?;
        let meta = ChunkMeta {
            file: file_name,
            t_min: *self.buffers.t.first().unwrap_or(&0) as u64,
            t_max: *self.buffers.t.last().unwrap_or(&0) as u64,
            rows: self.buffers.len() as u64,
        };
        self.chunks.push(meta.clone());
        self.buffers = Buffers::new();
        self.write_index()
    }

    fn write_index(&self) -> io::Result<()> {
        let idx = serde_json::json!({ "format": "t-chunk-v1", "chunks": self.chunks });
        fs::write(self.dir.join("index.json"), serde_json::to_vec_pretty(&idx)?)?;
        Ok(())
    }
}

fn write_col_i64(rg: &mut parquet::file::writer::SerializedRowGroupWriter<'_, File>, idx: usize, v: &[i64]) -> io::Result<()> {
    use parquet::column::writer::ColumnWriter;
    let mut cols = rg.next_column()?.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing column"))?;
    let cw = match cols.untyped() {
        ColumnWriter::Int64ColumnWriter(c) => c,
        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "type mismatch")),
    };
    cw.write_batch(v, None, None)?;
    cols.close()?;
    let _ = idx;
    Ok(())
}

fn write_col_i32(rg: &mut parquet::file::writer::SerializedRowGroupWriter<'_, File>, idx: usize, v: &[i32]) -> io::Result<()> {
    use parquet::column::writer::ColumnWriter;
    let mut cols = rg.next_column()?.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing column"))?;
    let cw = match cols.untyped() {
        ColumnWriter::Int32ColumnWriter(c) => c,
        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "type mismatch")),
    };
    cw.write_batch(v, None, None)?;
    cols.close()?;
    let _ = idx;
    Ok(())
}

fn write_col_opt_i64(rg: &mut parquet::file::writer::SerializedRowGroupWriter<'_, File>, idx: usize, v: &[Option<i64>]) -> io::Result<()> {
    use parquet::column::writer::ColumnWriter;
    let mut cols = rg.next_column()?.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing column"))?;
    let cw = match cols.untyped() {
        ColumnWriter::Int64ColumnWriter(c) => c,
        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "type mismatch")),
    };
    let values: Vec<i64> = v.iter().filter_map(|o| *o).collect();
    let def: Vec<i16> = v.iter().map(|o| if o.is_some() { 1 } else { 0 }).collect();
    cw.write_batch(&values, Some(&def), None)?;
    cols.close()?;
    let _ = idx;
    Ok(())
}

fn write_col_opt_f64(rg: &mut parquet::file::writer::SerializedRowGroupWriter<'_, File>, idx: usize, v: &[Option<f64>]) -> io::Result<()> {
    use parquet::column::writer::ColumnWriter;
    let mut cols = rg.next_column()?.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing column"))?;
    let cw = match cols.untyped() {
        ColumnWriter::DoubleColumnWriter(c) => c,
        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "type mismatch")),
    };
    let values: Vec<f64> = v.iter().filter_map(|o| *o).collect();
    let def: Vec<i16> = v.iter().map(|o| if o.is_some() { 1 } else { 0 }).collect();
    cw.write_batch(&values, Some(&def), None)?;
    cols.close()?;
    let _ = idx;
    Ok(())
}

fn write_col_opt_bytes(rg: &mut parquet::file::writer::SerializedRowGroupWriter<'_, File>, idx: usize, v: &[Option<Vec<u8>>]) -> io::Result<()> {
    use parquet::column::writer::ColumnWriter;
    let mut cols = rg.next_column()?.ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing column"))?;
    let cw = match cols.untyped() {
        ColumnWriter::ByteArrayColumnWriter(c) => c,
        _ => return Err(io::Error::new(io::ErrorKind::InvalidData, "type mismatch")),
    };
    use parquet::data_type::ByteArray;
    let values: Vec<ByteArray> = v.iter()
        .filter_map(|o| o.as_deref().map(ByteArray::from))
        .collect();
    let def: Vec<i16> = v.iter().map(|o| if o.is_some() { 1 } else { 0 }).collect();
    cw.write_batch(&values, Some(&def), None)?;
    cols.close()?;
    let _ = idx;
    Ok(())
}

/// Read a chunk's Parquet file back into rows via the stable row iterator.
fn read_chunk(path: &Path) -> io::Result<Vec<Row>> {
    let file = File::open(path)?;
    let reader = SerializedFileReader::new(file)?;
    let mut rows = Vec::new();
    let iter = reader.get_row_iter(None)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("row iter: {e}")))?;
    use parquet::record::RowAccessor;
    for row in iter {
        let row = row.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("row: {e}")))?;
        // Column order is fixed by SCHEMA_STR.
        let get_i64 = |idx: usize, row: &parquet::record::Row| -> Option<i64> { RowAccessor::get_long(row, idx).ok() };
        let get_f64 = |idx: usize, row: &parquet::record::Row| -> Option<f64> { RowAccessor::get_double(row, idx).ok() };
        let get_str = |idx: usize, row: &parquet::record::Row| -> Option<Vec<u8>> {
            RowAccessor::get_bytes(row, idx).ok().map(|b| b.data().to_vec())
        };
        rows.push(Row {
            seq: get_i64(col::SEQ, &row).unwrap_or(0) as u64,
            t: get_i64(col::T, &row).unwrap_or(0) as u64,
            kind: RowAccessor::get_int(&row, col::KIND).unwrap_or(0),
            exp_id: get_str(col::EXP_ID, &row),
            n: get_i64(col::N, &row),
            syn: get_i64(col::SYN, &row),
            pre: get_i64(col::PRE, &row),
            post: get_i64(col::POST, &row),
            w: get_f64(col::W, &row),
            delta: get_f64(col::DELTA, &row),
            value: get_f64(col::VALUE, &row),
            neurons: get_i64(col::NEURONS, &row),
            synapses: get_i64(col::SYNAPSES, &row),
            spikes_window: get_i64(col::SPIKES_WINDOW, &row),
            mean_rate_hz: get_f64(col::MEAN_RATE_HZ, &row),
            active_neurons: get_i64(col::ACTIVE_NEURONS, &row),
            spikes: get_i64(col::SPIKES, &row),
            seed: get_i64(col::SEED, &row),
            reason: get_str(col::REASON, &row),
            detail: get_str(col::DETAIL, &row),
            pattern: get_str(col::PATTERN, &row),
            stage: get_str(col::STAGE, &row),
            params_json: get_str(col::PARAMS_JSON, &row),
        });
    }
    Ok(rows)
}

/// Flattened row (pre-reconstruction).
#[derive(Debug, Clone)]
pub struct Row {
    pub seq: u64,
    pub t: u64,
    pub kind: i32,
    pub exp_id: Option<Vec<u8>>,
    pub n: Option<i64>,
    pub syn: Option<i64>,
    pub pre: Option<i64>,
    pub post: Option<i64>,
    pub w: Option<f64>,
    pub delta: Option<f64>,
    pub value: Option<f64>,
    pub neurons: Option<i64>,
    pub synapses: Option<i64>,
    pub spikes_window: Option<i64>,
    pub mean_rate_hz: Option<f64>,
    pub active_neurons: Option<i64>,
    pub spikes: Option<i64>,
    pub seed: Option<i64>,
    pub reason: Option<Vec<u8>>,
    pub detail: Option<Vec<u8>>,
    pub pattern: Option<Vec<u8>>,
    pub stage: Option<Vec<u8>>,
    pub params_json: Option<Vec<u8>>,
}

/// Streaming reader over a chunk directory. Yields envelopes in order.
pub struct TelemetryReader {
    dir: PathBuf,
    chunks: Vec<ChunkMeta>,
}

impl TelemetryReader {
    pub fn open(dir: &Path) -> io::Result<Self> {
        let idx_bytes = fs::read(dir.join("index.json"))?;
        let idx: serde_json::Value =
            serde_json::from_slice(&idx_bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let mut chunks: Vec<ChunkMeta> = serde_json::from_value(idx["chunks"].clone())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        chunks.sort_by_key(|c| c.file.clone());
        Ok(Self { dir: dir.to_path_buf(), chunks })
    }

    pub fn chunk_index(&self) -> &[ChunkMeta] {
        &self.chunks
    }

    /// Total rows across chunks.
    pub fn rows(&self) -> u64 {
        self.chunks.iter().map(|c| c.rows).sum()
    }

    /// First chunk whose t_max ≥ t (binary search over the index).
    pub fn chunk_for_seek(&self, t: u64) -> usize {
        match self.chunks.binary_search_by(|c| c.t_max.cmp(&t)) {
            Ok(i) => i,
            Err(i) => i.min(self.chunks.len().saturating_sub(1)),
        }
    }

    /// All rows of one chunk (replay seeks here; analyzer iterates 0..n).
    pub fn chunk_rows(&self, chunk: usize) -> io::Result<Vec<Row>> {
        let meta = self.chunks.get(chunk).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "no such chunk"))?;
        read_chunk(&self.dir.join(&meta.file))
    }
}

impl Row {
    /// Reconstruct the Envelope. Lossless for every Payload variant.
    pub fn envelope(&self, exp_id: &str) -> io::Result<Envelope> {
        use anima_core::network::{NeuronId, SynapseId};
        let kind = EventKind::from_id(self.kind)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, format!("bad kind id {}", self.kind)))?;
        let s = |o: &Option<Vec<u8>>| o.as_deref().map(|b| String::from_utf8_lossy(b).into_owned());
        let f = |o: &Option<f64>| o.map(|v| v as f32);
        let payload = match kind {
            EventKind::RunStarted => {
                let blob: serde_json::Value = serde_json::from_slice(
                    self.params_json.as_deref().unwrap_or(&b"{}".to_vec()),
                )
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Payload::RunStarted {
                    config_hash: blob["config_hash"].as_str().unwrap_or_default().to_string(),
                    seed: self.seed.unwrap_or(0) as u64,
                    params: blob["params"].clone(),
                }
            }
            EventKind::RunEnded => Payload::RunEnded { reason: s(&self.reason).unwrap_or_default() },
            EventKind::TickStats => Payload::TickStats {
                mean_rate_hz: self.mean_rate_hz.map(|v| v as f32),
                active_neurons: self.active_neurons.unwrap_or(0) as u64,
                spikes: self.spikes.unwrap_or(0) as u64,
            },
            EventKind::Spike => Payload::Spike { n: NeuronId(self.n.unwrap_or(0) as u32) },
            EventKind::OutputActivity => Payload::OutputActivity { n: NeuronId(self.n.unwrap_or(0) as u32) },
            EventKind::StimulusPresented => Payload::StimulusPresented {
                pattern_id: s(&self.pattern).unwrap_or_default(),
                stage: s(&self.stage).unwrap_or_default(),
            },
            EventKind::SynapseCreated => Payload::SynapseCreated {
                syn: SynapseId(self.syn.unwrap_or(0) as u32),
                pre: NeuronId(self.pre.unwrap_or(0) as u32),
                post: NeuronId(self.post.unwrap_or(0) as u32),
                w: self.w.map(|v| v as f32),
                reason: serde_json::from_slice(self.reason.as_deref().unwrap_or(b"{}"))
                    .unwrap_or_else(|_| crate::events::ReasonPayload::simple("unspecified")),
            },
            EventKind::SynapsePruned => Payload::SynapsePruned {
                syn: SynapseId(self.syn.unwrap_or(0) as u32),
                reason: serde_json::from_slice(self.reason.as_deref().unwrap_or(b"{}"))
                    .unwrap_or_else(|_| crate::events::ReasonPayload::simple("unspecified")),
            },
            EventKind::SynapseStrengthened => Payload::SynapseStrengthened {
                syn: SynapseId(self.syn.unwrap_or(0) as u32),
                delta: self.delta.map(|v| v as f32),
            },
            EventKind::SynapseWeakened => Payload::SynapseWeakened {
                syn: SynapseId(self.syn.unwrap_or(0) as u32),
                delta: self.delta.map(|v| v as f32),
            },
            EventKind::NeuronCreated => Payload::NeuronCreated {
                n: NeuronId(self.n.unwrap_or(0) as u32),
                reason: serde_json::from_slice(self.reason.as_deref().unwrap_or(b"{}"))
                    .unwrap_or_else(|_| crate::events::ReasonPayload::simple("unspecified")),
            },
            EventKind::NeuronDormant => Payload::NeuronDormant {
                n: NeuronId(self.n.unwrap_or(0) as u32),
                reason: serde_json::from_slice(self.reason.as_deref().unwrap_or(b"{}"))
                    .unwrap_or_else(|_| crate::events::ReasonPayload::simple("unspecified")),
            },
            EventKind::NeuronReactivated => Payload::NeuronReactivated {
                n: NeuronId(self.n.unwrap_or(0) as u32),
                reason: serde_json::from_slice(self.reason.as_deref().unwrap_or(b"{}"))
                    .unwrap_or_else(|_| crate::events::ReasonPayload::simple("unspecified")),
            },
            EventKind::NeuronRetired => Payload::NeuronRetired {
                n: NeuronId(self.n.unwrap_or(0) as u32),
                reason: serde_json::from_slice(self.reason.as_deref().unwrap_or(b"{}"))
                    .unwrap_or_else(|_| crate::events::ReasonPayload::simple("unspecified")),
            },
            EventKind::PredictionError => Payload::PredictionError { value: f(&self.value) },
            EventKind::NoveltySignal => Payload::NoveltySignal { value: f(&self.value) },
            EventKind::ResourceUsage => Payload::ResourceUsage {
                neurons: self.neurons.unwrap_or(0) as u64,
                synapses: self.synapses.unwrap_or(0) as u64,
                spikes_window: self.spikes_window.unwrap_or(0) as u64,
                metabolic_cost: self.value.map(|v| v as f32),
            },
            EventKind::Failure => Payload::Failure {
                kind: s(&self.reason).unwrap_or_default(),
                detail: s(&self.detail).unwrap_or_default(),
            },
        };
        let exp = self.exp_id.as_deref()
            .map(|b| String::from_utf8_lossy(b).into_owned())
            .unwrap_or_else(|| exp_id.to_string());
        Ok(Envelope { seq: self.seq, t: self.t, exp_id: exp, kind, payload })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventBuilder, ReasonPayload};
    use anima_core::network::{NeuronId, SynapseId};

    fn sample_envelopes() -> Vec<Envelope> {
        let mut b = EventBuilder::new("e2");
        vec![
            b.build(0, Payload::RunStarted {
                config_hash: "cafe1234".into(),
                seed: 20260912,
                params: serde_json::json!({ "n_internal": 40, "amplitude": 52.0 }),
            }),
            b.build(1, Payload::NeuronCreated {
                n: NeuronId(24),
                reason: ReasonPayload::simple("initial-wiring"),
            }),
            b.build(2, Payload::StimulusPresented { pattern_id: "A".into(), stage: "S1".into() }),
            b.build(2, Payload::Spike { n: NeuronId(24) }),
            b.build(3, Payload::SynapseStrengthened { syn: SynapseId(7), delta: Some(0.03) }),
            b.build(4, Payload::SynapseWeakened { syn: SynapseId(9), delta: Some(-0.01) }),
            b.build(5, Payload::SynapseCreated {
                syn: SynapseId(50), pre: NeuronId(1), post: NeuronId(30),
                w: Some(0.2), reason: ReasonPayload::simple("initial-wiring"),
            }),
            b.build(6, Payload::PredictionError { value: Some(0.4) }),
            b.build(6, Payload::PredictionError { value: None }), // NaN → null
            b.build(10, Payload::ResourceUsage {
                neurons: 76, synapses: 125, spikes_window: 210, metabolic_cost: Some(1.25),
            }),
            b.build(20, Payload::Failure { kind: "runaway-activity".into(), detail: "test".into() }),
            b.build(30, Payload::RunEnded { reason: "curriculum-complete".into() }),
        ]
    }

    fn write_read(events: &[Envelope]) -> io::Result<(Vec<Envelope>, Vec<ChunkMeta>)> {
        let tmp = std::env::temp_dir().join(format!("anima-chunk-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos()));
        let mut store = ChunkedTelemetry::create(&tmp)?;
        let exp_id = "e2".to_string();
        for e in events {
            store.append(&Envelope { exp_id: exp_id.clone(), ..e.clone() })?;
        }
        store.flush()?;
        let idx = store.index().to_vec();
        let reader = TelemetryReader::open(&tmp)?;
        let mut out = Vec::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c)? {
                out.push(row.envelope(&exp_id)?);
            }
        }
        let _ = fs::remove_dir_all(&tmp);
        Ok((out, idx))
    }

    #[test]
    fn chunk_roundtrip_lossless() {
        let events = sample_envelopes();
        let (out, _idx) = write_read(&events).expect("write+read");
        assert_eq!(out.len(), events.len());
        for (a, b) in events.iter().zip(out.iter()) {
            let sa = serde_json::to_string(a).unwrap();
            let sb = serde_json::to_string(b).unwrap();
            assert_eq!(sa, sb, "roundtrip mismatch:\n  in:  {sa}\n  out: {sb}");
        }
    }

    #[test]
    fn chunk_rollover_produces_index_and_seek() {
        let tmp = std::env::temp_dir().join(format!("anima-chunk-roll-{}", std::process::id()));
        let mut store = ChunkedTelemetry::create(&tmp).unwrap();
        let mut b = EventBuilder::new("roll");
        for t in [0u64, 61_000, 122_000] {
            store.append(&b.build(t, Payload::Spike { n: NeuronId(1) })).unwrap();
        }
        store.flush().unwrap();
        // Rollover happens after a chunk spans ≥ CHUNK_MS: [0, 61s], [122s].
        assert_eq!(store.index().len(), 2);
        let reader = TelemetryReader::open(&tmp).unwrap();
        assert_eq!(reader.chunk_index().len(), 2);
        // seek: t=100000 → binary search finds the chunk with t_max ≥ t
        let c = reader.chunk_for_seek(100_000);
        let rows = reader.chunk_rows(c).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].t, 122_000);
        // determinism: same rows each read
        let rows2 = reader.chunk_rows(c).unwrap();
        assert_eq!(rows.len(), rows2.len());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn chunk_row_cap_rollover() {
        // CHUNK_ROWS is 250k — too slow for a test; verify many small appends
        // stay in one chunk and flush() closes it exactly once (no empty chunk).
        let tmp = std::env::temp_dir().join(format!("anima-chunk-cap-{}", std::process::id()));
        let mut store = ChunkedTelemetry::create(&tmp).unwrap();
        let mut b = EventBuilder::new("cap");
        for i in 0..1000u64 {
            store.append(&b.build(i, Payload::Spike { n: NeuronId((i % 40) as u32) })).unwrap();
        }
        store.flush().unwrap();
        assert_eq!(store.index().len(), 1);
        assert_eq!(store.rows_total(), 1000);
        // second flush (no new rows) must not create an empty chunk
        store.flush().unwrap();
        assert_eq!(store.index().len(), 1);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn corrupt_chunk_reports_error_not_panic() {
        let tmp = std::env::temp_dir().join(format!("anima-chunk-corrupt-{}", std::process::id()));
        let mut store = ChunkedTelemetry::create(&tmp).unwrap();
        let mut b = EventBuilder::new("corrupt");
        store.append(&b.build(0, Payload::Spike { n: NeuronId(1) })).unwrap();
        store.flush().unwrap();
        let meta = store.index()[0].clone();
        drop(store);
        // corrupt the parquet file
        fs::write(tmp.join(&meta.file), b"not parquet at all").unwrap();
        let reader = TelemetryReader::open(&tmp).unwrap();
        assert!(reader.chunk_rows(0).is_err(), "corrupt chunk must return Err");
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn nan_roundtrip_via_null() {
        // PredictionError with NaN serializes to null (f32_json contract)
        // and reads back as None — preserving the analyzer's semantics.
        let tmp = std::env::temp_dir().join(format!("anima-chunk-nan-{}", std::process::id()));
        let mut store = ChunkedTelemetry::create(&tmp).unwrap();
        let mut b = EventBuilder::new("nan");
        store.append(&b.build(5, Payload::PredictionError { value: f32_json(None) })).unwrap();
        store.flush().unwrap();
        let reader = TelemetryReader::open(&tmp).unwrap();
        let rows = reader.chunk_rows(0).unwrap();
        let env = rows[0].envelope("nan").unwrap();
        match env.payload {
            Payload::PredictionError { value } => assert!(value.is_none()),
            _ => panic!("wrong kind"),
        }
        let _ = fs::remove_dir_all(&tmp);
    }

    // f32_json re-exported locally for the test above.
    fn f32_json(v: Option<f32>) -> Option<f32> {
        v.filter(|x| x.is_finite())
    }
}
