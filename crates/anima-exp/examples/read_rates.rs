fn main() {
    let path = std::env::args().nth(1).expect("snapshot path");
    let snaps = anima_telemetry::recorder::read_snapshots(std::path::Path::new(&path)).unwrap();
    for s in &snaps {
        let rates: Vec<f32> = s.neurons.iter()
            .filter(|n| n.class == "internal")
            .filter_map(|n| n.rate_hz)
            .collect();
        let mean = rates.iter().sum::<f32>() / rates.len().max(1) as f32;
        let max = rates.iter().cloned().fold(0.0f32, f32::max);
        println!("t={:>7} mean={:>7.1} max={:>7.1}", s.tick, mean, max);
    }
}
