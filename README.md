# ANIMA

> **A developmental artificial nervous system that forages.** A spiking LIF network receives
> retinal/proprioceptive frames from a physical 3D body in an arena and controls it to find and
> consume food — written from scratch in Rust, with **no hand-designed internal architecture, no
> pretrained models, and no external memory modules.**

<div align="center">

**Read the friendly, from-the-start white paper**

**[🧠 docs/white-paper.md](docs/white-paper.md)**  — a research story told in plain language:
what we built, every idea explained, our wins *and* our honest failures, and the two walls we
measured. (A deeper technical reproduction log lives in the code comments and git history.)

</div>

---

## What is this?

ANIMA is an attempt to grow, rather than build, an intelligent agent. We fix only the *boundaries* —
sensory inputs in, motor outputs out, a physical world, a survival rule, and fundamental plasticity
laws — and let the network organize its own internals. The research program is committed to
**laboratory discipline**: every experiment is env-gated (identity-safe), deterministic, and
pre-registered before it runs.

The result so far is a genuine forager: it corrects its own steering bug, sustains feeding under
shaping, and can be given a home second-drive it returns to when rewarded. Two measured walls — an
internal-band saturation that blocks selective learning, and a fitness-economy ceiling — are mapped
in the white paper, told from the start in plain language.

## Getting started

```bash
cargo build --release -p anima-world --bin world_survival

# Frozen default-identity check (must print exactly):
./target/release/world_survival 20260924 77 60
#   WL2 RESULT beats=60 died_at=none final_energy=40.0 food_touches=0 novel_flags=0

cargo test --workspace        # 198 tests, 0 failures
```

## Key ideas

| Idea | Detail |
|---|---|
| **Self-organization** | STDP + structural plasticity grow the topology; no hand-designed internals |
| **Env-gated experiments** | every mechanism is a `SL2_*` / `WL2_*` switch; unset ⇒ byte-identical baseline |
| **Determinism** | seeded RNG, no `HashMap` iteration order ⇒ same args + env ⇒ identical runs |
| **Reward = food only** | steering is the action, never rewarded directly (charter) |

## The organism

The core experiment lives in [`crates/anima-world/src/bin/world_survival.rs`](crates/anima-world/src/bin/world_survival.rs);
world dynamics (energy, food, physics) in [`crates/anima-world/src/world.rs`](crates/anima-world/src/world.rs).

```
        retina / proprio / smell                 motor rates
   ┌─────────────┐      ┌──────────────┐      ┌──────────────┐
   │ 62 inputs    │─────▶│ internal pool │─────▶│ 12 outputs    │
   │ (poisson)    │      │ (LIF + STDP) │      │ (LIF → body)  │
   └─────────────┘      └──────────────┘      └──────────────┘
                                          world (energy, food, physics)
```

## Selected mechanisms (env-gated)

| Gate | Mechanism | Status |
|---|---|---|
| `SL2_STASIS` | stable food + re-entry guard + tank cap | ✅ verified (3 → 36 genuine touches/life) |
| `SL2_HMEXP` | hunger-modulated search (settle-when-fed) | ✅ verified |
| `SL2_NEST` / `SL2_NESTFIT` | home second-drive + homing reward | ✅ verified (repeated return-home on seed 7) |
| `SL2_RSTDPP` | dopamine-gated STDP (three-factor) | ◻ measured — fires, non-selective under saturation |
| `SL2_NOVELTY` | minimal-criterion novelty selection | ◻ measured — steers, ceiling unchanged |
| `SL2_MB` | mushroom-body expansion + input-locus plasticity | ◻ unproven — awaits de-saturation (sparsity premise) |
| `SL2_IINH/THETA/AMPL/HET` | de-saturation campaign | ✗ measured negative (all four bounded) |

## The two walls (short version)

1. **Internal-band neural saturation** — the expansion layer fires at 100% every tick, so every
   reward-gated learning rule is a non-selective global wash the GA ignores. Exhaustively probed
   (competition, threshold, amplitude, heterogeneity) and found temporally bistable: cold→sparse,
   then a sharp phase-transition to saturated.
2. **Fitness-economy ceiling** — evolution pins at ~121–137.6 depending on scoring family;
   `final_energy + 3·touches` saturates once food is reachable.

Both are explained with their measured evidence in the
[white paper](docs/white-paper.md) — written so anyone can follow the reasoning.

## Repository layout

```
crates/anima-core/        spiking network core (LIF, STDP, structural plasticity)
crates/anima-world/       the 3D world, the organism, the experiment binary
docs/white-paper.md       the white paper — readable, from-the-start research story
```

## Reproducibility & discipline

Every claim in the white paper is backed by a deterministic run and verifiable from this repo:
frozen identity line, two-run determinism diff, 198-test suite, and env-gated observability
probes (`KCPROBE`, `SW_PROBE`, `MB_PROBE`, `NEST_PROBE`). Experiment-protocol discipline is captured
in the `anima-experiment-protocol` skill used across sessions.

## License

See [`paper/`](paper/) for the older manuscript lineage. (License TBD — ask if you'd like MIT/Apache added.)