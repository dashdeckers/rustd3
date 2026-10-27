// d3-color's rgb, hsl and color, its test helpers, and d3-interpolate's
// interpolateRgb at gamma 1. The vendored d3 parses string literals here: one
// it rejects is an Err in Rust, a types: skip, except in color(s), which
// stays a Result for `strictEqual(…, null)`. A color is {t: "rgb"} or
// {t: "hsl"}, printed by Display (d3's toString); its fields are public, so
// writes to them and copy({…}) (struct update) translate.
import { color as d3color, hsl as d3hsl } from "../../d3/d3-color/src/index.js";
import { calleeName, ex, needNum, skip, val } from "../expr.mjs";

const FIELDS = { rgb: ["r", "g", "b", "opacity"], hsl: ["h", "s", "l", "opacity"] };
const TYPES = { rgb: "Rgb", hsl: "Hsl" };
const NAMES = ["color", "rgb", "hsl", "interpolateRgb"];

/** `e` as `space` (rgb or hsl): d3's rgb(e) or hsl(e) of a string literal or a color. */
export function into(e, space) {
  if (e.t === space) return e;
  if (FIELDS[e.t]) return { c: `${TYPES[space]}::from(${e.c})`, t: space };
  if (e.t !== "str" || e.raw === undefined) skip(`js: a ${e.t} as a color (coercion)`);
  if (!d3color(e.raw)) skip("types: a string that is no CSS color is an Err, where d3 makes NaN channels");
  return { c: `${e.c}.parse::<${TYPES[space]}>().expect("a CSS color")`, t: space };
}

/** A d3 test helper `assert…Equal(actual, a, b, c, opacity)` as the prelude predicate `fn` over `space`. */
const helper = (fn, space) => ([actual, ...expected], id) => {
  if (actual.t !== space) skip("js: instanceof");
  return `check(${id}, Kind::Strict, &${fn}(&${actual.c}, ${expected.map((x) => needNum(x, "a channel")).join(", ")}), &true);`;
};

const str = (c) => ({ c, t: "str" });
const METHODS = {
  brighter: (e, [k]) => ({ c: `${e.c}.brighter(${k ? needNum(k, "k") : "1.0"})`, t: e.t }),
  darker: (e, [k]) => ({ c: `${e.c}.darker(${k ? needNum(k, "k") : "1.0"})`, t: e.t }),
  clamp: (e) => ({ c: `${e.c}.clamp()`, t: e.t }),
  displayable: (e) => ({ c: `${e.c}.displayable()`, t: "bool" }),
  copy: (e) => e,
  rgb: (e) => into(e, "rgb"),
  formatRgb: (e) => str(`${e.c}.to_string()`),
  formatHsl: (e) => str(`${into(e, "hsl").c}.format_hsl()`),
  formatHex: (e) => str(`${into(e, "rgb").c}.format_hex()`),
  formatHex8: (e) => str(`${into(e, "rgb").c}.format_hex8()`),
};
METHODS.hex = METHODS.formatHex;

export default {
  display: ["rgb", "hsl"],
  identifiers: Object.fromEntries(NAMES.map((n) => [n, { c: n, t: "d3fn" }])),
  assertions: {
    assertRgbApproxEqual: helper("rgb_approx_equal", "rgb"),
    assertRgbEqual: helper("rgb_equal", "rgb"),
    assertHslEqual: helper("hsl_equal", "hsl"),
  },
  call(node, ctx) {
    const [first] = node.arguments;
    if (node.callee.property?.name === "copy" && first?.type === "ObjectExpression") {
      const e = val(ex(node.callee.object, ctx));
      const set = (p) => (FIELDS[e.t].includes(p.key?.name) ? `${p.key.name}: ${needNum(val(ex(p.value, ctx)), "a channel")}` : skip("js: a property other than a channel"));
      return FIELDS[e.t] ? { c: `${TYPES[e.t]} { ${first.properties.map(set).join(", ")}, ..${e.c} }`, t: e.t } : undefined;
    }
    const name = calleeName(node, ctx);
    if (!NAMES.includes(name) || node.type !== "CallExpression") return undefined;
    const args = node.arguments.map((a) => val(ex(a, ctx)));
    if (name === "interpolateRgb") {
      if (args.length !== 2) skip(`js: ${args.length} arguments to interpolateRgb (undefined)`);
      return { c: `interpolate_rgb(${into(args[0], "rgb").c}, ${into(args[1], "rgb").c})`, t: "fn", arity: 1, ret: "rgb" };
    }
    if (name === "color") {
      if (args.length !== 1 || args[0].t !== "str") skip("js: color of other than one string (coercion)");
      const parsed = args[0].raw !== undefined && d3color(args[0].raw);
      return parsed ? into(args[0], parsed instanceof d3hsl ? "hsl" : "rgb") : { c: `${args[0].c}.parse::<Rgb>()`, t: "result-rgb" };
    }
    if (args.length === 1) return into(args[0], name);
    const [a, b, c, o] = args.map((x) => needNum(x, `an ${name} channel`));
    if (name === "rgb") return { c: `Rgb::new(${a}, ${b}, ${c})${o ? `.with_opacity(${o})` : ""}`, t: "rgb" };
    return { c: `Hsl { h: ${a}, s: ${b}, l: ${c}, opacity: ${o ?? "1.0"} }`, t: "hsl" };
  },
  method(obj, prop, args) {
    if (obj.t === "d3fn" && prop === "gamma") skip("gap: phase 23 (interpolateRgb.gamma)");
    return FIELDS[obj.t] ? (METHODS[prop]?.(obj, args) ?? skip(`gap: ${obj.t}.${prop}`)) : undefined;
  },
  // c.r = v, c.h += v, ++c.g on a color variable; d3's TestColor prototypes are js.
  assign(node, ctx) {
    const update = node.type === "UpdateExpression";
    const left = update ? node.argument : node.left;
    const t = left.object?.type === "Identifier" && !left.computed && ctx.vars.get(left.object.name)?.t;
    if (t === "local") skip("js: prototypes");
    if (!FIELDS[t]?.includes(left.property.name) || !(update || ["=", "+=", "-=", "*=", "/="].includes(node.operator))) return undefined;
    const target = `${ex(left.object, ctx).c}.${left.property.name}`;
    const stmt = update ? `${target} ${node.operator[0]}= 1.0;` : `${target} ${node.operator} ${needNum(val(ex(node.right, ctx)), "a channel")};`;
    return { c: update && !node.prefix ? `{ let old = ${target}; ${stmt} old }` : `{ ${stmt} ${target} }`, t: "num", stmt };
  },
  field(obj, prop) {
    if (obj.t === "d3fn") skip("js: prototypes");
    return FIELDS[obj.t]?.includes(prop) ? { c: `${obj.c}.${prop}`, t: "num" } : undefined;
  },
};
