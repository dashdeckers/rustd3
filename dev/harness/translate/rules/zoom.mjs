// d3-zoom's ZoomTransform: zoomIdentity and new ZoomTransform(k, x, y) are
// interact::Transform values {t: "zoom"}, equal when their fields are;
// apply and invert take a literal [x, y] and return an (f64, f64) tuple.
import { calleeName, ex, needNum, skip } from "../expr.mjs";

const zoom = (c) => ({ c, t: "zoom" });
const AXIS = { applyX: "apply_x", applyY: "apply_y", invertX: "invert_x", invertY: "invert_y" };

function nums(args, n, what) {
  if (args.length !== n) skip(`js: ${args.length} arguments to ${what} for ${n} (undefined)`);
  return args.map((a) => needNum(a, `${what} argument`));
}

function point([p, ...rest]) {
  if (rest.length || p?.t !== "vec" || p.items?.length !== 2) skip("js: a point other than a literal [x, y]");
  return `(${p.items.join(", ")})`;
}

export default {
  identifiers: { zoomIdentity: zoom("interact::Transform::IDENTITY") },
  call(node, ctx) {
    if (node.type !== "NewExpression" || calleeName(node, ctx) !== "ZoomTransform") return undefined;
    const [k, x, y] = nums(node.arguments.map((a) => ex(a, ctx)), 3, "ZoomTransform");
    return zoom(`interact::Transform { k: ${k}, x: ${x}, y: ${y} }`);
  },
  method(obj, prop, args) {
    if (obj.t !== "zoom") return undefined;
    if (prop === "scale") return zoom(`${obj.c}.scale(${nums(args, 1, prop)})`);
    if (prop === "translate") return zoom(`${obj.c}.translate(${nums(args, 2, prop).join(", ")})`);
    if (prop === "apply" || prop === "invert") return { c: `${obj.c}.${prop}(${point(args)})`, t: "pair" };
    if (prop === "toString") return { c: `${obj.c}.to_string()`, t: "str" };
    if (Object.hasOwn(AXIS, prop)) return { c: `${obj.c}.${AXIS[prop]}(${nums(args, 1, prop)})`, t: "num" };
    return undefined;
  },
  equal: (a, b) => (a.t === "zoom" && b.t === "zoom" ? `(${a.c} == ${b.c})` : undefined),
};
