// Empirical I_aff: for one run, over drive windows, per-tick total delivered
// input current (amplitude*w) vs the gate's sum(w), per internal neuron.
// Uses telemetry input spikes (n<24) + drive-end snapshot weights (proxy).
fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
    let idx = reader.chunk_index();
    let mut inp_spikes: Vec<(u64, u32)> = Vec::new(); // (t, input neuron id)
    let mut pres: Vec<(String, u64)> = Vec::new();
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                5 => { if let Some(env) = row.envelope("e").ok() {
                    if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } = env.payload {
                        if stage == "S1" { pres.push((pattern_id, env.t)); } } } }
                3 => { if let Some(n) = row.n { if n < 24 { inp_spikes.push((row.t, n as u32)); } } }
                _ => {}
            }
        }
    }
    pres.sort_by_key(|p| p.1);
    let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
    let mid = snaps.iter().find(|s| s.tick >= 44_000).unwrap_or(snaps.last().unwrap());
    // input->internal weight map (post -> sum w, per pre neuron id)
    let mut w: std::collections::BTreeMap<(u32, u32), f32> = std::collections::BTreeMap::new();
    for syn in &mid.synapses {
        if syn.pre < 24 && syn.post >= 24 { w.insert((syn.pre, syn.post), syn.w.unwrap_or(0.0)); }
    }
    // during a 500ms A presentation, per internal neuron: count input spikes + delivered current
    let amp = 52.0f32;
    let (_, a_t) = pres.iter().find(|(p, _)| p == "A").unwrap().clone();
    let t0 = a_t;
    let mut per_tick: std::collections::BTreeMap<u32, (f32, f32)> = std::collections::BTreeMap::new(); // post -> (sum w, sum amp*w)
    for &(t, n) in &inp_spikes {
        if t >= t0 && t < t0 + 500 {
            for ((pre, post), wv) in &w { let _ = post; let _ = wv; if *pre == n {
                    let e = per_tick.entry(*post).or_insert((0.0, 0.0));
                    e.0 += wv;
                    e.1 += amp * wv;
                }
            }
        }
    }
    let vals: Vec<(u32, (f32, f32))> = per_tick.iter().map(|(k, v)| (*k, *v)).collect();
    let n = vals.len();
    if n == 0 { println!("no mapped afferents"); return; }
    let mean_w = vals.iter().map(|(_, (a, _))| a).sum::<f32>() / n as f32;
    let mean_amp = vals.iter().map(|(_, (_, b))| b).sum::<f32>() / n as f32;
    println!("A-window, {n} internal neurons with input activity:");
    let mut ws: Vec<f32> = vals.iter().map(|(_,(a,_))| *a).collect(); ws.sort_by(|x,y| x.partial_cmp(y).unwrap());
    let mut as_: Vec<f32> = vals.iter().map(|(_,(_,b))| *b).collect(); as_.sort_by(|x,y| x.partial_cmp(y).unwrap());
    println!("  I_aff gate (sum w):   min {:.4} med {:.4} max {:.4}", ws[0], ws[ws.len()/2], ws[ws.len()-1]);
    println!("  I_aff membrane (amp*w): min {:.2} med {:.2} max {:.2}", as_[0], as_[as_.len()/2], as_[as_.len()-1]);
}
