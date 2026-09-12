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
                dormancy_rate_hz: 0.1,
                dormancy_ms: 30000,
                recovery_rate_hz: 1.0,
                retirement_ms: 300000,
                wiring_synapses: 5,
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
                PatternSpec { id: "A".into(), channels: vec!["A".into()], rate_hz: 20.0, duration_ms: 100, jitter_ms: 2.0 },
                PatternSpec { id: "D".into(), channels: vec!["A".into(), "B".into()], rate_hz: 20.0, duration_ms: 100, jitter_ms: 2.0 },
            ],
            stage: vec![
                StageSpec { id: "S0".into(), present: vec![], reps: 0, order: "interleaved".into(), off_ms: 0, silence_ms: Some(200) },
                StageSpec { id: "S1".into(), present: vec!["A".into()], reps: 3, order: "interleaved".into(), off_ms: 100, silence_ms: None },
                StageSpec { id: "S2".into(), present: vec!["D".into()], reps: 1, order: "blocked".into(), off_ms: 100, silence_ms: None },
            ],
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
}
