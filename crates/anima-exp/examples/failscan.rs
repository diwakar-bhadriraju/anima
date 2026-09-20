//! Read-only: print Failure + RunEnded events with ticks.
fn main() {
    for dir in std::env::args().skip(1) {
        let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut nfail = 0u64;
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if row.kind == 4 { continue; } // placeholder
                if row.kind == 17 {
                    let env = row.envelope("e").ok();
                    if let Some(env) = env {
                        if let anima_telemetry::events::Payload::Failure { kind, .. } = env.payload {
                            println!("{} t={} kind={}", dir.rsplit('/').next().unwrap(), row.t, kind);
                            nfail += 1;
                        }
                    }
                }
            }
        }
        if nfail == 0 { println!("{}: NO failure events", dir.rsplit('/').next().unwrap()); }
    }
}
