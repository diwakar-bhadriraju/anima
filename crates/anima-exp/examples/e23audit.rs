//! E23 read-only causal audit (committed runs only).
//! Telemetry+snapshots only; no organism contact.
//! Usage: e23audit <run-dir> <arm>
use std::collections::BTreeMap;
const N_IN: u32 = 24;
const OFF: u64 = 1;

fn main() {
    let dir = std::env::args().nth(1).expect("run dir");
    let _arm = std::env::args().nth(2).unwrap_or_default();
    let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
    let idx = reader.chunk_index();
    let mut stims: Vec<(String, u64)> = Vec::new();
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    let mut out_spikes: Vec<(u64, u32)> = Vec::new();
    let mut stdp_up: Vec<(u64, u32, f32)> = Vec::new();
    let mut stdp_dn: Vec<(u64, u32, f32)> = Vec::new();
    let mut created: Vec<(u64, u32, u32, u32, String)> = Vec::new();
    let mut pruned: Vec<(u64, u32, String)> = Vec::new();
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                4 => out_spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                5 => { if let Ok(e) = row.envelope("e") { if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, .. } = e.payload { stims.push((pattern_id, e.t)); } } }
                6 => { if let Ok(e) = row.envelope("e") { if let anima_telemetry::events::Payload::SynapseCreated { syn, pre, post, reason, .. } = e.payload { created.push((e.t, syn.0, pre.0, post.0, reason.trigger)); } } }
                7 => { if let Ok(e) = row.envelope("e") { if let anima_telemetry::events::Payload::SynapsePruned { syn, reason } = e.payload { pruned.push((e.t, syn.0, reason.trigger)); } } }
                8 => { if let Ok(e) = row.envelope("e") { if let anima_telemetry::events::Payload::SynapseStrengthened { syn, delta } = e.payload { stdp_up.push((e.t, syn.0, delta.unwrap_or(0.0))); } } }
                9 => { if let Ok(e) = row.envelope("e") { if let anima_telemetry::events::Payload::SynapseWeakened { syn, delta } = e.payload { stdp_dn.push((e.t, syn.0, delta.unwrap_or(0.0))); } } }
                _ => {}
            }
        }
    }
    let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
    let base = |k: u64| 5000 + 2000 * k;
    let is_a: Vec<bool> = (0..200u64).map(|k| stims.iter().find(|(p, t)| *t == base(k) + OFF && (p == "A" || p == "C")).map(|(p, _)| p == "A").unwrap_or(false)).collect();

    // 12) FINE TRAJECTORY: per-trial vote sign relative to antecedent (rolling)
    let world_log = std::fs::read_to_string(std::path::Path::new(&dir).join("e19-world.log")).unwrap_or_default();
    println!("== fine trajectory (per-10-trial blocks: P(g1|A), P(g1|C), benign rate)");
    for b in (0..200u64).step_by(10) {
        let (mut a1, mut an, mut c1, mut cn, mut ben) = (0u32, 0u32, 0u32, 0u32, 0u32);
        for k in b..b + 10 {
            let line = world_log.lines().find(|l| l.starts_with(&format!("trial={} ", k + 1))).unwrap_or("");
            let vote = line.split("vote").nth(1).unwrap_or("").trim();
            let g1: u32 = vote.split_whitespace().next().and_then(|t| t.split('=').nth(1)).and_then(|v| v.parse().ok()).unwrap_or(0);
            let g2: u32 = vote.split_whitespace().nth(1).and_then(|t| t.split('=').nth(1)).and_then(|v| v.parse().ok()).unwrap_or(0);
            let d = vote.split("->").nth(1).unwrap_or("x").trim();
            if d == "Match" { ben += 1; }
            if g1 > g2 { if is_a[k as usize] { a1 += 1; an += 1; } else { c1 += 1; cn += 1; } }
            else if g2 > g1 { if is_a[k as usize] { an += 1; } else { cn += 1; } }
        }
        println!("trials {:3}-{:-3}: P(g1|A)={:.2} P(g1|C)={:.2} benign={}/10", b + 1, b + 10, if an > 0 { a1 as f64 / an as f64 } else { -1.0 }, if cn > 0 { c1 as f64 / cn as f64 } else { -1.0 }, ben);
    }

    // 3/8) STDP event timing relative to the consequence window [500,1000)
    // and the vote/stimulus window [0,500). Sum deltas by epoch.
    let epoch_of = |t: u64| -> u8 { // trial-relative
        let k = ((t - OFF) - 5000) / 2000;
        let off = ((t - OFF) - 5000) % 2000;
        let _ = k;
        if off < 500 { 0 } else if off < 1000 { 1 } else { 2 } // stimulus / consequence / ITI
    };
    let (mut su0, mut su1, mut su2, mut sd0, mut sd1, mut sd2) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
    for (t, _, d) in &stdp_up { match epoch_of(*t) { 0 => su0 += *d as f64, 1 => su1 += *d as f64, _ => su2 += *d as f64 } }
    for (t, _, d) in &stdp_dn { match epoch_of(*t) { 0 => sd0 += *d as f64, 1 => sd1 += *d as f64, _ => sd2 += *d as f64 } }
    println!("== STDP summed deltas by trial epoch: stim: +{su0:.3}/-{:.3}  CONSEQ: +{su1:.3}/-{:.3}  ITI: +{su2:.3}/-{:.3}", -sd0, -sd1, -sd2);

    // Which SYNAPSES are strengthened in the consequence epoch? Join with the
    // latest snapshot ≤ t to get pre/post. Output→input and internal→input paths.
    let snap_at = |t: u64| -> Option<&anima_telemetry::recorder::NetworkStateSnapshot> {
        snaps.iter().rev().find(|s| s.tick <= t)
    };
    let mut conseq_paths: BTreeMap<String, f64> = BTreeMap::new();
    for (t, syn, d) in &stdp_up {
        if epoch_of(*t) == 1 {
            if let Some(s) = snap_at(*t) {
                if let Some(sy) = s.synapses.iter().find(|x| x.id == *syn) {
                    let pre_class = if sy.pre < 24 { "IN" } else if sy.pre < 64 { "INT" } else { "OUT" };
                    let post_class = if sy.post < 24 { "IN" } else if sy.post < 64 { "INT" } else { "OUT" };
                    *conseq_paths.entry(format!("{pre_class}->{post_class}")).or_insert(0.0) += *d as f64;
                }
            }
        }
    }
    println!("== consequence-epoch LTP by pathway: {conseq_paths:?}");

    // 1/4) Pathway weights EARLY vs LATE (snapshots at 5000+40*2000 vs 5000+200*2000):
    let w_at = |tick: u64| -> BTreeMap<(u32, u32), f32> {
        let s = snaps.iter().find(|s| s.tick == tick).expect("snapshot");
        s.synapses.iter().filter(|x| x.plastic).map(|x| ((x.pre, x.post), x.w.unwrap_or(0.0))).collect()
    };
    let t_early = 5000 + 40 * 2000;
    let t_late = 5000 + 160 * 2000;
    let (we, wl) = (w_at(t_early), w_at(t_late));
    let mut dw_paths: BTreeMap<String, (f64, usize)> = BTreeMap::new();
    for (k, w0) in &we {
        if let Some(w1) = wl.get(k) {
            let pre_class = if k.0 < 24 { "IN" } else if k.0 < 64 { "INT" } else { "OUT" };
            let post_class = if k.1 < 24 { "IN" } else if k.1 < 64 { "INT" } else { "OUT" };
            let e = dw_paths.entry(format!("{pre_class}->{post_class}")).or_insert((0.0, 0));
            e.0 += (*w1 - *w0) as f64;
            e.1 += 1;
        }
    }
    println!("== pathway dW (E->L snapshots, plastic synapses alive both): {dw_paths:?}");

    // Output-neuron afferents by sensory side: A-side (0-7) vs C-side (8-15) weight sums, early vs late
    let out_aff = |tick: u64| -> (f64, f64) {
        let s = snaps.iter().find(|s| s.tick == tick).expect("snapshot");
        let (mut a, mut c) = (0.0f64, 0.0f64);
        for x in s.synapses.iter().filter(|x| x.plastic && x.post >= 64) {
            let w = x.w.unwrap_or(0.0) as f64;
            if x.pre < 8 { a += w; } else if (8..16).contains(&x.pre) { c += w; }
        }
        (a, c)
    };
    let (ae, ce) = out_aff(t_early);
    let (al, cl) = out_aff(t_late);
    println!("== output-neuron afferents: A-side(0-7) {ae:.3}->{al:.3} (d={:+.3}); C-side(8-15) {ce:.3}->{cl:.3} (d={:+.3})", al - ae, cl - ce);
    // per group
    let grp_aff = |tick: u64, g: &std::ops::Range<u32>| -> (f64, f64) {
        let s = snaps.iter().find(|s| s.tick == tick).expect("snapshot");
        let (mut a, mut c) = (0.0f64, 0.0f64);
        for x in s.synapses.iter().filter(|x| x.plastic && g.contains(&x.post)) {
            let w = x.w.unwrap_or(0.0) as f64;
            if x.pre < 8 { a += w; } else if (8..16).contains(&x.pre) { c += w; }
        }
        (a, c)
    };
    let (g1a0, g1c0) = grp_aff(t_early, &(64u32..70));
    let (g1a1, g1c1) = grp_aff(t_late, &(64u32..70));
    let (g2a0, g2c0) = grp_aff(t_early, &(70u32..76));
    let (g2a1, g2c1) = grp_aff(t_late, &(70u32..76));
    println!("== g1(64-69) A-side {g1a0:.3}->{g1a1:.3} C-side {g1c0:.3}->{g1c1:.3}");
    println!("== g2(70-75) A-side {g2a0:.3}->{g2a1:.3} C-side {g2c0:.3}->{g2c1:.3}");

    // 7) structural: creations/prunes by epoch
    let (mut c0, mut c1, mut c2) = (0u64, 0u64, 0u64);
    for (t, _, _, _, _) in &created { match epoch_of(*t) { 0 => c0 += 1, 1 => c1 += 1, _ => c2 += 1 } }
    println!("== M3 maturations by epoch: stim={c0} conseq={c1} iti={c2} (total {})", created.len());
    let (mut p0, mut p1, mut p2) = (0u64, 0u64, 0u64);
    for (t, _, _) in &pruned { match epoch_of(*t) { 0 => p0 += 1, 1 => p1 += 1, _ => p2 += 1 } }
    println!("== prunes by epoch: stim={p0} conseq={p1} iti={p2} (total {})", pruned.len());

    // 2) output activity profile early vs late, per stimulus type (first 500ms)
    let out_rate = |lo: u64, hi: u64, g: &std::ops::Range<u32>, cond: &dyn Fn(bool) -> bool| -> f64 {
        let (mut n, mut tr) = (0u64, 0u64);
        for k in lo..hi {
            if !cond(is_a[k as usize]) { continue; }
            tr += 1;
            let b = base(k) + OFF;
            n += out_spikes.iter().filter(|(t, x)| *t >= b && *t < b + 500 && g.contains(x)).count() as u64;
        }
        n as f64 / tr.max(1) as f64
    };
    println!("== output spikes/trial (stimulus epoch): E g1 A={:.1} C={:.1} g2 A={:.1} C={:.1} | L g1 A={:.1} C={:.1} g2 A={:.1} C={:.1}",
        out_rate(0, 40, &(64..70), &|c| c), out_rate(0, 40, &(64..70), &|c| !c), out_rate(0, 40, &(70..76), &|c| c), out_rate(0, 40, &(70..76), &|c| !c),
        out_rate(160, 200, &(64..70), &|c| c), out_rate(160, 200, &(64..70), &|c| !c), out_rate(160, 200, &(70..76), &|c| c), out_rate(160, 200, &(70..76), &|c| !c));
}
