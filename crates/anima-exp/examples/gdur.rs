//! X-series: empirical g_drive replay (read-only). Deterministically
//! reconstructs the network's g_drive EMA from recorded input spikes +
//! nearest-preceding snapshot weights. Reports mean g during presentation
//! windows, during off-windows, at last-offset, at read, and the
//! distribution across neurons during drive.
use std::collections::BTreeMap;

fn main() {
    for dir in std::env::args().skip(1) {
        let name = dir.rsplit('/').next().unwrap();
        let tdir = std::path::Path::new(&dir).join("telemetry");
        let reader = match anima_telemetry::TelemetryReader::open(&tdir) {
            Ok(r) => r,
            Err(_) => { continue; }
        };
        let idx = reader.chunk_index();
        let mut inp: Vec<(u64, u32)> = Vec::new();
        let mut pres: Vec<(String, u64)> = Vec::new();
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
                    3 => { if let Some(n) = row.n { if n < 24 { inp.push((row.t, n as u32)); } } }
                    _ => {}
                }
            }
        }
        pres.sort_by_key(|p| p.1);
        inp.sort_by_key(|p| p.0);
        if inp.is_empty() { println!("{name}\tno-input"); continue; }
        let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let snap_times: Vec<u64> = snaps.iter().map(|s| s.tick).collect();
        // weight map (pre,post) -> w at nearest snapshot <= t
        let w_at = |t: u64| -> BTreeMap<(u32, u32), f32> {
            let st = snap_times.iter().rev().find(|&&st| st <= t).copied().unwrap_or(snap_times[0]);
            let s = snaps.iter().find(|s| s.tick == st).unwrap();
            let mut m = BTreeMap::new();
            for syn in &s.synapses {
                if syn.pre < 24 && syn.post >= 24 {
                    m.insert((syn.pre, syn.post), syn.w.unwrap_or(0.0));
                }
            }
            m
        };
        let amp = 52.0f32;
        let vth = 1.0f32;
        let lam = (-1.0f32 / 20.0).exp();
        let n_int = 52usize;
        let mut g = vec![0.0f32; n_int];
        let (mut sum_on, mut n_on) = (0.0f64, 0u64);
        let (mut sum_off, mut n_off) = (0.0f64, 0u64);
        let mut g_during: Vec<f32> = Vec::new();
        let t_last = pres.last().unwrap().1;
        let mut wmap = w_at(pres[0].1);
        let mut last_refresh = u64::MAX;
        let mut g_offset = 0.0f32;
        let mut g_read = 0.0f32;
        let mut inp_i = 0usize;
        let t_end = t_last + 2000 + 1600;
        for t in 0..t_end {
            // refresh weights at snapshot cadence
            if t % 200 == 0 {
                let st = snap_times.iter().rev().find(|&&st| st <= t).copied().unwrap_or(snap_times[0]);
                if st != last_refresh {
                    wmap = w_at(t);
                    last_refresh = st;
                }
            }
            // x from input spikes at this tick
            let mut x = vec![0.0f32; n_int];
            while inp_i < inp.len() && inp[inp_i].0 <= t {
                if inp[inp_i].0 == t {
                    let pre = inp[inp_i].1;
                    for ((p, post), w) in &wmap {
                        if *p == pre {
                            let idx = *post as usize - 24;
                            if idx < n_int { x[idx] += amp * w; }
                        }
                    }
                }
                inp_i += 1;
            }
            for xi in x.iter_mut() { *xi = (*xi / vth).min(1.0); }
            for k in 0..n_int { g[k] = lam * g[k] + (1.0 - lam) * x[k]; }
            let in_win = pres.iter().any(|(_, pt)| *pt <= t && t < *pt + 500);
            let mean_g = g.iter().sum::<f32>() as f64 / n_int as f64;
            if in_win {
                sum_on += mean_g; n_on += 1;
                g_during.extend_from_slice(&g);
            } else if t > pres[0].1 && t < t_last {
                sum_off += mean_g; n_off += 1;
            }
            if t == t_last + 500 { g_offset = g.iter().sum::<f32>() / n_int as f32; }
            if t == t_last + 500 + 1500 { g_read = g.iter().sum::<f32>() / n_int as f32; }
        }
        let mut sorted = g_during.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let (med, p90) = if sorted.is_empty() { (0.0, 0.0) } else {
            (sorted[sorted.len() / 2], sorted[(sorted.len() * 9) / 10])
        };
        println!("{name}\tg_on={:.4} (n={n_on})\tg_off={:.4} (n={n_off})\tg_offset={:.6}\tg_read={:.6}\tmed_on={:.4}\tp90_on={:.4}",
            sum_on / n_on.max(1) as f64,
            if n_off > 0 { sum_off / n_off as f64 } else { f64::NAN },
            g_offset, g_read, med, p90);
    }
}