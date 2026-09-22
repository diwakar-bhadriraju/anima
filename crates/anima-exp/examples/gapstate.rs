//! gapstate: Phase III Level-4 feasibility probe — does the E-nogain
//! substrate hold ANY predecessor-distinct internal state across the
//! 1500 ms inter-presentation gap?
//!
//! For an alternating (il) run: for each presentation of pair (p, p'),
//! count internal spikes in [p_end, p'=next_start) (the gap), split by
//! the predecessor pattern. If the gap is essentially silent OR the
//! gap-activity is not predecessor-distinct, no persistence primitive
//! exists and one is required for temporal prediction.
use anima_telemetry::TelemetryReader;
use anima_telemetry::events::Payload;

fn main() {
    for dir in std::env::args().skip(1) {
        let reader = TelemetryReader::open(&std::path::Path::new(&dir).join("telemetry")).unwrap();
        let idx = reader.chunk_index();
        let mut pres: Vec<(u64, String, String)> = Vec::new();
        let mut n_gap: Vec<(u64, u32)> = Vec::new();
        for c in 0..idx.len() {
            for row in reader.chunk_rows(c).unwrap() {
                if let Ok(e) = row.envelope("e") {
                    match &e.payload {
                        Payload::StimulusPresented { pattern_id, stage } => {
                            pres.push((row.t, pattern_id.clone(), stage.clone()));
                        }
                        Payload::Spike { n } => { if n.0 >= 24 && n.0 < 76 { n_gap.push((row.t, n.0)); } }
                        _ => {}
                    }
                }
            }
        }
        pres.sort_by_key(|p| p.0);
        let name = dir.rsplit('/').next().unwrap().to_string();
        println!("H\t{name}");
        // gaps: presentation i ends at i.t+500; next starts at i+1.t
        let mut by_pred: std::collections::BTreeMap<String, Vec<u32>> = Default::default();
        let mut total_gap_spikes = 0u64;
        let mut n_gaps = 0usize;
        for i in 0..pres.len().saturating_sub(1) {
            let (t0, p0, _) = &pres[i];
            let (t1, _p1, _) = &pres[i + 1];
            if *t1 <= *t0 || *t1 - *t0 > 3000 { continue; } // only the 2s-cadence gaps
            let gap_start = t0 + 500;
            let count = n_gap.iter().filter(|(t, _)| *t >= gap_start && *t < *t1).count() as u32;
            by_pred.entry(p0.clone()).or_default().push(count);
            total_gap_spikes += count as u64;
            n_gaps += 1;
        }
        for (p, v) in &by_pred {
            let m = v.iter().sum::<u32>() as f32 / v.len().max(1) as f32;
            println!("{name}\tgap-after-{p}\tcounts={v:?}\tmean={m:.3}");
        }
        println!("{name}\tTOTAL\tgaps={n_gaps}\ttotal_gap_spikes={total_gap_spikes}\tmean_per_gap={:.3}",
            total_gap_spikes as f32 / n_gaps.max(1) as f32);
        // per-gap 52-dim internal vectors; predecessor within/cross cosine
        let mut vecs: Vec<(u64, String, Vec<f32>)> = Vec::new();
        for i in 0..pres.len().saturating_sub(1) {
            let (t0, p0, _) = (&pres[i].0, pres[i].1.clone(), ());
            let (t1, _, _) = (&pres[i + 1].0, (), ());
            if *t1 <= *t0 || *t1 - *t0 > 3000 { continue; }
            let gs = t0 + 500;
            let mut v = vec![0.0f32; 52];
            for (t, n) in &n_gap { if *t >= gs && *t < *t1 { let j=(n-24) as usize; if j<52 { v[j]+=1.0; } } }
            vecs.push((gs, p0.clone(), v));
        }
        fn cos(a: &[f32], b: &[f32]) -> f32 {
            let (mut n, mut na, mut nb) = (0.0f32, 0.0f32, 0.0f32);
            for (x, y) in a.iter().zip(b.iter()) { n += x*y; na += x*x; nb += y*y; }
            if na<=0.0 || nb<=0.0 { 0.0 } else { n / (na.sqrt()*nb.sqrt()) }
        }
        let pats: Vec<&String> = vecs.iter().map(|x| &x.1).collect::<Vec<_>>();
        let mut within = Vec::new(); let mut cross = Vec::new();
        let a = vecs.iter().filter(|x| x.1 == "A").cloned().collect::<Vec<_>>();
        let c = vecs.iter().filter(|x| x.1 == "C").cloned().collect::<Vec<_>>();
        for i in 0..a.len().max(c.len()) {
            if i < a.len() && i+1 < a.len() { within.push(cos(&a[i].2, &a[i+1].2)); }
            if i < a.len() && i < c.len() { cross.push(cos(&a[i].2, &c[i].2)); }
        }
        let m = |v:&Vec<f32>| if v.is_empty() {0.0} else { v.iter().sum::<f32>()/v.len() as f32 };
        // split early (first 12) vs late (last 13)
        let (a_e, a_l) = (a[..a.len().min(12)].to_vec(), a[a.len().saturating_sub(13)..].to_vec());
        let (c_e, c_l) = (c[..c.len().min(12)].to_vec(), c[c.len().saturating_sub(13)..].to_vec());
        let mut we = Vec::new(); for i in 0..a_e.len().min(5).max(0) { if i+1<a_e.len() { we.push(cos(&a_e[i].2,&a_e[i+1].2)); } }
        let mut xe = Vec::new(); for i in 0..a_e.len().min(c_e.len()) { xe.push(cos(&a_e[i].2,&c_e[i].2)); }
        let mut wl = Vec::new(); for i in 0..a_l.len().min(5).max(0) { if i+1<a_l.len() { wl.push(cos(&a_l[i].2,&a_l[i+1].2)); } }
        let mut xl = Vec::new(); for i in 0..a_l.len().min(c_l.len()) { xl.push(cos(&a_l[i].2,&c_l[i].2)); }
        let (wv,cv,wee,xee,wll,xll) = (m(&within), m(&cross), m(&we), m(&xe), m(&wl), m(&xl));
        println!("{name}\tGAPVEC\twithinA-A={wv:.3}\tcrossA-C={cv:.3}\tearly_with={wee:.3} early_cross={xee:.3}\tlate_with={wll:.3} late_cross={xll:.3}");
        println!();
    }
}