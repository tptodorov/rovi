import { mkdir, rm, copyFile, writeFile } from 'node:fs/promises';
import { execFileSync } from 'node:child_process';

const root = new URL('./', import.meta.url), output = new URL('dist/iphone/', root);
await rm(new URL('dist/', root), { recursive: true, force: true });
await mkdir(output, { recursive: true });
for (const name of ['index.html', 'style.css', 'app.js', 'wire.js', 'hold.js', 'ui.js', 'transport.js', 'session.js']) {
  await copyFile(new URL(name, root), new URL(name, output));
}
const revision = execFileSync('git', ['rev-parse', '--short=12', 'HEAD'], { cwd: root, encoding: 'utf8' }).trim();
const dirty = execFileSync('git', ['status', '--porcelain', '--', '.'], { cwd: root, encoding: 'utf8' }).trim();
await writeFile(new URL('version.txt', output), `${revision}${dirty ? '-dirty' : ''}\n`);
console.log(`Built dist/iphone/ (${revision}${dirty ? '-dirty' : ''}); no deployment performed.`);
