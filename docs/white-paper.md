# ANIMA: Teaching a Tiny Brain to Find Food, All By Itself

*A research story from the beginning, with every idea explained as it appears. If a term feels foreign, keep going; the first time we use it, we define it.*

## Abstract

How much of an agent can you grow with a world, a body, and one need, and no engineer telling the
brain what to do? We built ANIMA, a developmental spiking neural organism written from scratch in
Rust, and studied what it can learn on its own. The organism lives in a 3D arena, senses its
surroundings, controls its own body, finds and eats food, and dies when its energy runs out. It is
deterministic and experiments are pre-registered, so every result can be re-run. We show that the
organism learns to forage, that persistent food supports a genuine feeding loop, and that a home
and rest drive can emerge when returning home is rewarded. Two measured limitations block further
progress: the internal layer saturates, so selective within-life learning is not possible yet, and
the fitness economy saturates once food is reachable, so evolution stops improving. The conclusion
is scoped to foraging and self-organization; we do not claim general intelligence or biological
equivalence.

## Contributions

- A self-organizing spiking neural foraging organism, implemented from scratch in Rust with no
  hand-designed internal architecture, no pretrained models, and no external memory modules.
- A deterministic, pre-registered experimental framework (frozen identity, env-gated switches,
  seeded randomness) that makes every claim re-runnable.
- An empirical characterization of the selective-learning wall: the internal band saturates, so
  reward-gated plasticity becomes non-selective, and no tested competition, threshold, amplitude,
  or heterogeneity lever fixes it.
- An empirical characterization of the evolutionary wall: fitness saturates once food is reachable,
  pinning evolution at a measured ceiling regardless of the mechanism stack tested.

## 1. The big question (why we built this)

We wanted to see if a computer creature could teach itself to survive. Not a real animal, and not
a robot with instructions. A pretend animal, inside a little 3D world, with a tiny brain made of
simulated nerve cells. The foundational goal we gave it was food: find food and eat it before your
energy runs out. Everything else it has to figure out on its own. Later, as a separate experimental
extension, we added a home and rest drive (section 6); that second need was introduced after the
food-foraging foundation, and the game rules changed to include it. This section tells the
foundational story first, and the extension appears where it happened in time.

Why care? Real animals do not come with a designer who tells every nerve cell what to do. Their brains organize themselves. They grow, learn, and adapt. Most "artificial brains" skip that part: an engineer decides the wiring. We refused that. We want to know whether a brain can organize itself when all it gets is a world, a body, and hunger.

If it works, we learn something about how learning might actually happen. And it edges toward the dream of artificial life that keeps getting better on its own, the way evolution improved animals over millions of years.

## 2. Who, where, when

One engineer built this, in a programming language called Rust (people like it because it is fast and safe). The experiment lives in this repository. The heart of it is one big file: `crates/anima-world/src/bin/world_survival.rs`.

The work in this document came from a long campaign. Every experiment was run, measured, and written down before it started, including the failures. That journaling habit, more than any single result, is why we trust the conclusions here.

## 3. Meet the creature

Picture a tiny bug in a very large empty box. The box (the arena) is 100 units on a side.

The creature has a body that can move in 3D. It can fly forward, turn left and right, tilt up and down, and stop. Every moment it stays alive costs a little energy. When the tank hits zero, it dies. It has no other clock.

It has senses. A retina (a grid of pixel cells) sees objects and edges in front of it. It feels its own body position. If we switch on the smell option, it can also smell food. Food sits somewhere in the arena. If the creature gets within a small distance of it (we call that distance the food radius), it gains 30 energy. Eating is the only reward that exists. Nothing else in the world pays.

One rule matters more than any other: nobody designed this brain. No one told a single nerve cell "turn right when you smell food." The creature has to discover that by itself, or starve.

## 4. How the brain works, with every word defined

A neuron is a single nerve cell. Think of it as a light bulb that blinks when it builds up enough electric charge. A spike is one blink. Neurons talk to each other only by blinking.

A synapse is the wire between two neurons. The strength of the wire decides how much one neuron's blink nudges the next one. Stronger wire, bigger nudge.

A spiking neural network is a brain made of these blinking bulbs and their wires. When enough blinks land on one neuron, it charges up and blinks too.

The specific rule each bulb follows here is called a LIF neuron. LIF is short for Leaky, Integrate, Fire. Leaky means the charge slowly drains away. Integrate means blinks add up. Fire means the bulb blinks once the charge crosses a threshold. It is a simplified pretend neuron, but it is close enough to real biology to stay interesting.

STDP is the learning rule. The phrase people use is "neurons that fire together wire together." If two connected neurons blink at almost the same time, their wire gets stronger. If they blink far apart in time, the wire gets weaker. That is how the brain changes itself. No human tunes a single wire by hand.

Plasticity is just the fancy word for "the brain can change." STDP is one kind of plasticity.

The brain has three groups of bulbs. Inputs bring in what the eyes see. An internal pool of bulbs does the processing. Outputs turn into body movements, like steer or thrust. The whole loop looks like this: eyes to brain to movement to the world, energy goes down, the brain changes itself along the way, food fixes the energy if the creature finds it. Death happens if it does not. Then a new life begins.

Two words that keep coming up, because they are the reason the science is trustworthy:

Seeded randomness. The inputs do not fire on a fixed schedule. They fire at random moments, like a firecracker popping at unpredictable times, but with a stable average. Real neurons behave roughly this way. The randomness starts from a number called the seed, so we can replay the exact same randomness every time.

Determinism. Same seed plus same settings equals the same result, every single run. That sounds small, but it is huge for experiments. When we see a difference between two runs, we know the difference came from what we changed, not from luck.

Env-gate. Every experiment is built as a switch you can turn on or off. With all switches off, the creature behaves exactly like the original frozen version. This protects the baseline: nothing we test can silently ruin the thing we started from.

## 5. The rules we refused to break

Before any experiment, we locked four rules. They are the reason you can trust anything in this document.

One: no hand-designed brain. We never tell a specific neuron what to do. Only the general learning rules exist, and the wiring grows by itself.

Two: no pre-trained models. We do not borrow a smart model someone else trained.

Three: no extra memory boxes. The creature cannot carry a notebook and write down where food is. Its only memory is the changing wires of its own brain.

Four: reward equals food, and nothing else. We never reward "turned toward the food," "looked smart," or "moved a lot." Only eating counts. This is the foundational rule. The later home and rest drive (section 6) was introduced as an explicit experimental extension, with returning home as its own reward, and the paper marks it as a separate chapter of the story rather than part of the original charter.

And rule five, the one under everything: every claim has to be measured and repeatable. No "trust us." The numbers have to come from a real run you can re-run and get the same answer.

These rules are also why the failures in section 7 are useful. They are honest, and none of them could have been faked by changing the setup.

## 6. What the creature can do (the wins)

The creature did real things, and some of them surprised us.

It found and fixed its own steering mix-up, which is not what we set out to do but is the most important moment of the whole project. For a long time, the creature kept flying away from food. We eventually traced that to a single bug: when food was to its right, the command that should have turned right actually turned left. One sign error had been silently sabotaging months of runs. The moment we fixed it, the creature started chasing food like a normal hungry thing. This is the bug we keep telling people about, because it explains why so many early "failures" were really one bug in disguise.

It forages well. Under a teaching schedule (a curriculum where food starts close and moves farther as it improves), a single creature ate as many as 126 meals in one life. That is a strong forager by any measure we have.

It builds a real feeding loop when food stays put. By default, food teleports to a random new spot when eaten. That makes remembering anything useless. We built a switch, `SL2_STASIS`, that keeps food where it is after the first find. With that, the creature went from 3 meals per life to 36. It had found a patch and, mostly, stayed on it.

It lives long when the difficulty is honest. With a bigger energy tank and food that moves a little farther each time it wins (a ladder of difficulty), one creature survived 2,230 beats. Its usual lifespan was 60 to 200. It did not die of boredom; it died because the ladder got genuinely too hard.

It learned a home. We added a second need: a nest the creature should return to for rest, and returning restores its stamina. Then we made the interesting choice of also rewarding returns. One lineage of the creature started coming home repeatedly. It foraged out, came back, rested, and went out again. That is the first real hint of the behavior we actually care about: go find food far away, then find your way home.

Hunger changes how it searches, the way it should. When well fed, the creature settles down and stops wandering, which keeps it near a food patch. When hungry, it searches wide to find the next meal. Real foragers do exactly this.

## 7. What did not work (the honest failures)

Science does not work unless you write down the failures too. Here they are.

A curiosity switch, which rewarded the creature for seeing thing it had not seen before, made it explore more but did not make it survive better. The reward was interesting, and it just did not translate into food.

Several learning rules, in the family of "make the wires change when eating happens," all did something. The wires really did change. We could measure it. But the change was too small and too scattered to matter to survival, so evolution never noticed it. This pattern came up again and again: easy to make the brain change, hard to make the change count.

An area-restricted-search reward, which is the fancy name for "reward the creature for coming back to food quickly after leaving it," changed the score numbers but never let anything escape the wall (section 8). It made us better at measuring, and that was about it.

Even a mushroom-body-style brain, inspired by how real fly brains learn, fired correctly but could not be selective. It ran into the same wall.

The deep lesson from all of these: learning rules are cheap. Making learning matter to survival is the hard part, and it is blocked by two walls that we finally measured and understood. That is next.

## 8. The two walls (the main finding)

After dozens of experiments, we found two walls standing between this creature and a self-improving life. Understanding why they exist is the real contribution of this project.

Wall one: the brain is too excited. Every light is always on.

Remember the light-bulb neurons? In this creature's brain, every single bulb blinks all the time. We proved it with a probe: 256 out of 256 internal neurons were firing at every beat. Nothing was ever quiet.

Here is why that kills learning. A learning brain needs differences. When you eat, the brain should strengthen the few wires that led to the meal and leave the rest alone. But if every neuron was firing, then everything was active right before eating, so the reward strengthens everything equally. That is not learning. It is a participation trophy for the whole brain, and nobody learns anything. No differences means no selective learning, and no selective learning means no real learning.

We attacked that wall four ways, and measured each attempt.

Competition first. The idea was that neurons could shut their neighbors down when they fired together. It did not help: the strong input just re-fired them.

Raising the threshold second. Make it harder to fire. It did not help either, because the input was strong enough to fire them anyway.

Turning the input down third. This created a cliff instead of a slope. The brain went from everything-on to everything-off, with no middle ground where some neurons were quiet and others were not. There was no sweet spot.

Making each neuron different fourth. Give every bulb a slightly different firing difficulty, the way real brains have. At the edge of that setup, the brain started quiet. Good. Then, around beats 20 to 30, a phase transition happened. Like a pile of sand that suddenly avalanches, every neuron flipped on at once and the brain was saturated again.

So the honest summary of wall one: the brain is stuck in everything-on, all the time, and no amount of competition, thresholds, or variety fixed it. Selective learning cannot happen inside that regime. This is the deeper of the two walls.

Wall two: evolution stops caring once food is reachable.

The creature evolves across generations (section 9). Fitness, the score that decides who gets to be a parent, is roughly this: energy left at the end of life, plus 3 points per meal. Here is the problem. Once any creature can reach food, energy fills up, and every decent creature scores about the same. There is no gradient left for evolution to climb. The population settles around a fitness of about 121 to 138, depending on which scoring formula you use, and stays there no matter what learning rule we added.

The plain version: once finding food is good enough, evolution cannot tell a great forager from an average one, so it stops improving. The scoring itself is the ceiling.

Together, the two walls explain essentially every failure in this project. The creature cannot learn selectively, and it cannot keep evolving. Those two facts do most of the explaining.

## Results at a glance

| Capability | Status | Evidence |
|---|---|---|
| 3D embodiment | Demonstrated | 6-DOF body, physics arena, retina sensing |
| Movement and steering | Demonstrated | after fixing a sign bug, it steers toward food |
| Food acquisition | Demonstrated | up to 126 meals in one life under a curriculum |
| Persistent food foraging | Demonstrated | 3 to 36 meals per life with stable-food mode |
| Long survival | Demonstrated | up to 2,230 beats under a difficulty ladder |
| Home / rest behavior | Partial | recurring returns on seed 7; not on seed 20260924 |
| Selective within-life learning | Not demonstrated | internal band saturates; reward is non-selective |
| Evolutionary improvement | Partial | reaches a ceiling (~121 to 137.6 by scoring family) |
| Open-ended improvement | Not demonstrated | the two walls above are not crossed |

The status column uses three words only: demonstrated, partial, or not demonstrated. "Not
demonstrated" means no measured run crossed that bar yet, not that it is impossible.

## 9. How evolution and learning fit together

There are two time-scales at work, and the whole project is about whether they can connect.

Within one life, a single creature lives about 100 beats, sees food, and its STDP changes its wires. That is practice.

Across generations, we keep a small population of 4 creatures. We score each one, keep the best as the parent, let the best two breed, and add small random changes, called mutations, to their wires. Over generations, the population can, in principle, discover better and better brains. This is the DNA layer: the thing passed down is the wiring, nudged a little each generation.

The catch we measured: evolution only sees changes big enough to change survival. The tiny wire-changes from in-life learning are far too small and scattered to move the fitness score, so evolution ignores them. The two time-scales stay disconnected. That is why the creature did a lot of individual things correctly and yet never got fundamentally better over generations.

## 10. What it all means

The creature is real in the way that matters: it moves, senses, finds food, survives, and dies. That part fully works, and it took real engineering to get there.

We found and fixed a genuine bug that had been quietly sabotaging everything for a long time.

We measured, rather than guessed, the two walls that block a self-improving brain: the everything-on saturation that makes selective learning impossible, and the good-enough-is-enough ceiling that stops evolution.

That is a cleaner result than "we tried a bunch of stuff." We now know exactly what to attack next, and why.

## Research lineage

The organism was shaped by experiment, not designed around a fixed answer. Each family below posed a question, tried an intervention, recorded a result, and made a decision. The experiment index (`experiments/README.md`) and the correction notes (`research/corrections.md`) carry the protocols and failed hypotheses in more detail.

- **Foundational dynamics (E1 to E5 era).** Question: can a minimal LIF network tie sensation to movement at all? Intervention: small static spiking nets in the arena. Result: the loop worked, but the organism was sensitive to a sign bug in steering. Decision: fixed the bug, then built determinism and env-gating around the working loop.
- **Foraging shaping (curriculum).** Question: can shaping help the organism reach food? Intervention: food starts close and moves farther as the creature improves. Result: strong local pursuit, up to 126 meals a life. Decision: kept shaping as the training scaffold.
- **Persistent food (SL2_STASIS).** Question: does stable food enable a feeding loop? Intervention: keep food in place after the first find, with a re-entry guard. Result: 3 to 36 meals a life. Decision: this became the base for the later experiments.
- **Selective-learning mechanisms (chemical gating, eligibility, R-STDP, reward gating).** Question: can reward shape within-life learning? Intervention: several reward-gated plasticity rules. Result: the wires changed, but the change was too small and non-selective to matter to survival. Decision: measured the saturation wall that caused it.
- **Mushroom-body motif (SL2_MB).** Question: does the fly's sparse expansion + input-locus credit help? Intervention: a 256-neuron sparse expansion with reward-gated input plasticity. Result: fired correctly, but non-selectively under saturation. Decision: recorded as unproven until the band is de-saturated.
- **De-saturation campaign (SL2_IINH, THETA, AMPL, HET).** Question: which lever removes the everything-on regime? Intervention: competition, threshold, input scaling, and heterogeneity. Result: all four bounded negative; the band is temporally bistable. Decision: the phase transition itself is the next target.
- **Home and rest (SL2_NEST, SL2_NESTFIT).** Question: can a second need drive return-home behavior? Intervention: a nest landmark, a stamina constraint, and a homing reward. Result: recurring returns on seed 7; seed 20260924 stayed parked. Decision: recorded as partial and seed-fragile.

This is the lineage that constrained the organism step by step. It is kept visible precisely because the failures, corrections, and measured walls carry the scientific value.

## 11. What we would do next

Break wall one by keeping the brain cool. If we can stop the whole brain from flipping on at once, selective learning becomes possible for the first time. That means attacking the phase transition itself, or keeping most neurons quiet most of the time.

Fix wall two by changing the scoring. Give evolution a reason to keep caring, by rewarding things like how steadily the creature keeps eating over its whole life, not just whether it ever reached food.

Give it a real memory, but a network-native one. Now that food can stay put, remembering where food is would actually pay off. The memory has to live in the brain's own wires, because external maps are against our rules.

And on the engineering side, run experiments faster. Real fly-brain simulations run on graphics cards. Ours runs on a single thread. Parallelizing would let us test many more ideas in the same time.

## Limitations and threats to validity

The claims here rest on a small, deterministic setup, and the boundaries below should shape how you
read them. We deliberately distinguish what was **not demonstrated** from what is **impossible**:
nothing in this list says a behavior cannot exist, only that no measured run in this project showed
it.

- Small neural population. The largest internal pool we ran in evolution was 256 neurons; the
  default is 40. That is a tiny network, and the saturation wall is partly a property of its size.
- Small evolutionary population. Each generation holds only 4 creatures, with 2 evolution seeds.
  That is a weak statistical base; a result that appears on one seed and not another is real here,
  but it does not establish robustness.
- Synthetic 3D environment. The arena, retina, and body are simplified models, not a physical
  world. Real-world confounding (noise, partial observability, contact) is absent.
- Simplified LIF model. We use Leaky-Integrate-Fire neurons, not a detailed biophysical model.
  LIF captures spiking dynamics well enough for this question but has no dendritic computation,
  no sustained local inhibition of the kind real circuits use, and no analog neuromodulation beyond
  what we added explicitly.
- Limited seeds. Only seeds 20260924 and 7 were used across the campaign. Seed-dependence is a
  documented finding (the home behavior on one seed, not the other), and it tempers the generality
  of every positive result.
- Current saturation regime. The organism spends most of its time in an everything-on regime, which
  prevents selective learning. The de-saturation campaign bounded four levers negative, but that is
  evidence about this setup, not a general proof.
- Fitness formulation dependence. The observed ceiling depends on the scoring function. A
  different fitness shape could move it, which is why the fitness economy is listed as an open
  target rather than a law.
- No demonstrated continual selective learning, and no demonstrated open-ended evolution. These are
  the two central gaps behind the two walls. We show why they are blocked, and we do not claim they
  are solved.

Finally, reproducibility guardrails: every result is a seeded, env-gated run; the frozen identity
line and the 198-test suite guard against silent drift; and the appendix lists the exact constants
and commands.

## 12. How you can see it yourself

You need Rust installed. Then:

```bash
cargo build --release -p anima-world --bin world_survival
```

The first thing to check is the frozen identity line. It is the contract that says nothing silently changed:

```bash
./target/release/world_survival 20260924 77 60
# prints:
# WL2 RESULT beats=60 died_at=none final_energy=40.0 food_touches=0 novel_flags=0
```

Every feature is a switch you put in front of the command. For example, stable food:

```bash
SL2_STASIS=1 ./target/release/world_survival --explore --smell 20260924 77 60
```

Every number and switch you need to re-run any result in this document is in the appendix right below.

## 13. Appendix: every number, explained

This appendix carries the exact values behind every claim above, each defined the first time it appears. It is long, and that is the point. If you want to remake any experiment, this is the recipe.

Life. A creature starts with 100 energy. Each beat drains 1.0. Food gives +30 when the creature is within distance 6.0. Death happens at exactly 0. With the big-tank switch `SL2_PROG`, life starts at 4000 energy instead.

Scoring. A dead creature always scores 0, no exceptions. For a living creature, the plain score is final_energy + 3 x meals. The density switch (`SL2_DENSITY`) adds 20 x meals squared, divided by the number of beats. The persistence switch (`SL2_ARS`) adds 4 x the sum of exp(−gap ÷ 20), where gap is the number of beats since the previous meal; quick returns to food earn more. The homing switch (`SL2_NESTFIT`) adds 40 x nest_returns, where nest_returns counts how many times the creature came back home after truly leaving the nest radius.

Evolution. Four creatures per generation. The best creature is carried into the next generation unchanged. If the same winner holds the top spot for 5 generations, it is carried with mutations instead, so it cannot stall forever. Children copy a parent's wiring, and each copied wire is nudged by ±10%, with probability 0.10. The best parent has 2 children; the runner-up has 1. Food starts at distance 8 and moves 3 farther each generation, capped at 40.

Learning at meal time. When the creature eats, the wires feeding recently active neurons get a boost. The boost is 0.03 x e x 8, where e is the neuron's recent-activity trace, a number between 0 and 1. So the largest single boost is about 0.24 per wire. The dopamine version (`SL2_RSTDPP`) pulses a reward signal to 1.0 at the meal, decays it by ×0.6 each beat, and scales the learning gain by 1 + 3 times that signal, capped at 4x.

Home mode (`SL2_NEST`). Stamina starts at 100. Away from the nest, it drains 0.4 x speed per beat, where speed is the creature's velocity. Inside the nest radius of 8.0, stamina refills 60 per beat. Below stamina 25, movement weakens; at 0, it stops. Returning counts only after the creature has truly left the radius, which prevents a parked creature from farming free points.

Settle-when-fed (`SL2_HMEXP`). Wandering noise is 0.4 + 0.6 x hunger, where hunger is 1 minus energy divided by start_energy. Fed creatures wander at 40 percent of normal; starving creatures wander at full strength.

The big-brain mode (`SL2_MB`). The internal pool grows to 256 neurons, and each one connects to about 6 inputs, which is a sparse wiring pattern inspired by real brains.

Network defaults. Input strength (amplitude) is 52. The probability that any two neurons are connected at birth is 0.038. The firing threshold is 1.0. These three numbers control the creature's baseline temperament.

Probes. These are debugging switches that print extra lines. They all write to the error stream, so you capture them like this: `2>&1 >/dev/null | grep KCPROBE`. `KCPROBE` prints how many internal neurons fire in a beat. `SW_PROBE` prints how much the wiring changed. `MB_PROBE` prints what the learning bump actually touched. `NEST_PROBE` prints the home position and stamina.

Identity and seeds. The frozen identity command is `./target/release/world_survival 20260924 77 60`, and its exact output is `WL2 RESULT beats=60 died_at=none final_energy=40.0 food_touches=0 novel_flags=0`. The two evolution seeds used throughout are 20260924 and 7.

## Conclusion

We built a self-organizing spiking organism and measured, honestly, what it can and cannot do. It
forages. It survives. With stable food it builds a feeding loop, and with a home and rest drive it
returns home on one of two seeds. Those are real, reproducible results. They sit inside two clearly
measured boundaries: the internal band saturates, so selective within-life learning is not
available yet, and the fitness economy saturates once food is reachable, so open-ended evolution is
not available yet. The contribution of this project is not that the walls are crossed; it is that
the walls are mapped with evidence, that the explanatory dead ends are recorded instead of hidden,
and that every claim can be re-run. The current research boundary is that precise: a capable
forager, with selective learning and open-ended improvement both still open problems.

## Acknowledgements

This research used the LLM inference API from freeinterference.org, and it helped a lot. Their
service let us work through the ideas in this paper, the dead ends and the alive ones, at a pace
that would not have been possible on our own. Thank you.

---

*Everything in this document came from a run we can re-run. The code is the final source of truth,
and it is all in this repository, waiting for the next person who wants to pick up the question.*