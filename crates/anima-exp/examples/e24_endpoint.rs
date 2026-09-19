//! E24 endpoint instrument (FROZEN at protocol commit, before execution).
//! Primary endpoint M per run:
//!   abort-on-runaway-activity  -> M = +inf (top rank)
//!   otherwise                  -> M = log10(1 + S10),
//!     S10 = endogenous (non-input) spikes in [silence_onset, +10 s),
//!     silence_onset = last StimulusPresented t + 500 (delivery convention:
//!     relative window, arm-symmetric).
//! Secondary: regime class (classifier frozen as in v21phase), drive-end
//! top-u (id+mag) + top3, active set, silence trajectory (2 s bins over
//! full 20 s silence), output-class participation (ids/counts from snapshot
//! class field).
//! Output: TSV rows seed, arm, end, M, S10, regime, topu, top3, active,
//! out_act, out_spk, traj.
use std::collections::BTreeMap;

fn main() {
    for dir in std::env::args().skip(1) {
        let name = dir.rsplit('/').next().unwrap().to_string();
        let parts: Vec<&str> = name.split('-').collect();
        let seed = parts[1].trim_start_matches('s').to_string();
        let arm = parts[2].to_string();
        let tdir = std::path::Path::new(&dir).join("telemetry");
        let reader = match anima_telemetry::TelemetryReader::open(&tdir) {
            Ok(r) => r,
            Err(_) => { println!("{seed}\t{arm}\tNO-TELEMETRY"); continue; }
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
        let fail = std::fs::read_to_string(std::path::Path::new(&dir).join("metrics.json")).ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .map(|m| {
                let f = m.get("failures").cloned().unwrap_or(serde_json::Value::Null);
                if f.is_array() && !f.as_array().unwrap().is_empty() {
                    let k = f[0].get("kind").and_then(|x| x.as_str()).unwrap_or("?").to_string();
                    let t = f[0].get("t").and_then(|x| x.as_u64()).unwrap_or(0);
                    format!("abort:{k}@{t}")
                } else { "complete".to_string() }
            }).unwrap_or_else(|| "?".to_string());

        // snapshots: classes + drive-end u
        let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let s_end = snaps.iter().filter(|s| s.tick <= last_end).next_back().unwrap();
        let mut topu: Vec<(u32, f32)> = s_end.neurons.iter().map(|n| (n.id, n.u_slow.unwrap_or(0.0))).collect();
        topu.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let top = topu[0];
        let top3: f32 = topu.iter().take(3).map(|x| x.1).sum();
        let class_of: BTreeMap<u32, String> = s_end.neurons.iter().map(|n| (n.id, n.class.clone())).collect();

        let end = spikes.last().map(|s| s.0 + 1).unwrap_or(last_end);
        let sil: Vec<(u64, u32)> = spikes.iter().filter(|(t, n)| *t >= last_end && *n >= 24).cloned().collect();
        let s10 = sil.iter().filter(|(t, _)| *t < last_end + 10_000).count();

        if fail != "complete" {
            println!("{seed}\t{arm}\t{fail}\tinf\t{s10}\t-\ttopu=n{}:{:.2}\t{top3:.2}\t-\t-\t-\t-", top.0, top.1);
            continue;
        }

        let mut per: BTreeMap<u32, u64> = BTreeMap::new();
        for &(_, n) in &sil { *per.entry(n).or_default() += 1; }
        let active = per.len();
        let sil_ms = end.saturating_sub(last_end).max(1);
        let max_hz = per.values().map(|&c| c as f64 * 1000.0 / sil_ms as f64).fold(0.0, f64::max);
        let regime = if sil.is_empty() {
            "silent".to_string()
        } else if active <= 8 && max_hz > 48.0 {
            "pacemaker".to_string()
        } else if active <= 8 {
            "sparse-core".to_string()
        } else {
            "irregular".to_string()
        };
        // output participation in silence
        let mut out_ids = Vec::new();
        let mut out_spk = 0u64;
        for (&n, &c) in &per {
            if class_of.get(&n).map(|s| s == "output").unwrap_or(false) { out_ids.push(n); out_spk += c; }
        }
        let m = (s10 as f64 + 1.0).log10();
        // trajectory: 2 s bins over full silence
        let nb = ((sil_ms + 1999) / 2000).max(1) as usize;
        let mut traj = vec![0u64; nb.min(10)];
        for &(t, _) in &sil { let i = ((t - last_end) / 2000) as usize; if i < traj.len() { traj[i] += 1; } }
        let traj_s: Vec<String> = traj.iter().map(|x| x.to_string()).collect();
        let outs: Vec<String> = out_ids.iter().map(|x| x.to_string()).collect();
        println!("{seed}\t{arm}\t{fail}\t{m:.4}\t{s10}\t{regime}\ttopu=n{}:{:.2}\t{top3:.2}\t{active}\t{}\t{out_spk}\t{}",
            top.0, top.1, outs.join("+"), traj_s.join(","));
    }
}
