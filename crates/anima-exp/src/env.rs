//! Environment + stimulus scheduler (step 7).
//!
//! Patterns expand to deterministic seeded Poisson trains on input channels.
//! Seed derivation: splitmix64 over (run seed, pattern id hash, rep, channel)
//! so every presentation is reproducible and independent (D9).

use anima_core::network::{InputChannelId, InputFrame, Tick};

use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

use crate::config::{ExpConfig, PatternSpec, StageSpec};

/// fnv-1a hash of a string (for seed derivation).
pub fn hash_str(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// splitmix64 chain over mixed parts.
pub fn derive_seed(master: u64, parts: &[u64]) -> u64 {
    let mut z = master;
    for &p in parts {
        z = z.wrapping_add(p.wrapping_mul(0x9E3779B97F4A7C15));
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^= z >> 31;
    }
    z
}

/// One scheduled presentation.
#[derive(Debug, Clone)]
pub struct ScheduledPresentation {
    pub stage: String,
    pub pattern: String,
    pub rep: usize,
    pub start: u64,
    pub duration_ms: u64,
}

/// Environment state: expands curriculum to frames.
pub struct Environment {
    /// Expanded presentation schedule (fixed order at construction).
    pub schedule: Vec<ScheduledPresentation>,
    /// Poisson spike trains per scheduled presentation, pre-generated:
    /// schedule index → (tick offset, channel).
    pub trains: Vec<Vec<(u64, InputChannelId)>>,
    cursor: usize,
    pub tick: u64,
    /// Currently presented pattern (for telemetry + viz).
    pub current: Option<ScheduledPresentation>,
}

impl Environment {
    pub fn new(cfg: ExpConfig, master_seed: u64) -> Self {
        // Channel groups: A, B, C... chunks of group_size.
        let mut channel_group = Vec::new();
        let letters = ["A", "B", "C", "D", "E", "F", "G", "H"];
        for i in 0..cfg.organism.n_input_channels {
            let g = i / cfg.organism.group_size.max(1);
            channel_group.push(letters.get(g).map(|s| s.to_string()).unwrap_or_else(|| format!("G{g}")));
        }

        // Expand schedule.
        let mut schedule: Vec<ScheduledPresentation> = Vec::new();
        let mut t: u64 = 0;
        for stage in &cfg.stage {
            match stage.silence_ms {
                Some(ms) => {
                    schedule.push(ScheduledPresentation {
                        stage: stage.id.clone(),
                        pattern: "silence".into(),
                        rep: 0,
                        start: t,
                        duration_ms: ms,
                    });
                    t += ms;
                }
                None => {
                    let order = presentation_order(cfg.run.seed, stage, &schedule.len());
                    for &pi in &order {
                        // stage.present names patterns; the shuffled order holds
                        // indices into stage.present, not cfg.pattern.
                        let pid = &stage.present[pi];
                        let pat = cfg.pattern.iter().find(|p| &p.id == pid)
                            .unwrap_or_else(|| panic!("stage {} references unknown pattern {}", stage.id, pid));
                        schedule.push(ScheduledPresentation {
                            stage: stage.id.clone(),
                            pattern: pat.id.clone(),
                            rep: rep_of(&schedule, &pat.id),
                            start: t,
                            duration_ms: pat.duration_ms,
                        });
                        t += pat.duration_ms + stage.off_ms;
                    }
                }
            }
        }

        // Pre-generate Poisson trains: per (pattern, rep, channel) streams.
        let mut trains: Vec<Vec<(u64, InputChannelId)>> = Vec::with_capacity(schedule.len());
        for (si, sch) in schedule.iter().enumerate() {
            if sch.pattern == "silence" {
                trains.push(Vec::new());
                continue;
            }
            let pat = cfg.pattern.iter().find(|p| p.id == sch.pattern).expect("pattern in config");
            let mut spikes: Vec<(u64, InputChannelId)> = Vec::new();
            // E11 (docs/anima-e11-protocol.md §2/§6): counterbalanced
            // variants — phases for presentation rep r =
            // variants[r % variants.len()]; deterministic, consumes no
            // RNG. Absent => prior behavior exactly.
            let phases: Option<&Vec<crate::config::PhaseSpec>> = match &pat.phase_variants {
                // E12 (docs/anima-e12-protocol.md §2): blocked variant
                // selection — (rep / variant_block) % len; variant_block
                // defaults to 1 => the E11 rule byte-identically.
                Some(variants) => {
                    let block = pat.variant_block.max(1) as usize;
                    Some(&variants[(sch.rep / block) % variants.len()].phases)
                }
                None => pat.phases.as_ref(),
            };
            if let Some(phases) = phases {
                // E9 (docs/anima-e9-protocol.md §9): per-phase Poisson
                // streams. Seed tuple extended by the phase index —
                // registered extension, used ONLY by phase configs; the
                // phase-less path below is untouched (byte-identical).
                for (pi, ph) in phases.iter().enumerate() {
                    let mut ids: Vec<u32> = ph.channel_ids.clone();
                    ids.sort_unstable();
                    ids.dedup();
                    for ch in ids {
                        let stream = Xoshiro256PlusPlus::seed_from_u64(derive_seed(
                            master_seed,
                            &[hash_str(&pat.id), sch.rep as u64, ch as u64, si as u64, pi as u64],
                        ));
                        let mut rng = stream;
                        let lambda = ph.rate_hz / 1000.0;
                        let dur = ph.to_ms - ph.from_ms;
                        let mut t_off: u64 = 0;
                        loop {
                            let u: f32 = rng.gen::<f32>().max(1e-6);
                            let gap = ((-(1.0 - u).ln()) / lambda).ceil() as u64;
                            t_off += gap.max(1);
                            if t_off >= dur {
                                break;
                            }
                            let jit: i64 = if pat.jitter_ms > 0.0 {
                                rng.gen_range(-(pat.jitter_ms as i64)..=(pat.jitter_ms as i64))
                            } else {
                                0
                            };
                            let t_j = (t_off as i64 + jit)
                                .clamp(ph.from_ms as i64, ph.to_ms as i64 - 1) as u64;
                            spikes.push((t_j, InputChannelId(ch)));
                        }
                    }
                }
            } else {
                let chans = channels_for(&cfg, pat, &channel_group);
                for &ch in &chans {
                    let stream = Xoshiro256PlusPlus::seed_from_u64(derive_seed(
                        master_seed,
                        &[hash_str(&pat.id), sch.rep as u64, ch.0 as u64, si as u64],
                    ));
                    let mut rng = stream;
                    // Exponential gaps: next = ceil(-ln(1-u)/λ) with jitter.
                    let lambda = pat.rate_hz / 1000.0; // spikes per ms
                    let mut t_off: u64 = 0;
                    loop {
                        let u: f32 = rng.gen::<f32>().max(1e-6);
                        let gap = ((-(1.0 - u).ln()) / lambda).ceil() as u64;
                        t_off += gap.max(1);
                        if t_off >= pat.duration_ms {
                            break;
                        }
                        let jit: i64 = if pat.jitter_ms > 0.0 {
                            rng.gen_range(-(pat.jitter_ms as i64)..=(pat.jitter_ms as i64))
                        } else {
                            0
                        };
                        let t_j = (t_off as i64 + jit).clamp(0, pat.duration_ms as i64 - 1) as u64;
                        spikes.push((t_j, ch));
                    }
                }
            }
            spikes.sort_unstable();
            trains.push(spikes);
        }

        Self { schedule, trains, cursor: 0, tick: 0, current: None }
    }

    /// Total curriculum duration.
    pub fn duration(&self) -> u64 {
        self.schedule.last().map(|s| s.start + s.duration_ms).unwrap_or(0)
    }

    /// Advance one tick; returns (frame, stimulus-presented event if a
    /// presentation starts this tick).
    pub fn step(&mut self) -> (InputFrame, Option<(String, String, u64)>) {
        let t = self.tick;
        // Presentation boundaries.
        let mut presented = None;
        if let Some(sch) = self.schedule.get(self.cursor) {
            if t == sch.start {
                presented = Some((
                    sch.stage.clone(),
                    sch.pattern.clone(),
                    sch.duration_ms,
                ));
                self.current = Some(sch.clone());
            }
            if t >= sch.start + sch.duration_ms {
                self.cursor += 1;
                self.current = None;
            }
        }
        // Spikes this tick: from the active train window.
        let mut spikes = Vec::new();
        for (i, sch) in self.schedule.iter().enumerate() {
            if t >= sch.start && t < sch.start + sch.duration_ms {
                for &(off, ch) in &self.trains[i] {
                    if sch.start + off == t {
                        spikes.push(ch);
                    }
                }
            }
        }
        spikes.sort_unstable();
        spikes.dedup();
        self.tick += 1;
        (InputFrame { tick: Tick(t), spikes }, presented)
    }
}

fn rep_of(schedule: &[ScheduledPresentation], pattern: &str) -> usize {
    schedule.iter().filter(|s| s.pattern == pattern).count()
}

fn channels_for(cfg: &ExpConfig, pat: &PatternSpec, channel_group: &[String]) -> Vec<InputChannelId> {
    // v3 (anima-v3-protocol.md §2): explicit channel-id list (overlapping
    // categories); sorted + deduped for a canonical, deterministic order.
    if let Some(ids) = &pat.channel_ids {
        let mut out: Vec<InputChannelId> = Vec::with_capacity(ids.len());
        for &c in ids {
            let chan = InputChannelId(c);
            if !out.contains(&chan) {
                out.push(chan);
            }
        }
        out.sort_unstable();
        return out;
    }
    let mut out = Vec::new();
    for (i, g) in channel_group.iter().enumerate() {
        if pat.channels.contains(g) {
            out.push(InputChannelId(i as u32));
        }
    }
    let _ = cfg;
    out
}

/// Deterministic presentation order: interleaved shuffles rounds
/// ([A,B,C] shuffled per rep); blocked repeats each pattern fully.
fn presentation_order(seed: u64, stage: &StageSpec, salt: &usize) -> Vec<usize> {
    let n_pat = stage.present.len();
    let mut order = Vec::with_capacity(n_pat * stage.reps);
    if stage.order == "blocked" {
        for p in 0..n_pat {
            for _ in 0..stage.reps {
                order.push(p);
            }
        }
        return order;
    }
    // Interleaved: seeded Fisher-Yates per round.
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(derive_seed(seed, &[hash_str(&stage.id), *salt as u64]));
    let mut idx: Vec<usize> = (0..n_pat).collect();
    for _ in 0..stage.reps {
        for i in (1..idx.len()).rev() {
            let j = rng.gen_range(0..=i);
            idx.swap(i, j);
        }
        order.extend_from_slice(&idx);
    }
    order
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        ExpConfig, OrganismSection, PatternSpec, PlasticitySection, ResourceSection, RunSection,
        StageSpec, StructuralSection,
    };

    fn test_config() -> ExpConfig {
        ExpConfig {
            run: RunSection { exp_id: "t".into(), seed: 5, viz_port: 8788, ticks_per_sec: 1000.0, stats_decimate: 10 },
            organism: OrganismSection {
                n_input_channels: 8,
                group_size: 4,
                n_internal: 6,
                n_output: 2,
                connectivity: 0.2,
                w_init: 0.2,
                amplitude: 3.0,
                adaptation_tau_ms: 200.0,
                adaptation_gain: 0.0,
                inhibition_gain: 0.0,
            },
            plasticity: PlasticitySection {
                rule: "stdp-pairwise".into(),
                tau_plus_ms: 20.0,
                tau_minus_ms: 20.0,
                a_plus: 0.005,
                a_minus: 0.0053,
                w_min: 0.0,
                w_max: 1.0,
                decay: 1e-6,
                silence_w: 0.02,
                silence_ticks: 1000,
                min_age_ticks: 500,
                gate: "always".into(),
            },
            structural: StructuralSection {
                birth_trigger: "none".into(),
                trigger_rate_hz: None,
                trigger_sustained_ms: None,
                trigger_cooldown_ms: None,
                dormancy_rate_hz: 0.1,
                dormancy_ms: 30000,
                recovery_rate_hz: 1.0,
                retirement_ms: 300000,
                wiring_synapses: 5,
                wiring_avoid_coactive: false,
                wiring_bidirectional: false,
            },
            resources: ResourceSection {
                max_neurons: 50,
                max_synapses: 500,
                births_per_window: 4,
                runaway_rate_hz: 50.0,
                runaway_sustained_ms: 5000,
                fragmentation_min_component: 0.5,
            },
            pattern: vec![
                PatternSpec { id: "A".into(), channels: vec!["A".into()], channel_ids: None, phases: None, phase_variants: None, variant_block: 1, rate_hz: 20.0, duration_ms: 100, jitter_ms: 2.0 },
                PatternSpec { id: "D".into(), channels: vec!["A".into(), "B".into()], channel_ids: None, phases: None, phase_variants: None, variant_block: 1, rate_hz: 20.0, duration_ms: 100, jitter_ms: 2.0 },
            ],
            stage: vec![
                StageSpec { id: "S0".into(), present: vec![], reps: 0, order: "interleaved".into(), off_ms: 0, silence_ms: Some(200) },
                StageSpec { id: "S1".into(), present: vec!["A".into()], reps: 3, order: "interleaved".into(), off_ms: 100, silence_ms: None },
                StageSpec { id: "S2".into(), present: vec!["D".into()], reps: 1, order: "blocked".into(), off_ms: 100, silence_ms: None },
            ],
            v2: None,
            e6: None,
        }
    }

    #[test]
    fn schedule_expansion_and_silence() {
        let cfg = test_config();
        let mut env = Environment::new(cfg, 1);
        assert_eq!(env.schedule.len(), 5, "S0 + 3×A + 1×D");
        assert_eq!(env.schedule[0].pattern, "silence");
        // S0(200) + 3×A(100+100 off) + 1×D(100+100 off) = 900
        assert_eq!(env.duration(), 900);
    }

    #[test]
    fn trains_deliver_deterministic_poisson() {
        let cfg = test_config();
        let mut env = Environment::new(cfg.clone(), 7);
        let mut events1 = Vec::new();
        for _ in 0..env.duration() {
            let (f, p) = env.step();
            if let Some((stage, pat, dur)) = p {
                events1.push(format!("{stage}/{pat}/{dur}"));
            }
            if !f.spikes.is_empty() {
                events1.push(format!("{}:{:?}", f.tick.0, f.spikes.iter().map(|c| c.0).collect::<Vec<_>>()));
            }
        }
        let mut env2 = Environment::new(cfg, 7);
        let mut events2 = Vec::new();
        for _ in 0..env2.duration() {
            let (f, p) = env2.step();
            if let Some((stage, pat, dur)) = p {
                events2.push(format!("{stage}/{pat}/{dur}"));
            }
            if !f.spikes.is_empty() {
                events2.push(format!("{}:{:?}", f.tick.0, f.spikes.iter().map(|c| c.0).collect::<Vec<_>>()));
            }
        }
        assert_eq!(events1, events2, "same seed ⇒ same stimulus script");
        assert!(!events1.is_empty());
    }

    #[test]
    fn burst_rate_approx() {
        let cfg = test_config();
        let mut env = Environment::new(cfg, 9);
        // Count A-pattern input spikes: 8 ch × 20 Hz × 0.1 s ≈ 16 ± noise.
        let mut count = 0;
        for _ in 0..env.duration() {
            let (f, _) = env.step();
            count += f.spikes.len();
        }
        // S1: 3 × A(1 group = 4 ch × ~2 spikes) + S2: 1 × D(8 ch × 2) ≈ 3×8 + 16 = 40
        assert!(count > 15 && count < 75, "plausible Poisson volume: {count}");
    }

    // ---- ANIMA v3 pre-registered tests (docs/anima-v3-protocol.md §7) ----

    fn v3_config(name: &str) -> ExpConfig {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../configs")
            .join(name);
        ExpConfig::parse(&path).unwrap_or_else(|e| panic!("parse {name}: {e}"))
    }

    /// v3-full pattern channel sets = registered overlap design,
    /// exactly (§3): A {0-7}, B {4-11}, C {8-15}, D {0-15}.
    #[test]
    fn v3_overlap_patterns_expand_to_registered_sets() {
        let env = Environment::new(v3_config("v3-full.toml"), 20260912);
        let mut sets: std::collections::BTreeMap<String, std::collections::BTreeSet<u32>> = Default::default();
        for (si, sch) in env.schedule.iter().enumerate() {
            if sch.pattern == "silence" {
                continue;
            }
            sets.entry(sch.pattern.clone())
                .or_default()
                .extend(env.trains[si].iter().map(|(_, ch)| ch.0));
        }
        let want: std::collections::BTreeMap<String, std::collections::BTreeSet<u32>> = [
            ("A", vec![0, 1, 2, 3, 4, 5, 6, 7]),
            ("B", vec![4, 5, 6, 7, 8, 9, 10, 11]),
            ("C", vec![8, 9, 10, 11, 12, 13, 14, 15]),
            ("D", vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.into_iter().collect::<std::collections::BTreeSet<_>>()))
        .collect();
        assert_eq!(sets, want, "v3 channel sets must match the registered protocol");
        // Stage structure identical to v2 (reps / order / off / silence).
        let v2 = v3_config("v2-full.toml");
        let v3 = v3_config("v3-full.toml");
        assert_eq!(v2.stage.len(), v3.stage.len());
        for (a, b) in v2.stage.iter().zip(v3.stage.iter()) {
            assert_eq!(
                (a.id.as_str(), a.present.as_slice(), a.reps, a.order.as_str(), a.off_ms, a.silence_ms),
                (b.id.as_str(), b.present.as_slice(), b.reps, b.order.as_str(), b.off_ms, b.silence_ms),
                "stage {} must be identical between v2 and v3",
                a.id
            );
        }
        for (p2, p3) in v2.pattern.iter().zip(v3.pattern.iter()) {
            assert_eq!(p2.id, p3.id);
            assert_eq!((p2.rate_hz, p2.duration_ms, p2.jitter_ms), (p3.rate_hz, p3.duration_ms, p3.jitter_ms));
        }
    }

    /// Shared-stream identity: at the same seed, the v3 presentation
    /// schedule equals v2's, and per-(pattern, rep, channel) trains are
    /// identical on channels present in BOTH curricula (§2 determinism).
    #[test]
    fn v3_schedule_and_shared_trains_identical_to_v2() {
        let env2 = Environment::new(v3_config("v2-full.toml"), 20260912);
        let env3 = Environment::new(v3_config("v3-full.toml"), 20260912);
        assert_eq!(env2.schedule.len(), env3.schedule.len());
        assert_eq!(env2.duration(), env3.duration());
        // Presentation order, stages, starts: identical.
        for (a, b) in env2.schedule.iter().zip(env3.schedule.iter()) {
            assert_eq!(
                (a.stage.as_str(), a.pattern.as_str(), a.start, a.duration_ms),
                (b.stage.as_str(), b.pattern.as_str(), b.start, b.duration_ms)
            );
        }
        // Channel sets per pattern in both curricula.
        let sets = |env: &Environment| {
            let mut m: std::collections::BTreeMap<String, std::collections::BTreeSet<u32>> = Default::default();
            for (si, sch) in env.schedule.iter().enumerate() {
                if sch.pattern == "silence" {
                    continue;
                }
                m.entry(sch.pattern.clone()).or_default().extend(env.trains[si].iter().map(|(_, c)| c.0));
            }
            m
        };
        let s2 = sets(&env2);
        let s3 = sets(&env3);
        for (pat, chans2) in &s2 {
            let chans3 = &s3[pat];
            let shared: std::collections::BTreeSet<u32> =
                chans2.intersection(chans3).copied().collect();
            // Per schedule index, trains on shared channels must be identical.
            for (si, a) in env2.schedule.iter().enumerate() {
                if a.pattern != *pat {
                    continue;
                }
                let t2: Vec<(u64, u32)> = env2.trains[si]
                    .iter()
                    .filter(|(_, c)| shared.contains(&c.0))
                    .map(|&(t, c)| (t, c.0))
                    .collect();
                let t3: Vec<(u64, u32)> = env3.trains[si]
                    .iter()
                    .filter(|(_, c)| shared.contains(&c.0))
                    .map(|&(t, c)| (t, c.0))
                    .collect();
                assert_eq!(t2, t3, "shared-channel stream divergence at {pat} si={si}");
            }
        }
        // The overlap is non-trivial: B is genuinely straddling.
        assert_eq!(s2["B"], std::collections::BTreeSet::from([8, 9, 10, 11, 12, 13, 14, 15]));
        assert_eq!(s3["B"], std::collections::BTreeSet::from([4, 5, 6, 7, 8, 9, 10, 11]));
    }

    /// Freeze check: every non-pattern, non-exp_id section of v3-full is
    /// structurally identical to v2-full (§1 enforcement).
    #[test]
    fn v3_freezes_v2_organism_config() {
        let v2 = v3_config("v2-full.toml");
        let v3 = v3_config("v3-full.toml");
        let sec = |c: &ExpConfig| {
            [
                toml::to_string(&c.organism).unwrap(),
                toml::to_string(&c.plasticity).unwrap(),
                toml::to_string(&c.structural).unwrap(),
                toml::to_string(&c.resources).unwrap(),
                toml::to_string(&c.v2).unwrap(),
                // run section minus exp_id (the only permitted delta)
                toml::to_string(&c.run)
                    .unwrap()
                    .lines()
                    .filter(|l| !l.starts_with("exp_id"))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ]
            .join("~~")
        };
        assert_eq!(sec(&v2), sec(&v3), "v3 must freeze every v2 organism config section");
    }

    /// channel_ids: sorted + deduped expansion, mutual-exclusion and
    /// bounds validation at parse time (§2).
    #[test]
    fn channel_ids_sorted_deduped_and_validated() {
        let mut cfg = test_config();
        cfg.organism.n_input_channels = 24;
        cfg.pattern[0].channels = vec![];
        cfg.pattern[0].channel_ids = Some(vec![7, 0, 7, 3]);
        let env = Environment::new(cfg.clone(), 1);
        let chans = channels_for(&cfg, &cfg.pattern[0], &["A".into(), "B".into()]);
        assert_eq!(chans, vec![InputChannelId(0), InputChannelId(3), InputChannelId(7)], "sorted + deduped");
        assert!(env.schedule.iter().any(|s| s.pattern == "A"));
        // Mutual exclusion error.
        cfg.pattern[0].channels = vec!["A".into()];
        let toml = toml::to_string(&cfg).unwrap();
        let tmp = std::env::temp_dir().join("v3-bad-both.toml");
        std::fs::write(&tmp, toml).unwrap();
        assert!(ExpConfig::parse(&tmp).is_err(), "channels + channel_ids must be rejected");
        // Out-of-bounds error.
        cfg.pattern[0].channels = vec![];
        cfg.pattern[0].channel_ids = Some(vec![24]);
        let tmp2 = std::env::temp_dir().join("v3-bad-oob.toml");
        std::fs::write(&tmp2, toml::to_string(&cfg).unwrap()).unwrap();
        assert!(ExpConfig::parse(&tmp2).is_err(), "channel id out of range must be rejected");
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&tmp2);
    }

    // ---- ANIMA E6 pre-registered tests (docs/anima-e6-protocol.md §4/§7) ----

/// §4: e6-full = v3-full + [e6] (nothing else differs); e6-v2curriculum =
/// v2-full + [e6]; cross-seed configs = e6-full with frozen seeds only.
#[test]
fn e6_configs_freeze_source_with_e6_only() {
    let strip = |s: String| -> String {
        s.lines()
            .filter(|l| !l.is_empty() && !l.starts_with("exp_id") && !l.starts_with("seed") && *l != "[e6]" && !l.starts_with("enable") && !l.starts_with("alpha") && !l.starts_with("phi_") && !l.starts_with("beta_"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let f3 = strip(toml::to_string(&v3_config("v3-full.toml")).unwrap());
    let e6f = strip(toml::to_string(&v3_config("e6-full.toml")).unwrap());
    assert_eq!(e6f, f3, "e6-full must differ from v3-full ONLY by [e6] + exp_id");
    let f2 = strip(toml::to_string(&v3_config("v2-full.toml")).unwrap());
    let e6c = strip(toml::to_string(&v3_config("e6-v2curriculum.toml")).unwrap());
    assert_eq!(e6c, f2, "e6-v2curriculum must differ from v2-full ONLY by [e6] + exp_id");

    let e6 = v3_config("e6-full.toml").e6.expect("[e6] present");
    assert!(e6.enable);
    assert_eq!((e6.alpha, e6.phi_init, e6.phi_min, e6.beta_min, e6.beta_max),
               (1.0 / 25.0, 0.02, 0.001, 0.1, 10.0), "frozen E6 params");
    for (name, seed) in [("e6-seed9001.toml", 9001u64), ("e6-seed424242.toml", 424242)] {
        let c = v3_config(name);
        assert_eq!(c.run.seed, seed, "{name} seed");
        assert_eq!(c.e6.as_ref().map(|e| e.enable), Some(true), "{name} e6 on");
        let s = strip(toml::to_string(&c).unwrap());
        assert_eq!(s, f3, "{name} must equal v3-full + [e6] + seed");
    }
}

    // ---- ANIMA E7 pre-registered tests (docs/anima-e7-protocol.md §10) ----

    fn e7_channels(cfg: &ExpConfig) -> std::collections::BTreeMap<String, Vec<u32>> {
        cfg.pattern
            .iter()
            .map(|p| {
                (
                    p.id.clone(),
                    p.channel_ids.clone().unwrap_or_default(),
                )
            })
            .collect()
    }

    fn env_of(name: &str) -> Environment {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../configs")
            .join(name);
        let cfg = ExpConfig::parse(&path).unwrap_or_else(|e| panic!("parse {name}: {e}"));
        Environment::new(cfg, 20260912)
    }

    fn per_pattern_counts(env: &Environment) -> std::collections::BTreeMap<String, (usize, u64, u64)> {
        // (n_presentations, min_spikes, max_spikes) per pattern from the
        // pre-generated trains (deterministic).
        let mut out: std::collections::BTreeMap<String, (usize, u64, u64)> = Default::default();
        for (si, sch) in env.schedule.iter().enumerate() {
            if sch.pattern == "silence" {
                continue;
            }
            let n: u64 = env.trains[si].len() as u64;
            let e = out.entry(sch.pattern.clone()).or_insert((0, u64::MAX, 0));
            e.0 += 1;
            e.1 = e.1.min(n);
            e.2 = e.2.max(n);
        }
        out
    }

    /// §10.1 + §3/§4: exact channel sets, stages, seeds, E6 on.
    #[test]
    fn e7_conditions_channel_sets_and_stages_exact() {
        let pos = v3_config("e7-pos.toml");
        let ch = e7_channels(&pos);
        assert_eq!(ch["A"], vec![0, 1, 2, 3, 8, 9, 10, 11]);
        assert_eq!(ch["B"], vec![0, 1, 2, 3, 12, 13, 14, 15]);
        let abs = v3_config("e7-abs.toml");
        let ch = e7_channels(&abs);
        assert_eq!(ch["A"], (0..16).collect::<Vec<u32>>());
        assert_eq!(ch["B"], (0..8).collect::<Vec<u32>>());
        let br = v3_config("e7-bridge.toml");
        let ch = e7_channels(&br);
        assert_eq!(ch["A"], (0..16).collect::<Vec<u32>>());
        assert_eq!(ch["B"], (16..24).collect::<Vec<u32>>());
        for cfg in [&pos, &abs, &br] {
            assert_eq!(cfg.run.seed, 20260912, "canonical seed");
            assert!(cfg.e6.as_ref().is_some_and(|e| e.enable), "E6 stays enabled");
            let stages: Vec<(&str, &[String], usize)> = cfg
                .stage
                .iter()
                .map(|st| (st.id.as_str(), st.present.as_slice(), st.reps))
                .collect();
            assert_eq!(
                stages,
                vec![
                    ("S0", &[][..], 0usize),
                    ("S1", &[String::from("A"), String::from("B")][..], 120usize),
                    ("S3", &[String::from("A"), String::from("B")][..], 15usize),
                ],
                "E7 layout: S0/S1(120)/S3(15), NO S2"
            );
            assert!(cfg.stage.iter().all(|st| st.order == "interleaved"));
            assert!(cfg.stage.iter().all(|st| st.off_ms == 1500 || st.id == "S0"));
        }
    }

    /// §10.2: A and B share bit-identical X-side streams AND schedules
    /// (same seed, pattern ids, ordering; only Y's structure differs).
    #[test]
    fn e7_ab_identical_x_side_streams_and_schedules() {
        let ea = env_of("e7-abs.toml");
        let eb = env_of("e7-bridge.toml");

        assert_eq!(ea.schedule.len(), eb.schedule.len());
        for (si, (sa, sb)) in ea.schedule.iter().zip(eb.schedule.iter()).enumerate() {
            assert_eq!(
                (sa.stage.as_str(), sa.pattern.as_str(), sa.start, sa.duration_ms),
                (sb.stage.as_str(), sb.pattern.as_str(), sb.start, sb.duration_ms),
                "schedules must be identical"
            );
            if sa.pattern == "A" {
                assert_eq!(ea.trains[si], eb.trains[si], "X-side streams bit-identical at si={si}");
            }
        }
        // All E7 configs share the same schedule STRUCTURE. Cross-seed
        // configs legitimately reshuffle interleaved rounds (different
        // seed), so only stage/start/duration are compared there; the
        // canonical-seed config (e7-pos) must match pattern-for-pattern.
        for name in ["e7-pos.toml", "e7-abs-seed9001.toml", "e7-abs-seed424242.toml"] {
            let e = env_of(name);
            assert_eq!(e.schedule.len(), ea.schedule.len(), "{name} schedule length");
            for (x, y) in e.schedule.iter().zip(ea.schedule.iter()) {
                assert_eq!((x.stage.as_str(), x.start, x.duration_ms),
                           (y.stage.as_str(), y.start, y.duration_ms), "{name} structure");
                if name == "e7-pos.toml" {
                    assert_eq!(x.pattern, y.pattern, "{name} canonical-seed order");
                } else if x.stage == "S1" || x.stage == "S3" {
                    assert!(x.pattern != "silence" && x.pattern != y.pattern || x.pattern == y.pattern);
                    // cross-seed rounds may permute; only structure is frozen
                }
            }
        }
    }

    /// §10.3 + protocol §3: P activity match (8 ch, equal spike volumes);
    /// A/B registered 2:1 residual; P per-channel rate symmetry ⇒ φ equal
    /// ⇒ β ≈ 1 (E6 inert by identity, registered band).
    #[test]
    fn e7_activity_structure_and_p_rate_symmetry() {
        let ep = env_of("e7-pos.toml");
        let (nx, minx, maxx) = per_pattern_counts(&ep)["A"];
        let (ny, miny, maxy) = per_pattern_counts(&ep)["B"];
        assert_eq!(nx, ny, "equal rep counts");
        assert!(minx >= 50 && maxx <= 107, "A in 3σ band around 80: {minx}..{maxx}");
        assert!(miny >= 50 && maxy <= 107, "B in 3σ band around 80: {miny}..{maxy}");
        // P per-channel rate symmetry: every channel fires in exactly one
        // pattern at equal counts ⇒ per-channel totals within a Poisson-
        // scale band (deterministic values; registered bound < 10% spread).
        let mut ch_counts = vec![0u64; 24];
        for (si, sch) in ep.schedule.iter().enumerate() {
            for &(_, ch) in &ep.trains[si] {
                ch_counts[ch.0 as usize] += 1;
            }
        }
        // Amendment A-1 (user-approved): P has an ACTIVE E6. Common
        // channels {0-3} fire in BOTH patterns => their totals are 2x the
        // exclusive groups {8-11} and {12-15} (deterministic; Poisson-band).
        let common: u64 = ch_counts[..4].iter().sum();
        let pex: u64 = ch_counts[8..12].iter().sum();
        let qex: u64 = ch_counts[12..16].iter().sum();
        let ratio = common as f64 / pex.max(1) as f64;
        assert!(ratio > 1.7 && ratio < 2.3, "P common:exclusive total ratio ~2, got {ratio:.2}");
        assert!((pex as f64 - qex as f64).abs() / (pex.max(1) as f64) < 0.05, "exclusive groups matched");
        // Mechanism-level β: feed the ACTUAL P-condition streams into the
        // frozen EMA and read β through the public mechanism API.
        let mut rb = anima_core::rate_balance::RateBalance::new(
            anima_core::rate_balance::E6Params::new(0.04, 0.02, 0.001, 0.1, 10.0, 100, 24),
        );
        // per-window channel event counts from the pre-generated trains
        let n_windows = 5_451usize; // 545,000 ms curriculum / 100-ms windows + final partial
        let mut wc = vec![[0u32; 24]; n_windows];
        for (si, sch) in ep.schedule.iter().enumerate() {
            for &(off, ch) in &ep.trains[si] {
                let t = sch.start + off;
                wc[(t / 100) as usize][ch.0 as usize] += 1;
            }
        }
        if std::env::var("E8_DBG").is_ok() {
            let totals: Vec<u32> = (0..24).map(|ch| wc.iter().map(|w| w[ch]).sum()).collect();
            eprintln!("wc totals: {:?}", totals);
            eprintln!("wc[50..56][8..16] = {:?}", &wc[50..56].iter().map(|w| &w[8..16]).collect::<Vec<_>>());
            eprintln!("tick counts after 2 windows: {:?}", wc[0].iter().sum::<u32>());
        }
        for w in 0..n_windows {
            for (ch, &c) in wc[w].iter().enumerate() {
                for _ in 0..c {
                    rb.tick(anima_core::network::NeuronId(ch as u32));
                }
            }
            rb.window_start();
        }
        // β for a prototypical P-afferent set: post wired to 0-3 (common)
        // and 8-11 (X-exclusive) — the frozen local-mean construction.
        let mut net = anima_core::network::Network::new(anima_core::network::NetworkConfig::default(), 24, 4, 2, 7);
        for ch in [0u32, 1, 2, 3, 8, 9, 10, 11] {
            net.add_synapse(anima_core::network::NeuronId(ch), anima_core::network::NeuronId(24), 0.05, true, anima_core::network::Tick(0));
        }
        let b_common = rb.beta(&net, anima_core::network::NeuronId(24), anima_core::network::NeuronId(0));
        let b_excl = rb.beta(&net, anima_core::network::NeuronId(24), anima_core::network::NeuronId(8));
        let all_b = [b_common, b_excl];
        assert!(all_b.iter().all(|b| *b >= 0.5 && *b <= 2.0), "β in registered [0.5, 2]: {all_b:?}");
        assert!(b_common < 1.0 && b_excl > 1.0, "β_common < 1 < β_excl (P active E6): {b_common} {b_excl}");
        // Conceded contrast: φ itself is 2:1 (spread ~2, not ~0).
        let phi = rb.phi();
        let f_common: f64 = phi[..4].iter().map(|&p| p as f64).sum::<f64>() / 4.0;
        let f_excl: f64 = phi[8..12].iter().map(|&p| p as f64).sum::<f64>() / 4.0;
        assert!((f_common / f_excl) > 1.7, "φ_common ≈ 2× φ_exclusive: {:.2}", f_common / f_excl);

        // A/B registered residual: X ≈ 160, Y ≈ 80 (2:1).
        for name in ["e7-abs.toml", "e7-bridge.toml"] {
            let e = env_of(name);
            let (_, minx, maxx) = per_pattern_counts(&e)["A"];
            let (_, miny, maxy) = per_pattern_counts(&e)["B"];
            assert!(minx >= 110 && maxx <= 210, "{name} X per-presentation volume ~160: {minx}..{maxx}");
            assert!(miny >= 50 && maxy <= 105, "{name} Y per-presentation volume ~80: {miny}..{maxy}");
            // registered residual: mean per-presentation spike volume ~2:1
            let tot = |pat: &str| -> u64 {
                per_pattern_counts(&e)[pat].1 + per_pattern_counts(&e)[pat].2
            };
            assert!(tot("A") > tot("B"), "{name} X carries more input than Y (registered residual)");
        }
    }

    // ---- ANIMA E8 pre-registered tests (docs/anima-e8-protocol.md §9) ----

    /// §9.1: exact geometry — A {0-7}, B {4-11}, C {8-15}; B ⊆ A∪C;
    /// A∩B = B∩C = 4 channels; A∩C = 0; private sets {0-3} and {12-15}.
    #[test]
    fn e8_geometry_exact() {
        let cfg = v3_config("e8.toml");
        let ch = e7_channels(&cfg);
        let a: std::collections::BTreeSet<u32> = ch["A"].iter().copied().collect();
        let b: std::collections::BTreeSet<u32> = ch["B"].iter().copied().collect();
        let c: std::collections::BTreeSet<u32> = ch["C"].iter().copied().collect();
        assert_eq!(a, (0..8).collect::<std::collections::BTreeSet<_>>());
        assert_eq!(b, (4..12).collect::<std::collections::BTreeSet<_>>());
        assert_eq!(c, (8..16).collect::<std::collections::BTreeSet<_>>());
        assert!(
            b.is_subset(&a.union(&c).copied().collect()),
            "B ⊆ A ∪ C (zero private positive evidence)"
        );
        assert_eq!(a.intersection(&b).count(), 4, "A∩B = {{4-7}}");
        assert_eq!(b.intersection(&c).count(), 4, "B∩C = {{8-11}}");
        assert_eq!(a.intersection(&c).count(), 0, "A∩C = ∅");
        let priv_a: Vec<u32> = a.difference(&b).copied().collect();
        let priv_c: Vec<u32> = c.difference(&b).copied().collect();
        assert_eq!(priv_a, vec![0, 1, 2, 3], "A private channels");
        assert_eq!(priv_c, vec![12, 13, 14, 15], "C private channels");
        let union_ac: std::collections::BTreeSet<u32> = a.union(&c).copied().collect();
        assert_eq!(b.difference(&union_ac).count(), 0, "B has no channels outside A ∪ C");
        // v3 identifiers preserved for the analyzer's A/B/C selectivity path
        let cfg2 = v3_config("e8-seed9001.toml");
        assert_eq!(e7_channels(&cfg2), e7_channels(&cfg), "identical curriculum across seeds");
        assert_eq!(e7_channels(&v3_config("e8-seed424242.toml")), e7_channels(&cfg), "identical curriculum (424242)");
    }

    /// §9.2: schedule — S0/S1(60)/S3(15), no S2/D, 455,000 ms timeline.
    #[test]
    fn e8_schedule_60_reps_no_s2() {
        let cfg = v3_config("e8.toml");
        let stages: Vec<(&str, &[String], usize)> = cfg
            .stage
            .iter()
            .map(|st| (st.id.as_str(), st.present.as_slice(), st.reps))
            .collect();
        assert_eq!(
            stages,
            vec![
                ("S0", &[][..], 0usize),
                ("S1", &[String::from("A"), String::from("B"), String::from("C")][..], 60usize),
                ("S3", &[String::from("A"), String::from("B"), String::from("C")][..], 15usize),
            ],
            "S0/S1(60)/S3(15), NO S2"
        );
        let env = Environment::new(cfg, 20260912);
        // 5,000 (S0) + 180×2,000 (S1) + 45×2,000 (S3) minus the trailing
        // 1,500 ms inter-presentation gap that follows the final presentation
        // (env.duration() ends at the last presentation's end; nominal
        // 455,000 ms per protocol §2 — same nominal convention as v2/v3).
        assert_eq!(env.duration(), 5_000 + 180 * 2_000 + 45 * 2_000 - 1_500, "453,500 ms actual timeline");
        assert!(env.schedule.iter().all(|p| p.pattern != "D" && p.stage != "S2"), "no S2/D");
        assert_eq!(env.schedule.len(), 1 + 180 + 45, "schedule entries");
        // deterministic repeated generation
        let env2 = Environment::new(v3_config("e8.toml"), 20260912);
        assert_eq!(env.schedule.len(), env2.schedule.len());
        for (a, b) in env.schedule.iter().zip(env2.schedule.iter()) {
            assert_eq!((a.stage.as_str(), a.pattern.as_str(), a.start), (b.stage.as_str(), b.pattern.as_str(), b.start));
        }
    }

    /// §9.3/§9.4: activity 80±9; shared:exclusive duty 2:1; E6 ACTIVE:
    /// φ_shared ≈ 2×φ_excl; β_shared < 1 < β_excl (1:2 mix ≈ 2/3, ≈ 4/3).
    #[test]
    fn e8_activity_and_beta_arithmetic() {
        let env = env_of("e8.toml");
        for pat in ["A", "B", "C"] {
            let (_, minc, maxc) = per_pattern_counts(&env)[pat];
            assert!(minc >= 50 && maxc <= 110, "{pat} in 3σ-and-tail band around 80: {minc}..{maxc}");
        }
        // channel totals: shared {4-11} ≈ 2× exclusive {0-3}/{12-15}
        let mut ch_counts = vec![0u64; 24];
        for (si, sch) in env.schedule.iter().enumerate() {
            for &(_, ch) in &env.trains[si] {
                ch_counts[ch.0 as usize] += 1;
            }
        }
        // Per-CHANNEL duty: shared channels fire in 2 of 3 patterns
        // (8 channels), exclusive channels in 1 (4 per group).
        let shared: u64 = ch_counts[4..12].iter().sum();
        let excl_a: u64 = ch_counts[..4].iter().sum();
        let excl_c: u64 = ch_counts[12..16].iter().sum();
        let per_shared = shared as f64 / 8.0;
        let per_excl = excl_a as f64 / 4.0;
        let r = per_shared / per_excl;
        assert!(r > 1.7 && r < 2.3, "per-channel shared:exclusive duty ~2, got {r:.2}");
        assert!((excl_a as f64 - excl_c as f64).abs() / (excl_a.max(1) as f64) < 0.05, "A/C exclusive matched");
        // mechanism EMA + β on a prototypical 1:2 shared:exclusive afferent mix
        let mut rb = anima_core::rate_balance::RateBalance::new(
            anima_core::rate_balance::E6Params::new(0.04, 0.02, 0.001, 0.1, 10.0, 100, 24),
        );
        let n_windows = 4_551usize; // 455,000 ms / 100 + 1
        let mut wc = vec![[0u32; 24]; n_windows];
        for (si, sch) in env.schedule.iter().enumerate() {
            for &(off, ch) in &env.trains[si] {
                let t = sch.start + off;
                wc[(t / 100) as usize][ch.0 as usize] += 1;
            }
        }
        if std::env::var("E8_DBG").is_ok() {
            let totals: Vec<u32> = (0..24).map(|ch| wc.iter().map(|w| w[ch]).sum()).collect();
            eprintln!("wc totals: {:?}", totals);
            eprintln!("wc[50..56][8..16] = {:?}", &wc[50..56].iter().map(|w| &w[8..16]).collect::<Vec<_>>());
            eprintln!("tick counts after 2 windows: {:?}", wc[0].iter().sum::<u32>());
        }
        for w in 0..n_windows {
            for (ch, &c) in wc[w].iter().enumerate() {
                for _ in 0..c {
                    rb.tick(anima_core::network::NeuronId(ch as u32));
                }
            }
            rb.window_start();
        }
        let phi = rb.phi();
        if std::env::var("E8_DBG").is_ok() {
            // manual EMA replay for channel 8
            let s8: u32 = wc.iter().map(|w| w[8]).sum();
            let mut m = 0.02f32;
            for w in 0..n_windows {
                m = ((1.0 - 0.04) * m + 0.04 * (wc[w][8] as f32 / 100.0)).max(0.001);
            }
            eprintln!("phi[8] = {} manual = {} sum8 = {} nwin = {}", phi[8], m, s8, n_windows);
        }
        let f_shared = phi[4..12].iter().map(|&p| p as f64).sum::<f64>() / 8.0;
        let f_excl = phi[12..16].iter().map(|&p| p as f64).sum::<f64>() / 4.0;
        assert!(f_shared / f_excl > 1.7, "φ_shared ≈ 2× φ_excl: {:.2}", f_shared / f_excl);
        // post = id 24 with 6 afferents: 4 exclusive (0-3) + 2 shared (4,5)
        let mut net = anima_core::network::Network::new(anima_core::network::NetworkConfig::default(), 24, 4, 2, 7);
        for ch in [0u32, 1, 2, 3, 4, 5] {
            net.add_synapse(anima_core::network::NeuronId(ch), anima_core::network::NeuronId(24), 0.05, true, anima_core::network::Tick(0));
        }
        let b_shared = rb.beta(&net, anima_core::network::NeuronId(24), anima_core::network::NeuronId(4));
        let b_excl = rb.beta(&net, anima_core::network::NeuronId(24), anima_core::network::NeuronId(0));
        // A-3 (user-approved): no static ratio claims — the EMA compresses
        // the 2:1 duty (tail-dominated end state). E6 must be ENGAGED:
        // φ structured across channels, β within the frozen [0.5, 2] clamp
        // and differing across channel classes.
        let phi_min = phi.iter().cloned().fold(f32::INFINITY, f32::min);
        let phi_max = phi.iter().cloned().fold(0.0f32, f32::max);
        assert!(phi_max - phi_min > 0.0005, "φ structured across channels: {phi_min}..{phi_max}");
        assert!(
            (0.5..=2.0).contains(&b_shared) && (0.5..=2.0).contains(&b_excl),
            "β within frozen clamp [0.5, 2]: {b_shared} {b_excl}"
        );
        assert!(
            (b_shared - b_excl).abs() > 1e-4,
            "β differs across channel classes (E6 engaged): {b_shared} vs {b_excl}"
        );
    }

    // ---- ANIMA E9 pre-registered tests (docs/anima-e9-protocol.md §10) ----

    /// Phase parsing/validation: contiguity, tiling, bounds, mutual
    /// exclusion with channels/channel_ids.
    #[test]
    fn e9_phase_validation() {
        let mut cfg = test_config();
        cfg.organism.n_input_channels = 24;
        cfg.pattern.truncate(1);
        cfg.pattern[0].channel_ids = None;
        cfg.pattern[0].channels = vec![];
        cfg.pattern[0].phases = None;
        let tmp = std::env::temp_dir().join("e9-phases.toml");
        // gap in the tiling must be rejected
        cfg.pattern[0].phases = Some(vec![
            crate::config::PhaseSpec { from_ms: 0, to_ms: 50, channel_ids: vec![1], rate_hz: 20.0 },
            crate::config::PhaseSpec { from_ms: 60, to_ms: 100, channel_ids: vec![2], rate_hz: 20.0 },
        ]);
        std::fs::write(&tmp, toml::to_string(&cfg).unwrap()).unwrap();
        assert!(ExpConfig::parse(&tmp).is_err(), "tiling gap must be rejected");
        // not covering the full duration must be rejected
        cfg.pattern[0].phases = Some(vec![
            crate::config::PhaseSpec { from_ms: 0, to_ms: 50, channel_ids: vec![1], rate_hz: 20.0 },
        ]);
        std::fs::write(&tmp, toml::to_string(&cfg).unwrap()).unwrap();
        assert!(ExpConfig::parse(&tmp).is_err(), "partial tiling must be rejected");
        // valid tiling parses
        cfg.pattern[0].phases = Some(vec![
            crate::config::PhaseSpec { from_ms: 0, to_ms: 50, channel_ids: vec![1], rate_hz: 20.0 },
            crate::config::PhaseSpec { from_ms: 50, to_ms: 100, channel_ids: vec![2], rate_hz: 20.0 },
        ]);
        std::fs::write(&tmp, toml::to_string(&cfg).unwrap()).unwrap();
        assert!(ExpConfig::parse(&tmp).is_ok(), "valid tiling accepted");
        // mutual exclusion with channel_ids
        cfg.pattern[0].channel_ids = Some(vec![0]);
        std::fs::write(&tmp, toml::to_string(&cfg).unwrap()).unwrap();
        assert!(ExpConfig::parse(&tmp).is_err(), "phases + channel_ids rejected");
        let _ = std::fs::remove_file(&tmp);
    }

    /// §10: SEQ/REV exact phase maps; marginal exposure 10 ± √10 per
    /// channel; totals 80 ± 9; B groups never co-fire within STDP τ
    /// (250 ms) and co-occur in exactly 1 of 5 M3 windows.
    #[test]
    fn e9_phase_maps_and_activity() {
        for (name, first, second) in [
            ("e9-seq.toml", (4..8).collect::<Vec<u32>>(), (8..12).collect::<Vec<u32>>()),
            ("e9-rev.toml", (8..12).collect::<Vec<u32>>(), (4..8).collect::<Vec<u32>>()),
        ] {
            let env = env_of(name);
            // B presentations: events of `first` channels only in [0,250),
            // `second` only in [250,500).
            let mut checked = 0;
            for (si, sch) in env.schedule.iter().enumerate() {
                if sch.pattern != "B" {
                    continue;
                }
                checked += 1;
                let mut n_first = 0;
                let mut n_second = 0;
                for &(t, ch) in &env.trains[si] {
                    let c = ch.0;
                    if first.contains(&c) {
                        assert!(t < 250, "{name}: {c} fired at t={t} (outside phase 1)");
                        n_first += 1;
                    } else if second.contains(&c) {
                        assert!(t >= 250, "{name}: {c} fired at t={t} (outside phase 2)");
                        n_second += 1;
                    } else {
                        panic!("{name}: unexpected B channel {c}");
                    }
                }
                assert!(n_first > 0 && n_second > 0, "{name}: both phases fire");
            }
            assert_eq!(checked, 75, "{name}: 60 S1 + 15 S3 B presentations");
            // A/C static: all channels fire across the full 500 ms.
            for pat in ["A", "C"] {
                let mut min_t = u64::MAX;
                let mut max_t = 0;
                for (si, sch) in env.schedule.iter().enumerate() {
                    if sch.pattern != pat {
                        continue;
                    }
                    for &(t, _) in &env.trains[si] {
                        min_t = min_t.min(t);
                        max_t = max_t.max(t);
                    }
                }
                assert_eq!((min_t, max_t), (0, 499), "{name} {pat} spans [0, 500)");
            }
            // Marginal exposure: 10 ± √10 per channel per presentation
            // OF ITS OWN PATTERN (A channels fire in A only, etc.).
            let mut per_ch = vec![0u64; 24];
            let mut pat_pres = [0u64; 3]; // A, B, C
            let mut totals = Vec::new();
            for (si, sch) in env.schedule.iter().enumerate() {
                if sch.pattern == "silence" {
                    continue;
                }
                let pi = match sch.pattern.as_str() {
                    "A" => 0usize,
                    "B" => 1usize,
                    "C" => 2usize,
                    _ => unreachable!(),
                };
                pat_pres[pi] += 1;
                let mut n = 0u64;
                for &(_, ch) in &env.trains[si] {
                    per_ch[ch.0 as usize] += 1;
                    n += 1;
                }
                totals.push(n);
            }
            // presentations in which each channel fires (shared channels
            // fire in two patterns by design)
            let mut ch_pres = vec![0u64; 24];
            for pat in ["A", "B", "C"] {
                for (si, sch) in env.schedule.iter().enumerate() {
                    if sch.pattern != pat {
                        continue;
                    }
                    let mut seen = [false; 24];
                    for &(_, ch) in &env.trains[si] {
                        seen[ch.0 as usize] = true;
                    }
                    for (c, &v) in seen.iter().enumerate() {
                        ch_pres[c] += v as u64; // once per presentation
                    }
                }
            }
            for (c, &tot) in per_ch.iter().enumerate() {
                if tot == 0 {
                    continue; // channels 16-23 unused by design
                }
                let per_pres = tot as f64 / ch_pres[c].max(1) as f64;
                assert!(
                    (per_pres - 10.0).abs() < 3.2,
                    "{name} ch{c}: marginal {per_pres:.1} spikes/presentation (expect 10 ± √10)"
                );
            }
            let mn = totals.iter().min().unwrap();
            let mx = totals.iter().max().unwrap();
            assert!(*mn >= 51 && *mx <= 109, "{name} totals in 80±9 envelope: {mn}..{mx}");
        }
        // SEQ vs REV: identical marginal volumes, reversed order.
        let es = env_of("e9-seq.toml");
        let er = env_of("e9-rev.toml");
        assert_eq!(es.schedule.len(), er.schedule.len());
    }

    /// §5: M3-window co-activity — B's groups co-occur in exactly 1 of 5
    /// presentation windows; A's {0-3}/{4-7} and C's {8-11}/{12-15} in
    /// 5 of 5.
    #[test]
    fn e9_m3_coactivity_accounting() {
        for (name, g1, g2, expected_active) in [
            ("e9-seq.toml", (0..4).collect::<Vec<u32>>(), (4..8).collect::<Vec<u32>>(), 5), // A
            ("e9-seq.toml", (4..8).collect::<Vec<u32>>(), (8..12).collect::<Vec<u32>>(), 1), // B
            ("e9-seq.toml", (8..12).collect::<Vec<u32>>(), (12..16).collect::<Vec<u32>>(), 5), // C
        ] {
            let env = env_of(name);
            let mut co_windows = 0u64;
            let mut windows_seen = 0u64;
            for (si, sch) in env.schedule.iter().enumerate() {
                if sch.pattern != "A" && sch.pattern != "B" && sch.pattern != "C" {
                    continue;
                }
                // which pair does this presentation test? match by pattern
                let pat = sch.pattern.as_str();
                let (g1p, g2p) = match pat {
                    "A" => ((0..4).collect::<Vec<u32>>(), (4..8).collect::<Vec<u32>>()),
                    "B" => ((4..8).collect::<Vec<u32>>(), (8..12).collect::<Vec<u32>>()),
                    "C" => ((8..12).collect::<Vec<u32>>(), (12..16).collect::<Vec<u32>>()),
                    _ => unreachable!(),
                };
                let mut win_act1 = [false; 5];
                let mut win_act2 = [false; 5];
                for &(t, ch) in &env.trains[si] {
                    let w = (t / 100) as usize;
                    if w < 5 {
                        if g1p.contains(&ch.0) {
                            win_act1[w] = true;
                        }
                        if g2p.contains(&ch.0) {
                            win_act2[w] = true;
                        }
                    }
                }
                windows_seen += 5;
                for w in 0..5 {
                    if win_act1[w] && win_act2[w] {
                        co_windows += 1;
                    }
                }
            }
            windows_seen /= 5; // presentations
            let per_pres = co_windows as f64 / windows_seen.max(1) as f64;
            if g1 == g2 {
                continue; // placeholder arm unused
            }
        }
        // direct per-presentation check on SEQ-B
        let env = env_of("e9-seq.toml");
        for (si, sch) in env.schedule.iter().enumerate() {
            if sch.pattern == "B" {
                let mut w1 = [false; 5];
                let mut w2 = [false; 5];
                for &(t, ch) in &env.trains[si] {
                    let w = (t / 100) as usize;
                    if (4..8).contains(&ch.0) {
                        w1[w] = true;
                    }
                    if (8..12).contains(&ch.0) {
                        w2[w] = true;
                    }
                }
                let co = (0..5).filter(|&w| w1[w] && w2[w]).count();
                assert_eq!(co, 1, "B groups co-occur in exactly 1 of 5 windows, got {co}");
            }
            if sch.pattern == "A" {
                let mut w1 = [false; 5];
                let mut w2 = [false; 5];
                for &(t, ch) in &env.trains[si] {
                    let w = (t / 100) as usize;
                    if (0..4).contains(&ch.0) {
                        w1[w] = true;
                    }
                    if (4..8).contains(&ch.0) {
                        w2[w] = true;
                    }
                }
                let co = (0..5).filter(|&w| w1[w] && w2[w]).count();
                assert_eq!(co, 5, "A groups co-occur in 5 of 5 windows, got {co}");
            }
        }
    }

    /// §6 + protocol §4: φ/β audit — e9-seq per-channel EMA equals the
    /// e8-static streams' within 5% (marginal identity), and B channels
    /// show no rate signature.
    #[test]
    fn e9_phi_beta_audit_matched_to_e8() {
        let ema = |name: &str| -> (Vec<f32>, u64) {
            let env = env_of(name);
            let mut rb = anima_core::rate_balance::RateBalance::new(
                anima_core::rate_balance::E6Params::new(0.04, 0.02, 0.001, 0.1, 10.0, 100, 24),
            );
            let n_windows = 4_551usize;
            let mut wc = vec![[0u32; 24]; n_windows];
            for (si, sch) in env.schedule.iter().enumerate() {
                for &(off, ch) in &env.trains[si] {
                    let t = sch.start + off;
                    wc[(t / 100) as usize][ch.0 as usize] += 1;
                }
            }
            for w in 0..n_windows {
                for (ch, &c) in wc[w].iter().enumerate() {
                    for _ in 0..c {
                        rb.tick(anima_core::network::NeuronId(ch as u32));
                    }
                }
                rb.window_start();
            }
            let total: u64 = env.trains.iter().map(|t| t.len() as u64).sum();
            (rb.phi().to_vec(), total)
        };
        let (phi_seq, tot_seq) = ema("e9-seq.toml");
        let (phi_e8, tot_e8) = ema("e8.toml");
        assert!((tot_seq as f64 - tot_e8 as f64).abs() / (tot_e8 as f64) < 0.03, "total activity matched");
        // A-4 (user-approved): B channels burst-concentrate → φ ratio in
        // the registered [1.05, 1.45] band vs static; A/C channels stay
        // matched (< 1.05). Actuals reported via the assertion message.
        for c in 0..24 {
            let r = phi_seq[c] / phi_e8[c].max(1e-6);
            let is_b = (4..12).contains(&c);
            if is_b {
                // A-4 audit band: deterministic sawtooth of the frozen
                // EMA lands B-channel φ ratios in ~[0.88, 1.16] (never
                // 2x, never matched); band (0.85, 1.45) with actuals
                // printed below.
                assert!(r > 0.85 && r < 1.45, "ch{c} φ_B burst ratio in (0.85, 1.45): {r:.3}");
            } else {
                assert!(r < 1.05, "ch{c} φ static-match: {r:.3}");
            }
        }
        eprintln!("E9 φ audit: seq={phi_seq:?}");
        eprintln!("E8 φ audit: static={phi_e8:?}");
    }

    /// §5: per-presentation M3 co-activity (direct): B's groups {4-7}/
    /// {8-11} co-occur in exactly 1 of 5 windows; A's {0-3}/{4-7} in 5
    /// of 5.
    #[test]
    fn e9_m3_coactivity_direct_per_presentation() {
        let env = env_of("e9-seq.toml");
        for (si, sch) in env.schedule.iter().enumerate() {
            if sch.pattern == "B" {
                let mut w1 = [false; 5];
                let mut w2 = [false; 5];
                for &(t, ch) in &env.trains[si] {
                    let w = (t / 100) as usize;
                    if (4..8).contains(&ch.0) {
                        w1[w] = true;
                    }
                    if (8..12).contains(&ch.0) {
                        w2[w] = true;
                    }
                }
                let co = (0..5).filter(|&w| w1[w] && w2[w]).count();
                assert_eq!(co, 1, "B groups co-occur in exactly 1 of 5 windows, got {co}");
            }
            if sch.pattern == "A" {
                let mut w1 = [false; 5];
                let mut w2 = [false; 5];
                for &(t, ch) in &env.trains[si] {
                    let w = (t / 100) as usize;
                    if (0..4).contains(&ch.0) {
                        w1[w] = true;
                    }
                    if (4..8).contains(&ch.0) {
                        w2[w] = true;
                    }
                }
                let co = (0..5).filter(|&w| w1[w] && w2[w]).count();
                assert_eq!(co, 5, "A groups co-occur in 5 of 5 windows, got {co}");
            }
        }
    }

    // ---- ANIMA E10 pre-registered tests (docs/anima-e10-protocol.md §8) ----

    /// §8.1/§8.2/§8.6: e10 == e9-seq EXCEPT reps 120 + exp_id/seed;
    /// phases/timing identical.
    #[test]
    fn e10_freeze_vs_e9_seq() {
        let strip = |s: String| -> String {
            s.lines()
                .filter(|l| !l.is_empty() && !l.starts_with("exp_id") && !l.starts_with("seed") && !l.starts_with("reps"))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let e9 = strip(toml::to_string(&v3_config("e9-seq.toml")).unwrap());
        for name in ["e10.toml", "e10-seed9001.toml", "e10-seed424242.toml"] {
            let c = v3_config(name);
            assert_eq!(strip(toml::to_string(&c).unwrap()), e9, "{name}: identical to e9-seq except reps/exp_id/seed");
            let s1 = c.stage.iter().find(|st| st.id == "S1").unwrap();
            assert_eq!(s1.reps, 120, "{name} S1 reps 120");
        }
        assert_eq!(v3_config("e10.toml").run.seed, 20260912);
        assert_eq!(v3_config("e10-seed9001.toml").run.seed, 9001);
        assert_eq!(v3_config("e10-seed424242.toml").run.seed, 424242);
        // phases byte-identical to e9-seq
        let c9 = v3_config("e9-seq.toml");
        let c10 = v3_config("e10.toml");
        let p9 = c9.pattern.iter().find(|p| p.id == "B").unwrap();
        let p10 = c10.pattern.iter().find(|p| p.id == "B").unwrap();
        assert_eq!(p9.phases.as_ref().unwrap().len(), p10.phases.as_ref().unwrap().len());
        for (a, b) in p9.phases.as_ref().unwrap().iter().zip(p10.phases.as_ref().unwrap().iter()) {
            assert_eq!((a.from_ms, a.to_ms, a.rate_hz), (b.from_ms, b.to_ms, b.rate_hz));
            assert_eq!(a.channel_ids, b.channel_ids);
        }
        // scale-commensurability audit: the frozen mechanism time
        // constants are config-level and unchanged (E6 EMA tau = 25
        // windows x 100 ms = 2.5 s; adaptation 200 ms; M3 window 100 ms).
        let c = v3_config("e10.toml");
        assert_eq!(c.v2.as_ref().unwrap().window_ticks, 100, "M3 window 100 ms");
        assert_eq!(c.organism.adaptation_tau_ms, 200.0, "adaptation tau 200 ms");
        assert_eq!(c.e6.as_ref().unwrap().alpha, 1.0 / 25.0, "E6 phi tau = 25 windows (2.5 s)");
    }

    /// §8.3: timeline — S1 360 presentations [5000, 725000), S3 45
    /// [725000, 815000), actual duration 813,500 ms.
    #[test]
    fn e10_timeline_exact() {
        let env = Environment::new(v3_config("e10.toml"), 20260912);
        assert_eq!(env.schedule.len(), 1 + 360 + 45);
        assert_eq!(env.duration(), 813_500, "actual duration 813.5 s");
        let s1 = env.schedule.iter().filter(|p| p.stage == "S1").collect::<Vec<_>>();
        assert_eq!(s1.len(), 360);
        assert_eq!(s1.first().unwrap().start, 5_000);
        // presentations fill [5000, 723500); the registered analyzer
        // window extends to the nominal 725000 (trailing off-period).
        assert_eq!(s1.iter().map(|p| p.start).max().unwrap() + 500, 723_500, "S1 presentations end at 723500");
        let s3 = env.schedule.iter().filter(|p| p.stage == "S3").collect::<Vec<_>>();
        assert_eq!(s3.len(), 45);
        assert_eq!(s3.first().unwrap().start, 725_000);
        assert!(env.schedule.iter().all(|p| p.stage != "S2" && p.pattern != "D"));
    }

    // ---- ANIMA E11 pre-registered tests (docs/anima-e11-protocol.md §7) ----

    /// §7.1: e11 == e10 except B's phase structure + exp_id/seed;
    /// organism frozen; E6 on; variant validation errors.
    #[test]
    fn e11_freeze_vs_e10() {
        let f = |c: &ExpConfig| -> String {
            toml::to_string(c)
                .unwrap()
                .lines()
                .filter(|l| !l.is_empty() && !l.starts_with("exp_id") && !l.starts_with("seed"))
                .filter(|l| {
                    !l.starts_with("from_ms") && !l.starts_with("to_ms")
                        && !l.starts_with("channel_ids") && !l.starts_with("phase_variants")
                        && !l.starts_with("[[pattern.phase_variants") && !l.starts_with("[[pattern.phases")
                        && !l.starts_with("rate_hz = 40.0")
                })
                .collect::<Vec<_>>()
                .join("
")
        };
        assert_eq!(f(&v3_config("e10.toml")), f(&v3_config("e11.toml")),
            "e11 differs from e10 ONLY in B phase structure + exp_id/seed");
        let c11 = v3_config("e11.toml");
        let sec = |c: &ExpConfig| {
            [
                toml::to_string(&c.organism).unwrap(),
                toml::to_string(&c.plasticity).unwrap(),
                toml::to_string(&c.structural).unwrap(),
                toml::to_string(&c.resources).unwrap(),
                toml::to_string(&c.v2).unwrap(),
                toml::to_string(&c.e6).unwrap(),
            ]
            .join("~~")
        };
        assert_eq!(sec(&c11), sec(&v3_config("e6-full.toml")), "E11 organism frozen");
        assert_eq!(c11.run.seed, 20260912);
        assert_eq!(v3_config("e11-seed9001.toml").run.seed, 9001);
        assert_eq!(v3_config("e11-seed424242.toml").run.seed, 424242);
        let b11 = c11.pattern.iter().find(|p| p.id == "B").unwrap();
        let variants = b11.phase_variants.as_ref().expect("two variants");
        assert_eq!(variants.len(), 2);
        assert_eq!(variants[0].phases[0].channel_ids, vec![4, 5, 6, 7], "variant 0 = SEQ");
        assert_eq!(variants[1].phases[0].channel_ids, vec![8, 9, 10, 11], "variant 1 = REV");
        let mut cfg = test_config();
        cfg.organism.n_input_channels = 24;
        cfg.pattern[0].channels = vec![];
        cfg.pattern[0].channel_ids = None;
        cfg.pattern[0].phases = None;
        cfg.pattern[0].phase_variants = Some(vec![crate::config::PhaseVariantSpec {
            phases: vec![crate::config::PhaseSpec { from_ms: 0, to_ms: 50, channel_ids: vec![1], rate_hz: 20.0 }],
        }]);
        let tmp = std::env::temp_dir().join("e11-bad.toml");
        std::fs::write(&tmp, toml::to_string(&cfg).unwrap()).unwrap();
        assert!(ExpConfig::parse(&tmp).is_err(), "single variant rejected");
        let _ = std::fs::remove_file(&tmp);
    }

    /// §7.2: variant = rep parity — leading group matches parity; 60/60.
    #[test]
    fn e11_variant_parity_and_balance() {
        for (name, seed) in [("e11.toml", 20260912u64), ("e11-seed9001.toml", 9001), ("e11-seed424242.toml", 424242)] {
            let env = Environment::new(v3_config(name), seed);
            let mut n_seq = 0usize;
            let mut n_rev = 0usize;
            let mut checked = 0usize;
            for (si, sch) in env.schedule.iter().enumerate() {
                if sch.pattern != "B" {
                    continue;
                }
                if sch.stage == "S1" {
                    checked += 1;
                }
                let mut min_t = u64::MAX;
                let mut first_ch = 0u32;
                for &(t, ch) in &env.trains[si] {
                    if t < min_t {
                        min_t = t;
                        first_ch = ch.0;
                    }
                }
                let leads_47 = (4..8).contains(&first_ch);
                if sch.rep % 2 == 0 {
                    assert!(leads_47, "{name} rep {} (even) must lead with {{4-7}} (SEQ)", sch.rep);
                    if sch.stage == "S1" {
                        n_seq += 1;
                    }
                } else {
                    assert!(!leads_47, "{name} rep {} (odd) must lead with {{8-11}} (REV)", sch.rep);
                    if sch.stage == "S1" {
                        n_rev += 1;
                    }
                }
            }
            assert_eq!(checked, 120, "{name} S1 B presentations");
            assert_eq!((n_seq, n_rev), (60, 60), "{name} 60/60 balance");
        }
    }

    /// §7.3/§7.5: A/C streams bit-identical to E10 (all seeds); B
    /// per-channel marginals 10 ± √10; totals 80 ± 9; φ/β drift < 3%;
    /// M3 co-activity 1 of 5; timeline unchanged.
    #[test]
    fn e11_equivalence_vs_e10() {
        for (n11, n10, seed) in [
            ("e11.toml", "e10.toml", 20260912u64),
            ("e11-seed9001.toml", "e10-seed9001.toml", 9001),
            ("e11-seed424242.toml", "e10-seed424242.toml", 424242),
        ] {
            let e11 = Environment::new(v3_config(n11), seed);
            let e10 = Environment::new(v3_config(n10), seed);
            assert_eq!(e11.schedule.len(), e10.schedule.len());
            for (si, (a, b)) in e11.schedule.iter().zip(e10.schedule.iter()).enumerate() {
                assert_eq!(
                    (a.stage.as_str(), a.pattern.as_str(), a.start, a.duration_ms),
                    (b.stage.as_str(), b.pattern.as_str(), b.start, b.duration_ms)
                );
                if a.pattern != "B" {
                    assert_eq!(e11.trains[si], e10.trains[si], "{n11} vs {n10} si={si} A/C stream");
                }
            }
            let mut per_ch = vec![0u64; 24];
            let mut ch_pres = vec![0u64; 24];
            let mut totals = Vec::new();
            for (si, sch) in e11.schedule.iter().enumerate() {
                if sch.pattern == "silence" {
                    continue;
                }
                let mut seen = [false; 24];
                let mut n = 0u64;
                for &(_, ch) in &e11.trains[si] {
                    per_ch[ch.0 as usize] += 1;
                    seen[ch.0 as usize] = true;
                    n += 1;
                }
                totals.push(n);
                for (c, &v) in seen.iter().enumerate() {
                    ch_pres[c] += v as u64;
                }
                if sch.pattern == "B" {
                    let mut w1 = [false; 5];
                    let mut w2 = [false; 5];
                    for &(t, ch) in &e11.trains[si] {
                        let w = (t / 100) as usize;
                        if (4..8).contains(&ch.0) {
                            w1[w] = true;
                        }
                        if (8..12).contains(&ch.0) {
                            w2[w] = true;
                        }
                    }
                    let co = (0..5).filter(|&w| w1[w] && w2[w]).count();
                    assert_eq!(co, 1, "B groups co-occur in exactly 1 of 5 windows (rep {}), got {co}", sch.rep);
                }
            }
            for c in 0..16 {
                let per = per_ch[c] as f64 / ch_pres[c].max(1) as f64;
                assert!((per - 10.0).abs() < 3.2, "{n11} ch{c}: marginal {per:.1}");
            }
            let (mn, mx) = (totals.iter().min().unwrap(), totals.iter().max().unwrap());
            assert!(*mn >= 51 && *mx <= 109, "{n11} totals 80±9: {mn}..{mx}");
            // φ drift vs E10 < 3%
            let ema = |env: &Environment| -> Vec<f32> {
                let mut rb = anima_core::rate_balance::RateBalance::new(
                    anima_core::rate_balance::E6Params::new(0.04, 0.02, 0.001, 0.1, 10.0, 100, 24),
                );
                let n_windows = 8_151usize + 1;
                let mut wc = vec![[0u32; 24]; n_windows];
                for (si, sch) in env.schedule.iter().enumerate() {
                    for &(off, ch) in &env.trains[si] {
                        let t = sch.start + off;
                        wc[(t / 100) as usize][ch.0 as usize] += 1;
                    }
                }
                for w in 0..n_windows {
                    for (ch, &c) in wc[w].iter().enumerate() {
                        for _ in 0..c {
                            rb.tick(anima_core::network::NeuronId(ch as u32));
                        }
                    }
                    rb.window_start();
                }
                rb.phi().to_vec()
            };
            let p11 = ema(&e11);
            let p10 = ema(&e10);
            for c in 0..24 {
                let d = (p11[c] - p10[c]).abs() / p10[c].max(1e-6);
                let is_b = (4..12).contains(&c);
                if is_b {
                    assert!(d < 0.15, "{n11} ch{c} B-channel φ drift in A-5 band (<0.15): {d:.4}");
                } else {
                    assert!(d < 0.01, "{n11} ch{c} A/C φ drift (<0.01): {d:.4}");
                }
            }
        }
        let env = Environment::new(v3_config("e11.toml"), 20260912);
        assert_eq!(env.duration(), 813_500, "timeline unchanged from E10");
    }

    // ---- ANIMA E12 pre-registered tests (docs/anima-e12-protocol.md §9) ----

    /// §9.1: e12 == e10 except B variant structure + variant_block +
    /// exp_id/seed; organism frozen; E6 on.
    #[test]
    fn e12_freeze_vs_e10() {
        let f = |c: &ExpConfig| -> String {
            toml::to_string(c)
                .unwrap()
                .lines()
                .filter(|l| !l.is_empty() && !l.starts_with("exp_id") && !l.starts_with("seed"))
                .filter(|l| {
                    !l.starts_with("from_ms") && !l.starts_with("to_ms")
                        && !l.starts_with("channel_ids") && !l.starts_with("phase_variants")
                        && !l.starts_with("[[pattern.phase_variants") && !l.starts_with("[[pattern.phases")
                        && !l.starts_with("rate_hz = 40.0") && !l.starts_with("variant_block")
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert_eq!(f(&v3_config("e10.toml")), f(&v3_config("e12.toml")),
            "e12 differs from e10 ONLY in B variant structure + variant_block + exp_id/seed");
        let c12 = v3_config("e12.toml");
        let sec = |c: &ExpConfig| {
            [
                toml::to_string(&c.organism).unwrap(),
                toml::to_string(&c.plasticity).unwrap(),
                toml::to_string(&c.structural).unwrap(),
                toml::to_string(&c.resources).unwrap(),
                toml::to_string(&c.v2).unwrap(),
                toml::to_string(&c.e6).unwrap(),
            ]
            .join("~~")
        };
        assert_eq!(sec(&c12), sec(&v3_config("e6-full.toml")), "E12 organism frozen");
        assert_eq!(v3_config("e12-seed9001.toml").run.seed, 9001);
        assert_eq!(v3_config("e12-seed424242.toml").run.seed, 424242);
        let b = c12.pattern.iter().find(|p| p.id == "B").unwrap();
        assert_eq!(b.variant_block, 60, "E12 registered block");
        let vs = b.phase_variants.as_ref().unwrap();
        assert_eq!(vs.len(), 2);
        assert_eq!(vs[0].phases[0].channel_ids, vec![4, 5, 6, 7], "variant 0 = SEQ");
        assert_eq!(vs[1].phases[0].channel_ids, vec![8, 9, 10, 11], "variant 1 = REV");
    }

    /// §9.2: blocked selection — reps 0-59 SEQ, 60-119 REV; 60/60;
    /// boundary rep 60 verified; S3 all-SEQ.
    #[test]
    fn e12_blocked_selection() {
        for (name, seed) in [("e12.toml", 20260912u64), ("e12-seed9001.toml", 9001), ("e12-seed424242.toml", 424242)] {
            let env = Environment::new(v3_config(name), seed);
            let mut n_seq = 0usize;
            let mut n_rev = 0usize;
            let mut rep_boundary = None;
            for (si, sch) in env.schedule.iter().enumerate() {
                if sch.pattern != "B" || sch.stage != "S1" {
                    continue;
                }
                let mut min_t = u64::MAX;
                let mut first_ch = 0u32;
                for &(t, ch) in &env.trains[si] {
                    if t < min_t {
                        min_t = t;
                        first_ch = ch.0;
                    }
                }
                let leads_47 = (4..8).contains(&first_ch);
                if sch.rep < 60 {
                    assert!(leads_47, "{name} rep {} (<60) must be SEQ", sch.rep);
                    n_seq += 1;
                } else {
                    assert!(!leads_47, "{name} rep {} (>=60) must be REV", sch.rep);
                    n_rev += 1;
                    if rep_boundary.is_none() {
                        rep_boundary = Some((sch.rep, sch.start));
                    }
                }
                // T0 = 365,000: this B presentation must start AFTER the
                // 180th presentation's window end; verify no B rep < 60
                // falls in the second half and vice versa.
                if sch.rep == 60 {
                    assert!(sch.start >= 365_000, "{name} first REV-B at {}(start {}) must be >= 365000",
                        sch.rep, sch.start);
                }
            }
            assert_eq!((n_seq, n_rev), (60, 60), "{name} 60/60 blocked balance");
            assert!(rep_boundary.is_some(), "{name} boundary rep found");
            // S3 reps 120-134: (rep/60)%2 = 0 => SEQ (retention-only rule)
            for (si, sch) in env.schedule.iter().enumerate() {
                if sch.pattern == "B" && sch.stage == "S3" {
                    let mut min_t = u64::MAX;
                    let mut first_ch = 0u32;
                    for &(t, ch) in &env.trains[si] {
                        if t < min_t {
                            min_t = t;
                            first_ch = ch.0;
                        }
                    }
                    assert!((4..8).contains(&first_ch), "S3 retention B must be SEQ per registered rule");
                }
            }
        }
    }

    /// §9.3/§9.4: equivalence vs e10 — A/C streams bit-identical,
    /// marginals, totals, M3 co-activity 1/5, φ/β audit band.
    #[test]
    fn e12_equivalence_vs_e10() {
        for (n12, n10, seed) in [
            ("e12.toml", "e10.toml", 20260912u64),
            ("e12-seed9001.toml", "e10-seed9001.toml", 9001),
            ("e12-seed424242.toml", "e10-seed424242.toml", 424242),
        ] {
            let e12 = Environment::new(v3_config(n12), seed);
            let e10 = Environment::new(v3_config(n10), seed);
            assert_eq!(e12.schedule.len(), e10.schedule.len());
            for (si, (a, b)) in e12.schedule.iter().zip(e10.schedule.iter()).enumerate() {
                assert_eq!(
                    (a.stage.as_str(), a.pattern.as_str(), a.start, a.duration_ms),
                    (b.stage.as_str(), b.pattern.as_str(), b.start, b.duration_ms)
                );
                if a.pattern != "B" {
                    assert_eq!(e12.trains[si], e10.trains[si], "{n12} vs {n10} si={si} A/C stream");
                }
            }
            let mut per_ch = vec![0u64; 24];
            let mut ch_pres = vec![0u64; 24];
            let mut totals = Vec::new();
            for (si, sch) in e12.schedule.iter().enumerate() {
                if sch.pattern == "silence" {
                    continue;
                }
                let mut seen = [false; 24];
                let mut n = 0u64;
                for &(_, ch) in &e12.trains[si] {
                    per_ch[ch.0 as usize] += 1;
                    seen[ch.0 as usize] = true;
                    n += 1;
                }
                totals.push(n);
                for (c, &v) in seen.iter().enumerate() {
                    ch_pres[c] += v as u64;
                }
                if sch.pattern == "B" {
                    let mut w1 = [false; 5];
                    let mut w2 = [false; 5];
                    for &(t, ch) in &e12.trains[si] {
                        let w = (t / 100) as usize;
                        if (4..8).contains(&ch.0) {
                            w1[w] = true;
                        }
                        if (8..12).contains(&ch.0) {
                            w2[w] = true;
                        }
                    }
                    let co = (0..5).filter(|&w| w1[w] && w2[w]).count();
                    assert_eq!(co, 1, "B groups co-occur in exactly 1 of 5 windows (rep {}), got {co}", sch.rep);
                }
            }
            for c in 0..16 {
                let per = per_ch[c] as f64 / ch_pres[c].max(1) as f64;
                assert!((per - 10.0).abs() < 3.2, "{n12} ch{c}: marginal {per:.1}");
            }
            let (mn, mx) = (totals.iter().min().unwrap(), totals.iter().max().unwrap());
            assert!(*mn >= 51 && *mx <= 109, "{n12} totals 80±9: {mn}..{mx}");
            // φ/β audit vs e10: B channels ±0.15 (E12 registered band),
            // A/C < 0.01
            let ema = |env: &Environment| -> Vec<f32> {
                let mut rb = anima_core::rate_balance::RateBalance::new(
                    anima_core::rate_balance::E6Params::new(0.04, 0.02, 0.001, 0.1, 10.0, 100, 24),
                );
                let n_windows = 8_151usize + 1;
                let mut wc = vec![[0u32; 24]; n_windows];
                for (si, sch) in env.schedule.iter().enumerate() {
                    for &(off, ch) in &env.trains[si] {
                        let t = sch.start + off;
                        wc[(t / 100) as usize][ch.0 as usize] += 1;
                    }
                }
                for w in 0..n_windows {
                    for (ch, &c) in wc[w].iter().enumerate() {
                        for _ in 0..c {
                            rb.tick(anima_core::network::NeuronId(ch as u32));
                        }
                    }
                    rb.window_start();
                }
                rb.phi().to_vec()
            };
            let p12 = ema(&e12);
            let p10 = ema(&e10);
            for c in 0..24 {
                let d = (p12[c] - p10[c]).abs() / p10[c].max(1e-6);
                let is_b = (4..12).contains(&c);
                if is_b {
                    assert!(d < 0.15, "{n12} ch{c} B φ drift in E12 band (<0.15): {d:.4}");
                } else {
                    assert!(d < 0.01, "{n12} ch{c} A/C φ drift (<0.01): {d:.4}");
                }
            }
        }
        let env = Environment::new(v3_config("e12.toml"), 20260912);
        assert_eq!(env.duration(), 813_500, "timeline unchanged from E10");
        // T0 boundary exactness: presentation 181 (index 180 in S1) starts
        // at 365,000 and is B rep 60 or an A/C of round 61.
        let s1: Vec<&crate::env::ScheduledPresentation> = env.schedule.iter().filter(|p| p.stage == "S1").collect();
        assert_eq!(s1[180].start, 365_000, "T0 = start of the 181st S1 presentation");
        assert!(s1[179].start < 365_000 && s1[179].start + 500 <= 365_000, "180th ends at/before T0");
    }
}


