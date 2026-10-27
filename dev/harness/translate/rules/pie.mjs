// d3-shape's pie() and arc(). A pie's layout of a literal array is compared through the prelude's
// `slices`, d3's `data` as its input position. An arc's radii and angles come from its setters, or as
// d3's default accessors read them, from the datum literal it is called with.
import { ex, needNum, skip, snake, val } from "../expr.mjs";
import { generators } from "./generator.mjs";

const API = "api: accessor functions, getters and contexts (pie lays out numbers; arc draws a Slice into a PathContext)";
const SNAKE = Object.fromEntries(["startAngle", "endAngle", "padAngle", "innerRadius", "outerRadius", "cornerRadius", "padRadius"].map((k) => [k, snake(k)]));
const num = (node, ctx) => needNum(val(ex(node, ctx)), "a pie or arc number");
const isNull = (n) => n?.type === "Literal" && n.value === null;

/** A setter's number; a function is d3's accessor. */
const number = (node, ctx) => (/Function/.test(node.type) ? skip(API) : num(node, ctx));

/** The arc `g` called with the datum `node`: its builder and its Slice, each key from g's setters before the datum's. */
function arcOf(g, node, ctx) {
  if (node && node.type !== "ObjectExpression") skip("js: a datum that is not an object literal");
  const d = Object.fromEntries((node?.properties ?? []).map((p) => [p.key.name, num(p.value, ctx)]));
  const need = (k) => g.set[k] ?? d[k] ?? skip(`js: ${k} read from a datum without it (undefined)`);
  const opt = (k) => (g.set[k] === undefined ? "" : `.${SNAKE[k]}(${g.set[k]})`);
  const slice = `shape::Slice { datum: 0, index: 0, value: 0.0, start_angle: ${need("startAngle")}, end_angle: ${need("endAngle")}, pad_angle: ${d.padAngle ?? "0.0"} }`;
  return [`shape::arc().inner_radius(${need("innerRadius")}).outer_radius(${need("outerRadius")})${["cornerRadius", "padRadius", "padAngle"].map(opt).join("")}`, slice];
}

/** sortValues' comparator, `(a, b) => a - b` or `b - a`: d3's ascending and descending. */
function sort(node) {
  const e = node.body?.body?.[0]?.argument ?? node.body;
  const [a, b] = node.params?.map((p) => p.name) ?? [];
  return e?.operator === "-" && { [`${a}-${b}`]: "Ascending", [`${b}-${a}`]: "Descending" }[`${e.left.name}-${e.right.name}`] || skip(API);
}

function set(g, prop, [arg], ctx) {
  const [arc, slice] = prop === "centroid" && g.t === "arc" ? arcOf(g, arg, ctx) : [];
  if (arc) return { c: `{ let (x, y) = ${arc}.centroid(&${slice}); vec![x, y] }`, t: "vec" };
  if (!arg || !SNAKE[prop] && !(g.t === "pie" && /^sort/.test(prop))) skip(API);
  if (g.t === "pie") {
    const order = prop === "sort" ? (isNull(arg) ? "Input" : skip(API)) : prop === "sortValues" ? sort(arg) : null;
    if (order) return { ...g, c: `${g.c}.sort(shape::pie::Sort::${order})` };
    return /Angle$/.test(prop) ? { ...g, c: `${g.c}.${SNAKE[prop]}(${number(arg, ctx)})` } : skip(API);
  }
  const v = prop === "padRadius" && isNull(arg) ? "None" : number(arg, ctx);
  return { ...g, set: { ...g.set, [prop]: /^pad/.test(prop) && v !== "None" ? `Some(${v})` : v } };
}

function run(g, [arg], ctx) {
  if (g.t === "arc") {
    const [arc, slice] = arcOf(g, arg, ctx);
    return { c: `drawn(|ctx| ${arc}.draw(&${slice}, ctx))`, t: "path" };
  }
  if (arg?.type !== "ArrayExpression") skip("types: data other than an array literal (layout takes numbers)");
  ctx.pie = arg.elements.map((e) => (e ? needNum(val(ex(e, ctx)), "a pie value") : skip("js: array hole")));
  return { c: `${g.c}.layout(vec![${ctx.pie.join(", ")}])`, t: "slices" };
}

/** d3's arcs as object literals against the last pie's input: `data` becomes the datum's position. */
function slices(node, ctx) {
  if (!node.elements.length || !node.elements.every((e) => e?.type === "ObjectExpression")) return undefined;
  const input = ctx.pie ?? skip("js: arcs without a pie");
  const items = node.elements.map((o, i) => {
    const f = Object.fromEntries(o.properties.map((p) => [p.key.name, p.key.name === "index" ? p.value.value : num(p.value, ctx)]));
    const at = input[i] === f.data ? i : input.indexOf(f.data);
    const n = (k) => `${snake(k)}: ${f[k]}`;
    return at < 0 ? skip("js: data that is not an input value") : `shape::Slice { datum: ${at}, ${["index", "value", "startAngle", "endAngle", "padAngle"].map(n).join(", ")} }`;
  });
  return { c: `slices(&[${items.join(", ")}])`, t: "obs" };
}

const make = { pie: () => ({ c: "shape::pie()", t: "pie" }), arc: () => ({ t: "arc", set: {} }) };

export default {
  ...generators({ types: ["pie", "arc"], make, set, run }),
  array: slices,
  method: (obj, prop, [f]) => (obj.t === "vec" && prop === "map" && f?.t === "fn" ? { c: `${obj.c}.into_iter().map(${f.c}).collect::<Vec<f64>>()`, t: "vec" } : undefined),
  observe: (e) => (e.t === "slices" ? `slices(&${e.c})` : e.t === "obs" ? e.c : undefined),
};
