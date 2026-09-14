#[test]
fn e6_dbg_clamp_facts() {
    use crate::network::{Network, NetworkConfig, NeuronId, Tick};
    use crate::rate_balance::{E6Params, RateBalance};
    let mut net = Network::new(NetworkConfig::default(), 6, 4, 2, 7);
    let live_in = net.incoming[6].iter()
        .filter(|&&sid| net.synapses[sid.idx()].pre.idx() < 6 && net.synapses[sid.idx()].silent_ticks != u64::MAX)
        .count();
    eprintln!("seeded live input afferents of post 6: {live_in}");
    for _ in 0..2 {
        for ch in 1..6u32 {
            net.add_synapse(NeuronId(ch), NeuronId(6), 0.05, true, Tick(0));
        }
    }
    net.add_synapse(NeuronId(0), NeuronId(6), 0.05, true, Tick(0));
    let mut total = 0; let mut ch0 = 0;
    for &sid in &net.incoming[6] {
        let s = &net.synapses[sid.idx()];
        if s.silent_ticks != u64::MAX && s.pre.idx() < 6 {
            total += 1;
            if s.pre.idx() == 0 { ch0 += 1; }
        }
    }
    eprintln!("after adds: total live input afferents = {total}, ch0 count = {ch0}");
    let mut r = RateBalance::new(E6Params::new(0.04, 0.02, 0.001, 0.1, 10.0, 100, 6));
    for i in 0..6 { r.phi[i] = 0.001; }
    r.phi[0] = 10.0;
    eprintln!("beta(ch0) = {}", r.beta(&net, NeuronId(6), NeuronId(0)));
}
