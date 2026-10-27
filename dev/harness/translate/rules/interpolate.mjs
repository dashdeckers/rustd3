// d3-interpolate's number interpolators: interpolateNumber(a, b),
// interpolateRound(a, b) and interpolateHue(a, b) are closures of t;
// piecewise over numbers with d3's value interpolate (or none) is
// interpolate::piecewise with interpolate::number; quantize(f, n) samples a
// closure, colors printed by Display.
import { calleeName, ex, needNum, skip, val } from "../expr.mjs";

const FNS = { interpolateNumber: "interpolate::number", interpolateRound: "interpolate::round", interpolateHue: "interpolate::hue" };

export default {
  call(node, ctx) {
    const name = calleeName(node, ctx);
    if (name === "piecewise") {
      const [first, values = first] = node.arguments;
      if (values !== first && (first.type !== "Identifier" || first.name !== "interpolate" || ctx.vars.has("interpolate"))) skip("gap: piecewise with another interpolator");
      const e = values?.type === "ArrayExpression" ? ex(values, ctx) : skip("js: piecewise of other than an array literal");
      if (e.t !== "vec") skip("types: d3's value interpolation of other than numbers (types decide)");
      return { c: `interpolate::piecewise(&[${e.items.join(", ")}], |a: &f64, b: &f64| interpolate::number(*a, *b))`, t: "fn", arity: 1 };
    }
    if (name === "quantize") {
      const [f, n] = node.arguments.map((a) => val(ex(a, ctx)));
      if (f?.t !== "fn" || f.arity !== 1 || !["num", "rgb", undefined].includes(f.ret)) skip("js: quantize of other than an interpolator");
      const count = /^\d+\.0$/.test(needNum(n, "a sample count")) ? n.c.slice(0, -2) : skip("types: a sample count other than a whole number");
      const c = `interpolate::quantize(${f.c}, ${count})`;
      return f.ret === "rgb" ? { c: `${c}.iter().map(ToString::to_string).collect::<Vec<_>>()`, t: "strs" } : { c, t: "vec" };
    }
    if (!FNS[name]) return undefined;
    const args = node.arguments.map((a) => ex(a, ctx));
    if (args.length !== 2) skip(`js: ${args.length} arguments to ${name} (undefined)`);
    return { c: `${FNS[name]}(${args.map((a) => needNum(a, `${name} argument`)).join(", ")})`, t: "fn", arity: 1 };
  },
};
