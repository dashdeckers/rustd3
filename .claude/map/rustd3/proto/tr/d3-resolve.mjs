// Usage: node --import ./d3-resolve.mjs script.mjs
// Resolves bare d3 module names (and internmap) to the vendored sources
// in ../d3-src, so d3 runs offline without node_modules.
import { register } from "node:module";
register("./d3-resolve-hooks.mjs", import.meta.url);
