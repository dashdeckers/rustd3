// d3-scale's quantize, quantile and threshold scales: scale::quantize(domain,
// range), scale::quantile and scale::threshold with d3's defaults. Setters
// before the scale is first read go into the constructor, so a range of
// strings types the scale ({t: "binned", kind, r: "num" | "str"}); setters
// on a variable assign it, as d3's mutate it. Applying a scale is
// `apply(x).copied()`, None where d3 returns its unknown (undefined);
// invertExtent is an Option of the pair, threshold's `[None, None]` where
// d3 returns [undefined, undefined], and `.every(isNaN)` of a quantize or
// quantile one is its None.
import { calleeName, ex, needNum, skip, val } from "../expr.mjs";
import { data } from "./stats.mjs";

const ROOTS = {
  scaleQuantize: ["quantize", "[0.0, 1.0]", "vec![0.0, 1.0]"],
  scaleQuantile: ["quantile", "Vec::<f64>::new()", "Vec::<f64>::new()"],
  scaleThreshold: ["threshold", "vec![0.5]", "vec![0.0, 1.0]"],
};
const METHODS = new Set(["domain", "range", "unknown", "thresholds", "quantiles", "invertExtent"]);

/** A range argument: strings, or numbers. */
function range(node, ctx) {
  const e = node.type === "ArrayExpression" && val(ex(node, ctx));
  return e?.t === "strs" ? { c: e.c, r: "str" } : { c: data(node, ctx), r: "num" };
}

/** A domain argument: two numbers for quantize, a Vec for the others. */
function domain(kind, node, ctx) {
  if (kind !== "quantize") return { c: data(node, ctx) };
  if (node.type !== "ArrayExpression" || node.elements.length !== 2) skip("types: a quantize domain of other than two numbers ([f64; 2])");
  return { c: `[${node.elements.map((e) => needNum(val(ex(e, ctx)), "a domain value")).join(", ")}]` };
}

/** A scale chain: its root (a constructor or a variable), then each method call. */
function chain(node, ctx) {
  const steps = [];
  let root = node;
  while (root.type === "CallExpression" && root.callee.type === "MemberExpression" && !root.callee.computed && METHODS.has(root.callee.property.name)) {
    steps.unshift({ prop: root.callee.property.name, args: root.arguments });
    root = root.callee.object;
  }
  const made = root.type === "CallExpression" && !root.arguments.length && ROOTS[calleeName(root, ctx)];
  const v = root.type === "Identifier" && ctx.vars.get(root.name)?.t === "binned" && val(ex(root, ctx));
  if (!made && !v) return undefined;
  const s = v ? { ...v, name: v.c, muts: "" } : { kind: made[0], r: "num", name: null };
  if (made) {
    let [, d, r] = made;
    while (steps[0]?.args.length && ["domain", "range"].includes(steps[0].prop)) {
      const { prop, args } = steps.shift();
      if (prop === "domain") d = domain(s.kind, args[0], ctx).c;
      else ({ c: r, r: s.r } = range(args[0], ctx));
    }
    s.c = `scale::${s.kind}(${d}, ${r})`;
  }
  const read = (suffix, t) => ({ c: s.muts ? `({ ${s.name} = ${s.name}${s.muts}; ${s.name}${suffix} })` : `${s.c}${suffix}`, t });
  for (const { prop, args } of steps) {
    if (prop === "unknown") skip("api: unknown values (apply returns None)");
    if (args.length && (prop === "domain" || prop === "range")) {
      const set = prop === "domain" ? domain(s.kind, args[0], ctx) : range(args[0], ctx);
      if (prop === "range" && set.r !== s.r) skip("types: a range of another type (the range type is the scale's)");
      if (s.name) s.muts += `.${prop}(${set.c})`;
      else s.c += `.${prop}(${set.c})`;
      continue;
    }
    if (prop === "domain" || prop === "range") return read(`.get_${prop}().to_vec()`, prop === "range" && s.r === "str" ? "strs" : "vec");
    if (prop !== "invertExtent") return read(`.${prop}().to_vec()`, "vec");
    const y = val(ex(args[0], ctx));
    if (y.t !== s.r) skip("types: a value of another type than the range's");
    return s.kind === "threshold" ? read(`.invert_extent(&${y.c}).unwrap_or([None, None])`, "maybes") : read(`.invert_extent(&${y.c})`, "extent");
  }
  const { kind, r } = s;
  if (!s.muts) return { c: s.c, t: "binned", kind, r };
  return { c: read(".clone()").c, t: "binned", kind, r, stmt: `${s.name} = ${s.name}${s.muts};` };
}

export default {
  call(node, ctx) {
    const isNaN = (a) => a?.name === "isNaN" || (a?.object?.name === "Number" && a.property?.name === "isNaN");
    if (node.callee.property?.name === "every" && isNaN(node.arguments[0])) {
      const e = val(ex(node.callee.object, ctx));
      if (e.t === "extent") return { c: `${e.c}.is_none()`, t: "bool" };
    }
    return chain(node, ctx);
  },
  apply(target, args) {
    if (target.t !== "binned") return undefined;
    if (args.length !== 1) skip(`js: ${args.length} arguments to a scale (undefined)`);
    const x = args[0].t === "undef" ? "f64::NAN" : needNum(args[0], "scale argument");
    return { c: `${target.c}.apply(${x}).copied()`, t: target.r === "str" ? "maybestr" : "maybe" };
  },
  method(obj, prop, args) {
    if (obj.t !== "vec" || prop !== "map" || args[0]?.t !== "binned") return undefined;
    return { c: `${obj.c}.iter().map(|&v| ${args[0].c}.apply(v).copied()).collect::<Vec<_>>()`, t: "list" };
  },
};
