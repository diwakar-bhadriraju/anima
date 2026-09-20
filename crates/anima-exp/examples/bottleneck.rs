//! V2.1 information-bottleneck map (read-only, one committed A/C run).
//! Representations over matched windows, per pattern (A/C):
//!   affw     : per-neuron afferent weight vectors at S1 start / drive end.
//!   inst     : spike-set binary vector at a single tick.
//!   rateDrn  : counts in [t, t+500) (during).
//!   rateOff  : counts in [t+500, t+2000) (off).
//!   cntP{50,250,500}: counts accumulated within the presentation.
//!   cntR     : counts accumulated from S1 start to window time.
//!   u        : reconstructed per-neuron u (exact substrate order).
//! Phases: start+t0, mid+t250, end+t500, post+t550, late+t1500.
//! Metrics per (rep, phase): cosine, L2, norm ratio A/C, top-10 Jaccard.
use std::collections::BTreeMap;

fn main() {
    for dir in std::env::args().skip(1) {
        let name = dir.rsplit('/').next().unwrap().to_string();
        let tdir = std::path::Path::new(&dir).join("telemetry");
        let reader = match anima_telemetry::TelemetryReader::open(&tdir) {
            Ok(r) => r, Err(_) => { println!("{name}\tno-telemetry"); continue; }
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
        spikes.sort_by_key(|s| s.0);
        let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        if snaps.is_empty() { continue; }
        const N: usize = 52;
        let idx_of = |nid: u32| -> Option<usize> { if nid >= 24 && nid < 76 { Some(nid as usize - 24) } else { None } };
        let cos = |a: &[f64], b: &[f64]| -> f64 {
            let d: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
            let na = a.iter().map(|x| x * x).sum::<f64>().sqrt();
            let nb = b.iter().map(|x| x * x).sum::<f64>().sqrt();
            if na > 0.0 && nb > 0.0 { d / (na * nb) } else { 0.0 }
        };
        let dist = |a: &[f64], b: &[f64]| -> f64 { (a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum::<f64>()).sqrt() };
        let norm = |a: &[f64]| -> f64 { (a.iter().map(|x| x * x).sum::<f64>()).sqrt() };
        let topj = |a: &[f64], b: &[f64]| -> f64 {
            let mut ai: Vec<(usize, f64)> = a.iter().enumerate().map(|(i, &x)| (i, x)).collect();
            let mut bi: Vec<(usize, f64)> = b.iter().enumerate().map(|(i, &x)| (i, x)).collect();
            ai.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap());
            bi.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap());
            let ta: Vec<usize> = ai.iter().take(10).map(|(i, _)| *i).collect();
            let tb: Vec<usize> = bi.iter().take(10).map(|(i, _)| *i).collect();
            ta.iter().filter(|x| tb.contains(x)).count() as f64 / 10.0
        };
        println!("RUN {name}");
        // afferent weights
        for (tag, s) in [("s1start", &snaps[0]), ("driveend", &snaps[snaps.len() - 1])] {
            let mut va = vec![0.0f64; N];
            let mut vc = vec![0.0f64; N];
            for syn in &s.synapses {
                if syn.pre < 24 && syn.post >= 24 {
                    let i = syn.post as usize - 24;
                    if i < N {
                        let w = syn.w.unwrap_or(0.0) as f64;
                        if syn.pre < 8 { va[i] += w } else { vc[i] += w }
                    }
                }
            }
            println!("  [affw {tag}] cos(A,C)={:.4} dist={:.4} ratio={:.3} J10={:.3}",
                cos(&va, &vc), dist(&va, &vc), norm(&va) / norm(&vc).max(1e-12), topj(&va, &vc));
        }
        // per (pattern, rep, phase) -> (count, sumvec)
        let mut acc: BTreeMap<(String, String, String), (usize, Vec<f64>)> = BTreeMap::new();
        let beta = 0.003125f64;
        let dec = (-1.0f64 / 10000.0).exp();
        let mut u = vec![0.0f64; N];
        let mut cum = vec![0.0f64; N];
        let mut si = 0usize;
        let t_last = pres.last().map(|(_, t)| *t).unwrap_or(0);
        for t in 0..t_last + 2500 {
            for x in u.iter_mut() { *x *= dec; }
            while si < spikes.len() && spikes[si].0 <= t {
                if let Some(i) = idx_of(spikes[si].1) { u[i] += beta; cum[i] += 1.0; }
                si += 1;
            }
            let tick_inst = {
                let mut v = vec![0.0f64; N];
                let mut j = si;
                while j > 0 && spikes[j - 1].0 == t { j -= 1; }
                for &(ts, nid) in &spikes[j..si] {
                    if let Some(i) = idx_of(nid) { v[i] = 1.0; }
                }
                let _ = &mut v;
                v
            };
            for (p, pt) in &pres {
                let rel = t as i64 - *pt as i64;
                let phase = match rel {
                    0 => "start+t0".to_string(),
                    250 => "mid+t250".to_string(),
                    500 => "end+t500".to_string(),
                    550 => "post+t550".to_string(),
                    1500 => "late+t1500".to_string(),
                    _ => continue,
                };
                let mut push = |rep: &str, v: &Vec<f64>| {
                    let e = acc.entry((p.clone(), rep.to_string(), phase.clone())).or_insert((0, vec![0.0; N]));
                    for (a, b) in e.1.iter_mut().zip(v) { *a += b; }
                    e.0 += 1;
                };
                push("inst", &tick_inst);
                push("u", &u);
                push("cntR", &cum);
                if rel == 500 {
                    // within-presentation count buckets + rate windows
                    let mut c50 = vec![0.0f64; N];
                    let mut c250 = vec![0.0f64; N];
                    let mut c500 = vec![0.0f64; N];
                    for &(ts, nid) in &spikes {
                        if ts >= *pt && ts < *pt + 500 {
                            if let Some(i) = idx_of(nid) {
                                c500[i] += 1.0;
                                if ts < *pt + 50 { c50[i] += 1.0; }
                                if ts < *pt + 250 { c250[i] += 1.0; }
                            }
                        }
                    }
                    push("cntP50", &c50);
                    push("cntP250", &c250);
                    push("cntP500", &c500);
                    push("rateDrn", &c500);
                    let mut ro = vec![0.0f64; N];
                    for &(ts, nid) in &spikes {
                        if ts >= *pt + 500 && ts < *pt + 2000 {
                            if let Some(i) = idx_of(nid) { ro[i] += 1.0; }
                        }
                    }
                    push("rateOff", &ro);
                }
            }
        }
        let reps = ["inst", "rateDrn", "rateOff", "cntP50", "cntP250", "cntP500", "cntR", "u"];
        let phases = ["start+t0", "mid+t250", "end+t500", "post+t550", "late+t1500"];
        println!("  {:<8} {:<12} {:>7} {:>7} {:>9} {:>6}  {}", "rep", "phase", "cos", "distL2", "ratioA/C", "J10", "(nA,nC)");
        for rep in reps {
            for ph in phases {
                // rateOff only at late+t1500; rateDrn & cntP* only at end+t500
                if rep == "rateDrn" && ph != "end+t500" { continue; }
                if rep == "rateOff" && ph != "late+t1500" { continue; }
                if (rep.starts_with("cntP")) && ph != "end+t500" { continue; }
                let a = acc.get(&("A".into(), rep.into(), ph.into())).cloned();
                let c = acc.get(&("C".into(), rep.into(), ph.into())).cloned();
                match (a, c) {
                    (Some((na, va)), Some((nc, vc))) => {
                        let ma: Vec<f64> = va.iter().map(|x| x / na as f64).collect();
                        let mc: Vec<f64> = vc.iter().map(|x| x / nc as f64).collect();
                        println!("  {rep:<8} {ph:<12} {:.4} {:7.2} {:9.3} {:6.2}  ({}, {})",
                            cos(&ma, &mc), dist(&ma, &mc),
                            norm(&ma) / norm(&mc).max(1e-12), topj(&ma, &mc), na, nc);
                    }
                    _ => {}
                }
            }
        }
        println!();
    }
}