import { mkdir, rm, readFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';

const root = new URL('./', import.meta.url), output = new URL('dist/iphone/', root);
const revision = execFileSync('git', ['rev-parse', '--short=12', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim();
const dirty = execFileSync('git', ['status', '--porcelain', '--', '.'], { cwd: root, encoding: 'utf8' }).trim();
const version = `${revision}${dirty ? '-dirty' : ''}`;
const assetName = name => name.replace(/\.(js|css)$/, `.${version}.$1`);
await rm(new URL('dist/', root), { recursive: true, force: true });
await mkdir(output, { recursive: true });
for (const name of ['index.html', 'style.css', 'app.js', 'wire.js', 'hold.js', 'ui.js', 'transport.js', 'session.js',
  'sensor-test.html', 'sensor-test.css', 'sensor-test.js', 'sensor-model.js', 'angles.js', 'physical.js']) {
  let body = await readFile(new URL(name, root), 'utf8');
  // Pin every module edge as well as entry assets: cached old imports cannot mix releases.
  if (name.endsWith('.js')) body = body.replace(/\bfrom\s*(['"])(\.\/[^'"]+\.js)\1/g, (_, quote, path) => `from ${quote}${assetName(path)}${quote}`);
  if (name.endsWith('.html')) body = body.replace(/(src|href)="([^"?]+\.(?:js|css))"/g, (_, attribute, path) => `${attribute}="${assetName(path)}"`).replace('content="source-checkout"', `content="${version}"`);
  await writeFile(new URL(assetName(name), output), body);
}
await writeFile(new URL('version.txt', output), `${version}\n`);
console.log(`Built dist/iphone/ (${version}); no deployment performed.`);
