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
                PatternSpec { id: "A".into(), channels: vec!["A".into()], channel_ids: None, rate_hz: 20.0, duration_ms: 100, jitter_ms: 2.0 },
                PatternSpec { id: "D".into(), channels: vec!["A".into(), "B".into()], channel_ids: None, rate_hz: 20.0, duration_ms: 100, jitter_ms: 2.0 },
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
}
