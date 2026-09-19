#!/usr/bin/env python3
"""E24 frozen config generator. 6 seeds x 5 arms = 30 runs.

Arms (all: beta=0.0046875, tau=5000, 40 presentations total,
500 ms each, 2 s cadence, S0 5 s silence, S2 20 s silence):
  a   : present=["A"],           reps=40, interleaved
  c   : present=["C"],           reps=40, interleaved
  il  : present=["A","C"],       reps=20, interleaved (seeded shuffle)
  bac : present=["A","C"],       reps=20, blocked (all A, then all C)
  bca : present=["C","A"],       reps=20, blocked (all C, then all A)
Seeds frozen: 20260912 424242 9001 123456 777 31337
Template: configs/v21probe-b0.00625-t5000.toml
"""
TMPL = open("configs/v21probe-b0.00625-t5000.toml").read()

SEEDS = [20260912, 424242, 9001, 123456, 777, 31337]
ARMS = {
    "a":   ('["A"]', 40, "interleaved"),
    "c":   ('["C"]', 40, "interleaved"),
    "il":  ('["A", "C"]', 20, "interleaved"),
    "bac": ('["A", "C"]', 20, "blocked"),
    "bca": ('["C", "A"]', 20, "blocked"),
}

for seed in SEEDS:
    for arm, (present, reps, order) in ARMS.items():
        exp_id = f"e24-s{seed}-{arm}"
        s = TMPL
        s = s.replace('exp_id = "v21probe-b0.00625-t5000"', f'exp_id = "{exp_id}"')
        s = s.replace("seed = 20260912", f"seed = {seed}")
        s = s.replace("slow_state_beta = 0.00625", "slow_state_beta = 0.0046875")
        s = s.replace('present = ["A"]\nreps = 40\norder = "interleaved"\noff_ms = 1500',
                      f'present = {present}\nreps = {reps}\norder = "{order}"\noff_ms = 1500')
        assert f'exp_id = "{exp_id}"' in s and "slow_state_beta = 0.0046875" in s, exp_id
        assert s != TMPL
        open(f"configs/{exp_id}.toml", "w").write(s)
        print(exp_id)
