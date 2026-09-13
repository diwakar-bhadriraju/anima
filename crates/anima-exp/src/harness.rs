//! Run loop (step 9): controls → env.step → net.step → plasticity →
//! structural → resources → recorder + broadcaster. Speed gate sleeps to
//! hold ticks/sec; wall-clock never enters events (D5).

use std::path::Path;
use std::time::Instant;

use anima_core::network::{NeuronClass, Network, NetworkConfig};
use anima_core::plasticity::{silent_synapse_pass, stdp_tick, StdpParams, Traces};
use anima_core::resources::{ResourceConfig, ResourceMonitor};
use anima_core::structural::{make_trigger, Signals, StructuralMonitor};
use anima_telemetry::events::{EventBuilder, Payload, ReasonPayload};
use anima_telemetry::recorder::{NeuronState, Recorder, SynapseState};
use anima_viz::{ServerHandle, UiNeuron, UiSynapse};

use crate::config::ExpConfig;
use crate::env::Environment;

/// Instrumentation state (U6a): novelty + prediction error computed from
/// input-channel rates against a running EWMA — measurement, not mechanism.
struct Instrumentation {
    n_channels: usize,
    /// 100-tick spike counts per channel.
    counts: Vec<u64>,
    /// EWMA per-channel rate (Hz).
    ewma: Vec<f32>,
    ewma_var: Vec<f32>,
    warmup_ticks: u64,
    last_error: f32,
    pe_mean: f32,
    pe_std: f32,
    novelty_threshold_sigma: f32,
}

impl Instrumentation {
    fn new(n_channels: usize) -> Self {
        Self {
            n_channels,
            counts: vec![0; n_channels],
            ewma: vec![0.0; n_channels],
            ewma_var: vec![1.0; n_channels],
            warmup_ticks: 10_000,
            last_error: 0.0,
            pe_mean: 0.0,
            pe_std: 1.0,
            novelty_threshold_sigma: 2.0,
        }
    }

    /// Returns (prediction_error, novelty_fired).
    fn tick(&mut self, frame: &anima_core::network::InputFrame) -> (f32, bool) {
        for &ch in &frame.spikes {
            self.counts[ch.0 as usize] += 1;
        }
        let mut novelty = false;
        if frame.tick.0 % 100 == 0 && frame.tick.0 > 0 {
            // rates over last 100 ms
            let rates: Vec<f32> = self
                .counts
                .iter()
                .map(|&c| c as f32 / 0.1)
                .collect();
            self.counts.iter_mut().for_each(|c| *c = 0);
            // EWMA (alpha = 0.02, ~10 s horizon)
            let alpha = 0.02f32;
            let mut dev = 0.0;
            for i in 0..self.n_channels {
                let d = (rates[i] - self.ewma[i]).abs();
                dev += d;
                self.ewma[i] += alpha * (rates[i] - self.ewma[i]);
                self.ewma_var[i] += alpha * (d * d - self.ewma_var[i]);
            }
            self.last_error = dev / self.n_channels as f32;
            // running mean/std of error
            self.pe_mean += alpha * (self.last_error - self.pe_mean);
            self.pe_std = (self.pe_std + alpha * (self.last_error - self.pe_mean).powi(2)).max(1e-6).sqrt();
            if frame.tick.0 > self.warmup_ticks
                && self.last_error > self.pe_mean + self.novelty_threshold_sigma * self.pe_std.sqrt()
            {
                novelty = true;
            }
        }
        (self.last_error, novelty)
    }
}

pub struct RunOutcome {
    pub dir: std::path::PathBuf,
    pub reason: String,
}

/// Execute the configured experiment. `live` attaches the viz server;
/// headless runs at max speed.
pub fn run(cfg: ExpConfig, cfg_path: &Path, live: bool) -> std::io::Result<RunOutcome> {
    let seed = cfg.run.seed;
    let mut net = Network::new(
        NetworkConfig {
            connectivity: cfg.organism.connectivity,
            w_init: cfg.organism.w_init,
            amplitude: cfg.organism.amplitude,
            ..NetworkConfig::default()
        },
        cfg.organism.n_input_channels,
        cfg.organism.n_internal,
        cfg.organism.n_output,
        seed,
    );
    // Channel groups for viz/analysis.
    let letters = ["A", "B", "C", "D", "E", "F", "G", "H"];
    for (i, ch) in net.channels.iter_mut().enumerate() {
        let g = i / cfg.organism.group_size.max(1);
        ch.group = letters.get(g).map(|s| s.to_string()).unwrap_or_else(|| format!("G{g}"));
    }

    let env = Environment::new(cfg.clone(), seed);
    let mut traces = Traces::new(&net, cfg.plasticity.tau_plus_ms);
    let params = StdpParams {
        tau_plus: cfg.plasticity.tau_plus_ms,
        tau_minus: cfg.plasticity.tau_minus_ms,
        a_plus: cfg.plasticity.a_plus,
        a_minus: cfg.plasticity.a_minus,
        decay: cfg.plasticity.decay,
        w_min: cfg.plasticity.w_min,
        w_max: cfg.plasticity.w_max,
    };
    // E2: rule selection is the ONLY experimental variable. Both rules
    // share StdpParams and the same trace/silence machinery.
    let multiplicative = matches!(cfg.plasticity.rule.as_str(), "stdp-multiplicative");
    let stdp = anima_core::plasticity::PairwiseStdp::new(params);
    let _ = &stdp; // silence-prune thresholds live on this type (either arm)
    let mut structural = StructuralMonitor::default();
    structural.dormancy_rate_hz = cfg.structural.dormancy_rate_hz;
    structural.dormancy_ms = cfg.structural.dormancy_ms;
    structural.recovery_rate_hz = cfg.structural.recovery_rate_hz;
    structural.retirement_ms = cfg.structural.retirement_ms;
    structural.wiring_synapses = cfg.structural.wiring_synapses;
    let mut trigger = make_trigger(
        &cfg.structural.birth_trigger,
        cfg.structural.trigger_rate_hz,
        cfg.structural.trigger_sustained_ms,
    );
    let mut resources = ResourceMonitor::new(ResourceConfig {
        max_neurons: cfg.resources.max_neurons,
        max_synapses: cfg.resources.max_synapses,
        births_per_window: cfg.resources.births_per_window,
        runaway_rate_hz: cfg.resources.runaway_rate_hz,
        runaway_sustained_ms: cfg.resources.runaway_sustained_ms,
        fragmentation_min_component: cfg.resources.fragmentation_min_component,
        ..ResourceConfig::default()
    });
    let mut instr = Instrumentation::new(cfg.organism.n_input_channels);

    let mut env = env;
    let mut recorder = Recorder::create(Path::new("runs"), &cfg.run.exp_id)?;
    let dir = recorder.dir().to_path_buf();

    let config_hash = {
        use sha2::{Digest, Sha256};
        let raw = std::fs::read(cfg_path).unwrap_or_default();
        let mut h = Sha256::new();
        h.update(&raw);
        format!("{:x}", h.finalize())
    };
    let params_json = serde_json::json!({
        "n_input_channels": cfg.organism.n_input_channels,
        "group_size": cfg.organism.group_size,
        "n_internal": cfg.organism.n_internal,
        "n_output": cfg.organism.n_output,
        "connectivity": cfg.organism.connectivity,
        "w_init": cfg.organism.w_init,
        "amplitude": cfg.organism.amplitude,
        "rule": cfg.plasticity.rule,
        "a_plus": cfg.plasticity.a_plus,
        "a_minus": cfg.plasticity.a_minus,
        "tau_plus_ms": cfg.plasticity.tau_plus_ms,
        "tau_minus_ms": cfg.plasticity.tau_minus_ms,
        "birth_trigger": cfg.structural.birth_trigger,
        "max_neurons": cfg.resources.max_neurons,
        "max_synapses": cfg.resources.max_synapses,
    });

    let server: Option<ServerHandle> = if live {
        Some(anima_viz::start(cfg.run.viz_port, "run", &cfg.run.exp_id, env.duration()))
    } else {
        None
    };
    let mut speed = cfg.run.ticks_per_sec.max(1.0);
    let mut paused = false;
    let mut step_pending: u64 = 0;
    let started = Instant::now();
    let mut events: Vec<Payload> = Vec::new();
    let mut spike_batch: Vec<u32> = Vec::new();
    let mut recorder_error: Option<std::io::Error> = None;
    let mut end_reason: Option<String> = None;

    // Emit structural bootstrap: initial neurons + synapses.
    let builder = EventBuilder::new(&cfg.run.exp_id);
    let mut b = builder;
    {
        let mut ev = b.build(0, Payload::RunStarted {
            config_hash: config_hash.clone(),
            seed,
            params: params_json.clone(),
        });
        if let Err(e) = write_env(&mut recorder, &server, &mut ev, &mut events) {
            recorder_error = Some(e);
        }
    }
    for n in &net.neurons {
        let mut ev = b.build(
            0,
            Payload::NeuronCreated {
                n: n.id,
                reason: ReasonPayload::simple(if n.class == NeuronClass::Input { "input-boundary" } else { "initial-wiring" }),
            },
        );
        if let Err(e) = write_env(&mut recorder, &server, &mut ev, &mut events) {
            recorder_error = Some(e);
        }
    }
    for s in net.live_synapses() {
        let mut ev = b.build(
            0,
            Payload::SynapseCreated {
                syn: s.id,
                pre: s.pre,
                post: s.post,
                w: anima_telemetry::events::f32_json(s.w),
                reason: ReasonPayload::simple("initial-wiring"),
            },
        );
        if let Err(e) = write_env(&mut recorder, &server, &mut ev, &mut events) {
            recorder_error = Some(e);
        }
    }
    if let Some(sv) = &server {
        sv.send_json(serde_json::json!({"t": "run-params", "params": params_json}).to_string());
    }

    // Coalesced weight-delta accumulators.
    let mut unemit: Vec<f32> = vec![0.0; net.synapses.len()];

    let mut tick_index: u64 = 0;
    while end_reason.is_none() && recorder_error.is_none() {
        // Controls.
        if let Some(sv) = &server {
            while let Ok(op) = sv.ctrl_rx.try_recv() {
                match op {
                    anima_viz::CtrlOp::Pause => paused = true,
                    anima_viz::CtrlOp::Resume => {
                        paused = false;
                        step_pending = 0;
                    }
                    anima_viz::CtrlOp::Step { n } => {
                        // advance exactly n ticks from pause (plan D5)
                        step_pending = step_pending.saturating_add(n);
                    }
                    anima_viz::CtrlOp::Speed { factor } => speed = factor.clamp(1.0, 10_000.0),
                    anima_viz::CtrlOp::Seek { .. } => {}
                }
            }
            if paused && step_pending == 0 {
                std::thread::sleep(std::time::Duration::from_millis(10));
                continue;
            }
            if step_pending > 0 {
                step_pending -= 1;
                if step_pending == 0 {
                    paused = true; // hold after stepping
                }
            }
        }

        // 1. environment
        let (frame, presented) = env.step();

        // 2. network dynamics
        let step = net.step(&frame);

        // 3. traces + plasticity (STDP per tick, gate = 1.0 always-on, U4a)
        traces.step(&net, &step.spikes);
        let changes = if multiplicative {
            anima_core::plasticity::stdp_tick_multiplicative(
                &params, &mut net, &traces, &step.spikes, 1.0,
            )
        } else {
            stdp_tick(&params, &mut net, &traces, &step.spikes, 1.0)
        };
        for c in changes {
            unemit[c.synapse.idx()] += c.after - c.before;
        }
        // Coalesced emission when |Δw| > 0.01: telemetry keeps every event;
        // the wire gets nothing here (thousands/s would starve WS clients) —
        // viz weight state syncs via the 10-tick VizState snapshot.
        for (i, acc) in unemit.iter_mut().enumerate() {
            if acc.abs() > 0.01 {
                let sid = anima_core::network::SynapseId(i as u32);
                if net.synapse_alive(sid) {
                    let payload = if *acc > 0.0 {
                        Payload::SynapseStrengthened { syn: sid, delta: anima_telemetry::events::f32_json(*acc) }
                    } else {
                        Payload::SynapseWeakened { syn: sid, delta: anima_telemetry::events::f32_json(*acc) }
                    };
                    let ev = b.build(net.tick.0, payload);
                    if let Err(e) = recorder.write(&ev) {
                        recorder_error = Some(e);
                    }
                }
                *acc = 0.0;
            }
        }
        // Passive decay per tick.
        for s in net.live_synapses_mut() {
            s.w = (s.w - params.decay).max(params.w_min);
        }

        // 4. structural pass (dormancy lifecycle; birth via trigger)
        let sig = Signals {
            prediction_error: instr.pe_mean,
            pe_mean: instr.pe_mean,
            pe_std: instr.pe_std.sqrt(),
            novelty: 0.0,
        };
        let se = structural.step(&mut net, trigger.as_mut(), &sig);
        for (id, reason) in &se.births {
            let mut ev = b.build(
                net.tick.0,
                Payload::NeuronCreated {
                    n: *id,
                    reason: ReasonPayload::from_core(reason),
                },
            );
            if let Err(e) = write_env(&mut recorder, &server, &mut ev, &mut events) {
                recorder_error = Some(e);
            }
        }
        for id in &se.reactivated {
            let mut ev = b.build(net.tick.0, Payload::NeuronReactivated { n: *id, reason: ReasonPayload::simple("rate-recovery") });
            if let Err(e) = write_env(&mut recorder, &server, &mut ev, &mut events) {
                recorder_error = Some(e);
            }
        }
        for id in &se.dormant {
            let mut ev = b.build(
                net.tick.0,
                Payload::NeuronDormant {
                    n: *id,
                    reason: ReasonPayload::simple("sustained-quiescence"),
                },
            );
            if let Err(e) = write_env(&mut recorder, &server, &mut ev, &mut events) {
                recorder_error = Some(e);
            }
        }
        for id in &se.retired {
            for sid in structural.retire_neuron(&mut net, *id) {
                let mut ev = b.build(
                    net.tick.0,
                    Payload::SynapsePruned {
                        syn: sid,
                        reason: ReasonPayload::simple("neuron-retired"),
                    },
                );
                if let Err(e) = write_env(&mut recorder, &server, &mut ev, &mut events) {
                    recorder_error = Some(e);
                }
            }
            let mut ev = b.build(
                net.tick.0,
                Payload::NeuronRetired {
                    n: *id,
                    reason: ReasonPayload::simple("sustained-quiescence"),
                },
            );
            if let Err(e) = write_env(&mut recorder, &server, &mut ev, &mut events) {
                recorder_error = Some(e);
            }
        }
        // Birth admission via resource caps.
        if !se.births.is_empty() {
            if let Err(f) = resources.admit_birth(&net) {
                end_reason = Some(format!("failure:{}", f.kind));
                let mut ev = b.build(
                    net.tick.0,
                    Payload::Failure { kind: f.kind.clone(), detail: f.detail.clone() },
                );
                let _ = write_env(&mut recorder, &server, &mut ev, &mut events);
            }
        }

        // 5. silent-synapse pruning (every 1000 ticks — cheap enough every tick
        //    but bounded here to reduce telemetry noise).
        if net.tick.0 % 1000 == 0 {
            let prunes = silent_synapse_pass(
                &mut net,
                stdp.silence_w,
                stdp.silence_ticks,
                stdp.min_age_ticks,
            );
            for (sid, reason) in prunes {
                let mut ev = b.build(
                    net.tick.0,
                    Payload::SynapsePruned {
                        syn: sid,
                        reason: ReasonPayload::simple(reason),
                    },
                );
                if let Err(e) = write_env(&mut recorder, &server, &mut ev, &mut events) {
                    recorder_error = Some(e);
                }
            }
        }

        // 6. instrumentation
        let (pe, novelty) = instr.tick(&frame);
        if net.tick.0 % 100 == 0 {
            let ev = b.build(
                net.tick.0,
                Payload::PredictionError { value: anima_telemetry::events::f32_json(pe) },
            );
            // telemetry-only (10/s wire frames still too chatty for WS)
            if let Err(e) = recorder.write(&ev) {
                recorder_error = Some(e);
            }
        }
        if novelty {
            let mut ev = b.build(
                net.tick.0,
                Payload::NoveltySignal { value: anima_telemetry::events::f32_json(pe) },
            );
            if let Err(e) = write_env(&mut recorder, &server, &mut ev, &mut events) {
                recorder_error = Some(e);
            }
        }

        // 7. spike + output events
        for &id in &step.spikes {
            let mut ev = b.build(net.tick.0, Payload::Spike { n: id });
            if let Err(e) = write_env(&mut recorder, &server, &mut ev, &mut events) {
                recorder_error = Some(e);
            }
            spike_batch.push(id.0);
        }
        for &id in &step.output_spikes {
            let mut ev = b.build(net.tick.0, Payload::OutputActivity { n: id });
            if let Err(e) = write_env(&mut recorder, &server, &mut ev, &mut events) {
                recorder_error = Some(e);
            }
        }

        // 8. presented-marker
        if let Some((stage, pattern, _dur)) = presented {
            let mut ev = b.build(
                net.tick.0,
                Payload::StimulusPresented {
                    pattern_id: pattern,
                    stage,
                },
            );
            if let Err(e) = write_env(&mut recorder, &server, &mut ev, &mut events) {
                recorder_error = Some(e);
            }
        }

        // 9. resources
        let (sample, failure) = resources.step(&net, step.spikes.len());
        if let Some(s) = sample {
            let mut ev = b.build(
                net.tick.0,
                Payload::ResourceUsage {
                    neurons: s.neurons as u64,
                    synapses: s.synapses as u64,
                    spikes_window: s.spikes_this_window,
                    metabolic_cost: anima_telemetry::events::f32_json(s.metabolic_cost),
                },
            );
            if let Err(e) = write_env(&mut recorder, &server, &mut ev, &mut events) {
                recorder_error = Some(e);
            }
        }
        if let Some(f) = failure {
            let mut ev = b.build(
                net.tick.0,
                Payload::Failure { kind: f.kind.clone(), detail: f.detail.clone() },
            );
            let _ = write_env(&mut recorder, &server, &mut ev, &mut events);
            end_reason = Some(format!("failure:{}", f.kind));
        }

        // 10. snapshot every 1000 ticks
        if net.tick.0 % 1000 == 0 {
            let snap = network_snapshot(&net);
            if let Err(e) = recorder.write_snapshot(&snap) {
                recorder_error = Some(e);
            }
        }

        // 11. viz update every 10 ticks (spike batch flush + state)
        if let Some(sv) = &server {
            if net.tick.0 % 10 == 0 {
                sv.send_json(
                    serde_json::json!({"msg": "spikes", "t": net.tick.0, "ids": spike_batch})
                        .to_string(),
                );
                spike_batch.clear();
                let neurons: Vec<UiNeuron> = net
                    .neurons
                    .iter()
                    .filter(|n| !n.retired)
                    .map(|n| UiNeuron {
                        id: n.id.0,
                        cls: class_str(n.class).into(),
                        group: if n.class == NeuronClass::Input {
                            net.channels[n.channel.unwrap().0 as usize].group.clone()
                        } else {
                            String::new()
                        },
                        rate: n.rate_hz,
                        state: if n.retired { "retired" } else if n.dormant_since.is_some() { "dormant" } else { "normal" }.into(),
                    })
                    .collect();
                let synapses: Vec<UiSynapse> = net
                    .live_synapses()
                    .map(|s| UiSynapse { id: s.id.0, src: s.pre.0, dst: s.post.0, w: s.w })
                    .collect();
                sv.update_state(net.tick.0, paused, speed, neurons, synapses);
            }
        } else {
            spike_batch.clear();
        }

        // 12. pacing (live only)
        if server.is_some() {
            let target_ns_per_tick = 1_000_000_000.0 / speed as f64;
            let done = (tick_index + 1) as f64;
            let target = done * target_ns_per_tick;
            let actual = started.elapsed().as_secs_f64() * 1e9;
            if target > actual {
                std::thread::sleep(std::time::Duration::from_nanos((target - actual) as u64));
            }
        }

        // Curriculum complete?
        if env.tick >= env.duration() + 1 {
            end_reason = Some("curriculum-complete".into());
        }
        tick_index += 1;
    }

    // RunEnded + flush.
    let mut ev = b.build(
        net.tick.0,
        Payload::RunEnded {
            reason: end_reason.clone().unwrap_or_else(|| "aborted".into()),
        },
    );
    let _ = write_env(&mut recorder, &server, &mut ev, &mut events);
    let _ = recorder.flush();
    // Recorder error keeps partial file (§22) — reported by caller.

    let reason = if let Some(e) = recorder_error {
        eprintln!("recorder error (partial telemetry preserved): {e}");
        "recorder-error".to_string()
    } else {
        end_reason.unwrap_or_else(|| "aborted".into())
    };
    Ok(RunOutcome { dir, reason })
}

fn write_env(
    recorder: &mut Recorder,
    server: &Option<ServerHandle>,
    env: &mut anima_telemetry::events::Envelope,
    _events: &mut Vec<Payload>,
) -> std::io::Result<()> {
    recorder.write(env)?;
    if let Some(sv) = server {
        sv.broadcast_event(env);
    }
    Ok(())
}

fn class_str(c: NeuronClass) -> &'static str {
    match c {
        NeuronClass::Input => "input",
        NeuronClass::Internal => "internal",
        NeuronClass::Output => "output",
    }
}

fn network_snapshot(net: &Network) -> anima_telemetry::recorder::NetworkStateSnapshot {
    anima_telemetry::recorder::NetworkStateSnapshot {
        tick: net.tick.0,
        neurons: net
            .neurons
            .iter()
            .map(|n| NeuronState {
                id: n.id.0,
                class: class_str(n.class).into(),
                v: anima_telemetry::events::f32_json(n.v),
                rate_hz: anima_telemetry::events::f32_json(n.rate_hz),
                dormant: n.dormant_since.is_some(),
                retired: n.retired,
                born: n.born.0,
            })
            .collect(),
        synapses: net
            .live_synapses()
            .map(|s| SynapseState {
                id: s.id.0,
                pre: s.pre.0,
                post: s.post.0,
                w: anima_telemetry::events::f32_json(s.w),
                plastic: s.plastic,
            })
            .collect(),
    }
}
