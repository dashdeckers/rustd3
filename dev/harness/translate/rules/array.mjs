// d3-array's tick functions, and the array methods and properties their
// tests and others' call.
import { ex, needNum } from "../expr.mjs";

export default {
  identifiers: {
    ticks: { c: "ticks::ticks", t: "fn", arity: 3, ret: "vec" },
    tickIncrement: { c: "ticks::tick_increment", t: "fn", arity: 3 },
    tickStep: { c: "ticks::tick_step", t: "fn", arity: 3 },
    nice: { c: "ticks::nice", t: "fn", arity: 3, ret: "vec" },
  },
  method(obj, prop, args) {
    if (obj.t !== "vec" || args.length) return undefined;
    if (prop === "pop") return { c: `${obj.c}.pop().expect("d3 test pop")`, t: "num" };
    if (prop === "reverse") return { c: `{ let mut v = ${obj.c}; v.reverse(); v }`, t: "vec" };
    return undefined;
  },
  // An index past the end panics, where JavaScript reads undefined.
  field(obj, prop, ctx, node) {
    if (obj.t !== "vec") return undefined;
    if (prop === "length") return { c: `(${obj.c}.len() as f64)`, t: "num" };
    if (node.computed) return { c: `${obj.c}[(${needNum(ex(node.property, ctx), "index")}) as usize]`, t: "num" };
    return undefined;
  },
};
