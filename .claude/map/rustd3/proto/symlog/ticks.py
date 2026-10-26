# Review F: symlog tick rules compared by where
# the ticks land on the axis.
import math, matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

def d3_ticks(a, b, n=10):
    # d3-array tickIncrement/ticks, enough for these ranges
    step = (b - a) / n
    p = 10 ** math.floor(math.log10(step))
    e = step / p
    m = 10 if e >= 7.07 else 5 if e >= 3.16 else 2 if e >= 1.41 else 1
    s = m * p
    i0, i1 = math.ceil(a / s), math.floor(b / s)
    return [round(i * s, 12) for i in range(i0, i1 + 1)]

def sl(x, c=1.0):
    return math.copysign(math.log1p(abs(x) / c), x)

def proposed(a, b, c=1.0, n=10):
    span = max(abs(a), abs(b)) / c
    if span < 100:                 # under two decades: nearly linear
        return d3_ticks(a, b, n)
    t = [0.0] if a <= 0 <= b else []
    k = 0
    while c * 10 ** k <= max(abs(a), abs(b)):
        for v in (c * 10 ** k, -c * 10 ** k):
            if a <= v <= b:
                t.append(v)
        k += 1
    t.sort()
    step = 1
    while len(t) // step > n:      # thin to about n
        step += 1
    keep = [v for v in t if v == 0 or round(math.log10(abs(v) / c)) % step == 0]
    return keep

def mpl(a, b, c=1.0):
    f, ax = plt.subplots()
    ax.set_yscale("symlog", linthresh=c)
    ax.set_ylim(a, b)
    t = [float(v) for v in ax.get_yticks() if a <= v <= b]
    plt.close(f)
    return t

def gap(t, a, b):
    # largest share of the axis without a tick
    p = sorted([sl(a)] + [sl(v) for v in t] + [sl(b)])
    return max(q - r for r, q in zip(p, p[1:])) / (sl(b) - sl(a))

for a, b in [(-1000, 1000), (0, 1e6), (-5, 50), (-3, 3), (-1e9, 1e9)]:
    print(f"[{a:g}, {b:g}]")
    for name, f in [("d3", d3_ticks), ("matplotlib", mpl), ("proposed", proposed)]:
        t = f(a, b)
        s = ", ".join(f"{v:g}" for v in t)
        print(f"  {name:10} {len(t):2} ticks, largest empty {100*gap(t,a,b):3.0f}%: {s}")
