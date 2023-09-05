#![no_std]
use log::info;
extern crate alloc;
use alloc::{boxed::Box, format, string::String, vec::Vec};
use core::{ops::Rem, result::Result};

pub type ID = String;
pub type BN = Box<N>;
pub type VN = Vec<N>;

macro_rules! bx {
    ($e:expr) => {
        Box::new($e)
    };
}

/// fomoscript AST node
#[derive(Debug, Clone)]
pub enum N {
    FuncCall {
        func: BN,
        args: VN,
    },
    Block(VN),
    If {
        condition: BN,
        path_true: BN,
        path_false: BN,
    },
    While {
        condition: BN,
        body: BN,
    },
    Set(ID, BN),
    Get(ID),
    Unary(Op, BN),
    Binary(Op, BN, BN),
    //Terminal nodes, the following nodes can be output by eval
    FuncDef {
        args_name: Vec<ID>,
        scope: BN,
    },
    FuncNativeDef(Native),
    Array(VN),
    Num(f64),
    Str(String),
    Unit,
}

///Native rust closure wrapper, to be inserted in the script
#[derive(Clone)]
pub struct Native(pub alloc::rc::Rc<dyn Fn(N, N, N, N) -> N>);

impl core::fmt::Debug for Native {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Native")
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Op {
    Mul = 0,
    Div = 1,
    Equals,
    NotEquals,
    Lesser,
    Greater,
    Modulus,
    And,
    Or,
    Plus,
    Plus2,
    Shift,
    Minus,
    Assign,
}
impl Op {
    fn term_separate(self) -> bool {
        self as u8 > 1
    }
}

impl N {
    pub fn as_f64(&self) -> f64 {
        match self {
            N::Num(x) => *x,
            _ => 0.0,
        }
    }
    ///Cast to to boolean. The equivalent of js: self == true
    pub fn to_bool(&self) -> bool {
        match self {
            N::Num(x) if *x != 0.0 => true,
            N::Str(s) => !s.is_empty(),
            N::Array(vec) => !vec.is_empty(),
            _ => false,
        }
    }
    pub fn to_str(&self) -> String {
        match self {
            N::Num(x) => format!("{}", x),
            N::Str(s) => s.clone(),
            e => format!("{:?}", e),
        }
    }
}

/// Token generated while lexing the code.
///
/// Consumed to produce the AST.
#[derive(Debug, Clone)]
enum Token {
    BlockStart,
    BlockEnd,
    If,
    Else,
    Comma,
    ParStart,
    ParEnd,
    While,
    Quoted(String),
    Bin(Op),
    N(N),
    Let(ID),
    Err(String),
    Assoc,
    ArrayStart,
    ArrayEnd,
}

/// Interpreter context, holds all state during execution.
pub struct Ctx {
    pub values: Vec<N>,
    pub idents: Vec<String>,
    pub code: Vec<char>,
    pub deep: usize,
}

impl Ctx {
    pub fn new() -> Ctx {
        Ctx {
            values: Vec::new(),
            idents: Vec::new(),
            code: Vec::new(),
            deep: 0,
        }
    }

    #[inline(always)]
    pub fn drain(&mut self, from: usize) {
        self.values.drain(from..);
        self.idents.drain(from..);
    }

    #[inline(always)]
    pub fn set_val(&mut self, name: &str, n: N) {
        self.idents.push(String::from(name));
        self.values.push(n);
    }

    #[inline(always)]
    pub fn set_val_absolute(&mut self, i: usize, name: &str, n: N) {
        self.idents[i] = String::from(name);
        self.values[i] = n;
    }
    /// Find a variable declared in the scope, or any parent scope
    ///
    /// returns the path and the variable value
    pub fn find_var(&mut self, name: &str) -> Option<(usize, &mut N)> {
        for (i, id) in self.idents.iter().enumerate().rev() {
            if id == name {
                return Some((i, &mut self.values[i]));
            }
        }
        info!("Unknown variable {}", name);
        None
    }

    pub fn insert_code(&mut self, code: &str) {
        self.code.extend(code.chars());
    }

    pub fn parse_next_expr(&mut self) -> Result<N, Error> {
        let mut i = 0;
        let res = parse_expr(&mut i, &self.code, 0)?;
        self.code.drain(0..i);
        Ok(res)
    }
}

impl Default for Ctx {
    fn default() -> Self {
        Ctx::new()
    }
}

pub fn parse_eval(code: &str) -> N {
    let mut ctx = Ctx::new();
    ctx.insert_code(code);
    let mut res = N::Unit;
    while let Ok(mut parent) = ctx.parse_next_expr() {
        res = eval(&mut parent, &mut ctx);
    }
    res
}

fn bool_n(b: bool) -> N {
    N::Num(if b { 1.0 } else { 0.0 })
}

///Interprets the node using the ctx/interpreter provided
pub fn eval(n: &N, ctx: &mut Ctx) -> N {
    ctx.deep += 1;
    if log::log_enabled!(log::Level::Info) {
        info!("\n{}eval {:?}", pa(ctx.deep), n);
        for i in 0..ctx.idents.len() {
            info!(
                "{}{} {:?}:{:?}",
                pa(ctx.deep),
                i,
                ctx.idents[i],
                ctx.values[i]
            );
        }
    }
    let res = match n {
        N::If {
            condition,
            path_true,
            path_false,
        } => match eval(condition, ctx).to_bool() {
            true => eval(path_true, ctx),
            false => eval(path_false, ctx),
        },
        N::While { condition, body } => {
            let mut res = N::Unit;
            while eval(condition, ctx).to_bool() {
                res = eval(body, ctx)
            }
            res
        }
        N::Block(arr) => {
            let variable_skip_begin = ctx.values.len();
            let mut res = N::Unit;
            for a in arr.iter() {
                res = eval(a, ctx);
            }
            if log::log_enabled!(log::Level::Info) {
                for i in variable_skip_begin..ctx.values.len() {
                    info!(
                        "block forget {} {:?}:  {:?}",
                        i, ctx.idents[i], ctx.values[i]
                    );
                }
            }
            ctx.drain(variable_skip_begin);
            res
        }
        N::Set(name, val) => {
            let val = eval(val, ctx);
            ctx.set_val(name, val);
            N::Unit
        }
        N::Get(name) => ctx.find_var(name).map(|e| e.1.clone()).unwrap_or(N::Unit),
        N::FuncCall { func, args } => match eval(func, ctx) {
            N::FuncNativeDef(native) => native.0(
                args.first().map(|e| eval(e, ctx)).unwrap_or(N::Unit),
                args.get(1).map(|e| eval(e, ctx)).unwrap_or(N::Unit),
                args.get(2).map(|e| eval(e, ctx)).unwrap_or(N::Unit),
                args.get(3).map(|e| eval(e, ctx)).unwrap_or(N::Unit),
            ),
            N::FuncDef {
                args_name,
                mut scope,
            } => {
                let variable_scope_index = ctx.values.len();
                for (i, arg_name) in args_name.iter().enumerate() {
                    let val = args.get(i).map(|e| eval(e, ctx)).unwrap_or(N::Unit);
                    ctx.set_val(arg_name, val);
                }
                let res = eval(&mut scope, ctx);
                if log::log_enabled!(log::Level::Info) {
                    for i in variable_scope_index..ctx.values.len() {
                        info!("forget {} {:?}:  {:?}", i, ctx.idents[i], ctx.values[i]);
                    }
                }
                ctx.drain(variable_scope_index);
                res
            }
            N::Array(mut v) => {
                if let Some(index) = args.first().map(|e| eval(e, ctx)) {
                    match index {
                        N::Num(i) => {
                            let mut i = i as isize;
                            if i < 0 {
                                i = v.len() as isize + (i as isize);
                            }
                            v.get(i as usize).cloned().unwrap_or(N::Unit)
                        }
                        N::FuncDef { args_name, scope } => {
                            for (index, e) in v.iter_mut().enumerate() {
                                let variable_scope_index = ctx.values.len();
                                if let Some(s) = args_name.first() {
                                    ctx.set_val(s, e.clone());
                                }
                                if let Some(s) = args_name.get(1) {
                                    ctx.set_val(s, N::Num(index as f64));
                                }
                                *e = eval(&scope, ctx);
                                ctx.drain(variable_scope_index);
                            }
                            N::Array(v)
                        }
                        _ => N::Unit,
                    }
                } else {
                    N::Num(v.len() as f64)
                }
            }
            _ => N::Unit,
        },
        N::Binary(op, l, r) => {
            if let Op::Assign = op {
                if let N::Get(name) = l.as_ref() {
                    if let Some((key, _)) = ctx.find_var(name) {
                        let v = eval(r, ctx);
                        ctx.set_val_absolute(key, name, v);
                    }
                }
                return N::Unit;
            }
            let lt = eval(l, ctx);
            let rt = eval(r, ctx);
            match (op.clone(), &lt, &rt) {
                (Op::Plus, N::Num(li), N::Num(ri)) => N::Num(li + ri),
                (Op::Plus, N::Num(li), N::Unit) => N::Num(*li),
                (Op::Plus, N::Unit, N::Num(ri)) => N::Num(*ri),
                (Op::Greater, N::Num(li), N::Num(ri)) => bool_n(li > ri),
                (Op::Lesser, N::Num(li), N::Num(ri)) => bool_n(li < ri),
                (Op::Equals, N::Num(li), N::Num(ri)) => bool_n(li == ri),
                (Op::Equals, N::Str(li), N::Str(ri)) => bool_n(li == ri),
                (Op::NotEquals, N::Num(li), N::Num(ri)) => bool_n(li != ri),
                (Op::NotEquals, N::Str(li), N::Str(ri)) => bool_n(li != ri),
                (Op::Minus, N::Num(li), N::Num(ri)) => N::Num(li - ri),
                (Op::Mul, N::Num(li), N::Num(ri)) => N::Num(li * ri),
                (Op::Div, N::Num(li), N::Num(ri)) => N::Num(li / ri),
                (Op::Modulus, N::Num(li), N::Num(ri)) => N::Num(li.rem(ri)),
                (Op::Plus, N::Str(li), ri) => N::Str(format!("{}{}", li, ri.to_str())),
                (Op::Plus, li, N::Str(ri)) => N::Str(format!("{}{}", li.to_str(), ri)),
                (Op::Plus2, N::Array(li), N::Array(ri)) => {
                    N::Array(li.iter().chain(ri).cloned().collect())
                }
                (Op::Plus, N::Array(li), ri) => {
                    N::Array(li.iter().chain(core::iter::once(ri)).cloned().collect())
                }
                (Op::Plus, li, N::Array(ri)) => {
                    N::Array(core::iter::once(li).chain(ri.iter()).cloned().collect())
                }
                (Op::And, N::Array(li), N::FuncDef { args_name, scope }) => {
                    let mut new_arr = Vec::new();
                    for (index, e) in li.iter().enumerate() {
                        let variable_scope_index = ctx.values.len();
                        if let Some(s) = args_name.first() {
                            ctx.set_val(s, e.clone());
                        }
                        if let Some(s) = args_name.get(1) {
                            ctx.set_val(s, N::Num(index as f64));
                        }
                        if eval(&scope, ctx).to_bool() {
                            new_arr.push(e.clone());
                        }
                        ctx.drain(variable_scope_index);
                    }
                    N::Array(new_arr)
                }
                (Op::Or, N::Array(li), N::FuncDef { args_name, scope }) => {
                    let mut acc = li.first().cloned().unwrap_or(N::Unit);

                    for e in li.iter().skip(1) {
                        let variable_scope_index = ctx.values.len();
                        if let Some(s) = args_name.first() {
                            ctx.set_val(s, acc);
                        }
                        if let Some(s) = args_name.get(1) {
                            ctx.set_val(s, e.clone());
                        }
                        acc = eval(&scope, ctx);
                        ctx.drain(variable_scope_index);
                    }
                    acc
                }
                (Op::And, li, ri) => bool_n(li.to_bool() && ri.to_bool()),
                (Op::Or, li, ri) => bool_n(li.to_bool() || ri.to_bool()),
                _ => {
                    info!("unknown bin  {:?} {:?} {:?}", lt, op, rt);
                    N::Unit
                }
            }
        }
        N::FuncDef { args_name, scope } => N::FuncDef {
            args_name: args_name.clone(),
            scope: bx!(dup(&mut args_name.clone(), &mut scope.clone(), ctx)),
        },
        N::Array(v) => N::Array(v.iter().map(|e| eval(e, ctx)).collect()),
        e => {
            info!("noop");
            e.clone()
        }
    };

    ctx.deep -= 1;
    res
}

/// Create a new FuncDef by replacing known variables (excluding shadowed)
pub fn dup(excl: &mut Vec<ID>, n: &mut N, ctx: &mut Ctx) -> N {
    info!("instanciate {:?}", n);
    match n {
        N::Block(scope) => N::Block(scope.iter_mut().map(|e| dup(excl, e, ctx)).collect()),
        N::While { condition, body } => N::While {
            condition: bx!(dup(excl, condition, ctx)),
            body: bx!(dup(excl, body, ctx)),
        },
        N::FuncCall { func, args } => N::FuncCall {
            func: bx!(dup(excl, func, ctx)),
            args: args.iter_mut().map(|e| dup(excl, e, ctx)).collect(),
        },
        N::FuncDef { args_name, scope } => N::FuncDef {
            args_name: args_name.clone(),
            scope: bx!(dup(excl, scope, ctx)),
        },
        N::If {
            condition,
            path_true,
            path_false,
        } => N::If {
            condition: bx!(dup(excl, condition, ctx)),
            path_true: bx!(dup(excl, path_true, ctx)),
            path_false: bx!(dup(excl, path_false, ctx)),
        },
        N::Get(name) => {
            if excl.contains(name) {
                return N::Get(name.clone());
            }
            match ctx.find_var(name) {
                Some((_, n)) => n.clone(),
                _ => N::Get(name.clone()),
            }
        }
        N::Set(name, val) => {
            if let Some(index) = excl.iter().position(|x| x == name) {
                excl.remove(index);
            }
            N::Set(name.clone(), val.clone())
        }
        N::Binary(op, l, r) => N::Binary(*op, bx!(dup(excl, l, ctx)), bx!(dup(excl, r, ctx))),
        N::Array(v) => N::Array(v.iter_mut().map(|e| dup(excl, e, ctx)).collect()),
        e => e.clone(),
    }
}

fn next_token(i: &mut usize, code: &[char]) -> Token {
    let skip_whitespaces = |i: &mut usize| {
        while *i < code.len() && (code[*i] == ' ' || code[*i] == '\n') {
            *i += 1;
        }
    };

    let parse_number = |i: &mut usize| {
        let backup_i = *i;
        let mut id = String::from("");
        while code.len() > *i && (code[*i].is_ascii_digit() || code[*i] == '.') {
            id = format!("{}{}", id, code[*i]);
            *i += 1;
        }
        if !id.is_empty() {
            if let Ok(j) = id.parse::<f64>() {
                Some(j)
            } else {
                *i = backup_i;
                None
            }
        } else {
            *i = backup_i;
            None
        }
    };

    let parse_ident = |i: &mut usize| {
        let mut id = String::from("");
        while code.len() > *i && (code[*i].is_alphanumeric() || code[*i] == '_') {
            id = format!("{}{}", id, code[*i]);
            *i += 1;
        }
        if !id.is_empty() {
            Some(id)
        } else {
            None
        }
    };

    let starts_with = |mut i: usize, e: &str| {
        if i + e.len() > code.len() {
            return false;
        }
        for c in e.chars() {
            if code[i] != c {
                return false;
            }
            i += 1;
        }
        true
    };
    loop {
        skip_whitespaces(i);

        if *i >= code.len() {
            break Token::Err(String::from("i>code"));
        }

        if code[*i] == '"' {
            let mut builder = String::from("");
            while *i + 1 < code.len() {
                *i += 1;
                match code[*i] {
                    '"' => {
                        *i += 1;
                        return Token::Quoted(builder);
                    }
                    c => builder.push(c),
                }
            }
            return Token::Err(String::from("i>code"));
        }

        for (s, tok) in [
            ("if", Token::If),
            ("else", Token::Else),
            ("while", Token::While),
        ] {
            if starts_with(*i, s)
                && *i + s.len() < code.len()
                && [' ', '{'].contains(&code[*i + s.len()])
            {
                *i += s.len();
                return tok;
            }
        }

        if starts_with(*i, "let ") && *i + 4 < code.len() {
            *i += 4;
            skip_whitespaces(i);
            let id = match parse_ident(i) {
                Some(id) => id,
                None => break Token::Err(String::from("no id after let # ")),
            };
            skip_whitespaces(i);
            if *i >= code.len() || code[*i] != '=' {
                break Token::Err(String::from("no equal after let 'id' # "));
            }
            *i += 1;
            break Token::Let(id);
        }

        if let Some(num) = parse_number(i) {
            break Token::N(N::Num(num));
        }

        if let Some(id) = parse_ident(i) {
            break Token::N(N::Get(id));
        }

        for (st, tok) in [
            ("==", Token::Bin(Op::Equals)),
            ("!=", Token::Bin(Op::NotEquals)),
            ("++", Token::Bin(Op::Plus2)),
            ("<<", Token::Bin(Op::Shift)),
            ("=>", Token::Assoc),
        ] {
            if starts_with(*i, st) {
                *i += 2;
                return tok;
            }
        }

        for (key, val) in [
            ('{', Token::BlockStart),
            ('}', Token::BlockEnd),
            (',', Token::Comma),
            ('(', Token::ParStart),
            (')', Token::ParEnd),
            ('=', Token::Bin(Op::Assign)),
            ('+', Token::Bin(Op::Plus)),
            ('-', Token::Bin(Op::Minus)),
            ('*', Token::Bin(Op::Mul)),
            ('/', Token::Bin(Op::Div)),
            ('>', Token::Bin(Op::Greater)),
            ('<', Token::Bin(Op::Lesser)),
            ('%', Token::Bin(Op::Modulus)),
            ('&', Token::Bin(Op::And)),
            ('|', Token::Bin(Op::Or)),
            ('[', Token::ArrayStart),
            (']', Token::ArrayEnd),
        ] {
            if code[*i] == key {
                *i += 1;
                return val;
            }
        }

        *i += 1;
    }
}

fn pa(i: usize) -> String {
    format!("{:width$}", "", width = i * 5)
}
type Error = &'static str;

fn parse_expr(i: &mut usize, code: &[char], pad: usize) -> Result<N, Error> {
    info!(
        "{}parse expr {:?}",
        pa(pad),
        &code[*i..(*i + 5).min(if code.is_empty() { *i } else { code.len() - 1 })]
    );
    let term = parse_term(i, code, pad + 1)?;

    let mut j = *i;
    let token = next_token(&mut j, code);

    if let Token::Bin(op) = token {
        // if op.term_separate()
        {
            *i = j;
            let term_right = parse_expr(i, code, pad + 1)?;
            let n = N::Binary(op, bx!(term), bx!(term_right));
            return Ok(n);
        }
    }

    Ok(term)
}

fn parse_term(i: &mut usize, code: &[char], pad: usize) -> Result<N, Error> {
    info!(
        "{}parse_term {:?}",
        pa(pad),
        &code[*i..(*i + 5).min(if code.is_empty() { *i } else { code.len() - 1 })]
    );

    let factor = parse_factor(i, code, pad + 1)?;
    let mut j = *i;
    let token = next_token(&mut j, code);
    info!("{:?}", token);
    match token {
        Token::Bin(Op::Mul) | Token::Bin(Op::Div) | Token::ParStart => {
            *i = j;
            match token {
                Token::Bin(op) if !op.term_separate() => {
                    let factor_right = parse_term(i, code, pad + 1)?;
                    let n = N::Binary(op, bx!(factor), bx!(factor_right));
                    return Ok(n);
                }

                Token::ParStart => {
                    info!("Function call start");
                    let mut args = Vec::new();
                    loop {
                        info!("args enum");
                        let mut j = *i;
                        let e = parse_expr(&mut j, code, pad + 1);
                        match e {
                            Ok(expr) => {
                                info!("args enum got");
                                *i = j;
                                args.push(expr);

                                let mut k = *i;
                                let token = next_token(&mut k, code);
                                if let Token::Comma = token {
                                    *i = k
                                }
                            }
                            Err(_) => {
                                info!("args enum end");
                                break;
                            }
                        }
                    }
                    let token = next_token(i, code);
                    if let Token::ParEnd = token {
                        return Ok(N::FuncCall {
                            func: bx!(factor),
                            args,
                        });
                    } else {
                        return Err("No parenthesis close");
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    Ok(factor)
}

fn parse_factor(i: &mut usize, code: &[char], pad: usize) -> Result<N, Error> {
    if *i >= code.len() {
        return Err("EOF");
    }

    info!(
        "{}parse_factor {:?}",
        pa(pad),
        &code[*i..(*i + 5).min(if code.is_empty() { *i } else { code.len() - 1 })]
    );

    let token = next_token(i, code);
    info!("{}{:?}", pa(pad), token);
    if let Token::BlockStart = token {
        let mut scope = Vec::new();

        loop {
            let mut j = *i;
            let e = parse_expr(&mut j, code, pad + 1);
            match e {
                Ok(expr) => {
                    *i = j;
                    scope.push(expr);
                }
                Err(_) => {
                    break;
                }
            }
        }
        let token = next_token(i, code);
        if let Token::BlockEnd = token {
            return Ok(N::Block(scope));
        } else {
            return Err("No block end");
        }
    }

    if let Token::ArrayStart = token {
        let mut es = Vec::new();

        loop {
            let mut j = *i;
            let next = parse_expr(&mut j, code, pad);
            match next {
                Ok(expr) => {
                    *i = j;
                    es.push(expr)
                }
                Err(_) => {
                    let next = next_token(i, code);
                    match next {
                        Token::Comma => continue,
                        Token::ArrayEnd => break,
                        _ => break,
                    }
                }
            }
        }

        return Ok(N::Array(es));
    }

    if let Token::ParStart = token {
        info!("Function definition start");
        let mut args_name = Vec::new();

        loop {
            let token = next_token(i, code);
            match token {
                Token::N(N::Get(name)) => {
                    info!("name {}", name);
                    args_name.push(name);
                }
                Token::Comma => {}
                Token::ParEnd => {
                    break;
                }
                _ => {}
            }
        }

        let token = next_token(i, code);
        if let Token::Assoc = token {
            let scope = parse_expr(i, code, pad + 1)?;
            let n = N::FuncDef {
                args_name,
                scope: bx!(scope),
            };

            return Ok(n);
        } else {
            return Err("No => after func def");
        }
    }

    if let Token::Quoted(s) = token {
        let n = N::Str(s);
        return Ok(n);
    }

    if let Token::While = token {
        let condition = parse_expr(i, code, pad + 1)?;
        let body = parse_expr(i, code, pad + 1)?;
        let n = N::While {
            condition: bx!(condition),
            body: bx!(body),
        };

        return Ok(n);
    }

    if let Token::If = token {
        let cond_expr = parse_expr(i, code, pad + 1)?;
        let true_expr = parse_expr(i, code, pad + 1)?;
        let mut j = *i;
        let token = next_token(&mut j, code);
        let else_expr;
        if let Token::Else = token {
            *i = j;
            else_expr = parse_expr(i, code, pad + 1)?;
        } else {
            else_expr = N::Unit;
        }
        let n = N::If {
            condition: bx!(cond_expr),
            path_true: bx!(true_expr),
            path_false: bx!(else_expr),
        };
        return Ok(n);
    }

    if let Token::Let(name) = token {
        let val = parse_expr(i, code, pad + 1)?;
        let n = N::Set(name, bx!(val));
        return Ok(n);
    }

    if let Token::N(N::Num(num)) = token {
        return Ok(N::Num(num));
    }

    if let Token::N(N::Get(name)) = token {
        return Ok(N::Get(name));
    }

    Err("No term found")
}

#[cfg(test)]
mod test;
