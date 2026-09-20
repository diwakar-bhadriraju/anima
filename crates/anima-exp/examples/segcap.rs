//! V2.1 local segmentation-candidate capability analysis (READ-ONLY).
//! Reconstructs per-neuron candidate local signals from the committed A/C
//! spike stream (exact substrate constants, weights from snapshots):
//!   I_aff     : input-channel afferent current (amp*w per input spike;
//!               EXACTLY ZERO in silence by construction — no input spikes)
//!   fast/slow : EMA of I_aff at tau in {5,20,100,250} ms
//!   S1        : fast(20ms) - slow(250ms)  [drive-contrast diff]
//!   i_syn     : total synaptic current (tau 5ms)
//!   S2        : d(i_syn)/dt per tick      [synaptic transient]
//!   i_adapt   : adaptation current (0.05/spike, tau 200ms)
//!   S3        : I_aff(20ms) vs i_adapt contrast (both local; reported as
//!               scaled z at each bucket — no new gain, raw units)
//! For each signal: bucket means (onset [0,50), rise [50,100), steady
//! [100,250), late [250,500), offt [500,550), post1 [550,1000), post2
//! [1000,2000)) pooled over presentations; median-neuron d' onset-vs-
//! steady and offt-vs-post1 (boundary detectability per neuron); and the
//! OFF-WINDOW false-positive check (max |signal| during silence).
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
        let lam = |tau: f64| (-1.0f64 / tau).exp();
        let amp = 52.0f64;
        // state
        let mut ia_f5 = vec![0.0f64; N];
        let mut ia_f20 = vec![0.0f64; N];
        let mut ia_s100 = vec![0.0f64; N];
        let mut ia_s250 = vec![0.0f64; N];
        let mut i_syn = vec![0.0f64; N];
        let mut i_adapt = vec![0.0f64; N];
        let mut isyn_prev = vec![0.0f64; N];
        let mut wmap: BTreeMap<(u32, u32), f64> = BTreeMap::new();
        let mut snap_i = 0usize;
        let by_tick: BTreeMap<u64, Vec<u32>> = {
            let mut m: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
            for &(t, nid) in &spikes { m.entry(t).or_default().push(nid); }
            m
        };
        // accumulators: (pres_idx, bucket, signal) -> (ticks, sum); also per-neuron sums
        // signals: 0=S1diff(20-250) 1=S2disyn 2=i_syn 3=i_adapt 4=Iaff20 5=Iaff5
        let mut acc: BTreeMap<(usize, usize, usize), (u64, f64)> = BTreeMap::new(); // pop sums
        let mut accn: BTreeMap<(usize, usize, usize, usize), (u64, f64)> = BTreeMap::new(); // per neuron
        let BUCKETS: [(&str, i64, i64); 8] = [
            ("onset0", 0, 10), ("onset10", 10, 50), ("rise", 50, 100), ("steady", 100, 250),
            ("late", 250, 500), ("offt", 500, 550), ("post1", 550, 1000), ("post2", 1000, 2000),
        ];
        // cover map tick -> pres idx
        let s1: Vec<(String, u64)> = pres.iter().filter(|(p, _)| p == "A" || p == "C").cloned().collect();
        let mut cover: BTreeMap<u64, usize> = BTreeMap::new();
        for (i, (_, t)) in s1.iter().enumerate() {
            for dt in 0..2000i64 { cover.insert(*t + dt as u64, i); }
        }
        // off-window: S2 after S1 (t >= last pres end + 2000)
        let sil_start = s1.last().map(|(_, t)| *t + 2500).unwrap_or(0);
        let t_end = sil_start + 20000;
        // per-neuron silence maxima (false-boundary check)
        let mut sil_max = vec![0.0f64; 6];
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
                ia_f5[i] = lam(5.0) * ia_f5[i];
                ia_f20[i] = lam(20.0) * ia_f20[i];
                ia_s100[i] = lam(100.0) * ia_s100[i];
                ia_s250[i] = lam(250.0) * ia_s250[i];
                i_syn[i] = lam(5.0) * i_syn[i];
                i_adapt[i] = lam(200.0) * i_adapt[i];
            }
            let tick_spks = by_tick.get(&t).cloned().unwrap_or_default();
            for nid in &tick_spks {
                if let Some(i) = idx_of(*nid) {
                    i_adapt[i] += 0.05;
                }
                // ALL spikers (input + internal) deposit into i_syn (substrate semantics)
                for ((pre, post), w) in &wmap {
                    if *pre == *nid {
                        if let Some(j) = idx_of(*post) { i_syn[j] += amp * w; }
                    }
                }
                // input afferents (pre < 24) additionally feed I_aff EMAs
                if *nid < 24 {
                    for ((pre, post), w) in &wmap {
                        if *pre == *nid {
                            if let Some(j) = idx_of(*post) {
                                let d = amp * w;
                                ia_f5[j] += (1.0 - lam(5.0)) * d;
                                ia_f20[j] += (1.0 - lam(20.0)) * d;
                                ia_s100[j] += (1.0 - lam(100.0)) * d;
                                ia_s250[j] += (1.0 - lam(250.0)) * d;
                            }
                        }
                    }
                }
            }
            let sigs = [
                ia_f20.iter().zip(&ia_s250).map(|(f, s)| f - s).collect::<Vec<_>>(),
                i_syn.iter().zip(&isyn_prev).map(|(c, p)| c - p).collect::<Vec<_>>(),
                i_syn.clone(),
                i_adapt.clone(),
                ia_f20.clone(),
                ia_f5.clone(),
            ];
            for i in 0..N { isyn_prev[i] = i_syn[i]; }
            if let Some(&pi) = cover.get(&t) {
                let rel = t as i64 - s1[pi].1 as i64;
                for (bi, (_, b0, b1)) in BUCKETS.iter().enumerate() {
                    if rel >= *b0 && rel < *b1 {
                        for (si, v) in sigs.iter().enumerate() {
                            let e = acc.entry((pi, bi, si)).or_insert((0, 0.0));
                            e.0 += 1; e.1 += v.iter().sum::<f64>();
                            for i in 0..N {
                                let e2 = accn.entry((pi, bi, si, i)).or_insert((0, 0.0));
                                e2.0 += 1; e2.1 += v[i];
                            }
                        }
                        break;
                    }
                }
            }
            // off-window false-positive check: per-neuron max |S1| and |S2|
            if t >= sil_start && t < sil_start + 20000 {
                for i in 0..N {
                    if sigs[0][i].abs() > sil_max[0] { sil_max[0] = sigs[0][i].abs(); }
                    if sigs[1][i].abs() > sil_max[1] { sil_max[1] = sigs[1][i].abs(); }
                    if sigs[4][i] > sil_max[4] { sil_max[4] = sigs[4][i]; }
                }
            }
        }
        println!("RUN {name}");
        let names = ["S1 diff20-250", "S2 disyn", "i_syn", "i_adapt", "Iaff20", "Iaff5"];
        println!("1. population bucket means (sum over 52 neurons, per tick):");
        print!("   bucket    ");
        for n in names { print!("{:>10}", n); }
        println!();
        for (bi, (bname, _, _)) in BUCKETS.iter().enumerate() {
            print!("   {bname:<10}",);
            for si in 0..6 {
                let mut tot = 0.0f64;
                let mut cnt = 0u64;
                for (pi, _) in s1.iter().enumerate() {
                    if let Some((c, s)) = acc.get(&(pi, bi, si)) {
                        tot += s; cnt += c;
                    }
                }
                let tot_n = s1.len() as f64;
                print!("{:>10.4}", if cnt > 0 { tot / cnt as f64 / tot_n } else { 0.0 });
            }
            println!();
        }
        println!("2. median-neuron |d'| onset[0,50)vs steady[100,250) and offt[500,550)vs post1[550,1000):");
        for si in 0..6 {
            let mut d1 = Vec::new();
            let mut d2 = Vec::new();
            for i in 0..N {
                let mut bucket_vals = |b: usize| -> Vec<f64> {
                    let mut v = Vec::new();
                    for (pi, _) in s1.iter().enumerate() {
                        if let Some((c, s)) = accn.get(&(pi, b, si, i)) {
                            if *c > 0 { v.push(s / *c as f64); }
                        }
                    }
                    v
                };
                let o = bucket_vals(0); let st = bucket_vals(2);
                if o.len() >= 4 && st.len() >= 4 {
                    let mo = o.iter().sum::<f64>() / o.len() as f64;
                    let ms = st.iter().sum::<f64>() / st.len() as f64;
                    let vo = o.iter().map(|x| (x - mo).powi(2)).sum::<f64>() / o.len() as f64;
                    let vs = st.iter().map(|x| (x - ms).powi(2)).sum::<f64>() / st.len() as f64;
                    let den = ((vo + vs) / 2.0).sqrt();
                    if den > 1e-12 { d1.push((mo - ms) / den); }
                }
                let of = bucket_vals(4); let po = bucket_vals(5);
                if of.len() >= 4 && po.len() >= 4 {
                    let mo = of.iter().sum::<f64>() / of.len() as f64;
                    let ms = po.iter().sum::<f64>() / po.len() as f64;
                    let vo = of.iter().map(|x| (x - mo).powi(2)).sum::<f64>() / of.len() as f64;
                    let vs = po.iter().map(|x| (x - ms).powi(2)).sum::<f64>() / po.len() as f64;
                    let den = ((vo + vs) / 2.0).sqrt();
                    if den > 1e-12 { d2.push((mo - ms) / den); }
                }
            }
            let med = |v: &Vec<f64>| { let mut x = v.clone(); x.sort_by(|a, b| a.partial_cmp(b).unwrap()); x[x.len() / 2] };
            println!("   {:<12}: onset-vs-steady median d' = {:.2}   offt-vs-post1 median d' = {:.2}", names[si],
                if !d1.is_empty() { med(&d1) } else { f64::NAN },
                if !d2.is_empty() { med(&d2) } else { f64::NAN });
        }
        println!("3. off-window (20 s silence) false-boundary maxima (per-neuron max |signal|):");
        println!("   S1 diff20-250 max|x| = {:.4}   S2 disyn max|.| = {:.4}   Iaff20 max = {:.4}",
            sil_max[0], sil_max[1], sil_max[4]);
        println!();
    }
}