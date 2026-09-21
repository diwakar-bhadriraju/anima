//! papermass: read-only drive-end afferent mass split for this paper's
//! figures (Fig 2a: per-neuron A/C mass for the A-trained run; Fig 5:
//! end-state A/C masses for the regime comparison).
//!
//! Reads snapshots only (preserved runs); prints CSV rows:
//!   <run-name>,<neuron>,<mA>,<mC>,<mB>,<mOther>,<consolidated>
//! at the drive-end snapshot (t = 84,000 ms when present, else the last
//! snapshot). Mass = sum of amplitude-normalized weights (bare w) over
//! live excitatory synapses from the given channel groups onto the
//! neuron; "consolidated" = sum over consolidated ones only.
use anima_telemetry::recorder::read_snapshots;

fn main() {
    let mut all: Vec<(String, Vec<[f32; 5]>)> = Vec::new();
    for dir in std::env::args().skip(1) {
        let snaps = read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let last = snaps.last().unwrap();
        let drive = snaps.iter().find(|s| s.tick == 84_000).unwrap_or(last);
        let mut rows = vec![[0.0f32; 5]; drive.neurons.len()];
        for s in &drive.synapses {
            let pidx = s.post as usize;
            // established instrument convention: internal/output neurons 24..76
            if pidx < 24 || pidx >= 76 || pidx >= drive.neurons.len() { continue; }
            let Some(w) = s.w else { continue };
            if s.pre >= 24 { continue; } // afferents only (channel groups)
            let ci = if s.pre < 8 { 0 } else if s.pre < 16 { 1 } else { 2 };
            rows[pidx][ci] += w;
            if !s.consolidated { rows[pidx][3] += w; }
        }
        for r in rows.iter_mut() {
            r[4] = if r[0] + r[1] + r[2] > 0.0 { 1.0 } else { 0.0 };
        }
        let name = dir.rsplit('/').next().unwrap().to_string();
        println!("# RUN {}", name);
        for i in 24..76 {
            let r = &rows[i];
            println!("{}\t{}\t{:.4}\t{:.4}\t{:.4}\t{:.4}\t{:.0}",
                name, i, r[0], r[1], r[2], r[3], r[4]);
        }
        all.push((name, rows));
    }
    // aggregated drive-end means over the 52 internal/output neurons
    // (established instrument convention; input neurons excluded)
    for (name, rows) in &all {
        let n = 52.0f32;
        let sum = |j: usize| rows[24..76].iter().map(|r| r[j]).sum::<f32>() / n;
        println!("# MEAN {} A={:.4} C={:.4} B={:.4} uncons={:.4}",
            name, sum(0), sum(1), sum(2), sum(3));
    }
}