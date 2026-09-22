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
        let name = dir.rsplit('/').next().unwrap().to_string();
        // the two distinct patterns presented in S1
        let mut s1_pats: Vec<String> = Vec::new();
        for p in pres.iter().filter(|p| p.2 == "S1") {
            if !s1_pats.contains(&p.1) { s1_pats.push(p.1.clone()); }
            if s1_pats.len() >= 2 { break; }
        }
        if s1_pats.len() < 2 { println!("{name}\t(note: fewer than 2 S1 patterns, probe skipped)"); continue; }
        let (p0, p1) = (s1_pats[0].clone(), s1_pats[1].clone());
        // references = mean over the last 10 S1 presentations of each
        let s1: Vec<(u64, String, String)> = pres.iter().filter(|p| p.2 == "S1").cloned().collect();
        let ref_of = |pat: &str| -> Vec<f32> {
            let rows: Vec<&(u64, String, String)> = s1.iter().filter(|p| p.1 == pat).collect();
            let take = rows.len().min(10);
            let mut sum = vec![0.0f32; 52];
            for &r in &rows[rows.len() - take..] {
                let v = vector(r.0, &spk24);
                for (i, x) in v.iter().enumerate() { sum[i] += x; }
            }
            let n = take.max(1) as f32;
            sum.iter().map(|x| x / n).collect()
        };
        let ref0 = ref_of(&p0);
        let ref1 = ref_of(&p1);
        // probe: re-exposure stages that re-present p0 or p1
        for pat in [&p0, &p1] {
            let rows: Vec<&(u64, String, String)> = pres.iter()
                .filter(|p| p.2 != "S1" && &p.1 == pat && (p.2 == "S3A" || p.2 == "S3C" || p.2 == "S3A2" || p.2 == "S3C2"))
                .collect();
            if rows.is_empty() { println!("{name}\t{pat}\t(no probe stage)"); continue; }
            for (wi, &r) in rows.iter().enumerate() {
                let v = vector(r.0, &spk24);
                let c0 = cosine(&v, &ref0);
                let c1 = cosine(&v, &ref1);
                let rho = if pat == &p0 { c0 - c1 } else { c1 - c0 };
                println!("{name}\t{pat}\tw{wi}\trho={rho:.3}\tcos{p0}={c0:.3}\tcos{p1}={c1:.3}");
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