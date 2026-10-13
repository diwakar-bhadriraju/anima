#!/usr/bin/env bash
# ANIMA benchmark harness (Skynet).
#
# Deterministic, no network, fixed seeds. Emits:
#   METRIC foraging_efficiency=<value>   primary: meals per 1000 beats survived
#   METRIC beats_survived=<value>
#   METRIC final_energy=<value>
#   METRIC identity_ok=<0|1>             frozen-identity gate (must stay 1)
#
# Identity gate: the no-env single-life run MUST reproduce the frozen v0.1.0
# baseline byte-for-byte. If it does not, the harness fails (exit 1).
set -euo pipefail
cd "$(dirname "$0")"

BIN=./target/release/world_survival

# 1) Build (deterministic, no network).
cargo build --release -p anima-world --bin world_survival 2>&1 | tail -1

# 2) Frozen identity gate (byte-exact, single line).
EXPECTED='WL2 RESULT beats=60 died_at=none final_energy=40.0 food_touches=0 novel_flags=0'
ACTUAL="$("$BIN" 20260924 77 60 2>/dev/null | tail -1)"
identity_ok=0
[ "$ACTUAL" = "$EXPECTED" ] && identity_ok=1
echo "identity: $ACTUAL"
echo "METRIC identity_ok=$identity_ok"

# 3) Primary benchmark: short evolution, 2 seeds, fixed gens, no extra gates.
#    Extract the best final fitness across the log (GEN lines).
#    ARMS env (optional): "SL2_WTA=0.01 SL2_MB=1" style gate string applied to
#    BOTH the evolve probe and the identity probe — lets the autoresearch loop
#    measure a candidate arm instead of the frozen baseline. Default: empty =
#    pure identity baseline.
ARMS="${ARMS:-}"
# Structural gates (pool topology, network wiring) legitimately change the
# single-life output — the frozen-identity line does not apply to them.
# NONSTRUCTURAL=1 marks an arm that must still reproduce the frozen line.
STRUCTURAL="${STRUCTURAL:-1}"
LOG="$(mktemp)"
if [ -n "$ARMS" ]; then
  # Gated arm. Identity semantics:
  #   STRUCTURAL=1 (default): arm changes network structure -> skip the
  #   frozen-line check (it would fail by design) and report identity_ok=1
  #   only when the UNGATED frozen probe still passes (the default build
  #   remains the frozen artifact).
  #   STRUCTURAL=0: arm is a parameter-only gate -> the gated probe must
  #   reproduce the frozen line exactly (registered identity-safe gate).
  IDE_ACTUAL="$(env $ARMS "$BIN" 20260924 77 60 2>/dev/null | tail -1)"
  if [ "$STRUCTURAL" = "0" ]; then
    identity_ok=0
    [ "$IDE_ACTUAL" = "$EXPECTED" ] && identity_ok=1
  fi
  echo "identity(gated): $IDE_ACTUAL"
  env $ARMS SL2_NOTARGET=1 "$BIN" --evolve --explore --smell 20260924 7 --until 3 >"$LOG" 2>&1 || true
else
  SL2_NOTARGET=1 "$BIN" --evolve --explore --smell 20260924 7 --until 3 >"$LOG" 2>&1 || true
fi

best=$(grep -oE 'best=[0-9]+\.?[0-9]*' "$LOG" | cut -d= -f2 | sort -g | tail -1)
touches=$(grep -oE 'food_touches=[0-9]+' "$LOG" | cut -d= -f2 | sort -g | tail -1)
rm -f "$LOG"

best=${best:-0}
touches=${touches:-0}

# Primary metric: foraging efficiency (best fitness per 1000 beats of life).
# Explore+smell evolve forces BEATS_EVOLVE=100 per life (line 307); --until 3
# caps generations at 3 for a bounded harness.
beats=100
eff=$(awk -v b="$best" -v t="$beats" 'BEGIN{ if (t<=0) {print 0} else {printf "%.4f", b/t*1000} }')
echo "METRIC foraging_efficiency=$eff"
echo "METRIC best_fitness=$best"
echo "METRIC food_touches=$touches"

# Fail if the identity gate broke — the artifact is no longer the frozen baseline.
if [ "$identity_ok" != "1" ]; then
  echo "FAIL: frozen identity gate mismatch" >&2
  exit 1
fi
exit 0
