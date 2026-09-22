//! Level-5 r2 Slice-1 closed-loop analysis (docs/phase3/level5-r2-closedloop.md,
//! D-17). The closed loop next = f(output_prev) over the organism's
//! DETERMINISTIC measured output response is an iterated binary map on
//! {A, C}; its attractor is exactly computable by iterating the measured
//! A- and C-response reference vectors through the frozen tag map:
//!   vote(stimulated) = sum(out[64..70]) vs sum(out[70..76]) over that
//!   presentation's response.
//!   next = C if vote_A <= vote_C else A.
//! Iterate: state(0) = A -> f(A_resp) -> ... until a fixed point or cycle.
use anima_telemetry::TelemetryReader;
use anima_telemetry::events::Payload;

fn main() {
    for dir in std::env::args().skip(1) {
        let r = TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = r.chunk_index();
        let mut pres = vec![];
        let mut spk = vec![];
        for c in 0..idx.len() {
            for row in r.chunk_rows(c).unwrap() {
                if let Ok(e) = row.envelope("e") {
                    match &e.payload {
                        Payload::StimulusPresented { pattern_id, .. } => pres.push((row.t, pattern_id.clone())),
                        Payload::Spike { n } => if (64..76).contains(&n.0) { spk.push((row.t, n.0)); },
                        _ => {}
                    }
                }
            }
        }
        pres.sort_by_key(|p| p.0);
        let mut refs: std::collections::BTreeMap<String, Vec<f32>> = Default::default();
        let mut cnt: std::collections::BTreeMap<String, u32> = Default::default();
        for (t, p) in &pres {
            let mut v = vec![0.0f32; 12];
            for (st, s) in &spk { if *st >= *t && *st < t + 500 && (64..76).contains(s) { v[(s - 64) as usize] += 1.0; } }
            let e = refs.entry(p.clone()).or_insert_with(|| vec![0.0f32; 12]);
            for i in 0..12 { e[i] += v[i]; }
            *cnt.entry(p.clone()).or_insert(0) += 1;
        }
        for (p, v) in refs.iter_mut() { let c = cnt[p] as f32; for x in v.iter_mut() { *x /= c; } }
        let name = dir.rsplit('/').next().unwrap().to_string();
        for (lo_pat, _) in [("A", "C"), ("C", "A")] {
            let res = refs.get(lo_pat).cloned().unwrap_or_else(|| vec![0.0f32; 12]);
            let ta: f32 = res[..6].iter().sum();
            let tc: f32 = res[6..].iter().sum();
            let nxt = if ta <= tc { "C" } else { "A" };
            println!("{name} | from {lo_pat}: tagA={ta:.1} tagC={tc:.1} -> next={nxt}");
        }
        // iterate the map from both states to find the attractor
        let next_of = |p: &str| -> &str {
            let res = &refs[p];
            let ta: f32 = res[..6].iter().sum();
            let tc: f32 = res[6..].iter().sum();
            if ta <= tc { "C" } else { "A" }
        };
        let mut seen = std::collections::BTreeMap::new();
        let mut cur = "A";
        for step in 0..8u32 {
            if seen.insert(cur.to_string(), step).is_some() { break; }
            cur = next_of(cur);
        }
        let cyc: Vec<&str> = seen.keys().map(|s| s.as_str()).collect();
        println!("{name} | closed-loop map path A->... absorb into {{{}}}; attractor_len={}",
            cyc.join(","), seen.len());
    }
}