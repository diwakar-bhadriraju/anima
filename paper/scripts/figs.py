#!/usr/bin/env python3
"""Phase I paper figures from preserved-run data (scripts/gen_data.sh first).

Draws: fig2-organization, fig4-dilution, fig5-regimes, fig6-protection,
fig7-decomp into paper/figures/ as PDF+PNG.
Sources: committed instruments (rg8eval, clla_traj, papermass) and the
committed audit tables (x-synmem G3 numbers for the regime panel).
"""
import os, sys, statistics
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "data")
OUT = os.path.join(HERE, "..", "figures")
os.makedirs(OUT, exist_ok=True)

C_A = "#2a7a4b"; C_C = "#b0571f"; C_G = "#777"; C_R = "#a03030"; C_B = "#2b6cb0"

def rows_p(name):
    out = []
    with open(os.path.join(DATA, name + ".tsv")) as f:
        for line in f:
            x = line.rstrip("\n").split("\t")
            if x[0] == "P":
                out.append(dict(pat=x[1], ord=int(x[2]), t=int(x[3]), uniq=float(x[4]),
                    spikes=int(x[5]), burst=float(x[6]), ipA=float(x[7]), iwA=float(x[8]),
                    ipC=float(x[9]), iwC=float(x[10]), ipB=float(x[11]), iwB=float(x[12]),
                    irec=float(x[13]), iinh=float(x[14])))
    return out

def rows_x(name):
    out = []
    with open(os.path.join(DATA, name + ".tsv")) as f:
        for line in f:
            x = line.rstrip("\n").split("\t")
            if x[0] == "X": out.append((int(x[1]), x[2]))
    return out

# ---------------- fig2: organization ----------------
def fig2():
    fig, (a, b) = plt.subplots(1, 2, figsize=(8.4, 3.2))
    # (a) per-neuron A/C mass, A-trained drive end (papermass on e24-a)
    run, mA, mC = [], [], []
    with open(os.path.join(DATA, "regimes.csv")) as f:
        for line in f:
            x = line.rstrip("\n").split("\t")
            if len(x) < 6 or x[0] == "#": continue
            if "e24-s20260912-a-" in x[0]:
                run.append(int(x[1])); mA.append(float(x[2])); mC.append(float(x[3]))
    order = np.argsort(mA)
    x = np.arange(len(run))
    a.bar(x, np.array(mA)[order], color=C_A, width=0.8, label="A-channel mass")
    a.bar(x, np.array(mC)[order], color=C_C, width=0.8, bottom=np.array(mA)[order], label="C-channel mass")
    a.set_xlabel("neuron (sorted by A-mass)")
    a.set_ylabel("drive-end afferent mass")
    a.set_title("(a) A-trained: per-neuron masses")
    a.legend(fontsize=7, loc="upper left")
    a.set_xlim(-1, 52)
    # (b) per-presentation internal spikes, first block (baseline bac)
    r = rows_p("fe-s20260912-bac")
    pres = [x for x in r if x["pat"] == "A"][:20]
    b.plot([x["ord"] for x in pres], [x["spikes"] for x in pres], "-o", color=C_B, ms=3)
    b.axvline(20.5, color=C_G, ls="--", lw=0.8)
    b.set_xlabel("presentation (block 1: A)")
    b.set_ylabel("internal spikes per presentation")
    b.set_title("(b) count structure (baseline)")
    b.set_ylim(0, 2200)
    fig.tight_layout()
    fig.savefig(os.path.join(OUT, "fig2-organization.pdf"))
    fig.savefig(os.path.join(OUT, "fig2-organization.png"), dpi=150)

# ---------------- fig4: dilution ----------------
def fig4():
    fig, (a, b) = plt.subplots(1, 2, figsize=(8.4, 3.2))
    # (a) adjacent-state cosine, alternation (G3 table: k = 1,11,21,31,40)
    k = np.array([1, 11, 21, 31, 40]); cos = np.array([0.972, 0.983, 0.987, 0.989, 0.990])
    a.plot(k, cos, "-o", color=C_R, ms=4)
    a.set_xlabel("presentation k (A/C alternated)")
    a.set_ylabel("cos(state after C$_k$, state after A$_k$)")
    a.set_title("(a) episode identity dilutes (synaptic store)")
    a.set_ylim(0.95, 1.0)
    a.text(2, 0.9765, "G3 measured\\n(0.972 $\\to$ 0.990)", fontsize=7)
    # (b) within-block spike decay per registration (first 20 A presentations)
    for name, c, lab in [("fe-s20260912-bac", C_B, "baseline"),
                         ("rg8c-s20260912-bac", "#b8860b", "rg8c (capped)")]:
        r = [x for x in rows_p(name) if x["pat"] == "A"][:20]
        b.plot([x["ord"] for x in r], [x["spikes"] for x in r], "-o", color=c, ms=3, label=lab)
    b.set_xlabel("presentation (block 1: A)")
    b.set_ylabel("internal spikes per presentation")
    b.set_title("(b) count decay within block")
    b.legend(fontsize=7)
    fig.tight_layout()
    fig.savefig(os.path.join(OUT, "fig4-dilution.pdf"))
    fig.savefig(os.path.join(OUT, "fig4-dilution.png"), dpi=150)

# ---------------- fig5: regimes ----------------
def fig5():
    # end-state A/C masses (same cell/seed) — measured by papermass on the
    # e24 corpus (matches committed x-synmem G3 table at print precision)
    rows = {}
    with open(os.path.join(DATA, "regimes.csv")) as f:
        for line in f:
            x = line.rstrip("\n").split("\t")
            if len(x) < 6 or x[0] == "#": continue
            name = "A-only" if "e24-s20260912-a-" in x[0] else (
                   "C-only" if "e24-s20260912-c-" in x[0] else (
                   "BAC" if "e24-s20260912-bac-" in x[0] else (
                   "BCA" if "e24-s20260912-bca-" in x[0] else "ALT")))
            rows.setdefault(name, {"A": [], "C": []})
            rows[name]["A"].append(float(x[2])); rows[name]["C"].append(float(x[3]))
    names = ["A-only", "C-only", "BAC", "BCA", "ALT"]
    mA = [np.mean(rows[n]["A"]) for n in names]
    mC = [np.mean(rows[n]["C"]) for n in names]
    # verification against the committed audit table (x-synmem G3), tol 0.005:
    ref = {"A-only": (0.290, 0.026), "C-only": (0.031, 0.239),
           "BAC": (0.068, 0.137), "BCA": (0.234, 0.105)}
    for n, (ra, rc) in ref.items():
        assert abs(mA[names.index(n)] - ra) < 0.005 and abs(mC[names.index(n)] - rc) < 0.005, \
            f"instrument mismatch vs record: {n}"
    print("fig5 assertion: isolated/blocked bars reproduce the committed audit table")
    x = np.arange(len(names))
    fig, ax = plt.subplots(figsize=(7.2, 3.0))
    ax.bar(x - 0.2, mA, 0.4, color=C_A, label="A-channel mass")
    ax.bar(x + 0.2, mC, 0.4, color=C_C, label="C-channel mass")
    ax.set_xticks(x); ax.set_xticklabels(names)
    ax.set_ylabel("drive-end mean mass/neuron")
    ax.set_title("Regime trichotomy: winner / overwrite / superposition")
    ax.legend(fontsize=8)
    ax.axhline(0.14, color=C_G, ls=":", lw=0.8)
    fig.tight_layout()
    fig.savefig(os.path.join(OUT, "fig5-regimes.pdf"))
    fig.savefig(os.path.join(OUT, "fig5-regimes.png"), dpi=150)

# ---------------- fig6: protection ----------------
def fig6():
    fig, (a, b) = plt.subplots(1, 2, figsize=(8.4, 3.2))
    ts, ptot, pmax, w = [], [], [], []
    with open(os.path.join(DATA, "prot-s424242-il.tsv")) as f:
        for line in f:
            x = line.rstrip("\n").split("\t")
            if x[0] != "T": continue
            ts.append(int(x[1])); ptot.append(float(x[4])); w.append(float(x[5]))
            pmax.append(float(x[7]))
    a.plot(ts, pmax, color=C_R, lw=1.2, label="max P/neuron")
    a.plot(ts, ptot, color=C_B, lw=1.2, label="total P")
    a.axhline(0.6, color=C_G, ls="--", lw=0.8)
    a.text(2000, 0.62, "cap 0.60", fontsize=7)
    a.set_xlabel("t (ms)"); a.set_ylabel("protected mass")
    a.set_title("(a) P fills and pins at the cap")
    a.legend(fontsize=7)
    b.plot(ts, w, color=C_A, lw=1.2, label="working mass W")
    b.set_xlabel("t (ms)"); b.set_ylabel("total working mass")
    b.set_title("(b) working pool shrinks as P fills")
    b.set_ylim(0, 60)
    b.legend(fontsize=7)
    fig.tight_layout()
    fig.savefig(os.path.join(OUT, "fig6-protection.pdf"))
    fig.savefig(os.path.join(OUT, "fig6-protection.png"), dpi=150)

# ---------------- fig7: decomposition ----------------
def fig7():
    regs = [("baseline", "fe-s20260912-bac"), ("uncapped k$_g$=8", "rg8-s20260912-bac"),
            ("capped", "rg8c-s20260912-bac")]
    # aggregate over the 3 seeds per registration
    seedmap = {
        "fe":  ["fe-s20260912-bac", "fe-s9001-bac", "fe-s424242-bac"],
        "rg8": ["rg8-s20260912-bac", "rg8-s9001-bac", "rg8-s424242-bac"],
        "rg8c":["rg8c-s20260912-bac", "rg8c-s9001-bac", "rg8c-s424242-bac"],
    }
    def stats(sel):
        iwC, posts, perms = [], [], []
        for name in sel:
            r = [x for x in rows_p(name) if x["pat"] == "C" and x["ord"] == 21]
            iwC.append(r[0]["iwC"] / 52.0 if r else 0.0)
            posts.append(r[0]["uniq"] if r else 0.0)
            pr = [t for t, c in rows_x(name) if c == "C" and t >= 45001 and t < 85001]
            perms.append(len(pr))
        return iwC, posts, perms
    fig, axs = plt.subplots(1, 3, figsize=(9.6, 3.0))
    labels = ["baseline", "uncapped", "capped"]
    for ax, sel, lab, bar in zip(axs, ["fe", "rg8", "rg8c"], labels, [C_B, C_R, "#b8860b"]):
        iwC, posts, perms = stats(seedmap[sel])
        m = lambda v: (np.mean(v), np.min(v), np.max(v))
        mi, (lo, hi) = m(iwC)[0], (m(iwC)[1], m(iwC)[2])
        ax.bar(0, mi, 0.55, color=bar)
        ax.errorbar(0, mi, yerr=[[mi - lo], [hi - mi]], fmt="none", ecolor=C_G, capsize=3)
        ax.set_title(lab + "\\nfirst-C $I_W$ " + f"{mi:.2f}", fontsize=9)
        ax.set_ylim(0, 1.8); ax.set_xticks([])
    axs[0].set_ylabel("working C current/neuron at first exposure")
    for ax, sel, lab in zip(axs, ["fe", "rg8", "rg8c"], labels):
        iwC, posts, perms = stats(seedmap[sel])
        ax2 = ax.twinx()
        ax2.plot([0], [np.mean(posts)], "o", color=C_A, ms=6)
        ax2.set_yticks([])
        ax.text(0.28, 1.4, "posts/n " + f"{np.mean(posts):.2f}", fontsize=8, color=C_A)
    axs[2].text(0.28, 1.4, "perms " + f"{int(np.mean(perms))}", fontsize=8, color=C_G)
    fig.tight_layout()
    fig.savefig(os.path.join(OUT, "fig7-decomp.pdf"))
    fig.savefig(os.path.join(OUT, "fig7-decomp.png"), dpi=150)

for fn in [fig2, fig4, fig5, fig6, fig7]:
    fn()
    print("drew", fn.__name__)
print("figures ->", os.path.abspath(OUT))