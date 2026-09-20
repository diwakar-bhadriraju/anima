//! Read-only: per-run StimulusPresented counts by pattern (S1).
fn main() {
    for dir in std::env::args().skip(1) {
        use std::collections::BTreeMap;
        let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut counts: BTreeMap<(String, String), u32> = BTreeMap::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(env) = row.envelope("e") {
                    if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } = &env.payload {
                        *counts.entry((stage.clone(), pattern_id.clone())).or_default() += 1;
                    }
                }
            }
        }
        print!("{}:", dir.rsplit('/').next().unwrap());
        for ((stage, pat), n) in &counts {
            print!(" {stage}/{pat}={n}");
        }
        println!();
    }
}
