//! E14 resolution instrument (docs/anima-e14-protocol.md, frozen).
//! ANALYSIS-ONLY: imports anima_telemetry + std only (no anima_core);
//! opens run dirs read-only; never constructs an Environment.
//!
//! Per-REV-presentation trajectory over the committed E12 run
//! artifacts. Reference sets: A_ref/C_ref = the 10 A / 10 C
//! presentations of S1 rounds 51-60 (E13's T0 window). For REV
//! presentation k (rounds 61-120, one B per round):
//!   ab_k = pairwise mean cos(B_k, A_i) over refs
//!   bc_k = pairwise mean cos(B_k, C_i) over refs
//!   alignment_k = argmin(ab_k, bc_k);  independence_k = frozen 0.60
//! T0 (k=0) uses B rounds 51-60 vs the same refs — bit-identical
//! anchor vs E13's published T0.
//! Supporting columns per checkpoint with E13-identical windows:
//! A-C, windowed selectivity median, permanence, failures, rates.
//!
//! Outputs k*, the first unsustained A-flip (if any), and the
//! transition bound.
//!
//! Usage: e14_resolution <run-dir>

use std::collections::BTreeMap;

const N_IN: u32 = 24;
const BASE: u64 = 5_000;

fn main() {
    let dir = std::env::args().nth(1).expect("run dir");
    let tdir = std::path::Path::new(&dir).join("telemetry");
    let reader = anima_telemetry::TelemetryReader::open(&tdir).unwrap();
    let idx = reader.chunk_index();

    let mut stims: Vec<(String, String, u64, u64)> = Vec::new();
    let mut spikes: Vec<(u64, u32)> = Vec::new();
    let mut failures: Vec<u64> = Vec::new();
    let mut permanence: Vec<u64> = Vec::new();
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                3 => spikes.push((row.t, row.n.unwrap_or(0) as u32)),
                5 => {
                    if let Ok(env) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } = env.payload {
                            stims.push((pattern_id, stage, env.t, env.t + 500));
                        }
                    }
                }
                6 => {
                    if let Ok(e) = row.envelope("v2") {
                        if let anima_telemetry::events::Payload::SynapseCreated { reason, .. } = e.payload {
                            if reason.trigger == "candidate-permanence" {
                                permanence.push(e.t);
                            }
                        }
                    }
                }
                17 => {
                    if let Ok(e) = row.envelope("v2") {
                        if let anima_telemetry::events::Payload::Failure { .. } = e.payload {
                            failures.push(e.t);
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

    // Group presentations by pattern.
    let s1: Vec<&(String, String, u64, u64)> = stims.iter().filter(|p| p.1 == "S1").collect();
    fn pat(p: &(String, String, u64, u64)) -> &String {
        &p.0
    }
    fn round_of(p: &(String, String, u64, u64)) -> u64 {
        (p.2 - BASE) / 6000 + 1
    }
    fn pos_of(p: &(String, String, u64, u64)) -> u64 {
        (p.2 - BASE) / 2000 % 3
    }

    // Reference sets: A/C in rounds 51-60.
    let mut a_ref: Vec<&(String, String, u64, u64)> = vec![];
    let mut c_ref: Vec<&(String, String, u64, u64)> = vec![];
    for p in &s1 {
        let r = round_of(p);
        if (51..=60).contains(&r) {
            if pat(p) == "A" {
                a_ref.push(p);
            } else if pat(p) == "C" {
                c_ref.push(p);
            }
        }
    }
    assert_eq!(a_ref.len(), 10, "A reference window");
    assert_eq!(c_ref.len(), 10, "C reference window");

    let vec_of = |p: &(String, String, u64, u64)| -> BTreeMap<u32, f64> {
        let mut v: BTreeMap<u32, f64> = BTreeMap::new();
        for &(t, n) in &spikes {
            if t >= p.2 && t < p.3 && n >= N_IN {
                *v.entry(n).or_insert(0.0) += 1.0;
            }
        }
        v
    };
    let mut a_ref_v: Vec<BTreeMap<u32, f64>> = a_ref.iter().map(|p| vec_of(p)).collect();
    let mut c_ref_v: Vec<BTreeMap<u32, f64>> = c_ref.iter().map(|p| vec_of(p)).collect();
    // NB: refs are used unchanged; they are the fixed anchor sets.

    // T0: B of rounds 51-60 vs refs.
    let mut b_t0: Vec<&(String, String, u64, u64)> = vec![];
    for p in &s1 {
        let r = round_of(p);
        if (51..=60).contains(&r) && pat(p) == "B" {
            b_t0.push(p);
        }
    }
    assert_eq!(b_t0.len(), 10, "T0 B window");
    let (ab0, bc0) = ab_bc(b_t0.iter().map(|p| vec_of(p)).collect(), &a_ref_v, &c_ref_v);
    println!("T0 (B rounds 51-60): A-B {ab0:.3}, B-C {bc0:.3}");

    // Per-REV trajectory: REV k = B of global round 60+k.
    let mut b_rev: Vec<(&(String, String, u64, u64), u64)> = vec![];
    for p in &s1 {
        let r = round_of(p);
        if (61..=120).contains(&r) && pat(p) == "B" {
            b_rev.push((p, r - 60));
        }
    }
    assert_eq!(b_rev.len(), 60, "REV B presentations");
    println!("round-61 B position: {}", pos_of(&b_rev[0].0));
    println!("round-120 B position: {}", pos_of(&b_rev[59].0));

    let mut rows: Vec<(u64, f64, f64, u64)> = vec![];
    for (p, k) in &b_rev {
        let v = vec_of(p);
        let (_ab, _bc) = ab_bc(vec![v.clone()], &a_ref_v, &c_ref_v);
        rows.push((*k, _ab, _bc, v.values().sum::<f64>() as u64));
    }

    // Earliest-movement point: k* = first k with alignment A sustained
    // through k..10 (suffix rule).
    let mut k_star: Option<u64> = None;
    let mut first_flip: Option<u64> = None;
    for i in 0..rows.len() {
        let (k, ab, bc, _) = rows[i];
        if ab < bc {
            if first_flip.is_none() {
                first_flip = Some(k);
            }
            let sustained = rows
                .iter()
                .skip(i)
                .take_while(|(kk, _, _, _)| *kk <= 10)
                .all(|(_, a, b, _)| a < b);
            if sustained && k_star.is_none() {
                k_star = Some(k);
            }
        }
    }
    println!("first unsustained A-flip: REV{}", first_flip.map_or("(none)".to_string(), |k| k.to_string()));
    match k_star {
        Some(k) => {
            if k == 1 {
                println!("transition bound: (T0, REV1]");
            } else {
                println!("transition bound: (REV{}, REV{}]", k - 1, k);
            }
        }
        None => println!("transition bound: none (no sustained flip by REV10)"),
    }

    // Checkpoint table: T0, REV1..REV10, REV20..60 (trailing window
    // columns: selectivity, permanence, failures, rates).
    println!("===== per-REV trajectory (ab_k, bc_k, spikes) =====");
    for (k, ab, bc, nsp) in &rows {
        if *k <= 10 || *k % 10 == 0 {
            println!("REV{k}: A-B {ab:.3}, B-C {bc:.3}, alignment {}, indep {}, spikes {nsp}",
                if *ab < *bc { "A" } else { "C" },
                if *ab < 0.60 && *bc < 0.60 { "true" } else { "false" });
        }
    }

    // Supporting columns at T10/T20/../T60 and REV1..9: trailing-window
    // selectivity/permanence/failures/rates, same windows as E13.
    let window_cols = |lo: u64, hi: u64| -> (f64, usize, usize, f32, f32) {
        let win: Vec<&(String, String, u64, u64)> = s1
            .iter()
            .filter(|p| p.2 >= lo && p.2 < hi)
            .copied()
            .collect();
        let mut per_neuron: BTreeMap<u32, Vec<f64>> = BTreeMap::new();
        let mut pats: Vec<&str> = vec![];
        for p in &win {
            if !pats.contains(&p.0.as_str()) {
                pats.push(p.0.as_str());
            }
            let v = vec_of(p);
            for (n, &c) in &v {
                let e = per_neuron.entry(*n).or_insert_with(|| vec![0.0; pats.len()]);
                let pi = pats.iter().position(|x| *x == p.0.as_str()).unwrap();
                e[pi] += c;
            }
        }
        let mut sels = Vec::new();
        for rates in per_neuron.values() {
            let mut s = rates.clone();
            s.sort_by(|a, b| b.partial_cmp(a).unwrap());
            if s[0] > 1e-9 {
                sels.push((s[0] - s.get(1).copied().unwrap_or(0.0)) / s[0]);
            }
        }
        sels.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let med = sels.get(sels.len() / 2).copied().unwrap_or(0.0);
        let perm = permanence.iter().filter(|&&t| t >= lo && t < hi).count();
        let wfail = failures.iter().filter(|&&t| t >= lo && t < hi).count();
        let rates: Vec<f32> = snaps
            .iter()
            .filter(|s| s.tick >= lo && s.tick < hi)
            .flat_map(|s| s.neurons.iter().filter(|n| n.class == "internal").filter_map(|n| n.rate_hz))
            .collect();
        let rmean = rates.iter().sum::<f32>() / rates.len().max(1) as f32;
        let rmax = rates.iter().cloned().fold(0.0f32, f32::max);
        (med, perm, wfail, rmean, rmax)
    };
    println!("===== trailing-window supporting columns =====");
    for k in [1u64, 2, 3, 4, 5, 6, 7, 8, 9, 10, 20, 30, 40, 50, 60] {
        let (lo, hi) = if k >= 10 {
            (BASE + 6000 * (50 + k), BASE + 6000 * (60 + k))
        } else {
            (BASE + 6000 * 50, BASE + 6000 * (60 + k))
        };
        let (sel, perm, wfail, rmean, rmax) = window_cols(lo, hi);
        println!("REV{k}: selectivity {sel:.3}, permanence {perm}, failures {wfail}, rates {rmean:.1}/{rmax:.1} Hz");
    }

    println!("===== A-C (trailing window, E13 method) =====");
    for k in [10u64, 20, 30, 40, 50, 60] {
        let (lo, hi) = (BASE + 6000 * (50 + k), BASE + 6000 * (60 + k));
        let win: Vec<&(String, String, u64, u64)> = s1.iter().filter(|p| p.2 >= lo && p.2 < hi).copied().collect();
        let mut ac = Vec::new();
        for (i, pa) in win.iter().filter(|p| p.0 == "A").enumerate() {
            for (j, pc) in win.iter().filter(|p| p.0 == "C").enumerate() {
                if i == j {
                    ac.push(cos(&vec_of(pa), &vec_of(pc), true));
                }
            }
        }
        let m = ac.iter().sum::<f64>() / ac.len().max(1) as f64;
        println!("T{k} A-C: {m:.3}");
    }
}

fn ab_bc(
    b_vecs: Vec<BTreeMap<u32, f64>>,
    a_ref: &[BTreeMap<u32, f64>],
    c_ref: &[BTreeMap<u32, f64>],
) -> (f64, f64) {
    let mut ab = Vec::new();
    let mut bc = Vec::new();
    for vb in &b_vecs {
        for va in a_ref {
            ab.push(cos(vb, va, true));
        }
        for vc in c_ref {
            bc.push(cos(vb, vc, true));
        }
    }
    let m = |c: &[f64]| c.iter().sum::<f64>() / c.len().max(1) as f64;
    (m(&ab), m(&bc))
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
            (
                if sx > 0.0 { x / sx } else { 0.0 },
                if sy > 0.0 { y / sy } else { 0.0 },
            )
        } else {
            (x, y)
        };
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        dot / (na.sqrt() * nb.sqrt())
    }
}