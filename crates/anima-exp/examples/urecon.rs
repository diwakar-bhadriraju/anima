//! V2.1 u trajectory reconstruction (read-only): recompute u per neuron
//! deterministically from the recorded spike stream (u *= exp(-1/tau_s)
//! per tick; u += beta per non-input spike), sampling at 10ms resolution
//! around each presentation. Answers: does u differ A-vs-C DURING the
//! stimulus (intra-stim read), at +50ms, +200ms, +1500ms? The snapshots
//! (1s cadence) alias all windows to off-phase reads; this closes that.
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
        spikes.sort_by_key(|s| s.0);
        let tau_s = 10000.0f32; // b0.003125-t10000 cell (validated below)
        let beta = 0.003125f32;
        // reconstruct: advance tick by tick from 0 to end; coarsely 1ms exact.
        let t_end = pres.last().map(|(_, t)| *t + 2500).unwrap_or(0);
        let n = 52usize;
        let mut u = [0.0f32; 52];
        let mut sp_i = 0usize;
        let dec = (-1.0f32 / tau_s).exp();
        // sample points per presentation
        let mut samples: BTreeMap<(String, u64), Vec<[f32; 52]>> = BTreeMap::new();
        let mut cnt: BTreeMap<(String, u64), u32> = BTreeMap::new();
        let mut recon_at_44k = 0.0f64;
        for t in 0..t_end {
            // substrate order: decay-then-read, increment-after-spike
            for x in u.iter_mut() { *x *= dec; }
            while sp_i < spikes.len() && spikes[sp_i].0 <= t {
                let nid = spikes[sp_i].1;
                if nid >= 24 && nid < 76 { u[(nid - 24) as usize] += beta; }
                sp_i += 1;
            }
            if t == 44_000 {
                recon_at_44k = (u.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
            }
            for (p, pt) in &pres {
                let offs = [0u64, 50, 200, 500, 1500, 250];
                for &o in &offs {
                    if t == pt + o {
                        samples.entry((p.clone(), o + 1)).or_default().push(u);
                        *cnt.entry((p.clone(), o + 1)).or_default() += 1;
                    }
                }
            }
        }
        let meanv = |vs: &Vec<[f32; 52]>| -> Vec<f32> {
            let n = vs.len().max(1);
            (0..52).map(|i| vs.iter().map(|v| v[i]).sum::<f32>() / n as f32).collect()
        };
        let cos = |a: &[f32], b: &[f32]| -> f64 {
            let d: f64 = a.iter().zip(b).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
            let na = (a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
            let nb = (b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
            if na > 0.0 && nb > 0.0 { d / (na * nb) } else { 0.0 }
        };
        let norm = |v: &[f32]| -> f64 { (v.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt() };
        println!("RUN {name}");
        println!("  offsets: 0=START, 50=+50ms, 200=+200ms, 250=mid, 500=end, 1500=+1500ms (key-encoded +1)");
        for o in [1u64, 51, 201, 251, 501, 1501] {
            let a = samples.get(&("A".into(), o)).cloned().unwrap_or_default();
            let c = samples.get(&("C".into(), o)).cloned().unwrap_or_default();
            if !a.is_empty() && !c.is_empty() {
                let (ma, mc) = (meanv(&a), meanv(&c));
                println!("  offset {o:+5}ms: cos(A,C)={:.4} dist={:.2} |A|={:.3} |C|={:.3} ratio={:.3} (nA={}, nC={})",
                    cos(&ma, &mc),
                    (ma.iter().zip(&mc).map(|(x, y)| (x - y).powi(2)).sum::<f32>() as f64).sqrt(),
                    norm(&ma), norm(&mc), norm(&ma) / norm(&mc).max(1e-9), a.len(), c.len());
            } else if !a.is_empty() {
                let ma = meanv(&a);
                println!("  offset {o:+5}ms: (A-only) |A|={:.3} (n={})", norm(&ma), a.len());
            } else {
                println!("  offset {o:+5}ms: insufficient");
            }
        }
        // validate reconstruction against a snapshot (u_off at 44k vs snapshot 44k)
        let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let mid = snaps.iter().find(|s| s.tick >= 44_000).unwrap();
        let snap_u: f64 = mid.neurons.iter().filter(|n| n.id >= 24).map(|n| n.u_slow.unwrap_or(0.0) as f64).map(|x| x * x).sum::<f64>().sqrt();
        println!("  reconstruction validation: snapshot ‖u‖≈{snap_u:.2} vs reconstructed {recon_at_44k:.2} at t=44k");
        println!();
    }
}