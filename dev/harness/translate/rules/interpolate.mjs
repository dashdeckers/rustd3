// d3-interpolate's number interpolators: interpolateNumber(a, b) and
// interpolateRound(a, b) are closures of t.
import { calleeName, ex, needNum, skip } from "../expr.mjs";

const FNS = { interpolateNumber: "interpolate::number", interpolateRound: "interpolate::round" };

export default {
  call(node, ctx) {
    const name = calleeName(node, ctx);
    if (!FNS[name]) return undefined;
    const args = node.arguments.map((a) => ex(a, ctx));
    if (args.length !== 2) skip(`js: ${args.length} arguments to ${name} (undefined)`);
    return { c: `${FNS[name]}(${args.map((a) => needNum(a, `${name} argument`)).join(", ")})`, t: "fn", arity: 1 };
  },
};
