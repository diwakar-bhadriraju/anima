//! Dump latch sets (drive-end) for C-S1 decoding analysis.
fn main() {
    for dir in std::env::args().skip(1) {
        let name = dir.rsplit('/').next().unwrap();
        let parts: Vec<&str> = name.split('-').collect();
        let (arm, seed, cur) = (parts[1], parts[2].trim_start_matches('s'), parts[3]);
        let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut last_end = 0u64;
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if row.kind == 5 {
                    if let Some(env) = row.envelope("e").ok() {
                        if let anima_telemetry::events::Payload::StimulusPresented { .. } = env.payload {
                            last_end = (env.t + 500).max(last_end);
                        }
                    }
                }
            }
        }
        let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let s_end = snaps.iter().rev().find(|s| s.tick <= last_end).unwrap();
        let ids: Vec<String> = s_end.neurons.iter().filter(|n| n.id >= 24 && n.z_latch == Some(1)).map(|n| n.id.to_string()).collect();
        println!("{seed}\t{arm}\t{cur}\t{}", ids.join(","));
    }
}
