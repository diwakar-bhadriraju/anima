// Distribution of prediction error over time + when it exceeds mu+2sigma
fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let reader = anima_telemetry::TelemetryReader::open(std::path::Path::new(&dir)).unwrap();
    let idx = reader.chunk_index().clone();
    // Track EWMA like Instrumentation (alpha 0.02): recompute bound from
    // the error stream to see when a trigger would have fired.
    let mut pe_mean = 0.0f32;
    let mut pe_std = 1.0f32;
    let mut over = 0u64;
    let mut last_t = 0u64;
    let alpha = 0.02f32;
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            if row.kind == 14 {
                let e = row.value.unwrap_or(0.0) as f32;
                pe_mean += alpha * (e - pe_mean);
                pe_std = (pe_std + alpha * (e - pe_mean).powi(2)).max(1e-6).sqrt();
                let bound = pe_mean + 2.0 * pe_std;
                if e > bound { over += 1; }
                last_t = row.t;
            }
        }
    }
    println!("sustained-over episodes would-be: {over} (last t {last_t})");
    if over > 0 {
        // recompute with timestamps
    }
}
