//! CLLA F4 causal stability audit — read-only trajectory reconstruction.
//! Per snapshot (1 kHz grid): rate (P2 quantity), exc mass, protected P,
//! working W (+ distortion vs target t_e-P), consolidated count, cumulative
//! permanence events, inhibitory mass, per-neuron max P. Plus first-event
//! markers (first permanence, first spk-crossing of 50 Hz mean).
use std::collections::BTreeMap;

fn main() {
    for dir in std::env::args().skip(1) {
        let snaps = anima_telemetry::recorder::read_snapshots(
            &std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let reader = anima_telemetry::TelemetryReader::open(
            &std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut spikes: BTreeMap<u64, u32> = BTreeMap::new();
        let mut perm_events: Vec<u64> = Vec::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(env) = row.envelope("e") {
                    match &env.payload {
                        anima_telemetry::events::Payload::Spike { n } => {
                            if n.0 >= 24 && n.0 < 76 {
                                *spikes.entry(row.t).or_default() += 1;
                            }
                        }
                        anima_telemetry::events::Payload::SynapseCreated { syn: _, pre: _, post: _, w: _, reason } => {
                            if reason.trigger == "candidate-permanence" {
                                perm_events.push(row.t);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        perm_events.sort_unstable();
        let t_e = 0.8_f32;
        let cap = 0.75_f32 * t_e;
        println!("RUN {}", dir.rsplit('/').next().unwrap());
        println!("  first_permanence={:?} n_perm={}", perm_events.first(), perm_events.len());
        let mut cum_perm = 0usize;
        let mut first_p = None;
        let mut first_spk50 = None;
        for (i, s) in snaps.iter().enumerate() {
            while cum_perm < perm_events.len() && perm_events[cum_perm] <= s.tick {
                cum_perm += 1;
            }
            // P2 quantity: mean rate_hz over class=internal neurons.
            let internal: Vec<f32> = s.neurons.iter()
                .filter(|n| n.class == "internal" && !n.retired)
                .map(|n| n.rate_hz.unwrap_or(0.0))
                .collect();
            let mean_rate = if internal.is_empty() { 0.0 } else {
                internal.iter().sum::<f32>() / internal.len() as f32
            };
            let mut p = vec![0.0f32; 64]; // post ids 24..64 internal + outputs up to 76
            let mut wsum = vec![0.0f32; 64];
            let mut cons_cnt = vec![0u32; 64];
            let mut inh_mass = 0.0f32;
            for syn in &s.synapses {
                if syn.plastic {
                    if syn.consolidated {
                        if syn.post >= 24 && syn.post < 88 { p[syn.post as usize - 24] += syn.w.unwrap_or(0.0); }
                        if syn.post >= 24 && syn.post < 88 { cons_cnt[syn.post as usize - 24] += 1; }
                    } else if syn.post >= 24 && syn.post < 88 {
                        wsum[syn.post as usize - 24] += syn.w.unwrap_or(0.0);
                    }
                } else if syn.post >= 24 {
                    inh_mass += syn.w.unwrap_or(0.0);
                }
            }
            let p_total: f32 = p.iter().sum();
            let w_total: f32 = wsum.iter().sum();
            let p_max = p.iter().cloned().fold(0.0f32, f32::max);
            let n_cons: u32 = cons_cnt.iter().sum();
            let exc_total = p_total + w_total;
            // working distortion: mean |W - (t_e - P)| / (t_e - P) over neurons with W>0
            let mut dist_sum = 0.0f32; let mut dist_n = 0u32;
            for j in 0..64 {
                let target = (t_e - p[j]).max(0.0);
                if wsum[j] > 0.0 && target > 0.0 {
                    dist_sum += (wsum[j] - target).abs() / target;
                    dist_n += 1;
                }
            }
            let dist = if dist_n > 0 { dist_sum / dist_n as f32 } else { 0.0 };
            // spike count in (tick-1000, tick]
            let mut spk1k = 0u32;
            for t in s.tick.saturating_sub(1000)..=s.tick {
                spk1k += spikes.get(&t).copied().unwrap_or(0);
            }
            if first_p.is_none() && p_total > 0.0 { first_p = Some(s.tick); }
            if first_spk50.is_none() && mean_rate > 50.0 { first_spk50 = Some(s.tick); }
            let tsec = format!("{:.3}", s.tick as f64 / 1000.0);
            let tag = if s.tick == 84000 { "driveend".to_string() } else if i % 5 == 0 { tsec.clone() } else { String::new() };
            println!("T\t{}\t{}\t{mean_rate:.2}\t{p_total:.4}\t{w_total:.4}\t{exc_total:.4}\t{p_max:.4}\t{n_cons}\t{cum_perm}\t{inh_mass:.4}\t{dist:.3}\t{spk1k}\t{tag}", s.tick, tsec);
        }
        println!("  first_P>0={:?} first_mean_rate>50={:?} final_Pmax={:.4} n_cons_final={}",
            first_p, first_spk50,
            snaps.last().map(|s| s.synapses.iter()
                .filter(|x| x.consolidated && x.plastic && x.post >= 24 && x.post < 88)
                .map(|x| x.w.unwrap_or(0.0)).sum::<f32>()).unwrap_or(0.0),
            snaps.last().map(|s| s.synapses.iter().filter(|x| x.consolidated).count()).unwrap_or(0));
        println!();
    }
}