import { readFileSync, writeFileSync, readdirSync, existsSync } from 'node:fs';
import { dirname, resolve, relative, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import MarkdownIt from 'markdown-it';

const here = dirname(fileURLToPath(import.meta.url));
export const root = resolve(here, '../..');
const output = 'openspec/index.html';
const md = new MarkdownIt({ html: false });
const esc = md.utils.escapeHtml;
export const slug = text => text.toLowerCase().replace(/[^\p{L}\p{N}\s_-]/gu, '').replace(/\s/g, '-');
const docId = path => slug(path.replace(/\.md$/, '').replaceAll('/', '--'));

function files(dir) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name, 'en'))
    .flatMap(entry => entry.isDirectory() ? files(join(dir, entry.name)) : entry.name.endsWith('.md') ? [join(dir, entry.name)] : []);
}

export function loadDocuments() {
  return files(join(root, 'openspec')).map(path => ({ path: relative(root, path), text: readFileSync(path, 'utf8') }));
}

// Resolve links into this page when the target document is included; other repo links remain relative.
export function rewriteLink(href, source, documents) {
  if (/^[a-z][a-z\d+.-]*:/i.test(href) || href.startsWith('//')) return href;
  const hashAt = href.indexOf('#');
  const path = hashAt < 0 ? href : href.slice(0, hashAt);
  const fragment = hashAt < 0 ? '' : decodeURIComponent(href.slice(hashAt + 1));
  const target = path ? relative(root, resolve(root, dirname(source), decodeURIComponent(path))) : source;
  if (documents.has(target)) return `#${docId(target)}${fragment ? '--' + fragment : ''}`;
  return relative(resolve(root, dirname(output)), resolve(root, target)).split('/').map(encodeURIComponent).join('/') + (fragment ? '#' + encodeURIComponent(fragment) : '');
}

export function renderDocument(doc, documents) {
  const tokens = md.parse(doc.text, {});
  const headings = new Map();
  let result = '', scenario = false, requirement = false;
  for (let i = 0; i < tokens.length; i++) {
    const token = tokens[i];
    for (const child of token.children ?? []) {
      for (const attr of ['href', 'src']) {
        const href = child.attrGet(attr);
        if (href !== null) child.attrSet(attr, rewriteLink(href, doc.path, documents));
      }
    }
    if (token.type === 'heading_open') {
      const level = Number(token.tag.slice(1));
      const title = tokens[i + 1].content;
      const base = slug(title), count = headings.get(base) ?? 0;
      headings.set(base, count + 1);
      const id = docId(doc.path) + '--' + base + (count ? '-' + count : '');
      if (level <= 4 && scenario) { result += '</details>'; scenario = false; }
      if (level <= 3 && requirement) { result += '</article>'; requirement = false; }
      if (level === 3 && title.startsWith('Requirement: ')) {
        result += `<article class="requirement" id="${id}"><h3>${esc(title.slice(13))}</h3>`;
        requirement = true;
      } else if (level === 4 && title.startsWith('Scenario: ')) {
        result += `<details class="scenario" id="${id}"><summary>${esc(title.slice(10))}</summary>`;
        scenario = true;
      } else {
        result += `<${token.tag} id="${id}">${md.renderInline(title)}</${token.tag}>`;
      }
      i += 2;
    } else {
      result += md.renderer.render([token], md.options, {});
    }
  }
  if (scenario) result += '</details>';
  if (requirement) result += '</article>';
  return result.replace(/(<li>\s*(?:<p>\s*)?)\[([ xX])\] /g, (_, prefix, mark) => `${prefix}<input type="checkbox" disabled aria-label="Recorded task ${mark === ' ' ? 'pending' : 'complete'}"${mark === ' ' ? '' : ' checked'}> `);
}

const sectionText = (text, title) => {
  const parts = text.split(/^## /m);
  return parts.find(part => part.startsWith(title + '\n'))?.slice(title.length).trim() ?? '';
};

export function buildPage(documents = loadDocuments()) {
  const order = doc => doc.path.endsWith('/spec.md') ? 0 : ({ 'design.md': 1, 'tasks.md': 2, 'verification.md': 3, 'proposal.md': 4 }[doc.path.split('/').at(-1)] ?? 5);
  documents = [...documents].sort((a, b) => order(a) - order(b) || a.path.localeCompare(b.path, 'en'));
  const paths = new Set(documents.map(doc => doc.path));
  const proposals = documents.filter(doc => /changes\/[^/]+\/proposal\.md$/.test(doc.path));
  const requirements = documents.reduce((n, doc) => n + (doc.text.match(/^### Requirement: /gm)?.length ?? 0), 0);
  const scenarios = documents.reduce((n, doc) => n + (doc.text.match(/^#### Scenario: /gm)?.length ?? 0), 0);
  const tasks = documents.filter(doc => doc.path.endsWith('/tasks.md'));
  const total = tasks.reduce((n, doc) => n + (doc.text.match(/^- \[[ xX]\]/gm)?.length ?? 0), 0);
  const done = tasks.reduce((n, doc) => n + (doc.text.match(/^- \[[xX]\]/gm)?.length ?? 0), 0);
  const label = doc => {
    const parts = doc.path.split('/');
    if (parts.at(-1) === 'spec.md') return parts.at(-2).replaceAll('-', ' ');
    return parts.at(-1).replace('.md', '');
  };
  const kind = doc => doc.path.includes('/archive/') ? 'Archived change' : doc.path.startsWith('openspec/specs/') ? 'Baseline' : 'Change document';
  const nav = documents.map(doc => `<a href="#${docId(doc.path)}">${esc(label(doc))}<small>${esc(kind(doc))} · ${esc(doc.path.split('/')[2])}</small></a>`).join('');
  const overviews = proposals.map(doc => {
    const status = doc.text.split('\n\n').find(part => part.startsWith('**Status:**')) ?? 'See the proposal for acceptance status.';
    return `<div class="overview"><h2>${esc(doc.text.split('\n')[0].replace(/^# /, ''))}</h2>
      <div class="notice">${renderDocument({ ...doc, text: status }, paths)}</div>
      <h3>Why this matters</h3>${renderDocument({ ...doc, text: sectionText(doc.text, 'Why') }, paths)}
      <h3>What you get</h3>${renderDocument({ ...doc, text: sectionText(doc.text, 'What Changes') }, paths)}
      <a class="button" href="#${docId(doc.path)}${sectionText(doc.text, 'Evidence and remaining decisions') ? '--evidence-and-remaining-decisions' : ''}">Open decisions &amp; evidence →</a></div>`;
  }).join('');
  const body = documents.map(doc => `<section class="document" id="${docId(doc.path)}" aria-label="${esc(label(doc))}">
    <header class="doc-header"><span class="eyebrow">${esc(kind(doc))}</span><a href="${esc(relative(resolve(root, 'openspec'), resolve(root, doc.path)))}">Markdown source ↗</a><div class="path">${esc(doc.path)}</div></header>
    ${doc.path.endsWith('/spec.md') ? `<h2>${esc(label(doc))}</h2>` : ''}
    ${renderDocument(doc, paths)}</section>`).join('');
  const stateDoc = 'openspec/changes/add-four-wheel-api/specs/car-control-api/spec.md';
  let flow = '';
  if (paths.has(stateDoc)) {
    const prefix = '#' + docId(stateDoc) + '--requirement-';
    flow = readFileSync(join(here, 'state-flow.html'), 'utf8').replaceAll('REQUIREMENT:', prefix);
  }
  return readFileSync(join(here, 'page.html'), 'utf8')
    .replace('<!-- STYLE -->', `<style>${readFileSync(join(here, 'reader.css'), 'utf8')}</style>`)
    .replace('<!-- NAV -->', nav)
    .replace('<!-- OVERVIEW -->', overviews || '<p>No active proposals. Browse baseline and archived documents below.</p>')
    .replace('<!-- METRICS -->', `<div><b>${requirements}</b> requirements</div><div><b>${scenarios}</b> scenarios</div><div><b>${done} / ${total}</b> tasks checked in source</div>`)
    .replace('<!-- FLOW -->', flow)
    .replace('<a href="#state-flow">Motion state flow</a>', flow ? '<a href="#state-flow">Motion state flow</a>' : '')
    .replace('<!-- DOCUMENTS -->', body)
    .replace('<!-- SCRIPT -->', `<script>${readFileSync(join(here, 'reader.js'), 'utf8')}</script>`);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const html = buildPage();
  if (process.argv.includes('--check')) {
    if (!existsSync(resolve(root, output)) || readFileSync(resolve(root, output), 'utf8') !== html) {
      console.error('Spec reader is stale. Run npm run spec:build and commit openspec/index.html.');
      process.exitCode = 1;
    } else console.log('Spec reader is current.');
  } else {
    writeFileSync(resolve(root, output), html);
    console.log(`Generated ${output}`);
  }
}
