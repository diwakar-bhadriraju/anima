//! Read-only: reserved candidates by channel cohort from CandidatePool rows.
//! Prints per row: tick, reserved count pre<8 (A), 8..16 (C), >=16 (O).
fn main() {
    for dir in std::env::args().skip(1) {
        let reader = anima_telemetry::TelemetryReader::open(
            &std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(env) = row.envelope("e") {
                    if let anima_telemetry::events::Payload::CandidatePool { pools } = &env.payload {
                        let mut ra = 0usize; let mut rc = 0usize; let mut ro = 0usize;
                        for p in pools {
                            for s in &p.slots {
                                if !s.2 { continue; }
                                if s.0 < 8 { ra += 1; } else if s.0 < 16 { rc += 1; } else { ro += 1; }
                            }
                        }
                        println!("PC\t{}\t{}\t{}\t{}", row.t, ra, rc, ro);
                    }
                }
            }
        }
        println!();
    }
}
