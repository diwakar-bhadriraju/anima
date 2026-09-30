# ANIMA research lineage

This folder indexes how the project was constrained by experiment. It is not a second copy of the
white paper; it names the lineage and points at the primary records. The fail-open summary is in
the white paper's [Research lineage](../docs/white-paper.md#research-lineage) section and the
corrections journal is [corrections.md](corrections.md).

## The throughline

1. **Foundations (E1 onward).** Build a deterministic spiking network that senses a 3D world and
   drives a body. Records: `docs/anima-e6..e8-protocol.md`, `docs/decision-log.md`.
2. **Growth and evolution.** Let structural plasticity grow the topology and let generations select
   it. Records: `docs/anima-e9..e14-*`, `docs/anima-e18..e24-*`.
3. **Foraging (world-survival).** Focus the organism on food in an arena. This is the artifact's
   core; the white paper covers it end to end.
4. **The two walls.** Selective in-life learning and open-ended evolution both saturate. The
   de-saturation campaign and the fitness economy found the precise reasons.

## Where the lineage evidence lives

- Experiment protocols and records: [`docs/anima-e*.md`](../docs/)
- Decisions and autonomy log: [`docs/decision-log.md`](../docs/decision-log.md),
  [`docs/autonomy-log.md`](../docs/autonomy-log.md)
- Corrections journal: [corrections.md](corrections.md)
- The readable narrative with measured numbers: [../docs/white-paper.md](../docs/white-paper.md)
- The older peer-style manuscript (a separate lineage): [`paper/`](../paper)

## A note on failed work

The value of a lineage is in what stayed and what fell away. We deliberately keep the failures:
every negative result here either stopped a direction or re-shaped it. None of them is a bug to
hide; they are the evidence that the organism was discovered, not designed.

## Navigation

- Want the story? Read the [white paper](../docs/white-paper.md).
- Want a specific experiment? Use the [experiment index](../experiments/README.md).
- Want the corrections? Read [corrections.md](corrections.md).