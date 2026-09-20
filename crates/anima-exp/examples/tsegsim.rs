//! READ-ONLY deterministic trace simulation of the proposed episode-buffer
//! architecture on committed A/C spike streams (docs/x-segarch.md §2).
//! Proposed equations, verbatim:
//!   on own spike AND Iaff20 > 0:  b += beta
//!   b *= exp(-dt/250 ms)                       (tau_b = 250 ms, S1 slow leg)
//!   u += rho * b per tick                      (rho = 0.05, frozen)
//!   u persists indefinitely (NO decay, per proposal)
//! Iaff20 = EMA(tau=20ms) of input-channel afferent current amp*w.
//! Output per run: TSV per episode end (t+500): ep ordinal, pattern, tick,
//! u-norm, b-norm, and full u/b vectors for paired analysis.
use std::collections::BTreeMap;

fn main() {
    for dir in std::env::args().skip(1) {
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
        // substrate constants from config (committed runs)
        let beta = match dir.contains("0.00625") { true => 0.00625f64, false => 0.0046875f64 };
        let beta = if dir.contains("0.003125") { 0.003125f64 } else { beta };
        const N: usize = 52;
        let idx_of = |nid: u32| -> Option<usize> { if nid >= 24 && nid < 76 { Some(nid as usize - 24) } else { None } };
        let tau_b = 250.0f64;
        let rho = 0.05f64;
        let d_b = (-1.0f64 / tau_b).exp();
        let d_ia = (-1.0f64 / 20.0).exp();
        let amp = 52.0f64;
        let mut b = vec![0.0f64; N];
        let mut u = vec![0.0f64; N];
        let mut ia20 = vec![0.0f64; N];
        let mut wmap: BTreeMap<(u32, u32), f64> = BTreeMap::new();
        let mut snap_i = 0usize;
        let by_tick: BTreeMap<u64, Vec<u32>> = {
            let mut m: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
            for &(t, nid) in &spikes { m.entry(t).or_default().push(nid); }
            m
        };
        let s1: Vec<(String, u64)> = pres.iter().filter(|(p, _)| p == "A" || p == "C").cloned().collect();
        let mut a_ord = 0usize;
        let mut c_ord = 0usize;
        println!("RUN {}", dir.rsplit('/').next().unwrap());
        println!("beta={beta} eps={} (A={} C={})", s1.len(), s1.iter().filter(|(p,_)| p=="A").count(), s1.iter().filter(|(p,_)| p=="C").count());
        let t_end = s1.last().map(|(_, t)| *t + 2500).unwrap_or(0);
        for t in 0..t_end {
            if t % 1000 == 0 {
                while snap_i + 1 < snaps.len() && snaps[snap_i + 1].tick <= t { snap_i += 1; }
                wmap.clear();
                for syn in &snaps[snap_i].synapses {
                    if syn.pre < 24 && syn.post >= 24 {
                        let w = syn.w.unwrap_or(0.0) as f64;
                        if w > 0.0 { wmap.insert((syn.pre, syn.post), w); }
                    }
                }
            }
            for i in 0..N {
                ia20[i] *= d_ia;
                b[i] *= d_b;
                u[i] += rho * b[i];
            }
            let tick_spks = by_tick.get(&t).cloned().unwrap_or_default();
            for nid in &tick_spks {
                for ((pre, post), w) in &wmap {
                    if *pre == *nid {
                        if let Some(j) = idx_of(*post) {
                            ia20[j] += (1.0 - d_ia) * amp * w;
                        }
                    }
                }
            }
            for nid in &tick_spks {
                if let Some(i) = idx_of(*nid) {
                    if ia20[i] > 0.0 { b[i] += beta; }
                }
            }
            // record at episode end
            for (p, pt) in &s1 {
                if t == pt + 500 {
                    let un = (u.iter().map(|x| x * x).sum::<f64>()).sqrt();
                    let bn = (b.iter().map(|x| x * x).sum::<f64>()).sqrt();
                    let ord = if p == "A" { a_ord; a_ord += 1; a_ord - 1 } else { c_ord; c_ord += 1; c_ord - 1 };
                    let uu: Vec<String> = u.iter().map(|x| format!("{x:.5}")).collect();
                    let bb: Vec<String> = b.iter().map(|x| format!("{x:.5}")).collect();
                    println!("E\t{ord}\t{p}\t{t}\t{un:.4}\t{bn:.4}\t{}\t{}", uu.join(","), bb.join(","));
                }
            }
        }
        println!();
    }
}