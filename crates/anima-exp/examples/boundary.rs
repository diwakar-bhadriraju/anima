//! V2.1 presentation-boundary local-signal analysis (READ-ONLY, committed
//! A/C spike stream). No runs, no implementation.
//!
//! Reconstructs per-neuron local state traces deterministically from the
//! recorded spike stream (exact substrate equations/constants):
//!   i_syn   : += amplitude*w per pre-spike (tau_syn=5 ms; weights from
//!             nearest-preceding snapshot; input & internal pres)
//!   i_adapt : += 0.05 per own spike (tau=200 ms)
//!   rate    : EMA toward 1000 on own spike (alpha=1/1000)
//!   u       : += beta per own spike (tau=10 s)
//!   spk     : own spike count this tick
//! Reports:
//!   1. population mean rate per alignment bucket (onset/rise/early/steady/
//!      late/offt/post1/post2) for A and C.
//!   2. per-neuron boundary d' for each signal: "onset [0,20ms)" vs
//!      "steady [100,250ms)" over presentations (pooled variance):
//!      median d' over neurons + count of neurons with |d'|>1.5.
//!   3. dilution curve: cos(A,C) of {windowed count, cumulative count, u}
//!      after k in {1,2,5,10,20,40} presentations of each pattern.
use std::collections::BTreeMap;

fn main() {
    for dir in std::env::args().skip(1) {
        let name = dir.rsplit('/').next().unwrap().to_string();
        let tdir = std::path::Path::new(&dir).join("telemetry");
        let reader = match anima_telemetry::TelemetryReader::open(&tdir) {
            Ok(r) => r, Err(_) => { continue; }
        };
        let idx = reader.chunk_index();
        let mut pres: Vec<(String, u64)> = Vec::new();
        let mut spikes: Vec<(u64, u32)> = Vec::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                match row.kind {
                    5 => { if let Some(env) = row.envelope("e").ok() {
                        if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } = env.payload {
                            if stage == "S1" { pres.push((pattern_id, env.t)); } } } }
                    3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                    _ => {}
                }
            }
        }
        pres.sort_by_key(|p| p.1);
        let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        if snaps.is_empty() { continue; }
        const N: usize = 52;
        let idx_of = |nid: u32| -> Option<usize> { if nid >= 24 && nid < 76 { Some(nid as usize - 24) } else { None } };
        let d_adapt = (-1.0f64 / 200.0).exp();
        let d_syn = (-1.0f64 / 5.0).exp();
        let d_u = (-1.0f64 / 10000.0).exp();
        let alpha = 1.0f64 / 1000.0;
        let amp = 52.0f64;

        // maps: presentation-covering-window per tick; presentation ordinal per (pattern,tick)
        let s1: Vec<(String, u64)> = pres.iter().filter(|(p, _)| p == "A" || p == "C").cloned().collect();
        let mut ord: BTreeMap<u64, usize> = BTreeMap::new(); // tick -> presentation ordinal (0-based in s1)
        let mut cover: BTreeMap<u64, usize> = BTreeMap::new(); // tick -> ordinal of covering presentation (rel<2000)
        for (i, (_, t)) in s1.iter().enumerate() {
            ord.insert(*t, i);
            for dt in 0..2000i64 { cover.insert(*t + dt as u64, i); }
        }
        let by_tick: BTreeMap<u64, Vec<u32>> = {
            let mut m: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
            for &(t, nid) in &spikes { m.entry(t).or_default().push(nid); }
            m
        };
        // state
        let mut i_syn = vec![0.0f64; N];
        let mut i_adapt = vec![0.0f64; N];
        let mut rate = vec![0.0f64; N];
        let mut u = vec![0.0f64; N];
        let mut spk = vec![0.0f64; N];
        let mut cum = vec![0.0f64; N];
        let mut wmap: BTreeMap<(u32, u32), f64> = BTreeMap::new();
        let mut snap_i = 0usize;
        // accumulators: (pres_ord, bucket, signal, neuron) -> (ticks, sum)
        let mut acc: BTreeMap<(usize, usize, usize, usize), (u64, f64)> = BTreeMap::new();
        const SIGNALS: [&str; 5] = ["rate", "i_adapt", "i_syn", "u", "spk"];
        let BUCKETS: [(&str, i64, i64); 8] = [
            ("onset", 0, 20), ("rise", 20, 50), ("early", 50, 100), ("steady", 100, 250),
            ("late", 250, 500), ("offt", 500, 550), ("post1", 550, 1000), ("post2", 1000, 2000),
        ];
        // windowed/cumulative/u per presentation (in s1 order)
        let mut win: Vec<Vec<f64>> = vec![vec![0.0; N]; s1.len()];
        let mut cu: Vec<Vec<f64>> = vec![vec![0.0; N]; s1.len()];
        let mut usnap: Vec<Vec<f64>> = vec![vec![0.0; N]; s1.len()];
        let t_end = s1.last().map(|(_, t)| *t + 2500).unwrap_or(0);
        for t in 0..t_end {
            if t % 1000 == 0 {
                while snap_i + 1 < snaps.len() && snaps[snap_i + 1].tick <= t { snap_i += 1; }
                wmap.clear();
                for syn in &snaps[snap_i].synapses {
                    if syn.pre < 24 && syn.post >= 24 {
                        wmap.insert((syn.pre, syn.post), syn.w.unwrap_or(0.0) as f64);
                    }
                }
            }
            for i in 0..N {
                i_syn[i] *= d_syn;
                i_adapt[i] *= d_adapt;
                u[i] *= d_u;
                rate[i] *= (1.0 - alpha);
                spk[i] = 0.0;
            }
            let tick_spks = by_tick.get(&t).cloned().unwrap_or_default();
            for nid in &tick_spks {
                if let Some(i) = idx_of(*nid) {
                    i_adapt[i] += 0.05;
                    u[i] += 0.003125;
                    cum[i] += 1.0;
                    rate[i] += alpha * 1000.0;
                    spk[i] = 1.0;
                }
                for ((pre, post), w) in &wmap {
                    if *pre == *nid {
                        if let Some(j) = idx_of(*post) { i_syn[j] += amp * w; }
                    }
                }
            }
            // bucket accumulation + windowed/cumulative/u snapshots
            if let Some(&pi) = cover.get(&t) {
                let rel = t as i64 - s1[pi].1 as i64;
                if rel >= 0 && rel < 2000 {
                    for (bi, (_, b0, b1)) in BUCKETS.iter().enumerate() {
                        if rel >= *b0 && rel < *b1 {
                            for i in 0..N {
                                let vals = [rate[i], i_adapt[i], i_syn[i], u[i], spk[i]];
                                for (si, v) in vals.iter().enumerate() {
                                    let e = acc.entry((pi, bi, si, i)).or_insert((0, 0.0));
                                    e.0 += 1;
                                    e.1 += v;
                                }
                            }
                            break;
                        }
                    }
                }
            }
            if let Some(&pi) = cover.get(&t) {
                let pt = s1[pi].1;
                if t == pt + 500 {
                    // windowed count spans [pt, pt+500): recompute from spikes
                    let mut wc = vec![0.0f64; N];
                    for &(ts, nid) in &spikes {
                        if ts >= pt && ts < pt + 500 {
                            if let Some(i) = idx_of(nid) { wc[i] += 1.0; }
                        }
                    }
                    win[pi] = wc;
                    cu[pi] = cum.clone();
                    usnap[pi] = u.clone();
                }
            }
        }
        // ---- 1. population rate per bucket (A vs C) ----
        let mut pop_a = vec![0.0f64; 8];
        let mut pop_c = vec![0.0f64; 8];
        let mut n_a = vec![0u64; 8];
        let mut n_c = vec![0u64; 8];
        for (pi, (p, _)) in s1.iter().enumerate() {
            for (bi, _) in BUCKETS.iter().enumerate() {
                let mut sum = 0.0f64;
                for i in 0..N {
                    if let Some((c, s)) = acc.get(&(pi, bi, 0, i)) {
                        if *c > 0 { sum += s / *c as f64; }
                    }
                }
                if p == "A" { pop_a[bi] += sum; n_a[bi] += 1; } else { pop_c[bi] += sum; n_c[bi] += 1; }
            }
        }
        println!("RUN {name}");
        println!("1. population mean rate (sum over neurons, Hz-ish):");
        for (bi, (bname, _, _)) in BUCKETS.iter().enumerate() {
            println!("   {bname:<7}: A={:.3}  C={:.3}", pop_a[bi] / n_a[bi].max(1) as f64, pop_c[bi] / n_c[bi].max(1) as f64);
        }
        // ---- 2. per-neuron boundary d' per signal: onset vs steady ----
        println!("2. boundary d' (onset [0,20) vs steady [100,250)), per signal:");
        for (si, sg) in SIGNALS.iter().enumerate() {
            let mut ds: Vec<f64> = Vec::new();
            for i in 0..N {
                let mut o = Vec::new();
                let mut st = Vec::new();
                for (pi, (_, _)) in s1.iter().enumerate() {
                    if let Some((c, s)) = acc.get(&(pi, 0, si, i)) {
                        if *c > 0 { o.push(s / *c as f64); }
                    }
                    if let Some((c, s)) = acc.get(&(pi, 3, si, i)) {
                        if *c > 0 { st.push(s / *c as f64); }
                    }
                }
                if o.len() >= 4 && st.len() >= 4 {
                    let mo = o.iter().sum::<f64>() / o.len() as f64;
                    let ms = st.iter().sum::<f64>() / st.len() as f64;
                    let vo = o.iter().map(|x| (x - mo).powi(2)).sum::<f64>() / o.len() as f64;
                    let vs = st.iter().map(|x| (x - ms).powi(2)).sum::<f64>() / st.len() as f64;
                    let denom = ((vo + vs) / 2.0).sqrt();
                    if denom > 1e-12 { ds.push((mo - ms) / denom); }
                }
            }
            if !ds.is_empty() {
                ds.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let med = ds[ds.len() / 2];
                let n1 = ds.iter().filter(|d| d.abs() > 1.5).count();
                println!("   {sg:<8}: median d'={med:+.2}  |d'|>1.5 in {n1}/{N} neurons");
            }
        }
        // ---- 3. dilution curve ----
        let cos = |a: &[f64], b: &[f64]| -> f64 {
            let d: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
            let na = a.iter().map(|x| x * x).sum::<f64>().sqrt();
            let nb = b.iter().map(|x| x * x).sum::<f64>().sqrt();
            if na > 0.0 && nb > 0.0 { d / (na * nb) } else { 0.0 }
        };
        let idx_a: Vec<usize> = (0..s1.len()).filter(|&i| s1[i].0 == "A").collect();
        let idx_c: Vec<usize> = (0..s1.len()).filter(|&i| s1[i].0 == "C").collect();
        println!("3. dilution cos(A,C) at k presentations of each pattern:");
        println!("   k      windowed   cumulative   u");
        for k in [1usize, 2, 5, 10, 20, 40] {
            let (ka, kc) = (idx_a.len().min(k), idx_c.len().min(k));
            let (mut wa, mut wc_) = (vec![0.0f64; N], vec![0.0f64; N]);
            let (mut ca, mut cc) = (vec![0.0f64; N], vec![0.0f64; N]);
            let (mut ua, mut uc) = (vec![0.0f64; N], vec![0.0f64; N]);
            for i in 0..ka {
                for j in 0..N { wa[j] += win[idx_a[i]][j]; ca[j] += cu[idx_a[i]][j]; ua[j] += usnap[idx_a[i]][j]; }
            }
            for i in 0..kc {
                for j in 0..N { wc_[j] += win[idx_c[i]][j]; cc[j] += cu[idx_c[i]][j]; uc[j] += usnap[idx_c[i]][j]; }
            }
            for j in 0..N { wa[j] /= ka.max(1) as f64; wc_[j] /= kc.max(1) as f64; }
            println!("   {k:>2}     {:.4}     {:.4}       {:.4}", cos(&wa, &wc_), cos(&ca, &cc), cos(&ua, &uc));
        }
        println!();
    }
}