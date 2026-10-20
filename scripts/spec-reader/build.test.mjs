import { test } from 'node:test';
import assert from 'node:assert/strict';
import { buildPage, loadDocuments, renderDocument, rewriteLink, slug } from './build.mjs';

const source = 'openspec/changes/demo/specs/control/spec.md';
const paths = new Set([source, 'openspec/changes/demo/proposal.md']);
test('links resolve within the reader or back to the repository', () => {
  assert.equal(rewriteLink('../../proposal.md#why', source, paths), '#openspec--changes--demo--proposal--why');
  assert.equal(rewriteLink('#requirement-stop', source, paths), '#openspec--changes--demo--specs--control--spec--requirement-stop');
  assert.equal(rewriteLink('../../../../../docs/SPECIFICATION.md', source, paths), '../docs/SPECIFICATION.md');
  assert.equal(rewriteLink('https://example.com/a#b', source, paths), 'https://example.com/a#b');
});
test('requirements contain scenarios, escaped content and disabled source task boxes', () => {
  const html = renderDocument({ path: source, text: `### Requirement: Stop

The car SHALL stop. <script>alert(1)</script>

#### Scenario: An owner stops

- **WHEN** stop arrives
- **THEN** outputs stop

#### Scenario: An owner stops

- **WHEN** it repeats
- **THEN** it stays stopped

### Requirement: Start

- [ ] Pending
- [x] Done
` }, paths);
  assert.equal((html.match(/class="requirement"/g) ?? []).length, 2);
  assert.equal((html.match(/<details/g) ?? []).length, 2);
  assert.match(html, /<\/details><\/article><article/);
  assert.match(html, /--scenario-an-owner-stops-1/);
  assert.match(html, /&lt;script&gt;/);
  assert.doesNotMatch(html, /<script>/);
  assert.match(html, /disabled aria-label="Recorded task pending"/);
  assert.match(html, /disabled aria-label="Recorded task complete" checked/);
});
test('every current requirement and scenario is included, with unique and reachable anchors', () => {
  const docs = loadDocuments(), html = buildPage(docs);
  for (const [heading, className] of [['### Requirement: ', 'requirement'], ['#### Scenario: ', 'scenario']]) {
    const count = docs.reduce((n, doc) => n + doc.text.split('\n').filter(line => line.startsWith(heading)).length, 0);
    assert.equal((html.match(new RegExp(`class="${className}"`, 'g')) ?? []).length, count);
  }
  const ids = [...html.matchAll(/\bid="([^"]+)"/g)].map(match => match[1]);
  assert.equal(ids.length, new Set(ids).size, 'duplicate anchors');
  for (const [, link] of html.matchAll(/href="#([^"]+)"/g)) assert.ok(ids.includes(link), `broken link #${link}`);
  assert.match(html, /A planned check is not a passing test/);
  assert.match(html, /Task boxes are read-only/);
  assert.equal(html, buildPage(docs), 'generation must be deterministic');
});
test('inline emphasis and code use the same heading slug as source links', () => {
  assert.equal(slug('Requirement: `ARM` / STOP'), 'requirement-arm--stop');
});

test('motion state flow survives archive and links to the baseline when available', () => {
  const baseline = 'openspec/specs/car-control-api/spec.md';
  const change = 'openspec/changes/add-four-wheel-api/specs/car-control-api/spec.md';
  for (const paths of [[baseline], [change], [change, baseline]]) {
    const html = buildPage(paths.map(path => ({ path, text: '## Purpose\nCar control.' })));
    const target = paths.includes(baseline) ? 'openspec--specs--car-control-api--spec' : 'openspec--changes--add-four-wheel-api--specs--car-control-api--spec';
    assert.match(html, /id="state-flow"/, `missing flow for ${paths}`);
    assert.ok(html.includes(`href="#${target}--requirement-boot-state"`), `wrong flow target for ${paths}`);
  }
});
