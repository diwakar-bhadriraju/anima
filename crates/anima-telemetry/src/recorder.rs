//! Recorder: append-only JSONL telemetry + zstd-compressed snapshot frames.
//!
//! Run dir = `runs/<exp>-<UTC timestamp>/`; refuses to overwrite an existing
//! dir. Write error aborts the run and preserves the partial file (§22).

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::events::Envelope;

/// Full network state snapshot (neurons + synapses), written every 1000
/// ticks into `snapshots.bin.zst` as length-prefixed frames.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStateSnapshot {
    pub tick: u64,
    pub neurons: Vec<NeuronState>,
    pub synapses: Vec<SynapseState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeuronState {
    pub id: u32,
    pub class: String,
    pub v: Option<f32>,
    pub rate_hz: Option<f32>,
    pub dormant: bool,
    pub retired: bool,
    pub born: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynapseState {
    pub id: u32,
    pub pre: u32,
    pub post: u32,
    pub w: Option<f32>,
    pub plastic: bool,
}

pub struct Recorder {
    dir: PathBuf,
    telemetry: BufWriter<File>,
    snapshot: Option<zstd::stream::write::AutoFinishEncoder<'static, BufWriter<File>>>,
    events_written: u64,
}

/// UTC timestamp dir component: `YYYYmmddTHHMMSSZ` (second resolution —
/// caller retries if the dir exists).
pub fn utc_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, mo, d, h, mi, s) = civil_from_unix(now);
    format!("{y:04}{mo:02}{d:02}T{h:02}{mi:02}{s:02}Z")
}

/// Days/civil-time conversion (Howard Hinnant's algorithm, u64 seconds).
fn civil_from_unix(secs: u64) -> (i64, u32, u32, u32, u32, u32) {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d, (rem / 3600) as u32, ((rem % 3600) / 60) as u32, (rem % 60) as u32)
}

impl Recorder {
    /// Create run dir `runs/<exp_id>-<ts>/` with telemetry + snapshot files.
    /// Refuses an existing dir.
    pub fn create(runs_root: &Path, exp_id: &str) -> io::Result<Self> {
        let dir = runs_root.join(format!("{exp_id}-{}", utc_timestamp()));
        if dir.exists() {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists, format!("run dir {} exists", dir.display())));
        }
        fs::create_dir_all(&dir)?;
        let telemetry = BufWriter::new(
            OpenOptions::new()
                .create_new(true)
                .append(true)
                .open(dir.join("telemetry.jsonl"))?,
        );
        let snapshot_file = OpenOptions::new()
            .create_new(true)
            .append(true)
            .open(dir.join("snapshots.bin.zst"))?;
        let snapshot =
            zstd::Encoder::new(BufWriter::new(snapshot_file), 3)?.auto_finish();
        Ok(Self { dir, telemetry, snapshot: Some(snapshot), events_written: 0 })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn events_written(&self) -> u64 {
        self.events_written
    }

    /// Append one JSONL line. Error aborts the run (caller's policy).
    pub fn write(&mut self, env: &Envelope) -> io::Result<()> {
        let mut line = serde_json::to_string(env)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        line.push('\n');
        self.telemetry.write_all(line.as_bytes())?;
        self.events_written += 1;
        Ok(())
    }

    /// Append a compressed snapshot frame: [u32 LE length][bincode-less JSON bytes].
    pub fn write_snapshot(&mut self, snap: &NetworkStateSnapshot) -> io::Result<()> {
        if let Some(enc) = self.snapshot.as_mut() {
            let bytes = serde_json::to_vec(snap)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            enc.write_all(&(bytes.len() as u32).to_le_bytes())?;
            enc.write_all(&bytes)?;
        }
        Ok(())
    }

    /// Flush both streams (keep files usable mid-run).
    pub fn flush(&mut self) -> io::Result<()> {
        self.telemetry.flush()?;
        if let Some(enc) = self.snapshot.as_mut() {
            enc.flush()?;
        }
        Ok(())
    }
}

/// Iterate a snapshots.bin.zst file frame by frame (JSON per frame).
pub fn read_snapshots(path: &Path) -> io::Result<Vec<NetworkStateSnapshot>> {
    let raw = fs::read(path)?;
    // Buffered decoder implements io::Read for frame parsing.
    let mut dec = zstd::Decoder::new(io::Cursor::new(raw))?;
    let mut frames = Vec::new();
    loop {
        let mut len_buf = [0u8; 4];
        match dec.read_exact(&mut len_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
        let len = u32::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0u8; len];
        dec.read_exact(&mut buf)?;
        frames.push(
            serde_json::from_slice(&buf)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        );
    }
    Ok(frames)
}

/// Parse a telemetry.jsonl file into envelopes (used by replay/report).
pub fn read_telemetry(path: &Path) -> io::Result<Vec<Envelope>> {
    let raw = fs::read(path)?;
    let mut out = Vec::new();
    for line in raw.split(|&b| b == b'\n') {
        if line.is_empty() {
            continue;
        }
        out.push(
            serde_json::from_slice(line)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        );
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventBuilder, Payload};

    fn sample_events() -> Vec<Envelope> {
        let mut b = EventBuilder::new("e1");
        vec![
            b.build(0, Payload::RunStarted {
                config_hash: "abc".into(),
                seed: 42,
                params: serde_json::json!({}),
            }),
            b.build(5, Payload::TickStats {
                mean_rate_hz: Some(3.5),
                active_neurons: 7,
                spikes: 9,
            }),
            b.build(6, Payload::Spike { n: anima_core::network::NeuronId(3) }),
            b.build(9, Payload::RunEnded { reason: "complete".into() }),
        ]
    }

    #[test]
    fn recorder_roundtrip() {
        let tmp = std::env::temp_dir().join(format!("anima-test-{}", std::process::id()));
        fs::create_dir_all(&tmp).unwrap();
        let events = sample_events();
        let dir;
        {
            let mut rec = Recorder::create(&tmp, "rt").unwrap();
            dir = rec.dir().to_path_buf();
            for e in &events {
                rec.write(e).unwrap();
            }
            rec.write_snapshot(&NetworkStateSnapshot {
                tick: 10,
                neurons: vec![NeuronState {
                    id: 0, class: "input".into(), v: Some(0.0), rate_hz: Some(1.0),
                    dormant: false, retired: false, born: 0,
                }],
                synapses: vec![SynapseState { id: 0, pre: 0, post: 1, w: Some(0.2), plastic: true }],
            }).unwrap();
            rec.write_snapshot(&NetworkStateSnapshot {
                tick: 20,
                neurons: vec![],
                synapses: vec![],
            }).unwrap();
            rec.flush().unwrap();
        }
        let parsed = read_telemetry(&dir.join("telemetry.jsonl")).unwrap();
        assert_eq!(parsed.len(), events.len());
        for (a, b) in parsed.iter().zip(events.iter()) {
            assert_eq!(serde_json::to_string(a).unwrap(), serde_json::to_string(b).unwrap());
        }
        let snaps = read_snapshots(&dir.join("snapshots.bin.zst")).unwrap();
        assert_eq!(snaps.len(), 2);
        assert_eq!(snaps[0].tick, 10);
        assert_eq!(snaps[1].synapses.len(), 0);
        // Second create in same second: dir exists ⇒ AlreadyExists (skip if
        // clock ticked).
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn refuses_existing_dir() {
        let tmp = std::env::temp_dir().join(format!("anima-test-x-{}", std::process::id()));
        fs::create_dir_all(&tmp).unwrap();
        let first = Recorder::create(&tmp, "dup").unwrap();
        let dir = first.dir().to_path_buf();
        drop(first);
        let err = Recorder::create(dir.parent().unwrap(), "dup").is_err()
            || { // same-second same-name attempt
                let second = Recorder::create(dir.parent().unwrap(), "dup");
                match second {
                    Ok(r2) => { let _ = fs::remove_dir_all(r2.dir()); false }
                    Err(_) => true,
                }
            };
        assert!(err, "creating into an existing dir path must fail or be distinct");
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn civil_time_sanity() {
        // 2026-01-01T00:00:00Z = 1767225600
        let (y, m, d, h, mi, s) = civil_from_unix(1_767_225_600);
        assert_eq!((y, m, d, h, mi, s), (2026, 1, 1, 0, 0, 0));
        let (y, m, d, h, mi, s) = civil_from_unix(0);
        assert_eq!((y, m, d, h, mi, s), (1970, 1, 1, 0, 0, 0));
    }
}
