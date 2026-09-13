// Count v2 structural events by reason from a chunk dir.
fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let reader = anima_telemetry::TelemetryReader::open(std::path::Path::new(&dir)).unwrap();
    let idx = reader.chunk_index();
    let mut counts = std::collections::BTreeMap::<String, u64>::new();
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            if let Ok(e) = row.envelope("x") {
                use anima_telemetry::events::Payload;
                match &e.payload {
                    Payload::SynapseCreated { reason, .. } => *counts.entry(format!("created:{}", reason.trigger)).or_insert(0) += 1,
                    Payload::SynapsePruned { reason, .. } => *counts.entry(format!("pruned:{}", reason.trigger)).or_insert(0) += 1,
                    _ => {}
                }
            }
        }
    }
    for (k, v) in &counts { println!("{k}: {v}"); }
    if counts.is_empty() { println!("(no structural events)"); }
}
