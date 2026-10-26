// Translates d3 test files (mocha + node:assert) into Rust test functions for the PoC harness.
// Every assertion becomes `check(id, kind, actual, expected)`; constructs that only test
// JavaScript semantics (coercion, function identity, instanceof, null) become `skip(id, reason)`.
import * as acorn from "./vendor/acorn/acorn.mjs";
import { readFileSync, writeFileSync } from "node:fs";
import { basename } from "node:path";

class Skip extends Error {}
const skip = (reason) => { throw new Skip(reason); };

const EASE = {
  easeLinear: "Linear",
  easeQuad: "QuadInOut", easeQuadIn: "QuadIn", easeQuadOut: "QuadOut", easeQuadInOut: "QuadInOut",
  easeCubic: "CubicInOut", easeCubicIn: "CubicIn", easeCubicOut: "CubicOut", easeCubicInOut: "CubicInOut",
  easePoly: "PolyInOut", easePolyIn: "PolyIn", easePolyOut: "PolyOut", easePolyInOut: "PolyInOut",
  easeSin: "SinInOut", easeSinIn: "SinIn", easeSinOut: "SinOut", easeSinInOut: "SinInOut",
  easeExp: "ExpInOut", easeExpIn: "ExpIn", easeExpOut: "ExpOut", easeExpInOut: "ExpInOut",
  easeCircle: "CircleInOut", easeCircleIn: "CircleIn", easeCircleOut: "CircleOut", easeCircleInOut: "CircleInOut",
  easeBounce: "BounceOut", easeBounceIn: "BounceIn", easeBounceOut: "BounceOut", easeBounceInOut: "BounceInOut",
  easeBack: "BackInOut", easeBackIn: "BackIn", easeBackOut: "BackOut", easeBackInOut: "BackInOut",
  easeElastic: "ElasticOut", easeElasticIn: "ElasticIn", easeElasticOut: "ElasticOut", easeElasticInOut: "ElasticInOut",
};
const PATH_METHODS = {
  moveTo: ["move_to", "unit"], lineTo: ["line_to", "unit"], closePath: ["close_path", "unit"],
  quadraticCurveTo: ["quadratic_curve_to", "unit"], bezierCurveTo: ["bezier_curve_to", "unit"],
  rect: ["rect", "unit"], arc: ["arc", "result"], arcTo: ["arc_to", "result"],
};
const ARRAY_FNS = { ticks: ["ticks", "vec"], tickIncrement: ["tick_increment", "num"], tickStep: ["tick_step", "num"], nice: ["nice", "vec"] };
const MATH_FNS = { sqrt: "sqrt", abs: "abs", floor: "floor", ceil: "ceil", sin: "sin", cos: "cos", exp: "exp", log: "ln" };
const MATH_CONSTS = { PI: "PI", E: "E", SQRT2: "SQRT_2" };

const snake = (s) => s.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase();
const num = (raw) => {
  let r = raw.replace(/_/g, "");
  if (r.startsWith(".")) r = "0" + r;
  if (!/[.eE]/.test(r)) r += ".0";
  return r;
};
const isNum = (e) => e.t === "num";
const needNum = (e, what) => { if (!isNum(e)) skip(`${what} is not a number (JS coercion)`); return e.c; };

function ex(node, ctx) {
  switch (node.type) {
    case "Literal":
      if (typeof node.value === "number") return { c: num(/^0[xob]/i.test(node.raw) ? String(node.value) : node.raw), t: "num" };
      if (typeof node.value === "string") return { c: JSON.stringify(node.value), t: "str" };
      if (typeof node.value === "boolean") return { c: String(node.value), t: "bool" };
      if (typeof node.value === "bigint") skip("BigInt argument (JS-only type)");
      if (node.regex) skip("regex literal");
      return skip("null literal (JS coercion)");
    case "Identifier": {
      if (node.name === "NaN") return { c: "f64::NAN", t: "num" };
      if (node.name === "Infinity") return { c: "f64::INFINITY", t: "num" };
      if (node.name === "undefined") return { c: "Option::<f64>::None", t: "undef" };
      if (ctx.vars.has(node.name)) return ctx.vars.get(node.name);
      if (EASE[node.name]) return { c: `ease::${EASE[node.name]}::default()`, t: "ease", ident: node.name };
      return skip(`unsupported identifier ${node.name}`);
    }
    case "MemberExpression":
      if (node.object.name === "Math" && MATH_CONSTS[node.property.name]) return { c: MATH_CONSTS[node.property.name], t: "num" };
      {
        const o = val(ex(node.object, ctx)), f = node.property.name;
        if (o.t === "spec") {
          if (["fill", "align", "sign"].includes(f)) return { c: `${o.c}.${f}.to_string()`, t: "str" };
          if (f === "symbol") return { c: `${o.c}.symbol.map(String::from).unwrap_or_default()`, t: "str" };
          if (f === "type") return { c: `${o.c}.kind.map(String::from).unwrap_or_default()`, t: "str" };
          if (["zero", "comma", "trim"].includes(f)) return { c: `${o.c}.${f}`, t: "bool" };
          if (["width", "precision"].includes(f)) return { c: `${o.c}.${f}.map(|w| w as f64)`, t: "optnum" };
        }
      }
      return skip(`unsupported member ${node.property.name}`);
    case "UnaryExpression": {
      const a = ex(node.argument, ctx);
      if (node.operator === "-") return { c: `-${needNum(a, "negation")}`, t: "num" };
      if (node.operator === "+" && isNum(a)) return a;
      return skip(`unary ${node.operator} (JS coercion)`);
    }
    case "BinaryExpression": {
      const a = ex(node.left, ctx), b = ex(node.right, ctx);
      if (node.operator === "+" && ["path", "str", "fmt", "spec", "result-fmt", "result-spec"].includes(a.t) && b.c === '""') return { c: `${val(a).c}.to_string()`, t: "str" };
      if (["+", "-", "*", "/", "%"].includes(node.operator) && isNum(a) && isNum(b)) return { c: `(${a.c} ${node.operator} ${b.c})`, t: "num" };
      if (["===", "<", ">", "<=", ">="].includes(node.operator) && isNum(a) && isNum(b)) return { c: `(${a.c} ${node.operator === "===" ? "==" : node.operator} ${b.c})`, t: "bool" };
      return skip(`binary ${node.operator} on ${a.t}/${b.t}`);
    }
    case "ArrayExpression": {
      const els = node.elements.map((e) => needNum(ex(e, ctx), "array element"));
      return { c: els.length ? `vec![${els.join(", ")}]` : "Vec::<f64>::new()", t: "vec" };
    }
    case "ObjectExpression": return skip("object argument (valueOf coercion)");
    case "CallExpression": return call(node, ctx);
    default: return skip(`unsupported ${node.type}`);
  }
}

function call(node, ctx) {
  const callee = node.callee;
  if (callee.type === "Identifier" && callee.name === "formatLocale") return { c: localeLiteral(node.arguments[0]), t: "locale" };
  const args = node.arguments.map((a) => ex(a, ctx));
  if (callee.type === "Identifier") {
    const name = callee.name;
    if (name === "path") return { c: "Path::new()", t: "path" };
    if (name === "pathRound") {
      if (!args.length) return { c: "Path::with_digits(3.0).unwrap()", t: "path" };
      return { c: `Path::with_digits(${needNum(args[0], "digits")})`, t: "result-path" };
    }
    if (ARRAY_FNS[name]) {
      const [fn, t] = ARRAY_FNS[name];
      if (args.length !== 3) skip(`${name} with ${args.length} arguments`);
      const c = `${fn}(${args.map((a) => needNum(a, `${name} argument`)).join(", ")})`;
      return { c: t === "vec" && name === "nice" ? `${c}.to_vec()` : c, t };
    }
    if (name === "format") return { c: `format(${needStr(args[0])})`, t: "result-fmt" };
    if (name === "formatPrefix") return { c: `format_prefix(${needStr(args[0])}, ${needNum(args[1], "formatPrefix value")})`, t: "result-fmt" };
    if (name === "formatSpecifier") return { c: `${needStr(args[0])}.parse::<Specifier>()`, t: "result-spec" };
    if (name === "formatLocale") return { c: localeLiteral(node.arguments[0]), t: "locale" };
    if (name === "precisionFixed") return { c: `precision_fixed(${needNum(args[0], "step")})`, t: "num" };
    if (name === "precisionPrefix") return { c: `precision_prefix(${needNum(args[0], "step")}, ${needNum(args[1], "value")})`, t: "num" };
    if (name === "precisionRound") return { c: `precision_round(${needNum(args[0], "step")}, ${needNum(args[1], "max")})`, t: "num" };
    if (name === "isNaN") return { c: `${needNum(args[0], "isNaN")}.is_nan()`, t: "bool" };
    if (name === "precision") return { c: `precision(&${args[0].c}, ${needNum(args[1], "precision")})`, t: "str" };
    if (name === "out" || name === "inOut") {
      if (args[0].t !== "ease") skip(`${name} of non-ease`);
      return { c: `${name === "out" ? "out" : "in_out"}(${args[0].c})`, t: "fn" };
    }
    const target = ex(callee, ctx);
    return applyCallable(target, args);
  }
  if (callee.type === "MemberExpression") {
    const prop = callee.property.name;
    if (callee.object.type === "Identifier" && callee.object.name === "Math") {
      if (prop === "pow") return { c: `${needNum(args[0], "pow")}.powf(${needNum(args[1], "pow")})`, t: "num" };
      if (MATH_FNS[prop]) return { c: `(${needNum(args[0], prop)}).${MATH_FNS[prop]}()`, t: "num" };
      skip(`Math.${prop}`);
    }
    const obj = val(ex(callee.object, ctx));
    if (obj.t === "locale" && prop === "format") return { c: `${obj.c}.format(${needStr(args[0])})`, t: "result-fmt" };
    if (obj.t === "locale" && prop === "formatPrefix") return { c: `${obj.c}.format_prefix(${needStr(args[0])}, ${needNum(args[1], "value")})`, t: "result-fmt" };
    if ((obj.t === "fmt" || obj.t === "spec") && prop === "toString") return { c: `${obj.c}.to_string()`, t: "str" };
    if (obj.t === "path") {
      if (prop === "toString") return { c: `${obj.c}.to_string()`, t: "str" };
      const m = PATH_METHODS[prop];
      if (!m) skip(`path.${prop}`);
      let a = args.map((x, i) => {
        if (prop === "arc" && i === 5) {
          if (x.t === "bool") return x.c;
          if (x.t === "num" && /^\d/.test(x.c)) return String(parseFloat(x.c) !== 0);
          skip("non-boolean ccw (JS truthiness)");
        }
        return needNum(x, `path.${prop} argument`);
      });
      if (prop === "arc" && a.length === 5) a.push("false");
      return { c: `${obj.c}.${m[0]}(${a.join(", ")})`, t: m[1] };
    }
    if (obj.t === "ease" && ["exponent", "overshoot", "amplitude", "period"].includes(prop)) {
      return { c: `${obj.c}.${prop}(${needNum(args[0], prop)})`, t: "ease" };
    }
    if (obj.t === "vec" && prop === "pop") return { c: `${obj.c}.pop().unwrap()`, t: "num" };
    if (obj.t === "vec" && prop === "reverse") return { c: `{ let mut v = ${obj.c}; v.reverse(); v }`, t: "vec" };
    return skip(`method ${prop} on ${obj.t}`);
  }
  if (callee.type === "CallExpression") return applyCallable(call(callee, ctx), args);
  return skip(`callee ${callee.type}`);
}

function applyCallable(target, args) {
  target = val(target);
  if (target.t === "fmt") return { c: `${target.c}.format(${needNum(args[0], "formatted value")})`, t: "str" };
  if (target.t === "ease") return { c: `${target.c}.ease(${needNum(args[0], "ease argument")})`, t: "num" };
  if (target.t === "fn") return { c: `${target.c}(${needNum(args[0], "argument")})`, t: "num" };
  return skip(`calling a ${target.t}`);
}

const val = (e) => (e.t && e.t.startsWith("result-") ? { c: `${e.c}.unwrap()`, t: e.t.slice(7) } : e);
const needStr = (e) => { if (e.t !== "str") skip(`${e.t} specifier (JS coercion)`); return e.c; };
function localeLiteral(node) {
  if (!node || node.type !== "ObjectExpression") skip("gap: locale definition loaded at runtime");
  const fields = node.properties.map((p) => {
    const k = p.key.name ?? p.key.value, v = p.value;
    const str = (x) => { if (x.type !== "Literal" || typeof x.value !== "string") skip("gap: non-literal locale field"); return `${JSON.stringify(x.value)}.into()`; };
    if (["decimal", "thousands", "percent", "minus", "nan"].includes(k)) return `${k}: Some(${str(v)})`;
    if (k === "grouping") return `grouping: Some(vec![${v.elements.map((e) => e.value + "usize").join(", ")}])`;
    if (k === "currency") return `currency: Some([${v.elements.map(str).join(", ")}])`;
    if (k === "numerals") return `numerals: Some(vec![${v.elements.map(str).join(", ")}])`;
    return skip(`gap: locale field ${k}`);
  });
  return `Locale::new(Definition { ${[...fields, "..Default::default()"].join(", ")} })`;
}
const asObs = (e) => { e = val(e); return ["path", "fmt", "spec"].includes(e.t) ? `${e.c}.to_string()` : e.c; };

let nextId = 0;
const asserts = [];

function assertionKind(expr) {
  if (expr.type !== "CallExpression") return null;
  const c = expr.callee;
  if (c.type === "Identifier" && ["assertInDelta", "assertPathEqual", "assert"].includes(c.name)) return c.name;
  if (c.type === "MemberExpression" && c.object.name === "assert") return `assert.${c.property.name}`;
  return null;
}

function countAssertions(node) {
  let n = 0;
  (function walk(x) {
    if (!x || typeof x !== "object") return;
    if (x.type === "ExpressionStatement" && assertionKind(x.expression)) n++;
    for (const k in x) if (k !== "loc") walk(x[k]);
  })(node);
  return n;
}

function statement(s, ctx, out) {
  if (s.type === "VariableDeclaration") {
    for (const d of s.declarations) {
      const e = ex(d.init, ctx);
      const name = snake(d.id.name);
      if (e.t.startsWith("result-")) {
        out.push(`let mut ${name} = ${e.c}.unwrap();`);
        ctx.vars.set(d.id.name, { c: name, t: e.t.slice(7) });
      } else {
        out.push(`let mut ${name} = ${e.c};`);
        ctx.vars.set(d.id.name, { c: name, t: e.t });
      }
    }
    return;
  }
  if (s.type !== "ExpressionStatement") skip(`statement ${s.type}`);
  if (s.expression.type === "SequenceExpression") {
    for (const e of s.expression.expressions) statement({ type: "ExpressionStatement", expression: e, loc: s.loc }, ctx, out);
    return;
  }
  const kind = assertionKind(s.expression);
  if (!kind) {
    const e = ex(s.expression, ctx);
    out.push(e.t === "result" ? `${e.c}.unwrap();` : `${e.c};`);
    return;
  }
  const id = nextId++;
  const line = s.loc.start.line;
  try {
    const a = s.expression.arguments;
    let code;
    if (kind === "assert.strictEqual" || kind === "assert.deepStrictEqual" || kind === "assert.equal") {
      const x = ex(a[0], ctx), y = ex(a[1], ctx);
      if (x.t === "ease" || y.t === "ease" || x.t === "fn") skip("function identity");
      code = `check(${id}, Kind::Strict, &${asObs(x)}, &${asObs(y)});`;
    } else if (kind === "assert.isNaN") {
      code = `check(${id}, Kind::Strict, &${needNum(ex(a[0], ctx), "isNaN")}.is_nan(), &true);`;
    } else if (kind === "assertInDelta") {
      const x = ex(a[0], ctx), y = ex(a[1], ctx);
      const d = a[2] ? needNum(ex(a[2], ctx), "delta") : "1e-6";
      code = `check(${id}, Kind::InDelta(${d}), &${asObs(x)}, &${asObs(y)});`;
    } else if (kind === "assertPathEqual") {
      const x = ex(a[0], ctx), y = ex(a[1], ctx);
      code = `check(${id}, Kind::PathEq, &${asObs(x)}, &${asObs(y)});`;
    } else if (kind === "assert.throws") {
      const fn = a[0];
      let body = fn.body;
      if (body.type === "BlockStatement") {
        if (body.body.length !== 1 || body.body[0].type !== "ExpressionStatement") skip("throws with a multi-statement body");
        body = body.body[0].expression;
      }
      const e = ex(body, ctx);
      if (e.t !== "result" && !e.t.startsWith("result-")) skip("gap: throws on an infallible call");
      code = `check(${id}, Kind::Strict, &(${e.c}).is_err(), &true);`;
    } else if (kind === "assert") {
      if (a[0].type === "BinaryExpression" && a[0].operator === "instanceof") skip("instanceof");
      const e = ex(a[0], ctx);
      if (e.t !== "bool") skip("truthiness assertion");
      code = `check(${id}, Kind::Strict, &${e.c}, &true);`;
    } else {
      skip(`assertion ${kind}`);
    }
    out.push(code);
    asserts.push({ id, status: "run", reason: "", line, test: ctx.test, module: ctx.module });
  } catch (err) {
    if (!(err instanceof Skip)) throw err;
    out.push(`skip(${id}, ${JSON.stringify(err.message)});`);
    asserts.push({ id, status: "skip", reason: err.message, line, test: ctx.test, module: ctx.module });
  }
}

function translateFile(file, module) {
  const ast = acorn.parse(readFileSync(file, "utf8"), { ecmaVersion: "latest", sourceType: "module", locations: true });
  const fns = [];
  for (const node of ast.body) {
    if (node.type !== "ExpressionStatement" || node.expression.type !== "CallExpression" || !["it", "test"].includes(node.expression.callee.name)) continue;
    const [nameNode, fnNode] = node.expression.arguments;
    const test = `${basename(file)}: ${nameNode.value}`;
    const ctx = { vars: new Map(), test, module };
    const body = [];
    const firstId = nextId;
    try {
      for (const s of fnNode.body.body) statement(s, ctx, body);
    } catch (err) {
      if (!(err instanceof Skip)) throw err;
      // Setup outside any assertion uses JS-only semantics: skip every assertion in this test.
      nextId = firstId;
      asserts.splice(asserts.findIndex((x) => x.id >= firstId) >>> 0);
      body.length = 0;
      const n = countAssertions(fnNode.body);
      for (let k = 0; k < n; k++) {
        const id = nextId++;
        body.push(`skip(${id}, ${JSON.stringify("setup: " + err.message)});`);
        asserts.push({ id, status: "skip", reason: "setup: " + err.message, line: node.loc.start.line, test, module });
      }
    }
    fns.push({ test, body });
  }
  return fns;
}

const root = process.argv[2];
const files = {
  path: ["d3-path/test/path-test.js", "d3-path/test/pathRound-test.js"],
  ease: ["back", "bounce", "circle", "cubic", "elastic", "exp", "linear", "poly", "quad", "sin"].map((n) => `d3-ease/test/${n}-test.js`),
  ticks: ["ticks", "tickIncrement", "tickStep", "nice"].map((n) => `d3-array/test/${n}-test.js`),
  format: ["arabicLocale", "defaultLocale", "format", "format-trim", "format-type-%", "format-type-b", "format-type-c", "format-type-d", "format-type-e", "format-type-f", "format-type-g", "format-type-n", "format-type-none", "format-type-o", "format-type-p", "format-type-r", "format-type-s", "format-type-x", "formatPrefix", "formatSpecifier", "locale", "precisionFixed", "precisionPrefix", "precisionRound"].map((n) => `d3-format/test/${n}-test.js`),
};
let rust = `// Generated by translate.mjs from d3's own test files. Do not edit.\n#![allow(unused_mut, unused_parens, clippy::all)]\nuse crate::prelude::*;\n\n`;
const registry = [];
let fnIndex = 0;
for (const [module, list] of Object.entries(files)) {
  for (const f of list) {
    for (const { test, body } of translateFile(`${root}/${f}`, module)) {
      const fname = `t${fnIndex++}`;
      rust += `// ${test}\nfn ${fname}() {\n${body.map((l) => "    " + l).join("\n")}\n}\n\n`;
      registry.push(`    (${JSON.stringify(module)}, ${JSON.stringify(test)}, ${fname}),`);
    }
  }
}
rust += `pub const TESTS: &[(&str, &str, fn())] = &[\n${registry.join("\n")}\n];\n\n`;
rust += `pub const ASSERTS: &[(usize, &str, &str, &str, &str)] = &[\n${asserts.map((a) => `    (${a.id}, ${JSON.stringify(a.module)}, ${JSON.stringify(a.test)}, ${JSON.stringify(a.status)}, ${JSON.stringify(a.reason)}),`).join("\n")}\n];\n`;
writeFileSync(process.argv[3], rust);
const by = (m) => asserts.filter((a) => a.module === m);
for (const m of Object.keys(files)) {
  const xs = by(m);
  console.log(`${m}: ${xs.length} assertions, ${xs.filter((a) => a.status === "run").length} translated, ${xs.filter((a) => a.status === "skip").length} skipped`);
}
const isJs = (r) => /coercion|identity|instanceof|truthiness|null|undefined|valueOf|JS-only/.test(r) && !/gap:/.test(r);
for (const m of Object.keys(files)) {
  const xs = by(m).filter((a) => a.status === "skip");
  console.log(`  ${m}: js-only ${xs.filter((a) => isJs(a.reason)).length}, gaps ${xs.filter((a) => !isJs(a.reason)).length}`);
}
const reasons = {};
for (const a of asserts.filter((a) => a.status === "skip")) reasons[a.reason] = (reasons[a.reason] || 0) + 1;
console.log(reasons);
