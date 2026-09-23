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

use std::collections::HashMap;
use crate::network::{
    dcore_alpha_p, dcore_theta_sim, dcore_tracks, NeuronClass, NeuronId, Network, SynapseId, Tick, V2Params,
};
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
    /// Phase II-A D-core: per-neuron per-track protected/working input
    /// current (len 2*n, index i*K+t). Zero always when d_core off.
    res_ip_t: Vec<f32>,
    res_iw_t: Vec<f32>,
    /// Phase II-A D-core: per-neuron per-channel delivered-current window
    /// accumulator (len n * dcore_tracks() * 24, row-major i*24+c).
    /// The neuron's own usage vector x_i(w).
    ctx_acc: Vec<f32>,
    /// Phase II-A D-core: current window context per neuron (0/1),
    /// computed at window start from ctx_acc; used for M3 tagging.
    cur_ctx: Vec<u8>,
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

/// Cosine of two 24-dim vectors (0 when either norm is 0).
fn cosine_row(a: &[f32], b: &[f32]) -> f32 {
    let mut num = 0.0f32; let mut na = 0.0f32; let mut nb = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        num += x * y; na += x * x; nb += y * y;
    }
    if na <= 0.0 || nb <= 0.0 { 0.0 } else { num / (na.sqrt() * nb.sqrt()) }
}

/// D-core tie/novelty rule: track with the smaller protected mass
/// (deterministic; ties => lower index). Pure function of snapshot state.
fn protected_track_choice(net: &Network, post: usize, k: usize) -> u8 {
    let mut p = vec![0.0f32; k];
    if let Some(incoming) = net.incoming.get(post) {
        for &sid in incoming.iter() {
            let s = &net.synapses[sid.idx()];
            if s.silent_ticks != u64::MAX && !s.inhibitory && s.consolidated {
                let t = (s.track as usize).min(k - 1);
                p[t] += s.w;
            }
        }
    }
    let mut best = 0u8;
    for t in 1..k {
        if p[t] < p[best as usize] {
            best = t as u8;
        }
    }
    best
}

impl V2Plasticity {
    /// Build candidate pools in the frozen order (neuron-id order, partners
    /// channel-id then neuron-id; D8: no candidates onto input neurons).
    pub fn new(net: &mut Network, params: V2Params, e6: Option<E6Params>) -> Self {
        let n = net.neurons.len();
        let mut candidates = vec![Vec::new(); n];
        for post in 0..n {
            if net.neurons[post].class == NeuronClass::Input || net.neurons[post].class == NeuronClass::Inhibitory {
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
            res_ip_t: vec![0.0; n * dcore_tracks()],
            res_iw_t: vec![0.0; n * dcore_tracks()],
            ctx_acc: vec![0.0; n * 24],
            cur_ctx: vec![0; n],
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
        if !self.alloc_residual_enabled() && !self.dormant_reserve_enabled() && !self.d_core_enabled() {
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
                if self.d_core_enabled() && (i * 24 + pre.0 as usize) < self.ctx_acc.len() {
                    self.ctx_acc[i * 24 + pre.0 as usize] += cur;
                    let t = (s.track as usize).min(dcore_tracks() - 1);
                    if s.consolidated {
                        self.res_ip_t[i * dcore_tracks() + t] += cur;
                    } else {
                        self.res_iw_t[i * dcore_tracks() + t] += cur;
                    }
                }
                if s.consolidated {
                    self.res_ip[i] += cur;
                } else {
                    self.res_iw[i] += cur;
                }
            }
        }
    }

    /// Phase II-A D-core flag (docs/x-phase2-a-protocol.md).
    pub fn d_core_enabled(&self) -> bool {
        self.params.d_core
    }

    /// Current window context of a neuron (tests/instrumentation).
    pub(crate) fn current_ctx(&self, post: usize) -> u8 {
        self.cur_ctx.get(post).copied().unwrap_or(0)
    }

    /// Phase II-AR candidate-E flag.
    pub fn d_claim_enabled(&self) -> bool {
        self.params.d_core && self.params.d_claim
    }

    /// Phase III sparse-commit flag.
    pub fn d_sparse_enabled(&self) -> bool {
        self.params.d_core && self.params.d_claim && self.params.d_sparse
    }

    /// Committed track of a neuron under sparse-commit: Some(t) iff the
    /// protected share R_t = P_t/(P_0+P_1) > 0.5 (majority; ties/empty =>
    /// not committed). Pure local read of the post's incoming protection.
    pub(crate) fn committed_track(&self, net: &Network, post: usize) -> Option<u8> {
        let mut p = [0.0f32; 2];
        for &sid in &net.incoming[post] {
            let s = &net.synapses[sid.idx()];
            if s.silent_ticks != u64::MAX && !s.inhibitory && s.consolidated {
                let t = (s.track as usize).min(1);
                p[t] += s.w;
            }
        }
        let tot = p[0] + p[1];
        if tot <= 0.0 { return None; }
        if p[0] > p[1] && p[0] / tot >= crate::network::dcore_theta_commit() { Some(0) }
        else if p[1] > p[0] && p[1] / tot >= crate::network::dcore_theta_commit() { Some(1) }
        else { None }
    }

    /// Phase III-A sparse-commit pass: for each neuron committed to track t,
    /// decay OTHER-track working afferents toward the churn floor
    /// (floor_drop/window) so its effective integration becomes
    /// track-selective -> disjoint responder sets across memories.
    /// Ran at window end (after ctx_update, before M4). Deterministic.
    fn sparse_commit(&mut self, net: &mut Network) {
        if !self.d_sparse_enabled() { return; }
        let drop = crate::network::dcore_floor_drop();
        for post in 0..net.neurons.len() {
            if net.neurons[post].class == NeuronClass::Input || net.neurons[post].class == NeuronClass::Inhibitory { continue; }
            let Some(t) = self.committed_track(net, post) else { continue };
            let incoming: Vec<SynapseId> = net.incoming[post].clone();
            for sid in incoming {
                let s = &net.synapses[sid.idx()];
                if s.silent_ticks == u64::MAX || s.inhibitory || s.consolidated { continue; }
                if s.track != t {
                    net.synapses[sid.idx()].w =
                        (net.synapses[sid.idx()].w * drop).clamp(net.cfg.w_min, net.cfg.w_max);
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

    /// Phase II-AR two-regime normalization (protocol §1.4). See the
    /// branch comment; deterministic, invariant-preserving.
    fn normalize_claim(&mut self, net: &mut Network) {
        let theta = self.params.theta_prune;
        let t_e = self.params.t_e;
        for post in 0..net.neurons.len() {
            if net.neurons[post].class == NeuronClass::Input || net.neurons[post].class == NeuronClass::Inhibitory {
                continue;
            }
            let p_tot = self.consolidated_mass(net, NeuronId(post as u32));
            let b = t_e - p_tot;
            if b <= 0.0 {
                continue;
            }
            let incoming: Vec<SynapseId> = net.incoming[post].clone();
            // sparse-commit: a committed neuron's OTHER-track working
            // afferents are de-budgeted into the floor class (single-track
            // integration), so the dropout is not undone by the per-track
            // upscale.
            let ct = if self.d_sparse_enabled() { self.committed_track(net, post) } else { None };
            let mut claimed = [0.0f32; 2];
            let mut n_unc = 0usize;
            let mut unc_idxs: Vec<usize> = Vec::new();
            for &sid in &incoming {
                let s = &net.synapses[sid.idx()];
                if s.silent_ticks != u64::MAX && !s.inhibitory && !s.consolidated {
                    let t = (s.track as usize).min(1);
                    let is_committed_track = ct.map_or(true, |c| t as u8 == c);
                    if (ct.is_none() || is_committed_track) && s.track != 2 {
                        claimed[t] += s.w;
                    } else {
                        n_unc += 1;
                        unc_idxs.push(sid.idx());
                    }
                }
            }
            let f = if n_unc > 0 { theta.min(b / n_unc as f32) } else { 0.0 };
            let n_cl = if ct.is_some() {
                if claimed[ct.unwrap() as usize] > 0.0 { 1usize } else { 0usize }
            } else {
                (if claimed[0] > 0.0 { 1usize } else { 0usize })
                     + (if claimed[1] > 0.0 { 1usize } else { 0usize })
            };
            // (a) claimed tracks capacity-matched to (b - f*n_unc)/n_cl
            if n_cl > 0 {
                let claimed_budget = (b - f * n_unc as f32).max(0.0);
                let per_claim = claimed_budget / n_cl as f32;
                for t in 0..2usize {
                    if claimed[t] <= 0.0 {
                        continue;
                    }
                    let factor = per_claim / claimed[t];
                    for sid in incoming.iter().copied() {
                        let s = &mut net.synapses[sid.idx()];
                        if s.silent_ticks == u64::MAX || s.inhibitory || s.consolidated || s.track == 2 {
                            continue;
                        }
                        if (s.track as usize).min(1) == t {
                            s.w = (s.w * factor).clamp(net.cfg.w_min, net.cfg.w_max);
                        }
                    }
                }
            }
            // (b) floor unclaimed at f
            if f > 0.0 {
                for &sid in &unc_idxs {
                    let s = &mut net.synapses[sid];
                    s.w = s.w.max(f);
                }
            }
            // (c) backstop: never exceed B (absorb clamp/floor drift)
            let total: f32 = incoming.iter().map(|&sid| {
                let s = &net.synapses[sid.idx()];
                if s.silent_ticks != u64::MAX && !s.inhibitory && !s.consolidated { s.w } else { 0.0 }
            }).sum();
            if total > b {
                let mut claimed_work = 0.0f32;
                for &sid in &incoming {
                    let s = &net.synapses[sid.idx()];
                    if s.silent_ticks != u64::MAX && !s.inhibitory && !s.consolidated && s.track != 2 {
                        claimed_work += s.w;
                    }
                }
                if claimed_work > 1e-9 {
                    let excess = total - b;
                    let factor = ((claimed_work - excess) / claimed_work).max(0.0);
                    for sid in incoming.iter().copied() {
                        let s = &mut net.synapses[sid.idx()];
                        if s.silent_ticks == u64::MAX || s.inhibitory || s.consolidated || s.track == 2 {
                            continue;
                        }
                        s.w = (s.w * factor).clamp(net.cfg.w_min, net.cfg.w_max);
                    }
                }
            }
        }
    }

    /// Phase II-A D-core (protocol §6): per-neuron context update from the
    /// JUST-FINISHED window's per-channel delivered-current vectors.
    /// Deterministic; consumes NO RNG. Skipped entirely when d_core is
    /// off (identity).
    /// Called after a NEW neuron is appended to `net` (structural birth,
    /// E4-family): grow the internal bookkeeping arrays so subsequent
    /// per-tick/window accesses stay in bounds. Without this, a birth
    /// desyncs V2Plasticity's vecs from net.neurons (index OOB at window).
    pub fn on_neuron_appended(&mut self, net: &Network) {
        let n = net.neurons.len();
        let k = dcore_tracks();
        self.fired.push(false);
        self.res_ip.push(0.0);
        self.res_iw.push(0.0);
        self.res_ip_t.extend(std::iter::repeat(0.0).take(k));
        self.res_iw_t.extend(std::iter::repeat(0.0).take(k));
        self.ctx_acc.extend(std::iter::repeat(0.0).take(24));
        self.cur_ctx.push(0);
        self.res_gate.push(1.0);
        self.fired_channels.push(Default::default());
        self.candidates.push(Vec::new());
        self.live_e.push(0);
        self.live_i.push(0);
        let _ = n;
    }

    fn ctx_update(&mut self, net: &mut Network, _tick: Tick) {
        let k = dcore_tracks();
        let dims = 24usize;
        for i in 0..net.neurons.len() {
            if net.neurons[i].class == NeuronClass::Input || net.neurons[i].class == NeuronClass::Inhibitory {
                continue;
            }
            // protocol §6.2a: no working input current delivered => no update
            let w_work = self.res_iw_t[i * k] + self.res_iw_t[i * k + 1];
            if w_work <= 0.0 {
                continue;
            }
            let row = &self.ctx_acc[i * dims..(i + 1) * dims];
            let mut ctx = 0u8;
            let mut best = -1.0f32;
            let protos = net.neurons[i].ctx_protos.clone(); // local copy (borrow-safe)
            if protos.is_empty() {
                // bootstrap: both prototypes zero => novel bind to lower
                // protected track (tie rule), hard set
                let pt = protected_track_choice(net, i, k);
                ctx = pt;
                let mut v = net.neurons[i].ctx_protos.clone();
                if v.len() < k * dims { v.resize(k * dims, 0.0); }
                v[pt as usize * dims..(pt as usize + 1) * dims].copy_from_slice(row);
                net.neurons[i].ctx_protos = v;
            } else {
                for t in 0..k {
                    let p = &protos[t * dims..(t + 1) * dims];
                    let sval = cosine_row(p, row);
                    if sval > best {
                        best = sval;
                        ctx = t as u8;
                    }
                }
                if best >= dcore_theta_sim() {
                    // reuse: soft update winning prototype
                    let mut v = net.neurons[i].ctx_protos.clone();
                    if v.len() < k * dims { v.resize(k * dims, 0.0); }
                    for d in 0..dims {
                        let idx = ctx as usize * dims + d;
                        v[idx] += dcore_alpha_p() * (row[d] - v[idx]);
                    }
                    net.neurons[i].ctx_protos = v;
                } else {
                    // novel bind: hard set on the less-protected track
                    let pt = protected_track_choice(net, i, k);
                    ctx = pt;
                    let mut v = net.neurons[i].ctx_protos.clone();
                    if v.len() < k * dims { v.resize(k * dims, 0.0); }
                    v[pt as usize * dims..(pt as usize + 1) * dims].copy_from_slice(row);
                    net.neurons[i].ctx_protos = v;
                }
            }
            self.cur_ctx[i] = ctx;
            // Phase II-AR claim rule (§1.3): unclaimed (track 2) incoming
            // working synapses whose PRE fired this window join the post's
            // current context. Deterministic; first-fire-only (once
            // 0/1, never back to 2). Afferent + recurrent.
            if self.d_claim_enabled() {
                let incoming: Vec<SynapseId> = net.incoming[i].clone();
                for sid in incoming {
                    let s = &net.synapses[sid.idx()];
                    if s.track != 2 || s.inhibitory || s.silent_ticks == u64::MAX {
                        continue;
                    }
                    if self.fired.get(s.pre.idx()).copied().unwrap_or(false) {
                        net.synapses[sid.idx()].track = ctx;
                    }
                }
            }
        }
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

        // --- Phase II-A D-core: context update from the JUST-FINISHED
        // window, BEFORE any per-track budget/pass runs (protocol §6.2) ---
        if self.d_core_enabled() {
            self.ctx_update(net, tick);
            self.sparse_commit(net);
            self.ctx_acc.fill(0.0);
            self.res_ip_t.fill(0.0);
            self.res_iw_t.fill(0.0);
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
                if net.neurons[post].class == NeuronClass::Input || net.neurons[post].class == NeuronClass::Inhibitory {
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
        // sparse-commit: per-post committed track so the unclaimed
        // churn-exemption is lifted for other-track afferents of committed
        // neurons (dropout reaches M4). None when sparse off.
        let committed: std::collections::HashMap<usize, u8> = if self.d_sparse_enabled() {
            (0..net.neurons.len()).filter_map(|p| {
                self.committed_track(net, p).map(|t| (p, t))
            }).collect()
        } else {
            Default::default()
        };
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
            if self.d_claim_enabled() && s.track == 2 {
                // E: unclaimed substrate is churn-exempt (floor), UNLESS a
                // sparse-committed neuron has claimed the other track (then
                // this unclaimed afferent is other-track -> prune-eligible).
                let lift = self.d_sparse_enabled()
                    && committed.get(&s.post.idx()).map_or(false,
                        |&t| s.track != t);
                if !lift {
                    self.low_windows[sid] = 0;
                    continue;
                }
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
                if self.d_core_enabled() {
                    // Phase II-A protocol §7: M3-born synapse receives the
                    // neuron's CURRENT window context tag (deterministic;
                    // initial wiring keeps the default 0).
                    net.synapses[syn.idx()].track = self.cur_ctx[post_id.idx()];
                }
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
        // Phase II-A D-core (protocol §4/§7): per-track normalization.
        // Working target per populated track T_t = (t_e - P_tot)/n_pop
        // (V2.3 capacity-matched family; total <= t_e ALWAYS); consolidated
        // synapses excluded exactly as the CLLA branch below.
        if self.d_core_enabled() {
            if self.d_claim_enabled() {
                // Phase II-AR candidate E (§1.4): TWO-REGIME normalization.
                // Claimed working mass is capacity-matched per track to
                // (B - floor_total)/n_claimed; unclaimed mass is floored at
                // min(theta_prune, B/n_unc) (budget-capped churn-exempt
                // floor), so the surviving dormant substrate is preserved
                // WITHOUT allowing the floor to break the invariant
                // sum_working <= B. A post-normalize backstop rescales the
                // claimed set to absorb any clamp drift. Deterministic.
                self.normalize_claim(net);
            } else {
                let k = dcore_tracks();
                for post in 0..net.neurons.len() {
                    if net.neurons[post].class == NeuronClass::Input || net.neurons[post].class == NeuronClass::Inhibitory {
                        continue;
                    }
                    let incoming: Vec<SynapseId> = net.incoming[post].clone();
                    let post_id = NeuronId(post as u32);
                    let p_tot = self.consolidated_mass(net, post_id);
                    let mut w_sum = [0.0f32; 2];
                    for &sid in &incoming {
                        let s = &net.synapses[sid.idx()];
                        if s.silent_ticks != u64::MAX && !s.inhibitory && !s.consolidated {
                            let t = (s.track as usize).min(k - 1);
                            w_sum[t] += s.w;
                        }
                    }
                    let populated: Vec<usize> = (0..k).filter(|&t| w_sum[t] > 0.0).collect();
                    if populated.is_empty() {
                        continue;
                    }
                    let target = self.params.t_e - p_tot;
                    if target <= 0.0 {
                        continue;
                    }
                    let per_track = target / populated.len() as f32;
                    for &t in &populated {
                        if (w_sum[t] - per_track).abs() < 1e-9 {
                            continue;
                        }
                        let factor = per_track / w_sum[t];
                        for sid in incoming.iter().copied() {
                            let s = &mut net.synapses[sid.idx()];
                            if s.silent_ticks == u64::MAX || s.inhibitory || s.consolidated {
                                continue;
                            }
                            if (s.track as usize).min(k - 1) == t {
                                s.w = (s.w * factor).clamp(net.cfg.w_min, net.cfg.w_max);
                            }
                        }
                    }
                }
            }
            return;
        }
        // CLLA (docs/anima-clla-protocol.md §3.7): consolidated synapses are
        // excluded from normalization. Working budget = t_e - P; factor =
        // (t_e - P)/W applied to working (unconsolidated) only, clamped to
        // [w_min, w_max]. When flag off this branch is never entered
        // (identity). Protocol freezes m2_buckets=1, so CLLA takes
        // precedence whenever assembly_protect is set.
        if self.params.assembly_protect {
            for post in 0..net.neurons.len() {
                if net.neurons[post].class == NeuronClass::Input || net.neurons[post].class == NeuronClass::Inhibitory {
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
                if net.neurons[post].class == NeuronClass::Input || net.neurons[post].class == NeuronClass::Inhibitory {
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
            if net.neurons[post].class == NeuronClass::Input || net.neurons[post].class == NeuronClass::Inhibitory {
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
            // d_ing: fixed shunting from INs (Inhibitory-class pre) is a
            // separate structural mechanism, not M6 plastic inhibition.
            if net.neurons[pre.idx()].class == crate::network::NeuronClass::Inhibitory {
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
        if pre == post_id || neur.class == NeuronClass::Input || neur.class == NeuronClass::Inhibitory || connected(pre) {
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
        // Phase III Level-4 `d_ing`: fixed shunting from inhibitory
        // INs (pre = Inhibitory class) is a separate structural
        // mechanism, NOT the plastic M6 anti-Hebbian budget. Excluded.
        if net.neurons[s.pre.idx()].class == crate::network::NeuronClass::Inhibitory {
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