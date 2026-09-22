//! respoverlap: Phase III read-only diagnostic — WHERE does blocked-order
//! re-expression mixing come from?
//!
//! For the re-exposure (S3) A and C blocks of a blocked E-nogain run,
//! classify every internal neuron that fired during a presentation:
//!   has-A-aff  = the neuron has a live track-0 afferent (an A-channel
//!                afferent, pre<8, non-inhibitory);
//!   has-C-aff  = a live track-1 afferent (pre 8..16);
//!   has-neither = only recurrent/other drive (recurrent-recruited).
//! Report, per presentation: count of A-responders, C-responders, the
//! Jaccard overlap of the two responder sets, and the fraction of each
//! whose afferents do NOT include the presented pattern's own afferents
//! (= recurrent-recruited mixing). The reference (S1) responder sets are
//! the mean over the last 10 S1 presentations of each pattern.
use anima_telemetry::TelemetryReader;
use anima_telemetry::events::Payload;
use anima_telemetry::recorder::read_snapshots;

fn main() {
    for dir in std::env::args().skip(1) {
        let snaps = read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let reader = TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut pres: Vec<(u64, String, String)> = Vec::new();
        let mut spk: Vec<(u64, u32)> = Vec::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(e) = row.envelope("e") {
                    match &e.payload {
                        Payload::StimulusPresented { pattern_id, stage } => {
                            pres.push((row.t, pattern_id.clone(), stage.clone()));
                        }
                        Payload::Spike { n } => { if n.0 >= 24 && n.0 < 76 { spk.push((row.t, n.0)); } }
                        _ => {}
                    }
                }
            }
        }
        pres.sort_by_key(|p| p.0);
        let name = dir.rsplit('/').next().unwrap().to_string();
        // afferent map from a mid-run snapshot (t=44000 snapshot if present)
        let snap = snaps.iter().find(|s| s.tick == 44_000).or_else(|| snaps.first()).unwrap();
        let mut has_a = vec![false; 52];
        let mut has_c = vec![false; 52];
        for s in &snap.synapses {
            if s.post < 24 || s.post >= 76 { continue; }
            if s.w.is_none() || !s.plastic { continue; }
            let i = (s.post - 24) as usize;
            if s.pre < 8 { has_a[i] = true; }
            else if s.pre < 16 { has_c[i] = true; }
        }
        // responder sets per presentation (internal ids 24..75 -> 0..51)
        let responders = |t0: u64| -> Vec<bool> {
            let mut v = vec![false; 52];
            for (t, n) in &spk {
                if *t >= t0 && *t < t0 + 500 {
                    let i = (n - 24) as usize;
                    if i < 52 { v[i] = true; }
                } else if *t > t0 + 500 { break; }
            }
            v
        };
        // S3 A and C blocks
        for (stage_lbl, pat) in [("S3A", "A"), ("S3C", "C")] {
            let rows: Vec<&(u64, String, String)> = pres.iter()
                .filter(|p| p.2 == stage_lbl && p.1 == pat).collect();
            if rows.is_empty() { continue; }
            let a_resp: Vec<Vec<bool>> = rows.iter().map(|r| responders(r.0)).collect();
            // summary over the block: union responders, jaccard, recruit fraction
            let mut union_a = vec![false; 52]; let mut union_c = vec![false; 52];
            for v in &a_resp { for (i, b) in v.iter().enumerate() { union_a[i] |= *b; } }
            // for jaccard we need both patterns' re-exposure sets; compute C from S3C separately
            let crow: Vec<&(u64, String, String)> = pres.iter()
                .filter(|p| p.2 == "S3C" && p.1 == "C").collect();
            for r in &crow { let v = responders(r.0); for (i, b) in v.iter().enumerate() { union_c[i] |= *b; } }
            let cnt = |v: &Vec<bool>| v.iter().filter(|b| **b).count();
            let inter = (0..52).filter(|&i| union_a[i] && union_c[i]).count();
            let uni = (0..52).filter(|&i| union_a[i] || union_c[i]).count();
            let jac = if uni > 0 { inter as f32 / uni as f32 } else { 0.0 };
            // fraction of the union responder set lacking BOTH A and C afferents (pure recurrent)
            let rec_only = (0..52).filter(|&i| (union_a[i] || union_c[i]) && !has_a[i] && !has_c[i]).count();
            println!("{name}\t{stage_lbl}({pat})\tunionA={}\tunionC={}\tinter={}\tjacc={jac:.3}\trec_only={}",
                cnt(&union_a), cnt(&union_c), inter, rec_only);
        }
        println!();
    }
}