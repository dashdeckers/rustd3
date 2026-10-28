"""matplotlib 3.11.2's statistics behind rustd3-figure's `helpers`, with
numpy and scipy as matplotlib calls them.

Writes stats.json next to this file, for Phase 38's spot-check data X (n =
15) and two small datasets: `cbook.boxplot_stats` with `whis=1.5` and
`whis=(5, 95)` (and over a single value), numpy's default "linear"
quantiles, `Axes.hist` with 10 and 5 equal bins, density, cumulative
counts and weights, with numpy's "auto" bins (and their fallback to
Sturges when the IQR is 0), over d3.bin's nice edges, and stacked over
shared edges (with density); `cbook.violin_stats` with matplotlib's
Gaussian KDE at 100 points; `Axes.ecdf`, plain and complementary.
rustd3-figure's tests/main/helpers compare through rustd3-testkit's `Obs`.

    python3 dev/fixtures/stats.py           # write stats.json
    python3 dev/fixtures/stats.py --check   # compare with it at 1e-9

`--check` recomputes every value and compares it with the committed JSON
within a relative 1e-9, exiting 1 and naming the differing keys
otherwise. numpy runs at its baseline instruction set, as in kde.py.
"""

import json
import math
import os
import pathlib
import sys

os.environ["NPY_ENABLE_CPU_FEATURES"] = "X86_V2"

import matplotlib  # noqa: E402

matplotlib.use("Agg")

import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402
import scipy  # noqa: E402
from matplotlib import cbook, mlab  # noqa: E402

assert matplotlib.__version__ == "3.11.2", matplotlib.__version__
assert scipy.__version__ == "1.18.1", scipy.__version__

X = [1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1, 5.6, 6.2, 7.9, 9.5, 14.0]
# An IQR of 0: numpy's "auto" falls back to Sturges.
FLAT = [1.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 3.0, 9.0]
A = [1.0, 2.0, 2.0, 3.0]
B = [2.0, 3.0, 3.0]
STACKED_EDGES = [0.5, 1.5, 2.5, 3.5]
WEIGHTS = [float(w) for w in range(1, 16)]
TOLERANCE = 1e-9


def floats(values):
    """`values` as a list of Python floats."""
    return [float(v) for v in values]


def boxplot(values, whis):
    """`cbook.boxplot_stats` under rustd3's field names."""
    (s,) = cbook.boxplot_stats(values, whis=whis)
    return {
        "q1": float(s["q1"]),
        "median": float(s["med"]),
        "q3": float(s["q3"]),
        "iqr": float(s["iqr"]),
        "mean": float(s["mean"]),
        "whisker_low": float(s["whislo"]),
        "whisker_high": float(s["whishi"]),
        "notch_low": float(s["cilo"]),
        "notch_high": float(s["cihi"]),
        "fliers": floats(s["fliers"]),
    }


def hist(values, bins, **options):
    """`Axes.hist`'s edges and bar heights."""
    fig, ax = plt.subplots()
    counts, edges, _ = ax.hist(values, bins=bins, **options)
    plt.close(fig)
    return {"edges": floats(edges), "counts": floats(counts)}


def stacked(density):
    """`Axes.hist` of A and B stacked over STACKED_EDGES: each group's
    spans [bottom, top] per bin, A at the bottom."""
    fig, ax = plt.subplots()
    tops, edges, _ = ax.hist([A, B], bins=STACKED_EDGES, stacked=True, density=density)
    plt.close(fig)
    bottoms = [np.zeros(len(edges) - 1), tops[0]]
    return {
        "edges": floats(edges),
        "a": [[float(b), float(t)] for b, t in zip(bottoms[0], tops[0])],
        "b": [[float(b), float(t)] for b, t in zip(bottoms[1], tops[1])],
    }


def violin(values, points):
    """`cbook.violin_stats` with the KDE `Axes.violinplot` uses."""

    def kde(data, coords):
        return mlab.GaussianKDE(data, "scott").evaluate(coords)

    (s,) = cbook.violin_stats([values], kde, points=points)
    peak = int(np.argmax(s["vals"]))
    return {
        "points": points,
        "coords": floats(s["coords"]),
        "vals": floats(s["vals"]),
        "mean": float(s["mean"]),
        "median": float(s["median"]),
        "min": float(s["min"]),
        "max": float(s["max"]),
        "peak": [float(s["coords"][peak]), float(s["vals"][peak])],
        "bandwidth": float(np.sqrt(mlab.GaussianKDE(values, "scott").covariance[0, 0])),
    }


def ecdf(values, complementary):
    """`Axes.ecdf`'s step vertices."""
    fig, ax = plt.subplots()
    line = ax.ecdf(values, complementary=complementary)
    xs, ys = floats(line.get_xdata()), floats(line.get_ydata())
    drawstyle = line.get_drawstyle()
    plt.close(fig)
    return {"x": xs, "y": ys, "drawstyle": drawstyle}


def compute():
    """Every value of the fixture."""
    return {
        "values": X,
        "quantiles": {
            "p": [0.25, 0.5, 0.75],
            "values": floats(np.quantile(X, [0.25, 0.5, 0.75])),
        },
        "boxplot": {
            "iqr": boxplot(X, 1.5),
            "percentiles": boxplot(X, (5, 95)),
            "single": boxplot([3.0], 1.5),
        },
        "hist": {
            "count10": hist(X, 10),
            "count10_density": hist(X, 10, density=True),
            "auto": hist(X, "auto"),
            "auto_flat": {"values": FLAT, **hist(FLAT, "auto")},
            "count5": hist(X, 5),
            "count5_cumulative": hist(X, 5, cumulative=True),
            "count5_weights": {"weights": WEIGHTS, **hist(X, 5, weights=WEIGHTS)},
            "count5_cumulative_density": hist(X, 5, cumulative=True, density=True),
            "nice": hist(X, list(range(0, 17, 2))),
        },
        "stacked": {
            "groups": {"a": A, "b": B},
            "counts": stacked(False),
            "density": stacked(True),
        },
        "violin": violin(X, 100),
        "ecdf": {
            "plain": ecdf(X, False),
            "complementary": ecdf(X, True),
        },
    }


def differences(expected, actual, at=""):
    """The keys under which `actual` differs from `expected` beyond
    TOLERANCE (relative), or in shape."""
    if isinstance(expected, dict) and isinstance(actual, dict):
        out = []
        for key in sorted(set(expected) | set(actual)):
            if key not in expected or key not in actual:
                out.append(f"{at}/{key}")
            else:
                out += differences(expected[key], actual[key], f"{at}/{key}")
        return out
    if isinstance(expected, list) and isinstance(actual, list):
        if len(expected) != len(actual):
            return [at]
        return [d for i, (e, a) in enumerate(zip(expected, actual)) for d in differences(e, a, f"{at}/{i}")]
    numbers = (int, float)
    if isinstance(expected, numbers) and isinstance(actual, numbers):
        e, a = float(expected), float(actual)
        if e == a or (math.isnan(e) and math.isnan(a)):
            return []
        return [] if abs(e - a) <= TOLERANCE * max(abs(e), abs(a)) else [at]
    return [] if expected == actual else [at]


def main():
    path = pathlib.Path(__file__).with_name("stats.json")
    fixture = compute()
    if sys.argv[1:] == ["--check"]:
        differing = differences(json.loads(path.read_text()), fixture)
        for key in differing:
            print(f"differs: {key}")
        sys.exit(1 if differing else 0)
    path.write_text(json.dumps(fixture, indent=1, sort_keys=True) + "\n")


main()
