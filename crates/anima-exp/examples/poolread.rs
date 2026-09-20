//! Read-only: CandidatePool telemetry (dormant-reserve instrumentation).
//! Per run: per CandidatePool row emit (tick, n_slots, n_reserved,
//! n_eligible_waiting, n_neurons); plus per-block summaries at the end.
fn main() {
    for dir in std::env::args().skip(1) {
        let reader = anima_telemetry::TelemetryReader::open(
            &std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut n_rows = 0usize;
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(env) = row.envelope("e") {
                    if let anima_telemetry::events::Payload::CandidatePool { pools } = &env.payload {
                        let mut n_slots = 0usize;
                        let mut n_res = 0usize;
                        let mut n_wait = 0usize;
                        for p in pools {
                            n_slots += p.slots.len();
                            for s in &p.slots {
                                if s.2 { n_res += 1; }
                                if s.3 { n_wait += 1; }
                            }
                        }
                        println!("POOL\t{}\t{}\t{}\t{}\t{}", row.t, n_slots, n_res, n_wait, pools.len());
                        n_rows += 1;
                    }
                }
            }
        }
        eprintln!("{}: pool rows={}", dir.rsplit('/').next().unwrap(), n_rows);
        println!();
    }
}