//! Report generator: telemetry → report.md with §21 sections.
//! Interpretation verdicts are rule-based from pre-registered thresholds,
//! each labeled `auto-generated — review`.

use serde::Serialize;

use crate::analyzer::{analyze, Metrics};

#[derive(Debug, Clone, Serialize)]
pub struct Verdict {
    pub claim: String,
    pub verdict: String,
    pub detail: String,
}

/// Pre-registered E1 thresholds (docs/e1-protocol.md) — the ONLY thresholds
/// the auto-report uses.
pub mod thresholds {
    pub const ASSEMBLY_RATIO: f32 = 2.0;
    pub const SELECTIVITY_MEDIAN: f32 = 0.5;
    pub const NOVELTY_RATIO: f32 = 2.0;
    pub const RETENTION_SUPPORTED: f32 = 0.8;
    pub const RETENTION_WEAK: f32 = 0.5;
}

pub fn verdicts(m: &Metrics) -> Vec<Verdict> {
    let mut out = Vec::new();

    // Assembly formation.
    let early = m.assembly_score_early_s1;
    let late = m.assembly_score_late_s1;
    let sel: Vec<f32> = m
        .selectivity
        .values()
        .filter_map(|v| v.as_ref().copied().filter(|x| x.is_finite()))
        .collect();
    let median_sel = median(&sel);
    let assembly_ok = match (early, late) {
        (Some(e), Some(l)) if e.abs() > 1e-9 => Some(l >= thresholds::ASSEMBLY_RATIO * e),
        (Some(_), Some(_)) => None, // early ≈ 0: any positive late score ⇒ unbounded ratio
        _ => None,
    };
    let sel_ok = median_sel.map(|s| s > thresholds::SELECTIVITY_MEDIAN);
    let verdict = match (assembly_ok, sel_ok) {
        (Some(true), Some(true)) => "supported",
        (Some(true), None) | (None, Some(true)) => "weakly supported",
        (Some(false), _) | (_, Some(false)) => "inconclusive",
        (None, None) => "inconclusive",
    };
    out.push(Verdict {
        claim: "assembly formation".into(),
        verdict: verdict.into(),
        detail: format!(
            "assembly score early-S1={:?} late-S1={:?} (threshold: late ≥ {}× early); median selectivity={:?} (threshold > {})",
            early, late, thresholds::ASSEMBLY_RATIO, median_sel, thresholds::SELECTIVITY_MEDIAN
        ),
    });

    // Novelty discrimination.
    if let Some(n) = &m.novelty {
        let (d, p) = match (n.d_distance_to_nearest, n.learned_pairwise_max) {
            (Some(d), Some(p)) => (d, p),
            _ => (f32::NAN, f32::NAN),
        };
        let verdict = if p.abs() > 1e-9 && d >= thresholds::NOVELTY_RATIO * p {
            "supported"
        } else {
            "inconclusive"
        };
        out.push(Verdict {
            claim: "novelty discrimination".into(),
            verdict: verdict.into(),
            detail: format!(
                "D-to-nearest-learned distance={d:?} vs max learned pairwise={p:?} (threshold: D ≥ {}× pairwise max)",
                thresholds::NOVELTY_RATIO
            ),
        });
    }

    // Retention per pattern.
    for pat in ["A", "B", "C"] {
        if let Some(r) = m.retention.get(pat).and_then(|v| *v) {
            let v = if r >= thresholds::RETENTION_SUPPORTED {
                "supported"
            } else if r >= thresholds::RETENTION_WEAK {
                "weakly supported"
            } else {
                "inconsistent"
            };
            out.push(Verdict {
                claim: format!("retention ({pat})"),
                verdict: v.into(),
                detail: format!(
                    "S3 response = {:.1}% of late-S1 (thresholds: ≥{:.0}% supported, ≥{:.0}% weakly)",
                    r * 100.0,
                    thresholds::RETENTION_SUPPORTED * 100.0,
                    thresholds::RETENTION_WEAK * 100.0
                ),
            });
        }
    }

    out
}

fn median(v: &[f32]) -> Option<f32> {
    if v.is_empty() {
        return None;
    }
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Some(s[s.len() / 2])
}

/// Generate report.md content from telemetry events.
pub fn generate_report(
    exp_id: &str,
    events: &[crate::events::Envelope],
    protocol_summary: &str,
    config_summary: &str,
    follow_up: &str,
) -> String {
    let m = analyze(events);
    let dur_s = events.last().map(|e| e.t).unwrap_or(0) as f64 / 1000.0;
    let vs = verdicts(&m);

    let mut r = String::new();
    r.push_str(&format!("# Experiment {exp_id} — auto-generated report\n\n"));
    r.push_str("> **auto-generated — review.** Verdicts are rule outputs from\n");
    r.push_str("> pre-registered thresholds; humans own final interpretation.\n\n");

    r.push_str("## Hypothesis\n\n");
    r.push_str(protocol_summary);
    r.push_str("\n\n## Setup\n\n");
    r.push_str(&format!("- Duration: {dur_s:.1} s sim-time\n"));
    r.push_str(&format!("- Events recorded: {}\n", events.len()));
    r.push_str(config_summary);
    r.push_str("\n\n## Organism config\n\nSee RunStarted event in telemetry.jsonl (config hash + seed recorded).\n");

    r.push_str("\n## Results\n\n");
    r.push_str(&format!("- Assembly score (within − cross pattern cosine): early-S1 = {:?}, late-S1 = {:?}\n", m.assembly_score_early_s1, m.assembly_score_late_s1));
    if let Some(n) = &m.novelty {
        r.push_str(&format!(
            "- Novelty: D distance to nearest learned = {:?}, max learned pairwise = {:?}; D population rate = {:?} Hz vs learned baseline {:?} Hz\n",
            n.d_distance_to_nearest, n.learned_pairwise_max, n.d_population_rate_hz, n.learned_baseline_rate_hz
        ));
        r.push_str("- NOTE: novelty is computed by instrumentation (U6a), not a claim that the organism *represents* novelty.\n");
    } else {
        r.push_str("- Novelty: insufficient data (no S2/D presentations found).\n");
    }
    r.push_str("\n### Retention (S3 vs late-S1)\n\n");
    r.push_str("| pattern | ratio |\n|---|---|\n");
    for pat in ["A", "B", "C"] {
        let v = m.retention.get(pat).and_then(|x| *x).map(|x| format!("{x:.3}")).unwrap_or_else(|| "—".into());
        r.push_str(&format!("| {pat} | {v} |\n"));
    }

    r.push_str("\n## Interpretation (pre-registered thresholds)\n\n");
    if vs.is_empty() {
        r.push_str("No verdicts computable from this telemetry.\n");
    }
    for v in &vs {
        r.push_str(&format!("- **{} — {}** *(auto-generated — review)*: {}\n", v.claim, v.verdict, v.detail));
    }

    r.push_str("\n## Structural timeline\n\n");
    r.push_str(&format!(
        "- Neuron births: {} — {}\n",
        m.structure.births.len(),
        m.structure.births.iter().take(5).map(|(t, n)| format!("t={t}ms id={n}")).collect::<Vec<_>>().join("; ")
    ));
    r.push_str(&format!("- Synapses created: {}, pruned: {}\n", m.structure.synapse_created, m.structure.synapse_pruned));
    r.push_str(&format!(
        "- Dormancies: {}, reactivations: {}, retirements: {}\n",
        m.structure.dormancies.len(),
        m.structure.reactivations.len(),
        m.structure.retirements.len()
    ));

    r.push_str("\n## Behavioral summary\n\n");
    for (stage, rate) in &m.stage_population_rates {
        r.push_str(&format!("- Stage {stage}: population rate ≈ {rate:?} Hz\n"));
    }

    r.push_str("\n## Resource usage\n\n");
    if let Some(last) = m.resources.samples.last() {
        r.push_str(&format!("- Final: {} neurons, {} synapses at t={} ms\n", last.neurons, last.synapses, last.t));
    }
    r.push_str(&format!("- Total metabolic cost: {:?}\n", m.resources.total_metabolic_cost));
    r.push_str(&format!("- Total spikes (windows): {}\n", m.resources.samples.iter().map(|s| s.spikes_window).sum::<u64>()));

    r.push_str("\n## Failure modes\n\n");
    if m.failures.is_empty() {
        r.push_str("- None.\n");
    } else {
        for f in &m.failures {
            r.push_str(&format!("- t={} ms: **{}** — {}\n", f.t, f.kind, f.detail));
        }
    }

    r.push_str("\n## Next experiment\n\n");
    r.push_str(follow_up);
    r.push('\n');
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventBuilder, Payload};
    use anima_core::network::NeuronId;

    #[test]
    fn verdict_rules_from_thresholds() {
        let mut b = EventBuilder::new("e1");
        let mut evs = vec![b.build(0, Payload::RunStarted {
            config_hash: "h".into(),
            seed: 1,
            params: serde_json::json!({"n_input_channels": 2}),
        })];
        // S1 2×A (10 spikes), S3 1×A (8 spikes) ⇒ retention 0.8 = supported.
        evs.push(b.build(1000, Payload::StimulusPresented { pattern_id: "A".into(), stage: "S1".into() }));
        for i in 0..10 {
            evs.push(b.build(1010 + i * 20, Payload::Spike { n: NeuronId(5) }));
        }
        evs.push(b.build(3000, Payload::StimulusPresented { pattern_id: "A".into(), stage: "S1".into() }));
        for i in 0..10 {
            evs.push(b.build(3010 + i * 20, Payload::Spike { n: NeuronId(5) }));
        }
        evs.push(b.build(8000, Payload::StimulusPresented { pattern_id: "A".into(), stage: "S3".into() }));
        for i in 0..8 {
            evs.push(b.build(8010 + i * 20, Payload::Spike { n: NeuronId(5) }));
        }
        let m = analyze(&evs);
        let vs = verdicts(&m);
        let retention = vs.iter().find(|v| v.claim.starts_with("retention")).expect("retention verdict present");
        assert_eq!(retention.verdict, "supported", "{}", retention.detail);
        // Assembly verdict exists even without enough presentations.
        assert!(vs.iter().any(|v| v.claim == "assembly formation"));
    }

    #[test]
    fn report_has_all_sections() {
        let mut b = EventBuilder::new("e1");
        let evs = vec![
            b.build(0, Payload::RunStarted { config_hash: "h".into(), seed: 1, params: serde_json::json!({}) }),
            b.build(100, Payload::ResourceUsage { neurons: 10, synapses: 20, spikes_window: 5, metabolic_cost: Some(0.1) }),
            b.build(200, Payload::RunEnded { reason: "complete".into() }),
        ];
        let rep = generate_report("e1", &evs, "Hypothesis text.", "- setup line", "E2 next.");
        for section in [
            "# Experiment e1",
            "## Hypothesis",
            "## Setup",
            "## Organism config",
            "## Results",
            "## Interpretation (pre-registered thresholds)",
            "## Structural timeline",
            "## Behavioral summary",
            "## Resource usage",
            "## Failure modes",
            "## Next experiment",
            "auto-generated — review",
        ] {
            assert!(rep.contains(section), "missing section: {section}");
        }
    }
}
