// d3-format's names. Tests run under format::Locale::EN_US, d3's default;
// formatLocale({…}) is Locale::new over a Definition literal; specifiers
// have read-only fields; the precision functions' NaN is None (opt).
import { calleeName, ex, needNum, rustStr, skip } from "../expr.mjs";

const EN_US = "format::Locale::EN_US";
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
  if (object?.type !== "ObjectExpression") skip("gap: phase 26 (locale tables)");
  const props = new Map(object.properties.map((p) => [p.key.name ?? p.key.value, p.value]));
  const str = (v) => (v?.type === "Literal" && typeof v.value === "string" ? `${rustStr(v.value)}.into()` : skip("js: a locale field that is not a string (coercion)"));
  const size = (e) => (Number.isInteger(e.value) && e.value >= 0 ? String(e.value) : skip("js: a group size that is not a count"));
  const list = (v, n, f) => (v.type === "ArrayExpression" && (!n || v.elements.length === n) ? v.elements.map(f).join(", ") : skip("js: a malformed locale array"));
  const fields = [];
  for (const [key, v] of props) {
    if (TEXT.includes(key)) fields.push(`${key}: ${str(v)}`);
    // d3 groups only when thousands is given too.
    else if (key === "grouping" && props.has("thousands")) fields.push(`grouping: vec![${list(v, 0, size)}].into()`);
    else if (key === "currency") fields.push(`currency: [${list(v, 2, str)}]`);
    else if (key === "numerals") fields.push(`numerals: Some([${list(v, 10, str)}])`);
    else if (key !== "grouping") skip(`js: locale field ${key}`);
  }
  return `format::Locale::new(format::Definition { ${[...fields, "..Default::default()"].join(", ")} })`;
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
      return { c: definition(node.arguments[0], ctx), t: "locale" };
    }
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
    if (s.type === "ForOfStatement" && text.includes("readdirSync")) skip("gap: phase 26 (locale tables)");
    if (s.type === "ForStatement" && text.includes('+("1e"')) skip("js: numbers parsed from strings (coercion)");
  },
};
