// d3-time-format's names, and d3-scale's time tickFormat, under
// time::Locale::EN_US (d3's default) and America/Los_Angeles. A format is
// {t: "tfmt", c, utc}: a time::Format and whether d3 made it with utcFormat
// (its calls take &TimeZone::UTC) or timeFormat (&la). timeFormatLocale's
// definition, an object literal or a vendored JSON table read now, is a
// time::Definition literal with d3's ladder; scale.tickFormat() is the scale
// itself, "tickfmt", applied through Time::tick_label. Parsing is Later, and
// timeFormatDefaultLocale rebinds d3's global default, which Rust has not.
import { readFileSync, readdirSync } from "node:fs";
import { calleeName, ex, idOf, rustStr, skip, val } from "../expr.mjs";
import { civil, date, zone } from "./time.mjs";

const EN_US = "time::Locale::EN_US";
const TABLES = new URL("../../d3/d3-time-format/locale/", import.meta.url);
const PARSING = "api: parsing d3 format strings (Later; jiff's strtime and time::iso_parse parse)";
// d3's definition keys, the Definition field each fills and its length.
const FIELDS = [
  ["dateTime", "date_time"], ["date", "date"], ["time", "time"], ["periods", "periods", 2], ["days", "days", 7],
  ["shortDays", "short_days", 7], ["months", "months", 12], ["shortMonths", "short_months", 12],
];

const spec = (e) => (e?.t === "str" ? e.c : skip(`types: a ${e?.t ?? "missing"} specifier (format takes a string)`));
const make = (locale, s, utc) => ({ c: `${locale}.format(${spec(s)})`, t: "tfmt", utc });
const json = (file) => JSON.parse(readFileSync(new URL(file, TABLES), "utf8"));
const byCodeUnit = (a, b) => (a < b ? -1 : a > b ? 1 : 0);

/** A time::Definition literal holding d3's definition `d`, with d3's ladder. */
function fields(d) {
  const str = (v) => (typeof v === "string" ? `${rustStr(v)}.into()` : skip("js: a locale field that is not a string (coercion)"));
  const out = FIELDS.map(([key, field, n]) => {
    const v = d[key] ?? skip(`js: a locale table without ${key} (undefined)`);
    if (!n) return `${field}: ${str(v)}`;
    return Array.isArray(v) && v.length === n ? `${field}: [${v.map(str).join(", ")}]` : skip(`js: ${key} that is not ${n} names`);
  });
  return `time::Definition { ${[...out, "ladder: time::Ladder::D3"].join(", ")} }`;
}

/** timeFormatLocale's argument: an object literal, or a top-level const reading a vendored table. */
function definition(node, ctx) {
  const init = node?.type === "Identifier" ? ctx.file.helpers.get(node.name)?.node : node;
  const file = init && /readFileSync\("\.\/locale\/([\w-]+\.json)"\)/.exec(ctx.file.source.slice(init.start, init.end));
  if (file) return fields(json(file[1]));
  if (init?.type !== "ObjectExpression") skip("js: a locale definition that is neither an object literal nor a vendored table");
  const value = (v) => (v.type === "Literal" ? v.value : v.type === "ArrayExpression" ? v.elements.map(value) : null);
  return fields(Object.fromEntries(init.properties.map((p) => [p.key.name ?? p.key.value, value(p.value)])));
}

/**
 * locale-test.js's loop over every JSON table: a Rust loop over each
 * table's sorted keys and Definition, its assertion checking the keys and
 * each Definition built through Locale::new.
 */
function everyTable(s, ctx) {
  const files = readdirSync(TABLES).filter((f) => /\.json$/i.test(f)).sort();
  const lines = [`let tables: [(&[&str], time::Definition); ${files.length}] = [`];
  for (const file of files) lines.push(`    (&[${Object.keys(json(file)).sort(byCodeUnit).map(rustStr).join(", ")}], ${fields(json(file))}),`);
  lines.push("];", "for (keys, definition) in tables {");
  for (const x of s.body.body) {
    const e = x.expression;
    if (x.type === "IfStatement" || x.type === "VariableDeclaration") continue; // the filter and the loop's binding
    if (e?.callee?.property?.name === "deepStrictEqual") {
      const id = idOf(e, ctx.file);
      lines.push(`    check(${rustStr(id)}, Kind::Strict, &keys, &${ex(e.arguments[1], ctx).c});`);
      ctx.asserts.push({ id, skip: null });
    } else if (e && calleeName(e, ctx) === "timeFormatLocale") lines.push("    time::Locale::new(definition);");
    else skip(`gap: ${x.type} in the loop over locale tables`);
  }
  return [...lines, "}"];
}

export default {
  call(node, ctx) {
    const name = calleeName(node, ctx);
    const { callee } = node;
    if (callee.type === "MemberExpression" && callee.object.name === "Date" && callee.property.name === "UTC") {
      return { c: `(${civil("utc", node.arguments.map((a) => val(ex(a, ctx))), ctx).c}.as_millisecond() as f64)`, t: "num" };
    }
    if (callee.type === "MemberExpression" && callee.property.name === "tickFormat") {
      const scale = val(ex(callee.object, ctx));
      if (scale.t !== "tscale") return undefined;
      if (!node.arguments.length) return { c: scale.c, t: "tickfmt" };
      return make(EN_US, val(ex(node.arguments[1] ?? skip("js: tickFormat(count) without a specifier (undefined)"), ctx)), scale.utc);
    }
    if (name === "timeParse" || name === "utcParse") skip(PARSING);
    if (name === "timeFormatDefaultLocale") skip("api: timeFormatDefaultLocale rebinds d3's global default locale (no mutable default)");
    if (name === "timeFormatLocale") return { c: `time::Locale::new(${definition(node.arguments[0], ctx)})`, t: "tlocale" };
    if (ctx.file.helpers.get(name)?.kind === "fn") skip(`helpers: ${name}(), a file-level function choosing among file-level formats (Time::tick_label ports it)`);
    const args = () => node.arguments.map((a) => val(ex(a, ctx)));
    if (name === "timeFormat" || name === "utcFormat") return make(EN_US, args()[0], name === "utcFormat");
    if (name === "isoFormat") return { c: `time::iso_format(${date(args()[0])})`, t: "str" };
    if (name === "isoParse") return { c: `time::iso_parse(${spec(args()[0])})`, t: "result-date" };
    return undefined;
  },
  method(obj, prop, args) {
    if (obj.t !== "tlocale") return undefined;
    if (prop === "parse" || prop === "utcParse") skip(PARSING);
    return prop === "format" || prop === "utcFormat" ? make(obj.c, args[0], prop === "utcFormat") : undefined;
  },
  apply(target, args, ctx) {
    const at = () => date(val(args[0] ?? skip("js: a format called without a date (undefined)")));
    if (target.t === "tfmt") return { c: `${target.c}.format(${at()}, ${zone(target.utc, ctx)})`, t: "str" };
    if (target.t === "tickfmt") return { c: `${target.c}.tick_label(${at()}, &${EN_US})`, t: "str" };
    return undefined;
  },
  statement(s, ctx) {
    const text = ctx.file.source.slice(s.start, s.end);
    if (s.type === "ForOfStatement" && text.includes('readdirSync("locale")')) return everyTable(s, ctx);
  },
};
