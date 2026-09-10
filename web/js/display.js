/*
  The one walk.

  The tree, the paths column and the matrix are three drawings of the same
  ordered list of display rows. They are produced here, once, so that a branch
  opening cannot move one of them relative to another.

  A display row is either a node or a group. A **closed** group is one row whose
  members are every node beneath it, so every edge inside the subtree rolls up
  onto that one row: closing a branch summarises it exactly and hides nothing.
  An **open** group is a heading with no members of its own — its rows carry
  themselves.
*/
'use strict';

import { S, caseOk, subtreeNodes, layerRoot, reachFrom } from './state.js';

export function buildDisplay() {
  const rows = [];
  const rootId = layerRoot();
  if (!rootId || !S.G.has(rootId)) return rows;

  const rec = (gid, depth, bars, last) => {
    const g = S.G.get(gid);
    if (!caseOk(g)) return;
    // Siblings read in the order whoever wrote them put them in, not the order
    // their folders happen to sort in. The node table is folder-ordered because
    // generation has to be deterministic; the tree is not a directory listing.
    const own  = (S.gnodes.get(gid) || []).slice()
      .sort((a, b) => (S.rows[a].order - S.rows[b].order) || (a - b));
    const kids = (S.gkids.get(gid) || []).filter(k => caseOk(S.G.get(k)))
      .sort((a, b) => (S.G.get(a).order - S.G.get(b).order) || a.localeCompare(b));
    const has  = own.length > 0 || kids.length > 0;
    const open = has && S.expanded.has(gid);

    rows.push({
      kind: 'group', id: gid, label: g.label, owner: g.owner, tone: g.tone || 'slate',
      depth, bars: bars.slice(), last, hasKids: has, open, box: !!g.box,
      members: open ? [] : subtreeNodes(gid), crosses: '', parent: g.parent,
    });
    if (!open) return;

    // A row at depth d draws d gutter cells: levels 0..d-2 continue an
    // ancestor's line, level d-1 is its own elbow or tee.
    const childBars = depth === 0 ? [] : bars.concat([!last]);
    const items = own.map(i => ({ t: 'n', i })).concat(kids.map(k => ({ t: 'g', k })));
    items.forEach((it, n) => {
      const isLast = n === items.length - 1;
      if (it.t === 'g') { rec(it.k, depth + 1, childBars, isLast); return; }
      const r = S.rows[it.i];
      rows.push({
        kind: 'node', id: r.id, label: r.label, owner: r.owner, tone: g.tone || 'slate',
        depth: depth + 1, bars: childBars.slice(), last: isLast,
        hasKids: false, open: false, box: false,
        members: [it.i], crosses: r.crosses, parent: gid, row: r,
      });
    });
  };

  rec(rootId, 0, [], true);
  return rows;
}

/**
 * Everything the drawings need to know about the selection, computed once.
 *
 * Direct and further are kept apart because they mean different things: one hop
 * is an interface you own, three hops is a consequence you have to be told
 * about.
 */
export function relations(disp) {
  const out = { dOut: new Set(), dIn: new Set(), rOut: new Set(), rIn: new Set(), sel: null };
  const sel = disp.find(d => d.id === S.selected);
  if (!sel) return out;
  out.sel = sel;
  for (const i of sel.members) {
    for (const c of S.consumers[i]) out.dOut.add(c);
    for (const p of S.producers[i]) out.dIn.add(p);
  }
  out.rOut = reachFrom(sel.members, S.consumers);
  out.rIn  = reachFrom(sel.members, S.producers);
  return out;
}

/** The fill a row gets, relative to the selection. Learned once, used in three places. */
export function pillClass(d, rel) {
  if (!rel.sel) return '';
  if (d.id === S.selected) return ' sel';
  const hit = set => d.members.some(i => set.has(i));
  const o = hit(rel.dOut), n = hit(rel.dIn);
  if (o && n) return ' both';
  if (o) return ' out';
  if (n) return ' in';
  const po = hit(rel.rOut), pn = hit(rel.rIn);
  if (po && pn) return ' both';
  if (po) return ' outp';
  if (pn) return ' inp';
  return '';
}

/** The ancestor of display row `i` at gutter level `level`. */
export function ancestorAt(disp, i, level) {
  let d = disp[i];
  while (d && d.depth > level) d = disp.find(x => x.id === d.parent);
  return d ? d.id : '';
}
