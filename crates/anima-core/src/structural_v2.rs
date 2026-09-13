//! ANIMA v2 structural plasticity (docs/anima-v2-protocol.md, M2–M6).
//!
//! Fast path (per tick): nothing here — STDP + adaptation run elsewhere.
//! Slow path: `V2Plasticity::window` every `window_ticks` ticks, in the
//! frozen order: M4 prune → M3 candidates (accumulate → permanence/die →
//! redraw; M5 eviction inside permanence) → M2 normalize → M6 anti-Hebbian
//! → M5 budget-invariant check.
//!
//! Determinism: every random draw comes from `net.rng` in a fixed order;
//! iteration is over Vec index order; tie-breaks by lowest SynapseId.

use crate::network::{NeuronClass, NeuronId, Network, SynapseId, Tick, V2Params};

/// One candidate afferent (M3): a contact with a weight that can become a
/// real synapse upon reinforcement.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub pre: NeuronId,
    pub w: f32,
}

/// Structural mutation reported to the harness for telemetry.
#[derive(Debug, Clone)]
pub enum V2Event {
    SynapseCreated {
        syn: SynapseId,
        pre: NeuronId,
        post: NeuronId,
        w: f32,
        reason: &'static str,
    },
    SynapsePruned {
        syn: SynapseId,
        reason: &'static str,
    },
}

/// Per-window budget usage reported to ResourceUsage telemetry (M5).
#[derive(Debug, Clone, Default)]
pub struct V2Report {
    pub live_exc: u64,
    pub live_inh: u64,
    pub total_exc_budget: u64,
    pub total_inh_budget: u64,
}

/// Per-window firing accumulator + M2–M6 state.
pub struct V2Plasticity {
    params: V2Params,
    /// Neurons that fired at least once in the current window.
    fired: Vec<bool>,
    /// Consecutive windows below the prune threshold, per synapse id.
    low_windows: Vec<u64>,
    /// Candidate slots, per neuron id (non-input neurons only).
    pub candidates: Vec<Vec<Candidate>>,
    /// Live excitatory/inhibitory counts per neuron (M5 accounting).
    live_e: Vec<usize>,
    live_i: Vec<usize>,
    /// Budget usage snapshot for telemetry.
    pub report: V2Report,
}

impl V2Plasticity {
    /// Build candidate pools in the frozen order (neuron-id order, partners
    /// channel-id then neuron-id; D8: no candidates onto input neurons).
    pub fn new(net: &mut Network, params: V2Params) -> Self {
        let n = net.neurons.len();
        let mut candidates = vec![Vec::new(); n];
        for post in 0..n {
            if net.neurons[post].class == NeuronClass::Input {
                continue; // D8: input neurons are pure sources
            }
            let mut pool = Vec::with_capacity(params.c_slots);
            while pool.len() < params.c_slots {
                match draw_candidate(net, &pool, post, &params) {
                    Some(pre) => pool.push(Candidate { pre, w: params.w_c_init }),
                    None => break,
                }
            }
            candidates[post] = pool;
        }
        let (live_e, live_i) = count_live(net);
        Self {
            params,
            fired: vec![false; n],
            low_windows: vec![0; net.synapses.len()],
            candidates,
            live_e,
            live_i,
            report: V2Report::default(),
        }
    }

    /// Fast-path hook: record which neurons fired this tick (no mutation).
    pub fn tick(&mut self, spikes: &[NeuronId]) {
        for &n in spikes {
            if let Some(f) = self.fired.get_mut(n.idx()) {
                *f = true;
            }
        }
    }

    /// One structural window. Frozen order; returns events for telemetry.
    pub fn window(&mut self, net: &mut Network, tick: Tick) -> Vec<V2Event> {
        let mut events = Vec::new();

        // --- M4: competitive pruning (excitatory only; frees slots) ---
        if !self.params.disable_m3_m4 {
            self.prune_low(net, &mut events);
        }

        // --- M3: candidate accumulate → permanence/die → redraw ---
        if !self.params.disable_m3_m4 {
            let mut candidates = std::mem::take(&mut self.candidates);
            for post in 0..net.neurons.len() {
                if net.neurons[post].class == NeuronClass::Input {
                    continue;
                }
                candidates[post] = self.candidate_pass(net, post, std::mem::take(&mut candidates[post]), tick, &mut events);
            }
            self.candidates = candidates;
        }

        // --- M2: normalize excitatory afferents per neuron ---
        if !self.params.disable_m2 {
            self.normalize(net);
        }

        // --- M6: anti-Hebbian inhibitory update ---
        if !self.params.disable_m6 {
            self.inhibitory(net);
        }

        // --- M5: budget-invariant check + report ---
        self.budget_check(net);

        // Reset window firing state for the next window.
        self.fired.iter_mut().for_each(|f| *f = false);
        events
    }

    /// M4: prune live excitatory synapses whose weight stayed below
    /// theta_prune for >= prune_windows consecutive windows, exactly as
    /// frozen (no exceptions).
    fn prune_low(&mut self, net: &mut Network, events: &mut Vec<V2Event>) {
        if self.low_windows.len() < net.synapses.len() {
            self.low_windows.resize(net.synapses.len(), 0);
        }
        for sid in 0..net.synapses.len() {
            let s = &net.synapses[sid];
            if s.silent_ticks == u64::MAX || s.inhibitory {
                self.low_windows[sid] = 0;
                continue;
            }
            if s.w < self.params.theta_prune {
                self.low_windows[sid] += 1;
                if self.low_windows[sid] >= self.params.prune_windows {
                    let sid = SynapseId(sid as u32);
                    net.prune_synapse(sid);
                    self.low_windows[sid.idx()] = 0;
                    events.push(V2Event::SynapsePruned { syn: sid, reason: "competitive-prune" });
                }
            } else {
                self.low_windows[sid] = 0;
            }
        }
    }

    /// M3 for one neuron: return the updated candidate pool + emit events.
    #[allow(clippy::too_many_arguments)]
    fn candidate_pass(
        &mut self,
        net: &mut Network,
        post: usize,
        mut pool: Vec<Candidate>,
        tick: Tick,
        events: &mut Vec<V2Event>,
    ) -> Vec<Candidate> {
        let post_id = NeuronId(post as u32);
        let mut i = 0;
        while i < pool.len() {
            let pre = pool[i].pre;
            let coactive = self.fired[post] && self.fired[pre.idx()];
            if coactive {
                pool[i].w += self.params.delta_perm;
            } else {
                pool[i].w *= self.params.decay_c;
            }
            if pool[i].w >= self.params.theta_permanent {
                // Permanence: needs a free B_e slot; evict lowest-weight
                // live excitatory synapse otherwise (frozen tie-break).
                self.evict_for(net, post_id, events);
                let w = self.params.w_c_permanent;
                let syn = net.add_synapse(pre, post_id, w, true, tick);
                self.live_e[post] += 1;
                events.push(V2Event::SynapseCreated {
                    syn,
                    pre,
                    post: post_id,
                    w,
                    reason: "candidate-permanence",
                });
                // Slot now occupied by a real synapse; withdraw candidate
                // and redraw fresh (the old pre is now connected).
                pool.swap_remove(i);
                if let Some(npre) = draw_candidate(net, &pool, post, &self.params) {
                    pool.push(Candidate { pre: npre, w: self.params.w_c_init });
                }
            } else if pool[i].w < self.params.theta_die {
                pool.swap_remove(i);
                if let Some(npre) = draw_candidate(net, &pool, post, &self.params) {
                    pool.push(Candidate { pre: npre, w: self.params.w_c_init });
                }
            } else {
                i += 1;
            }
        }
        pool
    }

    /// M5: evict the lowest-weight live excitatory incoming synapse of
    /// `post` when its budget is full. Tie-break: lowest SynapseId.
    fn evict_for(&mut self, net: &mut Network, post: NeuronId, events: &mut Vec<V2Event>) {
        if self.params.disable_m5 {
            return; // ablation arm v2−M5: unbounded slots, no eviction
        }
        if self.live_e[post.idx()] < self.params.b_e {
            return;
        }
        let mut worst: Option<(f32, SynapseId)> = None;
        for &sid in &net.incoming[post.idx()] {
            let s = &net.synapses[sid.idx()];
            if s.silent_ticks == u64::MAX || s.inhibitory {
                continue;
            }
            match worst {
                None => worst = Some((s.w, sid)),
                Some((w0, id0)) => {
                    if s.w < w0 || (s.w == w0 && sid.0 < id0.0) {
                        worst = Some((s.w, sid));
                    }
                }
            }
        }
        if let Some((_, sid)) = worst {
            net.prune_synapse(sid);
            self.low_windows[sid.idx()] = 0;
            self.live_e[post.idx()] -= 1;
            events.push(V2Event::SynapsePruned { syn: sid, reason: "budget-eviction" });
        }
    }

    /// M2: rescale each neuron's live incoming excitatory weights to total
    /// t_e, then clamp to [w_min, w_max]. Invariant: post-pass sum <= t_e.
    fn normalize(&mut self, net: &mut Network) {
        for post in 0..net.neurons.len() {
            if net.neurons[post].class == NeuronClass::Input {
                continue;
            }
            let incoming: Vec<SynapseId> = net.incoming[post].clone();
            let mut sum: f32 = 0.0;
            for &sid in &incoming {
                let s = &net.synapses[sid.idx()];
                if s.silent_ticks != u64::MAX && !s.inhibitory {
                    sum += s.w;
                }
            }
            if sum <= 0.0 || (sum - self.params.t_e).abs() < 1e-9 {
                continue;
            }
            let factor = self.params.t_e / sum;
            for sid in incoming {
                let s = &mut net.synapses[sid.idx()];
                if s.silent_ticks == u64::MAX || s.inhibitory {
                    continue;
                }
                s.w = (s.w * factor).clamp(net.cfg.w_min, net.cfg.w_max);
            }
        }
    }

    /// M6: anti-Hebbian update — co-activity strengthens suppression.
    fn inhibitory(&mut self, net: &mut Network) {
        for sid in 0..net.synapses.len() {
            let (inhib, alive, pre, post, w) = {
                let s = &net.synapses[sid];
                (s.inhibitory, s.silent_ticks != u64::MAX, s.pre, s.post, s.w)
            };
            if !inhib || !alive {
                continue;
            }
            let co = self.fired[pre.idx()] && self.fired[post.idx()];
            let nw = if co {
                (w + self.params.a_inh).min(self.params.w_inh_max)
            } else {
                (w * self.params.decay_inh).min(self.params.w_inh_max)
            };
            net.synapses[sid].w = nw;
        }
    }

    /// M5: budget-invariant check + report snapshot.
    fn budget_check(&mut self, net: &Network) {
        let (live_e, live_i) = count_live(net);
        self.live_e.clone_from(&live_e);
        self.live_i.clone_from(&live_i);
        let n = net.neurons.len();
        self.report = V2Report {
            live_exc: live_e.iter().map(|&x| x as u64).sum(),
            live_inh: live_i.iter().map(|&x| x as u64).sum(),
            total_exc_budget: (n as u64) * (self.params.b_e as u64),
            total_inh_budget: (n as u64) * (self.params.b_i as u64),
        };
        // A-4 (implementation audit): the protocol's budget-invariant check
        // is enforced as a hard assert every window (blocking runaway),
        // except in the v2-m5 ablation arm where the budget is off by
        // registration.
        if !self.params.disable_m5 {
            assert!(
                live_e.iter().all(|&x| x <= self.params.b_e),
                "v2 budget invariant violated: excitatory count over B_e (post={})",
                live_e.iter().position(|&x| x > self.params.b_e).unwrap_or(0)
            );
            assert!(
                live_i.iter().all(|&x| x <= self.params.b_i),
                "v2 budget invariant violated: inhibitory count over B_i (post={})",
                live_i.iter().position(|&x| x > self.params.b_i).unwrap_or(0)
            );
        }
    }
}

/// Draw one candidate afferent for `post` in the frozen order:
/// unconnected input channels first (channel-id order, Bernoulli
/// p_cand_in), then unconnected non-input neurons (neuron-id order,
/// Bernoulli p_cand_rec). No duplicate against live synapses or other
/// candidates; D8: never target an input neuron (post is never input).
fn draw_candidate(
    net: &mut Network,
    pool: &[Candidate],
    post: usize,
    params: &V2Params,
) -> Option<NeuronId> {
    use rand::Rng;
    let connected = |pre: NeuronId| -> bool {
        if net.incoming[post].iter().any(|&sid| {
            let s = &net.synapses[sid.idx()];
            s.silent_ticks != u64::MAX && s.pre == pre
        }) {
            return true;
        }
        pool.iter().any(|c| c.pre == pre)
    };
    // (1) input channels, channel-id order, acceptance p_cand_in.
    for ch in 0..net.channels.len() {
        let pre = net.channels[ch].target;
        if connected(pre) {
            continue;
        }
        if net.rng.gen::<f32>() < params.p_cand_in {
            return Some(pre);
        }
    }
    // (2) non-input neurons, neuron-id order, acceptance p_cand_rec.
    let post_id = NeuronId(post as u32);
    for neur in net.neurons.iter() {
        let pre = neur.id;
        if pre == post_id || neur.class == NeuronClass::Input || connected(pre) {
            continue;
        }
        if net.rng.gen::<f32>() < params.p_cand_rec {
            return Some(pre);
        }
    }
    None
}

fn count_live(net: &Network) -> (Vec<usize>, Vec<usize>) {
    let n = net.neurons.len();
    let mut e = vec![0; n];
    let mut i = vec![0; n];
    for s in &net.synapses {
        if s.silent_ticks == u64::MAX {
            continue;
        }
        if s.inhibitory {
            i[s.post.idx()] += 1;
        } else {
            e[s.post.idx()] += 1;
        }
    }
    (e, i)
}