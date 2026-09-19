//! Stage-1 identity gate: dump (t, kind, n, w?) rows functional fingerprint of a run:
//! all non-marker event rows (kind != 5) -> (t, kind, neuron, extra f32 bits) for hashing.
fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
    let idx = reader.chunk_index();
    let mut h: u64 = 0xcbf29ce484222325; // fnv-1a
    let mut count = 0u64;
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            if row.kind == 5 { continue; }
            h ^= row.kind as u64; h = h.wrapping_mul(0x100000001b3);
            h ^= row.t; h = h.wrapping_mul(0x100000001b3);
            h ^= row.n.unwrap_or(0) as u64; h = h.wrapping_mul(0x100000001b3);
            if let Some(w) = row.w { h ^= w.to_bits() as u64; h = h.wrapping_mul(0x100000001b3); }
            count += 1;
        }
    }
    println!("{dir}: rows={count} fnv={h:016x}");
}
