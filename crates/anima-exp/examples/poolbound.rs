//! Read-only: at the block boundary (C-block onset t~45000 in blocked runs),
//! per neuron: pool occupancy, reserved status, eligible-waiting; count
//! slots whose pre is an input channel (id<24) vs recurrent, and cohort
//! (A<8, C 8..16). Emits boundary rows only (t in {44000,45000,46000}).
fn main() {
    for dir in std::env::args().skip(1) {
        let reader = anima_telemetry::TelemetryReader::open(
            &std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut by_t: std::collections::BTreeMap<u64, Vec<(u32, f32, bool, bool)>> =
            std::collections::BTreeMap::new(); // (pre, w, reserved, waiting)
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(env) = row.envelope("e") {
                    if let anima_telemetry::events::Payload::CandidatePool { pools } = &env.payload {
                        if row.t == 44000 || row.t == 45000 || row.t == 46000 {
                            let mut slots = Vec::new();
                            for p in pools {
                                for s in &p.slots {
                                    slots.push((s.0, s.1, s.2, s.3));
                                }
                            }
                            by_t.insert(row.t, slots);
                        }
                    }
                }
            }
        }
        for (t, slots) in &by_t {
            let n_input = slots.iter().filter(|s| s.0 < 24).count();
            let n_recur = slots.len() - n_input;
            let n_a = slots.iter().filter(|s| s.0 < 8).count();
            let n_c = slots.iter().filter(|s| s.0 >= 8 && s.0 < 16).count();
            let n_res = slots.iter().filter(|s| s.2).count();
            let n_wait = slots.iter().filter(|s| s.3).count();
            let free_slots = slots.iter().filter(|s| !s.2 && !s.3).count();
            println!("B\t{}\tslots={}\tinput={}\trecur={}\tA={}\tC={}\treserved={}\twaiting={}\tfree={}",
                t, slots.len(), n_input, n_recur, n_a, n_c, n_res, n_wait, free_slots);
        }
        // Cochort of C (8..15) slots at onset with w
        if let Some(slots) = by_t.get(&44000) {
            let cslots: Vec<(u32, f32, bool)> = slots.iter()
                .filter(|s| s.0 >= 8 && s.0 < 16)
                .map(|s| (s.0, s.1, s.2)).collect();
            let with_w = cslots.len();
            let meanw = if with_w > 0 {
                cslots.iter().map(|s| s.1 as f64).sum::<f64>() / with_w as f64
            } else { 0.0 };
            println!("C-AT-ONSET\t{}\t{}\t{:.4}", dir.rsplit('/').next().unwrap(), with_w, meanw);
        }
        println!();
    }
}
