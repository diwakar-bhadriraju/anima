fn main() {
    for dir in std::env::args().skip(1) {
        let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(env) = row.envelope("e") {
                    match &env.payload {
                        anima_telemetry::events::Payload::RunEnded { .. } => {
                            println!("{} RunEnded t={}", dir.rsplit('/').next().unwrap(), row.t);
                        }
                        anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } => {
                            if stage == "S2" { println!("{} S2 starts t={}", dir.rsplit('/').next().unwrap(), row.t); }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
