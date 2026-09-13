// Audit smoke: verify v2 structural events + budget fields exist in the
// run without evaluating any metric.
fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let reader = anima_telemetry::TelemetryReader::open(std::path::Path::new(&dir)).unwrap();
    let idx = reader.chunk_index();
    let mut created = 0u64; let mut pruned = 0u64; let mut budget = 0u64;
    let mut reasons = std::collections::BTreeMap::<String, u64>::new();
    let mut max_live_exc = 0u64;
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            if let Ok(e) = row.envelope("v2") {
                use anima_telemetry::events::Payload;
                match &e.payload {
                    Payload::SynapseCreated { reason, .. } => {
                        created += 1;
                        *reasons.entry(reason.trigger.clone()).or_insert(0) += 1;
                    }
                    Payload::SynapsePruned { reason, .. } => {
                        pruned += 1;
                        *reasons.entry(reason.trigger.clone()).or_insert(0) += 1;
                    }
                    Payload::ResourceUsage { live_exc, .. } => {
                        budget += 1;
                        if let Some(v) = live_exc { max_live_exc = max_live_exc.max(*v); }
                    }
                    _ => {}
                }
            }
        }
    }
    println!("created={created} pruned={pruned} budget_rows={budget} max_live_exc={max_live_exc}");
    for (k, v) in &reasons { println!("  reason {k}: {v}"); }
}
