"""The wheel figure's data and fitted domains, for rustd3-examples' wheels.

Replays `simulate()` of `dev/examples/src/wheels.rs` (seed 11, its LCG
and Box-Muller) and `trend`,
and writes wheels.json next to this file: the sample counts, the mark,
markers and bar counts of the figure, per fitted handle the extent of
what its marks report and its domain padded by 5% per side (dots, lines
and densities not held; the histograms' counts held at 0, where the
rule at 20 also reports), each domain checked against matplotlib's own
autoscaling of the same artists. rustd3-examples' tests/main/wheels.rs
compares through rustd3-testkit's `Obs`.

    python3 dev/fixtures/wheels.py
"""

import json
import math
import pathlib

import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt  # noqa: E402

assert matplotlib.__version__ == "3.11.2", matplotlib.__version__

WHEELS = ["FL", "FR", "RL", "RR"]
SOURCES = ["accel x", "accel z", "bearing temp"]
DRIVES = 600
MIN_COUNT = 20.0
MARGIN = 0.05


def simulate():
    """The samples as (wheel, source, bin, value), in the Rust order."""
    seed = 11

    def u():
        nonlocal seed
        seed = (seed * 1_664_525 + 1_013_904_223) & 0xFFFFFFFF
        return (seed >> 8) / (1 << 24)

    def gauss():
        a, b = max(u(), 1e-12), u()
        return math.sqrt(-2.0 * math.log(a)) * math.cos(math.tau * b)

    out = []
    for wheel in WHEELS:
        worn = wheel == "RL"
        for i in range(DRIVES):
            m, sd = (50.0, 15.0) if i % 2 == 0 else (120.0, 25.0)
            v = min(max(m + sd * gauss(), 0.0), 199.9)
            b = int(v / 10.0)
            fast = max(v - 100.0, 0.0)
            x = 0.2 + 0.006 * v + 0.08 * gauss()
            z = 0.3 + 0.009 * v + 0.1 * gauss() + (0.012 * fast if worn else 0.0)
            out.append((wheel, "accel x", b, x))
            out.append((wheel, "accel z", b, z))
            if i % 3 == 0:
                t = 35.0 + 0.12 * v + 2.0 * gauss() + (0.15 * fast if worn else 0.0)
                out.append((wheel, "bearing temp", b, t))
    return out


def trend(pts):
    """Least squares of value on bin index, at the first and last bin."""
    n = len(pts)
    mx = sum(b for b, _ in pts) / n
    my = sum(v for _, v in pts) / n
    sxy = sxx = 0.0
    for b, v in pts:
        sxy += (b - mx) * (v - my)
        sxx += (b - mx) ** 2
    slope = sxy / sxx
    bins = [b for b, _ in pts]
    return [(b, my + slope * (b - mx)) for b in (min(bins), max(bins))]


def padded(lo, hi, sticky=()):
    """[lo, hi] widened by MARGIN of its span per side, stopped at sticky values."""
    span = hi - lo
    a, b = lo - MARGIN * span, hi + MARGIN * span
    for s in sticky:
        if a < s <= lo:
            a = s
        if hi <= s < b:
            b = s
    return [a, b]


def autoscaled(draw):
    """matplotlib's y limits after `draw(ax)`, at its default 5% margins."""
    fig, ax = plt.subplots()
    draw(ax)
    ax.autoscale_view()
    lo, hi = ax.get_ylim()
    plt.close(fig)
    return [lo, hi]


samples = simulate()
of = {(w, s): [(b, v) for (w2, s2, b, v) in samples if (w2, s2) == (w, s)] for w in WHEELS for s in SOURCES}
trends = {key: trend(pts) for key, pts in of.items()}
counts = {}
for w, s, b, _ in samples:
    counts[(w, s, b)] = counts.get((w, s, b), 0) + 1

handle = {"accel x": "vib", "accel z": "vib", "bearing temp": "temp"}
reported = {"vib": [], "temp": []}
for (w, s), pts in of.items():
    reported[handle[s]] += [v for _, v in pts] + [v for _, v in trends[(w, s)]]
data = {h: [v for (w, s), pts in of.items() if handle[s] == h for _, v in pts] for h in reported}
bars = list(counts.values())

extents = {h: [min(vs), max(vs)] for h, vs in reported.items()}
extents["count"] = [min(bars + [0.0, MIN_COUNT]), max(bars + [0.0, MIN_COUNT])]
domains = {h: padded(*extents[h]) for h in ("vib", "temp")}
domains["count"] = padded(*extents["count"], sticky=[0.0])

# matplotlib: dots and lines are not held, bars hold their base.
for h in ("vib", "temp"):
    ys = reported[h]
    mpl = autoscaled(lambda ax: ax.plot(range(len(ys)), ys, "o"))
    assert all(abs(a - b) <= 1e-12 * max(1.0, abs(b)) for a, b in zip(mpl, domains[h])), (h, mpl)
mpl = autoscaled(lambda ax: (ax.bar(range(len(bars)), bars), ax.axhline(MIN_COUNT)))
assert all(abs(a - b) <= 1e-12 * max(1.0, abs(b)) for a, b in zip(mpl, domains["count"])), mpl

fixture = {
    "samples": len(samples),
    "per_wheel": {s: sum(1 for (w, s2, _, _) in samples if w == "FL" and s2 == s) for s in SOURCES},
    # Per wheel: dots, a trend line, a density and a histogram per source,
    # and the minimum-count rule.
    "marks": len(WHEELS) * (4 * len(SOURCES) + 1),
    "markers": len(WHEELS) * len(SOURCES),
    "bars": len(bars),
    "bars_per_wheel": {w: sum(1 for (w2, _, _) in counts if w2 == w) for w in WHEELS},
    "max_count": max(bars),
    "data": {h: [min(vs), max(vs)] for h, vs in data.items()},
    "extents": extents,
    "domains": domains,
    "trends": {f"{w} {s}": [[b, v] for b, v in trends[(w, s)]] for w in WHEELS for s in SOURCES},
}

path = pathlib.Path(__file__).with_name("wheels.json")
path.write_text(json.dumps(fixture, indent=1, sort_keys=True) + "\n")
