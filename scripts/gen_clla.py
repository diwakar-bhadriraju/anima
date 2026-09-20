#!/usr/bin/env python3
"""CLLA frozen-config generator (docs/anima-clla-protocol.md §2).

13 configs, exactly per protocol:
  - il/bac/bca arms x seeds {20260912, 424242, 9001}: e24 curriculum
    with assembly_protect = true (only increments on the committed
    e24 configs: the three CLLA fields; everything else identical).
  - d arm x 3 seeds: novel D = channels 0-15 (A u C), 20 Hz, 500 ms,
    40 reps, interleaved. New pattern; no other change.
  - ident arm: il curriculum seed 20260912 with assembly_protect
    NOT SET (implicit false) -> must reproduce the committed
    e24-s20260912-il run byte-identically (identity gate).

Base configs: the committed e24 configs (configs/e24-s<seed>-{il,bac,bca}.toml).
D arm is derived from the e24-il base with the S1 present list replaced.
"""
import io, os, sys

SEEDS = [20260912, 424242, 9001]
ARMS = ["il", "bac", "bca"]
CLIA = """
assembly_protect = true
p_max_frac = 0.75
w_consolidate_min = 0.05"""

def add_clla(toml_text: str) -> str:
    # Insert CLLA fields at the top of the [v2] table (after "[v2]"
    # token + following newline), preserving all existing lines.
    marker = "[v2]"
    assert marker in toml_text, "config must have a [v2] table"
    idx = toml_text.index(marker) + len(marker)
    assert toml_text[idx] == "\n", "expect [v2] followed by newline"
    return toml_text[:idx] + CLIA + toml_text[idx:]

def d_pattern() -> str:
    return """[[pattern]]
id = "D"
channels = []
channel_ids = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]
rate_hz = 20.0
duration_ms = 500
jitter_ms = 2.0
"""

def main() -> None:
    made = []
    for seed in SEEDS:
        for arm in ARMS:
            src = f"configs/e24-s{seed}-{arm}.toml"
            assert os.path.exists(src), src
            text = io.open(src, encoding="utf-8").read()
            out = add_clla(text)
            out = out.replace(f'exp_id = "e24-s{seed}-{arm}"', f'exp_id = "clla-s{seed}-{arm}"')
            dst = f"configs/clla-s{seed}-{arm}.toml"
            io.open(dst, "w", encoding="utf-8").write(out)
            made.append(dst)
        # D arm: derived from e24-il with pattern D.
        src = f"configs/e24-s{seed}-il.toml"
        text = io.open(src, encoding="utf-8").read()
        text = add_clla(text)
        text = text.replace(f'exp_id = "e24-s{seed}-il"', f'exp_id = "clla-s{seed}-d"')
        # Replace S1 presentation list (A, C interleaved) with D; pattern D
        # appended at the end. Patterns A/B/C stay defined (unused).
        text = text.replace('present = ["A", "C"]\nreps = 20\norder = "interleaved"',
                            'present = ["D"]\nreps = 40\norder = "interleaved"')
        text = text.rstrip() + "\n\n" + d_pattern()
        dst = f"configs/clla-s{seed}-d.toml"
        io.open(dst, "w", encoding="utf-8").write(text)
        made.append(dst)
    # ident arm: il curriculum, CLLA fields omitted (flag=false default).
    src = "configs/e24-s20260912-il.toml"
    text = io.open(src, encoding="utf-8").read()
    text = text.replace('exp_id = "e24-s20260912-il"', 'exp_id = "clla-ident-il"')
    dst = "configs/clla-ident-il.toml"
    io.open(dst, "w", encoding="utf-8").write(text)
    made.append(dst)
    print(f"wrote {len(made)} configs")
    for m in made:
        print(" ", m)

if __name__ == "__main__":
    main()