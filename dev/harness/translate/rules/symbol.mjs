// d3-shape's symbol(): Symbol::{type}.draw(size, …) into a fresh SvgPath, or after context(p) into the
// prelude's Polygon (d3's polygonContext). A size function returning its datum takes the call's argument.
import { ex, needNum, skip, val } from "../expr.mjs";
import { generators } from "./generator.mjs";

const FILL = ["Circle", "Cross", "Diamond", "Square", "Star", "Triangle", "Wye"];
const API = "api: symbol types and sizes as accessors, and getters (Symbol::draw takes a size)";
const num = (node, ctx) => needNum(val(ex(node, ctx)), "a size");
/** A size: a number, or "datum" for `function(d) { return d; }`. */
const size = (n, ctx) => (!/Function/.test(n.type) ? num(n, ctx) : n.params.length === 1 && (n.body.body?.[0]?.argument ?? n.body).name === n.params[0].name ? "datum" : skip(API));

function kind(node) {
  if (/^symbol(Asterisk|Diamond2|Plus|Square2|Times|Triangle2)$/.test(node?.name)) skip("api: stroke symbols (Later)");
  return FILL.includes(node?.name?.slice(6)) ? `shape::Symbol::${node.name.slice(6)}` : skip(API);
}

const gen = generators({
  types: ["symbol"],
  make: { symbol: ([t, s], ctx) => ({ t: "symbol", type: t ? kind(t) : "shape::Symbol::Circle", size: s ? size(s, ctx) : "64.0" }) },
  set(g, prop, [arg], ctx) {
    if (!arg) skip(API);
    return { type: () => ({ ...g, type: kind(arg) }), size: () => ({ ...g, size: size(arg, ctx) }), context: () => ({ ...g, into: val(ex(arg, ctx)).c }) }[prop]?.();
  },
  run(g, [d], ctx) {
    const s = g.size === "datum" ? num(d ?? skip("js: no datum (undefined)"), ctx) : g.size;
    return g.into ? { c: `${g.type}.draw(${s}, &mut ${g.into})`, t: "unit" } : { c: `drawn(|ctx| ${g.type}.draw(${s}, ctx))`, t: "path" };
  },
});

export default {
  ...gen,
  identifiers: { symbolsFill: { c: "shape::Symbol::FILL", t: "symbols" }, symbolsStroke: { t: "stroke" }, symbols: { t: "alias" },
    ...Object.fromEntries(FILL.map((n) => [`symbol${n}`, { c: `shape::Symbol::${n}`, t: "symbol-type" }])) },
  call(node, ctx) {
    if (node.callee.name === "polygonContext") return { c: "Polygon::default()", t: "polygon" };
    const p = node.callee.property?.name === "area" && val(ex(node.callee.object, ctx));
    return p?.t === "polygon" ? { c: `${p.c}.area()`, t: "num" } : gen.call(node, ctx);
  },
  array: (node) => (node.elements.length && node.elements.every((e) => /^symbol[A-Z]/.test(e?.name)) ? { c: `[${node.elements.map(kind).join(", ")}]`, t: "symbols" } : undefined),
  equal: (a, b) => (a.t === "alias" ? skip("js: object identity (symbols === symbolsFill)") : a.t === "symbols" && b.t === "symbols" ? `(${a.c} == ${b.c})` : undefined),
};
