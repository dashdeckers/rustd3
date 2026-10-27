"""matplotlib 3.11.2's tick locators for the symlog, logit and asinh scales.

Writes scales.json next to this file: for each case the domain, the
locator's parameters and the ticks it places inside the domain (what an
axis shows), called without an axis so that the values do not depend on
a figure's size. rustd3-core's tests/main/fixtures.rs compares them with
its own rules.

    python3 dev/fixtures/scales.py

numpy runs at its baseline instruction set: its AVX-512 sinh, arcsinh and
log10 differ from the C library's by an ulp, which moves AsinhLocator's
rounding at exact powers of ten (arcsinh(1e9)), so the dispatched code
would make the values depend on the machine.
"""

import json
import os
import pathlib

os.environ["NPY_ENABLE_CPU_FEATURES"] = "X86_V2"

import matplotlib  # noqa: E402
import matplotlib.ticker as ticker  # noqa: E402

assert matplotlib.__version__ == "3.11.2", matplotlib.__version__

COUNT = 10

# (domain, constant, reason): reason is None when the case is compared.
# "symlog-thinned": matplotlib emits more than 2 × count ticks where rustd3
# thins to every k-th decade. "symlog-linear": rustd3 ticks linearly, below
# 100 constants or because fewer than two decades fall inside the domain.
SYMLOG = [
    ([0, 1e6], 1, None),
    ([-1000, 1000], 1, None),
    ([0, 100], 1, None),
    ([1, 1e4], 1, None),
    ([-1e4, 10], 1, None),
    ([-1e5, -10], 1, None),
    ([-10, 10], 0.01, None),
    ([0, 1e5], 10, None),
    ([-1e9, 1e9], 1, "symlog-thinned"),
    ([-3, 3], 1, "symlog-linear"),
    ([-5, 50], 1, "symlog-linear"),
    ([1000, 2000], 1, "symlog-linear"),
]

LOGIT = [
    [0.001, 0.999],
    [1e-6, 1 - 1e-6],
    [1e-9, 0.5],
    [0.01, 0.99],
    [0.2, 0.7],
    [0.002, 0.998],
    [0.1, 0.9],
    [0.3, 0.9999],
    [1e-4, 0.3],
]

# (domain, linear width)
ASINH = [
    ([-1000, 1000], 1),
    ([0, 1e6], 1),
    ([-3, 3], 1),
    ([0, 100], 1),
    ([-1e9, 1e9], 1),
    ([-50, 5], 1),
    ([1, 1e4], 1),
    ([-50, 45], 1),
    ([-1000, 1000], 10),
    ([-2e5, 3e3], 1),
    ([0.5, 2], 1),
]


def rounded(v):
    """A float rounded to 12 significant digits, so reruns print the same."""
    return float(f"{float(v):.12g}")


def inside(values, domain):
    a, b = min(domain), max(domain)
    return [rounded(v) for v in values if a <= v <= b]


def symlog(domain, constant):
    locator = ticker.SymmetricalLogLocator(linthresh=constant, base=10)
    return inside(locator.tick_values(*domain), domain)


def logit(domain):
    locator = ticker.LogitLocator(nbins=COUNT)
    return inside(locator.tick_values(*domain), domain)


def asinh(domain, width):
    # AsinhLocator.__call__'s symthresh rule, which tick_values leaves out.
    locator = ticker.AsinhLocator(width, numticks=COUNT + 1, base=10)
    vmin, vmax = min(domain), max(domain)
    if vmin * vmax < 0 and abs(1 + vmax / vmin) < locator.symthresh:
        bound = max(abs(vmin), abs(vmax))
        values = locator.tick_values(-bound, bound)
    else:
        values = locator.tick_values(vmin, vmax)
    return inside(values, domain)


def case(domain, ticks, **params):
    return {"count": COUNT, "domain": [rounded(v) for v in domain], "ticks": ticks, **params}


def main():
    symlog_cases = []
    for domain, constant, reason in SYMLOG:
        ticks = symlog(domain, constant)
        if reason == "symlog-thinned":
            assert len(ticks) > 2 * COUNT, (domain, ticks)
        extra = {"compared": reason is None, **({"reason": reason} if reason else {})}
        symlog_cases.append(case(domain, ticks, constant=rounded(constant), **extra))
    fixture = {
        "matplotlib": matplotlib.__version__,
        "symlog": symlog_cases,
        "logit": [case(d, logit(d), compared=True) for d in LOGIT],
        "asinh": [case(d, asinh(d, w), compared=True, linear_width=rounded(w)) for d, w in ASINH],
    }
    path = pathlib.Path(__file__).with_name("scales.json")
    path.write_text(json.dumps(fixture, indent=1, sort_keys=True) + "\n")


if __name__ == "__main__":
    main()
