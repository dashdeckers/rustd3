// d3-time under America/Los_Angeles, the zone of d3's test script. An
// interval is {t: "interval", c, utc}: a time::Interval and whether d3's
// name for it is utc… (its calls take &TimeZone::UTC) or time… (&la, the
// test's `let la`); two intervals are equal when their Interval values are,
// since the zone is an argument of each call. Dates are "date" (Timestamp:
// the prelude's local, utc and millis), "odate" (Option<Timestamp>, an
// interval's result, None where d3 has an Invalid Date) and "dates"
// (Vec<Timestamp>); assertions see them through the prelude's ts and tss,
// RFC 3339 with nine fraction digits. `+date` and date.getTime() are exact
// epoch milliseconds.
import { calleeName, ex, needNum, skip, val } from "../expr.mjs";

const LA = 'let la = zone("America/Los_Angeles");';
const WEEKDAYS = ["Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday"];
const UNITS = { Millisecond: "Millisecond(1)", Second: "Second(1)", Minute: "Minute(1)", Hour: "Hour(1)", Day: "Day(1)", Month: "Month(1)", Year: "Year(1)", Week: "Week(Weekday::Sunday, 1)" };
for (const day of WEEKDAYS) UNITS[day] = `Week(Weekday::${day}, 1)`;
const ALIAS = { c: "()", t: "alias" };
const identifiers = { unixDay: { c: "Interval::UnixDay(1)", t: "interval", utc: true }, unixDays: ALIAS };
for (const [unit, code] of Object.entries(UNITS)) {
  identifiers[`time${unit}`] = { c: `Interval::${code}`, t: "interval", utc: false };
  identifiers[`utc${unit}`] = { c: `Interval::${code}`, t: "interval", utc: true };
  identifiers[`time${unit}s`] = identifiers[`utc${unit}s`] = ALIAS;
}
const DATES = new Set(["date", "odate"]);
const ROUNDING = new Set(["floor", "ceil", "round"]);

/** The zone argument of an interval's calls; `&la` declares la in the test. */
function zone(utc, ctx) {
  if (utc) return "&TimeZone::UTC";
  ctx.lets.set("la", LA);
  return "&la";
}

/** A date argument as Timestamp code; d3 coerces anything else. */
function date(e) {
  if (e?.t === "date") return e.c;
  if (e?.t === "odate") return `${e.c}.expect("d3 test: a valid date")`;
  return skip(`js: ${e?.t === "num" ? "a number" : (e?.t ?? "a missing value")} as a date (coercion)`);
}

/** A literal whole number ≥ `min` as Rust code of the integer type `ty`. */
function whole(e, what, ty, min = 0) {
  const c = needNum(e ?? skip(`js: a missing ${what} (undefined)`), what);
  if (c === "f64::NAN") skip(`js: a NaN ${what} (coercion)`);
  if (!/^-?\d+(\.\d+)?$/.test(c)) skip(`gap: a ${what} other than a literal`);
  const n = Number(c);
  if (!Number.isInteger(n)) skip(`js: a ${what} that is not a whole number (Math.floor)`);
  if (n < min) skip(`types: a ${what} below ${min} (${ty})`);
  return `${n}_${ty}`;
}

const number = (e) => (DATES.has(e.t) ? { c: `(${date(e)}.as_millisecond() as f64)`, t: "num" } : undefined);

/** The prelude's local(…) or utc(…): ECMAScript's MakeDate over 1 to 7 fields. */
function civil(name, args, ctx) {
  if (args.length < 1 || args.length > 7) skip(`js: ${name} with ${args.length} fields`);
  const fields = `&[${args.map((a) => needNum(a, `${name} field`)).join(", ")}]`;
  return { c: name === "utc" ? `utc(${fields})` : `local(${zone(false, ctx)}, ${fields})`, t: "date" };
}

/** timeTicks or utcTicks over a count or an interval. */
function ticks(utc, [a, b, n], ctx) {
  const [start, stop] = [date(a), date(b)];
  if (n?.t === "interval") return { c: `time::ticks_with(${start}, ${stop}, ${n.c}, ${zone(utc, ctx)})`, t: "dates" };
  const count = whole(n, "tick count", "usize");
  return { c: utc ? `time::utc_ticks(${start}, ${stop}, ${count})` : `time::ticks(${start}, ${stop}, ${count}, ${zone(false, ctx)})`, t: "dates" };
}

export default {
  identifiers,
  call(node, ctx) {
    const name = calleeName(node, ctx);
    const args = () => node.arguments.map((a) => val(ex(a, ctx)));
    if (node.type === "NewExpression") {
      if (name !== "Date") return undefined;
      const [ms, ...fields] = args();
      if (!ms) skip("api: ambient clock (new Date)");
      if (fields.length) return Number(ms.c) >= 0 && Number(ms.c) < 100 ? skip("js: new Date's two-digit years") : civil("local", [ms, ...fields], ctx);
      return needNum(ms, "new Date argument") === "f64::NAN" ? skip("js: an invalid date (new Date(NaN))") : { c: `millis(${ms.c})`, t: "date" };
    }
    if (name === "local" || name === "utc") return civil(name, args(), ctx);
    if (name === "timeTicks" || name === "utcTicks") return ticks(name === "utcTicks", args(), ctx);
    if (name !== "timeTickInterval" && name !== "utcTickInterval") return undefined;
    const [a, b, n] = args();
    const [start, stop, count, utc] = [date(a), date(b), whole(n, "tick count", "usize"), name === "utcTickInterval"];
    return { c: utc ? `time::utc_tick_interval(${start}, ${stop}, ${count})` : `time::tick_interval(${start}, ${stop}, ${count}, ${zone(false, ctx)})`, t: "interval", utc };
  },
  method(obj, prop, args, ctx) {
    if (DATES.has(obj.t) && prop === "getTime" && !args.length) return number(obj);
    if (obj.t !== "interval") return undefined;
    const [a, b, c] = args.map(val);
    const z = () => zone(obj.utc, ctx);
    if (ROUNDING.has(prop)) return { c: `${obj.c}.${prop}(${date(a)}, ${z()})`, t: "odate" };
    if (prop === "offset") return { c: `${obj.c}.offset(${date(a)}, ${b ? whole(b, "offset step", "i64", -Infinity) : "1_i64"}, ${z()})`, t: "odate" };
    if (prop === "count") return { c: `${obj.c}.count(${date(a)}, ${date(b)}, ${z()}).map_or(f64::NAN, |n| n as f64)`, t: "num" };
    if (prop === "every") return { ...obj, c: `${obj.c}.every(${whole(a, "every step", "u32", 1)}).expect("d3 test: a step of 1 or more")` };
    if (prop !== "range") return undefined;
    const range = c ? `range_step(${date(a)}, ${date(b)}, ${whole(c, "range step", "u32")}, ${z()})` : `range(${date(a)}, ${date(b)}, ${z()})`;
    return { c: `${obj.c}.${range}.collect::<Vec<_>>()`, t: "dates" };
  },
  field(obj, prop, ctx, node) {
    if (obj.t === "interval" && prop === "range") return ALIAS;
    if (obj.t === "interval") skip(`api: interval.${prop} as a value (d3's every intervals lack count and every; an Interval has both)`);
    if (obj.t !== "dates" || !node.computed) return undefined;
    return { c: `${obj.c}[${whole(ex(node.property, ctx), "index", "usize")}]`, t: "date" };
  },
  apply(target, args, ctx) {
    if (target.t !== "interval") return undefined;
    if (!args.length) skip("api: ambient clock (interval() floors new Date)");
    return { c: `${target.c}.floor(${date(val(args[0]))}, ${zone(target.utc, ctx)})`, t: "odate" };
  },
  equal(a, b) {
    if (a.t === "alias" || b.t === "alias") skip("api: d3's plural aliases of range (one name per function)");
    return a.t === "interval" && b.t === "interval" ? `(${a.c} == ${b.c})` : undefined;
  },
  array(node, ctx) {
    const values = node.elements.map((e) => (e ? val(ex(e, ctx)) : skip("js: array hole")));
    if (!values.length || !values.every((v) => v.t === "date")) return undefined;
    return { c: `vec![${values.map((v) => v.c).join(", ")}]`, t: "dates" };
  },
  number,
  observe(e) {
    if (DATES.has(e.t)) return `ts(${e.c})`;
    return e.t === "dates" ? `tss(&${e.c})` : undefined;
  },
};
