// d3-scale's continuous scales and scaleIdentity. scaleLinear, scalePow,
// scaleSqrt, scaleLog, scaleSymlog and scaleIdentity are scale::linear() …
// scale::identity(), whose domain and range are one array; a chain of method
// calls is translated whole, so that setters on a variable assign it
// (`s = s.domain(…)`) and later reads see the change as d3's do. Tick
// formats are under EN_US, d3's default. A scale value is {t: "scale", tr,
// n} (its transform and stop count). A linear chain whose domain and range
// both reach n > 2 stops becomes scale::piecewise(…) followed by its calls;
// unequal lengths, a variable or chain changing n, and range values other
// than numbers or colors are skips decided at the first step that reads the
// scale. Colors (CSS strings, rgb(…) and hsl(…)) make an Rgb range, r: "rgb",
// whose apply is an Rgb; a variable whose range changes type is shadowed.
import { color } from "../../d3/d3-color/src/index.js";
import { calleeName, ex, needNum, rustStr, skip, val } from "../expr.mjs";
import array from "./array.mjs";
import { blend, into } from "./color.mjs";

const EN_US = "&format::Locale::EN_US";
const MAKE = {
  scaleLinear: ["linear", "linear"], scalePow: ["pow", "pow"], scaleSqrt: ["sqrt", "pow"], scaleLog: ["log", "log"],
  scaleSymlog: ["symlog", "symlog"], scaleIdentity: ["identity", "identity"],
};
const SETTERS = { domain: "domain", range: "range", rangeRound: "range_round" };
const BUILDERS = ["exponent", "base", "constant"];
const METHODS = new Set([...Object.keys(SETTERS), ...BUILDERS, "clamp", "unknown", "interpolate", "nice", "ticks", "tickFormat", "copy", "invert"]);
const INTERPOLATORS = { interpolateNumber: "number", interpolateRound: "round" };

/** A tick count as a usize literal; d3's 10 when absent. */
function count(e) {
  if (!e || e.t === "undef") return "10";
  if (e.t === "num" && /^\d+\.0$/.test(e.c)) return e.c.slice(0, -2);
  return skip("types: a tick count other than a whole number (counts are usize)");
}

/** A specifier literal as an Option<&str>. */
function specifier(node) {
  if (!node) return "None";
  if (node.type === "Literal" && typeof node.value === "string") return `Some(${rustStr(node.value)})`;
  return skip("types: a specifier other than a string (tick formats take a &str)");
}

/** A domain or range argument: its Rust array, its length, and why its values cannot be one. */
function stops(node, what, ctx) {
  if (node.type === "NewExpression") return { n: 2, reason: `js: a ${what} from an iterable` };
  if (node.type !== "ArrayExpression") {
    const e = val(ex(node, ctx));
    if (!e.items) skip(`js: a ${what} array held in a variable (Rust copies arrays)`);
    return { c: `[${e.items.join(", ")}]`, n: e.items.length };
  }
  const n = node.elements.length;
  const kinds = node.elements.map((el) => (el?.type === "Literal" ? typeof el.value : el?.type));
  if (kinds.includes("NewExpression")) return { n, reason: `js: Dates and Number objects in a ${what} (coercion)` };
  const colors = node.elements.every((el) => el?.type === "CallExpression" || (typeof el?.value === "string" && color(el.value)));
  if (what === "range" && colors) return { c: `[${node.elements.map((el) => into(val(ex(el, ctx)), "rgb").c).join(", ")}]`, n, r: "rgb" };
  if (kinds.includes("CallExpression")) return { n, reason: `js: calls as ${what} values (d3's runtime interpolate)` };
  if (kinds.includes("ObjectExpression") || kinds.includes("ArrayExpression")) return { n, reason: "js: objects and arrays as range values (d3's runtime interpolate)" };
  if (kinds.includes("string")) return { n, reason: `js: strings as ${what} values (coercion)` };
  return { c: `[${node.elements.map((el) => needNum(ex(el, ctx), `a ${what} value`)).join(", ")}]`, n };
}

/** A scale's chain: its root's value, then each method call in turn. */
function chain(root, steps, ctx) {
  const name = [...ctx.vars.values()].includes(root) ? root.c : null;
  let code = root.c;
  let muts = name ? "" : null; // setters applied to the variable `name`
  let n = root.n ?? 2;
  const sizes = { domain: { n }, range: { n } };
  const emitted = []; // the lengths of the setters written so far
  let r = root.r ?? "num"; // the range type
  const retyped = () => muts && r !== (root.r ?? "num");
  const SHADOW = "types: a variable whose range changes type (the range type is part of a scale's)";
  // Blocks are parenthesized, so that a method call may follow them in any position.
  const current = () => (retyped() ? skip(SHADOW) : muts ? `({ ${name} = ${name}${muts}; ${name}.clone() })` : code);
  const settled = () => {
    const [d, r] = [sizes.domain, sizes.range];
    if (d.n !== r.n) skip("types: a domain and range of different lengths (the type sets the count)");
    if (d.reason || r.reason) skip(d.reason ?? r.reason);
    if (d.n === n) return;
    if (root.tr === "identity") skip("types: an identity scale with other than two stops (its type holds two)");
    if (name !== null || emitted.some((k) => k !== d.n)) skip("types: a scale whose stop count changes (N is part of its type)");
    if (root.tr !== "linear") skip("api: polylinear log, pow and symlog scales (piecewise is linear only)");
    if (!code.startsWith("scale::linear()")) skip("gap: a polylinear scale built from another scale");
    code = `scale::piecewise(${d.c}, ${r.c})${code.slice("scale::linear()".length)}`;
    n = d.n;
  };
  const build = (suffix) => (muts !== null ? (muts += suffix) : (code += suffix));
  const read = (suffix, t) => ({ c: retyped() ? skip(SHADOW) : muts ? `({ ${name} = ${name}${muts}; ${name}${suffix} })` : `${code}${suffix}`, t });
  for (const { prop, args } of steps) {
    const arg = (k) => (args[k] ? val(ex(args[k], ctx)) : undefined);
    if (SETTERS[prop] && args.length) {
      const what = prop === "domain" ? "domain" : "range";
      sizes[what] = stops(args[0], what, ctx);
      if (root.tr === "identity") sizes.domain = sizes.range = sizes[what];
      if (prop === "rangeRound" && sizes.range.r) skip("js: rangeRound over colors (interpolateRound makes NaN)");
      if (what === "range" && !sizes.range.reason) r = sizes.range.r ?? "num";
      if (!sizes[what].reason) {
        build(`.${SETTERS[prop]}(${sizes[what].c})`);
        emitted.push(sizes[what].n);
      }
      continue;
    }
    settled();
    if (!args.length && ["clamp", "unknown", "interpolate", ...BUILDERS].includes(prop)) skip(`api: d3's ${prop}() getter (only domain and range have get_)`);
    if (r !== "num" && ["range", "unknown"].includes(prop)) skip(`gap: the ${prop} of a color range`);
    if (prop === "domain" || prop === "range") return read(`.get_${prop}().to_vec()`, "vec");
    if (prop === "ticks") return read(`.ticks(${count(arg(0))})`, "vec");
    if (prop === "tickFormat") return read(`.tick_format_in(${EN_US}, ${count(arg(0))}, ${specifier(args[1])})`, "result-tickfmt");
    if (prop === "invert" && r !== "num") skip("types: invert exists only for number ranges (E0599)");
    if (prop === "invert") return read(`.invert(${needNum(arg(0), "inverted value")})`, "num");
    if (prop === "copy") {
      code = current();
      if (!muts) code += ".clone()";
      muts = null;
    } else if (prop === "nice") build(`.nice(${count(arg(0))})`);
    else if (prop === "clamp") {
      const flag = arg(0);
      build(`.clamp(${flag.t === "bool" ? flag.c : skip("js: a clamp flag other than a boolean (truthiness)")})`);
    } else if (prop === "unknown") build(`.unknown(${needNum(arg(0), "unknown value")})`);
    else if (BUILDERS.includes(prop)) build(`.${prop}(${needNum(arg(0), prop)})`);
    else build(interpolator(args[0], r, ctx));
  }
  settled();
  if (!retyped()) return { c: current(), t: "scale", tr: root.tr, n, r, ...(muts ? { stmt: `${name} = ${name}${muts};` } : {}) };
  // Only a statement can shadow the variable with the new type.
  return {
    get c() {
      return skip(SHADOW);
    },
    t: "scale",
    tr: root.tr,
    n,
    r,
    get stmt() {
      root.r = r;
      return `let mut ${name} = ${name}${muts};`;
    },
  };
}

/** `.interpolate(f)` for d3's number interpolators over numbers and its color interpolators over colors. */
function interpolator(node, r, ctx) {
  const name = node?.type === "Identifier" && !ctx.vars.has(node.name) && node.name;
  if (r === "num" && INTERPOLATORS[name]) return `.interpolate(|a: &f64, b: &f64, t: f64| interpolate::${INTERPOLATORS[name]}(*a, *b)(t))`;
  const colors = r === "rgb" && blend(name, [{ c: "*a", t: "rgb" }, { c: "*b", t: "rgb" }]);
  if (colors) return `.interpolate(|a: &Rgb, b: &Rgb, t: f64| ${colors.c}(t))`;
  if (node?.type === "Identifier" && ctx.vars.get(node.name)?.t === "local") skip("js: a custom interpolator returning arrays");
  return skip("types: an interpolator of other values than the range's");
}

export default {
  display: ["fmt"],
  identifiers: { roundEpsilon: { c: "round_epsilon", t: "fn", arity: 1 } },
  call(node, ctx) {
    const name = calleeName(node, ctx);
    if (node.type === "NewExpression") skip(`js: new ${name} (coercion)`);
    if (node.callee.property?.name === "filter") skip("js: filter(String) keeps the non-empty labels (truthiness)");
    if (name === "scaleRadial") skip("api: d3's scaleRadial is not part of the design");
    if (MAKE[name]) {
      const [make, tr] = MAKE[name];
      const [d, r] = node.arguments;
      const one = tr === "identity" ? "domain" : "range";
      const steps = r ? [{ prop: "domain", args: [d] }, { prop: "range", args: [r] }] : d ? [{ prop: one, args: [d] }] : [];
      return chain({ c: `scale::${make}()`, t: "scale", tr }, steps, ctx);
    }
    if (name === "tickFormat") {
      const [a, b, n] = node.arguments.map((x) => val(ex(x, ctx)));
      const ends = `[${needNum(a, "start")}, ${needNum(b, "stop")}]`;
      return { c: `scale::tick_format(${EN_US}, ${ends}, ${count(n)}, ${specifier(node.arguments[3])})`, t: "result-fmt" };
    }
    const steps = [];
    let root = node;
    while (root.type === "CallExpression" && root.callee.type === "MemberExpression" && !root.callee.computed && METHODS.has(root.callee.property.name)) {
      steps.unshift({ prop: root.callee.property.name, args: root.arguments });
      root = root.callee.object;
    }
    if (!steps.length) return undefined;
    const value = val(ex(root, ctx));
    return value.t === "scale" ? chain(value, steps, ctx) : undefined;
  },
  apply(target, args) {
    if (args.length !== 1) skip(`js: ${args.length} arguments to a ${target.t} (undefined)`);
    if (target.t === "scale") return { c: `${target.c}.apply(${needNum(args[0], "scale argument")})`, t: target.r === "rgb" ? "rgb" : "num" };
    if (target.t === "tickfmt" || target.t === "fmt") return { c: `${target.c}.format(${needNum(args[0], "formatted value")})`, t: "str" };
    return undefined;
  },
  // ticks.map(f): a closure of the values first, then of f, as d3 evaluates them.
  method(obj, prop, args, ctx) {
    if (obj.t === "vec" && prop === "push") skip("js: an array changed after a scale copied it (Rust copies arrays)");
    if (obj.t !== "vec" || prop !== "map") return array.method(obj, prop, args, ctx);
    const f = args[0] ? val(args[0]) : skip("js: map without a function (undefined)");
    if (f.t === "tickfmt" || f.t === "fmt") {
      return { c: `{ let values_ = &${obj.c}; let format_ = ${f.c}; values_.iter().map(|v| format_.format(*v)).collect::<Vec<_>>() }`, t: "strs" };
    }
    if (f.t === "fn" && f.arity === 1 && (f.ret ?? "num") === "num") return { c: `${obj.c}.iter().map(|v| ${f.c}(*v)).collect::<Vec<f64>>()`, t: "vec" };
    return skip(`js: map with a ${f.t}`);
  },
  field: array.field,
};
