fn main() {
    let path = std::env::args().nth(1).unwrap();
    let snaps = anima_telemetry::recorder::read_snapshots(std::path::Path::new(&path)).unwrap();
    println!("{}", serde_json::to_string_pretty(&snaps[1]).unwrap());
}
