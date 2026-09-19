//! Diagnosis instrument: A/C afferent structure per internal neuron.
//! For each non-input neuron: sum of w from A-channels (0-7) vs C-channels (8-15),
//! from the DRIVE-END snapshot of a run (post-plasticity). Reports:
//! - channel dominance distribution (how many internal neurons are A-dominant,
//!   C-dominant, balanced)
//! - winner neurons' afferent profiles
//! Plus: overlap of silence-active sets between arms within seed (from telemetry).
use std::collections::BTreeMap;
fn main() {
    for dir in std::env::args().skip(2) {
        let name = dir.rsplit('/').next().unwrap().to_string();
        let snaps = anima_telemetry::recorder::read_snapshots(&std::path::Path::new(&dir).join("snapshots.bin.zst")).unwrap();
        let tick_arg: Option<u64> = std::env::args().nth(2).and_then(|x| x.parse().ok());
        let s = match tick_arg { Some(t) => snaps.iter().filter(|s| s.tick <= t).next_back().unwrap(), None => snaps.last().unwrap() };
        let mut dom: BTreeMap<u32, (f32, f32)> = BTreeMap::new(); // neuron -> (wA, wC)
        let mut plastic_count = 0;
        for syn in &s.synapses {
            let (pre, post, w) = (syn.pre, syn.post, syn.w.unwrap_or(0.0));
            if pre < 24 && post >= 24 && post < 76 && syn.plastic {
                let e = dom.entry(post).or_insert((0.0, 0.0));
                if pre < 8 { e.0 += w } else { e.1 += w }
                plastic_count += 1;
            }
        }
        let (mut na, mut nc, mut nb) = (0, 0, 0);
        for (_, (wa, wc)) in &dom {
            let r = wa / wc.max(1e-9);
            if r > 1.5 { na += 1 } else if r < 1.0/1.5 { nc += 1 } else { nb += 1 }
        }
        println!("{}\tplastic_in_syn={}\tA-dom={}\tC-dom={}\tbal={}\tof {}",
            name, plastic_count, na, nc, nb, dom.len());
        // per-neuron ratio list for the top-3 u neurons
        let mut byu: Vec<(u32, f32)> = s.neurons.iter().filter(|n| n.id >= 24).map(|n| (n.id, n.u_slow.unwrap_or(0.0))).collect();
        byu.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        for (id, u) in byu.iter().take(3) {
            if let Some((wa, wc)) = dom.get(id) {
                println!("  {} top-u n{} u={:.2} wA={:.2} wC={:.2} ratio={:.2}", name, id, u, wa, wc, wa/wc.max(1e-9));
            }
        }
    }
}
