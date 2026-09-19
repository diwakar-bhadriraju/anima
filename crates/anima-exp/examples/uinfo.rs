//! V2.1 read-only information audit on committed artifacts.
//! Q: does u carry antecedent-specific information or generic persistence?
//! Committed facts: probe runs are A-ONLY (no C trials exist in ANY v2.1 run);
//! Stage-C v21c runs DO NOT EXIST (erratum). What IS auditable:
//! (1) u state per neuron over time in the wedge runs (spatial org, saturation);
//! (2) endogenous spike patterns: structured vs generic (Fano, pairwise corr);
//! (3) u-vs-rate relationship (does u just elevate rate?);
//! (4) A-drive-period vs silence-period u/spike structure.
//! Usage: uinfo <run-dir>
fn main() {
    let dir = std::env::args().nth(1).unwrap();
    let reader = anima_telemetry::TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
    let idx = reader.chunk_index();
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            if row.kind == 3 { if let Some(n) = row.n { if n >= 24 { spikes.push((row.t, n as u32)); } } }
        }
    }
    let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
    // u per-neuron trajectory at silence start/mid/end
    let u_vec = |t: u64| -> Vec<f32> { snaps.iter().find(|s| s.tick == t).map(|s| s.neurons.iter().map(|n| n.u_slow.unwrap_or(0.0)).collect()).unwrap_or_default() };
    let us = |v: &[f32]| f64::sqrt(v.iter().map(|x| (*x as f64).powi(2)).sum::<f64>() / v.len().max(1) as f64);
    let mean = |v: &[f32]| v.iter().map(|&x| x as f64).sum::<f64>() / v.len().max(1) as f64;
    let (u0, u1, u2) = (u_vec(85_000), u_vec(95_000), u_vec(105_000));
    println!("u: mean {:+.4}->{:+.4}->{:+.4} | rms {:.4}->{:.4}->{:.4} (t=85k/95k/105k)", mean(&u0), mean(&u1), mean(&u2), us(&u0), us(&u1), us(&u2));
    // spatial: how many neurons above half the max u at each time; Gini-style concentration
    let conc = |v: &[f32]| -> f64 {
        let mut s: Vec<f64> = v.iter().map(|&x| x as f64).collect();
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let tot: f64 = s.iter().sum();
        if tot <= 0.0 { return 0.0; }
        let n = s.len() as f64;
        1.0 - 2.0 * (0..s.len()).map(|i| s[i] / tot * (1.0 - (i as f64 + 0.5) / n)).sum::<f64>()
    };
    println!("u concentration (Gini): {:.3}->{:.3}->{:.3} | neurons with u > half-max: {}/{}/{}",
        conc(&u0), conc(&u1), conc(&u2),
        u0.iter().filter(|&&x| x > 0.5 * u0.iter().cloned().fold(0.0f32, f32::max)).count(),
        u1.iter().filter(|&&x| x > 0.5 * u1.iter().cloned().fold(0.0f32, f32::max)).count(),
        u2.iter().filter(|&&x| x > 0.5 * u2.iter().cloned().fold(0.0f32, f32::max)).count());
    // internal vs output u
    let (mut ui, mut uo) = (Vec::new(), Vec::new());
    for (i, n) in snaps.iter().find(|s| s.tick == 95_000).unwrap().neurons.iter().enumerate() {
        let u = n.u_slow.unwrap_or(0.0);
        if n.class == "internal" { ui.push((i as u32, u)); } else if n.class == "output" { uo.push((i as u32, u)); }
    }
    println!("u t=95k: internal mean {:.4} max {:.4} | output mean {:.4} max {:.4}", mean(&ui.iter().map(|(_, u)| *u).collect::<Vec<_>>()), ui.iter().map(|(_, u)| *u).fold(0.0f32, f32::max), mean(&uo.iter().map(|(_, u)| *u).collect::<Vec<_>>()), uo.iter().map(|(_, u)| *u).fold(0.0f32, f32::max));
    // endogenous spiking: per-neuron counts in silence, Fano factor in 1s bins, pairwise co-fire
    let n_neurons = 76usize;
    let mut counts = vec![0u64; n_neurons];
    for &(t, n) in &spikes { if t >= 85_000 { counts[(n - 24) as usize] += 1; } }
    let active: Vec<usize> = (0..n_neurons).filter(|&i| counts[i] > 0).collect();
    let tot: u64 = counts.iter().sum();
    println!("silence spikes={tot} active_neurons={}/52-noninput... (ids 24..76)", active.len());
    // Fano in 1s bins for the 5 most active
    let mut by_sec: Vec<Vec<u64>> = vec![vec![0u64; 20]; n_neurons];
    for &(t, n) in &spikes { if t >= 85_000 && t < 105_000 { by_sec[(n - 24) as usize][((t - 85_000) / 1000) as usize] += 1; } }
    let mut ranked: Vec<usize> = (0..n_neurons).collect();
    ranked.sort_by_key(|&i| std::cmp::Reverse(counts[i]));
    for &i in ranked.iter().take(5) {
        let b = &by_sec[i];
        let m = b.iter().sum::<u64>() as f64 / 20.0;
        let var = b.iter().map(|&x| (x as f64 - m).powi(2)).sum::<f64>() / 20.0;
        println!("  neuron {}: count={} mean/s={:.2} Fano={:.3}", i + 24, counts[i], m, if m > 0.0 { var / m } else { 0.0 });
    }
    // u trajectory across the DRIVE period (t=5000..85000): does u saturate?
    let u_traj = |t: u64| mean(&u_vec(t));
    let mut line = String::new();
    for t in [5_000u64, 15_000, 25_000, 35_000, 45_000, 55_000, 65_000, 75_000, 84_000] {
        line.push_str(&format!("{t}:{:.3} ", u_traj(t)));
    }
    println!("u mean during drive: {line}");
    // pairwise: population rate correlation between halves (structured vs independent)
    let pop_rate: Vec<f64> = (0..20u64).map(|s| spikes.iter().filter(|(t, _)| (85_000 + s * 1000..85_000 + (s + 1) * 1000).contains(t)).count() as f64).collect();
    let pm = pop_rate.iter().sum::<f64>() / 20.0;
    let pv = pop_rate.iter().map(|x| (x - pm).powi(2)).sum::<f64>() / 20.0;
    let lag1: f64 = (1..20).map(|i| (pop_rate[i] - pm) * (pop_rate[i - 1] - pm)).sum::<f64>() / 20.0;
    println!("pop rate/s: mean={pm:.1} sd={:.1} autocorr_lag1≈{:.2}", pv.sqrt(), lag1 / pv);
}
// (appended analysis in main via a second pass is complex; instead run the tool
// with a second arg to dump u trajectories during the DRIVE period)
