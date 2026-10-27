// d3-array's statistics, range, bisect, bin and threshold rules over arrays
// of numbers. An array argument is a Vec<f64> whose null and undefined
// elements are NaN, which d3 skips as it skips NaN; written as a literal,
// such an array is [Option<f64>; N] (t "maybes"), d3's undefined. An
// accessor reading one property, `(d) => d.x` or a file-level `function
// unbox(b) { return b.value; }`, is applied at translation time to object
// literals or to `[…].map(box)` with box a function returning
// `{value: value}`. Reductions are Option<f64> where d3 returns undefined.
// bin() chains are the Bin builder; calling one lays a Vec<f64> out into
// ((x0, x1), values) pairs, which d3's test helper box(values, x0, x1)
// builds too.
import { calleeName, ex, needNum, skip, snake, val } from "../expr.mjs";

const OPTIONAL = new Set(["min", "max", "mean", "median", "variance", "deviation"]);
const RULES = { thresholdSturges: "Sturges", thresholdScott: "Scott", thresholdFreedmanDiaconis: "FreedmanDiaconis" };
const BIN = new Set(["domain", "thresholds", "value"]);
const FIELDS = { x0: "b.0.0", x1: "b.0.1", length: "(b.1.len() as f64)" };
const UNDEFINED = { type: "Identifier", name: "undefined" };
const ALIAS = { c: "()", t: "alias" };

/** Whether `node` is d3's null or undefined. */
const missing = (node) => (node.type === "Literal" && node.value === null) || (node.type === "Identifier" && node.name === "undefined");

/** A number argument: absent or undefined is NaN (d3's `+x`), and null too where d3 tests `x != null`. */
function num(node, ctx, nullIsNaN = false) {
  if (!node || (missing(node) && (nullIsNaN || node.type === "Identifier"))) return "f64::NAN";
  return needNum(val(ex(node, ctx)), "argument");
}

/** A translated array as Vec<f64> code: a variable cloned so it stays usable, NaN for d3's undefined. */
function numbers(e) {
  if (e?.t === "vec") return /^\w+$/.test(e.c) ? `${e.c}.clone()` : e.c;
  if (e?.t === "maybes") return `${e.c}.iter().map(|v| v.unwrap_or(f64::NAN)).collect::<Vec<f64>>()`;
  return skip(`js: ${e?.t} values (coercion)`);
}

/** An array of numbers as Vec<f64> code, projected through `accessor` first. */
export const data = (node, ctx, accessor) => numbers(val(ex(accessor ? project(node, accessor, ctx) : node, ctx)));

/** The property an accessor reads: `(d) => d.x`, or a file-level function returning `b.value`. */
function property(fn, ctx) {
  const f = fn.type === "Identifier" && !ctx.vars.has(fn.name) ? ctx.file.helpers.get(fn.name)?.node : fn;
  const body = f?.body?.type === "BlockStatement" ? f.body.body.length === 1 && f.body.body[0].argument : f?.body;
  if (f?.params?.length !== 1 || body?.type !== "MemberExpression" || body.computed || body.object.name !== f.params[0].name) {
    skip("js: an accessor other than one property read (coercion)");
  }
  return body.property.name;
}

/** `values` with each element replaced by the property `accessor` reads. */
function project(values, accessor, ctx) {
  const prop = property(accessor, ctx);
  const read = (obj) =>
    obj?.type === "ObjectExpression"
      ? (obj.properties.find((p) => (p.key?.name ?? p.key?.value) === prop)?.value ?? UNDEFINED)
      : skip("js: an accessor over values other than object literals");
  if (values.type === "ArrayExpression") return { type: "ArrayExpression", elements: values.elements.map(read) };
  const box = values.type === "CallExpression" && values.arguments[0]?.type === "Identifier" && ctx.file.helpers.get(values.arguments[0].name)?.node;
  const made = box?.body?.body?.[0]?.argument;
  const array = values.callee?.object;
  if (values.callee?.property?.name !== "map" || array?.type !== "ArrayExpression" || made?.type !== "ObjectExpression" || read(made).name !== box.params[0]?.name) {
    skip("js: an accessor over values other than boxed numbers");
  }
  return array;
}

/** A bin() chain: the Bin builder, or undefined when `node` is none. */
function bin(node, ctx) {
  const steps = [];
  let root = node;
  while (root.type === "CallExpression" && root.callee.type === "MemberExpression" && !root.callee.computed && BIN.has(root.callee.property.name)) {
    steps.unshift(root);
    root = root.callee.object;
  }
  const variable = root.type === "Identifier" && ctx.vars.get(root.name);
  let code = variable?.t === "bin" ? val(ex(root, ctx)).c : root.type === "CallExpression" && calleeName(root, ctx) === "bin" ? "array::bin()" : null;
  if (code === null) return undefined;
  for (const step of steps) {
    const prop = step.callee.property.name;
    const [arg] = step.arguments;
    if (!arg) skip(`api: bin's ${prop}() getter`);
    if (prop === "value" || /Function/.test(arg.type)) skip(`api: a function or constant as bin's ${prop} (Thresholds is a closed set)`);
    if (prop === "domain") {
      if (arg.type !== "ArrayExpression" || arg.elements.length !== 2) skip("types: a bin domain other than two numbers");
      code += `.domain([${arg.elements.map((e) => num(e, ctx)).join(", ")}])`;
    } else {
      const e = RULES[arg.name] ? null : val(ex(arg, ctx));
      const rule = RULES[arg.name] ?? (e.t === "num" ? `Count((${e.c}) as usize)` : `Values(${data(arg, ctx)})`);
      code += `.thresholds(array::bin::Thresholds::${rule})`;
    }
  }
  return { c: code, t: "bin" };
}

export default {
  identifiers: { bisect: ALIAS, bisectRight: ALIAS, histogram: ALIAS, bin: ALIAS },
  // An object declared in a test skips only the assertions that use it; sparse arrays and file reads skip their test.
  statement(s, ctx) {
    const source = ctx.file.source.slice(s.start, s.end);
    if (/ << |await /.test(source)) skip(`js: ${source.includes("await") ? "a file read" : "a sparse array"}`);
    if (s.type === "ForOfStatement") skip("gap: a for…of loop");
    if (s.type !== "VariableDeclaration") return;
    const objects = s.declarations.filter((d) => d.init?.type === "ObjectExpression");
    for (const d of objects) ctx.vars.set(d.id.name, { get c() { return skip("js: an object compared by identity or valueOf"); }, t: "object" });
    s.declarations = s.declarations.filter((d) => !objects.includes(d));
  },
  equal: (a, b) => (a.t === "alias" || b.t === "alias" ? skip("api: d3's aliases and default accessors (one name per function)") : undefined),
  array(node, ctx) {
    const { elements } = node;
    if (elements.length && elements.every((e) => e?.type === "CallExpression" && calleeName(e, ctx) === "box" && e.arguments.length === 3)) {
      return { c: `vec![${elements.map((e) => ex(e, ctx).c).join(", ")}]`, t: "buckets" };
    }
    if (elements.length && elements.every((e) => e?.type === "ArrayExpression")) {
      return { c: `vec![${elements.map((e) => numbers(val(ex(e, ctx)))).join(", ")}]`, t: "rows" };
    }
    if (!elements.some((e) => e && missing(e))) return undefined;
    const items = elements.map((e) => (!e ? skip("js: array hole") : missing(e) ? "Option::<f64>::None" : `Some(${needNum(val(ex(e, ctx)), "array element")})`));
    return { c: `[${items.join(", ")}]`, t: "maybes" };
  },
  call(node, ctx) {
    const name = calleeName(node, ctx);
    const [a, b, c, d] = node.arguments;
    if (node.type === "NewExpression") skip(`js: new ${node.callee.name} (coercion)`);
    if (name === "quantileIndex" || name === "medianIndex") skip(`api: ${name} (not in v0.1)`);
    if (name === "box" && node.arguments.length === 3) return { c: `((${num(b, ctx)}, ${num(c, ctx)}), ${data(a, ctx)})`, t: "bucket" };
    if (name === "count") return { c: `(array::count(${data(a, ctx, b)}) as f64)`, t: "num" };
    if (name === "sum") return { c: `array::sum(${data(a, ctx, b)})`, t: "num" };
    if (OPTIONAL.has(name)) return { c: `array::${name}(${data(a, ctx, b)})`, t: "maybe" };
    if (name === "extent") return { c: `{ let e_ = array::extent(${data(a, ctx, b)}); [e_.map(|e| e[0]), e_.map(|e| e[1])] }`, t: "maybes" };
    if (name === "quantile") return { c: `array::quantile(${data(a, ctx, c)}, ${num(b, ctx)})`, t: "maybe" };
    if (name === "quantileSorted") return { c: `array::quantile_sorted(&${data(a, ctx)}, ${num(b, ctx)})`, t: "maybe" };
    if (name === "range") {
      const args = node.arguments.length < 2 ? ["0.0", num(a, ctx), "1.0"] : [num(a, ctx), num(b, ctx), c ? num(c, ctx) : "1.0"];
      return { c: `array::range(${args.join(", ")})`, t: "vec" };
    }
    if (name === "bisectLeft" || name === "bisectRight") {
      const [lo, hi] = [c ? `(${num(c, ctx)}) as usize` : "0", d ? `(${num(d, ctx)}) as usize` : "v_.len()"];
      return { c: `{ let v_ = ${data(a, ctx)}; let lo_ = ${lo}; (lo_ + array::${snake(name)}(&v_[lo_..${hi}], ${num(b, ctx, true)})) as f64 }`, t: "num" };
    }
    if (RULES[name]) {
      const ends = name === "thresholdSturges" ? "" : `, ${num(b, ctx)}, ${num(c, ctx)}`;
      return { c: `(array::bin::${snake(RULES[name])}(&${data(a, ctx)}${ends}) as f64)`, t: "num" };
    }
    // bins.map(b => [b.x0, b.x1]) and bins.map(b => b.length)
    const f = node.callee.property?.name === "map" && a;
    const parts = f?.type === "ArrowFunctionExpression" && (f.body.type === "ArrayExpression" ? f.body.elements : [f.body]);
    if (parts && parts.every((p) => p?.type === "MemberExpression" && p.object.name === f.params[0]?.name && FIELDS[p.property.name])) {
      const bins = val(ex(node.callee.object, ctx));
      const fields = parts.map((p) => FIELDS[p.property.name]);
      const rows = f.body.type === "ArrayExpression";
      if (bins.t === "buckets") return { c: `${bins.c}.iter().map(|b| ${rows ? `vec![${fields.join(", ")}]` : fields[0]}).collect::<Vec<_>>()`, t: rows ? "rows" : "vec" };
    }
    return bin(node, ctx);
  },
  apply(target, args) {
    if (target.t !== "bin") return undefined;
    const values = numbers(args[0]);
    const bucket = "((b.x0, b.x1), b.indices.iter().map(|&i| d_[i]).collect::<Vec<f64>>())";
    return { c: `{ let d_: Vec<f64> = ${values}; ${target.c}.layout(&d_).iter().map(|b| ${bucket}).collect::<Vec<_>>() }`, t: "buckets" };
  },
};
