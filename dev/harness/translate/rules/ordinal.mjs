// d3-scale's scaleOrdinal over number or string keys and values: `s(x)` is
// `apply_or_insert(x)`, d3's implicit growth, until `.unknown(…)` is set,
// then `apply(&x)`; `.unknown(undefined)` sets none and leaves `apply`.
// Key and value types are named once the test is read (finish): a
// scale is {t: "ordinal", types: {id, k, v}, implicit}.
import { skip } from "../expr.mjs";
import { calls, chain, list } from "./band.mjs";

const RUST = { num: "f64", str: "&str" };
const METHODS = new Set(["domain", "range", "unknown", "copy"]);
const scales = [];

/** Notes that `slot` (k or v) of `types` holds `e`, a list of them when `of`; two types on one scale are a skip. */
function holds(types, slot, e, of) {
  if ((of && e.t === "vec" && !e.items?.length && e.c.startsWith("Vec::")) || e.t === "undef") return;
  const t = (of ? { vec: "num", strs: "str" } : { num: "num", str: "str" })[e.t] ?? skip(`js: a ${e.t} as ordinal ${slot === "k" ? "keys" : "values"} (coercion)`);
  if (types[slot] && types[slot] !== t) skip(`types: ${slot === "k" ? "keys" : "values"} of two types on one scale`);
  types[slot] = t;
}

const step = (value) => (prop, [a], [raw]) => {
  if (prop === "copy") return { copy: true };
  const slot = prop === "domain" ? "k" : "v";
  if (!a && prop !== "unknown") return { read: `.get_${prop}().to_vec()`, t: value.types[slot] === "str" ? "strs" : "vec" };
  if (!a) skip("api: d3's unknown() getter (only domain and range have get_)");
  if (raw.type === "NewExpression") skip("js: an iterable as a domain or range");
  holds(value.types, slot, a, prop !== "unknown");
  if (prop !== "unknown") return { set: `.${prop}(${list(a, raw)})` };
  value.implicit = false;
  return { set: a.t === "undef" ? "" : `.unknown(${a.c})` };
};

function make() {
  const types = { id: scales.length, k: null, v: null };
  scales.push(types);
  return { c: `scale::ordinal(Vec::<__K${types.id}__>::new(), Vec::<__V${types.id}__>::new())`, t: "ordinal", types, implicit: true };
}

export default {
  call(node, ctx) {
    const found = calls(node, ctx, { scaleOrdinal: make }, METHODS);
    return found?.value.t === "ordinal" ? chain(found.value, found.steps, ctx, step(found.value)) : undefined;
  },
  apply(target, [key, ...rest]) {
    if (target.t !== "ordinal") return undefined;
    if (!key || rest.length) skip("js: other than one argument to an ordinal scale (undefined)");
    holds(target.types, "k", key, false);
    return { c: target.implicit ? `${target.c}.apply_or_insert(${key.c})` : `${target.c}.apply(&${key.c})`, t: "ordval" };
  },
  finish: (line) => line.replace(/__([KV])(\d+)__/g, (_, slot, id) => RUST[scales[id][slot.toLowerCase()]] ?? "&str"),
};
