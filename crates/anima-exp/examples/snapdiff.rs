//! Identity-gate snapshot comparison: per-frame JSON SHA-256 of two
//! snapshots.bin.zst files (read-only). Reports differing frame count +
//! first differences. Load-bearing for the CLLA identity gate.
fn main() {
    let a = std::env::args().nth(1).unwrap();
    let b = std::env::args().nth(2).unwrap();
    let sa = anima_telemetry::recorder::read_snapshots(std::path::Path::new(&a)).unwrap();
    let sb = anima_telemetry::recorder::read_snapshots(std::path::Path::new(&b)).unwrap();
    println!("frames: {} vs {}", sa.len(), sb.len());
    use sha2::Digest;
    let mut ndiff = 0usize;
    for (x, y) in sa.iter().zip(sb.iter()) {
        let jx = serde_json::to_vec(x).unwrap();
        let jy = serde_json::to_vec(y).unwrap();
        let hx = format!("{:x}", sha2::Sha256::digest(&jx));
        let hy = format!("{:x}", sha2::Sha256::digest(&jy));
        if hx != hy {
            ndiff += 1;
            if ndiff <= 6 {
                println!("DIFF tick {}: {} vs {}", x.tick, &hx[..12], &hy[..12]);
            }
        }
    }
    println!("differing frames: {ndiff}");
    if sa.len() != sb.len() {
        println!("frame count differs (leftover or missing frames)");
    }
}