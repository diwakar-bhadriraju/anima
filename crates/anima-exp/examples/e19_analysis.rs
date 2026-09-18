//! E19 analysis instrument (docs/anima-e19-protocol.md, frozen).
//! ANALYSIS-ONLY: imports anima_telemetry + std (no anima_core).
//!
//! Reads the run dir + e19-world.log; reports per window
//! (E=1-40, M=81-120, L=161-200):
//!   BD = |P(vote=g1|A) - P(vote=g1|C)| (no-action excluded, reported)
//!   BR (benign rate), no-action rate, output spike rates
//!   internal divergence D (E18 metric) on probe epochs
//!   A-C sanity, permanence, failures, snapshot max rate
//! Shortcut detections: fixed-action (BD~0 + one-sided votes),
//! quiescence (no-action > 0.5 in L), schedule audit (E18 rule).
//!
//! Usage: e19_analysis <run-dir>

use std::collections::BTreeMap;

const N_IN: u32 = 24;
const OFF: u64 = 1; // delivery-tick convention

fn main() {
    let dir = std::env::args().nth(1).expect("run dir");
    let tdir = std::path::Path::new(&dir).join("telemetry");
    let reader = anima_telemetry::TelemetryReader::open(&tdir).unwrap();
    let idx = reader.chunk_index();

    let mut stims: Vec<(String, u64)> = Vec::new();
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    let mut out_spikes: Vec<(u64, u32)> = Vec::new();
    let mut permanence: u64 = 0;
    let mut failures: u64 = 0;
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                4 => out_spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                5 => {
                    if let Ok(e) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, .. } = e.payload {
                            stims.push((pattern_id, e.t));
                        }
                    }
                }
                6 => {
                    if let Ok(e) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::SynapseCreated { reason, .. } = e.payload {
                            if reason.trigger == "candidate-permanence" {
                                permanence += 1;
                            }
                        }
                    }
                }
                17 => {
                    if let Ok(e) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::Failure { .. } = e.payload {
                            failures += 1;
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let snaps = anima_telemetry::recorder::read_snapshots(
        std::path::Path::new(&dir).join("snapshots.bin.zst").as_path(),
    )
    .expect("snapshots");
    let max_rate = snaps
        .iter()
        .flat_map(|s| s.neurons.iter().filter(|n| n.class == "internal").filter_map(|n| n.rate_hz))
        .fold(0.0f32, f32::max);

    // World log: per-trial votes.
    let world_log = std::fs::read_to_string(std::path::Path::new(&dir).join("e19-world.log"))
        .expect("e19-world.log");
    let mut votes: Vec<(bool, String)> = Vec::new(); // (is_A, vote-line)
    for line in world_log.lines() {
        if line.starts_with("trial=") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let is_a = parts[1].split('=').nth(1) == Some("A");
            let vote = if line.contains("vote") {
                line.split("vote").nth(1).unwrap_or("").trim().to_string()
            } else {
                String::new() // trial whose action window never closed
            };
            votes.push((is_a, vote));
        }
    }
    assert_eq!(votes.len(), 200, "200 trials in world log");
    let voted_g1 = |v: &str| v.ends_with("g1>") || v.contains("g1=") && v.contains("-> ");
    // parse "vote g1=N g2=M -> Decision"
    let parse = |line_v: &str| -> (u32, u32, String) {
        let l = line_v.trim().trim_start_matches("vote").trim();
        let g1: u32 = l.split_whitespace().next().and_then(|t| t.split('=').nth(1)).and_then(|v| v.parse().ok()).unwrap_or(0);
        let g2: u32 = l.split_whitespace().nth(1).and_then(|t| t.split('=').nth(1)).and_then(|v| v.parse().ok()).unwrap_or(0);
        let d = l.split("->").nth(1).unwrap_or("NoAction").trim().to_string();
        if d.is_empty() { (g1, g2, "NoVote".to_string()) } else { (g1, g2, d) }
    };

    // Antecedent identity per trial from the world log (analysis-side).
    let is_a: Vec<bool> = votes.iter().map(|(a, _)| *a).collect();
    // Schedule audit
    let seq: Vec<u32> = is_a.iter().map(|&a| if a { 0 } else { 1 }).collect();
    let lag1 = |v: &[u32]| -> f64 {
        let n = v.len() as f64;
        let mean = v.iter().sum::<u32>() as f64 / n;
        let num: f64 = (1..v.len()).map(|i| (v[i] as f64 - mean) * (v[i - 1] as f64 - mean)).sum();
        let den: f64 = v.iter().map(|&x| (x as f64 - mean).powi(2)).sum();
        if den == 0.0 { 0.0 } else { num / den }
    };
    println!("schedule audit: lag-1 autocorr = {:.3}", lag1(&seq));
    for w in 0..5u64 {
        let v = &seq[(w * 40) as usize..((w + 1) * 40) as usize];
        let a = v.iter().filter(|&&x| x == 0).count();
        println!("  balance window {}: A={} C={}", w + 1, a, 40 - a);
    }

    let vec_of = |lo: u64, hi: u64| -> BTreeMap<u32, f64> {
        let mut v: BTreeMap<u32, f64> = BTreeMap::new();
        for &(t, n) in &spikes {
            if t >= lo && t < hi && n >= N_IN && n < 64 {
                *v.entry(n).or_insert(0.0) += 1.0;
            }
        }
        v
    };
    let base_of = |k: u64| 5000 + 3000 * k;

    for (label, lo, hi) in [("E", 0u64, 40u64), ("M", 80, 120), ("L", 160, 200)] {
        // Behavioral
        let mut a_g1 = 0u32; let mut a_n = 0u32; let mut c_g1 = 0u32; let mut c_n = 0u32;
        let mut no_action = 0u32; let mut benign = 0u32;
        let mut g1_spikes = 0u64; let mut g2_spikes = 0u64;
        for k in lo..hi {
            let line = world_log.lines().find(|l| l.starts_with(&format!("trial={} ", k + 1))).unwrap_or("");
            let (g1, g2, d) = parse(line);
            if d == "NoAction" || d == "NoVote" {
                no_action += 1;
            } else if is_a[k as usize] {
                a_n += 1;
                a_g1 += (g1 > g2) as u32;
            } else {
                c_n += 1;
                c_g1 += (g1 > g2) as u32;
            }
            if d == "Match" {
                benign += 1;
            }
            let _ = (g1, g2);
            // output spikes in this trial's action window
            let b = base_of(k) + OFF;
            g1_spikes += out_spikes.iter().filter(|(t, n)| *t >= b + 1800 && *t < b + 2300 && (64..70).contains(n)).count() as u64;
            g2_spikes += out_spikes.iter().filter(|(t, n)| *t >= b + 1800 && *t < b + 2300 && (70..76).contains(n)).count() as u64;
        }
        let p_a = if a_n > 0 { a_g1 as f64 / a_n as f64 } else { 0.0 };
        let p_c = if c_n > 0 { c_g1 as f64 / c_n as f64 } else { 0.0 };
        let bd = (p_a - p_c).abs();
        let trials = hi - lo;
        println!(
            "[{label}] BD={bd:.3} (P(g1|A)={p_a:.3} n={a_n}, P(g1|C)={p_c:.3} n={c_n}) BR={:.3} no-action={}/{} out-spikes g1={g1_spikes} g2={g2_spikes}",
            benign as f64 / trials as f64, no_action, trials
        );
        // Internal divergence on probe epochs
        let b_a: Vec<BTreeMap<u32, f64>> = (lo..hi).filter(|&k| is_a[k as usize])
            .map(|k| vec_of(base_of(k) + OFF + 1300, base_of(k) + OFF + 1800)).collect();
        let b_c: Vec<BTreeMap<u32, f64>> = (lo..hi).filter(|&k| !is_a[k as usize])
            .map(|k| vec_of(base_of(k) + OFF + 1300, base_of(k) + OFF + 1800)).collect();
        let (x, _xl1) = pair_mean(&b_a, &b_c);
        println!("[{label}] internal D = {:.4} (cos {:.4}; nA={} nC={})", 1.0 - x, x, b_a.len(), b_c.len());
    }

    // A-C sanity over the last 40 antecedent epochs (L window).
    let a_vecs: Vec<BTreeMap<u32, f64>> = (160..200u64).filter(|&k| is_a[k as usize])
        .map(|k| vec_of(base_of(k) + OFF, base_of(k) + OFF + 500)).collect();
    let c_vecs: Vec<BTreeMap<u32, f64>> = (160..200u64).filter(|&k| !is_a[k as usize])
        .map(|k| vec_of(base_of(k) + OFF, base_of(k) + OFF + 500)).collect();
    let (ac, _) = pair_mean(&a_vecs, &c_vecs);
    println!("A-C sanity (L): mean cos = {:.4} (criterion < 0.60)", ac);
    println!("permanence events: {permanence}; failures: {failures}; snapshot max internal rate: {max_rate:.1} Hz");
    let _ = voted_g1;
}

fn pair_mean(x: &[BTreeMap<u32, f64>], y: &[BTreeMap<u32, f64>]) -> (f64, f64) {
    let mut raw = Vec::new();
    let mut l1 = Vec::new();
    for a in x {
        for b in y {
            raw.push(cos(a, b, false));
            l1.push(cos(a, b, true));
        }
    }
    let m = |c: &[f64]| c.iter().sum::<f64>() / c.len().max(1) as f64;
    (m(&raw), m(&l1))
}

fn cos(a: &BTreeMap<u32, f64>, b: &BTreeMap<u32, f64>, norm_l1: bool) -> f64 {
    let keys: std::collections::BTreeSet<u32> = a.keys().chain(b.keys()).copied().collect();
    let mut dot = 0.0f64;
    let mut na = 0.0f64;
    let mut nb = 0.0f64;
    for k in keys {
        let x = a.get(&k).copied().unwrap_or(0.0);
        let y = b.get(&k).copied().unwrap_or(0.0);
        let (x, y) = if norm_l1 {
            let sx = a.values().sum::<f64>();
            let sy = b.values().sum::<f64>();
            (if sx > 0.0 { x / sx } else { 0.0 }, if sy > 0.0 { y / sy } else { 0.0 })
        } else {
            (x, y)
        };
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na == 0.0 || nb == 0.0 { 0.0 } else { dot / (na.sqrt() * nb.sqrt()) }
}