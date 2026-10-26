// Resolves `d3-*` and `internmap` to their vendored sources under ../d3;
// every other specifier falls through to Node.
export async function resolve(spec, ctx, next) {
  if (/^(d3-[a-z-]+|internmap)$/.test(spec)) {
    return next(new URL(`../d3/${spec}/src/index.js`, import.meta.url).href, ctx);
  }
  return next(spec, ctx);
}
