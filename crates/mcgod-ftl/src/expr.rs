//! Lector y analizador de expresiones FreeMarker.

use crate::ast::{BinOp, Expr, RangeKind, StrPart};
use crate::lexer::find_interp_end;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum T {
    Ident(String),
    Num(f64),
    Str(Vec<StrPart>),
    Op(&'static str),
    Eof,
}

const OPS: &[&str] = &[
    "..<", "..!", "..*", "...", "??", "->", "==", "!=", "<=", ">=", "&&", "||", "+=", "-=", "*=",
    "/=", "++", "--", "..", ".", "?", "!", "=", "<", ">", "+", "-", "*", "/", "%", "(", ")", "[",
    "]", "{", "}", ",", ":",
];

pub fn lex(src: &str) -> Result<Vec<T>, String> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if src[i..].starts_with("<#--") {
            let end = src[i..].find("-->").ok_or("comentario <#-- sin cerrar")?;
            i += end + 3;
            continue;
        }
        // Texto crudo r"..."
        if c == 'r' && i + 1 < b.len() && (b[i + 1] == b'"' || b[i + 1] == b'\'') {
            let q = b[i + 1];
            let end = src[i + 2..].find(q as char).ok_or("texto sin cerrar")?;
            out.push(T::Str(vec![StrPart::Lit(
                src[i + 2..i + 2 + end].to_string(),
            )]));
            i += end + 3;
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' || c == '$' {
            let len = src[i..]
                .find(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '$'))
                .unwrap_or(src.len() - i);
            out.push(T::Ident(src[i..i + len].to_string()));
            i += len;
            continue;
        }
        if c.is_ascii_digit() {
            let mut j = i;
            while j < b.len() && b[j].is_ascii_digit() {
                j += 1;
            }
            // Un punto seguido de dígito es decimal; `1..3` es un rango.
            if j + 1 < b.len() && b[j] == b'.' && b[j + 1].is_ascii_digit() {
                j += 1;
                while j < b.len() && b[j].is_ascii_digit() {
                    j += 1;
                }
            }
            out.push(T::Num(src[i..j].parse().map_err(|_| "número inválido")?));
            i = j;
            continue;
        }
        if c == '"' || c == '\'' {
            let (parts, next) = lex_string(src, i)?;
            out.push(T::Str(parts));
            i = next;
            continue;
        }
        let op = OPS
            .iter()
            .find(|op| src[i..].starts_with(**op))
            .ok_or_else(|| format!("carácter inesperado {c:?}"))?;
        out.push(T::Op(op));
        i += op.len();
    }
    out.push(T::Eof);
    Ok(out)
}

fn lex_string(src: &str, start: usize) -> Result<(Vec<StrPart>, usize), String> {
    let b = src.as_bytes();
    let quote = b[start];
    let mut parts = Vec::new();
    let mut lit = String::new();
    let mut i = start + 1;
    while i < b.len() {
        let c = b[i];
        if c == quote {
            if !lit.is_empty() || parts.is_empty() {
                parts.push(StrPart::Lit(lit));
            }
            return Ok((parts, i + 1));
        }
        if c == b'\\' && i + 1 < b.len() {
            let e = b[i + 1] as char;
            match e {
                'n' => lit.push('\n'),
                't' => lit.push('\t'),
                'r' => lit.push('\r'),
                'b' => lit.push('\u{8}'),
                'f' => lit.push('\u{c}'),
                'l' => lit.push('<'),
                'g' => lit.push('>'),
                'a' => lit.push('&'),
                'x' => {
                    let hex: String = src[i + 2..]
                        .chars()
                        .take_while(|c| c.is_ascii_hexdigit())
                        .take(4)
                        .collect();
                    let code = u32::from_str_radix(&hex, 16).map_err(|_| "escape \\x inválido")?;
                    lit.push(char::from_u32(code).unwrap_or('?'));
                    i += 2 + hex.len();
                    continue;
                }
                other => lit.push(other),
            }
            i += 2;
            continue;
        }
        if c == b'$' && i + 1 < b.len() && b[i + 1] == b'{' {
            let end = find_interp_end(src, i + 2).ok_or("${ sin cerrar dentro de un texto")?;
            if !lit.is_empty() {
                parts.push(StrPart::Lit(std::mem::take(&mut lit)));
            }
            parts.push(StrPart::Expr(parse(&src[i + 2..end])?));
            i = end + 1;
            continue;
        }
        let ch = src[i..].chars().next().ok_or("texto inválido")?;
        lit.push(ch);
        i += ch.len_utf8();
    }
    Err("texto sin cerrar".into())
}

pub struct Parser {
    toks: Vec<T>,
    pos: usize,
}

/// Analiza una expresión completa.
pub fn parse(src: &str) -> Result<Expr, String> {
    let mut p = Parser::new(src)?;
    let e = p.expr()?;
    p.expect_eof()?;
    Ok(e)
}

impl Parser {
    pub fn new(src: &str) -> Result<Parser, String> {
        Ok(Parser {
            toks: lex(src)?,
            pos: 0,
        })
    }

    pub fn peek(&self) -> &T {
        &self.toks[self.pos]
    }

    pub fn peek_at(&self, n: usize) -> &T {
        self.toks.get(self.pos + n).unwrap_or(&T::Eof)
    }

    pub fn advance(&mut self) -> T {
        let t = self.toks[self.pos].clone();
        if self.pos + 1 < self.toks.len() {
            self.pos += 1;
        }
        t
    }

    pub fn at_eof(&self) -> bool {
        matches!(self.peek(), T::Eof)
    }

    pub fn eat_op(&mut self, op: &str) -> bool {
        if matches!(self.peek(), T::Op(o) if *o == op) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn is_op(&self, op: &str) -> bool {
        matches!(self.peek(), T::Op(o) if *o == op)
    }

    pub fn is_ident(&self, name: &str) -> bool {
        matches!(self.peek(), T::Ident(i) if i == name)
    }

    pub fn ident(&mut self) -> Result<String, String> {
        match self.advance() {
            T::Ident(i) => Ok(i),
            other => Err(format!("se esperaba un nombre y llegó {other:?}")),
        }
    }

    pub fn expect_op(&mut self, op: &str) -> Result<(), String> {
        if self.eat_op(op) {
            Ok(())
        } else {
            Err(format!("se esperaba {op:?} y llegó {:?}", self.peek()))
        }
    }

    pub fn expect_eof(&self) -> Result<(), String> {
        if self.at_eof() {
            Ok(())
        } else {
            Err(format!("sobra texto en la expresión: {:?}", self.peek()))
        }
    }

    pub fn expr(&mut self) -> Result<Expr, String> {
        self.bp(0)
    }

    fn infix(&self) -> Option<(BinOpOrRange, u8)> {
        let op = match self.peek() {
            T::Op(o) => *o,
            T::Ident(i) => match i.as_str() {
                "gt" => ">",
                "gte" => ">=",
                "lt" => "<",
                "lte" => "<=",
                _ => return None,
            },
            _ => return None,
        };
        Some(match op {
            "||" => (BinOpOrRange::Bin(BinOp::Or), 1),
            "&&" => (BinOpOrRange::Bin(BinOp::And), 2),
            "==" | "=" => (BinOpOrRange::Bin(BinOp::Eq), 3),
            "!=" => (BinOpOrRange::Bin(BinOp::Ne), 3),
            "<" => (BinOpOrRange::Bin(BinOp::Lt), 4),
            "<=" => (BinOpOrRange::Bin(BinOp::Le), 4),
            ">" => (BinOpOrRange::Bin(BinOp::Gt), 4),
            ">=" => (BinOpOrRange::Bin(BinOp::Ge), 4),
            ".." => (BinOpOrRange::Range(RangeKind::Inclusive), 5),
            "..<" | "..!" => (BinOpOrRange::Range(RangeKind::Exclusive), 5),
            "..*" => (BinOpOrRange::Range(RangeKind::Length), 5),
            "+" => (BinOpOrRange::Bin(BinOp::Add), 6),
            "-" => (BinOpOrRange::Bin(BinOp::Sub), 6),
            "*" => (BinOpOrRange::Bin(BinOp::Mul), 7),
            "/" => (BinOpOrRange::Bin(BinOp::Div), 7),
            "%" => (BinOpOrRange::Bin(BinOp::Mod), 7),
            _ => return None,
        })
    }

    fn bp(&mut self, min: u8) -> Result<Expr, String> {
        let mut lhs = self.unary()?;
        while let Some((op, bp)) = self.infix() {
            if bp <= min {
                break;
            }
            self.advance();
            lhs = match op {
                BinOpOrRange::Bin(op) => Expr::Bin(op, Box::new(lhs), Box::new(self.bp(bp)?)),
                BinOpOrRange::Range(kind) => {
                    if kind == RangeKind::Inclusive && !self.starts_operand() {
                        Expr::Range(Box::new(lhs), None, RangeKind::Open)
                    } else {
                        Expr::Range(Box::new(lhs), Some(Box::new(self.bp(bp)?)), kind)
                    }
                }
            };
        }
        Ok(lhs)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.eat_op("!") {
            return Ok(Expr::Not(Box::new(self.unary()?)));
        }
        if self.eat_op("-") {
            return Ok(Expr::Neg(Box::new(self.unary()?)));
        }
        if self.eat_op("+") {
            return self.unary();
        }
        self.postfix()
    }

    /// ¿El siguiente token puede empezar un operando? (para `x!` y `a..`)
    fn starts_operand(&self) -> bool {
        match self.peek() {
            T::Num(_) | T::Str(_) => true,
            T::Ident(i) => !matches!(
                i.as_str(),
                "as" | "gt" | "gte" | "lt" | "lte" | "using" | "in"
            ),
            T::Op(o) => matches!(*o, "(" | "[" | "{"),
            T::Eof => false,
        }
    }

    /// Nombre de una llamada `<@…>`: una expresión con accesos, llamadas y built-ins.
    pub fn callee(&mut self) -> Result<Expr, String> {
        self.postfix()
    }

    fn postfix(&mut self) -> Result<Expr, String> {
        let mut e = self.primary()?;
        loop {
            if self.eat_op(".") {
                let name = self.ident()?;
                e = Expr::Dot(Box::new(e), name);
            } else if self.eat_op("[") {
                let idx = self.expr()?;
                self.expect_op("]")?;
                e = Expr::Index(Box::new(e), Box::new(idx));
            } else if self.is_op("(") {
                self.advance();
                let args = self.args(")")?;
                e = Expr::Call(Box::new(e), args);
            } else if self.eat_op("??") {
                e = Expr::Exists(Box::new(e));
            } else if self.is_op("?") {
                self.advance();
                let name = self.ident()?;
                let args = if self.is_op("(") {
                    self.advance();
                    Some(self.args(")")?)
                } else {
                    None
                };
                e = Expr::Builtin(Box::new(e), name, args);
            } else if self.is_op("!") {
                self.advance();
                let default = if self.starts_operand() {
                    Some(Box::new(self.postfix_operand()?))
                } else {
                    None
                };
                return Ok(Expr::Default(Box::new(e), default));
            } else {
                return Ok(e);
            }
        }
    }

    /// Operando del operador por defecto: un valor con sus accesos, sin operadores binarios.
    fn postfix_operand(&mut self) -> Result<Expr, String> {
        self.postfix()
    }

    fn args(&mut self, close: &str) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if self.eat_op(close) {
            return Ok(args);
        }
        loop {
            args.push(self.expr()?);
            if self.eat_op(close) {
                return Ok(args);
            }
            // La coma entre argumentos es opcional en FreeMarker (FTL.jj: `[<COMMA>]`).
            self.eat_op(",");
            if self.at_eof() {
                return Err(format!("falta {close:?}"));
            }
        }
    }

    fn primary(&mut self) -> Result<Expr, String> {
        match self.advance() {
            T::Num(n) => Ok(Expr::Num(n)),
            T::Str(parts) => Ok(Expr::Str(parts)),
            T::Ident(name) => match name.as_str() {
                "true" => Ok(Expr::Bool(true)),
                "false" => Ok(Expr::Bool(false)),
                _ if self.is_op("->") => {
                    self.advance();
                    let body = self.expr()?;
                    Ok(Expr::Lambda(name, Rc::new(body)))
                }
                _ => Ok(Expr::Var(name)),
            },
            T::Op("(") => {
                // `(x) -> …`
                if let (T::Ident(name), T::Op(")"), T::Op("->")) = (
                    self.peek().clone(),
                    self.peek_at(1).clone(),
                    self.peek_at(2).clone(),
                ) {
                    self.advance();
                    self.advance();
                    self.advance();
                    let body = self.expr()?;
                    return Ok(Expr::Lambda(name, Rc::new(body)));
                }
                let e = self.expr()?;
                self.expect_op(")")?;
                Ok(e)
            }
            T::Op("[") => Ok(Expr::Seq(self.args("]")?)),
            T::Op("{") => {
                let mut pairs = Vec::new();
                if self.eat_op("}") {
                    return Ok(Expr::Hash(pairs));
                }
                loop {
                    let k = self.expr()?;
                    self.expect_op(":")?;
                    let v = self.expr()?;
                    pairs.push((k, v));
                    if self.eat_op("}") {
                        return Ok(Expr::Hash(pairs));
                    }
                    self.expect_op(",")?;
                }
            }
            other => Err(format!("expresión inesperada: {other:?}")),
        }
    }
}

enum BinOpOrRange {
    Bin(BinOp),
    Range(RangeKind),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(src: &str) -> String {
        format!("{:?}", parse(src).unwrap())
    }

    #[test]
    fn precedence() {
        assert!(p("1 + 2 * 3").starts_with("Bin(Add, Num(1.0), Bin(Mul"));
        assert!(p("a && b || c").starts_with("Bin(Or, Bin(And"));
        assert!(p("!a && b").starts_with("Bin(And, Not("));
        assert!(p("x gt 3 && y lte 2").starts_with("Bin(And, Bin(Gt"));
    }

    #[test]
    fn postfix_chains() {
        let s = p("data.getModels()?filter(e -> e.ok)?has_content");
        assert!(s.starts_with("Builtin(Builtin(Call(Dot(Var(\"data\"), \"getModels\"), [])"));
        assert!(s.contains("Lambda(\"e\""));
    }

    #[test]
    fn default_and_exists() {
        assert!(p("a.b!\"x\"").starts_with("Default(Dot"));
        assert!(p("(a.b)!").starts_with("Default(Dot"));
        assert!(p("a??").starts_with("Exists(Var"));
        assert!(p("a != b").starts_with("Bin(Ne"));
    }

    #[test]
    fn ranges_and_literals() {
        assert!(p("0..<3").starts_with("Range(Num(0.0), Some(Num(3.0)), Exclusive"));
        assert!(p("1..").starts_with("Range(Num(1.0), None, Open"));
        assert!(p("[1, 'a', {\"k\": true}]").starts_with("Seq([Num(1.0)"));
        assert!(p("1.5").starts_with("Num(1.5)"));
    }

    #[test]
    fn strings_with_interpolation_and_escapes() {
        let s = p(r#""a${x + 1}b\n""#);
        assert!(
            s.contains("Lit(\"a\")") && s.contains("Expr(Bin(Add") && s.contains("Lit(\"b\\n\")")
        );
        assert!(p(r#"r"C:\path\${x}""#).contains("C:\\\\path\\\\${x}"));
    }

    #[test]
    fn errors_are_reported() {
        assert!(parse("a +").is_err());
        assert!(parse("(a").is_err());
        assert!(parse("\"abc").is_err());
    }
}
