// d3-path's names: path() and pathRound() build an SvgPath, whose methods
// draw. arc and arcTo are fallible (d3 throws on a negative radius), and
// d3-path's test helper precision(path, digits) is the prelude's.
import { calleeName, ex, needNum, skip } from "../expr.mjs";

const DRAW = { moveTo: ["move_to", 2], lineTo: ["line_to", 2], closePath: ["close_path", 0], rect: ["rect", 4],
  quadraticCurveTo: ["quadratic_curve_to", 4], bezierCurveTo: ["bezier_curve_to", 6] };

function nums(args, n, what) {
  if (args.length !== n) skip(`js: ${args.length} arguments to ${what} for ${n} (undefined)`);
  return args.map((a) => needNum(a, `${what} argument`)).join(", ");
}

export default {
  display: ["path"],
  call(node, ctx) {
    const name = calleeName(node, ctx);
    if (name === "path") return { c: "SvgPath::new()", t: "path" };
    if (name === "pathRound") {
      const [d] = node.arguments;
      // pathRound() is three digits, pathRound(null) zero; digits are a u32.
      const digits = !d ? 3 : d.type === "Literal" && d.value === null ? 0 : d.value;
      if (!Number.isInteger(digits) || digits < 0) skip("types: with_digits takes a u32 (d3 throws for NaN and below 0)");
      return { c: `SvgPath::with_digits(${digits})`, t: "path" };
    }
    if (name === "precision") {
      const [s, p] = node.arguments.map((a) => ex(a, ctx));
      if (s?.t !== "str") skip("js: precision of a non-string (coercion)");
      return { c: `precision(&${s.c}, ${needNum(p, "precision digits")})`, t: "str" };
    }
    return undefined;
  },
  method(obj, prop, args) {
    if (obj.t !== "path") return undefined;
    if (prop === "toString") return { c: `${obj.c}.to_string()`, t: "str" };
    if (DRAW[prop]) return { c: `${obj.c}.${DRAW[prop][0]}(${nums(args, DRAW[prop][1], prop)})`, t: "unit" };
    if (prop === "arcTo") return { c: `${obj.c}.arc_to(${nums(args, 5, prop)})`, t: "result", expect: "d3 test arcTo" };
    if (prop !== "arc") return undefined;
    const [ccw = { c: "false", t: "bool" }] = args.slice(5);
    if (ccw.t !== "bool") skip("js: a non-boolean anticlockwise flag (truthiness)");
    return { c: `${obj.c}.try_arc(${nums(args.slice(0, 5), 5, prop)}, ${ccw.c})`, t: "result", expect: "d3 test arc" };
  },
  // The loops in path-test.js pass JavaScript's truthy and falsy values as the flag.
  statement(s) {
    if (s.type === "ForOfStatement") skip("js: truthy and falsy anticlockwise flags (truthiness)");
  },
};
