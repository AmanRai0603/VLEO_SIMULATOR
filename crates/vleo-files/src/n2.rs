//! The N2 of a block, and the loops of a design, drawn from its files' wires.
//!
//! docs/SYSTEM_MODEL.md, section 5. The N2 of a block is the matrix of its
//! children: each child on the diagonal, a mark where one feeds another. It
//! is drawn from the wires, never separately: a mark is there because a row
//! inside one child reads a port of a row inside the other.
//!
//! A loop is rows that read each other round a circle. It belongs to the
//! smallest block that holds every row in it, is declared there, and an
//! undeclared one is refused by name. [`Design::loops`] finds every loop the
//! wires make, the block each belongs on, and whether that block declares it.

use std::collections::{BTreeMap, BTreeSet};

use crate::error::{Error, ErrorKind};
use crate::meta::Kind;
use crate::model::File;

/// A design's blocks, how they contain each other and which rows feed which:
/// read from its group and node files.
pub struct Design {
    /// Each block's parent, across files: a group's top heading hangs from
    /// the block it mounts on.
    parent: BTreeMap<String, String>,
    /// Each block's place among its siblings.
    ord: BTreeMap<String, i64>,
    /// The rows: blocks with a node file of their own.
    rows: BTreeSet<String>,
    /// Row to row: the first's answer is read by the second.
    feeds: BTreeSet<(String, String)>,
    /// The loops declared, each on its block, with the rows it runs through.
    declared: Vec<(String, BTreeSet<String>)>,
}

/// The N2 of one block.
#[derive(Debug, PartialEq, Eq)]
pub struct N2 {
    pub block: String,
    /// Its children, in their order, on the diagonal.
    pub children: Vec<String>,
    /// `(i, j)`: something inside child `i` feeds something inside child `j`.
    pub marks: BTreeSet<(usize, usize)>,
}

/// A loop the wires make.
#[derive(Debug, PartialEq, Eq)]
pub struct Found {
    /// The rows it runs through.
    pub rows: BTreeSet<String>,
    /// The smallest block that holds them all: where it belongs.
    pub belongs_on: String,
    /// Whether that block declares a loop through exactly these rows.
    pub declared: bool,
}

fn malformed(why: impl Into<String>) -> Error {
    Error::new(ErrorKind::Malformed, why)
}

impl Design {
    /// The design that `files` are: group files and node files.
    pub fn of(files: &[(String, File)]) -> Result<Design, Error> {
        let mut d = Design {
            parent: BTreeMap::new(),
            ord: BTreeMap::new(),
            rows: BTreeSet::new(),
            feeds: BTreeSet::new(),
            declared: Vec::new(),
        };
        let mut mounted: BTreeMap<String, String> = BTreeMap::new();
        for (_, f) in files {
            for m in &f.mounts {
                mounted.insert(m.group_id.clone(), m.block_uid.clone());
            }
        }
        let mut wires: Vec<(String, String)> = Vec::new();
        for (path, f) in files {
            let kind = f.kind()?;
            if !matches!(kind, Kind::Group | Kind::Node) {
                continue;
            }
            let group = f.meta.get("group_id").cloned().unwrap_or_default();
            for b in &f.blocks {
                let parent = if b.parent_uid.is_empty() && kind == Kind::Group {
                    mounted.get(&group).cloned().unwrap_or_default()
                } else {
                    b.parent_uid.clone()
                };
                if d.parent.insert(b.uid.clone(), parent).is_some() {
                    return Err(malformed(format!("{path}: {} is in two files", b.uid)));
                }
                d.ord.insert(b.uid.clone(), b.ord);
                if kind == Kind::Node {
                    d.rows.insert(b.uid.clone());
                }
            }
            for w in &f.wires {
                // `<block>.<port>` in its own group, `<group>.<block>.<port>`
                // in another: the block is the one before the port.
                let mut parts = w.from_ref.rsplit('.');
                let (_, Some(from)) = (parts.next(), parts.next()) else {
                    return Err(malformed(format!(
                        "{path}: {}.{} is wired from {:?}, which names no block",
                        w.to_block, w.to_port, w.from_ref
                    )));
                };
                wires.push((from.to_string(), w.to_block.clone()));
            }
            for l in &f.loops {
                let rows = l
                    .members
                    .split(',')
                    .filter(|m| !m.is_empty())
                    .map(str::to_string)
                    .collect();
                d.declared.push((l.block_uid.clone(), rows));
            }
        }
        for (from, to) in wires {
            if !d.rows.contains(&from) {
                return Err(malformed(format!("{to} reads {from}, which is no row")));
            }
            d.feeds.insert((from, to));
        }
        Ok(d)
    }

    /// Every block that holds `block`, from its parent to the top.
    fn ancestors(&self, block: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut at = block;
        while let Some(p) = self.parent.get(at).filter(|p| !p.is_empty()) {
            if out.contains(p) {
                break;
            }
            out.push(p.clone());
            at = p;
        }
        out
    }

    /// The child of `block` that holds `row`, or none if `block` does not.
    fn child_holding(&self, block: &str, row: &str) -> Option<String> {
        let mut at = row.to_string();
        loop {
            let p = self.parent.get(&at)?;
            if p == block {
                return Some(at);
            }
            at = p.clone();
        }
    }

    /// The N2 of `block`: its children, and a mark wherever one feeds another.
    pub fn n2(&self, block: &str) -> N2 {
        let mut children: Vec<&String> = self
            .parent
            .iter()
            .filter(|(_, p)| p.as_str() == block)
            .map(|(c, _)| c)
            .collect();
        children.sort_by_key(|c| (self.ord.get(*c).copied().unwrap_or(0), (*c).clone()));
        let index: BTreeMap<&str, usize> = children
            .iter()
            .enumerate()
            .map(|(i, c)| (c.as_str(), i))
            .collect();
        let mut marks = BTreeSet::new();
        for (from, to) in &self.feeds {
            let (Some(a), Some(b)) = (
                self.child_holding(block, from),
                self.child_holding(block, to),
            ) else {
                continue;
            };
            if a != b {
                marks.insert((index[a.as_str()], index[b.as_str()]));
            }
        }
        N2 {
            block: block.to_string(),
            children: children.into_iter().cloned().collect(),
            marks,
        }
    }

    /// Every loop the wires make: rows that read each other round a circle,
    /// each with the smallest block that holds it and whether that block
    /// declares it.
    pub fn loops(&self) -> Vec<Found> {
        // Tarjan's strongly connected components, without recursion.
        let rows: Vec<&String> = self.rows.iter().collect();
        let at: BTreeMap<&str, usize> = rows
            .iter()
            .enumerate()
            .map(|(i, r)| (r.as_str(), i))
            .collect();
        let mut next: Vec<Vec<usize>> = vec![Vec::new(); rows.len()];
        for (from, to) in &self.feeds {
            if let (Some(&a), Some(&b)) = (at.get(from.as_str()), at.get(to.as_str())) {
                next[a].push(b);
            }
        }
        let n = rows.len();
        let (mut index, mut low) = (vec![usize::MAX; n], vec![0; n]);
        let mut on_stack = vec![false; n];
        let (mut stack, mut found, mut counter) = (Vec::new(), Vec::new(), 0);
        for start in 0..n {
            if index[start] != usize::MAX {
                continue;
            }
            let mut work: Vec<(usize, usize)> = vec![(start, 0)];
            index[start] = counter;
            low[start] = counter;
            counter += 1;
            stack.push(start);
            on_stack[start] = true;
            while let Some(&(v, i)) = work.last() {
                if i < next[v].len() {
                    let w = next[v][i];
                    if let Some(top) = work.last_mut() {
                        top.1 += 1;
                    }
                    if index[w] == usize::MAX {
                        index[w] = counter;
                        low[w] = counter;
                        counter += 1;
                        stack.push(w);
                        on_stack[w] = true;
                        work.push((w, 0));
                    } else if on_stack[w] {
                        low[v] = low[v].min(index[w]);
                    }
                    continue;
                }
                work.pop();
                if let Some(&(u, _)) = work.last() {
                    low[u] = low[u].min(low[v]);
                }
                if low[v] == index[v] {
                    let mut scc = BTreeSet::new();
                    while let Some(w) = stack.pop() {
                        on_stack[w] = false;
                        scc.insert(rows[w].clone());
                        if w == v {
                            break;
                        }
                    }
                    let self_loop = next[v].contains(&v);
                    if scc.len() > 1 || self_loop {
                        found.push(scc);
                    }
                }
            }
        }
        found
            .into_iter()
            .map(|rows| {
                let belongs_on = self.smallest_holding(&rows);
                let declared = self
                    .declared
                    .iter()
                    .any(|(on, members)| *on == belongs_on && *members == rows);
                Found {
                    rows,
                    belongs_on,
                    declared,
                }
            })
            .collect()
    }

    /// The smallest block that holds every one of `rows`.
    pub fn smallest_holding(&self, rows: &BTreeSet<String>) -> String {
        let mut common: Option<Vec<String>> = None;
        for r in rows {
            let mut chain = self.ancestors(r);
            chain.reverse();
            common = Some(match common {
                None => chain,
                Some(prev) => prev
                    .into_iter()
                    .zip(chain)
                    .take_while(|(a, b)| a == b)
                    .map(|(a, _)| a)
                    .collect(),
            });
        }
        common.and_then(|c| c.last().cloned()).unwrap_or_default()
    }
}
