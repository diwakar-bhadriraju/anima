//! E19 closed-loop world (docs/anima-e19-protocol.md, frozen).
//!
//! WORLD-LAYER ONLY: no organism contact. The world observes the
//! network's output spikes during the action window and injects a
//! consequence epoch (silence or disruption) — nothing else. The
//! organism receives ordinary channel spikes; no labels, no reward,
//! no teacher, no world-state channel.
//!
//! Trial (3000 ms; delivery-tick +1 convention applies at the
//! harness, world works in scheduled time):
//!   0-500     antecedent (A or C — from the env trial-block stage)
//!   500-1300  gap (silence)
//!   1300-1800 probe B
//!   1800-2300 action window (silence; world accumulates the vote)
//!   2300-2800 consequence (silence on match; else disruption on
//!             channels 16-23 @ 80 Hz)
//!   2800-3000 ITI (silence)
//!
//! Vote: group 1 = output neurons 64-69, group 2 = 70-75; majority
//! of accumulated output spikes in the action window; tie or total
//! quiescence = no-action. A-context correct = group 1; C-context
//! correct = group 2 (registered mapping).
//!
//! Disruption trains: pre-generated per trial from the run-seeded
//! RNG in trial order (registered draw order: after the env's
//! antecedent shuffle draws for its stage), independent of actions.
//! Deterministic: same seed -> same trains.

use std::collections::BTreeMap;

pub const G1: std::ops::Range<u32> = 64..70;
pub const G2: std::ops::Range<u32> = 70..76;
pub const TRIAL_MS: u64 = 3000;
pub const CONSEQ_START: u64 = 2300;
pub const CONSEQ_END: u64 = 2800;
pub const DISRUPT_RATE_HZ: f64 = 80.0;
pub const DISRUPT_CHANNELS: [u32; 8] = [16, 17, 18, 19, 20, 21, 22, 23];

#[derive(Debug, Clone)]
pub struct Trial {
    pub antecedent_is_a: bool,
    pub base: u64,
    /// Pre-generated disruption spikes (offset_ms, channel), 500 ms.
    pub disruption: Vec<(u64, u32)>,
    // Vote accumulation (public for tests).
    pub g1: u32,
    pub g2: u32,
    /// None until the consequence decision point (action-window end).
    pub decision: Option<Decision>,
    pub log: Vec<String>,
    /// E20 (D1 interface): vote window = [1800, 1950) — the post-probe
    /// echo. E19 semantics: [1800, 2300). Registered per experiment.
    pub vote_end: u64,
    /// E23 reflex mode: vote = [vote_start, vote_end) of the stimulus
    /// epoch (registered [100,500)); consequence = [500,1000).
    pub vote_start: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    Match,
    Mismatch,
    NoAction,
}

impl Trial {
    /// Feed one output spike (neuron id, scheduled ms) to the vote.
    /// E19/E20: [1800, vote_end). E23 reflex: [vote_start, vote_end).
    pub fn observe_output(&mut self, n: u32, t: u64) {
        if t >= self.base + self.vote_start && t < self.base + self.vote_end {
            if G1.contains(&n) {
                self.g1 += 1;
            } else if G2.contains(&n) {
                self.g2 += 1;
            }
        }
    }

    /// Close the vote at the action-window end; compute the decision.
    pub fn close_vote(&mut self) {
        if self.decision.is_some() {
            return;
        }
        let correct_g1 = self.antecedent_is_a;
        let d = if self.g1 == 0 && self.g2 == 0 {
            Decision::NoAction
        } else if self.g1 == self.g2 {
            Decision::NoAction
        } else {
            let voted_g1 = self.g1 > self.g2;
            if voted_g1 == correct_g1 {
                Decision::Match
            } else {
                Decision::Mismatch
            }
        };
        self.decision = Some(d.clone());
        self.log.push(format!(
            "vote g1={} g2={} -> {:?}",
            self.g1, self.g2, d
        ));
    }

    /// Benign = silence; disruption is delivered for Mismatch and
    /// NoAction (registered).
    pub fn disruption_due(&self) -> bool {
        matches!(self.decision, Some(Decision::Mismatch) | Some(Decision::NoAction))
    }

    /// World input spikes for the scheduled ms `t` (absolute).
    pub fn input_at(&self, t: u64) -> Vec<u32> {
        let (cs, ce) = if self.vote_start < 1000 { (500u64, 1000u64) } else { (CONSEQ_START, CONSEQ_END) };
        if t >= self.base + cs && t < self.base + ce && self.disruption_due() {
            let mut v: Vec<u32> = self
                .disruption
                .iter()
                .filter(|(o, _)| self.base + cs + o == t)
                .map(|(_, c)| *c)
                .collect();
            v.sort_unstable();
            v.dedup();
            v
        } else {
            Vec::new()
        }
    }
}

/// The closed-loop world for one E19 run.
pub struct World {
    pub trials: Vec<Trial>,
    /// Open-loop mode: consequence epoch always silence (control arm).
    pub open_loop: bool,
    cur: usize,
    benign: u64,
    per_decision: BTreeMap<String, u64>,
}

impl World {
    pub fn new(antecedents: &[(bool, u64)], seed: u64, open_loop: bool, vote_end: u64) -> Self {
        Self::with_vote_start(antecedents, seed, open_loop, vote_end, 1800)
    }

    /// Draw order (registered): Xoshiro from derive_seed(seed,
    /// [hash("e19-disruption")]) is consumed in trial order, 8
    /// channels x 40 draws each (Poisson 80 Hz / 500 ms = 40
    /// expected), sorted; independent of the antecedent draws.
    pub fn with_vote_start(antecedents: &[(bool, u64)], seed: u64, open_loop: bool, vote_end: u64, vote_start: u64) -> Self {
        use rand::Rng;
        use rand::SeedableRng;
        let mut rng =
            rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(crate::env::derive_seed(
                seed,
                &[crate::env::hash_str("e19-disruption")],
            ));
        let trials = antecedents
            .iter()
            .map(|&(is_a, base)| {
                let mut spikes = Vec::new();
                for &ch in DISRUPT_CHANNELS.iter() {
                    let n = rng.gen::<f64>();
                    // Poisson(40) via a fixed small-knuth? Simpler and
                    // still deterministic: exponential inter-arrival
                    // sampling at 80 Hz over 500 ms.
                    let mut t = 0.0f64;
                    loop {
                        let u: f64 = rng.gen();
                        t -= (u.max(1e-12)).ln() / DISRUPT_RATE_HZ * 1000.0;
                        if t >= 500.0 {
                            break;
                        }
                        spikes.push((t as u64, ch));
                    }
                    let _ = n;
                }
                spikes.sort_unstable();
                Trial {
                    antecedent_is_a: is_a,
                    base,
                    disruption: spikes,
                    g1: 0,
                    g2: 0,
                    decision: None,
                    log: Vec::new(),
                    vote_end,
                    vote_start,
                }
            })
            .collect();
        World {
            trials,
            open_loop,
            cur: 0,
            benign: 0,
            per_decision: BTreeMap::new(),
        }
    }

    /// Per-tick world hooks. `out_spikes` = this tick's output-neuron
    /// spikes (scheduled ms). Returns the world-injected channel ids
    /// for this tick.
    pub fn step(&mut self, t: u64, out_spikes: &[u32]) -> Vec<u32> {
        // Advance/close trials whose action window ended.
        for tr in self.trials.iter_mut() {
            if t == tr.base + tr.vote_end {
                tr.close_vote();
                let d = format!("{:?}", tr.decision.clone().unwrap_or(Decision::NoAction));
                *self.per_decision.entry(d).or_insert(0) += 1;
                if tr.decision == Some(Decision::Match) {
                    self.benign += 1;
                }
            }
        }
        // Observe output spikes (into the CURRENT trial only).
        let mut inj = Vec::new();
        for (i, tr) in self.trials.iter_mut().enumerate() {
            if t >= tr.base && t < tr.base + (if tr.vote_start < 1000 { 2000 } else { TRIAL_MS }) {
                self.cur = i;
                for &n in out_spikes {
                    tr.observe_output(n, t);
                }
                if !self.open_loop {
                    inj = tr.input_at(t);
                }
            }
        }
        inj
    }

    pub fn summary(&self) -> String {
        let total = self.trials.len();
        let d: Vec<String> = self
            .per_decision
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect();
        format!(
            "e19 world: trials={total} benign={} decisions[{}]",
            self.benign,
            d.join(" ")
        )
    }

    /// Per-trial log lines (analysis-side only).
    pub fn trial_lines(&self) -> Vec<String> {
        self.trials
            .iter()
            .enumerate()
            .map(|(i, t)| {
                format!(
                    "trial={} ante={} base={} {}",
                    i + 1,
                    if t.antecedent_is_a { "A" } else { "C" },
                    t.base,
                    t.log.first().cloned().unwrap_or_default()
                )
            })
            .collect()
    }
}