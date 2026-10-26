const root = new URL("../d3-src/", import.meta.url);
export async function resolve(spec, ctx, next) {
  if (/^(d3-[a-z-]+|internmap)$/.test(spec)) return next(new URL(`${spec}/src/index.js`, root).href, ctx);
  return next(spec, ctx);
}
