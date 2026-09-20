fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
    let idx = reader.chunk_index();
    let mut n_spk = 0u64; let mut n_ch = 0u64; let mut low_other = 0u64;
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            if let Ok(env) = row.envelope("e") {
                if let anima_telemetry::events::Payload::Spike { n } = &env.payload {
                    n_spk += 1;
                    if n.0 < 24 { n_ch += 1; }
                    else { low_other += 1; }
                }
            }
        }
    }
    println!("{dir}: spike events total={n_spk} n<24={n_ch} n>=24={low_other}");
}
