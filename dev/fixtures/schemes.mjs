// Every d3-scale-chromatic 3.1.0 interpolator at t = 0, 0.1, …, 1, as
// d3.rgb(…).formatRgb(), keyed by rustd3's snake-case name:
//   node --import ./dev/harness/translate/d3-resolve.mjs dev/fixtures/schemes.mjs
// writes dev/fixtures/schemes.json, sorted.
import { writeFileSync } from "node:fs";
import { rgb } from "d3-color";
import * as chromatic from "d3-scale-chromatic";

const snake = (name) => name.replace(/([a-z0-9])([A-Z])/g, "$1_$2").replace(/([A-Z])([A-Z][a-z])/g, "$1_$2").toLowerCase();
const schemes = Object.keys(chromatic)
  .filter((k) => k.startsWith("interpolate"))
  .map((k) => [snake(k.slice("interpolate".length)), chromatic[k]])
  .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0));
const samples = Object.fromEntries(schemes.map(([name, f]) => [name, Array.from({ length: 11 }, (_, i) => rgb(f(i / 10)).formatRgb())]));
writeFileSync(new URL("schemes.json", import.meta.url), `${JSON.stringify(samples, null, 2)}\n`);
