"""matplotlib 3.11.2's autoscaled axis limits, for rustd3's `scale::Pad`.

Writes pad.json next to this file: for each case the data extent, the
margin, the sticky values, the axis scale, whether the axis is inverted,
and the x limits matplotlib's `autoscale_view` sets for a line through the
extent. rustd3-core's tests/main/pad.rs fits the same cases with
`scale::Pad` and compares within 1e-12 relative.

    python3 dev/fixtures/pad.py

The cases are linear extents (flat, tiny and huge among them), sticky
edges, non-flat log extents and inverted axes: `Pad` widens a flat or
empty extent in transformed space, which differs from matplotlib's
`LogLocator.nonsingular` on a flat log extent. numpy runs at its baseline
instruction set, as in scales.py, so log10 rounds as the C library does on
any x86-64 machine.
"""

import json
import os
import pathlib

os.environ["NPY_ENABLE_CPU_FEATURES"] = "X86_V2"

import matplotlib  # noqa: E402

matplotlib.use("Agg")

import matplotlib.pyplot as plt  # noqa: E402

assert matplotlib.__version__ == "3.11.2", matplotlib.__version__

# (scale, extent, margin, sticky, inverted)
CASES = [
    ("linear", [0, 10], 0.05, [], False),
    ("linear", [3, 3], 0.05, [], False),
    ("linear", [0, 0], 0.05, [], False),
    ("linear", [-2, -2], 0.05, [], False),
    ("linear", [-7.5, 1e6], 0.1, [], False),
    ("linear", [1e-20, 2e-20], 0.05, [], False),
    ("linear", [0.1, 0.3], 0, [], False),
    ("linear", [-3, -1], 0.2, [], False),
    ("linear", [1e300, 1e300], 0.05, [], False),
    ("linear", [5, 5.000000000000001], 0.05, [], False),
    ("linear", [0, 4], 0.05, [0], False),
    ("linear", [-2, 4], 0.05, [0], False),
    ("linear", [0, 4], 0.05, [0, 4], False),
    ("linear", [1, 5], 0.1, [0, 5.00001], False),
    ("linear", [0, 1], 0.05, [2, 1, 0.5, 0, -1], False),
    ("linear", [2, 2], 0.05, [2], False),
    ("linear", [-4, 0], 0.05, [0], False),
    ("log", [1, 1000], 0.05, [], False),
    ("log", [0.01, 50], 0.1, [], False),
    ("log", [3, 7], 0.05, [], False),
    ("log", [1e-5, 1e5], 0.05, [], False),
    ("log", [1, 100], 0.05, [1], False),
    ("linear", [0, 10], 0.05, [], True),
    ("linear", [0, 4], 0.05, [0], True),
    ("linear", [3, 3], 0.05, [], True),
    ("log", [1, 1000], 0.05, [], True),
]


def limits(scale, extent, margin, sticky, inverted):
    """The x limits matplotlib autoscales to for a line through `extent`."""
    fig, ax = plt.subplots()
    (line,) = ax.plot(extent, [0, 0])
    line.sticky_edges.x[:] = sticky
    ax.set_xmargin(margin)
    ax.set_xscale(scale)
    if inverted:
        ax.invert_xaxis()
    lo, hi = ax.get_xlim()
    plt.close(fig)
    return [float(lo), float(hi)]


def main():
    cases = [
        {
            "extent": [float(v) for v in extent],
            "inverted": inverted,
            "limits": limits(scale, extent, margin, sticky, inverted),
            "margin": float(margin),
            "scale": scale,
            "sticky": [float(v) for v in sticky],
        }
        for scale, extent, margin, sticky, inverted in CASES
    ]
    fixture = {"matplotlib": matplotlib.__version__, "cases": cases}
    path = pathlib.Path(__file__).with_name("pad.json")
    path.write_text(json.dumps(fixture, indent=1, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
