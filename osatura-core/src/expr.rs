// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! A small expression language for procedural / driven channels.
//!
//! ```text
//! 20 * sin(time * 2 * pi)                 // oscillate
//! get("arm_up", "bone_angle") * 0.5       // follow another value (driver)
//! frame > 24 ? 10 : 0                     // conditional
//! wiggle(3, 5, 7) + clamp(time, 0, 1)     // noise + functions
//! ```
//! Variables: `frame`, `time` (seconds), `fps`, `frames`, `pi`, `t` (0..1 over the clip).
//! Operators: `+ - * / % ^`, comparisons, `&& || !`, `?:`.

use crate::noise::{fbm1, noise1};

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f32),
    Ident(String),
    Str(String),
    Sym(&'static str),
}

const SYMS: [&str; 17] = [
    "<=", ">=", "==", "!=", "&&", "||", "+", "-", "*", "/", "%", "^", "(", ")", ",", "<", ">",
];
const SYMS1: [&str; 3] = ["!", "?", ":"];

fn lex(src: &str) -> Result<Vec<Tok>, String> {
    let b: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    'outer: while i < b.len() {
        let c = b[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c.is_ascii_digit() || (c == '.' && i + 1 < b.len() && b[i + 1].is_ascii_digit()) {
            let s = i;
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == '.') {
                i += 1;
            }
            if i < b.len() && (b[i] == 'e' || b[i] == 'E') {
                let mut j = i + 1;
                if j < b.len() && (b[j] == '+' || b[j] == '-') {
                    j += 1;
                }
                if j < b.len() && b[j].is_ascii_digit() {
                    while j < b.len() && b[j].is_ascii_digit() {
                        j += 1;
                    }
                    i = j;
                }
            }
            let t: String = b[s..i].iter().collect();
            out.push(Tok::Num(t.parse().map_err(|_| format!("bad number '{t}'"))?));
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            let s = i;
            while i < b.len() && (b[i].is_alphanumeric() || b[i] == '_') {
                i += 1;
            }
            out.push(Tok::Ident(b[s..i].iter().collect()));
            continue;
        }
        if c == '"' || c == '\'' {
            let q = c;
            i += 1;
            let s = i;
            while i < b.len() && b[i] != q {
                i += 1;
            }
            if i >= b.len() {
                return Err("unterminated string".into());
            }
            out.push(Tok::Str(b[s..i].iter().collect()));
            i += 1;
            continue;
        }
        let rest: String = b[i..b.len().min(i + 2)].iter().collect();
        for s in SYMS.iter().chain(SYMS1.iter()) {
            if rest.starts_with(s) {
                out.push(Tok::Sym(s));
                i += s.len();
                continue 'outer;
            }
        }
        return Err(format!("unexpected character '{c}'"));
    }
    Ok(out)
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Var {
    Frame,
    Time,
    Fps,
    Frames,
    Pi,
    T01,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Bin {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
    And,
    Or,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Func {
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
    Abs,
    Sqrt,
    Pow,
    Min,
    Max,
    Clamp,
    Lerp,
    Smoothstep,
    Floor,
    Ceil,
    Round,
    Fract,
    Sign,
    Mod,
    Exp,
    Ln,
    Noise,
    Fbm,
    Wiggle,
    Saw,
    Tri,
    Square,
    Deg,
    Rad,
}

const FUNCS: &[(&str, Func, usize, usize)] = &[
    ("sin", Func::Sin, 1, 1),
    ("cos", Func::Cos, 1, 1),
    ("tan", Func::Tan, 1, 1),
    ("asin", Func::Asin, 1, 1),
    ("acos", Func::Acos, 1, 1),
    ("atan", Func::Atan, 1, 1),
    ("atan2", Func::Atan2, 2, 2),
    ("abs", Func::Abs, 1, 1),
    ("sqrt", Func::Sqrt, 1, 1),
    ("pow", Func::Pow, 2, 2),
    ("min", Func::Min, 2, 2),
    ("max", Func::Max, 2, 2),
    ("clamp", Func::Clamp, 3, 3),
    ("lerp", Func::Lerp, 3, 3),
    ("smoothstep", Func::Smoothstep, 3, 3),
    ("floor", Func::Floor, 1, 1),
    ("ceil", Func::Ceil, 1, 1),
    ("round", Func::Round, 1, 1),
    ("fract", Func::Fract, 1, 1),
    ("sign", Func::Sign, 1, 1),
    ("mod", Func::Mod, 2, 2),
    ("exp", Func::Exp, 1, 1),
    ("ln", Func::Ln, 1, 1),
    ("noise", Func::Noise, 1, 2),
    ("fbm", Func::Fbm, 2, 3),
    ("wiggle", Func::Wiggle, 2, 3),
    ("saw", Func::Saw, 1, 1),
    ("tri", Func::Tri, 1, 1),
    ("square", Func::Square, 1, 1),
    ("deg", Func::Deg, 1, 1),
    ("rad", Func::Rad, 1, 1),
];

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Num(f32),
    Var(VarRef),
    Neg(Box<Expr>),
    Not(Box<Expr>),
    Bin(BinRef, Box<Expr>, Box<Expr>),
    Cond(Box<Expr>, Box<Expr>, Box<Expr>),
    Call(FuncRef, Vec<Expr>),
    /// `get("target", "prop")` — current value of another animatable value.
    Get(String, String),
}

// Opaque wrappers keep the private enums out of the public API.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VarRef(Var);
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BinRef(Bin);
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FuncRef(Func);

pub struct Ctx<'a> {
    pub frame: f32,
    pub fps: f32,
    pub frames: f32,
    pub get: &'a dyn Fn(&str, &str) -> f32,
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
    /// Current nesting of parentheses, unary operators and conditionals (see [`MAX_DEPTH`]).
    depth: usize,
}

/// Longest expression source accepted, in bytes.
pub const MAX_SOURCE_BYTES: usize = 4096;
/// Deepest nesting (parentheses, unary operators, `?:`, function arguments) accepted.
pub const MAX_DEPTH: usize = 64;

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }
    fn eat(&mut self, s: &str) -> bool {
        if matches!(self.peek(), Some(Tok::Sym(x)) if *x == s) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, s: &str) -> Result<(), String> {
        if self.eat(s) {
            Ok(())
        } else {
            Err(format!("expected '{s}'"))
        }
    }

    fn enter(&mut self) -> Result<(), String> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(format!("expression is nested more than {MAX_DEPTH} levels deep"));
        }
        Ok(())
    }

    fn ternary(&mut self) -> Result<Expr, String> {
        self.enter()?;
        let r = self.ternary_inner();
        self.depth -= 1;
        r
    }

    fn ternary_inner(&mut self) -> Result<Expr, String> {
        let c = self.or()?;
        if self.eat("?") {
            let a = self.ternary()?;
            self.expect(":")?;
            let b = self.ternary()?;
            return Ok(Expr::Cond(Box::new(c), Box::new(a), Box::new(b)));
        }
        Ok(c)
    }
    fn or(&mut self) -> Result<Expr, String> {
        let mut l = self.and()?;
        while self.eat("||") {
            l = Expr::Bin(BinRef(Bin::Or), Box::new(l), Box::new(self.and()?));
        }
        Ok(l)
    }
    fn and(&mut self) -> Result<Expr, String> {
        let mut l = self.cmp()?;
        while self.eat("&&") {
            l = Expr::Bin(BinRef(Bin::And), Box::new(l), Box::new(self.cmp()?));
        }
        Ok(l)
    }
    fn cmp(&mut self) -> Result<Expr, String> {
        let mut l = self.add()?;
        loop {
            let op = if self.eat("<=") {
                Bin::Le
            } else if self.eat(">=") {
                Bin::Ge
            } else if self.eat("==") {
                Bin::Eq
            } else if self.eat("!=") {
                Bin::Ne
            } else if self.eat("<") {
                Bin::Lt
            } else if self.eat(">") {
                Bin::Gt
            } else {
                return Ok(l);
            };
            l = Expr::Bin(BinRef(op), Box::new(l), Box::new(self.add()?));
        }
    }
    fn add(&mut self) -> Result<Expr, String> {
        let mut l = self.mul()?;
        loop {
            let op = if self.eat("+") {
                Bin::Add
            } else if self.eat("-") {
                Bin::Sub
            } else {
                return Ok(l);
            };
            l = Expr::Bin(BinRef(op), Box::new(l), Box::new(self.mul()?));
        }
    }
    fn mul(&mut self) -> Result<Expr, String> {
        let mut l = self.unary()?;
        loop {
            let op = if self.eat("*") {
                Bin::Mul
            } else if self.eat("/") {
                Bin::Div
            } else if self.eat("%") {
                Bin::Mod
            } else {
                return Ok(l);
            };
            l = Expr::Bin(BinRef(op), Box::new(l), Box::new(self.unary()?));
        }
    }
    fn unary(&mut self) -> Result<Expr, String> {
        self.enter()?;
        let r = self.unary_inner();
        self.depth -= 1;
        r
    }

    fn unary_inner(&mut self) -> Result<Expr, String> {
        if self.eat("-") {
            return Ok(Expr::Neg(Box::new(self.unary()?)));
        }
        if self.eat("+") {
            return self.unary();
        }
        if self.eat("!") {
            return Ok(Expr::Not(Box::new(self.unary()?)));
        }
        self.pow()
    }
    fn pow(&mut self) -> Result<Expr, String> {
        let base = self.atom()?;
        if self.eat("^") {
            let exp = self.unary()?; // right-associative, allows 2^-1
            return Ok(Expr::Bin(BinRef(Bin::Pow), Box::new(base), Box::new(exp)));
        }
        Ok(base)
    }
    fn atom(&mut self) -> Result<Expr, String> {
        match self.toks.get(self.pos).cloned() {
            Some(Tok::Num(n)) => {
                self.pos += 1;
                Ok(Expr::Num(n))
            }
            Some(Tok::Sym("(")) => {
                self.pos += 1;
                let e = self.ternary()?;
                self.expect(")")?;
                Ok(e)
            }
            Some(Tok::Ident(name)) => {
                self.pos += 1;
                if self.eat("(") {
                    return self.call(&name);
                }
                let v = match name.as_str() {
                    "frame" => Var::Frame,
                    "time" => Var::Time,
                    "fps" => Var::Fps,
                    "frames" => Var::Frames,
                    "pi" => Var::Pi,
                    "t" => Var::T01,
                    _ => return Err(format!("unknown variable '{name}'")),
                };
                Ok(Expr::Var(VarRef(v)))
            }
            Some(t) => Err(format!("unexpected token {t:?}")),
            None => Err("unexpected end of expression".into()),
        }
    }
    fn call(&mut self, name: &str) -> Result<Expr, String> {
        if name == "get" {
            let a = match self.toks.get(self.pos).cloned() {
                Some(Tok::Str(s)) => s,
                _ => return Err("get() needs string arguments: get(\"target\", \"prop\")".into()),
            };
            self.pos += 1;
            self.expect(",")?;
            let b = match self.toks.get(self.pos).cloned() {
                Some(Tok::Str(s)) => s,
                _ => return Err("get() needs string arguments: get(\"target\", \"prop\")".into()),
            };
            self.pos += 1;
            self.expect(")")?;
            return Ok(Expr::Get(a, b));
        }
        let &(_, f, lo, hi) = FUNCS
            .iter()
            .find(|(n, ..)| *n == name)
            .ok_or_else(|| format!("unknown function '{name}'"))?;
        let mut args = Vec::new();
        if !self.eat(")") {
            loop {
                args.push(self.ternary()?);
                if self.eat(")") {
                    break;
                }
                self.expect(",")?;
            }
        }
        if args.len() < lo || args.len() > hi {
            return Err(format!("{name}() takes {lo}..{hi} arguments, got {}", args.len()));
        }
        Ok(Expr::Call(FuncRef(f), args))
    }
}

pub fn parse(src: &str) -> Result<Expr, String> {
    if src.len() > MAX_SOURCE_BYTES {
        return Err(format!("expression is {} bytes long, the limit is {MAX_SOURCE_BYTES}", src.len()));
    }
    let toks = lex(src)?;
    if toks.is_empty() {
        return Err("empty expression".into());
    }
    let mut p = Parser { toks, pos: 0, depth: 0 };
    let e = p.ternary()?;
    if p.pos != p.toks.len() {
        return Err(format!("unexpected trailing token {:?}", p.toks[p.pos]));
    }
    Ok(e)
}

fn b(v: bool) -> f32 {
    if v {
        1.0
    } else {
        0.0
    }
}

impl Expr {
    pub fn eval(&self, c: &Ctx) -> f32 {
        match self {
            Expr::Num(n) => *n,
            Expr::Var(VarRef(v)) => match v {
                Var::Frame => c.frame,
                Var::Time => c.frame / c.fps,
                Var::Fps => c.fps,
                Var::Frames => c.frames,
                Var::Pi => std::f32::consts::PI,
                Var::T01 => {
                    if c.frames > 0.0 {
                        c.frame / c.frames
                    } else {
                        0.0
                    }
                }
            },
            Expr::Neg(e) => -e.eval(c),
            Expr::Not(e) => b(e.eval(c) == 0.0),
            Expr::Cond(k, a, e) => {
                if k.eval(c) != 0.0 {
                    a.eval(c)
                } else {
                    e.eval(c)
                }
            }
            Expr::Get(t, p) => (c.get)(t, p),
            Expr::Bin(BinRef(op), l, r) => {
                let (x, y) = (l.eval(c), r.eval(c));
                match op {
                    Bin::Add => x + y,
                    Bin::Sub => x - y,
                    Bin::Mul => x * y,
                    Bin::Div => {
                        if y == 0.0 {
                            0.0
                        } else {
                            x / y
                        }
                    }
                    Bin::Mod => {
                        if y == 0.0 {
                            0.0
                        } else {
                            x.rem_euclid(y)
                        }
                    }
                    Bin::Pow => x.powf(y),
                    Bin::Lt => b(x < y),
                    Bin::Gt => b(x > y),
                    Bin::Le => b(x <= y),
                    Bin::Ge => b(x >= y),
                    Bin::Eq => b(x == y),
                    Bin::Ne => b(x != y),
                    Bin::And => b(x != 0.0 && y != 0.0),
                    Bin::Or => b(x != 0.0 || y != 0.0),
                }
            }
            Expr::Call(FuncRef(f), args) => {
                let a: Vec<f32> = args.iter().map(|e| e.eval(c)).collect();
                let v = match f {
                    Func::Sin => a[0].sin(),
                    Func::Cos => a[0].cos(),
                    Func::Tan => a[0].tan(),
                    Func::Asin => a[0].clamp(-1.0, 1.0).asin(),
                    Func::Acos => a[0].clamp(-1.0, 1.0).acos(),
                    Func::Atan => a[0].atan(),
                    Func::Atan2 => a[0].atan2(a[1]),
                    Func::Abs => a[0].abs(),
                    Func::Sqrt => a[0].max(0.0).sqrt(),
                    Func::Pow => a[0].powf(a[1]),
                    Func::Min => a[0].min(a[1]),
                    Func::Max => a[0].max(a[1]),
                    Func::Clamp => a[0].clamp(a[1].min(a[2]), a[1].max(a[2])),
                    Func::Lerp => a[0] + (a[1] - a[0]) * a[2],
                    Func::Smoothstep => {
                        let d = a[1] - a[0];
                        let t = if d == 0.0 { 1.0 } else { ((a[2] - a[0]) / d).clamp(0.0, 1.0) };
                        t * t * (3.0 - 2.0 * t)
                    }
                    Func::Floor => a[0].floor(),
                    Func::Ceil => a[0].ceil(),
                    Func::Round => a[0].round(),
                    Func::Fract => a[0] - a[0].floor(),
                    Func::Sign => {
                        if a[0] > 0.0 {
                            1.0
                        } else if a[0] < 0.0 {
                            -1.0
                        } else {
                            0.0
                        }
                    }
                    Func::Mod => {
                        if a[1] == 0.0 {
                            0.0
                        } else {
                            a[0].rem_euclid(a[1])
                        }
                    }
                    Func::Exp => a[0].exp(),
                    Func::Ln => a[0].max(1e-30).ln(),
                    Func::Noise => noise1(a[0], a.get(1).copied().unwrap_or(0.0) as u32),
                    Func::Fbm => fbm1(a[0], a.get(2).copied().unwrap_or(0.0) as u32, a[1].max(1.0) as u32),
                    Func::Wiggle => {
                        noise1(c.frame / c.fps * a[0], a.get(2).copied().unwrap_or(0.0) as u32) * a[1]
                    }
                    Func::Saw => a[0] - a[0].floor(),
                    Func::Tri => 1.0 - ((a[0] - a[0].floor()) * 2.0 - 1.0).abs() * 2.0 + 0.0,
                    Func::Square => {
                        if (a[0] - a[0].floor()) < 0.5 {
                            1.0
                        } else {
                            -1.0
                        }
                    }
                    Func::Deg => a[0].to_degrees(),
                    Func::Rad => a[0].to_radians(),
                };
                if v.is_finite() {
                    v
                } else {
                    0.0
                }
            }
        }
    }

    /// All `get("target","prop")` references (for dependency checks).
    pub fn gets(&self, out: &mut Vec<(String, String)>) {
        match self {
            Expr::Get(t, p) => out.push((t.clone(), p.clone())),
            Expr::Neg(e) | Expr::Not(e) => e.gets(out),
            Expr::Bin(_, l, r) => {
                l.gets(out);
                r.gets(out);
            }
            Expr::Cond(a, b, c) => {
                a.gets(out);
                b.gets(out);
                c.gets(out);
            }
            Expr::Call(_, args) => args.iter().for_each(|e| e.gets(out)),
            Expr::Num(_) | Expr::Var(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    // ---- stack safety (S4): these used to overflow the stack and abort the process ----

    #[test]
    fn deep_nesting_is_an_error_not_a_stack_overflow() {
        let parens = format!("{}1{}", "(".repeat(20_000), ")".repeat(20_000));
        let e = parse(&parens).unwrap_err();
        assert!(e.contains("limit") || e.contains("nested"), "{e}");
        let minus = format!("{}1", "-".repeat(200_000));
        assert!(parse(&minus).is_err());
        let nots = format!("{}1", "!".repeat(1_000));
        assert!(parse(&nots).is_err());
        let calls = format!("{}1{}", "abs(".repeat(5_000), ")".repeat(5_000));
        assert!(parse(&calls).is_err());
        let cond = format!("{}1{}", "1 ? ".repeat(1_000), " : 0".repeat(1_000));
        assert!(parse(&cond).is_err());
    }

    #[test]
    fn long_source_is_refused_and_ordinary_depth_still_works() {
        let long = format!("1{}", " + 1".repeat(2_000));
        assert!(long.len() > MAX_SOURCE_BYTES);
        assert!(parse(&long).unwrap_err().contains("bytes long"));
        let ok = format!("{}1{}", "(".repeat(30), ")".repeat(30));
        assert!(parse(&ok).is_ok());
        assert!(parse(&format!("{}1", "-".repeat(30))).is_ok());
        assert!(parse("sin(pi * t) * 10 + (1 - (2 - (3 - 4)))").is_ok());
    }

    use super::*;

    fn ev(src: &str) -> f32 {
        let e = parse(src).unwrap_or_else(|m| panic!("{src}: {m}"));
        let get = |t: &str, p: &str| if t == "x" && p == "prop" { 7.0 } else { 0.0 };
        e.eval(&Ctx { frame: 12.0, fps: 24.0, frames: 48.0, get: &get })
    }

    #[test]
    fn arithmetic_precedence_and_power() {
        assert_eq!(ev("1 + 2 * 3"), 7.0);
        assert_eq!(ev("(1 + 2) * 3"), 9.0);
        assert_eq!(ev("-2 ^ 2"), -4.0);
        assert_eq!(ev("2 ^ 3 ^ 2"), 512.0);
        assert_eq!(ev("2 ^ -1"), 0.5);
        assert_eq!(ev("7 % 4"), 3.0);
        assert_eq!(ev("1 / 0"), 0.0, "division by zero is defined as 0");
    }

    #[test]
    fn variables_functions_and_conditionals() {
        assert_eq!(ev("frame"), 12.0);
        assert_eq!(ev("time"), 0.5);
        assert_eq!(ev("t"), 0.25);
        assert!((ev("sin(pi / 2)") - 1.0).abs() < 1e-6);
        assert_eq!(ev("clamp(5, 0, 3)"), 3.0);
        assert_eq!(ev("lerp(10, 20, 0.25)"), 12.5);
        assert_eq!(ev("frame > 10 ? 1 : 2"), 1.0);
        assert_eq!(ev("frame > 10 && frame < 11 ? 1 : 2"), 2.0);
        assert_eq!(ev("!0"), 1.0);
        assert_eq!(ev("min(3, 4) + max(3, 4)"), 7.0);
    }

    #[test]
    fn get_reads_other_values() {
        assert_eq!(ev("get(\"x\", \"prop\") * 2"), 14.0);
        let mut v = Vec::new();
        parse("get('a','b') + get(\"c\",\"d\")").unwrap().gets(&mut v);
        assert_eq!(v.len(), 2);
    }

    #[test]
    fn noise_functions_are_bounded() {
        let v = ev("wiggle(3, 5, 7)");
        assert!(v.abs() <= 5.0);
        assert!(ev("noise(1.5)").abs() <= 1.0);
    }

    #[test]
    fn errors_are_reported() {
        assert!(parse("").is_err());
        assert!(parse("1 +").is_err());
        assert!(parse("foo(1)").is_err());
        assert!(parse("sin(1, 2)").is_err());
        assert!(parse("bar").is_err());
        assert!(parse("get(1, 2)").is_err());
        assert!(parse("1 2").is_err());
        assert!(parse("3 $ 4").is_err());
    }
}
