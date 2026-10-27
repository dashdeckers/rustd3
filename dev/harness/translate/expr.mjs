// The generic translation of one d3 test file (mocha + node:assert) into
// Rust: typed values, literals, arithmetic, arrays, Math.*, variables,
// statements, sequence expressions, the assertion kinds, setup skips and
// file-level helpers. d3's own names come from the suite's rules.
//
// A translated expression is a typed value {c, t}: c is Rust code, t its
// type: num (f64), str (&str or String; a literal keeps its text as `raw`), bool, vec (Vec<f64>; a literal
// keeps its element code as `items`), strs (a Vec of strings), undef
// (None), opt (an Option of a number whose None is d3's NaN), fn (a
// callable taking `arity` f64s and returning `ret`, num by default),
// result (Result<(), E>), result-<t> (Result<t, E>), local (a function
// declared in a test, inlined where a statement calls it), or a type a
// suite's rules introduce. A fallible value may name its `expect` message;
// a value with `stmt` is written as that statement when it stands alone.
//
// Rules (rules/<d3 module>.mjs) may provide:
//   identifiers                    {name: typed value} for d3's exports
//   call(node, ctx)                a call or `new` of d3's, from its AST
//   method(obj, prop, args, ctx)   a typed value, or undefined (a gap)
//   field(obj, prop, ctx, node)    a property read (`[…]` when computed)
//   apply(target, args, ctx)       calling a value that is not a fn
//   equal(a, b)                    strictEqual of two such values, as bool code
//   array(node, ctx)               an array literal as a type of the rules'
//   statement(s, ctx)              a statement's skip reason, by throwing it
//   display                        [t]: types observed through Display
//   assertions                     {name: (args, id, ctx) => check code} for
//                                  a test file's own assertion helpers
//   assign(node, ctx)              a write to a field (`=`, `+=`, `++`), or
//                                  undefined where fields are read-only
//   finish(line)                   each line of a translated test, rewritten
//                                  once the whole test is read
//   nullable                       [t]: Options whose None is d3's null
// combine(...rules) merges rule sets: the first that answers a hook wins.
//
// `assert.strictEqual(x, null)` checks a fallible x for Err, d3's null, and
// a nullable one for None.
//
// Every skip reason starts with the category the report groups by: js:
// (JavaScript semantics), types: (ruled out by the Rust types), api: (left
// out of the Rust API by design), gap: (not ported or translated yet;
// names the phase when one is planned), helpers: (a file-level helper the
// translator cannot express).
import * as acorn from "./vendor/acorn/acorn.mjs";

/** Abandons the assertion, or the test's setup, being translated. */
export class Skip extends Error {}
/** Skips with `reason`; a skipped assertion still runs `effect`, a statement d3 runs inside it (a setter). */
export const skip = (reason, effect) => {
  throw Object.assign(new Skip(reason), { effect });
};
export const CATEGORIES = ["js", "types", "api", "gap", "helpers"];

/** One rule set from several: lists and maps merged, each hook answered by the first rules that answer it. */
export function combine(...sets) {
  const first = (hook) => (...a) => {
    for (const r of sets) {
      const out = r[hook]?.(...a);
      if (out !== undefined) return out;
    }
    return undefined;
  };
  const hooks = ["call", "method", "field", "apply", "equal", "statement", "assign", "array"];
  return {
    ...Object.fromEntries(hooks.map((h) => [h, first(h)])),
    identifiers: Object.assign({}, ...sets.map((r) => r.identifiers)),
    assertions: Object.assign({}, ...sets.map((r) => r.assertions)),
    display: sets.flatMap((r) => r.display ?? []),
    nullable: sets.flatMap((r) => r.nullable ?? []),
    finish: (line) => sets.reduce((l, r) => r.finish?.(l) ?? l, line),
  };
}

const MATH_FNS = { sqrt: "sqrt", abs: "abs", floor: "floor", ceil: "ceil", sin: "sin", cos: "cos", exp: "exp", log: "ln" };
const MATH_CONSTS = {
  PI: "std::f64::consts::PI",
  E: "std::f64::consts::E",
  SQRT2: "std::f64::consts::SQRT_2",
  SQRT1_2: "std::f64::consts::FRAC_1_SQRT_2",
};
const NUMBER_CONSTS = { MIN_VALUE: "f64::from_bits(1)", MAX_VALUE: "f64::MAX", EPSILON: "f64::EPSILON" };
const ARITHMETIC = new Set(["+", "-", "*", "/", "%"]);
const COMPARE = { "===": "==", "<": "<", ">": ">", "<=": "<=", ">=": ">=" };
const ASSERTIONS = new Set(["assert", "assertInDelta", "assertPathEqual"]);
const RUST_TYPES = { num: "f64", bool: "bool", vec: "Vec<f64>" };
const PLAIN = new Set(["num", "str", "bool", "vec"]);

export const snake = (s) => s.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase();

/** A Rust string literal holding `s`. */
export function rustStr(s) {
  const escape = { '"': '\\"', "\\": "\\\\", "\n": "\\n", "\r": "\\r", "\t": "\\t" };
  const char = (ch) => {
    const code = ch.codePointAt(0);
    if (escape[ch]) return escape[ch];
    return code < 0x20 || code === 0x7f ? `\\u{${code.toString(16)}}` : ch;
  };
  return `"${[...s].map(char).join("")}"`;
}

const number = (raw) => {
  let r = raw.replace(/_/g, "");
  if (r.startsWith(".")) r = "0" + r;
  return /[.eE]/.test(r) ? r : r + ".0";
};
const nth = (list, i) => list[i] ?? skip("js: missing argument (undefined)");
export const needNum = (e, what) => (e.t === "num" ? e.c : skip(`js: ${what} is not a number (coercion)`));
const fallible = (t) => t === "result" || t.startsWith("result-");
const unwrap = (e) => `${e.c}.expect(${rustStr(e.expect ?? "d3 test")})`;
/** The value of a result-<t>; other values pass through. */
export const val = (e) => (e.t.startsWith("result-") ? { c: unwrap(e), t: e.t.slice(7) } : e);
/** The name a call's callee refers to, when it is not a local variable. */
export const calleeName = (node, ctx) =>
  node.callee.type === "Identifier" && !ctx.vars.has(node.callee.name) ? node.callee.name : null;

export function ex(node, ctx) {
  switch (node.type) {
    case "Literal":
      return literal(node);
    case "Identifier":
      return identifier(node.name, ctx);
    case "MemberExpression": {
      const name = node.computed ? "[…]" : node.property.name;
      const consts = { Math: MATH_CONSTS, Number: NUMBER_CONSTS }[node.object.name];
      if (node.object.type === "Identifier" && consts?.[name]) return { c: consts[name], t: "num" };
      const obj = val(ex(node.object, ctx));
      return ctx.rules.field?.(obj, name, ctx, node) ?? skip(`gap: property ${name} of ${obj.t}`);
    }
    case "UpdateExpression":
      return ctx.rules.assign?.(node, ctx) ?? skip(`gap: ${node.type}`);
    case "AssignmentExpression": {
      const assigned = ctx.rules.assign?.(node, ctx);
      if (assigned) return assigned;
      if (node.left.type !== "MemberExpression") return skip(`gap: assignment`);
      // Later uses of the object see the write, which Rust's read-only fields cannot make.
      if (node.left.object.type === "Identifier") ctx.written.add(node.left.object.name);
      return skip("types: a field write (fields are read-only)");
    }
    case "SequenceExpression":
      node.expressions.forEach((e) => ex(e, ctx));
      return skip("gap: a sequence expression");
    case "UnaryExpression": {
      const a = ex(node.argument, ctx);
      if (node.operator === "-") return { c: `-${needNum(a, "negation")}`, t: "num" };
      if (node.operator === "+" && a.t === "num") return a;
      return skip(`js: unary ${node.operator} (coercion)`);
    }
    case "BinaryExpression":
      return binary(node, ctx);
    case "ArrayExpression": {
      const typed = ctx.rules.array?.(node, ctx);
      if (typed) return typed;
      const values = node.elements.map((e) => (e ? ex(e, ctx) : skip("js: array hole")));
      if (values.length && values.every((v) => v.t === "str")) return { c: `vec![${values.map((v) => v.c).join(", ")}]`, t: "strs" };
      const items = values.map((v) => needNum(v, "array element"));
      return { c: items.length ? `vec![${items.join(", ")}]` : "Vec::<f64>::new()", t: "vec", items };
    }
    case "ObjectExpression":
      return skip("js: object argument (valueOf coercion)");
    case "CallExpression":
    case "NewExpression":
      return call(node, ctx);
    case "ArrowFunctionExpression":
    case "FunctionExpression":
      return closure(node, ctx);
    default:
      return skip(`gap: ${node.type}`);
  }
}

/** A function of one number whose body is an expression or one `return`: a Rust closure over f64. */
function closure(node, ctx) {
  const [param, ...rest] = node.params;
  const [only, ...more] = node.body.body ?? [{ type: "ReturnStatement", argument: node.body }];
  if (rest.length || more.length || param?.type !== "Identifier" || only?.type !== "ReturnStatement" || node.async || node.generator) {
    skip("gap: a function other than of one number returning an expression");
  }
  const vars = ctx.vars;
  ctx.vars = new Map(vars).set(param.name, { c: snake(param.name), t: "num" });
  try {
    return { c: `(|${snake(param.name)}: f64| ${needNum(val(ex(only.argument, ctx)), "a function's result")})`, t: "fn", arity: 1 };
  } finally {
    ctx.vars = vars;
  }
}

function literal(node) {
  if (typeof node.value === "number") {
    return { c: number(/^0[xob]/i.test(node.raw) ? String(node.value) : node.raw), t: "num" };
  }
  if (typeof node.value === "string") {
    if (/\p{Surrogate}/u.test(node.value)) skip("js: lone surrogate in a string");
    return { c: rustStr(node.value), t: "str", raw: node.value };
  }
  if (typeof node.value === "boolean") return { c: String(node.value), t: "bool" };
  if (typeof node.value === "bigint") return skip("js: BigInt");
  if (node.regex) return skip("js: regular expression");
  return skip("js: null (coercion)");
}

function identifier(name, ctx) {
  if (name === "NaN") return { c: "f64::NAN", t: "num" };
  if (name === "Infinity") return { c: "f64::INFINITY", t: "num" };
  if (name === "undefined") return { c: "Option::<f64>::None", t: "undef" };
  if (ctx.written.has(name)) skip("types: a value whose fields were written");
  const local = ctx.vars.get(name);
  if (local) {
    ctx.used.add(local.c);
    return local;
  }
  const mapped = ctx.rules.identifiers?.[name];
  if (mapped) return mapped;
  if (ctx.file.helpers.has(name)) return helper(name, ctx);
  return skip(`gap: identifier ${name}`);
}

function binary(node, ctx) {
  const op = node.operator;
  if (op === "instanceof") return skip("js: instanceof");
  const a = ex(node.left, ctx);
  const b = ex(node.right, ctx);
  const shown = val(a);
  if (op === "+" && b.c === '""' && (shown.t === "str" || ctx.rules.display?.includes(shown.t))) {
    return { c: `${shown.c}.to_string()`, t: "str" };
  }
  if (a.t !== "num" || b.t !== "num") return skip(`js: ${op} on ${a.t} and ${b.t} (coercion)`);
  if (ARITHMETIC.has(op)) return { c: `(${a.c} ${op} ${b.c})`, t: "num" };
  if (COMPARE[op]) return { c: `(${a.c} ${COMPARE[op]} ${b.c})`, t: "bool" };
  return skip(`gap: operator ${op}`);
}

function call(node, ctx) {
  const custom = ctx.rules.call?.(node, ctx);
  if (custom) return custom;
  if (node.type === "NewExpression") return skip("gap: new");
  const { callee } = node;
  const args = node.arguments.map((a) => ex(a, ctx));
  if (callee.type === "Identifier" && callee.name === "isNaN") return { c: nan(nth(args, 0)), t: "bool" };
  if (callee.type === "MemberExpression" && !callee.computed) {
    const prop = callee.property.name;
    if (callee.object.type === "Identifier" && callee.object.name === "Math") return math(prop, args);
    const obj = val(ex(callee.object, ctx));
    return ctx.rules.method?.(obj, prop, args, ctx) ?? skip(`gap: method ${prop} on ${obj.t}`);
  }
  const target = val(ex(callee, ctx));
  if (target.t !== "fn") return ctx.rules.apply?.(target, args, ctx) ?? skip(`gap: calling a ${target.t}`);
  if (args.length !== target.arity) skip(`js: ${args.length} arguments for ${target.arity} (undefined)`);
  return { c: `${target.c}(${args.map((a) => needNum(a, "argument")).join(", ")})`, t: target.ret ?? "num" };
}

/** `isNaN(e)`: None is an opt's NaN. */
const nan = (e) => (e.t === "opt" ? `${e.c}.is_none()` : `(${needNum(e, "isNaN argument")}).is_nan()`);

function math(prop, args) {
  // JavaScript's Math.round takes ties toward +∞, f64::round away from 0.
  if (prop === "round") return { c: `js_round(${needNum(nth(args, 0), "Math.round argument")})`, t: "num" };
  if (prop === "pow") {
    return { c: `(${needNum(nth(args, 0), "Math.pow base")}).powf(${needNum(nth(args, 1), "Math.pow exponent")})`, t: "num" };
  }
  if (MATH_FNS[prop]) return { c: `f64::${MATH_FNS[prop]}(${needNum(nth(args, 0), `Math.${prop} argument`)})`, t: "num" };
  return skip(`gap: Math.${prop}`);
}

// File-level helpers: a top-level `const` becomes a `let` in each test that
// uses it; a top-level `function f(params) { return expr; }` becomes an `fn`
// of f64 parameters in the generated module. Either is emitted only when
// translated code uses it; one the translator cannot express makes every
// assertion that uses it a `helpers:` skip.
function helper(name, ctx) {
  const h = ctx.file.helpers.get(name);
  if (h.state === "busy") skip(`helpers: ${name} refers to itself`);
  if (!h.state) define(name, h, ctx.file);
  if (h.state === "skip") skip(`${h.reason} (in ${name})`);
  for (const [k, line] of h.lets) if (!ctx.lets.has(k)) ctx.lets.set(k, line);
  for (const [k, code] of h.fns) ctx.fns.set(k, code);
  if (h.kind === "const") {
    ctx.lets.set(name, h.line);
    return { c: snake(name), t: h.t };
  }
  ctx.fns.set(name, h.code);
  return { c: snake(name), t: "fn", arity: h.arity, ret: h.t };
}

function define(name, h, file) {
  h.state = "busy";
  const ctx = scope(file);
  try {
    if (!h.node) skip("js: a variable without a value (undefined)");
    if (h.kind === "const") {
      const e = val(ex(h.node, ctx));
      if (!PLAIN.has(e.t)) skip(`helpers: a top-level ${e.t}`);
      Object.assign(h, { line: `let mut ${snake(name)} = ${e.c};`, t: e.t, lets: ctx.lets, fns: ctx.fns });
    } else {
      const { params, body, generator, async } = h.node;
      if (generator || async) skip("helpers: generator or async function");
      if (!params.every((p) => p.type === "Identifier")) skip("helpers: parameter pattern");
      const [ret] = body.body;
      if (body.body.length !== 1 || ret.type !== "ReturnStatement" || !ret.argument) skip("helpers: a body other than one return");
      for (const p of params) ctx.vars.set(p.name, { c: snake(p.name), t: "num" });
      const e = val(ex(ret.argument, ctx));
      if (!RUST_TYPES[e.t]) skip(`helpers: returns a ${e.t}`);
      const args = params.map((p) => `${ctx.used.has(snake(p.name)) ? "" : "_"}${snake(p.name)}: f64`);
      const lines = [...ctx.lets.values(), e.c].map((l) => `    ${l}`);
      const code = `fn ${snake(name)}(${args.join(", ")}) -> ${RUST_TYPES[e.t]} {\n${lines.join("\n")}\n}`;
      Object.assign(h, { code, t: e.t, arity: params.length, lets: new Map(), fns: ctx.fns });
    }
    h.state = "ok";
  } catch (err) {
    if (!(err instanceof Skip)) throw err;
    Object.assign(h, { state: "skip", reason: err.message });
  }
}

const scope = (file) => ({ file, rules: file.rules, vars: new Map(), lets: new Map(), fns: new Map(), used: new Set(), written: new Set() });

function assertionKind(e, rules) {
  if (e?.type !== "CallExpression") return null;
  const c = e.callee;
  if (c.type === "Identifier" && (ASSERTIONS.has(c.name) || rules.assertions?.[c.name])) return c.name;
  if (c.type === "MemberExpression" && !c.computed && c.object.type === "Identifier" && c.object.name === "assert") {
    return `assert.${c.property.name}`;
  }
  return null;
}

/** Every assertion call under `node`, in source order. */
function assertionsIn(node, rules) {
  const found = [];
  (function walk(x) {
    if (Array.isArray(x)) return x.forEach(walk);
    if (!x || typeof x.type !== "string") return;
    if (assertionKind(x, rules)) found.push(x);
    for (const [k, v] of Object.entries(x)) if (k !== "loc" && typeof v === "object") walk(v);
  })(node);
  return found.sort((a, b) => a.start - b.start);
}

const idOf = (callNode, file) => `${file.id}:${callNode.loc.start.line}:${callNode.loc.start.column + 1}`;

function statement(s, ctx, out) {
  ctx.rules.statement?.(s, ctx);
  if (s.type === "VariableDeclaration") {
    for (const d of s.declarations) {
      if (d.id.type !== "Identifier") skip("gap: destructuring");
      if (!d.init) skip("js: a variable without a value (undefined)");
      const { stmt, ...e } = val(ex(d.init, ctx));
      out.push({ declares: snake(d.id.name), c: e.c });
      ctx.vars.set(d.id.name, { ...e, c: snake(d.id.name) });
    }
    return;
  }
  if (s.type === "FunctionDeclaration") {
    ctx.vars.set(s.id.name, { c: snake(s.id.name), t: "local", node: s });
    return;
  }
  if (s.type === "TryStatement" && !s.handler) {
    for (const x of [...s.block.body, ...(s.finalizer?.body ?? [])]) statement(x, ctx, out);
    return;
  }
  if (s.type !== "ExpressionStatement") skip(`gap: ${s.type}`);
  const exprs = s.expression.type === "SequenceExpression" ? s.expression.expressions : [s.expression];
  for (const e of exprs) {
    const kind = assertionKind(e, ctx.rules);
    const local = e.type === "CallExpression" && e.callee.type === "Identifier" && ctx.vars.get(e.callee.name);
    if (kind) {
      assertion(kind, e, ctx, out);
    } else if (local?.t === "local") {
      inline(local.node, e.arguments, ctx, out);
    } else {
      const v = ex(e, ctx);
      out.push(v.stmt ?? (fallible(v.t) ? `${unwrap(v)};` : `${v.c};`));
    }
  }
}

/** A call of a test's own function: its body, in a block, with each parameter bound to its argument's value. */
function inline(fn, args, ctx, out) {
  if (fn.params.length !== args.length || !fn.params.every((p) => p.type === "Identifier")) skip("js: a local function called with other arguments (undefined)");
  const vars = ctx.vars;
  ctx.vars = new Map(vars);
  fn.params.forEach((p, i) => ctx.vars.set(p.name, val(ex(args[i], ctx))));
  out.push("{");
  for (const s of fn.body.body) statement(s, ctx, out);
  out.push("}");
  ctx.vars = vars;
}

function assertion(kind, node, ctx, out) {
  const id = idOf(node, ctx.file);
  const saved = { lets: new Map(ctx.lets), fns: new Map(ctx.fns), used: new Set(ctx.used) };
  // An assertion in an inlined function is listed once, however often it runs.
  const record = (skip) => {
    const prior = ctx.asserts.find((a) => a.id === id);
    if (prior && prior.skip !== skip) throw new Error(`${id}: translated one way and skipped another`);
    if (!prior) ctx.asserts.push({ id, skip });
  };
  try {
    out.push(check(kind, node.arguments, rustStr(id), ctx));
    record(null);
  } catch (err) {
    if (!(err instanceof Skip)) throw err;
    Object.assign(ctx, saved, err.effect ? { used: ctx.used } : {});
    if (err.effect) out.push(err.effect);
    out.push(`skip(${rustStr(id)}, ${rustStr(err.message)});`);
    record(err.message);
  }
}

function check(kind, args, id, ctx) {
  const arg = (i) => ex(nth(args, i), ctx);
  const obs = (e) => {
    const v = val(e);
    if (v.t === "fn") skip("js: function identity");
    return ctx.rules.display?.includes(v.t) ? `${v.c}.to_string()` : v.c;
  };
  const custom = ctx.rules.assertions?.[kind];
  if (custom) return custom(args.map((a) => val(ex(a, ctx))), id, ctx);
  switch (kind) {
    case "assert.strictEqual":
    case "assert.deepStrictEqual":
    case "assert.equal": {
      if (args[1]?.type === "Literal" && args[1].value === null) {
        const e = arg(0);
        if (ctx.rules.nullable?.includes(e.t)) return `check(${id}, Kind::Strict, &${e.c}, &());`;
        return fallible(e.t) ? `check(${id}, Kind::Strict, &${e.c}.is_err(), &true);` : skip("js: null (coercion)");
      }
      const [a, b] = [val(arg(0)), val(arg(1))];
      const same = ctx.rules.equal?.(a, b);
      if (same) return `check(${id}, Kind::Strict, &${same}, &true);`;
      return `check(${id}, Kind::Strict, &${obs(a)}, &${obs(b)});`;
    }
    case "assert.isNaN":
      return `check(${id}, Kind::Strict, &${nan(arg(0))}, &true);`;
    case "assertInDelta": {
      const [actual, expected] = [obs(arg(0)), obs(arg(1))];
      const delta = args.length > 2 ? needNum(arg(2), "delta") : "1e-6";
      return `check(${id}, Kind::InDelta(${delta}), &${actual}, &${expected});`;
    }
    case "assertPathEqual":
      return `check(${id}, Kind::PathEq, &${obs(arg(0))}, &${obs(arg(1))});`;
    case "assert.throws":
      return `throws(${id}, || ${thrower(nth(args, 0), ctx)});`;
    case "assert": {
      const e = val(arg(0));
      if (e.t !== "bool") skip("js: truthiness");
      return `check(${id}, Kind::Strict, &${e.c}, &true);`;
    }
    default:
      return skip(`gap: assertion ${kind}`);
  }
}

/** The fallible Rust call inside `assert.throws(() => call)`. */
function thrower(fn, ctx) {
  if (fn.type !== "ArrowFunctionExpression" && fn.type !== "FunctionExpression") skip("gap: assert.throws without a function");
  let body = fn.body;
  if (body.type === "BlockStatement") {
    if (body.body.length !== 1 || body.body[0].type !== "ExpressionStatement") skip("gap: assert.throws with several statements");
    body = body.body[0].expression;
  }
  const e = ex(body, ctx);
  if (!fallible(e.t)) skip("api: the Rust call does not fail");
  // An `Ok` is reported through Obs, which a displayed type reaches as text.
  return ctx.rules.display?.includes(e.t.slice(7)) ? `${e.c}.map(|v| v.to_string())` : e.c;
}

function translateTest(node, file) {
  const [title, fn] = node.expression.arguments;
  const name = `${file.name}: ${title.type === "Literal" ? title.value : file.source.slice(title.start, title.end)}`;
  const ctx = { ...scope(file), asserts: [] };
  let body = [];
  try {
    const statements = fn.body.type === "BlockStatement" ? fn.body.body : [{ type: "ExpressionStatement", expression: fn.body }];
    for (const s of statements) statement(s, ctx, body);
  } catch (err) {
    if (!(err instanceof Skip)) throw err;
    Object.assign(ctx, scope(file));
    ctx.asserts = assertionsIn(fn.body, file.rules).map((a) => ({ id: idOf(a, file), skip: err.message }));
    body = ctx.asserts.map((a) => `skip(${rustStr(a.id)}, ${rustStr(a.skip)});`);
  }
  const line = (l) => (typeof l === "string" ? l : `let mut ${ctx.used.has(l.declares) ? "" : "_"}${l.declares} = ${l.c};`);
  const finish = file.rules.finish ?? ((l) => l);
  return { name, lines: [...ctx.lets.values(), ...body.map(line)].map(finish), asserts: ctx.asserts, fns: ctx.fns };
}

const isTest = (node) =>
  node.type === "ExpressionStatement" &&
  node.expression.type === "CallExpression" &&
  node.expression.callee.type === "Identifier" &&
  ["it", "test"].includes(node.expression.callee.name) &&
  /Function/.test(node.expression.arguments[1]?.type ?? "");

/**
 * Translates the test file `source`, whose ids start with `id` (its path
 * under the vendored d3 root), under `rules`. Returns its tests ({name,
 * lines, asserts}) and the helper functions they use (name → Rust code).
 */
export function translateFile(source, { id, rules }) {
  const ast = acorn.parse(source, { ecmaVersion: "latest", sourceType: "module", locations: true });
  const file = { id, name: id.slice(id.lastIndexOf("/") + 1), rules, source, helpers: new Map() };
  for (const node of ast.body) {
    if (node.type === "FunctionDeclaration") file.helpers.set(node.id.name, { kind: "fn", node });
    if (node.type === "VariableDeclaration") {
      for (const d of node.declarations) if (d.id.type === "Identifier") file.helpers.set(d.id.name, { kind: "const", node: d.init });
    }
  }
  const tests = ast.body.filter(isTest).map((node) => translateTest(node, file));
  const fns = new Map();
  const seen = new Set();
  for (const t of tests) {
    for (const [k, code] of t.fns) fns.set(k, code);
    for (const a of t.asserts) {
      if (seen.has(a.id)) throw new Error(`duplicate assertion id ${a.id}`);
      if (a.skip !== null && !CATEGORIES.includes(a.skip.slice(0, a.skip.indexOf(":")))) throw new Error(`${a.id}: uncategorized skip "${a.skip}"`);
      seen.add(a.id);
    }
  }
  return { tests, fns };
}
