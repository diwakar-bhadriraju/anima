//! rexp: Phase II-AR read-only re-expression probe (protocol §5 S3).
//! Per presentation, build the 52-dim internal spike-count vector
//! (ids 24..75) over [t0, t0+500). Reference vectors = mean over the
//! LAST 10 presentations of each pattern in S1; re-expression blocks =
//! the S3 A and S3 C stages. Reports rho(A) = cos(v_A_late, v_A_ref) -
//! cos(v_A_late, v_C_ref) and rho(C), plus the raw cosines.
use anima_telemetry::TelemetryReader;
use anima_telemetry::events::Payload;

fn main() {
    for dir in std::env::args().skip(1) {
        let reader = TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut pres: Vec<(u64, String, String)> = Vec::new(); // (t, pattern, stage)
        let mut spk24: Vec<(u64, u32)> = Vec::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(e) = row.envelope("e") {
                    match &e.payload {
                        Payload::StimulusPresented { pattern_id, stage } => {
                            pres.push((row.t, pattern_id.clone(), stage.clone()));
                        }
                        Payload::Spike { n } => {
                            if n.0 >= 24 && n.0 < 76 { spk24.push((row.t, n.0)); }
                        }
                        _ => {}
                    }
                }
            }
        }
        pres.sort_by_key(|p| p.0);
        // S1 references: last 10 presentations of each pattern in S1
        let s1: Vec<(u64, String, String)> = pres.iter().filter(|p| p.2 == "S1").cloned().collect();
        let mut refs: std::collections::BTreeMap<String, Vec<f32>> = Default::default();
        for pat in ["A", "C"] {
            let rows: Vec<&(u64, String, String)> = s1.iter().filter(|p| p.1 == pat).collect();
            let take = rows.len().min(10);
            let mut sum = vec![0.0f32; 52];
            for &r in &rows[rows.len() - take..] {
                let v = vector(r.0, &spk24);
                for (i, x) in v.iter().enumerate() { sum[i] += x; }
            }
            let n = take.max(1) as f32;
            refs.insert(pat.to_string(), sum.iter().map(|x| x / n).collect());
        }
        // S3 blocks: stage names S3A (A) and S3C (C); report per-window
        // rho (trajectory across the 5 re-presentations) + last-3 mean.
        let name = dir.rsplit('/').next().unwrap().to_string();
        for (stage, pat) in [("S3A", "A"), ("S3C", "C")] {
            let rows: Vec<&(u64, String, String)> =
                pres.iter().filter(|p| p.2 == stage && p.1 == pat).collect();
            if rows.is_empty() { continue; }
            for (wi, &r) in rows.iter().enumerate() {
                let v = vector(r.0, &spk24);
                let cos_a = cosine(&v, refs.get("A").unwrap());
                let cos_c = cosine(&v, refs.get("C").unwrap());
                let rho = if pat == "A" { cos_a - cos_c } else { cos_c - cos_a };
                println!("{name}\t{pat}\tw{wi}\trho={rho:.3}\tcosA={cos_a:.3}\tcosC={cos_c:.3}");
            }
        }
        println!();
        println!();
    }
}

fn vector(t0: u64, spk: &[(u64, u32)]) -> [f32; 52] {
    let mut v = [0.0f32; 52];
    // spk is sorted by t; binary-ish scan (small scale)
    for (t, n) in spk {
        if *t >= t0 && *t < t0 + 500 {
            let i = (n - 24) as usize;
            if i < 52 { v[i] += 1.0; }
        } else if *t > t0 + 500 {
            break;
        }
    }
    v
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let mut na = 0.0; let mut nb = 0.0; let mut num = 0.0;
    for (x, y) in a.iter().zip(b.iter()) { num += x * y; na += x * x; nb += y * y; }
    if na <= 0.0 || nb <= 0.0 { 0.0 } else { num / (na.sqrt() * nb.sqrt()) }
}