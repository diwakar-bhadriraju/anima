//! D-52 lever (b) probe: does a WIDER output band (12 -> 24 output
//! neurons) give A/C distinct refs? Self-contained; builds a fresh net,
//! forms A/C (legacy 8ch), captures 24-dim refs, prints per-symbol
//! cosine + ref vectors. NO production code touched.
use anima_core::network::{Network, NetworkConfig, Tick, InputFrame, V2Params};
use anima_core::plasticity::{stdp_tick, Traces, StdpParams};
use anima_exp::io;

fn main() {
    let seed: u64 = std::env::args().nth(1).and_then(|a|a.parse().ok()).unwrap_or(20260912);
    let n_out: usize = std::env::args().nth(2).and_then(|a|a.parse().ok()).unwrap_or(24);
    let mut net = Network::new(v2cfg(), 24, 40, n_out, seed);
    // form A/C, legacy 8ch
    let known = ["A", "C"];
    let mut tr = Traces::new(&net, 20.0);
    let p = StdpParams{tau_plus:20.0,tau_minus:20.0,a_plus:0.005,a_minus:0.0053,decay:1e-6,w_min:0.0,w_max:1.0};
    for _rep in 0..20 { for sym in known {
        let st = io::symbol_trains(sym, seed);
        for t in 0..io::BEAT_MS {
            let f = InputFrame{tick:net.tick, spikes:st.iter().filter(|(tt,_)|*tt==t).map(|(_,c)|*c).collect()};
            let e = net.step(&f); tr.step(&net,&e.spikes);
            let _ = stdp_tick(&p,&mut net,&tr,&e.spikes,1.0,None);
            net.tick=Tick(net.tick.0+1);
        }
        for _ in 0..1500 { let f=InputFrame{tick:net.tick,spikes:vec![]};
            let e=net.step(&f); tr.step(&net,&e.spikes);
            let _ = stdp_tick(&p,&mut net,&tr,&e.spikes,1.0,None);
            net.tick=Tick(net.tick.0+1); }
    }}
    // capture 24-dim refs (output band = last n_out neurons)
    let out_lo = net.neurons.len() - n_out;
    let mut acc: std::collections::BTreeMap<String,Vec<f32>> = Default::default();
    let mut cnt: std::collections::BTreeMap<String,u32> = Default::default();
    for sym in known { for _ in 0..3 {
        let st=io::symbol_trains(sym,seed); let mut out=vec![0f32;n_out];
        for t in 0..io::BEAT_MS { let f=InputFrame{tick:net.tick,spikes:st.iter().filter(|(tt,_)|*tt==t).map(|(_,c)|*c).collect()};
            let e=net.step(&f); for c in &e.spikes { let ci=c.0 as usize; if ci>=out_lo && ci<out_lo+n_out { out[ci-out_lo]+=1.0; } }
            net.tick=Tick(net.tick.0+1); }
        let en=acc.entry(sym.to_string()).or_insert_with(||vec![0f32;n_out]); for i in 0..n_out{en[i]+=out[i];}
        *cnt.entry(sym.to_string()).or_insert(0)+=1;
    }}
    let refs: Vec<(String,Vec<f32>)> = acc.into_iter().map(|(p,v)|{let c=*cnt.get(&p).unwrap() as f32;(p,v.iter().map(|x|x/c).collect())}).collect();
    println!("== outprobe seed={seed} n_out={n_out} ==");
    // how many output neurons ACTUALLY fire?
    let active: Vec<usize> = (0..n_out).filter(|&i| refs[0].1[i]>0.0 || refs[1].1[i]>0.0).collect();
    println!("   active out neurons: {}/{} -> {:?}", active.len(), n_out, active);
    for i in 0..refs.len(){ for j in i+1..refs.len(){
        println!("   cos({}-{})={:.4}", refs[i].0, refs[j].0, io::cos(&refs[i].1,&refs[j].1));
    }}

    // === D-52 lever (a) temporal-codec comparison on identical nets ===
    // count-codec cosine already printed above. Build 5-bin temporal refs
    // and re-measure A/C cosine + decode acc.
    let bins_n = 5usize;
    let bin_ms = (io::BEAT_MS as usize) / bins_n;
    // temporal refs: out_temporal[dim= n_out*bins]
    let mut tacc: std::collections::BTreeMap<String, Vec<f32>> = Default::default();
    let mut tcnt: std::collections::BTreeMap<String, u32> = Default::default();
    for sym in known { for _ in 0..3 {
        let st=io::symbol_trains(sym,seed);
        let mut tt=vec![0f32; n_out*bins_n];
        for bind_t in 0..io::BEAT_MS { let f=InputFrame{tick:net.tick,spikes:st.iter().filter(|(tt0,_)|*tt0==bind_t).map(|(_,c)|*c).collect()};
            let e=net.step(&f);
            let bin=(bind_t as usize/bin_ms).min(bins_n-1);
            for c in &e.spikes { let ci=c.0 as usize; if ci>=out_lo && ci<out_lo+n_out { tt[(ci-out_lo)*bins_n + bin]+=1.0; } }
            net.tick=Tick(net.tick.0+1); }
        let en=tacc.entry(sym.to_string()).or_insert_with(||vec![0f32;n_out*bins_n]); for i in 0..n_out*bins_n{en[i]+=tt[i];}
        *tcnt.entry(sym.to_string()).or_insert(0)+=1;
    }}
    let trefs: Vec<(String,Vec<f32>)> = tacc.into_iter().map(|(p,v)|{let c=*tcnt.get(&p).unwrap() as f32;(p,v.iter().map(|x|x/c).collect())}).collect();
    for i in 0..trefs.len(){ for j in i+1..trefs.len(){
        println!("   TEMPORAL({}bin) cos({}-{})={:.4}", bins_n, trefs[i].0, trefs[j].0, io::cos(&trefs[i].1,&trefs[j].1));
    }}
    // held-out decode acc: count vs temporal, per symbol (present fresh)
    fn heldout_decode_acc(net:&mut Network, seed:u64, known:&[&str;2], refs:&[(String,Vec<f32>)], lo:usize, bins:usize, use_bins:bool)->f32 {
        let mut ok=0u32; let mut tot=0u32;
        for sym in known { for _ in 0..4 {
            let st=io::symbol_trains(sym,seed);
            let mut v=if use_bins { vec![0f32; (12*std::cmp::max(1,refs.len()))*0+refs[0].1.len()] } else { vec![0f32; refs[0].1.len()] };
            let bin_ms=(500usize)/(usize::max(1,bins));
            for t in 0..io::BEAT_MS { let f=InputFrame{tick:net.tick,spikes:st.iter().filter(|(tt,_)|*tt==t).map(|(_,c)|*c).collect()};
                let e=net.step(&f);
                let bin=(t as usize/bin_ms).min(bins-1);
                for c in &e.spikes { let ci=c.0 as usize; if ci>=lo { 
                    if use_bins { let idx=(ci-lo)*bins+bin; if idx<v.len(){v[idx]+=1.0;} } else { let idx=ci-lo; if idx<v.len(){v[idx]+=1.0;} }
                } }
                net.tick=Tick(net.tick.0+1); }
            // argmax cos
            let mut best: Option<(f32,&str)>=None;
            for (p,r) in refs { let c=io::cos(&v,r); if best.as_ref().map(|(bc,_)|c>*bc).unwrap_or(true){best=Some((c,p));} }
            if let Some((_,p))=best { if p==*sym{ok+=1;} tot+=1; }
        }}
        ok as f32 / tot.max(1) as f32
    }
    let ca = heldout_decode_acc(&mut net, seed, &known, &refs, out_lo, 1, false);
    let ta = heldout_decode_acc(&mut net, seed, &known, &trefs, out_lo, bins_n, true);
    println!("   held-out decode acc: count-codec={:.3}  temporal-codec={:.3}", ca, ta);
    for r in &refs { // print nonzero channels
        let nz: Vec<String> = r.1.iter().enumerate().filter(|(_,v)|**v>0.0).map(|(i,v)|format!("{i}:{v:.0}")).collect();
        println!("   ref {}: nonzeros [{}]", r.0, nz.join(", "));
    }
}
fn v2cfg() -> NetworkConfig {
    NetworkConfig {
        connectivity:0.038, w_init:0.2, amplitude:52.0, adaptation_tau_ms:200.0,
        adaptation_gain:0.05, inhibition_gain:0.0, slow_state_beta:0.0046875,
        slow_state_tau_ms:5000.0, slow_state_beta_drive:false, latch_enable:true,
        theta_rel_mean:1.0, theta_rel_sd:0.0,u_plateau_rel_mean:1.0,u_plateau_rel_sd:0.0,
        tau_het_rel_sd:0.0, phi_rel:0.5, eta_rel:0.0, v2:Some(v2p()), ..Default::default()
    }
}
fn v2p() -> V2Params {
    V2Params {
        p_in: 0.5, w_in_lo: 0.02, w_in_hi: 0.06, p_rec: 0.2, w_rec_lo: 0.005, w_rec_hi: 0.02,
        t_e: 0.8, assembly_protect: true, p_max_frac: 0.75, w_consolidate_min: 0.05,
        alloc_residual: true, dormant_reserve: false, recruit_gain: false,
        d_core: true, d_claim: true, d_sparse: false, d_elig: false, d_elig_ro: false, d_ing: false,
        c_slots: 6, w_c_init: 0.01, delta_perm: 0.01, decay_c: 0.99, theta_permanent: 0.05,
        w_c_permanent: 0.02, theta_die: 0.005, p_cand_in: 0.5, p_cand_rec: 0.5,
        theta_prune: 0.005, prune_windows: 10, b_e: 40, b_i: 10, p_inh: 0.3, w_inh_lo: 0.01,
        w_inh_hi: 0.03, a_inh: 0.005, decay_inh: 0.98, w_inh_max: 0.10, window_ticks: 100,
        m2_buckets: 1, m2_epoch_windows: 1, disable_m2: false, disable_m3_m4: false,
        disable_m5: false, disable_m6: false,
    }
}

