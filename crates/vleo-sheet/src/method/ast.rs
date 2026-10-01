//! The method language: the syntax tree.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Pow => "^",
            BinOp::Lt => "<",
            BinOp::Le => "<=",
            BinOp::Gt => ">",
            BinOp::Ge => ">=",
            BinOp::Eq => "==",
            BinOp::Ne => "!=",
            BinOp::And => "and",
            BinOp::Or => "or",
        }
    }
    pub(super) fn is_compare(self) -> bool {
        matches!(
            self,
            BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge | BinOp::Eq | BinOp::Ne
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    /// A literal, already in SI. `bare` is true when no unit was written,
    /// which lets a bare `0` stand for zero of any dimension.
    Num {
        si: f64,
        dim: Dim,
        bare: bool,
        line: usize,
    },
    Bool {
        value: bool,
        line: usize,
    },
    Var {
        name: String,
        line: usize,
    },
    Neg(Box<Expr>),
    Not(Box<Expr>),
    Bin {
        op: BinOp,
        l: Box<Expr>,
        r: Box<Expr>,
        line: usize,
    },
    Call {
        name: String,
        args: Vec<Expr>,
        line: usize,
    },
    /// A table for `interp`, already in SI.
    Table {
        si: Vec<f64>,
        dim: Dim,
        line: usize,
    },
}

impl Expr {
    pub fn line(&self) -> usize {
        match self {
            Expr::Num { line, .. }
            | Expr::Bool { line, .. }
            | Expr::Var { line, .. }
            | Expr::Bin { line, .. }
            | Expr::Call { line, .. }
            | Expr::Table { line, .. } => *line,
            Expr::Neg(e) | Expr::Not(e) => e.line(),
        }
    }
    /// The literal value, if this is one (a sign included).
    pub(super) fn literal(&self) -> Option<f64> {
        match self {
            Expr::Num { si, .. } => Some(*si),
            Expr::Neg(e) => e.literal().map(|v| -v),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        quantity: Option<String>,
        expr: Expr,
        line: usize,
    },
    Const {
        name: String,
        expr: Expr,
        line: usize,
    },
    Set {
        name: String,
        expr: Expr,
        line: usize,
    },
    If {
        arms: Vec<(Expr, Vec<Stmt>)>,
        otherwise: Option<Vec<Stmt>>,
        line: usize,
    },
    For {
        var: String,
        first: i64,
        last: i64,
        body: Vec<Stmt>,
        line: usize,
    },
    Refuse {
        reason: String,
        line: usize,
    },
    Return {
        expr: Expr,
        line: usize,
    },
}

impl Stmt {
    pub fn line(&self) -> usize {
        match self {
            Stmt::Let { line, .. }
            | Stmt::Const { line, .. }
            | Stmt::Set { line, .. }
            | Stmt::If { line, .. }
            | Stmt::For { line, .. }
            | Stmt::Refuse { line, .. }
            | Stmt::Return { line, .. } => *line,
        }
    }
}

/// A parsed method.
#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub body: Vec<Stmt>,
}
