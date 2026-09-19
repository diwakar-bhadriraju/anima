//! X3 boundary map recorder: one row per xs- run.
//! Fields: seed, drive, beta | end reason | regime (silence classifier) |
//! drive-end top-u (neuron id, magnitude) | silence trajectory summary
//! (spikes in first/last 5 s + half-life bins).
use std::collections::BTreeMap;

fn main() {
    for dir in std::env::args().skip(1) {
        let name = dir.rsplit('/').next().unwrap().to_string();
        // xs-s{seed}-{drive}-b{beta}-{ts}
        let parts: Vec<&str> = name.split('-').collect();
        let seed = parts[1].trim_start_matches('s').to_string();
        let drive = parts[2].to_string();
        let beta = parts[3].to_string();
        let tdir = std::path::Path::new(&dir).join("telemetry");
        let reader = match anima_telemetry::TelemetryReader::open(&tdir) {
            Ok(r) => r,
            Err(_) => { println!("{seed}\t{drive}\t{beta}\tNO-TELEMETRY"); continue; }
        };
        let idx = reader.chunk_index();
        let mut last_end = 0u64;
        let mut spikes: Vec<(u64, u32)> = Vec::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                match row.kind {
                    5 => {
                        if let Some(env) = row.envelope("e").ok() {
                            if let anima_telemetry::events::Payload::StimulusPresented { .. } = env.payload {
                                last_end = (env.t + 500).max(last_end);
                            }
                        }
                    }
                    3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                    _ => {}
                }
            }
        }
        spikes.sort_by_key(|s| s.0);
        // end reason from metrics.json failures
        let fail = std::fs::read_to_string(std::path::Path::new(&dir).join("metrics.json")).ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .map(|m| {
                let f = m.get("failures").cloned().unwrap_or(serde_json::Value::Null);
                if f.is_array() && !f.as_array().unwrap().is_empty() {
                    format!("abort@{}", f[0].get("t").and_then(|t| t.as_u64()).unwrap_or(0))
                } else { "complete".to_string() }
            }).unwrap_or_else(|| "?".to_string());
        // drive-end top-u from snapshots: last snapshot at/below last_end
        let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let s_end = snaps.iter().filter(|s| s.tick <= last_end).next_back().unwrap();
        let mut topu: Vec<(u32, f32)> = s_end.neurons.iter().map(|n| (n.id, n.u_slow.unwrap_or(0.0))).collect();
        topu.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let top = topu[0];
        let top3: f32 = topu.iter().take(3).map(|x| x.1).sum();
        // silence trajectory
        let end = spikes.last().map(|s| s.0 + 1).unwrap_or(last_end);
        let sil: Vec<(u64, u32)> = spikes.iter().filter(|(t, n)| *t >= last_end && *n >= 24).cloned().collect();
        let mut per: BTreeMap<u32, u64> = BTreeMap::new();
        for &(_, n) in &sil { *per.entry(n).or_default() += 1; }
        let first5 = sil.iter().filter(|(t, _)| *t < last_end + 5_000).count();
        let last5 = sil.iter().filter(|(t, _)| *t + 5_000 >= end).count();
        let active = per.len();
        let max_hz = per.values().map(|&c| c as f64 / 20.0).fold(0.0, f64::max); // per full 20s nominal
        let regime = if fail != "complete" {
            "-".to_string()
        } else if sil.is_empty() {
            "silent".to_string()
        } else if active <= 8 && max_hz > 48.0 {
            "pacemaker".to_string()
        } else if active <= 8 {
            "sparse-core".to_string()
        } else {
            "irregular".to_string()
        };
        println!("{seed}\t{drive}\t{beta}\t{fail}\t{regime}\ttopu=n{}:{:.2}\ttop3sum={:.2}\tactive={}\tfirst5={}\tlast5={}",
            top.0, top.1, top3, active, first5, last5);
    }
}
