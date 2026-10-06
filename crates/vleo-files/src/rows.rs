//! A file's rows, as bytes: how the page hands the library what its SQLite
//! read, and how the library answers.
//!
//! The page has its own SQLite (web/vendor/sqlite) and the library has none in
//! WebAssembly, so a file crosses between them as its tables. Each table is
//! written once, in a length-prefixed form `web/js/files.js` writes and reads
//! the same way:
//!
//! ```text
//! "VLEOROWS1"  u32 tables
//! per table:   str name  u32 columns  str column…  u32 rows
//! per row:     cell…     one per column
//! cell:        u8 0 (null) | u8 1, i64 | u8 2, str | u8 3, u32 len, bytes
//! str:         u32 len, UTF-8 bytes
//! ```
//!
//! Every number is little-endian. Nothing is optional and nothing is guessed:
//! bytes that are not exactly this are refused, by name.

use crate::error::{Error, ErrorKind};
use crate::model::{Cell, Table};

const MAGIC: &[u8] = b"VLEOROWS1";

fn put_u32(out: &mut Vec<u8>, n: usize) {
    out.extend_from_slice(&(n as u32).to_le_bytes());
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    put_u32(out, s.len());
    out.extend_from_slice(s.as_bytes());
}

/// Tables as bytes.
pub fn encode(tables: &[Table]) -> Vec<u8> {
    let mut out = MAGIC.to_vec();
    put_u32(&mut out, tables.len());
    for t in tables {
        put_str(&mut out, &t.name);
        put_u32(&mut out, t.columns.len());
        for c in &t.columns {
            put_str(&mut out, c);
        }
        put_u32(&mut out, t.rows.len());
        for row in &t.rows {
            for cell in row {
                match cell {
                    Cell::Null => out.push(0),
                    Cell::Int(i) => {
                        out.push(1);
                        out.extend_from_slice(&i.to_le_bytes());
                    }
                    Cell::Text(s) => {
                        out.push(2);
                        put_str(&mut out, s);
                    }
                    Cell::Blob(b) => {
                        out.push(3);
                        put_u32(&mut out, b.len());
                        out.extend_from_slice(b);
                    }
                }
            }
        }
    }
    out
}

/// A reader over bytes that refuses to run past their end.
pub struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

fn short() -> Error {
    Error::new(ErrorKind::Malformed, "the rows end before they say they do")
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Reader<'a> {
        Reader { bytes, at: 0 }
    }

    pub fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        let end = self.at.checked_add(n).ok_or_else(short)?;
        let s = self.bytes.get(self.at..end).ok_or_else(short)?;
        self.at = end;
        Ok(s)
    }

    pub fn u32(&mut self) -> Result<usize, Error> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]) as usize)
    }

    pub fn str(&mut self) -> Result<String, Error> {
        let n = self.u32()?;
        String::from_utf8(self.take(n)?.to_vec())
            .map_err(|_| Error::new(ErrorKind::Malformed, "a text in the rows is not UTF-8"))
    }

    /// A length-prefixed run of bytes.
    pub fn bytes(&mut self) -> Result<&'a [u8], Error> {
        let n = self.u32()?;
        self.take(n)
    }

    pub fn done(&self) -> bool {
        self.at == self.bytes.len()
    }
}

/// Tables from bytes [`encode`] wrote, refused if they are anything else.
pub fn decode(bytes: &[u8]) -> Result<Vec<Table>, Error> {
    let mut r = Reader::new(bytes);
    if r.take(MAGIC.len()).ok() != Some(MAGIC) {
        return Err(Error::new(
            ErrorKind::Malformed,
            "these are not a file's rows: they do not begin VLEOROWS1",
        ));
    }
    let n = r.u32()?;
    let mut tables = Vec::new();
    for _ in 0..n {
        let name = r.str()?;
        let ncols = r.u32()?;
        let columns = (0..ncols).map(|_| r.str()).collect::<Result<Vec<_>, _>>()?;
        let nrows = r.u32()?;
        let mut rows = Vec::new();
        for _ in 0..nrows {
            let mut row = Vec::with_capacity(ncols);
            for _ in 0..ncols {
                row.push(match r.take(1)?[0] {
                    0 => Cell::Null,
                    1 => {
                        let b = r.take(8)?;
                        Cell::Int(i64::from_le_bytes(b.try_into().map_err(|_| short())?))
                    }
                    2 => Cell::Text(r.str()?),
                    3 => Cell::Blob(r.bytes()?.to_vec()),
                    t => {
                        return Err(Error::new(
                            ErrorKind::Malformed,
                            format!("a cell of {name} is of kind {t}, which the rows do not have"),
                        ))
                    }
                });
            }
            rows.push(row);
        }
        tables.push(Table {
            name,
            columns,
            rows,
        });
    }
    if !r.done() {
        return Err(Error::new(
            ErrorKind::Malformed,
            "the rows go on after their last table",
        ));
    }
    Ok(tables)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_of_cell_comes_back_as_it_went() {
        let t = vec![Table {
            name: "t".into(),
            columns: vec!["a".into(), "b".into(), "c".into(), "d".into()],
            rows: vec![vec![
                Cell::Null,
                Cell::Int(-7),
                Cell::Text("é, \"x\"".into()),
                Cell::Blob(vec![0, 255]),
            ]],
        }];
        let bytes = encode(&t);
        assert_eq!(decode(&bytes).unwrap(), t);
        assert!(decode(&bytes[..bytes.len() - 1]).is_err(), "cut short");
        let mut more = bytes.clone();
        more.push(0);
        assert!(decode(&more).is_err(), "run on");
        assert!(decode(b"VLEOROWS2\0\0\0\0").is_err(), "another format");
    }
}
