// Runs d3 from the vendored sources, offline and without node_modules:
//   node --import ./dev/harness/translate/d3-resolve.mjs script.mjs
// Registers d3-resolve-hooks.mjs, which resolves the bare d3 module names.
import { register } from "node:module";
register("./d3-resolve-hooks.mjs", import.meta.url);
