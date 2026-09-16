#[test]
fn e11_audit_probe() {
    use crate::config::ExpConfig;
    // Reconstruct the E10 schedule (same shuffle) and compute the
    // variant=rep%2 pairing statistics: B's position in each round and
    // the preceding pattern, cross-tabulated by variant.
    for (name, seed) in [("e10.toml", 20260912u64), ("e10-seed9001.toml", 9001), ("e10-seed424242.toml", 424242)] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs").join(name);
        let cfg = ExpConfig::parse(&path).unwrap();
        let env = crate::env::Environment::new(cfg, seed);
        // rounds: S1 presentations grouped in 3 (A,B,C shuffled per round)
        let s1: Vec<&crate::env::ScheduledPresentation> = env.schedule.iter().filter(|p| p.stage == "S1").collect();
        let mut b_pos = [0u64; 3];      // position of B in round, by variant
        let mut b_pos_v = [[0u64; 3]; 2];
        let mut prev_v = [[0u64; 3]; 2]; // preceding pattern A/B/C by variant (0=A,1=B,2=C)
        let mut b_surrogate = 0usize;
        let mut prev_pat = 3usize; // none
        let mut b_reps_so_far = 0usize;
        for (i, p) in s1.iter().enumerate() {
            if p.pattern == "B" {
                let variant = b_reps_so_far % 2;
                let pos = i % 3;
                b_pos_v[variant][pos] += 1;
                prev_v[variant][prev_pat.min(2)] += 1;
                b_reps_so_far += 1;
                b_surrogate += 0;
            } else {
                prev_pat = if p.pattern == "A" { 0usize } else if p.pattern == "C" { 2 } else { 1 };
            }
        }
        eprintln!("{name}: B reps={b_reps_so_far}; position-by-variant SEQ(0)=A:{:?}? variant tables: {:?} ; preceding-by-variant: {:?}",
            b_pos_v[0], b_pos_v, prev_v);
    }
}
