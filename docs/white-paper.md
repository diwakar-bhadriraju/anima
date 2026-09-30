# ANIMA — Teaching a Tiny Brain to Find Food, All by Itself

*A research story told from the very beginning, with every idea explained.
If a term feels strange, keep reading — we explain each one the first time we use it.*

---

## 1. The Big Question (Why are we doing this?)

**What are we trying to do?** We want to build a living thing — not a real animal, but a
computer creature — that *teaches itself* to survive. Specifically: it lives in a little 3D world,
has a tiny brain made of pretend nerve cells, and the ONLY goal we give it is **find food and eat
it before you run out of energy.** Nothing else. It has to figure out everything else on its own.

**Why?** Because real animals (like a fly or a mouse) don't have a designer telling every nerve
cell exactly what to do. Their brains **organize themselves** — they grow, learn, and adapt. We
want to see if a computer brain can do the same. If we can grow a self-organizing brain that learns
to forage, that teaches us something deep about *how learning and thinking could happen* — and it's
a step toward the dream of "artificial life that really learns," not just a program we hand-wrote.

The long-term hope: a creature that doesn't just survive, but keeps getting better and better on
its own — the way evolution made animals over millions of years.

---

## 2. Who, Where, When

- **Who made it?** A small research project built by one engineer (in Rust, a programming
  language famous for being fast and safe), aided by careful, recorded experiment discipline.
- **Where?** This repository. The heart of the experiment lives in one big file:
  `crates/anima-world/src/bin/world_survival.rs`.
- **When?** The work in this document is from a long, honest research campaign — each experiment
  was run, measured, and written down (or "registered"), even the ones that failed.

---

## 3. Meet the Creature (What we built)

Imagine a tiny bee-like robot in a big empty box (the arena, 100 units on each side). It has:

- **A body** that can move in 3D — fly forward, turn left/right, tilt up/down, and stop. It burns a
  little energy every moment it's alive. When energy runs out, it **dies**.
- **Eyes and senses** — it looks at the world through a retina (a grid of pixels that see objects
  and edges), feels its own body position, and can smell food if we switch that on.
- **A brain** — a network of *pretend nerve cells* (see "How the brain works" below).
- **A goal** — food sits somewhere in the arena. If the creature flies close enough (within a
  small distance called the "food radius"), it gets +30 energy. Food is the only reward that exists.

**The most important rule:** we did NOT hand-design the brain. No one told the nerve cells "turn
right when you smell food." The creature has to *discover* all that by itself.

---

## 4. How the Brain Works (Every idea explained)

- **Neuron (nerve cell):** one tiny unit of the brain. Each neuron has a little electric charge.
  When enough charge builds up, it "fires" — sends a signal (a spike) to its neighbors.
  Think of a neuron as a single light bulb that blinks on when it's charged enough.
- **Spike:** the blink. Neurons communicate only by firing.
- **Spiking neural network:** a brain made of these blinking bulbs, connected by wires (synapses).
  Every blink can turn other bulbs toward firing.
- **LIF neuron:** the specific simple rule each bulb follows. L = Leaky (charge slowly drains),
  I = Integrate (blinks add up), F = Fire (when it crosses the threshold, it blinks).
  It's a pretend neuron that's still realistic enough to be interesting.
- **Synapse (wire):** the connection between two neurons. The strength of a wire decides how much
  one neuron's blink nudges the next one. Stronger wire = bigger nudge.
- **STDP (a learning rule):** the phrase "neurons that fire together wire together." When two
  connected neurons blink at almost the same time, their wire gets STRONGER. If they blink far
  apart in time, the wire gets weaker. This is how the brain changes itself — no one tunes the
  wires by hand.
- **Plasticity:** just a fancy word for "the brain can change." STDP is one kind of plasticity.
- **Input → internal → output:** the brain has three layers of bulbs. Inputs bring in what the eyes
  see. An internal pool of bulbs processes it. Outputs turn into body movements (steer, thrust...).
- **Poisson trains:** the way inputs fire is random in a specific statistical pattern — like a
  firecracker that pops at random moments, but with a predictable average. It mimics how real
  neurons behave. The randomness is "seeded," meaning we can replay the exact same randomness
  every time (that's how we keep experiments fair and repeatable).
- **Determinism:** same starting number (seed) + same settings = same exact result, every single
  time. It's what makes the science trustworthy: when we see a difference, we know it came from
  what we changed, not from luck.
- **Env-gate (a setting switch):** every experiment is built as a switch you flip on/off. With all
  switches off, the creature behaves exactly like the original "frozen" version. This guarantees
  we never ruin the baseline while testing new ideas.

**The whole picture:** eyes → brain → movement → world → energy → death, in a loop, over and over,
with the brain changing itself (via STDP) as it goes. That loop IS the experiment.

---

## 5. The Rules We Refused to Break (Why we trust our results)

Before we did any experiment, we locked these rules (the "charter"):

1. **No hand-designed brain.** We never tell a specific neuron how to behave. Only the general
   learning rules exist; the wiring grows by itself.
2. **No pre-trained models.** We don't use a smart model someone else trained on the internet.
3. **No extra memory boxes.** The creature can't have a notebook it writes locations into — its
   only "memory" is the changing wires of its own brain.
4. **Reward = food and nothing else.** We never reward "turning toward food" or "looking smart."
   Only eating counts. Anything the creature learns to do, it learns because eating is worth it.
5. **Every claim must be measured and repeatable.** No "trust us" — the numbers must come from a
   real run you can re-run and get the same answer.

These rules are exactly why the failures in this report are *useful*: they're honest, and they
couldn't have been faked by cheating the setup.

---

## 6. What the Creature Can Do (Our wins)

Here are the things that actually worked, told plainly:

1. **We found and fixed a real bug (the steering mix-up).** The creature was accidentally turning
   **away** from food whenever food was to its right. All our early runs "failed" because of this
   one sign error. Once fixed, the creature began chasing food properly — the single most important
   fix of the whole project.

2. **It can find food well.** When food is placed reasonably close, the creature learns to find and
   eat it many times in one lifetime — up to **126 meals** in a single life under a teaching
   schedule (a "curriculum" where food starts easy and gets harder). That's a strong forager.

3. **Stable food = a real feeding loop.** Normally, when the creature eats, food teleports away to
   a random spot. That makes "remember where food is" pointless. We built a switch (`SL2_STASIS`)
   where food stays put after the first find. The creature went from 3 meals per life to **36** —
   a real "stay here and keep eating" loop.

4. **It can live long.** With a bigger energy tank and food that moves a little farther each time
   it wins (a "ladder" of difficulty), the creature survived **2,230 beats** — a much longer life
   than the usual ~60-200 — and only died when the challenge genuinely beat it.

5. **It can learn a home.** We gave the world a second need: a "nest" the creature should return
   to to rest (returning restores its stamina/energy). And — the striking part — when we **rewarded**
   returning home, one lineage of the creature actually started **coming home repeatedly**. This is
   the first real hint of "forage far away, then come back home," split from just "stay alive".

6. **Hunger changes how it searches.** When well fed it *settles down* (stops wandering, stays
   where it is — good for staying near food). When hungry it *searches wide* to find the next meal.
   This matches how real foragers behave.

---

## 7. Things We Tried That Didn't Work (Our honest failures)

Science only works if you report what failed too. Here's the honest list:

- A "curiosity" switch that rewarded the creature for seeing things it hadn't seen before — didn't
  make it any better at surviving. (A neat idea, but the reward didn't help.)
- Several "learning rules" (ways to make the brain change when it eats): dopamine-gated STDP,
  reward-gated eligibility, and others. They all **did something** — the wires really did change —
  but the change was too small and too scattered to help survival in the long run.
- A "search close after winning" reward (area-restricted search) — it changed the score numbers
  but never let anything escape the wall (see section 8).
- Even a "brain with a mushroom-body-style expansion" (inspired by real fly brains) — it fired, but
  it couldn't be selective, and the whole thing hit the same wall.

**The deep lesson from all these failures:** making the brain's *wires* change is easy. Making that
change actually *matter* to survival is very hard — because of two walls we finally measured and
understood (next section).

---

## 8. The Two Walls (the main finding — and it's genuinely new)

After dozens of experiments, we found there are two big walls blocking the creature from becoming
a self-improving forager. Understanding *exactly why* couldn't happen is the heart of this research.

### Wall 1: The brain is "too excited" — every light is always on

Remember the light-bulb neurons? It turns out that in this creature's brain, **every light bulb is
blinking all the time.** The inputs are so strong that every internal neuron fires every single
moment. We proved this with a probe: 256 out of 256 internal neurons were active at every beat.

Why does that matter? Because a *learning* brain needs **differences**. When you eat, the brain
should strengthen the specific wires that did the right thing and leave the rest alone. But if
every neuron is firing all the time, then "everything was active right before eating" — so the
reward strengthens *everything equally*. That's not learning; it's like giving a participation
trophy to every student, so nobody learns anything. **No differences = no selective learning.**

We fought this wall hard, four ways, and measured each:
- **Competition between neurons** (neighbors shut each other down) — didn't help. The strong input
  re-fires them anyway.
- **Raising the threshold** (make it harder to fire) — didn't help. The input was strong enough to
  fire them anyway.
- **Turning the input down** — created a cliff: the brain went from "everything on" to "everything
  OFF" with no in-between. No sweet spot.
- **Making each neuron different** (so some are easier/harder to fire) — at the edge, the brain
  starts quiet (good!) but then a **phase transition** happens: like a pile of sand suddenly
  avalanching, by about beat 28 every neuron flips on at once and saturates again.

**In plain terms:** the creature's brain is stuck in "everything on, all the time," and no amount of
competition, thresholds, or variety could make it behave selectively. Selective learning simply
cannot happen while that's true. This is Wall 1 — the deeper of the two.

### Wall 2: Evolution stops improving once the food is reachable

The creature "evolves" across generations (see section 9). Fitness (how good a parent is) is
roughly *how much energy you ended with + 3 points per meal*. The problem: once any creature can
reach food, energy fills up, so fitness stops rising — **every good-enough creature scores almost
the same**, so there's no gradient left for evolution to climb. The population settles around a
fitness of ~121 to ~138 (depending on which scoring formula you use) and stays there, no matter
what learning rule we add.

**In plain terms:** once finding food is "good enough," evolution can't tell a great forager from
a decent one, so it stops improving. The scoring itself is the ceiling.

Together: **the creature can't learn selectively (Wall 1) and can't keep evolving (Wall 2).** Those
two facts explain essentially every failure we saw.

---

## 9. How Evolution and Learning Fit Together (How the creature could improve)

There are two time-scales working together:

- **Within one life (learning):** a single creature lives ~100 beats, sees food, and its STDP
  changes its wires. This is "practice."
- **Across generations (evolution):** we keep a small population (4 creatures). We score each one,
  keep the best as a parent, let the best two breed into the next generation, and add small random
  changes (mutations) to their wires. Over many generations the population can, in principle,
  discover better and better brains. This is the "DNA" — the thing that gets passed down is the
  wiring, mutated a little each generation.

**The catch our experiments exposed:** evolution only "sees" changes that are big enough to change
survival. The tiny wire-changes from within-life learning (Wall 1, non-selective) are far too small
and scattered to move the fitness score, so evolution ignores them. The two time-scales are
disconnected — which is exactly why the creature did a lot of individual things but never got
fundamentally better over generations.

---

## 10. What It All Means (Why this matters)

- The creature is a **real forager** — it moves, senses, finds food, dies, survives. That part
  fully works, and it was a lot of hard engineering to get there.
- We **found and fixed a genuine bug** that had been quietly sabotaging everything.
- We **discovered, with real measurements, the two walls** that block a self-improving brain:
  the "everything on" saturation (which makes selective learning impossible) and the "good-enough
  is enough" fitness ceiling (which stops evolution).
- This is a **cleaner, more honest result** than "we tried a bunch of stuff." We now know *exactly
  what to attack next*, and why.

---

## 11. What We'd Do Next (the road ahead)

1. **Break Wall 1 by keeping the brain "cool."** If we can prevent the whole brain from flipping
   on all at once (stop the phase transition / keep most neurons quiet most of the time), then
   selective learning becomes possible for the first time.
2. **Fix Wall 2 by changing the scoring.** Give evolution a reason to keep caring — reward things
   like "how steadily you keep eating over your whole life," not just "did you reach food."
3. **Give it a real memory (network-native).** Now that food can be stable, the creature could
   genuinely benefit from remembering *where food is* — and that memory should live in the brain's
   own wires, not in an external map (we refused external maps by rule).
4. **(Engineering) run more experiments faster.** Real fly-brain simulations run on graphics
   cards; ours is single-threaded. Parallelizing would let us test far more ideas quickly.

---

## 12. How You Can See It Yourself (simple steps)

You need Rust installed. Then:

```bash
cargo build --release -p anima-world --bin world_survival

# A frozen check that the tool works (this exact output is the "identity" we protect):
./target/release/world_survival 20260924 77 60
#   WL2 RESULT beats=60 died_at=none final_energy=40.0 food_touches=0 novel_flags=0
```

Every feature is a switch you prefix before the command. For example, the "stable food" switch:

```bash
SL2_STASIS=1 ./target/release/world_survival --explore --smell 20260924 77 60
```

If you want the full list of switches and the exact numbers behind the claims in this paper, the
deeper (more technical) companion document — `docs/white-paper.md` *was* technical; that material is
kept as the reproduction log. This readme is the human-readable version. For the raw details, look
at the code comments and git history, or ask and we'll walk through any single result.

---

## 13. Quick Facts (one-line glossary of the numbers we quoted)

| Term / number | What it means, in one line |
|---|---|
| ~100 beats / life | One lifetime is about 100 "heartbeats" of the simulation |
| Energy 100 → 0 | Start with 100 energy, lose 1 per beat, +30 when you eat; 0 = death |
| 126 meals | The most a single creature ate in one life (under a teaching schedule) |
| 3 → 36 meals | Eating jumped from 3 to 36 per life when food stayed put |
| 2,230 beats | The longest life we saw (with the big-tank + harder-ladder mode) |
| 256 / 256 | Probe showing EVERY internal neuron was firing (the saturation wall) |
| ~121–138 | The fitness ceiling evolution got stuck at, no matter what we tried |
| STDP | The "fire together, wire together" learning rule |
| LIF | The simple pretend-neuron rule (Leaky-Integrate-Fire) |
| Env-gate | An on/off switch for each experiment |
| Seed | The starting number that makes runs repeatable |

---

*This document is the readable version of a long, honest research campaign. Every claim here came
from a real, repeatable run — recorded, not remembered. The code is the ultimate source of truth,
and it's all in this repository for anyone to inspect or re-run.*