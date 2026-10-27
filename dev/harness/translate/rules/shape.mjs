// d3-shape's line(), area(), lineRadial() and areaRadial() over [[x, y], …] data, and its curves. A
// generator is {t: "line"} or {t: "area"}; calling one draws its data into
// a fresh SvgPath (the prelude's `drawn`, None for d3's null path). A curve
// is {t: "curve"} whose `param` (tension, alpha or beta) has a builder.
// Points are {t: "points"} with the code of each coordinate pair; a file's
// `reflect`-style helper maps them here, one {t: "point"} at a time. d3's
// accessors, getters, contexts and digits are not in the Rust API (`api:`).
import { calleeName, ex, skip, val } from "../expr.mjs";

const NAMES = ["Basis", "BasisClosed", "BasisOpen", "BumpX", "BumpY", "Bundle", "Cardinal", "CardinalClosed", "CardinalOpen",
  "CatmullRom", "CatmullRomClosed", "CatmullRomOpen", "Linear", "LinearClosed", "MonotoneX", "MonotoneY", "Natural", "Step",
  "StepAfter", "StepBefore"];
const PARAMS = { Bundle: ["beta", "0.85"], Cardinal: ["tension", "0.0"], CatmullRom: ["alpha", "0.5"] };
const ACCESSORS = ["x", "y", "x0", "x1", "y0", "y1", "defined", "context", "digits", "lineX0", "lineX1", "lineY0", "lineY1", "angle", "radius",
  "startAngle", "endAngle", "innerRadius", "outerRadius", "lineStartAngle", "lineEndAngle", "lineInnerRadius", "lineOuterRadius"];
const RADIAL = { lineRadial: ["shape::line::radial()", "line"], areaRadial: ["shape::area::radial()", "area"] };
const METHODS = new Set([...ACCESSORS, "curve", "tension", "alpha", "beta"]);
const ACCESSOR = "api: d3's accessors, getters and contexts (generators take geometry and draw into a PathContext)";
const generator = (e) => e.t === "line" || e.t === "area";

/** Curve::{name} with its parameter at `value` (d3's default when absent). */
function curve(name, value) {
  const [param, initial] = PARAMS[name.replace(/Closed|Open/, "")] ?? [];
  return { c: param ? `Curve::${name} { ${param}: ${value ?? initial} }` : `Curve::${name}`, t: "curve", name, param };
}

/** Points from pairs of coordinate code. */
const points = (pairs) => ({
  c: pairs.length ? `vec![${pairs.map(([x, y]) => `(${x}, ${y})`).join(", ")}]` : "Vec::<(f64, f64)>::new()",
  t: "points",
  pairs,
});
const pair = (e) => (e.t === "vec" && e.items?.length === 2 ? e.items : skip("types: a point of other than two coordinates"));

/** The data a generator draws: points or `[]`; points in a variable cloned, as `draw` consumes them. */
function data([arg], ctx) {
  const e = val(arg ? ex(arg, ctx) : skip("js: no data (undefined)"));
  if (e.t === "vec" && !e.items?.length) return points([]).c;
  if (e.t !== "points") skip(`js: a ${e.t} as data (coercion)`);
  return arg.type === "Identifier" ? `${e.c}.clone()` : e.c;
}

/** `[[x, y], …].map(f)` for a file-level `function f(p) { return [p[i], p[j]]; }`, applied to each point here. */
function mapped(node, ctx) {
  const [f] = node.arguments;
  const helper = f?.type === "Identifier" && ctx.file.helpers.get(f.name);
  if (helper?.kind !== "fn" || node.callee.object.type !== "ArrayExpression") return undefined;
  const { params, body } = helper.node;
  const [ret] = body.body;
  if (params.length !== 1 || body.body.length !== 1 || ret.type !== "ReturnStatement") skip(`helpers: ${f.name} is not one return`);
  const list = val(ex(node.callee.object, ctx));
  const pairs = list.t === "points" ? list.pairs : list.items?.length === 0 ? [] : skip(`js: ${f.name} of a ${list.t}`);
  const vars = ctx.vars;
  try {
    return points(pairs.map((p) => {
      ctx.vars = new Map(vars).set(params[0].name, { c: params[0].name, t: "point", pair: p });
      return pair(val(ex(ret.argument, ctx)));
    }));
  } finally {
    ctx.vars = vars;
  }
}

export default {
  identifiers: Object.fromEntries(NAMES.map((n) => [`curve${n}`, curve(n)])),
  nullable: ["path"],
  call(node, ctx) {
    const { callee } = node;
    const name = calleeName(node, ctx);
    if (name === "line" || name === "area" || RADIAL[name]) {
      const [c, t] = RADIAL[name] ?? [`shape::${name}()`, name];
      return node.arguments.length ? skip(ACCESSOR) : { c, t };
    }
    if (callee.type === "MemberExpression" && !callee.computed) {
      const prop = callee.property.name;
      if (prop === "map") return mapped(node, ctx);
      if (!METHODS.has(prop)) return undefined;
      const obj = val(ex(callee.object, ctx));
      const arg = () => val(ex(node.arguments[0], ctx));
      if (obj.t === "curve" && prop === obj.param) {
        const value = node.arguments.length ? arg() : skip(`api: d3's ${prop}() getter`);
        return value.t === "num" ? curve(obj.name, value.c) : skip(`js: ${prop}(${value.t}) coerced to a number`);
      }
      if (!generator(obj)) return undefined;
      if (prop !== "curve" || !node.arguments.length) skip(ACCESSOR);
      // d3's curve(_) ignores any further argument, as this does.
      const c = arg();
      return c.t === "curve" ? { c: `${obj.c}.curve(${c.c})`, t: obj.t } : skip(`js: a ${c.t} as a curve`);
    }
    if (callee.type !== "Identifier" && callee.type !== "CallExpression") return undefined;
    const g = val(ex(callee, ctx));
    return generator(g) ? { c: `drawn(|ctx| ${g.c}.draw(${data(node.arguments, ctx)}, ctx))`, t: "path" } : undefined;
  },
  array(node, ctx) {
    if (!node.elements.length || !node.elements.every((e) => e?.type === "ArrayExpression")) return undefined;
    return points(node.elements.map((e) => pair(val(ex(e, ctx)))));
  },
  field(obj, prop, ctx, node) {
    if (obj.t !== "point") return undefined;
    const i = node.computed && node.property.type === "Literal" ? node.property.value : NaN;
    return i === 0 || i === 1 ? { c: obj.pair[i], t: "num" } : skip(`js: ${prop} of a point`);
  },
  // Accessor functions are declared before use, and digits set on a
  // generator change what it draws afterwards: the whole test is api.
  statement(s, ctx) {
    if (s.type === "VariableDeclaration" && s.declarations.some((d) => /Function/.test(d.init?.type ?? ""))) skip(ACCESSOR);
    if (ctx.file.source.slice(s.start, s.end).includes(".digits(")) {
      skip("api: line.digits (a PathContext rounds: SvgPath::with_digits, d3's pathRound)");
    }
  },
};
