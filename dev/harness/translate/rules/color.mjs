// d3-color's rgb, hsl, lab, hcl, cubehelix, gray, lch and color, its test
// helpers, and d3-interpolate's color interpolators (interpolateRgb …
// interpolateCubehelixLong, `.gamma(y)` included). The vendored d3
// parses string literals here: one it rejects is an Err in Rust, a types:
// skip, except in color(s), which stays a Result for `strictEqual(…, null)`.
// A color is {t: "rgb"}, {t: "hsl"} and so on, printed by Display (d3's
// toString); its fields are public, so writes to them and copy({…}) (struct
// update) translate.
import { color as d3color, hsl as d3hsl } from "../../d3/d3-color/src/index.js";
import { calleeName, ex, needNum, skip, val } from "../expr.mjs";

const FIELDS = {
  rgb: ["r", "g", "b", "opacity"], hsl: ["h", "s", "l", "opacity"], lab: ["l", "a", "b", "opacity"],
  hcl: ["h", "c", "l", "opacity"], cubehelix: ["h", "s", "l", "opacity"],
};
const TYPES = { rgb: "Rgb", hsl: "Hsl", lab: "Lab", hcl: "Hcl", cubehelix: "Cubehelix" };
// Constructors of three channels; lch takes them in reverse, gray takes one.
const NEW = { rgb: "Rgb::new", lab: "Lab::new", hcl: "Hcl::new", cubehelix: "Cubehelix::new", lch: "Hcl::lch", gray: "Lab::gray" };
const SPACE = { lch: "hcl", gray: "lab" };
// d3's color interpolators: the Rust function, its space, and the function `.gamma(y)` makes.
const BLENDS = {
  interpolateRgb: ["rgb", "rgb", "rgb_gamma"], interpolateHsl: ["hsl", "hsl"], interpolateHslLong: ["hsl_long", "hsl"],
  interpolateLab: ["lab", "lab"], interpolateHcl: ["hcl", "hcl"], interpolateHclLong: ["hcl_long", "hcl"],
  interpolateCubehelix: ["cubehelix", "cubehelix", "cubehelix"], interpolateCubehelixLong: ["cubehelix_long", "cubehelix", "cubehelix_long"],
};
const NAMES = ["color", "rgb", "hsl", "lab", "hcl", "cubehelix", "gray", "lch", ...Object.keys(BLENDS)];

/** d3's color interpolator `name` of the colors `args`, or `name.gamma(y)`'s with `gamma` set: a closure of t. */
export function blend(name, args, gamma) {
  if (!BLENDS[name]) return undefined;
  const [f, space, gammaFn] = BLENDS[name];
  if (args.length !== 2) skip(`js: ${args.length} arguments to ${name} (undefined)`);
  const first = gamma !== undefined ? `${gamma}, ` : space === "cubehelix" ? "1.0, " : "";
  const c = `interpolate::${gamma !== undefined ? gammaFn : f}(${first}${args.map((a) => into(a, space).c).join(", ")})`;
  return { c, t: "fn", arity: 1, ret: "rgb" };
}

/**
 * `e` as `space`: d3's rgb(e), hsl(e), lab(e) … of a string literal or a
 * color. Lab and Hcl convert into each other directly, every other pair
 * through Rgb, as d3 does.
 */
export function into(e, space) {
  if (e.t === space) return e;
  if (FIELDS[e.t]) {
    const direct = e.t === "rgb" || space === "rgb" || (["lab", "hcl"].includes(e.t) && ["lab", "hcl"].includes(space));
    return { c: `${TYPES[space]}::from(${direct ? e.c : `Rgb::from(${e.c})`})`, t: space };
  }
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
  // d3's Color.displayable is rgb().displayable(); Hsl has its own.
  displayable: (e) => ({ c: `${(e.t === "hsl" ? e : into(e, "rgb")).c}.displayable()`, t: "bool" }),
  copy: (e) => e,
  rgb: (e) => into(e, "rgb"),
  formatRgb: (e) => str(`${e.c}.to_string()`),
  formatHsl: (e) => str(`${into(e, "hsl").c}.format_hsl()`),
  formatHex: (e) => str(`${into(e, "rgb").c}.format_hex()`),
  formatHex8: (e) => str(`${into(e, "rgb").c}.format_hex8()`),
};
METHODS.hex = METHODS.formatHex;

export default {
  display: Object.keys(TYPES),
  identifiers: Object.fromEntries(NAMES.map((n) => [n, { c: n, t: "d3fn" }])),
  assertions: {
    assertRgbApproxEqual: helper("rgb_approx_equal", "rgb"),
    assertRgbEqual: helper("rgb_equal", "rgb"),
    assertHslEqual: helper("hsl_equal", "hsl"),
    assertLabEqual: helper("lab_equal", "lab"),
    assertHclEqual: helper("hcl_equal", "hcl"),
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
    if (BLENDS[name]) return blend(name, args);
    if (name === "color") {
      if (args.length !== 1 || args[0].t !== "str") skip("js: color of other than one string (coercion)");
      const parsed = args[0].raw !== undefined && d3color(args[0].raw);
      return parsed ? into(args[0], parsed instanceof d3hsl ? "hsl" : "rgb") : { c: `${args[0].c}.parse::<Rgb>()`, t: "result-rgb" };
    }
    const space = SPACE[name] ?? name;
    const channels = name === "gray" ? 1 : 3;
    if (args.length === 1 && name !== "gray") return into(args[0], space);
    const made = args.map((x) => needNum(x, `an ${name} channel`));
    if (made.length < channels || made.length > channels + 1) skip(`js: ${made.length} arguments to ${name} (undefined)`);
    const [o] = made.splice(channels);
    if (name === "hsl") return { c: `Hsl { h: ${made[0]}, s: ${made[1]}, l: ${made[2]}, opacity: ${o ?? "1.0"} }`, t: space };
    const c = `${NEW[name]}(${made.join(", ")})`;
    if (o === undefined) return { c, t: space };
    return { c: name === "rgb" ? `${c}.with_opacity(${o})` : `${TYPES[space]} { opacity: ${o}, ..${c} }`, t: space };
  },
  method(obj, prop, args) {
    if (obj.t === "d3fn" && prop === "gamma" && BLENDS[obj.c]?.[2]) return { c: obj.c, t: "gamma", gamma: needNum(args[0], "gamma") };
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
  apply: (target, args) => (target.t === "gamma" ? blend(target.c, args.map(val), target.gamma) : undefined),
  field(obj, prop) {
    if (obj.t === "d3fn") skip("js: prototypes");
    return FIELDS[obj.t]?.includes(prop) ? { c: `${obj.c}.${prop}`, t: "num" } : undefined;
  },
};
