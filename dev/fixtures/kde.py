"""scipy 1.18.1's gaussian_kde and seaborn 0.13.2's KDE, for rustd3's
`stats::Kde`.

Writes kde.json next to this file, for Phase 34's spot-check data X (n =
15): Scott's bandwidth as scipy computes it (the sample standard
deviation, the factor n^(-1/5) and h = sqrt of the kernel covariance),
scipy's density at seven points, and seaborn's support grid (gridsize 200,
cut 3, `bw_method="scott"`) with the density on it. rustd3-figure's
tests/main/marks/density.rs compares through rustd3-testkit's `Obs`.

    python3 dev/fixtures/kde.py

numpy runs at its baseline instruction set, as in scales.py, so its
exponentials and sums round as on any x86-64 machine.
"""

import json
import os
import pathlib

os.environ["NPY_ENABLE_CPU_FEATURES"] = "X86_V2"

import matplotlib  # noqa: E402

matplotlib.use("Agg")

import numpy as np  # noqa: E402
import scipy  # noqa: E402
import seaborn  # noqa: E402
from scipy.stats import gaussian_kde  # noqa: E402
from seaborn._statistics import KDE  # noqa: E402

assert matplotlib.__version__ == "3.11.2", matplotlib.__version__
assert scipy.__version__ == "1.18.1", scipy.__version__
assert seaborn.__version__ == "0.13.2", seaborn.__version__

X = [1.2, 2.3, 2.9, 3.1, 3.4, 3.8, 4.0, 4.4, 4.7, 5.1, 5.6, 6.2, 7.9, 9.5, 14.0]
POINTS = [0.0, 2.0, 4.0, 4.4, 6.0, 10.0, 14.0]

kde = gaussian_kde(X, bw_method="scott")
estimator = KDE(bw_method="scott", gridsize=200, cut=3)
density, support = estimator(np.array(X))
peak = int(np.argmax(density))

fixture = {
    "values": X,
    "deviation": float(np.std(X, ddof=1)),
    "factor": float(kde.factor),
    "bandwidth": float(np.sqrt(kde.covariance[0, 0])),
    "points": POINTS,
    "density": [float(v) for v in kde(POINTS)],
    "grid": {
        "points": 200,
        "cut": 3,
        "support": [float(v) for v in support],
        "density": [float(v) for v in density],
        "peak": [float(support[peak]), float(density[peak])],
    },
}

path = pathlib.Path(__file__).with_name("kde.json")
path.write_text(json.dumps(fixture, indent=1, sort_keys=True) + "\n")
