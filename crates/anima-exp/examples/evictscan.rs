//! Read-only: audit M5 budget-evictions against the consolidated flag.
//! For each run: count budget-eviction events; for each, find the last
//! snapshot at or before the event and check whether the evicted synapse
//! was consolidated at that time (snapshots at 1 kHz grid; eviction
//! happens on structural windows inside [prev_snap, evict_tick]).
fn main() {
    for dir in std::env::args().skip(1) {
        let snaps = anima_telemetry::recorder::read_snapshots(
            &std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let reader = anima_telemetry::TelemetryReader::open(
            &std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut evicts: Vec<(u64, u32)> = Vec::new(); // (tick, synapse id)
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(env) = row.envelope("e") {
                    if let anima_telemetry::events::Payload::SynapsePruned { syn, reason } = env.payload {
                        if reason.trigger == "budget-eviction" {
                            evicts.push((row.t, syn.0));
                        }
                    }
                }
            }
        }
        let mut cons_hit = 0u64;
        let mut not_found = 0u64;
        let mut checked = 0u64;
        for (t, sid) in &evicts {
            // last snapshot with tick <= event tick and (tick - snap) <= 1050
            let mut prev: Option<&anima_telemetry::recorder::NetworkStateSnapshot> = None;
            for s in &snaps {
                if s.tick <= *t && t.saturating_sub(s.tick) <= 1050 {
                    prev = Some(s);
                }
            }
            match prev {
                Some(s) => {
                    match s.synapses.iter().find(|x| x.id == *sid) {
                        Some(syn) => {
                            checked += 1;
                            if syn.consolidated { cons_hit += 1; }
                        }
                        None => { not_found += 1; } // not live in frame (newer/older)
                    }
                }
                None => { not_found += 1; }
            }
        }
        println!("{}: evictions={} checked={} consolidated_victims={} unresolved={}",
            dir.rsplit('/').next().unwrap(), evicts.len(), checked, cons_hit, not_found);
    }
}
