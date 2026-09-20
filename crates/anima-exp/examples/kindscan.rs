fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
    let idx = reader.chunk_index();
    let mut k18 = 0u64; let mut total = 0u64;
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            total += 1;
            if row.kind == 18 { k18 += 1; }
        }
    }
    println!("rows={total} kind18={k18}");
}
