import https from 'node:https';
import { readFile, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, extname } from 'node:path';
import { execFileSync } from 'node:child_process';

const temp = await mkdtemp(join(tmpdir(), 'rovi-https-'));
execFileSync('openssl', ['req', '-x509', '-newkey', 'rsa:2048', '-nodes', '-days', '1',
  '-keyout', join(temp, 'key'), '-out', join(temp, 'cert'), '-subj', '/CN=localhost'], { stdio: 'ignore' });
const assets = new Set(['index.html', 'style.css', 'wire.js', 'hold.js', 'ui.js', 'app.js', 'session.js', 'transport.js', 'version.txt',
  'sensor-test.html', 'sensor-test.css', 'sensor-test.js', 'sensor-model.js', 'angles.js', 'physical.js']);
const assetsRoot = new URL(process.env.ROVI_IPHONE_DIST ? '../dist/iphone/' : '../', import.meta.url);
const server = https.createServer({ key: await readFile(join(temp, 'key')), cert: await readFile(join(temp, 'cert')) }, async (req, res) => {
  const name = new URL(req.url, 'https://localhost').pathname.slice(1) || 'index.html';
  const original = name.replace(/\.[a-f0-9]{12,40}(?:-dirty)?\.(js|css)$/, '.$1');
  if (!assets.has(original)) { res.writeHead(404); res.end(); return; }
  try {
    // Test fixtures import unversioned modules; still serve the actual built code.
    const version = process.env.ROVI_IPHONE_DIST ? (await readFile(new URL('version.txt', assetsRoot), 'utf8')).trim() : null;
    const filename = version && name === original ? name.replace(/\.(js|css)$/, `.${version}.$1`) : name;
    const body = await readFile(new URL(filename, assetsRoot));
    res.setHeader('Content-Type', ({ '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css' })[extname(name)] ?? 'text/plain');
    res.end(body);
  } catch { res.writeHead(404); res.end(); }
});
server.listen(8177, '127.0.0.1');
for (const signal of ['SIGINT', 'SIGTERM']) process.on(signal, async () => {
  server.close(); await rm(temp, { recursive: true, force: true }); process.exit();
});
