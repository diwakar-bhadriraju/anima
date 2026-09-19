#!/usr/bin/env python3
"""Post-V2.1 exploratory config generator (X-series, no E-number).

Families:
  xp1  — onset sweep: v21probe schedule, fine beta at fixed tau=5000
         (upward from the silent cell 0.00625 at tau=2500's neighbor),
         plus tau probes at the wedge flank.
  xp2  — A/C u-contrast probe: same schedule but interleaved A/C drive,
         for antecedent-conditioned u/spike analysis.
Derived from configs/v21probe-b0.00625-t5000.toml (committed Stage-B
probe template). Only [run].exp_id, slow_state fields, and S1 present
list change.
"""
import sys, os

TMPL = open("configs/v21probe-b0.00625-t5000.toml").read()

def variant(exp_id, beta, tau, present):
    s = TMPL
    s = s.replace('exp_id = "v21probe-b0.00625-t5000"', f'exp_id = "{exp_id}"')
    s = s.replace("slow_state_beta = 0.00625", f"slow_state_beta = {beta}")
    s = s.replace("slow_state_tau_ms = 5000", f"slow_state_tau_ms = {tau}")
    if present != ["A"]:
        s = s.replace('present = ["A"]', "present = " + str(present).replace("'", '"'))
    return s

if sys.argv[1] == "xp1":
    # onset sweep at tau=5000: between silent 0.00625(at t2500) and pacemaker 0.00625(t5000)
    # -> probe t5000 flank below/at wedge: 0.0015625, 0.003125, 0.0046875 ; and tau sweep at
    # beta=0.003125: 3500, 5000, 7000 (t10000 is pacemaker already).
    cells = [
        ("xp1-b0.0015625-t5000", 0.0015625, 5000, ["A"]),
        ("xp1-b0.003125-t5000", 0.003125, 5000, ["A"]),
        ("xp1-b0.0046875-t5000", 0.0046875, 5000, ["A"]),
        ("xp1-b0.003125-t3500", 0.003125, 3500, ["A"]),
        ("xp1-b0.003125-t7000", 0.003125, 7000, ["A"]),
    ]
elif sys.argv[1] == "xp2":
    cells = [
        ("xp2-b0.00625-t5000", 0.00625, 5000, ["A", "C"]),
        ("xp2-b0.003125-t10000", 0.003125, 10000, ["A", "C"]),
    ]
else:
    raise SystemExit("family?")

for exp_id, beta, tau, present in cells:
    p = f"configs/{exp_id}.toml"
    open(p, "w").write(variant(exp_id, beta, tau, present))
    print("wrote", p)
