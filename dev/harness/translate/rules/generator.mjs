// d3-shape's generators at translation time: typed values holding their setters' arguments as Rust code.
// A setter on a test variable rewrites the variable (d3's setters mutate), which has no Rust binding.
import { ex, skip, snake, val } from "../expr.mjs";

/** The identifier a chain `x.m(…)(…).n` starts from. */
function rootName(n) {
  while (n.type === "CallExpression" || n.type === "MemberExpression") n = n.type === "CallExpression" ? n.callee : n.object;
  return n.type === "Identifier" ? n.name : null;
}

/** Hooks for generators of `types`: `make` {d3 name: (args, ctx) => g}; `set(g, prop, args, ctx)` a
 * method's value (one of g's type is g's new state); `run(g, args, ctx)` calling g. Args are nodes. */
export function generators({ types, make, set, run }) {
  const ours = (node, ctx) => {
    const name = rootName(node);
    return name !== null && (ctx.vars.has(name) ? types.includes(ctx.vars.get(name).t) : Object.hasOwn(make, name));
  };
  return {
    call(node, ctx) {
      if (!ours(node, ctx)) return undefined;
      const { callee } = node;
      if (callee.type === "Identifier" && !ctx.vars.has(callee.name)) return make[callee.name](node.arguments, ctx);
      const g = val(ex(callee.type === "MemberExpression" ? callee.object : callee, ctx));
      if (!types.includes(g.t)) return undefined;
      if (callee.type !== "MemberExpression") return run(g, node.arguments, ctx);
      const out = set(g, callee.property.name, node.arguments, ctx);
      if (out?.t === g.t && g.name) ctx.vars.set(g.name, { ...out, name: g.name });
      return out?.t === g.t && g.name ? ctx.vars.get(g.name) : out;
    },
    statement(s, ctx) {
      const inits = s.type === "VariableDeclaration" ? s.declarations.map((d) => d.init) : [];
      // d3's idiom for an accessor's `this` and arguments: `const expected = {that: {}, args: [42]}`.
      if (inits.some((e) => e?.properties?.some((p) => p.key.name === "that"))) skip("api: accessor functions reading this or arguments (Rust takes values, not accessors)");
      if (inits.some((e) => /Function/.test(e?.type ?? "") && e.params.length !== 1)) skip("api: accessor functions (Rust takes values, not accessors)");
      if (!inits.some((e) => e && ours(e, ctx))) return undefined;
      return s.declarations.flatMap((d) => {
        const e = val(ex(d.init, ctx));
        ctx.vars.set(d.id.name, types.includes(e.t) ? { ...e, name: d.id.name } : { ...e, c: snake(d.id.name) });
        return types.includes(e.t) ? [] : [{ declares: snake(d.id.name), c: e.c }];
      });
    },
  };
}
