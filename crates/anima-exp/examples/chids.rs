use std::collections::BTreeSet;
fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
    let idx = reader.chunk_index();
    let mut ids = BTreeSet::new();
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            if let Ok(env) = row.envelope("e") {
                if let anima_telemetry::events::Payload::Spike { n } = &env.payload {
                    if n.0 < 24 { ids.insert(n.0); }
                }
            }
        }
    }
    println!("channel spike ids: {:?}", ids);
}
