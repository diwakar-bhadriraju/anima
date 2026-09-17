//! E15 structural-basis instrument (docs/anima-e15-protocol.md, frozen).
//! ANALYSIS-ONLY: imports anima_telemetry + std only (no anima_core);
//! opens run dirs read-only; never constructs an Environment.
//!
//! Reads committed E12 run artifacts. Registered instants:
//!   T0 = snapshot 364,000 (all seeds)
//!   REV1-pre = 364,000 (20260912, REV1-B pos 0) / 368,000 (9001, 424242)
//!   REV1-post = 366,000 / 370,000
//! Primary interval (REV1-pre, REV1-post]; REV1 presentation window
//! = [S_B, S_B+500) with S_B = 365,000 (20260912) / 369,000 (9001, 424242).
//!
//! Reports per seed: live excitatory/inhibitory topology at the three
//! instants; creations/prunings/candidate-permanence transitions in
//! the interval (with ms timestamps bucketed before/during/after the
//! presentation window); endpoint weight differences; structural
//! shares; M5 budget occupancy (snapshot-derived + ResourceUsage);
//! per-neuron structural churn; A-side (0-7) / C-side (8-15) /
//! lower-B (4-7) / upper-B (8-11) / other buckets; joined behavioral
//! anchors (E14, committed — not recomputed).
//!
//! No thresholds, no causality. Event timing vs endpoint difference
//! vs unresolved intra-interval weight evolution are reported in
//! separate sections.
//!
//! Usage: e15_structure <run-dir> <seed>

use std::collections::BTreeMap;

const T0: u64 = 364_000;

fn bucket(pre: u32) -> &'static str {
    if pre < 24 {
        match pre {
            0..=3 => "A-only(0-3)",
            4..=7 => "lower-B(4-7)",
            8..=11 => "upper-B(8-11)",
            12..=15 => "C-only(12-15)",
            _ => "inactive-input(16-23)",
        }
    } else {
        "recurrent"
    }
}

#[derive(Clone)]
struct State {
    id: u32,
    pre: u32,
    post: u32,
    w: f32,
    plastic: bool,
}

fn load_synapses(snaps: &[anima_telemetry::recorder::NetworkStateSnapshot], tick: u64) -> Vec<State> {
    snaps
        .iter()
        .find(|s| s.tick == tick)
        .expect("snapshot tick exists")
        .synapses
        .iter()
        .map(|s| State {
            id: s.id,
            pre: s.pre,
            post: s.post,
            w: s.w.unwrap_or(0.0),
            plastic: s.plastic,
        })
        .collect()
}

fn main() {
    let dir = std::env::args().nth(1).expect("run dir");
    let seed = std::env::args().nth(2).expect("seed");
    let (pre_tick, post_tick, sb) = if seed == "20260912" {
        (T0, 366_000u64, 365_000u64)
    } else {
        (368_000, 370_000, 369_000)
    };
    let tdir = std::path::Path::new(&dir).join("telemetry");
    let reader = anima_telemetry::TelemetryReader::open(&tdir).unwrap();
    let idx = reader.chunk_index();

    let mut created: Vec<(u64, u32, u32, u32, f32, String)> = Vec::new();
    let mut pruned: Vec<(u64, u32, String)> = Vec::new();
    let mut ru: Vec<(u64, u64, u64)> = Vec::new(); // (t, live_exc, live_inh)
    for c in 0..idx.len() {
        for row in reader.chunk_rows(c).unwrap() {
            match row.kind {
                6 => {
                    if let Ok(e) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::SynapseCreated { syn, pre, post, w, reason } = e.payload {
                            created.push((e.t, syn.0, pre.0, post.0, w.unwrap_or(0.0), reason.trigger));
                        }
                    }
                }
                7 => {
                    if let Ok(e) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::SynapsePruned { syn, reason } = e.payload {
                            pruned.push((e.t, syn.0, reason.trigger));
                        }
                    }
                }
                16 => {
                    if let Ok(e) = row.envelope("e") {
                        if let anima_telemetry::events::Payload::ResourceUsage { live_exc, live_inh, .. } = e.payload {
                            if let (Some(x), Some(y)) = (live_exc, live_inh) {
                                ru.push((e.t, x, y));
                            }
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

    let s_t0 = load_synapses(&snaps, T0);
    let s_pre = load_synapses(&snaps, pre_tick);
    let s_post = load_synapses(&snaps, post_tick);

    let perm_ids: std::collections::BTreeSet<u32> = created
        .iter()
        .filter(|(_, _, _, _, _, r)| r == "candidate-permanence")
        .map(|(_, id, _, _, _, _)| *id)
        .collect();

    let snap_stats = |v: &[State]| -> (usize, usize, u64, u64, BTreeMap<&'static str, (usize, f64)>) {
        let exc = v.iter().filter(|s| s.plastic).count();
        let inh = v.len() - exc;
        let (mut wsum_e, mut wsum_i) = (0.0f64, 0.0f64);
        let mut by_bucket: BTreeMap<&'static str, (usize, f64)> = BTreeMap::new();
        for s in v {
            if s.plastic {
                wsum_e += s.w as f64;
                let e = by_bucket.entry(bucket(s.pre)).or_insert((0, 0.0));
                e.0 += 1;
                e.1 += s.w as f64;
            } else {
                wsum_i += s.w as f64;
            }
        }
        (exc, inh, wsum_e.to_bits() as u64, wsum_i.to_bits() as u64, by_bucket)
    };
    let report_state = |label: &str, v: &[State]| {
        let (exc, inh, we, wi, buckets) = snap_stats(v);
        print!("  {label}: alive exc={exc} inh={inh} wsum_exc_bits={we} wsum_inh_bits={wi}");
        for (b, (n, w)) in &buckets {
            print!(" | {b}: n={n} w={w:.6}");
        }
        println!();
        let alive_perm = v.iter().filter(|s| s.plastic && perm_ids.contains(&s.id)).count();
        println!("  {label}: permanence synapses alive={alive_perm}");
    };

    println!("== seed {seed}");
    println!("== instants: T0={T0} pre={pre_tick} post={post_tick} REV1-B window=[{sb}, {})", sb + 500);
    report_state("T0", &s_t0);
    report_state("REV1-pre", &s_pre);
    report_state("REV1-post", &s_post);

    // --- Event timing (exact ms timestamps) ---
    let mut cr: Vec<&(u64, u32, u32, u32, f32, String)> = created
        .iter()
        .filter(|(t, _, _, _, _, _)| *t > pre_tick && *t <= post_tick)
        .collect();
    cr.sort_by_key(|e| e.0);
    let mut pr: Vec<&(u64, u32, String)> = pruned
        .iter()
        .filter(|(t, _, _)| *t > pre_tick && *t <= post_tick)
        .collect();
    pr.sort();
    println!("== EVENT TIMING: interval ({pre_tick}, {post_tick}]");
    println!("  creations: {} ({} permanence maturations)", cr.len(), cr.iter().filter(|e| e.5 == "candidate-permanence").count());
    let order = |t: u64| if t < sb { "before" } else if t < sb + 500 { "during" } else { "after" };
    for (t, id, pre, post, w, reason) in &cr {
        println!(
            "    created syn={id} pre={pre}({}) post={post} w={w} reason={reason} t={t} [{}]",
            bucket(*pre),
            order(*t)
        );
    }
    println!("  prunings: {}", pr.len());
    for (t, id, reason) in &pr {
        println!("    pruned syn={id} reason={reason} t={t} [{}]", order(*t));
    }
    let ru_interval: Vec<&(u64, u64, u64)> = ru.iter().filter(|(t, _, _)| *t > pre_tick && *t <= post_tick).collect();
    println!(
        "  ResourceUsage events in interval: {} (live_exc/inh at last: {}/{})",
        ru_interval.len(),
        ru_interval.last().map(|r| r.1).unwrap_or(0),
        ru_interval.last().map(|r| r.2).unwrap_or(0)
    );

    // --- Endpoint differences ---
    println!("== ENDPOINT STRUCTURAL DIFFERENCE: pre -> post");
    fn by_id<'a>(v: &'a [State]) -> BTreeMap<u32, &'a State> {
        v.iter().map(|s| (s.id, s)).collect()
    }
    let p_pre = by_id(&s_pre);
    let p_post = by_id(&s_post);
    let mut dw: BTreeMap<&'static str, (usize, usize, f64, f64)> = BTreeMap::new(); // bucket: (alive, changed, sum_dw_pos, sum_dw_neg)
    let mut churn: BTreeMap<u32, usize> = BTreeMap::new(); // post neuron: |symmetric diff|
    for (id, st) in &p_pre {
        let b = bucket(st.pre);
        let e = dw.entry(b).or_insert((0, 0, 0.0, 0.0));
        e.0 += 1;
        if st.plastic {
            if let Some(p2) = p_post.get(id) {
                let d = (p2.w - st.w) as f64;
                if d != 0.0 {
                    e.1 += 1;
                    if d > 0.0 {
                        e.2 += d;
                    } else {
                        e.3 += d;
                    }
                }
            } else {
                // pruned: reported via events above
            }
        } else if let Some(p2) = p_post.get(id) {
            let d = (p2.w - st.w) as f64;
            if d != 0.0 {
                e.1 += 1;
                if d > 0.0 {
                    e.2 += d;
                } else {
                    e.3 += d;
                }
            }
        }
    }
    // new synapses in post not in pre
    for (id, st) in &p_post {
        if !p_pre.contains_key(id) {
            let e = dw.entry(bucket(st.pre)).or_insert((0, 0, 0.0, 0.0));
            e.0 += 1;
            if st.plastic {
                e.1 += 1;
                e.2 += st.w as f64;
            }
        }
    }
    let mut weird = 0;
    for (b, (alive, changed, pos, neg)) in &dw {
        println!(
            "  bucket {b}: alive-at-pre-or-post={alive} synapses with endpoint dW!=0: {changed} (pos sum {pos:.6}, neg sum {neg:.6})"
        );
        weird += 1;
    }
    let _ = weird;
    // per-neuron churn (excitatory incoming sets)
    let mut pre_in: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    let mut post_in: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for s in &s_pre {
        if s.plastic {
            pre_in.entry(s.post).or_default().push(s.id);
        }
    }
    for s in &s_post {
        if s.plastic {
            post_in.entry(s.post).or_default().push(s.id);
        }
    }
    let mut posts: Vec<u32> = pre_in.keys().chain(post_in.keys()).copied().collect();
    posts.sort();
    posts.dedup();
    let mut churn_total = 0usize;
    let mut churn_neurons = 0usize;
    for p in posts {
        if p < 24 {
            continue;
        }
        let a = pre_in.get(&p).cloned().unwrap_or_default();
        let b2 = post_in.get(&p).cloned().unwrap_or_default();
        let sa: std::collections::BTreeSet<u32> = a.iter().copied().collect();
        let sb2: std::collections::BTreeSet<u32> = b2.iter().copied().collect();
        let sym = sa.symmetric_difference(&sb2).count();
        if sym > 0 {
            churn_total += sym;
            churn_neurons += 1;
            churn.insert(p, sym);
        }
    }
    println!("== per-neuron structural change: {churn_neurons} internal neurons with excitatory afferent churn, {churn_total} churned synapses (symmetric diff pre vs post)");
    for (p, n) in &churn {
        println!("    post-neuron {p}: {n} afferents changed");
    }

    // --- Structural shares (raw fractions, no cutoffs) ---
    println!("== STRUCTURAL SHARES (raw)");
    let total_exc_pre = s_pre.iter().filter(|s| s.plastic).count().max(1);
    let total_exc_post = s_post.iter().filter(|s| s.plastic).count().max(1);
    let created_count = cr.len();
    let pruned_count = pr.len();
    println!("  created / exc-pre = {:.4}", created_count as f64 / total_exc_pre as f64);
    println!("  pruned / exc-pre = {:.4}", pruned_count as f64 / total_exc_pre as f64);
    println!("  created / exc-post = {:.4}", created_count as f64 / total_exc_post as f64);
    let all_w_comparisons: usize = dw.values().map(|(a, _, _, _)| *a).sum();
    let changed_all: usize = dw.values().map(|(_, c, _, _)| *c).sum();
    println!(
        "  endpoint-dW!=0 / alive-both = {:.4} ({} / {})",
        changed_all as f64 / all_w_comparisons.max(1) as f64,
        changed_all,
        all_w_comparisons
    );

    // --- M5 budget occupancy ---
    println!("== M5 budget occupancy (snapshot-derived)");
    let count_per_post = |v: &[State]| -> (usize, usize) {
        let mut me = 0;
        let mut mi = 0;
        let mut by: BTreeMap<u32, (usize, usize)> = BTreeMap::new();
        for s in v {
            let e = by.entry(s.post).or_insert((0, 0));
            if s.plastic {
                e.0 += 1;
                me = me.max(e.0);
            } else {
                e.1 += 1;
                mi = mi.max(e.1);
            }
        }
        (me, mi)
    };
    let (me0, mi0) = count_per_post(&s_t0);
    let (me1, mi1) = count_per_post(&s_pre);
    let (me2, mi2) = count_per_post(&s_post);
    println!(
        "  max exc per neuron: T0={me0} pre={me1} post={me2} (B_e=40); max inh per neuron: T0={mi0} pre={mi1} post={mi2} (B_i=10); totals exc/inh pre: {}/{} post: {}/{}",
        s_pre.iter().filter(|s| s.plastic).count(),
        s_pre.len() - s_pre.iter().filter(|s| s.plastic).count(),
        s_post.iter().filter(|s| s.plastic).count(),
        s_post.len() - s_post.iter().filter(|s| s.plastic).count(),
    );

    // --- Representation anchors (joined from E14, frozen, not recomputed) ---
    println!("== REPRESENTATION (joined E14 anchors, committed 410f84d; not recomputed)");
    let (ab0, bc0, ab1, bc1) = match seed.as_str() {
        "20260912" => (0.680, 0.075, 0.190, 0.929),
        "9001" => (0.760, 0.097, 0.127, 0.672),
        _ => (0.554, 0.065, 0.112, 0.746),
    };
    println!("  T0 A-B={ab0:.3} B-C={bc0:.3} (E14 anchor); REV1 A-B={ab1:.3} B-C={bc1:.3}; k*=1");

    // --- Unresolved intra-interval weight evolution ---
    println!("== UNRESOLVED INTRA-INTERVAL WEIGHT EVOLUTION (not timestamped)");
    let residual = |v: &[State], other: &BTreeMap<u32, &State>| -> f64 {
        v.iter()
            .filter(|s| s.plastic)
            .map(|s| {
                let d = other.get(&s.id).map(|o| (o.w - s.w) as f64).unwrap_or(0.0);
                d
            })
            .sum::<f64>()
    };
    println!("  residual endpoint dW sum (exc, pre->post): {:.6}", residual(&s_pre, &p_post));
    println!("  (includes M2 normalize, M6, passive decay, |dW|<=0.01 STDP; not placeable in time)");
}