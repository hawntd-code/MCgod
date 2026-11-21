//! Árbol de una plantilla y de sus expresiones.

use std::rc::Rc;

pub type Body = Rc<[Node]>;

#[derive(Debug, Clone)]
pub enum Node {
    Text(String),
    Interp(Expr, u32),
    If(Vec<(Expr, Body)>, Option<Body>, u32),
    List {
        seq: Expr,
        vars: Vec<String>,
        body: Body,
        else_body: Option<Body>,
        line: u32,
    },
    /// `<#sep>` dentro de un `list`: solo se muestra si hay un siguiente elemento.
    Sep(Body),
    Assign {
        scope: Scope,
        items: Vec<(String, AssignOp, Option<Expr>)>,
        line: u32,
    },
    /// `<#assign x>…</#assign>`: guarda la salida del cuerpo.
    Capture {
        scope: Scope,
        name: String,
        body: Body,
    },
    MacroDef(Rc<Callable>),
    Return(Option<Expr>, u32),
    CallUser {
        callee: Expr,
        args: Args,
        body: Option<Body>,
        line: u32,
    },
    Nested(u32),
    Include(Expr, u32),
    Import(Expr, String, u32),
    Break,
    Continue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Namespace,
    Local,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Set,
    Add,
    Sub,
    Mul,
    Div,
    Inc,
    Dec,
}

#[derive(Debug, Clone)]
pub enum Args {
    Positional(Vec<Expr>),
    Named(Vec<(String, Expr)>),
}

/// Macro o función definida en una plantilla.
#[derive(Debug)]
pub struct Callable {
    pub name: String,
    pub params: Vec<(String, Option<Expr>)>,
    pub catch_all: Option<String>,
    pub body: Body,
    pub is_function: bool,
    pub template: Rc<str>,
}

#[derive(Debug, Clone)]
pub enum StrPart {
    Lit(String),
    Expr(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeKind {
    /// `a..b` incluye `b`
    Inclusive,
    /// `a..<b` o `a..!b`
    Exclusive,
    /// `a..*n`: `n` elementos
    Length,
    /// `a..`
    Open,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Str(Vec<StrPart>),
    Num(f64),
    Bool(bool),
    Var(String),
    Dot(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Builtin(Box<Expr>, String, Option<Vec<Expr>>),
    Default(Box<Expr>, Option<Box<Expr>>),
    Exists(Box<Expr>),
    Not(Box<Expr>),
    Neg(Box<Expr>),
    Bin(BinOp, Box<Expr>, Box<Expr>),
    Range(Box<Expr>, Option<Box<Expr>>, RangeKind),
    Seq(Vec<Expr>),
    Hash(Vec<(Expr, Expr)>),
    Lambda(String, Rc<Expr>),
}
