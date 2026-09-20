//! PLASTICITY-AS-MEMORY AUDIT instrumentation (READ-ONLY, 2026-09-20).
//! Per committed run: per-snapshot (1 kHz cadence) synaptic-state metrics.
//!
//! Groups (from committed configs): A = channels 0-7, C = channels 8-15,
//! B = phase-variant ch 4-11 (never presented in these runs), 16-23 idle.
//! Internal ids 24..76 (40 internal + 12 output, prior-instrument convention).
//! Inhibitory = plastic == false (M6 creation sets plastic=false, core
//! network.rs:534); excitatory aff/recurrent = plastic == true.
//!
//! Output rows:
//!   AGG  <tick> <affcos_t0> <affcos_prev> <reccos_t0> <reccos_prev>
//!        <inhcos_t0> <ucos_t0> <unorm> <selmean> <selabs> <spk1k>
//!        <n_aff> <n_rec> <n_inh> <affcos_pres> <reccos_pres> <inhcos_pres>
//!   (affcos_pres/reccos_pres/inhcos_pres = cosine vs previous
//!    post-presentation snapshot; first one vs itself = 1)
//!   NEUR <tick> <nid> <Amass> <Cmass> <Omas> <recsum> <inhsum> <top3ch>
//!   MAT  <tag> <tick> <nid> <24 ch masses>
use anima_telemetry::recorder::read_snapshots;
use std::collections::BTreeMap;

fn main() {
    for dir in std::env::args().skip(1) {
        let snaps = match read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")) {
            Ok(s) => s,
            Err(e) => { eprintln!("{dir}: {e}"); continue; }
        };
        let tdir = std::path::Path::new(&dir).join("telemetry");
        let reader = match anima_telemetry::TelemetryReader::open(&tdir) {
            Ok(r) => r,
            Err(_) => { eprintln!("{dir}: no telemetry"); continue; }
        };
        let idx = reader.chunk_index();
        let mut pres: Vec<(String, u64)> = Vec::new();
        let mut spk_by_tick: BTreeMap<u64, u32> = BTreeMap::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                match row.kind {
                    3 => { if let Some(n) = row.n { if n >= 24 && n < 76 {
                        *spk_by_tick.entry(row.t).or_default() += 1; } } }
                    5 => { if let Ok(env) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } = env.payload {
                            if stage == "S1" { pres.push((pattern_id, env.t)); } } } }
                    _ => {}
                }
            }
        }
        pres.sort_by_key(|p| p.1);
        const N: usize = 52;
        const C: usize = 24;
        let idx_of = |nid: u32| -> Option<usize> { if nid >= 24 && nid < 76 { Some(nid as usize - 24) } else { None } };
        let cos = |a: &[f64], b: &[f64]| -> f64 {
            let (mut ab, mut aa, mut bb) = (0.0f64, 0.0f64, 0.0f64);
            for (x, y) in a.iter().zip(b) { ab += x * y; aa += x * x; bb += y * y; }
            if aa <= 0.0 || bb <= 0.0 { 0.0 } else { ab / (aa * bb).sqrt() }
        };
        println!("RUN {}", dir.rsplit('/').next().unwrap());
        println!("pres={} (A={} C={})", pres.len(),
            pres.iter().filter(|(p, _)| p == "A").count(),
            pres.iter().filter(|(p, _)| p == "C").count());
        // post-presentation snapshot ticks: presentation t + 1000 (500 ms after end),
        // snapped to the 1 kHz snapshot grid (t is delivery-tick +1; snapshots at
        // tick % 1000 == 0), so use floor((t+999)/1000)*1000.
        let post_pres: Vec<(String, u64)> = pres.iter().map(|(p, t)| (p.clone(), ((t + 999) / 1000) * 1000)).collect();
        let mut prev_aff: Option<Vec<f64>> = None;
        let mut prev_rec: Option<Vec<f64>> = None;
        let mut prev_inh: Option<Vec<f64>> = None;
        let mut t0_aff: Option<Vec<f64>> = None;
        let mut t0_rec: Option<Vec<f64>> = None;
        let mut t0_inh: Option<Vec<f64>> = None;
        let mut t0_u: Option<Vec<f64>> = None;
        let mut pp_aff: Option<Vec<f64>> = None;
        let mut pp_rec: Option<Vec<f64>> = None;
        let mut pp_inh: Option<Vec<f64>> = None;
        let mut last_s1: Option<(u64, Vec<f64>)> = None;
        let mut first = true;
        let mut last_tick = 0u64;
        let mut win_spk = 0u32;
        for (i, s) in snaps.iter().enumerate() {
            let dt = s.tick.saturating_sub(last_tick);
            if dt > 0 {
                for t in (last_tick + 1)..=s.tick {
                    win_spk += spk_by_tick.get(&t).copied().unwrap_or(0);
                }
            }
            last_tick = s.tick;
            let mut aff = vec![0.0f64; N * C];
            let mut rec = vec![0.0f64; N];
            let mut inh = vec![0.0f64; N];
            for syn in &s.synapses {
                if syn.pre < 24 {
                    if let Some(j) = idx_of(syn.post) {
                        if let Some(w) = syn.w { if w > 0.0 { aff[j * C + syn.pre as usize] += w as f64; } }
                    }
                } else if let Some(j) = idx_of(syn.post) {
                    let w = syn.w.unwrap_or(0.0) as f64;
                    if syn.plastic { rec[j] += w; } else { inh[j] += w; }
                }
            }
            let u: Vec<f64> = s.neurons.iter()
                .filter(|n| n.id >= 24 && n.id < 76)
                .map(|n| n.u_slow.unwrap_or(0.0) as f64)
                .collect();
            let sel: Vec<f64> = (0..N).map(|j| {
                let a: f64 = aff[j*C..j*C+8].iter().sum();
                let cc: f64 = aff[j*C+8..j*C+16].iter().sum();
                let tot = a + cc;
                if tot <= 0.0 { 0.0 } else { (a - cc) / tot }
            }).collect();
            let selmean = sel.iter().sum::<f64>() / N as f64;
            let selabs = sel.iter().map(|x| x.abs()).sum::<f64>() / N as f64;
            if first {
                t0_aff = Some(aff.clone()); t0_rec = Some(rec.clone()); t0_inh = Some(inh.clone());
                t0_u = Some(u.clone()); first = false;
            }
            let (ac0, ap) = (cos(&aff, t0_aff.as_ref().unwrap()), cos(&aff, prev_aff.as_ref().unwrap_or(&aff)));
            let (rc0, rp) = (cos(&rec, t0_rec.as_ref().unwrap()), cos(&rec, prev_rec.as_ref().unwrap_or(&rec)));
            let (ic0, ip) = (cos(&inh, t0_inh.as_ref().unwrap()), cos(&inh, prev_inh.as_ref().unwrap_or(&inh)));
            let uc0 = cos(&u, t0_u.as_ref().unwrap());
            let unorm = (u.iter().map(|x| x * x).sum::<f64>()).sqrt();
            let n_aff = aff.iter().filter(|&&w| w > 0.0).count();
            let n_rec = rec.iter().filter(|&&w| w > 0.0).count();
            let n_inh = inh.iter().filter(|&&w| w > 0.0).count();
            // post-presentation cosine tracking
            let (pc_a, pc_r, pc_i) = if let Some((pt, _)) = post_pres.iter().find(|(_, t)| *t == s.tick) {
                let _ = pt;
                let a = cos(&aff, pp_aff.as_ref().unwrap_or(&aff));
                let r = cos(&rec, pp_rec.as_ref().unwrap_or(&rec));
                let i = cos(&inh, pp_inh.as_ref().unwrap_or(&inh));
                pp_aff = Some(aff.clone()); pp_rec = Some(rec.clone()); pp_inh = Some(inh.clone());
                (a, r, i)
            } else {
                (0.0, 0.0, 0.0)
            };
            let t = s.tick;
            println!("AGG\t{t}\t{ac0:.4}\t{ap:.4}\t{rc0:.4}\t{rp:.4}\t{ic0:.4}\t{uc0:.6}\t{unorm:.4}\t{selmean:.4}\t{selabs:.4}\t{win_spk}\t{n_aff}\t{n_rec}\t{n_inh}\t{pc_a:.4}\t{pc_r:.4}\t{pc_i:.4}");
            if post_pres.iter().any(|(_, pt)| *pt == s.tick) {
                for j in 0..N {
                    let a: f64 = aff[j*C..j*C+8].iter().sum();
                    let cc: f64 = aff[j*C+8..j*C+16].iter().sum();
                    let om: f64 = aff[j*C+16..j*C+24].iter().sum();
                    let mut top: Vec<usize> = (0..C).collect();
                    top.sort_by(|&x, &y| aff[j*C+y].partial_cmp(&aff[j*C+x]).unwrap());
                    println!("NEUR\t{}\t{}\t{a:.5}\t{cc:.5}\t{om:.5}\t{:.5}\t{:.5}\t{:?}", s.tick, j + 24, rec[j], inh[j], &top[..3]);
                }
            }
            let drive_end = ((post_pres.last().map(|(_, t)| *t).unwrap_or(85000) + 999) / 1000) * 1000;
            if s.tick == drive_end || (i + 1 == snaps.len()) {
                for j in 0..N {
                    let row: Vec<String> = aff[j*C..j*C+C].iter().map(|w| format!("{w:.6}")).collect();
                    println!("MAT\t{}\t{}\t{}", if s.tick == drive_end { "driveend" } else { "final" }, s.tick, j + 24);
                    println!("  {}", row.join(","));
                }
                last_s1 = Some((s.tick, aff.clone()));
            }
            prev_aff = Some(aff); prev_rec = Some(rec); prev_inh = Some(inh);
            // slide post-presentation window: drop processed ticks
            win_spk = 0;
        }
        let _ = last_s1;
        println!();
    }
}