//! V2.1 boundary-detector benchmark (READ-ONLY, committed runs).
//! Detectors and thresholds DECLARED BEFORE inspection (per x-segcap
//! scale; no post-hoc optimization):
//!   S1        = Iaff20 - Iaff250  (input-channel afferent EMA diff)
//!   onset     : first tick in [t, t+500) with S1 > +0.0020  (=10x recorded
//!               silence floor 0.0002, x-segcap section 1)
//!   offset    : first tick in [t+500, t+2000) with S1 < -0.0020
//!   i_syn     : total synaptic current (reconstructed; tau 5ms)
//!   offset    : first tick in [t+450, t+2000) with i_syn < 0.0010
//!               (10x recorded zero-floor at nominal resolution)
//! FP check: crossings (rise-through for S1-on, fall-through for S1-off and
//! i_syn-off) during the 20 s post-S1 silence, per neuron.
//! Outputs per run: detector-capable neurons (>=1 input synapse), per-
//! presentation TP rate over capable neurons, latency distributions
//! (median/p90 over (neuron,pres) events), FP counts per neuron per 20 s
//! silence, fraction of neurons with >=1 FP, plus mid-presentation extra
//! crossings (i_syn dips) reported separately.
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
        let TH_S1 = 0.0020f64;
        let TH_ISYN = 0.0010f64;
        let mut ia_f20 = vec![0.0f64; N];
        let mut ia_s250 = vec![0.0f64; N];
        let mut i_syn = vec![0.0f64; N]; // input-only (matches x-segcap definition)
        let mut i_syn_full = vec![0.0f64; N]; // all afferents (recurrent upper bound)
        let mut has_input = vec![false; N];
        let mut wmap: BTreeMap<(u32, u32), f64> = BTreeMap::new(); // input afferents (S1, i_syn_in)
        let mut wmap_rec: BTreeMap<(u32, u32), f64> = BTreeMap::new(); // ALL excitatory (i_syn_full, upper bound)
        let mut is_inhib: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new(); // unknown from snapshots; none flagged
        let mut snap_i = 0usize;
        let by_tick: BTreeMap<u64, Vec<u32>> = {
            let mut m: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
            for &(t, nid) in &spikes { m.entry(t).or_default().push(nid); }
            m
        };
        let s1: Vec<(String, u64)> = pres.iter().filter(|(p, _)| p == "A" || p == "C").cloned().collect();
        let mut cover: BTreeMap<u64, usize> = BTreeMap::new();
        for (i, (_, t)) in s1.iter().enumerate() {
            for dt in 0..2500i64 { cover.insert(*t + dt as u64, i); }
        }
        let sil_start = s1.last().map(|(_, t)| *t + 2500).unwrap_or(0);
        let t_end = sil_start + 20000;
        // detector state: previous tick values for crossing detection
        let mut prev_s1 = vec![0.0f64; N];
        let mut prev_isyn = vec![0.0f64; N];
        let mut prev_isfull = vec![0.0f64; N];
        // per presentation: first-crossing latency per neuron (None if missed)
        let mut lat_on: Vec<Vec<Option<u64>>> = vec![vec![None; N]; s1.len()];
        let mut lat_off: Vec<Vec<Option<u64>>> = vec![vec![None; N]; s1.len()];
        let mut lat_isyn: Vec<Vec<Option<u64>>> = vec![vec![None; N]; s1.len()];
        // FP counters during silence
        let mut fp_s1_on = vec![0u64; N];
        let mut fp_s1_off = vec![0u64; N];
        let mut fp_isyn = vec![0u64; N];
        // mid-presentation extra crossings (i_syn dips below threshold and
        // comes back, within [t, t+500)): count fall-throughs after first
        let mut mid_isyn = vec![0u64; N];
        let mut fp_isyn_full = vec![0u64; N];
        for t in 0..t_end {
            if t % 1000 == 0 {
                while snap_i + 1 < snaps.len() && snaps[snap_i + 1].tick <= t { snap_i += 1; }
                wmap.clear();
                wmap_rec.clear();
                for syn in &snaps[snap_i].synapses {
                    if syn.pre < 24 && syn.post >= 24 {
                        let w = syn.w.unwrap_or(0.0) as f64;
                        if w > 0.0 {
                            wmap.insert((syn.pre, syn.post), w);
                            has_input[syn.post as usize - 24] = true;
                        }
                    }
                    if syn.post >= 24 && !is_inhib.contains(&syn.id) {
                        wmap_rec.insert((syn.pre, syn.post), syn.w.unwrap_or(0.0) as f64);
                    }
                }
            }
            for i in 0..N {
                ia_f20[i] *= lam(20.0);
                ia_s250[i] *= lam(250.0);
                i_syn[i] *= lam(5.0);
                i_syn_full[i] *= lam(5.0);
            }
            let tick_spks = by_tick.get(&t).cloned().unwrap_or_default();
            for nid in &tick_spks {
                // all spikers deposit i_syn (substrate semantics)
                for ((pre, post), w) in &wmap {
                    if *pre == *nid {
                        if let Some(j) = idx_of(*post) { i_syn[j] += amp * w; }
                    }
                }
                // full i_syn: deposits from ALL presynapses (incl. recurrent)
                for ((pre, post), w) in &wmap_rec {
                    if *pre == *nid {
                        if let Some(j) = idx_of(*post) { i_syn_full[j] += amp * w; }
                    }
                }
                if *nid < 24 {
                    for ((pre, post), w) in &wmap {
                        if *pre == *nid {
                            if let Some(j) = idx_of(*post) {
                                let d = amp * w;
                                ia_f20[j] += (1.0 - lam(20.0)) * d;
                                ia_s250[j] += (1.0 - lam(250.0)) * d;
                            }
                        }
                    }
                }
            }
            let s1v: Vec<f64> = ia_f20.iter().zip(&ia_s250).map(|(f, s)| f - s).collect();
            // presentation windows
            if let Some(&pi) = cover.get(&t) {
                let pt = s1[pi].1;
                let rel = t as i64 - pt as i64;
                if rel >= 0 && rel < 500 {
                    for i in 0..N {
                        if lat_on[pi][i].is_none() && s1v[i] > TH_S1 {
                            lat_on[pi][i] = Some((rel) as u64);
                        }
                        if i_syn[i] < TH_ISYN && prev_isyn[i] >= TH_ISYN && lat_isyn[pi][i].is_none() {
                            // fall-through during presentation = mid dip
                            mid_isyn[i] += 1;
                        }
                    }
                }
                if rel >= 500 && rel < 2000 {
                    for i in 0..N {
                        if lat_off[pi][i].is_none() && s1v[i] < -TH_S1 {
                            lat_off[pi][i] = Some((rel - 500) as u64);
                        }
                        if lat_isyn[pi][i].is_none() && i_syn[i] < TH_ISYN && prev_isyn[i] >= TH_ISYN {
                            lat_isyn[pi][i] = Some((rel as i64 - 450).max(0) as u64);
                        }
                    }
                }
                if rel >= 450 && rel < 500 {
                    // pre-offset window also allowed for i_syn detection start
                    for i in 0..N {
                        if lat_isyn[pi][i].is_none() && i_syn[i] < TH_ISYN && prev_isyn[i] >= TH_ISYN {
                            lat_isyn[pi][i] = Some(0u64);
                        }
                    }
                }
            }
            // silence FP: crossings
            if t >= sil_start {
                for i in 0..N {
                    if prev_s1[i] <= TH_S1 && s1v[i] > TH_S1 { fp_s1_on[i] += 1; }
                    if prev_s1[i] >= -TH_S1 && s1v[i] < -TH_S1 { fp_s1_off[i] += 1; }
                    if prev_isyn[i] >= TH_ISYN && i_syn[i] < TH_ISYN { fp_isyn[i] += 1; }
                    if prev_isfull[i] >= TH_ISYN && i_syn_full[i] < TH_ISYN { fp_isyn_full[i] += 1; }
                }
            }
            for i in 0..N { prev_s1[i] = s1v[i]; prev_isyn[i] = i_syn[i]; prev_isfull[i] = i_syn_full[i]; }
        }
        // ---- aggregate ----
        let capable: Vec<usize> = (0..N).filter(|&i| has_input[i]).collect();
        let med = |v: &mut Vec<u64>| -> Option<u64> { if v.is_empty() { None } else { v.sort(); Some(v[v.len() / 2]) } };
        // S1 onset: TP over capable neurons; latency events
        let mut on_tp = 0u64;
        let mut on_lat = Vec::new();
        for pi in 0..s1.len() {
            for &i in &capable {
                if let Some(l) = lat_on[pi][i] { on_tp += 1; on_lat.push(l); }
            }
        }
        let mut off_tp = 0u64;
        let mut off_lat = Vec::new();
        for pi in 0..s1.len() {
            for &i in &capable {
                if let Some(l) = lat_off[pi][i] { off_tp += 1; off_lat.push(l); }
            }
        }
        let mut isyn_tp = 0u64;
        let mut isyn_lat = Vec::new();
        for pi in 0..s1.len() {
            for &i in &capable {
                if let Some(l) = lat_isyn[pi][i] { isyn_tp += 1; isyn_lat.push(l); }
            }
        }
        let tot = s1.len() as u64 * capable.len() as u64;
        let p90 = |v: &Vec<u64>| -> Option<u64> { if v.is_empty() { None } else { Some(v[(v.len() * 9) / 10]) } };
        println!("RUN {name}");
        println!("  detector-capable neurons (>=1 input afferent): {}/{}", capable.len(), N);
        println!("  presentations: {}", s1.len());
        println!("---- S1 onset  (S1 > +{TH_S1}) ----");
        println!("  TP rate (over neuron-presentation pairs): {:.3}%", 100.0 * on_tp as f64 / tot as f64);
        let mut ol = on_lat.clone();
        println!("  latency ms: median {:?} p90 {:?} n_events {}", med(&mut ol), p90(&ol), ol.len());
        println!("  FP in 20s silence: total {} (mean {:.3}/neuron), neurons with >=1 FP: {}/{}",
            fp_s1_on.iter().sum::<u64>(), fp_s1_on.iter().sum::<u64>() as f64 / N as f64,
            fp_s1_on.iter().filter(|&&x| x > 0).count(), N);
        println!("---- S1 offset (S1 < -{TH_S1}) ----");
        println!("  TP rate: {:.3}%", 100.0 * off_tp as f64 / tot as f64);
        let mut ol2 = off_lat.clone();
        println!("  latency from offset ms: median {:?} p90 {:?} n {}", med(&mut ol2), p90(&ol2), ol2.len());
        println!("  FP in 20s silence: total {} (mean {:.3}/neuron), neurons >=1 FP: {}/{}",
            fp_s1_off.iter().sum::<u64>(), fp_s1_off.iter().sum::<u64>() as f64 / N as f64,
            fp_s1_off.iter().filter(|&&x| x > 0).count(), N);
        println!("---- i_syn offset (i_syn < {TH_ISYN}) ----");
        println!("  TP rate: {:.3}%", 100.0 * isyn_tp as f64 / tot as f64);
        let mut il = isyn_lat.clone();
        println!("  latency from offset ms: median {:?} p90 {:?} n {}", med(&mut il), p90(&il), il.len());
        println!("  FP in 20s silence: total {} (mean {:.3}/neuron), neurons >=1 FP: {}/{}",
            fp_isyn.iter().sum::<u64>(), fp_isyn.iter().sum::<u64>() as f64 / N as f64,
            fp_isyn.iter().filter(|&&x| x > 0).count(), N);
        println!("  mid-presentation extra dips (fall-throughs during [t,t+500)): total {} (mean {:.2}/neuron)",
            mid_isyn.iter().sum::<u64>(), mid_isyn.iter().sum::<u64>() as f64 / N as f64);
        println!("  FULL-i_syn (incl. recurrent, upper bound) FP in 20s silence: total {} , neurons >=1 FP: {}/{}",
            fp_isyn_full.iter().sum::<u64>(), fp_isyn_full.iter().filter(|&&x| x > 0).count(), N);
        println!();
    }
}