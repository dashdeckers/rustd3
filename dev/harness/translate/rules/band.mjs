// d3-scale's scaleBand and scalePoint over string keys: scale::band and
// scale::point with their builders, `s(key)` as `apply(&key)` (None is d3's
// undefined) and `s.domain().map(s)`. A scale value is {t: "band", kind}.
import { calleeName, ex, needNum, skip, val } from "../expr.mjs";

const SETTERS = {
  range: "range", rangeRound: "range_round", round: "round", padding: "padding", paddingInner: "padding_inner",
  paddingOuter: "padding_outer", align: "align",
};
const METHODS = new Set(["domain", "copy", "bandwidth", "step", ...Object.keys(SETTERS)]);
const PUSH = "js: an array changed after a scale copied it (Rust copies arrays)";

/** A chain on `root`, as scale.mjs's: setters on a variable assign it, as d3's mutate it; `each(prop, args, raw)` gives {set}, {read, t} or {copy}. */
export function chain(root, steps, ctx, each) {
  const name = [...ctx.vars.values()].includes(root) ? root.c : null;
  let [code, muts] = [root.c, name ? "" : null];
  const current = () => (muts ? `({ ${name} = ${name}${muts}; ${name}.clone() })` : code);
  for (const { prop, args } of steps) {
    const out = each(prop, args.map((a) => val(ex(a, ctx))), args);
    if (out.read) return { c: muts ? `({ ${name} = ${name}${muts}; ${name}${out.read} })` : `${code}${out.read}`, t: out.t };
    if (out.copy) [code, muts] = [current() + (muts ? "" : ".clone()"), null];
    else if (muts !== null) muts += out.set;
    else code += out.set;
  }
  return { ...root, c: current(), ...(muts ? { stmt: `${name} = ${name}${muts};` } : {}) };
}

/** d3's constructor `make[name]()` with its initRange calls (one argument is the range), or `names` chained on a variable. */
export function calls(node, ctx, make, names) {
  const ctor = make[calleeName(node, ctx)];
  const [a, b] = node.arguments;
  if (ctor) return { value: ctor(), steps: b ? [{ prop: "domain", args: [a] }, { prop: "range", args: [b] }] : a ? [{ prop: "range", args: [a] }] : [] };
  const steps = [];
  let root = node;
  for (; root.type === "CallExpression" && names.has(root.callee.property?.name) && !root.callee.computed; root = root.callee.object) {
    steps.unshift({ prop: root.callee.property.name, args: root.arguments });
  }
  if (!steps.length || (root.type === "Identifier" && !ctx.vars.has(root.name))) return undefined;
  return { value: val(ex(root, ctx)), steps };
}

/** A list argument: one held in a variable cloned, as d3 copies it; `[]` of the scale's type. */
export const list = (e, raw) => (raw.type === "Identifier" ? `${e.c}.clone()` : e.t === "vec" && !e.items?.length ? "Vec::new()" : e.c);

const step = (kind) => (prop, [a], [raw]) => {
  if (!a && (prop === "domain" || prop === "range")) return { read: `.get_${prop}().to_vec()`, t: prop === "domain" ? "strs" : "vec" };
  if (prop === "bandwidth" || prop === "step") return { read: `.${prop}()`, t: "num" };
  if (prop === "copy") return { copy: true };
  if (!a) skip(`api: d3's ${prop}() getter (only domain and range have get_)`);
  if (prop === "domain") return a.t === "strs" || (a.t === "vec" && !a.items?.length) ? { set: `.domain(${list(a, raw)})` } : skip("types: band keys other than strings");
  if (prop === "round") return { set: `.round(${a.t === "bool" ? a.c : skip("js: a round flag other than a boolean (truthiness)")})` };
  if (kind === "point" && prop.startsWith("padding") && prop !== "padding") skip("types: d3's point scale has no inner or outer padding");
  if (!prop.startsWith("range")) return { set: `.${SETTERS[prop]}(${needNum(a, prop)})` };
  if (a.t === "strs") skip("js: strings as range values (coercion)");
  if (!a.items) skip("js: a range array held in a variable (Rust copies arrays)");
  return a.items.length === 2 ? { set: `.${SETTERS[prop]}([${a.items.join(", ")}])` } : skip("types: a band range of other than two numbers");
};

const make = (kind) => () => ({ c: `scale::${kind}(Vec::<&str>::new())`, t: "band", kind });
const MAKE = { scaleBand: make("band"), scalePoint: make("point") };

export default {
  call(node, ctx) {
    const found = calls(node, ctx, MAKE, METHODS);
    return found?.value.t === "band" ? chain(found.value, found.steps, ctx, step(found.value.kind)) : undefined;
  },
  apply(target, args) {
    if (target.t !== "band") return undefined;
    if (args.length !== 1 || args[0].t !== "str") skip("types: a band key other than a string");
    return { c: `${target.c}.apply(&${args[0].c})`, t: "maybe" };
  },
  // s.domain().map(s): each key's band.
  method(obj, prop, [f]) {
    if (obj.t === "strs" && prop === "push") skip(PUSH);
    if (prop !== "map" || f?.t !== "band") return undefined;
    return { c: `${obj.c}.iter().map(|k| ${f.c}.apply(k)).collect::<Vec<_>>()`, t: "list" };
  },
  field(obj, prop) {
    if (obj.t === "band" && obj.kind === "point") skip(`types: d3's point scale has no ${prop}`);
    return undefined;
  },
};
