//! Resource economy: caps, metabolic ledger, failure detectors.
//! A failed run is data — the harness never auto-restarts (§22).

use serde::{Deserialize, Serialize};

use crate::network::{NeuronClass, Network, Tick};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub max_neurons: usize,
    pub max_synapses: usize,
    /// D-49: when Some(k), the synapse budget is SIZE-SCALED k*live_neurons
    /// (recomputed per check) instead of the fixed max_synapses. Removes
    /// the hard budget as a confound to measure the representational slope
    /// above ~320 neurons. None = fixed-cap path (identity).
    pub size_scaled_synapses_k: Option<f32>,
    pub births_per_window: usize,
    /// Metabolic cost coefficients per tick.
    pub cost_per_neuron: f32,
    pub cost_per_synapse: f32,
    pub cost_per_spike: f32,
    /// Detector thresholds.
    pub runaway_rate_hz: f32,
    pub runaway_sustained_ms: u64,
    pub fragmentation_min_component: f32, // largest WCC / neurons
    pub check_every_ticks: u64,
    pub report_every_ticks: u64,
}

impl Default for ResourceConfig {
    fn default() -> Self {
        Self {
            max_neurons: 200,
            max_synapses: 2000,
            size_scaled_synapses_k: None,
            births_per_window: 4,
            cost_per_neuron: 1e-6,
            cost_per_synapse: 2e-7,
            cost_per_spike: 1e-4,
            runaway_rate_hz: 50.0,
            runaway_sustained_ms: 5_000,
            fragmentation_min_component: 0.6,
            check_every_ticks: 1_000,
            report_every_ticks: 100,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageSample {
    pub tick: Tick,
    pub neurons: usize,
    pub synapses: usize,
    pub spikes_this_window: u64,
    pub metabolic_cost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Failure {
    pub kind: String,
    pub detail: String,
    pub tick: Tick,
}

pub struct ResourceMonitor {
    pub cfg: ResourceConfig,
    /// Cumulative metabolic cost.
    pub total_cost: f32,
    /// Cost accumulated since last report.
    pub window_cost: f32,
    pub spikes_this_window: u64,
    /// Windowed births (window = check_every_ticks).
    births_this_window: usize,
    /// Runaway detector state: tick when mean rate first exceeded threshold.
    runaway_since: Option<Tick>,
    /// Total spikes across run.
    pub total_spikes: u64,
}

impl ResourceMonitor {
    pub fn new(cfg: ResourceConfig) -> Self {
        Self {
            cfg,
            total_cost: 0.0,
            window_cost: 0.0,
            spikes_this_window: 0,
            births_this_window: 0,
            runaway_since: None,
            total_spikes: 0,
        }
    }

    /// Called every tick with the tick's spike count. Returns (usage sample
    /// if due, failure if any).
    pub fn step(&mut self, net: &Network, spikes: usize) -> (Option<ResourceUsageSample>, Option<Failure>) {
        let tick = net.tick;
        let live_neurons = net.neurons.iter().filter(|n| !n.retired).count();
        let live_synapses = net.live_synapse_count();

        let cost = (live_neurons as f32 * self.cfg.cost_per_neuron
            + live_synapses as f32 * self.cfg.cost_per_synapse
            + spikes as f32 * self.cfg.cost_per_spike)
            * 1.0; // per tick
        self.total_cost += cost;
        self.window_cost += cost;
        self.spikes_this_window += spikes as u64;
        self.total_spikes += spikes as u64;

        let mut failure = None;

        // Caps.
        if live_neurons > self.cfg.max_neurons {
            failure = Some(Failure {
                kind: "resource-exhaustion".into(),
                detail: format!("neurons {live_neurons} > cap {}", self.cfg.max_neurons),
                tick,
            });
        }
        let cap = match self.cfg.size_scaled_synapses_k {
            Some(k) => (k * live_neurons as f32) as usize,
            None => self.cfg.max_synapses,
        };
        if live_synapses > cap {
            failure = Some(Some(Failure {
                kind: "resource-exhaustion".into(),
                detail: format!("synapses {live_synapses} > cap {cap} (k={:?}, n={live_neurons})",
                    self.cfg.size_scaled_synapses_k),
                tick,
            }))
            .unwrap();
        }

        // Runaway: mean rate over non-input neurons.
        let internal: Vec<f32> = net
            .neurons
            .iter()
            .filter(|n| n.class == NeuronClass::Internal && !n.retired)
            .map(|n| n.rate_hz)
            .collect();
        if !internal.is_empty() {
            let mean = internal.iter().sum::<f32>() / internal.len() as f32;
            if mean > self.cfg.runaway_rate_hz {
                let since = self.runaway_since.unwrap_or(tick);
                self.runaway_since = Some(since);
                if tick.0.saturating_sub(since.0) >= self.cfg.runaway_sustained_ms {
                    failure = Some(Failure {
                        kind: "runaway-activity".into(),
                        detail: format!("mean rate {mean:.1} Hz > {} Hz for {} ms",
                            self.cfg.runaway_rate_hz, self.cfg.runaway_sustained_ms),
                        tick,
                    });
                }
            } else {
                self.runaway_since = None;
            }
        }

        // Fragmentation: largest weakly-connected component (every check window).
        if tick.0 % self.cfg.check_every_ticks == 0 && tick.0 > 0 {
            if let Some(f) = self.check_fragmentation(net, tick) {
                failure = Some(f);
            }
            self.births_this_window = 0;
            self.window_cost = 0.0;
        }

        let sample = if tick.0 % self.cfg.report_every_ticks == 0 {
            let s = ResourceUsageSample {
                tick,
                neurons: live_neurons,
                synapses: live_synapses,
                spikes_this_window: self.spikes_this_window,
                metabolic_cost: self.window_cost,
            };
            self.spikes_this_window = 0;
            Some(s)
        } else {
            None
        };

        (sample, failure)
    }

    /// Largest weakly-connected component over live synapses, including
    /// input-channel neurons as roots (they are always connected sources).
    fn check_fragmentation(&self, net: &Network, tick: Tick) -> Option<Failure> {
        let n = net.neurons.len();
        if n == 0 {
            return None;
        }
        // Union-Find over live synapses (weak connectivity = undirected).
        let mut parent: Vec<u32> = (0..n as u32).collect();
        fn find(parent: &mut Vec<u32>, mut x: u32) -> u32 {
            while parent[x as usize] != x {
                let p = parent[x as usize];
                parent[x as usize] = parent[p as usize];
                x = parent[p as usize];
            }
            x
        }
        for s in net.live_synapses() {
            let a = find(&mut parent, s.pre.0);
            let b = find(&mut parent, s.post.0);
            if a != b {
                parent[a as usize] = b;
            }
        }
        let mut counts = vec![0u32; n];
        for i in 0..n {
            let root = find(&mut parent, i as u32);
            counts[root as usize] += 1;
        }
        let largest = counts.iter().copied().max().unwrap_or(0) as f32;
        let frac = largest / n as f32;
        if frac < self.cfg.fragmentation_min_component {
            return Some(Failure {
                kind: "fragmentation".into(),
                detail: format!("largest component {frac:.2} < {}", self.cfg.fragmentation_min_component),
                tick,
            });
        }
        None
    }

    /// Birth admission: respects births_per_window and neuron cap.
    /// Returns Err(failure) when a birth would violate a cap.
    pub fn admit_birth(&mut self, net: &Network) -> Result<(), Failure> {
        let live = net.neurons.iter().filter(|n| !n.retired).count();
        if live + 1 > self.cfg.max_neurons {
            return Err(Failure {
                kind: "resource-exhaustion".into(),
                detail: format!("birth would exceed max_neurons {}", self.cfg.max_neurons),
                tick: net.tick,
            });
        }
        if self.births_this_window >= self.cfg.births_per_window {
            return Err(Failure {
                kind: "birth-rate-limit".into(),
                detail: format!("births this window >= {}", self.cfg.births_per_window),
                tick: net.tick,
            });
        }
        self.births_this_window += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::{InputFrame, NetworkConfig};

    #[test]
    fn reports_usage_on_interval() {
        let mut net = Network::new(NetworkConfig::default(), 2, 4, 1, 9);
        let mut mon = ResourceMonitor::new(ResourceConfig::default());
        let mut samples = 0;
        for t in 0..=250 {
            let (s, f) = mon.step(&net, 0);
            assert!(f.is_none());
            if s.is_some() {
                samples += 1;
            }
            let _ = net.step(&InputFrame { tick: Tick(t), spikes: vec![] });
        }
        assert_eq!(samples, 3, "ticks 0,100,200 (plus tick accounting)");
    }

    #[test]
    fn birth_cap_failure() {
        let net = Network::new(NetworkConfig::default(), 2, 4, 1, 9);
        let mut mon = ResourceMonitor::new(ResourceConfig {
            max_neurons: 7, // exactly current live count
            ..ResourceConfig::default()
        });
        let live = net.neurons.iter().filter(|n| !n.retired).count();
        assert_eq!(live, 7);
        let err = mon.admit_birth(&net).unwrap_err();
        assert_eq!(err.kind, "resource-exhaustion");
    }

    #[test]
    fn birth_rate_limit() {
        let net = Network::new(NetworkConfig::default(), 2, 4, 1, 9);
        let mut mon = ResourceMonitor::new(ResourceConfig {
            births_per_window: 1,
            ..ResourceConfig::default()
        });
        assert!(mon.admit_birth(&net).is_ok());
        let err = mon.admit_birth(&net).unwrap_err();
        assert_eq!(err.kind, "birth-rate-limit");
    }

    #[test]
    fn runaway_detected() {
        let mut net = Network::new(NetworkConfig::default(), 2, 4, 1, 9);
        for n in net.neurons.iter_mut() {
            if n.class == NeuronClass::Internal {
                n.rate_hz = 60.0;
            }
        }
        net.tick = Tick(6_001); // not a fragmentation-check multiple (6000 is)
        let mut mon = ResourceMonitor::new(ResourceConfig::default());
        mon.runaway_since = Some(Tick(0));
        let (_, f) = mon.step(&net, 0);
        let f = f.expect("60Hz for 6s must trip runaway");
        assert_eq!(f.kind, "runaway-activity");
    }

    #[test]
    fn fragmentation_detected() {
        // Build a net, then sever: prune ALL synapses ⇒ n isolated components.
        let mut net = Network::new(NetworkConfig::default(), 2, 4, 1, 9);
        let ids: Vec<_> = net.synapses.iter().map(|s| s.id).collect();
        for id in ids {
            net.prune_synapse(id);
        }
        net.tick = Tick(1_000);
        let mon = ResourceMonitor::new(ResourceConfig::default());
        let f = mon.check_fragmentation(&net, net.tick).expect("fully severed net must fragment");
        assert_eq!(f.kind, "fragmentation");
    }
}
