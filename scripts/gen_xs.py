#!/usr/bin/env python3
"""X3 boundary map: uniform grid configs (exploratory, no E-number).

Grid (FROZEN before execution — do not expand/optimize mid-run):
  seeds  {20260912, 424242, 9001, 123456, 777}
  drive  {A-only, C-only, A/C interleaved}
  beta   {0.003125, 0.0046875, 0.00625}
  tau    5000
Template: configs/v21probe-b0.00625-t5000.toml (committed Stage-B probe).
Only exp_id, seed, slow_state_beta, present change.
"""
TMPL = open("configs/v21probe-b0.00625-t5000.toml").read()

SEEDS = [20260912, 424242, 9001, 123456, 777]
COMPS = {"a": ["A"], "c": ["C"], "ac": ["A", "C"]}
BETAS = [0.003125, 0.0046875, 0.00625]

for seed in SEEDS:
    for tag, present in COMPS.items():
        for beta in BETAS:
            s = TMPL
            exp_id = f"xs-s{seed}-{tag}-b{beta:g}"
            s = s.replace('exp_id = "v21probe-b0.00625-t5000"', f'exp_id = "{exp_id}"')
            s = s.replace("seed = 20260912", f"seed = {seed}")
            s = s.replace("slow_state_beta = 0.00625", f"slow_state_beta = {beta:g}")
            s = s.replace('present = ["A"]', "present = " + str(present).replace("'", '"'))
            open(f"configs/{exp_id}.toml", "w").write(s)
            print(exp_id)
