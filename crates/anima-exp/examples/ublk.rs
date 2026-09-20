//! V2.1 u-conditioning diagnostic for BLOCKED curricula (read-only, uses
//! committed runs): after 20 presentations of the first block (A in BAC,
//! C in BCA) vs after 20 of the second block, is the u-vector different?
//! u snapshots: midsnap near last-presentation end of each block is a
//! A-conditioned-u vs C-conditioned-u comparison at equal drive history
//! length (block 1 end vs block 2 end), 40 s later, off-window phase.
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
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if row.kind == 5 {
                    if let Some(env) = row.envelope("e").ok() {
                        if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } = env.payload {
                            if stage == "S1" { pres.push((pattern_id, env.t)); }
                        }
                    }
                }
            }
        }
        pres.sort_by_key(|p| p.1);
        let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let ticks: Vec<u64> = snaps.iter().map(|s| s.tick).collect();
        let u_after = |t: u64| -> Option<Vec<f32>> {
            let st = ticks.iter().find(|&&x| x >= t)?;
            let s = snaps.iter().find(|s| s.tick == *st)?;
            Some(s.neurons.iter().filter(|n| n.id >= 24).map(|n| n.u_slow.unwrap_or(0.0)).collect())
        };
        // split by block: first 20 presentations (block 1) and last 20 (block 2)
        let b1_end = pres[19].1 + 500;
        let b2_end = pres[39].1 + 500;
        if let (Some(u1), Some(u2)) = (u_after(b1_end), u_after(b2_end)) {
            let cos = |a: &[f32], b: &[f32]| -> f64 {
                let d: f64 = a.iter().zip(b).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
                let na = (a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
                let nb = (b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
                if na > 0.0 && nb > 0.0 { d / (na * nb) } else { 0.0 }
            };
            let norm = |v: &[f32]| -> f64 { (v.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt() };
            let top10 = |v: &[f32]| -> Vec<u32> {
                let mut ids: Vec<(u32, f32)> = v.iter().enumerate().map(|(i, &x)| (i as u32 + 24, x)).collect();
                ids.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
                ids.iter().take(10).map(|(i, _)| *i).collect()
            };
            let t1 = top10(&u1); let t2 = top10(&u2);
            let inter = t1.iter().filter(|x| t2.contains(x)).count();
            println!("{name}\tseq={}:{}->{}:{}\tu-cos={:.4}\tu-dist={:.2}\t|u1|={:.2}\t|u2|={:.2}\tJ10={:.3}",
                pres[0].0, pres[19].0, pres[20].0, pres[39].0,
                cos(&u1, &u2),
                (u1.iter().zip(&u2).map(|(x, y)| (x - y).powi(2)).sum::<f32>() as f64).sqrt(),
                norm(&u1), norm(&u2), inter as f64 / 10.0);
        } else {
            println!("{name}\tno-snaps-at-block-ends");
        }
    }
}