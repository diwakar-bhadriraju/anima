//! E21 temporal-capacity analysis (docs/anima-e21-protocol.md, frozen).
//! Telemetry-only. D per window + split-half noise floor NF (L).
//! Usage: e21_analysis <run-dir> <gap-ms>
use std::collections::BTreeMap;
const N_IN: u32 = 24;
const OFF: u64 = 1;

fn main() {
    let dir = std::env::args().nth(1).expect("run dir");
    let g: u64 = std::env::args().nth(2).expect("gap ms").parse().unwrap();
    let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
    let idx = reader.chunk_index();
    let mut stims: Vec<(String, u64)> = Vec::new();
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    let (mut perm, mut fail) = (0u64, 0u64);
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                5 => { if let Ok(e) = row.envelope("e") { if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, .. } = e.payload { stims.push((pattern_id, e.t)); } } }
                6 => { if let Ok(e) = row.envelope("e") { if let anima_telemetry::events::Payload::SynapseCreated { reason, .. } = e.payload { if reason.trigger == "candidate-permanence" { perm += 1; } } } }
                17 => { if let Ok(_) = row.envelope("e") { fail += 1; } }
                _ => {}
            }
        }
    }
    let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).as_ref().map(|_| ()).ok();
    let _ = snaps;
    let max_rate = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap()
        .iter().flat_map(|s| s.neurons.iter().filter(|n| n.class == "internal").filter_map(|n| n.rate_hz)).fold(0.0f32, f32::max);
    let base = |k: u64| 5000 + 2000 * k;
    let vec_of = |lo: u64, hi: u64| -> BTreeMap<u32, f64> {
        let mut v = BTreeMap::new();
        for &(t, n) in &spikes { if t >= lo && t < hi && n >= N_IN && n < 64 { *v.entry(n).or_insert(0.0) += 1.0; } }
        v
    };
    // antecedent identity per trial from stims (ante at base+OFF)
    let is_a: Vec<bool> = (0..200u64).map(|k| {
        stims.iter().find(|(p, t)| *t == base(k) + OFF && (p == "A" || p == "C")).map(|(p, _)| p == "A").unwrap_or(false)
    }).collect();
    let na = is_a.iter().filter(|&&x| x).count();
    let probe_lo = |k: u64| base(k) + 500 + g + OFF;
    let win = |lo: u64, hi: u64, cond: &dyn Fn(bool) -> bool| -> Vec<BTreeMap<u32, f64>> {
        (lo..hi).filter(|&k| cond(is_a[k as usize])).map(|k| vec_of(probe_lo(k), probe_lo(k) + 500)).collect()
    };
    let pm = |x: &[BTreeMap<u32, f64>], y: &[BTreeMap<u32, f64>]| -> f64 {
        let mut cs = Vec::new();
        for a in x { for b in y { cs.push(cos(a, b)); } }
        cs.iter().sum::<f64>() / cs.len().max(1) as f64
    };
    let d = |lo: u64, hi: u64| 1.0 - pm(&win(lo, hi, &|c| c), &win(lo, hi, &|c| !c));
    let (de, dm, dl) = (d(0, 40), d(80, 120), d(160, 200));
    // split-half NF on L: within-condition
    let a_all: Vec<usize> = (160..200usize).filter(|&k| is_a[k]).collect();
    let c_all: Vec<usize> = (160..200usize).filter(|&k| !is_a[k]).collect();
    let vh = |ks: &[usize]| -> Vec<BTreeMap<u32, f64>> { ks.iter().map(|&k| vec_of(probe_lo(k as u64), probe_lo(k as u64) + 500)).collect() };
    let (a_x, a_y) = (vh(&a_all[..a_all.len()/2]), vh(&a_all[a_all.len()/2..]));
    let (c_x, c_y) = (vh(&c_all[..c_all.len()/2]), vh(&c_all[c_all.len()/2..]));
    let nf = ((1.0 - pm(&a_x, &a_y)) + (1.0 - pm(&c_x, &c_y))) / 2.0;
    // A-C sanity (L antecedent epochs)
    let av: Vec<BTreeMap<u32, f64>> = (160..200u64).filter(|&k| is_a[k as usize]).map(|k| vec_of(base(k) + OFF, base(k) + OFF + 500)).collect();
    let cv: Vec<BTreeMap<u32, f64>> = (160..200u64).filter(|&k| !is_a[k as usize]).map(|k| vec_of(base(k) + OFF, base(k) + OFF + 500)).collect();
    let ac = pm(&av, &cv);
    println!("gap={g}ms: D_E={de:.4} D_M={dm:.4} D_L={dl:.4} NF={nf:.4} RETAINED(D_L-NF={:.4}{}) A-C={ac:.4} perm={perm} fail={fail} maxrate={max_rate:.1} balance={na}/{}", dl - nf, if dl - nf > 0.05 { " YES" } else { " no" }, 200 - na);
}
fn cos(a: &BTreeMap<u32, f64>, b: &BTreeMap<u32, f64>) -> f64 {
    use std::collections::BTreeSet;
    let keys: BTreeSet<u32> = a.keys().chain(b.keys()).copied().collect();
    let (mut dot, mut na, mut nb) = (0.0f64, 0.0f64, 0.0f64);
    for k in keys {
        let (x, y) = (a.get(&k).copied().unwrap_or(0.0), b.get(&k).copied().unwrap_or(0.0));
        dot += x * y; na += x * x; nb += y * y;
    }
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na.sqrt() * nb.sqrt()) }
}
