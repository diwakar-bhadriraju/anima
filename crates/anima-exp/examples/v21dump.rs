//! Dump per-presentation u vectors (post-presentation snapshot) for external stats.
use std::collections::BTreeMap;
fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
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
    for (p, t) in &pres {
        // last snapshot within [t+500, t+1000): u right after the presentation
        let tend = t + 500;
        if let Some(&st) = ticks.iter().rev().find(|&&x| x >= tend && x <= tend + 500) {
            let s = snaps.iter().find(|s| s.tick == st).unwrap();
            let v: Vec<String> = s.neurons.iter().map(|n| format!("{:.4}", n.u_slow.unwrap_or(0.0))).collect();
            println!("{p}\t{t}\t{}", v.join(","));
        }
    }
    let _ = BTreeMap::<u8, u8>::new;
}
