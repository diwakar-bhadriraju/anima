# ANIMA experiment index

This folder is an index, not a copy. Each experiment family points at the protocol, design, and
record files that already live in the repository. Start here to navigate a specific experiment;
start at the [white paper](../docs/white-paper.md) to read the whole story.

## How experiments are recorded

- Protocols and records live in [`docs/`](../docs/), named `anima-e<N>-protocol.md` (the experiment
  protocol), `anima-e<N>-design.md` (the design rationale), and `anima-e<N>-audit.md` (a follow-up
  audit). Early letters: E1 is the first experiment.
- The `clla` files under `docs/` are the capability-architecture records for the CLLA track.
  `decision-log.md` and `autonomy-log.md` hold cross-cutting decisions and autonomy notes.
- Generators for experiment configurations live in [`scripts/`](../scripts/), and the generated
  configurations they produced live in [`configs/`](../configs/) (609 files for the clla track).
- Each experiment run writes telemetry under `runs/` (git-ignored, disk-only). The in-repo record
  is the protocol plus its measured outcome, which is how this project keeps results reproducible
  without committing raw run logs.

## Where each family lives

| Family / era | Question | Primary files |
|---|---|---|
| E1 to E8 | Minimal spiking network, world, deterministic framework | `docs/anima-e6-protocol.md` ... `docs/anima-e8-protocol.md` |
| E9 to E14 | Structural growth, homeostasis, evolution | `docs/anima-e9-protocol.md` ... `docs/anima-e14-protocol.md` |
| E15 to E17 | Audits and corrections | `docs/anima-e15-audit.md` ... `docs/anima-e17-audit.md` |
| E18 to E24 | Later capabilities, regime capacity, ladder | `docs/anima-e18-design.md` ... `docs/anima-e24-protocol.md` |
| CLLA track | Capability-architecture records | `docs/anima-clla-*.md` |
| Cross-cutting | Decisions and autonomy log | `docs/decision-log.md`, `docs/autonomy-log.md` |
| World-survival campaign | The 3D foraging organism this artifact is about | [`docs/white-paper.md`](../docs/white-paper.md), [`crates/anima-world/src/bin/world_survival.rs`](../crates/anima-world/src/bin/world_survival.rs) |
| Older manuscript | Peer-style paper lineage (separate) | [`paper/`](../paper) |

Not every number between E1 and E24 has a protocol file; some audits replaced earlier designs. If a
document is missing for a given number, the nearest earlier and later files bracket it.

## Reproduce

The single most useful entry point is the frozen identity check and the 198-test suite:

```bash
cargo test --workspace
./target/release/world_survival 20260924 77 60
# WL2 RESULT beats=60 died_at=none final_energy=40.0 food_touches=0 novel_flags=0
```

The exact gates, seeds, and constants for every claim in the white paper are in
[Appendix: every number, explained](../docs/white-paper.md#13-appendix-every-number-explained).