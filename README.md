# ANIMA

A **developmental artificial nervous system** in Rust: a spiking LIF network receives
retinal/proprioceptive frames from a physical 3D body in an arena and controls it to forage
for food — written from scratch, with **no hand-designed internal architecture, no pretrained
models, and no external memory modules**.

> **Read the full technical white paper: [`docs/white-paper.md`](docs/white-paper.md)** —
> goal, method, measured wins & registered negatives, the two walls, and a complete
> reproduction specification (App. A).

## Key ideas

- **Self-organization**: the network grows its own internal topology (STDP + structural growth).
- **Env-gated research discipline**: every experiment is a `SL2_*`/`WL2_*` environment switch;
  with none set, output is byte-identical to the frozen baseline. Determinism is enforced
  (seeded RNG, no `HashMap` iteration order) — same args + env ⇒ identical runs.
- **Reward = food only**; steering is the action, never rewarded directly.

## Build & run

```bash
cargo build --release -p anima-world --bin world_survival

# Frozen default-identity check (must print exactly):
./target/release/world_survival 20260924 77 60
#   WL2 RESULT beats=60 died_at=none final_energy=40.0 food_touches=0 novel_flags=0

cargo test --workspace   # 198 tests, 0 failures
```

## The organism

The core experiment binary is [`crates/anima-world/src/bin/world_survival.rs`](crates/anima-world/src/bin/world_survival.rs)
and the world/energy/food dynamics live in [`crates/anima-world/src/world.rs`](crates/anima-world/src/world.rs).

Selected env-gated mechanisms (all disable-safe by default):

| Gate | Mechanism |
|---|---|
| `SL2_STASIS` | stable food + re-entry guard + tank-cap energy (3 → 36 genuine touches/life) |
| `SL2_HMEXP` | hunger-modulated search (settle-when-fed) |
| `SL2_NEST` / `SL2_NESTFIT` | home/rest second drive + homing reward (repeated return-home) |
| `SL2_RSTDPP` | dopamine-gated STDP (three-factor) |
| `SL2_NOVELTY` | minimal-criterion novelty selection |
| `SL2_MB` + friends | mushroom-body-style expansion + reward-gated input plasticity |
| `SL2_*` de-saturation levers | `IINH`, `THETA`, `AMPL`, `HET` (de-saturation campaign) |

## Results in one paragraph

ANIMA is a real forager: it corrects its own steering bug, finds and consumes food strongly under
shaping, survives long, and (with a home second-drive) can sustain repeated return-home behavior
when rewarded. Two measured walls block the open-ended goal: **internal-band neural saturation**
(which makes selective credit impossible — probed exhaustively across competition, threshold,
amplitude, and heterogeneity) and a **fitness-economy ceiling** (~121–137.6 depending on scoring
family). The white paper documents the measured evidence for each.