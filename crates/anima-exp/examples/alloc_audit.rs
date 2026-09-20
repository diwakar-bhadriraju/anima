//! CLLA allocation audit (read-only, 2026-09-21).
//! Per committed run:
//!   - S1 presentations (pattern, tick)
//!   - per presentation: mean over 52 neurons of input current from
//!     A-channels (pre<8), C-channels (8..16) delivered during the window,
//!     and per-neuron response spike count (protected-assembly activation)
//!   - per snapshot: per-neuron protected/working mass by group (A/C),
//!     headroom (cap - P), live consolidated afferent counts by group
//!   - cumulative candidate-permanence events bucketed by pre channel group
//! Output: PRES rows (per presentation) + SNAP rows + EVT rows.
use std::collections::BTreeMap;
use std::collections::BTreeSet;

fn main() {
    for dir in std::env::args().skip(1) {
        let snaps = anima_telemetry::recorder::read_snapshots(
            &std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let reader = anima_telemetry::TelemetryReader::open(
            &std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut pres: Vec<(String, u64)> = Vec::new();
        let mut spk: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
        let mut perm_events: Vec<(u64, u32)> = Vec::new(); // (tick, pre id)
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(env) = row.envelope("e") {
                    match &env.payload {
                        anima_telemetry::events::Payload::StimulusPresented { pattern_id, stage } => {
                            if stage == "S1" { pres.push((pattern_id.clone(), row.t)); }
                        }
                        anima_telemetry::events::Payload::Spike { n } => {
                            spk.entry(row.t).or_default().push(n.0);
                        }
                        anima_telemetry::events::Payload::SynapseCreated { syn: _, pre, post: _, w: _, reason } => {
                            if reason.trigger == "candidate-permanence" {
                                perm_events.push((row.t, pre.0));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        pres.sort_by_key(|p| p.1);
        println!("RUN {}", dir.rsplit('/').next().unwrap());
        // Snapshot weights: per-neuron A/C protected & working mass + counts.
        // aux: for input-current computation we need per-neuron afferent weights
        // grouped by channel; use the LATEST snapshot before each presentation.
        let mut snap_by_tick = BTreeMap::new();
        for s in &snaps {
            let mut pa = vec![0.0f32; 52]; let mut pc = vec![0.0f32; 52];
            let mut wa = vec![0.0f32; 52]; let mut wc = vec![0.0f32; 52];
            let mut pa_cnt = vec![0u32; 52]; let mut pc_cnt = vec![0u32; 52];
            let mut wa_cnt = vec![0u32; 52]; let mut wc_cnt = vec![0u32; 52];
            for syn in &s.synapses {
                if !syn.plastic || syn.post < 24 || syn.post >= 76 { continue; }
                let i = syn.post as usize - 24;
                let w = syn.w.unwrap_or(0.0);
                if syn.consolidated {
                    if syn.pre < 8 { pa[i] += w; pa_cnt[i] += 1; }
                    else if syn.pre < 16 { pc[i] += w; pc_cnt[i] += 1; }
                } else {
                    if syn.pre < 8 { wa[i] += w; wa_cnt[i] += 1; }
                    else if syn.pre < 16 { wc[i] += w; wc_cnt[i] += 1; }
                }
            }
            snap_by_tick.insert(s.tick, (pa, pc, wa, wc, pa_cnt, pc_cnt, wa_cnt, wc_cnt));
        }
        let cap = 0.75f32 * 0.8f32;
        let mut last_snap_i = 0usize;
        let mut cum_perm_a = 0usize; let mut cum_perm_c = 0usize; let mut cum_perm_o = 0usize;
        for (pi, (pat, t0)) in pres.iter().enumerate() {
            // advance snapshot cursor to latest snapshot <= t0
            while last_snap_i + 1 < snaps.len() && snaps[last_snap_i + 1].tick <= *t0 {
                last_snap_i += 1;
            }
            while cum_perm_a + cum_perm_c + cum_perm_o < perm_events.len()
                && perm_events[cum_perm_a + cum_perm_c + cum_perm_o].0 <= *t0
            {
                let e = perm_events[cum_perm_a + cum_perm_c + cum_perm_o];
                if e.1 < 8 { cum_perm_a += 1; }
                else if e.1 < 16 { cum_perm_c += 1; }
                else { cum_perm_o += 1; }
            }
            // fired channels in [t0, t0+500)
            let mut fired: BTreeSet<u32> = BTreeSet::new();
            for t in *t0..t0 + 500 {
                if let Some(ids) = spk.get(&t) {
                    for &n in ids { if n < 24 { fired.insert(n); } }
                }
            }
            // per-neuron response + input current (amplitude * w over fired channels)
            let snap = &snaps[last_snap_i];
            // rebuild per-neuron per-channel weights from this snapshot quickly:
            // we only need sums over fired channels; iterate synapses once per presentation.
            let mut i_a = vec![0.0f32; 52];
            let mut i_c = vec![0.0f32; 52];
            let amp = 52.0f32;
            for syn in &snap.synapses {
                if !syn.plastic || syn.post < 24 || syn.post >= 76 { continue; }
                if fired.contains(&syn.pre) {
                    let i = syn.post as usize - 24;
                    let w = syn.w.unwrap_or(0.0);
                    if syn.pre < 8 { i_a[i] += amp * w; }
                    else if syn.pre < 16 { i_c[i] += amp * w; }
                }
            }
            let mut resp = vec![0u32; 52];
            for t in *t0..t0 + 500 {
                if let Some(ids) = spk.get(&t) {
                    for &n in ids { if n >= 24 && n < 76 { resp[n as usize - 24] += 1; } }
                }
            }
            let m = |v: &Vec<f32>| v.iter().sum::<f32>() / 52.0f32;
            let mr = |v: &Vec<u32>| v.iter().sum::<u32>() as f32 / 52.0f32;
            let (pa, pc, wa, wc, _, _, _, _) = &snap_by_tick[&snaps[last_snap_i].tick];
            let mPA = m(pa); let mPC = m(pc); let mWA = m(wa); let mWC = m(wc);
            println!("PRES\t{}\t{}\t{}\t{mPA:.4}\t{mPC:.4}\t{mWA:.4}\t{mWC:.4}\t{:.4}\t{:.4}\t{:.2}\t{}\t{}\t{}",
                pat, pi + 1, t0, m(&i_a), m(&i_c), mr(&resp),
                cum_perm_a, cum_perm_c, cum_perm_o);
        }
        // SNAP rows: headroom and consolidated counts per snapshot (every snapshot)
        for s in &snaps {
            if let Some((pa, pc, wa, wc, pac, pcc, wac, wcc)) = snap_by_tick.get(&s.tick) {
                let psum: f32 = pa.iter().zip(pc).map(|(a, c)| a + c).sum();
                let headroom = (cap * 52.0f32 - psum) / 52.0f32;
                println!("SNAP\t{}\t{headroom:.4}\t{}\t{}\t{}\t{}\t{}\t{}",
                    s.tick,
                    pac.iter().sum::<u32>(), pcc.iter().sum::<u32>(),
                    wac.iter().sum::<u32>(), wcc.iter().sum::<u32>(),
                    pa.iter().sum::<f32>(), pc.iter().sum::<f32>());
            }
        }
        println!();
    }
}