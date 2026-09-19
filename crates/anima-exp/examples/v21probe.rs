//! V2.1 Stage B probe analysis: u existence/decay + endogenous silence-firing.
//! Schedule: S0 5000 silence | S1 = 40xA (start 5000, cadence 2000 => last A ends ~87500) | S2 = 20000 silence.
//! Usage: v21probe <run-dir> <beta> <tau_ms>
use std::collections::BTreeMap;
fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
    let idx = reader.chunk_index();
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    let mut fails = 0u64;
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            if row.kind == 3 { if let Some(n) = row.n { if n >= 24 { spikes.push((row.t, n as u32)); } } }
            if row.kind == 17 { fails += 1; }
        }
    }
    let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
    // S2 window: find it from markers? schedule known: S2 starts = 5000 + 40*2000 = 85000. Total = 105000.
    let s2_start = 85_000u64; let s2_end = 105_000u64;
    // B1: u decay from snapshots in S2
    let u_sum = |t: u64| -> f64 {
        snaps.iter().find(|s| s.tick == t).map(|s| s.neurons.iter().map(|n| n.u_slow.unwrap_or(0.0) as f64).sum()).unwrap_or(f64::NAN)
    };
    let mut decay_line = String::new();
    for t in [85_000u64, 87_500, 90_000, 95_000, 100_000] {
        decay_line.push_str(&format!("t={t}:{:.4} ", u_sum(t)));
    }
    // B2: endogenous firing in S2 per neuron
    let mut per_neuron: BTreeMap<u32, u64> = BTreeMap::new();
    for &(t, n) in &spikes { if t >= s2_start && t < s2_end { *per_neuron.entry(n).or_insert(0) += 1; } }
    let total: u64 = per_neuron.values().sum();
    let n_active = per_neuron.iter().filter(|(_, &c)| c > 0).count();
    // rate over S2 (20 s): Hz = count/20
    let max_rate = snaps.iter().flat_map(|s| s.neurons.iter().filter(|n| n.class == "internal").filter_map(|n| n.rate_hz)).fold(0.0f32, f32::max);
    println!("dir={} u_decay[{}] endogenous_spikes_20s={} neurons_firing={} mean_hz={:.2} max_snap_rate={:.1} fails={fails} last_active_t={:?}",
        dir.split('/').last().unwrap(), decay_line, total, n_active, total as f64 / 20.0, max_rate,
        spikes.iter().filter(|(t, _)| *t >= s2_start).map(|(t, _)| *t).max());
    // u at end of drive
    println!("  u(85000)={:.4} u(105000)={:.4}", u_sum(85_000), u_sum(105_000));
}
