// Release builds only: copies the app's pages (src/) to dist/ with every script and style
// sheet minified, so the shipped app's code is hard to read and a little faster to load.
// The source in the repo and `npm run dev` are untouched. Run by CI before `tauri build`.
import { cp, readdir, readFile, rm, stat, writeFile } from 'node:fs/promises';
import { join, extname } from 'node:path';
import { transform } from 'esbuild';

const SRC = 'src';
const OUT = 'dist';

async function walk(dir) {
  const out = [];
  for (const name of await readdir(dir)) {
    const p = join(dir, name);
    if ((await stat(p)).isDirectory()) out.push(...(await walk(p)));
    else out.push(p);
  }
  return out;
}

await rm(OUT, { recursive: true, force: true });
await cp(SRC, OUT, { recursive: true });
let before = 0;
let after = 0;
for (const file of await walk(OUT)) {
  const ext = extname(file);
  if (ext !== '.js' && ext !== '.css') continue;
  const code = await readFile(file, 'utf8');
  const res = await transform(code, {
    loader: ext === '.css' ? 'css' : 'js',
    minify: true,
    format: ext === '.js' ? 'esm' : undefined,
    target: ext === '.js' ? 'es2022' : undefined,
    legalComments: 'none',
    charset: 'utf8',
  });
  before += code.length;
  after += res.code.length;
  await writeFile(file, res.code);
}
console.log(`protect: ${Math.round(before / 1024)} KB of scripts and styles → ${Math.round(after / 1024)} KB in ${OUT}/`);
