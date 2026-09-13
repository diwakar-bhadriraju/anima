//! Analyzer: pure function telemetry → metrics.json.
//!
//! Computes: per-neuron per-pattern mean rates, selectivity index, assembly
//! score (within- vs cross-pattern cosine similarity, early vs late S1),
//! novelty response (profile distance of D to learned patterns + population
//! rate vs baseline), retention (S3 vs late-S1), structure timeline,
//! resource timeline.

use std::collections::{BTreeMap, HashMap};

use serde::Serialize;

use anima_core::network::SimMs;

use crate::events::{Envelope, Payload};

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub struct Metrics {
    /// pattern → neuron id → mean rate (Hz) during that pattern's
    /// presentations (per stage for A/B/C/D).
    pub per_pattern_rates: BTreeMap<String, BTreeMap<String, StageRates>>,
    /// neuron → selectivity index per stage window: (r_best − r_2nd)/r_best.
    pub selectivity: BTreeMap<String, Option<f32>>,
    /// assembly score: mean within-pattern cosine similarity minus
    /// cross-pattern, early-S1 vs late-S1 windows.
    pub assembly_score_early_s1: Option<f32>,
    pub assembly_score_late_s1: Option<f32>,
    /// Novelty: distance of D's response profile to nearest learned pattern,
    /// and the learned patterns' pairwise max distance for comparison.
    pub novelty: Option<NoveltyMetrics>,
    /// Retention: S3 vs late-S1 mean response ratio per pattern.
    pub retention: BTreeMap<String, Option<f32>>,
    /// Structural timeline.
    pub structure: StructureTimeline,
    /// Resource timeline (sampled).
    pub resources: ResourceTimeline,
    /// Failures observed.
    pub failures: Vec<FailureRecord>,
    /// Population rates per stage (all-internal mean, Hz).
    pub stage_population_rates: BTreeMap<String, Option<f32>>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct StageRates {
    pub mean_hz: Option<f32>,
    pub presentations: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct NoveltyMetrics {
    pub d_distance_to_nearest: Option<f32>,
    pub learned_pairwise_max: Option<f32>,
    pub d_population_rate_hz: Option<f32>,
    pub learned_baseline_rate_hz: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct StructureTimeline {
    pub births: Vec<(SimMs, u32)>,
    pub prunes: Vec<(SimMs, u32)>,
    pub dormancies: Vec<(SimMs, u32)>,
    pub reactivations: Vec<(SimMs, u32)>,
    pub retirements: Vec<(SimMs, u32)>,
    pub synapse_created: u64,
    pub synapse_pruned: u64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ResourceTimeline {
    pub samples: Vec<ResourceSamplePoint>,
    pub total_metabolic_cost: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceSamplePoint {
    pub t: SimMs,
    pub neurons: u64,
    pub synapses: u64,
    pub spikes_window: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FailureRecord {
    pub t: SimMs,
    pub kind: String,
    pub detail: String,
}

/// A stimulus presentation window: [start, end) in sim-ms with pattern id.
#[derive(Debug, Clone)]
struct Presentation {
    pattern: String,
    stage: String,
    start: SimMs,
    /// Onset + 500 ms response window (pattern duration).
    end: SimMs,
}

const RESPONSE_WINDOW_MS: SimMs = 500;

/// Pure telemetry → metrics.
pub fn analyze(events: &[Envelope]) -> Metrics {
    let mut presentations: Vec<Presentation> = Vec::new();
    let mut spikes: Vec<(SimMs, u32)> = Vec::new();
    let mut metrics = Metrics::default();

    for env in events {
        match &env.payload {
            Payload::StimulusPresented { pattern_id, stage } => {
                presentations.push(Presentation {
                    pattern: pattern_id.clone(),
                    stage: stage.clone(),
                    start: env.t,
                    end: env.t + RESPONSE_WINDOW_MS,
                });
            }
            Payload::Spike { n } | Payload::OutputActivity { n } => {
                spikes.push((env.t, n.0));
            }
            Payload::SynapseCreated { .. } => metrics.structure.synapse_created += 1,
            Payload::SynapsePruned { syn, .. } => {
                metrics.structure.prunes.push((env.t, syn.0));
            }
            Payload::NeuronCreated { n, .. } => metrics.structure.births.push((env.t, n.0)),
            Payload::NeuronDormant { n, .. } => metrics.structure.dormancies.push((env.t, n.0)),
            Payload::NeuronReactivated { n, .. } => {
                metrics.structure.reactivations.push((env.t, n.0))
            }
            Payload::NeuronRetired { n, .. } => metrics.structure.retirements.push((env.t, n.0)),
            Payload::ResourceUsage { neurons, synapses, spikes_window, metabolic_cost, .. } => {
                metrics.resources.samples.push(ResourceSamplePoint {
                    t: env.t,
                    neurons: *neurons,
                    synapses: *synapses,
                    spikes_window: *spikes_window,
                });
                if let Some(c) = metabolic_cost {
                    *metrics.resources.total_metabolic_cost.get_or_insert(0.0) += c;
                }
            }
            Payload::Failure { kind, detail } => {
                metrics.failures.push(FailureRecord { t: env.t, kind: kind.clone(), detail: detail.clone() });
            }
            _ => {}
        }
    }
    metrics.structure.synapse_pruned = metrics.structure.prunes.len() as u64;

    // Per-pattern per-stage mean rates (Hz) over the response window:
    // rate = spikes_in_window / (window_ms/1000) / presentations, per neuron.
    // Only internal-class counting is approximated by using all spike events
    // (input neurons' spikes are frame-driven; analyzer filters by known
    // input count via RunStarted params when available).
    let input_n: u32 = events
        .iter()
        .find_map(|e| match &e.payload {
            Payload::RunStarted { params, .. } => params.get("n_input_channels").and_then(|v| v.as_u64()).map(|v| v as u32),
            _ => None,
        })
        .unwrap_or(24);

    // per (stage, pattern) → neuron → total spikes
    let mut acc: BTreeMap<(String, String), BTreeMap<u32, u64>> = BTreeMap::new();
    let mut pres_count: BTreeMap<(String, String), u64> = BTreeMap::new();
    // Response vector per presentation: neuron → spike count (for cosine).
    let mut pres_vecs: Vec<(String, String, u64, BTreeMap<u32, u64>)> = Vec::new();
    for pres in &presentations {
        if pres.pattern == "silence" {
            continue;
        }
        let key = (pres.stage.clone(), pres.pattern.clone());
        *pres_count.entry(key.clone()).or_insert(0) += 1;
        let mut vec = BTreeMap::new();
        for &(t, n) in &spikes {
            if t >= pres.start && t < pres.end && n >= input_n {
                *acc.entry(key.clone()).or_default().entry(n).or_insert(0) += 1;
                *vec.entry(n).or_insert(0) += 1;
            }
        }
        pres_vecs.push((pres.stage.clone(), pres.pattern.clone(), pres.start, vec));
    }

    // Metrics per pattern per stage.
    for ((stage, pattern), neurons) in &acc {
        let n_pres = pres_count.get(&(stage.clone(), pattern.clone())).copied().unwrap_or(1).max(1);
        let stage_rates = metrics
            .per_pattern_rates
            .entry(pattern.clone())
            .or_default()
            .entry(stage.clone())
            .or_default();
        stage_rates.presentations = n_pres;
        stage_rates.mean_hz = Some(
            neurons.values().sum::<u64>() as f32 / (RESPONSE_WINDOW_MS as f32 / 1000.0) / n_pres as f32 / (neurons.len().max(1)) as f32,
        );
    }

    // Selectivity per stage over learned patterns (A/B/C): per neuron
    // (r_best − r_2nd)/r_best using per-pattern mean rates.
    let learned = ["A", "B", "C"];
    for stage in unique_stages(&presentations) {
        // neuron → pattern → rate
        let mut per_neuron: BTreeMap<u32, Vec<f32>> = BTreeMap::new();
        for pat in &learned {
            for pres in &presentations {
                if pres.pattern != *pat || pres.stage != stage || pres.pattern == "silence" {
                    continue;
                }
                for &(t, n) in &spikes {
                    if t >= pres.start && t < pres.end && n >= input_n {
                        let rates = per_neuron.entry(n).or_insert_with(|| vec![0.0; learned.len()]);
                        let idx = learned.iter().position(|p| p == pat).unwrap();
                        rates[idx] += 1.0;
                    }
                }
            }
        }
        // Normalize per presentation count.
        let counts: HashMap<String, u64> = pres_count
            .iter()
            .filter(|((s, _), _)| s == &stage)
            .map(|((_, p), c)| (p.clone(), *c))
            .collect();
        let mut sel_vals: Vec<f32> = Vec::new();
        let mut all_sel: BTreeMap<String, Option<f32>> = BTreeMap::new();
        for (n, mut rates) in per_neuron {
            for (i, pat) in learned.iter().enumerate() {
                let c = counts.get(*pat).copied().unwrap_or(1).max(1) as f32;
                rates[i] = rates[i] / c / (RESPONSE_WINDOW_MS as f32 / 1000.0);
            }
            let mut sorted = rates.clone();
            sorted.sort_by(|a, b| b.partial_cmp(a).unwrap());
            let best = sorted[0];
            let second = sorted.get(1).copied().unwrap_or(0.0);
            let sel = if best > 1e-9 { Some((best - second) / best) } else { None };
            all_sel.insert(n.to_string(), sel);
            if let Some(s) = sel {
                sel_vals.push(s);
            }
        }
        // Store per-neuron selectivity under "<stage>" prefix; median used by report.
        metrics.selectivity.extend(all_sel.into_iter().map(|(k, v)| (format!("{stage}:{k}"), v)));
        let _ = sel_vals;
        // Population rate per stage.
        let total_presentation_ms: f32 = presentations
            .iter()
            .filter(|p| p.stage == stage && p.pattern != "silence")
            .count() as f32
            * RESPONSE_WINDOW_MS as f32;
        let internal_spikes: u64 = presentations
            .iter()
            .filter(|p| p.stage == stage && p.pattern != "silence")
            .map(|p| spikes.iter().filter(|&&(t, n)| t >= p.start && t < p.end && n >= input_n).count() as u64)
            .sum();
        let n_neurons = spikes.iter().filter(|&&(_, n)| n >= input_n).map(|(_, n)| n).max().map(|m| m - input_n + 1).unwrap_or(1) as f32;
        metrics.stage_population_rates.insert(
            stage.clone(),
            (total_presentation_ms > 0.0)
                .then(|| Some(internal_spikes as f32 / (total_presentation_ms / 1000.0) / n_neurons))
                .flatten(),
        );
    }

    // Assembly score: mean within-pattern cosine similarity of neuron
    // response vectors minus mean cross-pattern similarity, for early-S1 and
    // late-S1 halves.
    if presentations.iter().any(|p| p.stage == "S1") {
        let s1_pres: Vec<&Presentation> =
            presentations.iter().filter(|p| p.stage == "S1" && p.pattern != "silence").collect();
        if s1_pres.len() >= 4 {
            let mid = s1_pres.len() / 2;
            let early_of =
                |t: &u64, pres: &Vec<&Presentation>| pres_index_lt(pres, *t) < mid;
            type Pv = (String, String, u64, BTreeMap<u32, u64>);
            let early: Vec<Pv> = pres_vecs
                .iter()
                .filter(|(st, _, t, _)| st == "S1" && early_of(t, &s1_pres))
                .cloned()
                .collect();
            let late: Vec<Pv> = pres_vecs
                .iter()
                .filter(|(st, _, t, _)| st == "S1" && !early_of(t, &s1_pres))
                .cloned()
                .collect();
            metrics.assembly_score_early_s1 = assembly_delta(&early);
            metrics.assembly_score_late_s1 = assembly_delta(&late);
        }
    }

    // Novelty: D (S2) response-profile distance to nearest learned pattern
    // vs max A↔B↔C pairwise distance — profiles from S3 re-test (current,
    // post-D) so learned profiles are comparable post-D. Fallback S1-late.
    let profile_of = |stage: &str, pattern: &str| -> Option<BTreeMap<u32, f32>> {
        let mut m = BTreeMap::new();
        let mut count = 0u64;
        for (st, pat, _, vec) in &pres_vecs {
            if st == stage && pat == pattern {
                count += 1;
                for (n, c) in vec {
                    *m.entry(*n).or_insert(0.0) += *c as f32;
                }
            }
        }
        if count == 0 {
            return None;
        }
        for v in m.values_mut() {
            *v /= count as f32;
        }
        Some(m)
    };
    let d_prof = profile_of("S2", "D");
    let a_prof = profile_of("S3", "A").or_else(|| profile_of("S1", "A"));
    let b_prof = profile_of("S3", "B").or_else(|| profile_of("S1", "B"));
    let c_prof = profile_of("S3", "C").or_else(|| profile_of("S1", "C"));
    if let Some(d) = &d_prof {
        let learned_profs: Vec<&BTreeMap<u32, f32>> = a_prof
            .as_ref()
            .into_iter()
            .chain(b_prof.as_ref().into_iter())
            .chain(c_prof.as_ref().into_iter())
            .collect();
        if !learned_profs.is_empty() {
            let d_dists: Vec<Option<f32>> =
                learned_profs.iter().map(|p| profile_distance(d, *p)).collect();
            let nearest = d_dists
                .into_iter()
                .flatten()
                .fold(f32::INFINITY, f32::min);
            let mut pair_max = 0.0f32;
            let pairs = [(a_prof.as_ref(), b_prof.as_ref()), (a_prof.as_ref(), c_prof.as_ref()), (b_prof.as_ref(), c_prof.as_ref())];
            for (x, y) in pairs {
                if let (Some(x), Some(y)) = (x, y) {
                    pair_max = pair_max.max(profile_distance(x, y).unwrap_or(0.0));
                }
            }
            // D population rate vs learned baseline.
            let d_rate = metrics
                .per_pattern_rates
                .get("D")
                .and_then(|s| s.get("S2"))
                .and_then(|r| r.mean_hz);
            let learned_rates: Vec<f32> = ["A", "B", "C"]
                .iter()
                .filter_map(|p| {
                    metrics.per_pattern_rates.get(*p).and_then(|s| s.get("S2").or_else(|| s.values().next())).and_then(|r| r.mean_hz)
                })
                .collect();
            let baseline: Option<f32> = (!learned_rates.is_empty())
                .then(|| learned_rates.iter().sum::<f32>() / learned_rates.len() as f32);
            metrics.novelty = Some(NoveltyMetrics {
                d_distance_to_nearest: if nearest.is_finite() { Some(nearest) } else { None },
                learned_pairwise_max: Some(pair_max),
                d_population_rate_hz: d_rate,
                learned_baseline_rate_hz: baseline,
            });
        }
    }

    // Retention: S3 vs late-S1 per pattern (population mean rate ratio).
    for pat in ["A", "B", "C"] {
        let late_s1 = late_s1_rate(&metrics, pat, &presentations, &spikes, input_n);
        let s3 = metrics.per_pattern_rates.get(pat).and_then(|s| s.get("S3")).and_then(|r| r.mean_hz);
        let ratio = match (s3, late_s1) {
            (Some(a), Some(b)) if b > 1e-9 => Some(a / b),
            _ => None,
        };
        metrics.retention.insert(pat.to_string(), ratio);
    }

    metrics
}

fn late_s1_rate(
    _metrics: &Metrics,
    pat: &str,
    presentations: &[Presentation],
    spikes: &[(SimMs, u32)],
    input_n: u32,
) -> Option<f32> {
    let s1: Vec<&Presentation> =
        presentations.iter().filter(|p| p.stage == "S1" && p.pattern == pat).collect();
    if s1.is_empty() {
        return None;
    }
    let mid = s1.len() / 2;
    let late = &s1[mid..];
    if late.is_empty() {
        return None;
    }
    // Same basis as per_pattern_rates mean_hz: per active neuron (neuron
    // spiking at least once in these windows), per presentation.
    let mut per_neuron: BTreeMap<u32, u64> = BTreeMap::new();
    for p in late {
        for &(t, n) in spikes {
            if t >= p.start && t < p.end && n >= input_n {
                *per_neuron.entry(n).or_insert(0) += 1;
            }
        }
    }
    if per_neuron.is_empty() {
        return Some(0.0);
    }
    let total: u64 = per_neuron.values().sum();
    Some(total as f32 / (RESPONSE_WINDOW_MS as f32 / 1000.0) / late.len() as f32 / per_neuron.len() as f32)
}
fn unique_stages(presentations: &[Presentation]) -> Vec<String> {
    let mut seen = Vec::new();
    for p in presentations {
        if !seen.contains(&p.stage) {
            seen.push(p.stage.clone());
        }
    }
    seen
}

/// Index of the first presentation starting at/after t (for early/late split).
fn pres_index_lt(sorted: &[&Presentation], t: SimMs) -> usize {
    sorted.iter().position(|p| p.start >= t).unwrap_or(sorted.len())
}

/// Mean within-pattern cosine similarity − mean cross-pattern similarity.
fn assembly_delta(vecs: &[(String, String, u64, BTreeMap<u32, u64>)]) -> Option<f32> {
    let mut within = Vec::new();
    let mut across = Vec::new();
    for i in 0..vecs.len() {
        for j in (i + 1)..vecs.len() {
            let c = cosine(&vecs[i].3, &vecs[j].3);
            match c {
                Some(c) if c.is_finite() => {
                    if vecs[i].1 == vecs[j].1 {
                        within.push(c);
                    } else {
                        across.push(c);
                    }
                }
                _ => {}
            }
        }
    }
    let mean = |v: &[f32]| (!v.is_empty()).then(|| v.iter().sum::<f32>() / v.len() as f32);
    Some(mean(&within)? - mean(&across)?)
}

fn cosine(a: &BTreeMap<u32, u64>, b: &BTreeMap<u32, u64>) -> Option<f32> {
    let dot: f32 = a.iter().filter_map(|(k, va)| b.get(k).map(|vb| (*va as f32) * (*vb as f32))).sum();
    let na: f32 = a.values().map(|v| (*v as f32).powi(2)).sum();
    let nb: f32 = b.values().map(|v| (*v as f32).powi(2)).sum();
    if na <= 0.0 || nb <= 0.0 {
        return None;
    }
    Some(dot / (na.sqrt() * nb.sqrt()))
}

/// Euclidean distance between normalized response profiles.
fn profile_distance(a: &BTreeMap<u32, f32>, b: &BTreeMap<u32, f32>) -> Option<f32> {
    let mut keys: Vec<&u32> = a.keys().chain(b.keys()).collect();
    keys.sort();
    keys.dedup();
    let sum: f32 = keys
        .iter()
        .map(|k| {
            let x = a.get(*k).copied().unwrap_or(0.0);
            let y = b.get(*k).copied().unwrap_or(0.0);
            (x - y).powi(2)
        })
        .sum();
    Some(sum.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventBuilder, Payload};
    use anima_core::network::NeuronId;

    fn env(seq: u64, t: SimMs, payload: Payload) -> Envelope {
        let mut b = EventBuilder::new("e1");
        b.seq = seq;
        b.build(t, payload)
    }

    #[test]
    fn counts_presentation_rates() {
        let mut evs = vec![env(0, 0, Payload::RunStarted {
            config_hash: "h".into(),
            seed: 1,
            params: serde_json::json!({"n_input_channels": 2}),
        })];
        // Pattern A in S1: presentation at t=1000; internal neuron 5 spikes
        // 10 times in the response window.
        evs.push(env(1, 1000, Payload::StimulusPresented { pattern_id: "A".into(), stage: "S1".into() }));
        for i in 0..10 {
            evs.push(env(2 + i, 1010 + i * 10, Payload::Spike { n: NeuronId(5) }));
        }
        let m = analyze(&evs);
        let a = m.per_pattern_rates.get("A").unwrap().get("S1").unwrap();
        assert_eq!(a.presentations, 1);
        // mean_hz = 10 spikes / 0.5 s / 1 pres / 1 neuron = 20 Hz
        let got = a.mean_hz.unwrap();
        assert!((got - 20.0).abs() < 1e-3, "got {got}");
    }

    #[test]
    fn retention_ratio_computed() {
        let mut evs = vec![env(0, 0, Payload::RunStarted {
            config_hash: "h".into(),
            seed: 1,
            params: serde_json::json!({"n_input_channels": 2}),
        })];
        // S1: 2×A with 10 spikes each; S3: 1×A with 8 spikes.
        for k in 0..2 {
            let base = 1000 + k * 2000;
            evs.push(env(1 + k * 20, base, Payload::StimulusPresented { pattern_id: "A".into(), stage: "S1".into() }));
            for i in 0..10 {
                evs.push(env(2 + k * 20 + i, base + i * 20, Payload::Spike { n: NeuronId(5) }));
            }
        }
        evs.push(env(60, 8000, Payload::StimulusPresented { pattern_id: "A".into(), stage: "S3".into() }));
        for i in 0..8 {
            evs.push(env(61 + i, 8010 + i * 20, Payload::Spike { n: NeuronId(5) }));
        }
        let m = analyze(&evs);
        let r = m.retention.get("A").unwrap().unwrap();
        assert!((r - 0.8).abs() < 0.05, "retention {r}");
    }

    #[test]
    fn failures_collected() {
        let evs = vec![env(0, 42, Payload::Failure {
            kind: "resource-exhaustion".into(),
            detail: "neurons 46 > cap 45".into(),
        })];
        let m = analyze(&evs);
        assert_eq!(m.failures.len(), 1);
        assert_eq!(m.failures[0].kind, "resource-exhaustion");
    }
}
