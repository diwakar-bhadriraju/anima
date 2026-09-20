//! CLLA protocol measurement (read-only, docs/anima-clla-protocol.md §3).
//! Per snapshot: per-neuron protected mass P, working mass W, cap checks;
//! drive-end protected A-mass / peak for F7. Reads consolidated flag from
//! SynapseState.
fn main() {
    for dir in std::env::args().skip(1) {
        let snaps = anima_telemetry::recorder::read_snapshots(
            &std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let mut cap_viol = 0u64;
        let mut max_p = 0.0f64;
        let mut max_p_ratio = 0.0f64;
        let mut peak_pa = 0.0f64;
        let mut drive_end_pa = 0.0f64;
        let mut drive_end_seen = 0u64;
        let cap = 0.75_f64 * 0.8_f64 + 0.04; // p_max_frac*t_e + epsilon (protocol §3.3)
        let t_e = 0.8_f64;
        for s in &snaps {
            let mut psum = vec![0.0f64; 76];
            let mut wsum = vec![0.0f64; 76];
            for syn in &s.synapses {
                let w = syn.w.unwrap_or(0.0) as f64;
                if syn.plastic {
                    if syn.consolidated { psum[syn.post as usize] += w; }
                    else { wsum[syn.post as usize] += w; }
                } else {
                    // inhibitory — ignore (M6)
                    if psum.len() > syn.post as usize { let _ = psum.len(); }
                }
            }
            // only non-input neurons 24..76
            for n in 24..76usize {
                if psum[n] > max_p { max_p = psum[n]; }
                let ratio = psum[n] / (0.75_f64 * t_e);
                if ratio > max_p_ratio { max_p_ratio = ratio; }
                if psum[n] > cap { cap_viol += 1; }
                if wsum[n] > (t_e - psum[n]) + 1e-6 { /* R2 check */ }
            }
            // F7: protected A-mass (consolidated, pre in 0..8)
            let pa: f64 = s.synapses.iter()
                .filter(|x| x.consolidated && x.pre < 8 && !x.plastic == false)
                .map(|x| x.w.unwrap_or(0.0) as f64).sum();
            // NOTE: plastic==true excitation; consolidated pre in A channels
            let pa_correct: f64 = s.synapses.iter()
                .filter(|x| x.consolidated && x.pre < 8 && x.plastic)
                .map(|x| x.w.unwrap_or(0.0) as f64).sum();
            let _ = pa;
            if pa_correct > peak_pa { peak_pa = pa_correct; }
            if s.tick == 84000 || (s.tick > 80000 && drive_end_seen == 0) {
                drive_end_pa = pa_correct;
                drive_end_seen = 1;
            }
        }
        println!("{}", dir.rsplit('/').next().unwrap());
        println!("  snapshots={} cap_violations(total over all neurons*snaps)={} max_P={:.4} max_P/(0.75*t_e)={:.2}", snaps.len(), cap_viol, max_p, max_p_ratio);
        println!("  F7: peak protected A-mass={:.4} drive_end={:.4} ratio={:.3} ({})", peak_pa, drive_end_pa, drive_end_pa / peak_pa.max(1e-12), if drive_end_pa < 0.5 * peak_pa { "F7 FAIL" } else { "F7 pass" });
        println!();
    }
}