//! E23 reflex-shaping analysis (docs/anima-e23-protocol.md, frozen).
//! Usage: e23_analysis <run-dir>
use std::collections::BTreeMap;
const N_IN: u32 = 24;
const OFF: u64 = 1;
fn main() {
    let dir = std::env::args().nth(1).expect("run dir");
    let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
    let idx = reader.chunk_index();
    let mut stims: Vec<(String, u64)> = Vec::new();
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    let mut out_spikes: Vec<(u64, u32)> = Vec::new();
    let (mut perm, mut fail) = (0u64, 0u64);
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                4 => out_spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                5 => { if let Ok(e) = row.envelope("e") { if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, .. } = e.payload { stims.push((pattern_id, e.t)); } } }
                6 => { if let Ok(e) = row.envelope("e") { if let anima_telemetry::events::Payload::SynapseCreated { reason, .. } = e.payload { if reason.trigger == "candidate-permanence" { perm += 1; } } } }
                17 => { if let Ok(_) = row.envelope("e") { fail += 1; } }
                _ => {}
            }
        }
    }
    let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
    let max_rate = snaps.iter().flat_map(|s| s.neurons.iter().filter(|n| n.class == "internal").filter_map(|n| n.rate_hz)).fold(0.0f32, f32::max);
    let world_log = std::fs::read_to_string(std::path::Path::new(&dir).join("e19-world.log")).unwrap_or_default();
    let base = |k: u64| 5000 + 2000 * k;
    let is_a: Vec<bool> = (0..200u64).map(|k| stims.iter().find(|(p, t)| *t == base(k) + OFF && (p == "A" || p == "C")).map(|(p, _)| p == "A").unwrap_or(false)).collect();
    let vec_of = |lo: u64, hi: u64| -> BTreeMap<u32, f64> {
        let mut v = BTreeMap::new();
        for &(t, n) in &spikes { if t >= lo && t < hi && n >= N_IN && n < 64 { *v.entry(n).or_insert(0.0) += 1.0; } }
        v
    };
    let pm = |x: &[BTreeMap<u32, f64>], y: &[BTreeMap<u32, f64>]| -> f64 {
        let mut cs = Vec::new();
        for a in x { for b in y { cs.push(cos(a, b)); } }
        cs.iter().sum::<f64>() / cs.len().max(1) as f64
    };
    for (label, lo, hi) in [("E", 0u64, 40u64), ("M", 80, 120), ("L", 160, 200)] {
        let (mut a_g1, mut a_n, mut c_g1, mut c_n, mut noact, mut benign) = (0u32, 0u32, 0u32, 0u32, 0u32, 0u32);
        let (mut g1s, mut g2s) = (0u64, 0u64);
        for k in lo..hi {
            let line = world_log.lines().find(|l| l.starts_with(&format!("trial={} ", k + 1))).unwrap_or("");
            let vote = line.split("vote").nth(1).unwrap_or("").trim();
            let g1: u32 = vote.split_whitespace().next().and_then(|t| t.split('=').nth(1)).and_then(|v| v.parse().ok()).unwrap_or(0);
            let g2: u32 = vote.split_whitespace().nth(1).and_then(|t| t.split('=').nth(1)).and_then(|v| v.parse().ok()).unwrap_or(0);
            let d = vote.split("->").nth(1).unwrap_or("NoVote").trim().to_string();
            if d == "NoAction" || d.is_empty() { noact += 1; }
            else if is_a[k as usize] { a_n += 1; a_g1 += (g1 > g2) as u32; }
            else { c_n += 1; c_g1 += (g1 > g2) as u32; }
            if d == "Match" { benign += 1; }
            let b = base(k) + OFF;
            g1s += out_spikes.iter().filter(|(t, n)| *t >= b + 100 && *t < b + 500 && (64..70).contains(n)).count() as u64;
            g2s += out_spikes.iter().filter(|(t, n)| *t >= b + 100 && *t < b + 500 && (70..76).contains(n)).count() as u64;
        }
        let pa = if a_n > 0 { a_g1 as f64 / a_n as f64 } else { 0.0 };
        let pc = if c_n > 0 { c_g1 as f64 / c_n as f64 } else { 0.0 };
        let av: Vec<_> = (lo..hi).filter(|&k| is_a[k as usize]).map(|k| vec_of(base(k) + OFF + 100, base(k) + OFF + 500)).collect();
        let cv: Vec<_> = (lo..hi).filter(|&k| !is_a[k as usize]).map(|k| vec_of(base(k) + OFF + 100, base(k) + OFF + 500)).collect();
        let d_int = 1.0 - pm(&av, &cv);
        println!("[{label}] BD={:.3} (P(g1|A)={:.3} n={}, P(g1|C)={:.3} n={}) noact={}/{} BR={:.3} outspk g1={g1s} g2={g2s} D_stim={d_int:.4}", (pa - pc).abs(), pa, a_n, pc, c_n, noact, hi - lo, benign as f64 / (hi - lo) as f64);
    }
    let av: Vec<_> = (160..200u64).filter(|&k| is_a[k as usize]).map(|k| vec_of(base(k) + OFF, base(k) + OFF + 500)).collect();
    let cv: Vec<_> = (160..200u64).filter(|&k| !is_a[k as usize]).map(|k| vec_of(base(k) + OFF, base(k) + OFF + 500)).collect();
    println!("A-C sanity (L): {:.4}", pm(&av, &cv));
    println!("perm={perm} fail={fail} maxrate={max_rate:.1}");
    let seq: Vec<u32> = is_a.iter().map(|&a| if a { 0 } else { 1 }).collect();
    let n = 200f64; let mean = seq.iter().sum::<u32>() as f64 / n;
    let num: f64 = (1..200).map(|i| (seq[i] as f64 - mean) * (seq[i - 1] as f64 - mean)).sum();
    let den: f64 = seq.iter().map(|&x| (x as f64 - mean).powi(2)).sum();
    println!("lag1={:.3}", num / den);
}
fn cos(a: &BTreeMap<u32, f64>, b: &BTreeMap<u32, f64>) -> f64 {
    use std::collections::BTreeSet;
    let keys: BTreeSet<u32> = a.keys().chain(b.keys()).copied().collect();
    let (mut dot, mut na, mut nb) = (0.0f64, 0.0f64, 0.0f64);
    for k in keys { let (x, y) = (a.get(&k).copied().unwrap_or(0.0), b.get(&k).copied().unwrap_or(0.0)); dot += x * y; na += x * x; nb += y * y; }
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na.sqrt() * nb.sqrt()) }
}
