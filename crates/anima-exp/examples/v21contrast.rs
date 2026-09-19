//! Post-V2.1 exploration P2: antecedent conditioning of u and endogenous activity.
//!
//! Input: an xp2-style run (S0 silence | S1 interleaved A/C drive | S2 silence).
//! Questions:
//!  (1) Is the u vector at S2 onset different after an A-last vs C-last drive?
//!      -> cosine between end-of-drive u vectors for A-last vs C-last runs is
//!         not available (single run interleaves); instead measure WITHIN run:
//!         u vectors after each A vs each C presentation (snapshot resolution
//!         1000 ticks) -> per-class mean vector, cross-class cosine, and
//!         per-class separation vs within-class baseline.
//!  (2) Is endogenous S2 spiking conditioned by the last antecedent?
//!      (single-run: only the final antecedent exists; report it + timing)
//!  (3) Does u track WHICH channel-set was active (A: ch0-7, C: ch8-15)?
//!      -> correlation of u-space position with afferent-class projection.
//! Read-only instrument. Deterministic.
use std::collections::BTreeMap;

fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
    let idx = reader.chunk_index();
    let mut pres: Vec<(String, u64)> = Vec::new(); // (pattern, t)
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                5 => {
                    if let Some(env) = row.envelope("e").ok() {
                        if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } = env.payload {
                            if stage == "S1" { pres.push((pattern_id, env.t)); }
                        }
                    }
                }
                3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                _ => {}
            }
        }
    }
    pres.sort_by_key(|p| p.1);
    spikes.sort_by_key(|s| s.0);
    let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
    let snap_ticks: Vec<u64> = snaps.iter().map(|s| s.tick).collect();

    // u vector at the first snapshot AFTER each presentation end (pres end = t+500).
    let u_after = |tend: u64| -> Option<Vec<f32>> {
        let tick = *snap_ticks.iter().find(|&&t| t >= tend + 400)?; // snapshot covering [end-100..end]
        let _ = tick;
        // snapshots carry state AT tick; find last snapshot <= tend+500 (post-stimulus decay in u negligible at tau>=5000 over <=500ms? report raw)
        let t = *snap_ticks.iter().rev().find(|&&t| t <= tend + 500)?;
        snaps.iter().find(|s| s.tick == t).map(|s| s.neurons.iter().map(|n| n.u_slow.unwrap_or(0.0)).collect())
    };

    let mut by_class: BTreeMap<String, Vec<Vec<f32>>> = BTreeMap::new();
    for (p, t) in &pres {
        if let Some(u) = u_after(*t + 500) { by_class.entry(p.clone()).or_default().push(u); }
    }
    let mean_vec = |vs: &Vec<Vec<f32>>| -> Vec<f32> {
        let n = vs[0].len();
        (0..n).map(|i| vs.iter().map(|v| v[i]).sum::<f32>() / vs.len() as f32).collect()
    };
    let cos = |a: &[f32], b: &[f32]| -> f64 {
        let d: f64 = a.iter().zip(b).map(|(x, y)| (*x as f64) * (*y as f64)).sum();
        let na = (a.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
        let nb = (b.iter().map(|x| (*x as f64).powi(2)).sum::<f64>()).sqrt();
        if na > 0.0 && nb > 0.0 { d / (na * nb) } else { 0.0 }
    };
    let classes: Vec<String> = by_class.keys().cloned().collect();
    for (i, c) in classes.iter().enumerate() {
        let vs = &by_class[c];
        println!("class {c}: n={}", vs.len());
        // within-class: split-half
        if vs.len() >= 4 {
            let h = vs.len() / 2;
            let (a, b) = (&vs[..h], &vs[h..]);
            println!("  within-{c} split-half cosine: {:.4}", cos(&mean_vec(&a.to_vec()), &mean_vec(&b.to_vec())));
        }
        for c2 in classes.iter().skip(i + 1) {
            println!("  cross-{c}-{c2} mean-vector cosine: {:.4}", cos(&mean_vec(&by_class[c]), &mean_vec(&by_class[c2])));
        }
    }
    // per-presentation separation: fraction of A-pairs closer (cosine) than A-C pairs
    if classes.len() == 2 {
        let (ca, cc) = (classes[0].clone(), classes[1].clone());
        let (va, vc) = (&by_class[&ca], &by_class[&cc]);
        if va.len() >= 2 && vc.len() >= 2 {
            let mut within: Vec<f64> = Vec::new();
            for i in 0..va.len() { for j in (i+1)..va.len() { within.push(cos(&va[i], &va[j])); } }
            let mut cross: Vec<f64> = Vec::new();
            for x in va { for y in vc { cross.push(cos(x, y)); } }
            let m = |v: &Vec<f64>| v.iter().sum::<f64>() / v.len() as f64;
            println!("A-A cos mean {:.4} | A-C cos mean {:.4} | gap {:+.4}", m(&within), m(&cross), m(&within) - m(&cross));
        }
    }
    // last antecedent + silence activity onset/structure
    let (last_p, last_t) = pres.last().unwrap().clone();
    let sil_start = last_t + 500;
    let end = spikes.last().map(|s| s.0 + 1).unwrap_or(sil_start);
    let sil: Vec<(u64, u32)> = spikes.iter().filter(|(t, n)| *t >= sil_start && *n >= 24).cloned().collect();
    println!("last antecedent: {last_p} (t={last_t}); silence [{sil_start},{end}) spikes(non-input)={}", sil.len());
    if !sil.is_empty() {
        let first = sil[0].0;
        println!("first endogenous spike at silence+{}ms", first - sil_start);
        let mut per: BTreeMap<u32, u64> = BTreeMap::new();
        for &(_, n) in &sil { *per.entry(n).or_default() += 1; }
        let top: Vec<_> = { let mut v: Vec<_> = per.into_iter().collect(); v.sort_by_key(|(_, c)| std::cmp::Reverse(*c)); v };
        println!("active neurons: {} | top: {:?}", top.len(), top.iter().take(8).map(|(n, c)| format!("{}:{}", n, c)).collect::<Vec<_>>());
        // firing over silence in 2s bins (does it decay?)
        let nb = ((end - sil_start) / 2000).max(1) as usize;
        let mut b = vec![0u64; nb];
        for &(t, _) in &sil { let i = ((t - sil_start) / 2000) as usize; if i < nb { b[i] += 1; } }
        println!("spikes per 2s bin: {:?}", b);
    }
}
// (analysis extension lives in v21contra2.rs)
