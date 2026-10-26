// d3-ease's names: every easing is an Ease value, calling one applies it,
// its parameter setters build struct-literal variants, and two easings are
// the same function when they are equal values. out and inOut, d3-ease's
// test helpers, are the prelude's.
import { calleeName, ex, needNum, skip } from "../expr.mjs";

const PARAMS = { Linear: [], Quad: [], Cubic: [], Sin: [], Exp: [], Circle: [], Bounce: [],
  Poly: ["exponent"], Back: ["overshoot"], Elastic: ["amplitude", "period"] };
const DEFAULTS = { exponent: "3.0", overshoot: "1.70158", amplitude: "1.0", period: "0.3" };
// d3's unsuffixed names: easeBounce and easeElastic are Out, the rest InOut.
const ALIAS = { Bounce: "Out", Elastic: "Out" };

/** The Ease value of `family` in `mode` with the parameters set so far. */
function ease(family, mode, set = {}) {
  const params = PARAMS[family];
  let c = `Ease::${family}(Mode::${mode})`;
  if (family === "Linear") c = "Ease::Linear";
  else if (params.length && !Object.keys(set).length) c = `Ease::${family.toLowerCase()}(Mode::${mode})`;
  else if (params.length) c = `Ease::${family} { mode: Mode::${mode}, ${params.map((p) => `${p}: ${set[p] ?? DEFAULTS[p]}`).join(", ")} }`;
  return { c, t: "ease", ease: { family, mode, set } };
}

const identifiers = { easeLinear: ease("Linear") };
for (const family of Object.keys(PARAMS).slice(1)) {
  identifiers[`ease${family}`] = ease(family, ALIAS[family] ?? "InOut");
  for (const mode of ["In", "Out", "InOut"]) identifiers[`ease${family}${mode}`] = ease(family, mode);
}

export default {
  identifiers,
  call(node, ctx) {
    const name = calleeName(node, ctx);
    if (name !== "out" && name !== "inOut") return undefined;
    const e = ex(node.arguments[0], ctx);
    if (e.t !== "ease") skip(`js: ${name} of a ${e.t}`);
    return { c: `${name === "out" ? "out" : "in_out"}(${e.c})`, t: "fn", arity: 1 };
  },
  // A setter keeps the other parameter, as d3's custom(a, p) does.
  method(obj, prop, args) {
    if (obj.t !== "ease" || !PARAMS[obj.ease.family].includes(prop)) return undefined;
    return ease(obj.ease.family, obj.ease.mode, { ...obj.ease.set, [prop]: needNum(args[0] ?? skip("js: no argument (undefined)"), prop) });
  },
  apply(target, args) {
    if (target.t !== "ease") return undefined;
    if (args.length !== 1) skip(`js: ${args.length} arguments to an easing (undefined)`);
    return { c: `${target.c}.apply(${needNum(args[0], "easing argument")})`, t: "num" };
  },
  equal: (a, b) => (a.t === "ease" && b.t === "ease" ? `(${a.c} == ${b.c})` : undefined),
};
