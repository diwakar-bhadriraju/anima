fn main() {
    for dir in std::env::args().skip(1) {
        let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(env) = row.envelope("e") {
                    if let anima_telemetry::events::Payload::RunEnded { reason } = env.payload {
                        println!("{} RunEnded t={} reason={:?}", dir.rsplit('/').next().unwrap(), row.t, reason);
                    }
                }
            }
        }
    }
}
