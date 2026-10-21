import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFile, readdir, access } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

test('published HTML and its transitive module graph pin one revision to prevent mixed cached releases', async () => {
  execFileSync(process.execPath, [fileURLToPath(new URL('../build.mjs', import.meta.url))]);
  const output = new URL('../dist/iphone/', import.meta.url);
  const version = (await readFile(new URL('version.txt', output), 'utf8')).trim();
  let edges = 0;
  for (const name of await readdir(output)) {
    if (!/\.(js|html)$/.test(name)) continue;
    const body = await readFile(new URL(name, output), 'utf8');
    if (name.endsWith('.html')) assert.ok(body.includes(`<meta name="rovi-build" content="${version}">`));
    const pattern = name.endsWith('.js') ? /\bfrom\s*['"]([^'"]+)['"]/g : /(?:src|href)="([^"]+\.(?:js|css)(?:\?[^"]*)?)"/g;
    for (const [, target] of body.matchAll(pattern)) {
      if (!target.startsWith('.') && name.endsWith('.js')) continue;
      const url = new URL(target, `https://example.test/iphone/${name}`);
      assert.ok(url.pathname.endsWith(`.${version}.js`) || url.pathname.endsWith(`.${version}.css`), `${name} -> ${target}`);
      await access(new URL(url.pathname.replace('/iphone/', ''), output)); edges++;
    }
  }
  assert.ok(edges > 10, 'Check the transitive module graph, not just entry scripts');
});
