import * as fs from 'fs';
import * as path from 'path';

// Regenerates src/data/{dict1_data,patterns_data}.rs from an upstream
// pinyin-pro JS checkout (packages/pinyin-pro/lib/data/*.ts).
// Usage: node scripts/gen_data.mjs (run from a tree containing both).

const root = new URL('..', import.meta.url).pathname;
const libData = path.join(root, 'packages/pinyin-pro/lib/data');
const outDir = path.join(root, 'src/data');
fs.mkdirSync(outDir, { recursive: true });

const rs = (s) => `"${s.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`;

// ---- dict1: invert pinyin -> [chars] to char -> pinyin ----
const dict1Src = fs.readFileSync(path.join(libData, 'dict1.ts'), 'utf8');
// keys may be quoted ('bǎng páng pāng') or bare (líng)
const entryRe = /^[ \t]*(?:'([^']*)'|([^\s:'"]+?))\s*:\s*\[([\s\S]*?)\]/gm;
const charMap = new Map();
let m, count = 0;
while ((m = entryRe.exec(dict1Src)) !== null) {
  const pinyin = m[1] ?? m[2];
  const body = m[3];
  const chars = [...body.matchAll(/'([^']*)'/g)].map((x) => x[1]);
  for (const ch of chars) {
    if (!charMap.has(ch)) charMap.set(ch, pinyin);
    count++;
  }
}
let out = `// AUTO-GENERATED from packages/pinyin-pro/lib/data/dict1.ts — do not edit.\n`;
out += `// char -> pinyin (space-separated readings, first is most common)\n`;
out += `pub static DICT1_ENTRIES: &[(&str, &str)] = &[\n`;
for (const [ch, py] of charMap) {
  out += `(${rs(ch)}, ${rs(py)}),\n`;
}
out += `];\n`;
fs.writeFileSync(path.join(outDir, 'dict1_data.rs'), out);
console.log(`dict1: ${charMap.size} chars, ${count} assignments`);

// ---- word dicts ----
function parseWordDict(file) {
  const src = fs.readFileSync(path.join(libData, file), 'utf8');
  const lines = src.split('\n');
  const entries = [];
  for (const line of lines) {
    const t = line.trim();
    if (!t || t.startsWith('import') || t.startsWith('export') || t.startsWith('}') || t.startsWith('{') || t.startsWith('//') || t.startsWith('*')) continue;
    const mm = t.match(/^(.+?)\s*:\s*(?:'([^']*)'|"([^"]*)")/);
    if (mm) entries.push([mm[1].trim(), mm[2] ?? mm[3]]);
  }
  return entries;
}

const d2 = parseWordDict('dict2.ts');
const d3 = parseWordDict('dict3.ts');
const d4 = parseWordDict('dict4.ts');
const d5 = parseWordDict('dict5.ts');
const surnames = parseWordDict('surname.ts');
console.log(`dict2=${d2.length} dict3=${d3.length} dict4=${d4.length} dict5=${d5.length} surnames=${surnames.length}`);

let p = `// AUTO-GENERATED from packages/pinyin-pro/lib/data/{dict2,dict3,dict4,dict5,surname}.ts — do not edit.\n`;
p += `use crate::segmentit::RawPattern;\n\n`;
const emit = (name, entries, prob, prio) => {
  p += `pub static ${name}: &[RawPattern] = &[\n`;
  for (const [zh, py] of entries) {
    const len = [...zh].length;
    p += `RawPattern { zh: ${rs(zh)}, pinyin: ${rs(py)}, prob: ${prob}, len: ${len}, priority: ${prio} },\n`;
  }
  p += `];\n`;
};
emit('PATTERNS5', d5, '2e-8', 1);
emit('PATTERNS4', d4, '2e-8', 1);
emit('PATTERNS3', d3, '2e-8', 1);
emit('PATTERNS2', d2, '2e-8', 1);
emit('PATTERNS_SURNAME', surnames, null, 10);
// surname prob is 1.0 + len; handle with placeholder then fix
p = p.replace(/prob: null/g, 'prob: 0.0');
p += `\n// Surname probability in JS is Probability.Surname(1.0) + stringLength(key).\n`;
p += `pub fn surname_prob(len: usize) -> f64 { 1.0 + len as f64 }\n`;
fs.writeFileSync(path.join(outDir, 'patterns_data.rs'), p);

// sanity: check dup keys across dicts
const seen = new Map();
for (const [arr, n] of [[d2, 'd2'], [d3, 'd3'], [d4, 'd4'], [d5, 'd5']]) {
  for (const [zh] of arr) {
    if (seen.has(zh)) console.log(`dup: ${zh} in ${seen.get(zh)} and ${n}`);
    seen.set(zh, n);
  }
}
console.log('done');
