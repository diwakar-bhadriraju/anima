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
use crate::rate_balance::{E6Params, RateBalance};

/// One candidate afferent (M3): a contact with a weight that can become a
/// real synapse upon reinforcement.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub pre: NeuronId,
    pub w: f32,
    /// Dormant-candidate reserve (docs/x-clla-dormant-reserve.md §3.1):
    /// set when this candidate's w first exceeds w_c_init (its pre once
    /// co-fired with its post). Reserved candidates decay to a floor
    /// (theta_die) instead of dying; bit is local, no timestamp.
    pub reserved: bool,
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
    /// Dormant-reserve instrumentation: candidate pool snapshot for one
    /// neuron (emitted at snapshot cadence when the reserve flag is on).
    CandidatePool {
        post: NeuronId,
        /// (pre, w, reserved, eligible_waiting) per slot, slot order.
        slots: Vec<(u32, f32, bool, bool)>,
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
    /// E6 rate balancing (docs/anima-e6-protocol.md §3); None = inert.
    pub rate_balance: Option<RateBalance>,
    /// CLLA allocation rule (docs/x-clla-allocation-rule.md): per-window
    /// delivered input-current sums split by consolidated flag.
    /// res_ip[i] = Σ amplitude·w over fired input-channel afferents with
    /// consolidated=true onto neuron i; res_iw[i] = same, unconsolidated.
    /// Zeroed at each window START; R = res_ip/(res_ip+res_iw) read at M3.
    res_ip: Vec<f32>,
    res_iw: Vec<f32>,
    /// Allocation gate g per neuron for the current window.
    res_gate: Vec<f32>,
    /// Dormant reserve (docs/x-clla-dormant-reserve.md): per-neuron set of
    /// input channels that fired this window (from the R-split delivery
    /// scan). Used for the pool-pressure test P2 ('co-active input channel
    /// fired with no pool candidate'). Cleared at window start.
    fired_channels: Vec<std::collections::BTreeSet<u32>>,
}

impl V2Plasticity {
    /// Debug accessor for tests.
    #[allow(dead_code)]
    pub fn debug_fired(&self, post: usize) -> &std::collections::BTreeSet<u32> {
        &self.fired_channels[post]
    }
}

impl V2Plasticity {
    /// Build candidate pools in the frozen order (neuron-id order, partners
    /// channel-id then neuron-id; D8: no candidates onto input neurons).
    pub fn new(net: &mut Network, params: V2Params, e6: Option<E6Params>) -> Self {
        let n = net.neurons.len();
        let mut candidates = vec![Vec::new(); n];
        for post in 0..n {
            if net.neurons[post].class == NeuronClass::Input {
                continue; // D8: input neurons are pure sources
            }
            let mut pool = Vec::with_capacity(params.c_slots);
            while pool.len() < params.c_slots {
                match draw_candidate(net, &pool, post, &params, None) {
                    Some(pre) => pool.push(Candidate { pre, w: params.w_c_init, reserved: false }),
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
            rate_balance: e6.map(RateBalance::new),
            res_ip: vec![0.0; n],
            res_iw: vec![0.0; n],
            res_gate: vec![1.0; n],
            fired_channels: (0..n).map(|_| std::collections::BTreeSet::new()).collect(),
        }
    }

    /// Fast-path hook: record which neurons fired this tick (no mutation),
    /// plus E6 input-channel event counts (§3.1).
    pub fn tick(&mut self, spikes: &[NeuronId]) {
        for &n in spikes {
            if let Some(f) = self.fired.get_mut(n.idx()) {
                *f = true;
            }
            if let Some(rb) = self.rate_balance.as_mut() {
                rb.tick(n);
            }
        }
    }

    /// β for one (pre → post) contact: 1.0 when E6 is disabled (exact v2
    /// path); the E6 balance factor otherwise. Applied at the M3 co-active
    /// accumulation and the STDP a⁺ branch — nowhere else.
    pub fn beta(&self, net: &Network, post: NeuronId, pre: NeuronId) -> f32 {
        match &self.rate_balance {
            Some(rb) => rb.beta(net, post, pre),
            None => 1.0,
        }
    }

    /// CLLA gate read by the harness (passive-decay exemption).
    pub fn assembly_enabled(&self) -> bool {
        self.params.assembly_protect
    }

    /// CLLA allocation rule enabled? (flag + base CLLA both required).
    pub fn alloc_residual_enabled(&self) -> bool {
        self.params.assembly_protect && self.params.alloc_residual
    }

    /// Current window's allocation gate per neuron (tests/instrumentation).
    pub(crate) fn residual_gate(&self, post: usize) -> f32 {
        self.res_gate[post]
    }

    /// Per-tick accumulation of delivered input current onto each post
    /// neuron, split by the synapse's consolidated flag. Mirrors the
    /// network deposit loop exactly (amplitude·w per fired input-channel
    /// afferent; excitatory only; post internal). No-op when the rule is
    /// off — accumulators stay zero and res_gate stays 1.0 (identity).
    pub fn accumulate_input_current(&mut self, net: &Network, spikes: &[NeuronId]) {
        if !self.alloc_residual_enabled() && !self.dormant_reserve_enabled() {
            return;
        }
        // Fired input channels this tick (per post neuron) for the reserve
        // pool-pressure test / first-exposure allocation — tracked when
        // the reserve OR the allocation rule is on.
        let track_channels = self.dormant_reserve_enabled() || self.alloc_residual_enabled();
        for &pre in spikes {
            if pre.0 >= net.channels.len() as u32 {
                continue; // input-channel afferents only (D8)
            }
            let outgoing: Vec<SynapseId> = net.outgoing[pre.idx()].clone();
            for sid in outgoing {
                let s = &net.synapses[sid.idx()];
                if s.silent_ticks == u64::MAX {
                    continue; // tombstoned (pruned) — mirror network deposit
                }
                if s.inhibitory || !s.plastic {
                    continue;
                }
                if s.post.idx() >= net.neurons.len() {
                    continue;
                }
                let cur = s.amplitude * s.w;
                let i = s.post.idx();
                if track_channels && i < self.fired_channels.len() {
                    self.fired_channels[i].insert(pre.0);
                }
                if s.consolidated {
                    self.res_ip[i] += cur;
                } else {
                    self.res_iw[i] += cur;
                }
            }
        }
    }

    /// Dormant-reserve flag (docs/x-clla-dormant-reserve.md).
    pub fn dormant_reserve_enabled(&self) -> bool {
        self.params.dormant_reserve
    }

    /// Candidate-pool snapshot for one neuron (instrumentation; emitted at
    /// snapshot cadence when the reserve flag is on; identity-safe).
    pub fn pool_snapshot(&self, post: usize) -> Vec<(u32, f32, bool, bool)> {
        self.candidates[post].iter()
            .map(|c| (c.pre.0, c.w, c.reserved, c.w >= self.params.theta_permanent))
            .collect()
    }

    /// Compute the allocation gate for every neuron from the JUST-FINISHED
    /// window's current sums, then zero the accumulators. Called at window
    /// start (pre-M4). Flag off: gate stays 1.0 (identity).
    fn compute_residual_gate(&mut self, net: &Network) {
        let n = self.res_gate.len();
        for i in 0..n {
            let ip = self.res_ip[i];
            let iw = self.res_iw[i];
            let total = ip + iw;
            let r = if total > 0.0 { ip / total } else { 1.0 }; // silence: fully explained
            let p = self.consolidated_mass(net, NeuronId(i as u32));
            let headroom = self.params.p_max_frac * self.params.t_e - p;
            self.res_gate[i] = if headroom > 0.0 { (1.0 - r).max(0.0) } else { 0.0 };
        }
        for v in self.res_ip.iter_mut() { *v = 0.0; }
        for v in self.res_iw.iter_mut() { *v = 0.0; }
    }

    /// One structural window. Frozen order; returns events for telemetry.
    pub fn window(&mut self, net: &mut Network, tick: Tick) -> Vec<V2Event> {
        let mut events = Vec::new();

        // --- E6 step 0 (frozen §3.1): φ ← EMA(previous window counts) ---
        if let Some(rb) = self.rate_balance.as_mut() {
            rb.window_start();
        }

        // --- CLLA allocation rule: compute g from the JUST-FINISHED window's
        // input-current sums, then reset accumulators (identity when off:
        // gate stays 1.0). Read by M3 below. ---
        self.compute_residual_gate(net);

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
        for fc in self.fired_channels.iter_mut() { fc.clear(); }
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
            if s.consolidated {
                self.low_windows[sid] = 0;
                continue; // CLLA: protected from M4
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
        let reserve = self.dormant_reserve_enabled();
        // First-exposure allocation (source-corrected, docs/x-clla-fe-impl-
        // audit.md §6): candidate draws bias to input channels that FIRED
        // this window per the module firing record (the same record M3
        // co-activity uses — content-addressable without any live
        // synapse). NOTE: the prior source (fired_channels[post], derived
        // from live outgoing synapses) was vacuous — a channel visible
        // there necessarily had a live afferent, which connected()
        // subsequently excluded. The `fired` vector observes every firing
        // input channel regardless of synapse survival.
        let fe_fired: Option<std::collections::BTreeSet<u32>> =
            if self.alloc_residual_enabled() {
                let n_input = net.channels.len().min(self.fired.len());
                Some((0..n_input).filter(|&ch| self.fired[ch]).map(|ch| ch as u32).collect())
            } else {
                None
            };
        let mut i = 0;
        while i < pool.len() {
            let pre = pool[i].pre;
            let coactive = self.fired[post] && self.fired[pre.idx()];
            if coactive {
                // E6 (frozen §3.3.2): co-active accumulation × β_pre. When
                // E6 is disabled β ≡ 1, exact v2 increment.
                // CLLA allocation rule (docs/x-clla-allocation-rule.md §2):
                // × g(post) — 1.0 when rule off (identity); 0 when headroom
                // exhausted or the window's input was fully explained.
                let g = if self.alloc_residual_enabled() {
                    self.res_gate[post]
                } else {
                    1.0
                };
                pool[i].w += self.params.delta_perm * self.beta(net, post_id, pre) * g;
                // Dormant reserve: first co-active accumulation marks the
                // candidate reserved (w > w_c_init is the derivable moment).
                if reserve && !pool[i].reserved && pool[i].w > self.params.w_c_init {
                    pool[i].reserved = true;
                }
            } else if reserve && pool[i].reserved {
                // Reserved: decay to the floor theta_die, never below.
                // Repeated non-coactivity cannot remove a reserved candidate.
                pool[i].w = (pool[i].w * self.params.decay_c).max(self.params.theta_die);
            } else {
                pool[i].w *= self.params.decay_c;
            }
            if pool[i].w >= self.params.theta_permanent {
                let cap_ok = !self.params.assembly_protect
                    || self.consolidated_mass(net, post_id) + self.params.w_c_permanent
                        <= self.params.p_max_frac * self.params.t_e;
                if !cap_ok && reserve {
                    // Headroom exhausted: eligible-waiting — pin at
                    // theta_permanent, hold the slot, retry next window.
                    // (docs/x-clla-dormant-reserve.md §2)
                    pool[i].w = self.params.theta_permanent;
                    i += 1;
                    continue;
                }
                // Permanence: needs a free B_e slot; evict lowest-weight
                // live excitatory synapse otherwise (frozen tie-break).
                self.evict_for(net, post_id, events);
                let w = self.params.w_c_permanent;
                let syn = net.add_synapse(pre, post_id, w, true, tick);
                // CLLA: consolidate at permanence iff (a) mechanism on,
                // (b) candidate weight cleared w_consolidate_min (protocol:
                // 0.05 = theta_permanent, so this holds exactly when the
                // permanence branch is reached), (c) cap headroom: P + w
                // <= p_max_frac * t_e (P = current consolidated mass).
                if self.params.assembly_protect
                    && pool[i].w >= self.params.w_consolidate_min
                    && self.consolidated_mass(net, post_id) + w
                        <= self.params.p_max_frac * self.params.t_e
                {
                    net.synapses[syn.idx()].consolidated = true;
                }
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
                if let Some(npre) = draw_candidate(net, &pool, post, &self.params, fe_fired.as_ref()) {
                    pool.push(Candidate { pre: npre, w: self.params.w_c_init, reserved: false });
                }
            } else if (pool[i].w < self.params.theta_die) && !(reserve && pool[i].reserved) {
                // Death → redraw. Reserved candidates never die on
                // inactivity (floor at theta_die).
                pool.swap_remove(i);
                if let Some(npre) = draw_candidate(net, &pool, post, &self.params, fe_fired.as_ref()) {
                    pool.push(Candidate { pre: npre, w: self.params.w_c_init, reserved: false });
                }
            } else {
                i += 1;
            }
        }
        // Dormant reserve: pool-pressure eviction (P1 ∧ P2) —
        // all slots reserved/eligible-waiting AND a co-active input
        // channel fired with no pool candidate. Evict by priority:
        // un-reserved > reserved-at-floor > eligible-waiting, ties by
        // lowest pool index; replacement = the missing fired channel.
        if reserve || self.alloc_residual_enabled() {
            self.reserve_pressure_evict(net, post, &mut pool, post_id);
        }
        pool
    }

    /// Candidate pool pressure (docs/x-clla-dormant-reserve.md §1, reused by
    /// first-exposure allocation docs/x-clla-first-exposure-audit.md §5/§8):
    /// when a channel that fired on this neuron has no pool candidate and no
    /// evictable-free slot exists, evict one per the priority ladder and bind
    /// the missing fired channel. Priority: un-reserved lowest-w (index tie)
    /// then reserved-at-floor (index tie) then eligible-waiting (index tie).
    /// No new state; w/reserved/permanence/index only.
    fn reserve_pressure_evict(
        &mut self,
        net: &Network,
        post: usize,
        pool: &mut Vec<Candidate>,
        post_id: NeuronId,
    ) {
        if pool.len() < self.params.c_slots {
            return;
        }
        // P2 (source-corrected): a fired input channel with no matching
        // candidate — read from the module firing record (`fired`) so
        // channels without live afferents are bindable; duplicates via
        // live/pooled handled by the caller's connected() and by
        // draw_candidate's exclusion.
        let fired: std::collections::BTreeSet<u32> =
            (0..net.channels.len().min(self.fired.len()))
                .filter(|&ch| self.fired[ch])
                .map(|ch| ch as u32)
                .collect();
        if fired.is_empty() {
            return;
        }
        let missing: Vec<u32> = fired.iter().copied()
            .filter(|&ch| !pool.iter().any(|c| c.pre.0 == ch))
            .collect();
        if missing.is_empty() {
            return;
        }
        // Eviction priority: 1 un-reserved (none — all held), 2 reserved
        // at floor (lowest index), 3 eligible-waiting (w == theta_permanent,
        // lowest index). Priority 2 strictly before 3 per the freeze.
        let mut victim: Option<usize> = None;
        for (idx, c) in pool.iter().enumerate() {
            match victim {
                None => victim = Some(idx),
                Some(v) => {
                    let vc = &pool[v];
                    let v_prio = if vc.w >= self.params.theta_permanent { 2 } else { 1 };
                    let c_prio = if c.w >= self.params.theta_permanent { 2 } else { 1 };
                    if c_prio < v_prio || (c_prio == v_prio && c.w < vc.w) {
                        victim = Some(idx);
                    }
                }
            }
        }
        if let Some(v) = victim {
            let ch = missing[0];
            pool[v] = Candidate { pre: NeuronId(ch), w: self.params.w_c_init, reserved: false };
        }
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
            if net.synapses.len() > self.low_windows.len() {
                // M3 permanence grew the synapse arena mid-window; keep the
                // prune counter vector index-aligned (regression: OOB).
                self.low_windows.resize(net.synapses.len(), 0);
            }
            net.prune_synapse(sid);
            self.low_windows[sid.idx()] = 0;
            self.live_e[post.idx()] -= 1;
            events.push(V2Event::SynapsePruned { syn: sid, reason: "budget-eviction" });
        }
    }

    /// CLLA: sum of consolidated (protected) live excitatory weights
    /// onto `post`. Pure local read of the post neuron's incoming set.
    pub(crate) fn consolidated_mass(&self, net: &Network, post_id: NeuronId) -> f32 {
        let incoming = &net.incoming[post_id.idx()];
        let mut p = 0.0f32;
        for &sid in incoming {
            let s = &net.synapses[sid.idx()];
            if s.silent_ticks != u64::MAX && !s.inhibitory && s.consolidated {
                p += s.w;
            }
        }
        p
    }

    /// M2: rescale each neuron's live incoming excitatory weights to total
    /// t_e, then clamp to [w_min, w_max]. Invariant: post-pass sum <= t_e.
    ///
    /// V2.3 (docs/v2_3-design.md §2): when m2_buckets > 1, the budget is
    /// CAPACITY-MATCHED and partitioned by write-epoch bucket tag: per-bucket
    /// target T = t_e / n_populated_buckets, so the total post-pass sum never
    /// exceeds t_e (testing cross-trace sharing at constant capacity).
    /// m2_buckets = 1 (default) is the exact baseline path (identity).
    fn normalize(&mut self, net: &mut Network) {
        let n_buckets = self.params.m2_buckets.max(1);
        // CLLA (docs/anima-clla-protocol.md §3.7): consolidated synapses are
        // excluded from normalization. Working budget = t_e - P; factor =
        // (t_e - P)/W applied to working (unconsolidated) only, clamped to
        // [w_min, w_max]. When flag off this branch is never entered
        // (identity). Protocol freezes m2_buckets=1, so CLLA takes
        // precedence whenever assembly_protect is set.
        if self.params.assembly_protect {
            for post in 0..net.neurons.len() {
                if net.neurons[post].class == NeuronClass::Input {
                    continue;
                }
                let incoming: Vec<SynapseId> = net.incoming[post].clone();
                let post_id = NeuronId(post as u32);
                let p = self.consolidated_mass(net, post_id);
                let mut w_sum: f32 = 0.0;
                for &sid in &incoming {
                    let s = &net.synapses[sid.idx()];
                    if s.silent_ticks != u64::MAX && !s.inhibitory && !s.consolidated {
                        w_sum += s.w;
                    }
                }
                let target = self.params.t_e - p;
                if w_sum <= 0.0 || target <= 0.0 || (w_sum - target).abs() < 1e-9 {
                    continue;
                }
                let factor = target / w_sum;
                for sid in incoming {
                    let s = &mut net.synapses[sid.idx()];
                    if s.silent_ticks == u64::MAX || s.inhibitory || s.consolidated {
                        continue;
                    }
                    s.w = (s.w * factor).clamp(net.cfg.w_min, net.cfg.w_max);
                }
            }
            return;
        }
        if n_buckets == 1 {
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
            return;
        }
        // Partitioned, capacity-matched (V2.3 §2).
        for post in 0..net.neurons.len() {
            if net.neurons[post].class == NeuronClass::Input {
                continue;
            }
            let incoming: Vec<SynapseId> = net.incoming[post].clone();
            let mut sums = vec![0.0f32; n_buckets as usize];
            for &sid in &incoming {
                let s = &net.synapses[sid.idx()];
                if s.silent_ticks != u64::MAX && !s.inhibitory {
                    let b = (s.m2_bucket as usize).min(sums.len() - 1);
                    sums[b] += s.w;
                }
            }
            let populated: Vec<usize> = (0..sums.len()).filter(|&b| sums[b] > 0.0).collect();
            if populated.is_empty() {
                continue;
            }
            let per_bucket = self.params.t_e / populated.len() as f32;
            for &b in &populated {
                if (sums[b] - per_bucket).abs() < 1e-9 {
                    continue;
                }
                if sums[b] <= per_bucket {
                    continue; // never boost; invariant: sum <= t_e
                }
                let factor = per_bucket / sums[b];
                for sid in &incoming {
                    let s = &mut net.synapses[sid.idx()];
                    if s.silent_ticks == u64::MAX || s.inhibitory {
                        continue;
                    }
                    if (s.m2_bucket as usize).min(sums.len() - 1) == b {
                        s.w = (s.w * factor).clamp(net.cfg.w_min, net.cfg.w_max);
                    }
                }
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
///
/// First-exposure allocation (docs/x-clla-first-exposure-audit.md §8):
/// when `bias_fired` is Some, the currently-firing input channels of
/// `post` are tried FIRST in channel-id order (deterministic, no RNG
/// draw, skip already-connected/pooled); only if no fired channel is
/// bindable does the draw fall back to the frozen random path. This lets
/// a novel input acquire a candidate at first exposure. flag-off / empty
/// fired set => exact frozen random path (identity: no RNG consumed by
/// the bias branch).
fn draw_candidate(
    net: &mut Network,
    pool: &[Candidate],
    post: usize,
    params: &V2Params,
    bias_fired: Option<&std::collections::BTreeSet<u32>>,
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
    // (0) First-exposure allocation: if enabled and this neuron saw firing
    // input channels this window, try them first in channel-id order.
    // Deterministic (no RNG consumed so the flag-off path is identical),
    // skips already-connected/pooled channels; falls back to the frozen
    // random draw when none is bindable.
    if let Some(fired) = bias_fired {
        for &ch in fired.iter() {
            let pre = NeuronId(ch);
            if pre.0 < net.channels.len() as u32 && !connected(pre) {
                return Some(pre);
            }
        }
    }
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