// d3-array's tick functions, and the array methods their tests call.
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
};
