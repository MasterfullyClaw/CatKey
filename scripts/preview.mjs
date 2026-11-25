// Serve the UI in a normal browser with a fake backend (src/mock.js) for quick design tweaks.
// Usage: npm run preview   then open http://localhost:5178  (?theme=light&skin=kuro&extras=1&nap=1)
import { createServer } from 'node:http';
import { readFile } from 'node:fs/promises';
import { extname, join, normalize } from 'node:path';

const root = new URL('../src/', import.meta.url).pathname.replace(/^\/(\w:)/, '$1');
const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.woff2': 'font/woff2', '.png': 'image/png', '.svg': 'image/svg+xml' };
createServer(async (req, res) => {
  const path = normalize(decodeURIComponent(new URL(req.url, 'http://x').pathname)).replace(/^([\\/])+/, '') || 'index.html';
  try {
    const body = await readFile(join(root, path));
    res.writeHead(200, { 'content-type': types[extname(path)] || 'application/octet-stream' });
    res.end(body);
  } catch {
    res.writeHead(404);
    res.end('not found');
  }
}).listen(5178, () => console.log('Cat Key preview: http://localhost:5178'));
