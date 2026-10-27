// d3-shape's stack(), orders and offsets. Literal series are Vec<Vec<[f64; 2]>> ("layers"), which
// Offset::apply changes in place. stack().keys([i…])(rows) lays out the columns rows[·][i] as
// (layers, stacking order), as the file's series(points, data, key, index) helper gives them.
import { ex, needNum, skip, val } from "../expr.mjs";
import { generators } from "./generator.mjs";

const API = "api: stack takes columns; keys and values as accessors, and getters, are not in the Rust API";
const isArray = (n) => n?.type === "ArrayExpression";
const num = (node, ctx) => needNum(val(ex(node, ctx)), "a stack value");
const round = (c) => `${c}.iter().map(|l| l.iter().map(|p| p.map(|v| js_round(v * 1e6) / 1e6)).collect::<Vec<_>>()).collect::<Vec<_>>()`;
const pair = (p, ctx) => (isArray(p) && p.elements.length === 2 ? `[${p.elements.map((e) => num(e, ctx)).join(", ")}]` : skip("types: a point of other than two values"));
const layers = (ls, ctx) => `vec![${ls.map((l) => `vec![${l.elements.map((p) => pair(p, ctx)).join(", ")}]`).join(", ")}]`;
const NAMES = [["Order", "None Ascending Descending InsideOut Appearance Reverse"], ["Offset", "None Expand Diverging Silhouette Wiggle"]];

const gen = generators({
  types: ["stack"],
  make: { stack: () => ({ c: "shape::Stack::default()", t: "stack" }) },
  set(g, prop, [arg], ctx) {
    if (prop === "keys" && arg?.elements?.every((e) => Number.isInteger(e?.value))) return { ...g, keys: arg.elements.map((e) => e.value) };
    const kind = { order: "Order", offset: "Offset" }[prop];
    const o = !arg || !kind ? skip(API) : arg.value === null ? { c: `shape::stack::${kind}::None`, t: prop } : val(ex(arg, ctx));
    return o.t === prop ? { ...g, c: `${g.c}.${prop}(${o.c})` } : skip(API);
  },
  run(g, [arg], ctx) {
    const rows = (g.keys && arg && val(ex(arg, ctx)).rows) || skip(API);
    const cols = g.keys.map((k) => `&[${rows.map((r) => r[k] ?? skip("js: a key past a row (undefined)")).join(", ")}][..]`);
    return { c: `{ let columns = [${cols.join(", ")}]; (${g.c}.layout(&columns), ${g.c}.stacking_order(&columns)) }`, t: "stacked" };
  },
});

export default {
  ...gen,
  identifiers: Object.fromEntries(NAMES.flatMap(([k, ns]) => ns.split(" ").map((n) => [`stack${k}${n}`, { c: `shape::stack::${k}::${n}`, t: k.toLowerCase() }]))),
  call(node, ctx) {
    if (node.type === "NewExpression") return node.callee.name === "Array" ? { c: `vec![Vec::new(); ${num(node.arguments[0], ctx)} as usize]`, t: "layers" } : undefined;
    if (node.callee.property?.name !== "map" || node.arguments[0]?.name !== "roundSeries") return gen.call(node, ctx);
    const e = val(ex(node.callee.object, ctx));
    return { stacked: { c: round(`${e.c}.0`), t: "layers" }, layers: { c: round(e.c), t: "layers" } }[e.t];
  },
  apply(target, [series, order]) {
    if (target.t === "order") return { c: `${target.c}.apply(&${series.c})`, t: "order-of" };
    const ok = target.t === "offset" && series?.t === "layers" && order?.t === "order-of";
    return ok ? { c: `{ let order = ${order.c}; ${target.c}.apply(&mut ${series.c}, &order); }`, t: "unit" } : undefined;
  },
  array(node, ctx) {
    const els = node.elements;
    if (els.length && els.every((e) => e?.callee?.name === "series")) {
      const order = els.map((e, i) => [e.arguments[3].value, i]).sort((a, b) => a[0] - b[0]).map(([, i]) => i);
      return { c: `(${layers(els.map((e) => e.arguments[0]), ctx)}, vec![${order.join(", ")}usize])`, t: "stacked" };
    }
    if (els.length && els.every((l) => isArray(l) && l.elements.every(isArray))) return { c: layers(els, ctx), t: "layers" };
    if (!els.length || !els.every((r) => isArray(r) && r.elements.length > 2)) return undefined;
    const rows = els.map((r) => r.elements.map((e) => num(e, ctx)));
    return { c: `vec![${rows.map((r) => `vec![${r.join(", ")}]`).join(", ")}]`, t: "rows", rows };
  },
  field(obj, prop, ctx, node) {
    const next = { layers: "layer", layer: "point", point: "num" }[obj.t];
    const i = node.computed ? node.property.value : NaN;
    if (!next || !Number.isInteger(i)) return undefined;
    return obj.t === "point" && i > 1 ? skip("js: a point's third value (undefined)") : { c: `${obj.c}[${i}]`, t: next };
  },
  // A "NaN" written into a series to compare it: not made, and the series is not read again (types:).
  statement(s, ctx) {
    let left = s.expression?.type === "AssignmentExpression" ? s.expression.left : null;
    if (left?.type !== "MemberExpression") return gen.statement(s, ctx);
    while (left.type === "MemberExpression") left = left.object;
    ctx.written.add(left.name);
    return [];
  },
};
