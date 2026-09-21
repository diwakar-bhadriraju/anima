#!/usr/bin/env bash
# regenerate the data files under paper/scripts/data from preserved runs.
# Requires the workspace built in release mode:
#   cargo build --release -p anima-exp --example rg8eval --example clla_traj --example papermass
set -euo pipefail
cd "$(dirname "$0")/../.."   # repo root
OUT=paper/scripts/data
mkdir -p "$OUT"
EX=target/release/examples

# per-presentation decomposition (post drive, currents, burst) — blocked
# baselines and both gain registrations (bac, three seeds)
declare -A RUNS=(
  [fe-s20260912-bac]=runs/clla-fe-s20260912-bac-20260921T153338Z
  [fe-s9001-bac]=runs/clla-fe-s9001-bac-20260921T153408Z
  [fe-s424242-bac]=runs/clla-fe-s424242-bac-20260921T153353Z
  [rg8-s20260912-bac]=runs/clla-rg8-s20260912-bac-20260921T171025Z
  [rg8-s9001-bac]=runs/clla-rg8-s9001-bac-20260921T171102Z
  [rg8-s424242-bac]=runs/clla-rg8-s424242-bac-20260921T171044Z
  [rg8c-s20260912-bac]=runs/clla-rg8c-s20260912-bac-20260921T173741Z
  [rg8c-s9001-bac]=runs/clla-rg8c-s9001-bac-20260921T173818Z
  [rg8c-s424242-bac]=runs/clla-rg8c-s424242-bac-20260921T173759Z
  [fe-s20260912-il]=runs/clla-fe-s20260912-il-20260921T153348Z
)
for name in "${!RUNS[@]}"; do
  $EX/rg8eval "${RUNS[$name]}" > "$OUT/$name.tsv" 2>/dev/null
done

# protected mass trajectories (bounded-registration witness run)
$EX/clla_traj runs/clla-s424242-il-20260920T181810Z > "$OUT/prot-s424242-il.tsv" 2>/dev/null
$EX/clla_traj runs/clla-s20260912-bac-20260920T181759Z > "$OUT/prot-s20260912-bac.tsv" 2>/dev/null

# regime end-state masses (A-trained, C-trained, blocked x2, alternating;
# e24 single-pattern runs and the G3 corpus)
$EX/papermass \
  runs/e24-s20260912-a-20260919T131927Z \
  runs/e24-s20260912-c-20260919T132007Z \
  runs/e24-s20260912-bac-20260919T132117Z \
  runs/e24-s20260912-bca-20260919T132154Z \
  runs/e24-s20260912-il-20260919T132037Z \
  > "$OUT/regimes.csv" 2>/dev/null

echo "data regenerated in $OUT"
ls "$OUT"