// d3-format's names. Tests run under format::Locale::EN_US, d3's default;
// formatLocale({…}) is Locale::new over a Definition literal; a test's
// locale("xx-YY") reads d3's JSON table at translation time: the bundled
// constant when core has one, else a Definition literal; specifiers have
// read-only fields; the precision functions' NaN is None (opt).
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { calleeName, ex, idOf, needNum, rustStr, skip } from "../expr.mjs";

const EN_US = "format::Locale::EN_US";
const TABLES = new URL("../../d3/d3-format/locale/", import.meta.url);
const BUNDLED = new URL("../../../../core/src/format/locales/", import.meta.url);
const TEXT = ["decimal", "thousands", "percent", "minus", "nan"];
const READ = {
  fill: [".fill().to_string()", "str"], align: [".align().to_string()", "str"], sign: [".sign().to_string()", "str"],
  symbol: [".symbol().map(|s| s.to_string()).unwrap_or_default()", "str"],
  type: [".kind().map(|k| k.to_string()).unwrap_or_default()", "str"],
  zero: [".zero()", "bool"], comma: [".comma()", "bool"], trim: [".trim()", "bool"],
  width: [".width()", "opt"], precision: [".precision()", "opt"],
};

const spec = (e) => (e?.t === "str" ? e.c : skip(`types: a ${e?.t} specifier (format takes a string)`));

/** locale.format(s) or locale.formatPrefix(s, value). */
function make(locale, name, args) {
  if (name === "format") return { c: `${locale}.format(${spec(args[0])})`, t: "result-fmt" };
  return { c: `${locale}.format_prefix(${spec(args[0])}, ${needNum(args[1] ?? skip("js: no value (undefined)"), "value")})`, t: "result-fmt" };
}

/** A Definition literal from formatLocale's argument, or a top-level const holding one. */
function definition(node, ctx) {
  const object = node?.type === "Identifier" ? ctx.file.helpers.get(node.name)?.node : node;
  if (object?.type !== "ObjectExpression") skip("js: a locale definition that is not an object literal");
  const value = (v) => (v.type === "Literal" ? v.value : v.type === "ArrayExpression" ? v.elements.map(value) : null);
  return fields(Object.fromEntries(object.properties.map((p) => [p.key.name ?? p.key.value, value(p.value)])));
}

/** A Definition literal holding d3's definition `d`: its strings, string arrays and group sizes. */
function fields(d) {
  const str = (v) => (typeof v === "string" ? `${rustStr(v)}.into()` : skip("js: a locale field that is not a string (coercion)"));
  const size = (n) => (Number.isInteger(n) && n >= 0 ? String(n) : skip("js: a group size that is not a count"));
  const list = (v, n, f) => (Array.isArray(v) && (!n || v.length === n) ? v.map(f).join(", ") : skip("js: a malformed locale array"));
  const out = [];
  for (const [key, v] of Object.entries(d)) {
    if (TEXT.includes(key)) out.push(`${key}: ${str(v)}`);
    // d3 groups only when thousands is given too.
    else if (key === "grouping" && "thousands" in d) out.push(`grouping: vec![${list(v, 0, size)}].into()`);
    else if (key === "currency") out.push(`currency: [${list(v, 2, str)}]`);
    else if (key === "numerals") out.push(`numerals: Some([${list(v, 10, str)}])`);
    else if (key !== "grouping") skip(`js: locale field ${key}`);
  }
  return `format::Definition { ${[...out, "..Default::default()"].join(", ")} }`;
}

const json = (file) => JSON.parse(readFileSync(new URL(file, TABLES), "utf8"));

/** A test's locale("xx-YY"): d3's table read now, as core's constant when it bundles the table. */
function table(node) {
  const [name] = node.arguments;
  if (name?.type !== "Literal" || typeof name.value !== "string") skip("js: a locale name that is not a string literal");
  const rust = name.value.replace("-", "_");
  if (existsSync(new URL(`${rust.toLowerCase()}.rs`, BUNDLED))) return `format::Locale::${rust.toUpperCase()}`;
  return `format::Locale::new(${fields(json(`${name.value}.json`))})`;
}

/**
 * locale-test.js's loop over every JSON table: a Rust loop over each
 * table's keys and Definition. The loop's `in` checks become `contains`,
 * formatLocale(locale) builds the locale.
 */
function everyTable(s, ctx) {
  const files = readdirSync(TABLES).filter((f) => /\.json$/i.test(f)).sort();
  const lines = [`let tables: [(&[&str], format::Definition); ${files.length}] = [`];
  for (const file of files) lines.push(`    (&[${Object.keys(json(file)).map(rustStr).join(", ")}], ${fields(json(file))}),`);
  lines.push("];", "for (keys, definition) in tables {");
  for (const x of s.body.body) {
    const e = x.expression;
    const [has, yes] = e?.arguments ?? [];
    if (x.type === "IfStatement" || x.type === "VariableDeclaration") continue; // the filter and the loop's binding
    if (has?.operator === "in" && yes?.value === true && e.callee.property?.name === "strictEqual") {
      const id = idOf(e, ctx.file);
      lines.push(`    check(${rustStr(id)}, Kind::Strict, &keys.contains(&${rustStr(has.left.value)}), &true);`);
      ctx.asserts.push({ id, skip: null });
    } else if (e && calleeName(e, ctx) === "formatLocale") lines.push("    format::Locale::new(definition);");
    else skip(`gap: ${x.type} in the loop over locale tables`);
  }
  return [...lines, "}"];
}

export default {
  display: ["fmt", "spec"],
  identifiers: {
    precisionFixed: { c: "format::precision_fixed", t: "fn", arity: 1, ret: "opt" },
    precisionPrefix: { c: "format::precision_prefix", t: "fn", arity: 2, ret: "opt" },
    precisionRound: { c: "format::precision_round", t: "fn", arity: 2, ret: "opt" },
    format: { c: "_", t: "fn", arity: 1 },
    formatPrefix: { c: "_", t: "fn", arity: 2 },
  },
  call(node, ctx) {
    const name = calleeName(node, ctx);
    if (node.type === "NewExpression") {
      if (name !== "FormatSpecifier") return undefined;
      const [o] = node.arguments;
      if (o?.type !== "ObjectExpression" || o.properties.length) skip("js: FormatSpecifier coerces its fields");
      return { c: "format::Specifier::default()", t: "spec" };
    }
    if (name === "formatLocale" || name === "formatDefaultLocale") {
      ctx.rebound ||= name === "formatDefaultLocale";
      return { c: `format::Locale::new(${definition(node.arguments[0], ctx)})`, t: "locale" };
    }
    const h = ctx.file.helpers.get(name);
    if (h?.kind === "fn" && ctx.file.source.slice(h.node.start, h.node.end).includes("./locale/")) return { c: table(node), t: "locale" };
    if (name !== "format" && name !== "formatPrefix" && name !== "formatSpecifier") return undefined;
    if (ctx.rebound && name !== "formatSpecifier") skip("js: formatDefaultLocale rebinds the module's exports");
    const args = node.arguments.map((a) => ex(a, ctx));
    if (name === "formatSpecifier") return { c: `${spec(args[0])}.parse::<format::Specifier>()`, t: "result-spec" };
    return make(EN_US, name, args);
  },
  method(obj, prop, args) {
    if (obj.t === "locale" && (prop === "format" || prop === "formatPrefix")) return make(obj.c, prop, args);
    if ((obj.t === "fmt" || obj.t === "spec") && prop === "toString") return { c: `${obj.c}.to_string()`, t: "str" };
    return undefined;
  },
  field(obj, prop) {
    if (obj.t === "spec" && READ[prop]) return { c: `${obj.c}${READ[prop][0]}`, t: READ[prop][1] };
    if (obj.t === "locale" && (prop === "format" || prop === "formatPrefix")) return { c: "_", t: "fn", arity: 1 };
    return undefined;
  },
  apply(target, args) {
    if (target.t !== "fmt") return undefined;
    if (args[0]?.t === "str") skip("types: Format::format takes a number, not a string");
    return { c: `${target.c}.format(${needNum(args[0] ?? skip("js: no value (undefined)"), "formatted value")})`, t: "str" };
  },
  // locale-test.js loops over the JSON locale tables; precisionPrefix-test.js
  // builds its numbers from strings, +("1e" + i).
  statement(s, ctx) {
    const text = ctx.file.source.slice(s.start, s.end);
    if (s.type === "ForOfStatement" && text.includes("readdirSync")) return everyTable(s, ctx);
    if (s.type === "ForStatement" && text.includes('+("1e"')) skip("js: numbers parsed from strings (coercion)");
  },
};
