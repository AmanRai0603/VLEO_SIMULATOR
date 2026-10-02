/*
  Markdown, the part a group writes, turned into the page.

  Headings, paragraphs, lists, quotes, code blocks, tables, emphasis, code
  and links; maths as $…$ and $$…$$; and the four embeds of the group folder
  (groups/SPEC.toml) — {{eq E1}}, {{fig id}}, {{guess q || a}}, {{node id}} —
  handed to the caller, which knows what each one names.

  NO RAW HTML. Every character of the source is escaped before anything is
  built from it, so a folder can put words and pictures on the page and
  nothing else. A tag typed into a file is shown as the text it is.
*/
'use strict';

import { esc } from './dom.js';

/**
 * The file split at its `## ` headings: `{ intro, sections: [{ title, body, line }] }`.
 * Used to check the required headings and to lay the stations out.
 */
export function sections(text) {
  const lines = String(text || '').replace(/\r/g, '').split('\n');
  const out = { intro: '', sections: [] };
  let cur = null, fence = false;
  lines.forEach((l, n) => {
    if (/^\s*(```|~~~)/.test(l)) fence = !fence;
    const m = !fence && /^##\s+(.+?)\s*#*\s*$/.exec(l);
    if (m) { cur = { title: m[1].trim(), body: '', line: n + 1 }; out.sections.push(cur); return; }
    if (cur) cur.body += l + '\n'; else out.intro += l + '\n';
  });
  return out;
}

/**
 * HTML from Markdown. `hooks.math(tex, display)` and `hooks.embed(name, arg, block)`
 * return HTML; anything they do not know they return as escaped text.
 */
export function renderMd(text, hooks = {}) {
  const math = hooks.math || ((t) => '<code>' + esc(t) + '</code>');
  const embed = hooks.embed || ((n, a) => esc('{{' + n + ' ' + a + '}}'));
  const lines = String(text || '').replace(/\r/g, '').split('\n');
  const html = [];
  let i = 0;

  const inline = s => {
    // Pull out the parts that must not be touched by emphasis: code, maths, embeds.
    const keep = [];
    const hold = h => '\u0000' + (keep.push(h) - 1) + '\u0000';
    let t = s.replace(/`([^`]+)`/g, (_, c) => hold('<code>' + esc(c) + '</code>'))
      .replace(/\{\{\s*([a-z]+)\s+([^}]*?)\s*\}\}/g, (_, n, a) => hold(embed(n, a, false)))
      .replace(/\$\$([^$]+)\$\$/g, (_, m) => hold(math(m, true)))
      .replace(/(^|[^\\$])\$([^$\n]+?)\$/g, (_, pre, m) => pre + hold(math(m, false)));
    t = esc(t)
      .replace(/\*\*([^*]+)\*\*/g, '<b>$1</b>')
      .replace(/(^|[^*\w])\*([^*\n]+)\*(?!\w)/g, '$1<i>$2</i>')
      .replace(/(^|[^_\w])_([^_\n]+)_(?!\w)/g, '$1<i>$2</i>')
      .replace(/\[([^\]]+)\]\(([^)\s]+)\)/g, (_, label, href) => {
        const safe = /^(https?:|mailto:|#)/i.test(href.replace(/&amp;/g, '&')) ? href : '#';
        return '<a href="' + safe + '" target="_blank" rel="noopener">' + label + '</a>';
      })
      .replace(/\\\$/g, '$');
    return t.replace(/\u0000(\d+)\u0000/g, (_, k) => keep[Number(k)]);
  };

  while (i < lines.length) {
    const l = lines[i];
    if (/^\s*$/.test(l)) { i++; continue; }
    // A fenced code block, kept exactly.
    let m = /^\s*(```|~~~)\s*([\w-]*)/.exec(l);
    if (m) {
      const fence = m[1], lang = m[2];
      const body = [];
      i++;
      while (i < lines.length && !lines[i].trim().startsWith(fence)) body.push(lines[i++]);
      i++;
      html.push('<pre class="md-code"' + (lang ? ' data-lang="' + esc(lang) + '"' : '') + '><code>' + esc(body.join('\n')) + '</code></pre>');
      continue;
    }
    // Display maths, which may run over several lines.
    if (/^\s*\$\$/.test(l)) {
      let body = l.trim().slice(2);
      if (body.endsWith('$$') && body.length > 1) { html.push(math(body.slice(0, -2), true)); i++; continue; }
      i++;
      while (i < lines.length && !lines[i].includes('$$')) body += '\n' + lines[i++];
      if (i < lines.length) body += '\n' + lines[i].slice(0, lines[i].indexOf('$$'));
      i++;
      html.push(math(body, true));
      continue;
    }
    // An embed on a line of its own is a block: a figure, an equation, a question.
    m = /^\s*\{\{\s*([a-z]+)\s+(.*?)\s*\}\}\s*$/.exec(l);
    if (m) { html.push(embed(m[1], m[2], true)); i++; continue; }
    m = /^(#{1,4})\s+(.+?)\s*#*\s*$/.exec(l);
    if (m) {
      const n = Math.min(6, m[1].length + 1);
      html.push('<h' + n + ' class="md-h">' + inline(m[2]) + '</h' + n + '>');
      i++; continue;
    }
    if (/^\s*>/.test(l)) {
      const body = [];
      while (i < lines.length && /^\s*>/.test(lines[i])) body.push(lines[i++].replace(/^\s*>\s?/, ''));
      html.push('<blockquote>' + renderMd(body.join('\n'), hooks) + '</blockquote>');
      continue;
    }
    if (/^\s*[-*+]\s+/.test(l) || /^\s*\d+[.)]\s+/.test(l)) {
      const ordered = /^\s*\d+[.)]\s+/.test(l);
      const items = [];
      while (i < lines.length && (/^\s*[-*+]\s+/.test(lines[i]) || /^\s*\d+[.)]\s+/.test(lines[i]) ||
        (/^\s{2,}\S/.test(lines[i]) && items.length))) {
        const s = lines[i];
        if (/^\s*([-*+]|\d+[.)])\s+/.test(s) && !/^\s{4,}/.test(s)) items.push(s.replace(/^\s*([-*+]|\d+[.)])\s+/, ''));
        else items[items.length - 1] += ' ' + s.trim();
        i++;
      }
      const tag = ordered ? 'ol' : 'ul';
      html.push('<' + tag + '>' + items.map(it => '<li>' + inline(it) + '</li>').join('') + '</' + tag + '>');
      continue;
    }
    // A table: a header row, a rule of dashes, then rows.
    if (/\|/.test(l) && i + 1 < lines.length && /^\s*\|?\s*:?-{2,}/.test(lines[i + 1])) {
      const cells = s => s.trim().replace(/^\||\|$/g, '').split('|').map(c => c.trim());
      const head = cells(l);
      i += 2;
      const rows = [];
      while (i < lines.length && /\|/.test(lines[i]) && lines[i].trim()) rows.push(cells(lines[i++]));
      html.push('<div class="tablewrap"><table class="md-table"><thead><tr>' + head.map(h => '<th>' + inline(h) + '</th>').join('') +
        '</tr></thead><tbody>' + rows.map(r => '<tr>' + r.map(c => '<td>' + inline(c) + '</td>').join('') + '</tr>').join('') +
        '</tbody></table></div>');
      continue;
    }
    // A paragraph: lines until a blank one or the start of another block.
    const para = [];
    while (i < lines.length && lines[i].trim() && !/^(#{1,4}\s|\s*(```|~~~)|\s*\$\$|\s*>|\s*[-*+]\s|\s*\d+[.)]\s)/.test(lines[i]) &&
      !/^\s*\{\{.*\}\}\s*$/.test(lines[i])) para.push(lines[i++]);
    if (!para.length) { para.push(lines[i++]); }
    html.push('<p>' + inline(para.join(' ')) + '</p>');
  }
  return html.join('\n');
}

/** Sentences of eight words or more, normalised, for the overlap check. */
export function longSentences(text) {
  return String(text || '')
    .replace(/```[\s\S]*?```/g, ' ').replace(/\$\$[\s\S]*?\$\$/g, ' ').replace(/\{\{[^}]*\}\}/g, ' ')
    .split(/(?<=[.!?])\s+|\n\s*\n/)
    .map(s => s.toLowerCase().replace(/[^a-z0-9 ]+/g, ' ').replace(/\s+/g, ' ').trim())
    .filter(s => s.split(' ').length >= 8);
}
