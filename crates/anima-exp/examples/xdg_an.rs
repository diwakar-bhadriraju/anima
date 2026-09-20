//! X-series drive-gated diagnostic instrumentation (docs/x-spec-drive-gated.md).
//! Per run: presentation windows; during-window cosine(A,C); off-window
//! cosine(A,C) + L2 distance; u norms at stimulus offset and off-window read;
//! g_drive mean at offset/read; top-10-u Jaccard between A-window and
//! C-window u states; endogenous off-window rate + Fano; abort status.
//! Output: TSV.
use std::collections::BTreeMap;

fn main() {
    for dir in std::env::args().skip(1) {
        let name = dir.rsplit('/').next().unwrap();
        let parts: Vec<&str> = name.split('-').collect(); // xdg-{arm}-{cur}-s{seed}-{ts}
        let arm = parts[1];
        let cur = parts[2];
        let seed = parts[3].trim_start_matches('s');
        let tdir = std::path::Path::new(&dir).join("telemetry");
        let reader = match anima_telemetry::TelemetryReader::open(&tdir) {
            Ok(r) => r,
            Err(_) => { println!("{arm}\t{cur}\t{seed}\tNO-TELEMETRY"); continue; }
        };
        let idx = reader.chunk_index();
        let mut pres: Vec<(String, u64)> = Vec::new();
        let mut spikes: Vec<(u64, u32)> = Vec::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                match row.kind {
                    5 => {
                        if let Some(env) = row.envelope("e").ok() {
                            if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } = env.payload {
                                if stage == "S1" { pres.push((pattern_id, env.t)); }
                            }
                        }
                    }
                    3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                    _ => {}
                }
            }
        }
        pres.sort_by_key(|p| p.1);
        spikes.sort_by_key(|s| s.0);
        let fail = std::fs::read_to_string(std::path::Path::new(&dir).join("metrics.json")).ok()
            .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
            .map(|m| {
                let f = m.get("failures").cloned().unwrap_or(serde_json::Value::Null);
                if f.is_array() && !f.as_array().unwrap().is_empty() { "abort" } else { "ok" }
            }).unwrap_or("?");

        // Spike-count vectors per presentation window and off-window
        let mut vdur_a: BTreeMap<u32, f64> = BTreeMap::new();
        let mut vdur_c: BTreeMap<u32, f64> = BTreeMap::new();
        let mut voff_a: BTreeMap<u32, f64> = BTreeMap::new();
        let mut voff_c: BTreeMap<u32, f64> = BTreeMap::new();
        let mut dur_count_a = 0u64; let mut dur_count_c = 0u64;
        let mut off_count_a = 0u64; let mut off_count_c = 0u64;
        let mut off_vecs_a: Vec<BTreeMap<u32, f64>> = Vec::new();
        let mut off_vecs_c: Vec<BTreeMap<u32, f64>> = Vec::new();
        for (p, t) in &pres {
            for &(ts, n) in &spikes {
                if n < 24 { continue; }
                if ts >= *t && ts < *t + 500 {
                    let v = if p == "A" { &mut vdur_a } else { &mut vdur_c };
                    *v.entry(n).or_default() += 1.0;
                    if p == "A" { dur_count_a += 1; } else { dur_count_c += 1; }
                } else if ts >= *t + 500 && ts < *t + 2000 {
                    let v = if p == "A" { &mut voff_a } else { &mut voff_c };
                    *v.entry(n).or_default() += 1.0;
                    if p == "A" { off_count_a += 1; } else { off_count_c += 1; }
                }
            }
            // per-presentation off-window vector (for spread stats)
            let mut v: BTreeMap<u32, f64> = BTreeMap::new();
            for &(ts, n) in &spikes {
                if n >= 24 && ts >= *t + 500 && ts < *t + 2000 { *v.entry(n).or_default() += 1.0; }
            }
            if p == "A" { off_vecs_a.push(v); } else { off_vecs_c.push(v); }
        }
        let cos = |a: &BTreeMap<u32, f64>, b: &BTreeMap<u32, f64>| -> f64 {
            let keys: std::collections::BTreeSet<u32> = a.keys().chain(b.keys()).copied().collect();
            let d: f64 = keys.iter().map(|k| a.get(k).unwrap_or(&0.0) * b.get(k).unwrap_or(&0.0)).sum();
            let na = a.values().map(|x| x.powi(2)).sum::<f64>().sqrt();
            let nb = b.values().map(|x| x.powi(2)).sum::<f64>().sqrt();
            if na > 0.0 && nb > 0.0 { d / (na * nb) } else { 0.0 }
        };
        let c_dur = cos(&vdur_a, &vdur_c);
        let c_off = cos(&voff_a, &voff_c);
        // L2 distance between off-window mean vectors
        let keys: std::collections::BTreeSet<u32> = voff_a.keys().chain(voff_c.keys()).copied().collect();
        let md: f64 = keys.iter().map(|k| { let d = voff_a.get(k).unwrap_or(&0.0) - voff_c.get(k).unwrap_or(&0.0); d * d }).sum::<f64>().sqrt();
        // per-presentation off-window cosines (spread) + L2 mean
        let off_pair_cos = |va: &Vec<BTreeMap<u32,f64>>, vc: &Vec<BTreeMap<u32,f64>>| -> Vec<f64> {
            let n = va.len().min(vc.len());
            (0..n).map(|i| cos(&va[i], &vc[i])).collect()
        };
        let off_pairs = off_pair_cos(&off_vecs_a, &off_vecs_c);
        let off_mean_cos = if off_pairs.is_empty() { f64::NAN } else { off_pairs.iter().sum::<f64>() / off_pairs.len() as f64 };

        // snapshots: u and g at stimulus offset (last presentation end) and read (offset+1500)
        let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let last_end = pres.last().map(|(_, t)| *t + 500).unwrap_or(0);
        let snap_at = |t: u64| snaps.iter().rev().find(|s| s.tick <= t).unwrap();
        let (s_off, s_read) = (snap_at(last_end), snap_at(last_end + 1500));
        let unorm = |s: &anima_telemetry::recorder::NetworkStateSnapshot| -> f64 {
            (s.neurons.iter().filter(|n| n.id >= 24).map(|n| n.u_slow.unwrap_or(0.0) as f64).map(|x| x * x).sum::<f64>()).sqrt()
        };
        let gmean = |s: &anima_telemetry::recorder::NetworkStateSnapshot| -> f64 {
            let v: Vec<f64> = s.neurons.iter().filter(|n| n.id >= 24).map(|n| n.g_drive.unwrap_or(0.0) as f64).collect();
            let n = v.len().max(1) as f64;
            v.iter().sum::<f64>() / n
        };
        let (u_off, u_read) = (unorm(s_off), unorm(s_read));
        let (g_off, g_read) = (gmean(s_off), gmean(s_read));
        // top-10 u Jaccard at offset snapshot between arms is per-run; here report
        // within the run: top-10 neurons at offset snapshot (ids).
        let mut byu: Vec<(u32, f32)> = s_off.neurons.iter().filter(|n| n.id >= 24).map(|n| (n.id, n.u_slow.unwrap_or(0.0))).collect();
        byu.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let top10: Vec<u32> = byu.iter().take(10).map(|x| x.0).collect();
        // endogenous off-window stats: S2 silence = after S1's last presentation + 2000
        let sil_start = last_end + 2000;
        let end = spikes.last().map(|s| s.0 + 1).unwrap_or(sil_start);
        let sil: Vec<u64> = spikes.iter().filter(|(t, n)| *t >= sil_start && *n >= 24).map(|(t, _)| *t).collect();
        let sil_len_ms = end.saturating_sub(sil_start).max(1);
        let rate = sil.len() as f64 / (sil_len_ms as f64 / 1000.0);
        // Fano in 1s bins
        let nb = (sil_len_ms / 1000).max(1) as usize;
        let mut bins = vec![0.0f64; nb];
        for t in &sil { let b = ((t - sil_start) / 1000) as usize; if b < bins.len() { bins[b] += 1.0; } }
        let m = bins.iter().sum::<f64>() / bins.len() as f64;
        let fano = if m > 0.0 { bins.iter().map(|x| (x - m).powi(2)).sum::<f64>() / bins.len() as f64 / m } else { 0.0 };
        println!("{arm}\t{cur}\t{seed}\t{fail}\tc_dur={c_dur:.4}\tc_off={c_off:.4}\toff_mpc={off_mean_cos:.4}\toff_l2={md:.2}\tu_off={u_off:.2}\tu_read={u_read:.2}\tg_off={g_off:.5}\tg_read={g_read:.5}\ttop10={}\trate={rate:.1}\tfano={fano:.2}\tdur={}/{} off={}/{}",
            top10.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
            dur_count_a, dur_count_c, off_count_a, off_count_c);
    }
}