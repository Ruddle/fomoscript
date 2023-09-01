#![no_std]
use log::info;
extern crate alloc;
use alloc::{collections::BTreeMap, format, string::String, vec::Vec};
use core::{ops::Rem, result::Result};

/// An index to an AST node
pub type NI = usize;

/// fomoscript AST.
/// Nodes are stored in a Vec, and reference other nodes with their index.
pub type AST = Vec<N>;

/// fomoscript AST node
#[derive(Debug, Clone)]
pub enum N {
    FuncCall {
        func: NI,
        args: Vec<NI>,
    },
    Block(Vec<NI>),
    If {
        condition: NI,
        path_true: NI,
        path_false: NI,
    },
    While {
        condition: NI,
        body: NI,
    },
    Set {
        name: String,
        val: NI,
    },
    Get {
        name: String,
    },
    Binary {
        op: BinOp,
        l: NI,
        r: NI,
    },
    //Terminal nodes, the following nodes can be output by eval
    FuncDef {
        args_name: Vec<String>,
        scope: NI,
    },
    FuncNativeDef(Native),
    Array(Vec<NI>),
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOp {
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
    Minus,
    Assign,
}
impl BinOp {
    pub fn term_separate(self) -> bool {
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
pub enum Token {
    BlockStart,
    BlockEnd,
    If,
    Else,
    Comma,
    ParStart,
    ParEnd,
    While,
    Quoted(String),
    Bin(BinOp),
    N(N),
    Let(String),
    Err(String),
    FuncCallStart(String),
    FuncDefStart { args: Vec<String> },
}

/// Interpreter context, holds all state during execution.
pub struct Ctx {
    pub ast: AST,
    pub variables: BTreeMap<String, N>,
    pub path: String,
}

impl Ctx {
    pub fn new(ast: AST) -> Ctx {
        Ctx {
            ast,
            variables: BTreeMap::new(),
            path: String::from("_"),
        }
    }
    pub fn get_n(&self, idx: NI) -> N {
        self.ast[idx].clone()
    }
    pub fn set_var_scoped(&mut self, name: &str, n: N) {
        let key = format!("{}{}", self.path, name);
        self.variables.insert(key, n);
    }
    pub fn set_var_absolute(&mut self, path: &str, n: N) {
        self.variables.insert(String::from(path), n);
    }
    /// Find a variable declared in the scope, or any parent scope
    ///
    /// returns the path and the variable value
    pub fn find_var(&self, name: &str) -> Option<(String, N)> {
        let mut base = self.path.clone();
        loop {
            let key = format!("{}{}", base, name);
            let res = self.variables.get(&key);
            match res {
                Some(r) => {
                    return Some((key, r.clone()));
                }
                None => {
                    if base.pop().is_none() {
                        info!("Unknown variable {}", name);
                        return None;
                    }
                }
            }
        }
    }
}

fn bool_n(b: bool) -> N {
    N::Num(if b { 1.0 } else { 0.0 })
}

///Interprets the ctx at the node index specified (0 for root)
pub fn eval(ni: &NI, ctx: &mut Ctx) -> N {
    match ctx.get_n(*ni) {
        N::If {
            condition,
            path_true,
            path_false,
        } => match eval(&condition, ctx).to_bool() {
            true => eval(&path_true, ctx),
            false => eval(&path_false, ctx),
        },
        N::While { condition, body } => {
            let mut res = N::Unit;
            while eval(&condition, ctx).to_bool() {
                res = eval(&body, ctx)
            }
            res
        }
        N::Block(arr) => {
            ctx.path.push('_');
            let mut res = N::Unit;
            for a in arr.iter() {
                res = eval(a, ctx);
            }
            ctx.path.pop();
            res
        }
        N::Set { name, val } => {
            let val = eval(&val, ctx);
            ctx.set_var_scoped(&name, val);
            N::Unit
        }
        N::Get { name } => ctx.find_var(&name).map(|e| e.1).unwrap_or(N::Unit),
        N::FuncCall { func, args } => match eval(&func, ctx) {
            N::FuncNativeDef(native) => native.0(
                args.first().map(|e| eval(e, ctx)).unwrap_or(N::Unit),
                args.get(1).map(|e| eval(e, ctx)).unwrap_or(N::Unit),
                args.get(2).map(|e| eval(e, ctx)).unwrap_or(N::Unit),
                args.get(3).map(|e| eval(e, ctx)).unwrap_or(N::Unit),
            ),
            N::FuncDef { args_name, scope } => {
                for (i, arg_name) in args_name.iter().enumerate() {
                    let val = args.get(i).map(|e| eval(e, ctx)).unwrap_or(N::Unit);
                    ctx.path.push('_');
                    ctx.set_var_scoped(arg_name, val);
                    ctx.path.pop();
                }
                ctx.path.push('_');
                let res = eval(&scope, ctx);
                ctx.path.pop();
                res
            }
            _ => N::Unit,
        },

        N::Binary { op, l, r } => {
            if let BinOp::Assign = op {
                let n = ctx.get_n(l);
                if let N::Get { name } = n {
                    if let Some((key, _)) = ctx.find_var(&name) {
                        let v = eval(&r, ctx);
                        ctx.set_var_absolute(&key, v);
                    }
                }
                return N::Unit;
            }
            let lt = eval(&l, ctx);
            let rt = eval(&r, ctx);
            match (op, &lt, &rt) {
                (BinOp::Plus, N::Num(li), N::Num(ri)) => N::Num(li + ri),
                (BinOp::Greater, N::Num(li), N::Num(ri)) => bool_n(li > ri),
                (BinOp::Lesser, N::Num(li), N::Num(ri)) => bool_n(li < ri),
                (BinOp::Equals, N::Num(li), N::Num(ri)) => bool_n(li == ri),
                (BinOp::Equals, N::Str(li), N::Str(ri)) => bool_n(li == ri),
                (BinOp::NotEquals, N::Num(li), N::Num(ri)) => bool_n(li != ri),
                (BinOp::NotEquals, N::Str(li), N::Str(ri)) => bool_n(li != ri),
                (BinOp::And, li, ri) => bool_n(li.to_bool() && ri.to_bool()),
                (BinOp::Or, li, ri) => bool_n(li.to_bool() || ri.to_bool()),
                (BinOp::Minus, N::Num(li), N::Num(ri)) => N::Num(li - ri),
                (BinOp::Mul, N::Num(li), N::Num(ri)) => N::Num(li * ri),
                (BinOp::Div, N::Num(li), N::Num(ri)) => N::Num(li / ri),
                (BinOp::Modulus, N::Num(li), N::Num(ri)) => N::Num(li.rem(ri)),
                (BinOp::Plus, N::Str(li), ri) => N::Str(format!("{}{}", li, ri.to_str())),
                (BinOp::Plus, li, N::Str(ri)) => N::Str(format!("{}{}", li.to_str(), ri)),
                _ => {
                    info!("ERROR: bin {:?} {:?}", lt, rt);
                    N::Unit
                }
            }
        }
        e => e.clone(),
    }
}

pub fn next_token(i: &mut usize, code: &[char]) -> Token {
    let skip_whitespaces = |i: &mut usize| {
        while *i < code.len() && (code[*i] == ' ' || code[*i] == '\n') {
            *i += 1;
        }
    };
    let skip_comma = |i: &mut usize| {
        if *i < code.len() && code[*i] == ',' {
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
            loop {
                *i += 1;
                if *i >= code.len() {
                    return Token::Err(String::from("i>code"));
                }
                let c = code[*i];
                if c != '"' {
                    builder.push(c);
                } else {
                    *i += 1;
                    return Token::Quoted(builder);
                }
            }
        }
        if starts_with(*i, "if") && *i + 2 < code.len() {
            let c2 = code[*i + 2];
            if c2 == ' ' || c2 == '{' {
                *i += 2;
                break Token::If;
            }
        }
        if starts_with(*i, "else") && *i + 4 < code.len() {
            let c2 = code[*i + 4];
            if c2 == ' ' || c2 == '{' {
                *i += 4;
                break Token::Else;
            }
        }
        if starts_with(*i, "while") && *i + 5 < code.len() {
            let c2 = code[*i + 5];
            if c2 == ' ' || c2 == '{' {
                *i += 5;
                break Token::While;
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
        if code[*i] == '(' {
            //let i_backup = *i;
            *i += 1;
            if *i >= code.len() {
                return Token::Err(String::from("i>code"));
            }
            let mut idents = Vec::new();
            loop {
                skip_whitespaces(i);
                match parse_ident(i) {
                    Some(id) => idents.push(id),
                    None => break,
                };
                skip_comma(i);
            }

            skip_whitespaces(i);

            if *i >= code.len() || code[*i] != ')' {
                break Token::Err(String::from("no end parenthesis after args"));
            }
            *i += 1;
            skip_whitespaces(i);

            if *i + 1 >= code.len() || code[*i] != '=' || code[*i + 1] != '>' {
                break Token::Err(String::from("no => after args"));
            }
            *i += 2;

            break Token::FuncDefStart { args: idents };
        }

        if let Some(num) = parse_number(i) {
            break Token::N(N::Num(num));
        }

        if let Some(id) = parse_ident(i) {
            skip_whitespaces(i);
            if *i < code.len() && code[*i] == '(' {
                *i += 1;
                break Token::FuncCallStart(id);
            }
            break Token::N(N::Get { name: id });
        }
        if starts_with(*i, "==") {
            *i += 2;
            break Token::Bin(BinOp::Equals);
        }
        if starts_with(*i, "!=") {
            *i += 2;
            break Token::Bin(BinOp::NotEquals);
        }
        for (key, val) in [
            (',', Token::Comma),
            (')', Token::ParEnd),
            ('{', Token::BlockStart),
            ('}', Token::BlockEnd),
            ('+', Token::Bin(BinOp::Plus)),
            ('-', Token::Bin(BinOp::Minus)),
            ('*', Token::Bin(BinOp::Mul)),
            ('/', Token::Bin(BinOp::Div)),
            ('=', Token::Bin(BinOp::Assign)),
            ('>', Token::Bin(BinOp::Greater)),
            ('<', Token::Bin(BinOp::Lesser)),
            ('%', Token::Bin(BinOp::Modulus)),
            ('&', Token::Bin(BinOp::And)),
            ('|', Token::Bin(BinOp::Or)),
        ] {
            if code[*i] == key {
                *i += 1;
                return val;
            }
        }

        *i += 1;
    }
}

pub fn insert_in_parent(ast: &mut AST, parent: NI, child: NI) {
    match &mut ast[parent] {
        N::Block(v) => {
            v.push(child);
        }
        N::FuncCall { args, .. } => {
            args.push(child);
        }
        _ => {}
    }
}

fn pa(i: usize) -> String {
    format!("{:width$}", "", width = i * 3)
}
type Error = &'static str;
pub fn parse_ast(code: &[char]) -> Result<AST, Error> {
    let mut i = 0;
    let mut ast = Vec::new();
    parse_expr(&mut ast, &mut i, code, 0).map(|_| ast)
}

pub fn parse_expr(ast: &mut AST, i: &mut usize, code: &[char], pad: usize) -> Result<NI, Error> {
    if *i >= code.len() {
        return Err("EOF");
    }

    info!(
        "{}parse expr {:?}",
        pa(pad),
        &code[*i..(*i + 5).min(code.len() - 1)]
    );
    let term = parse_term(ast, i, code, pad + 1)?;

    let mut j = *i;
    let token = next_token(&mut j, code);

    if let Token::Bin(op) = token {
        if op.clone().term_separate() {
            *i = j;
            let term_right = parse_expr(ast, i, code, pad + 1)?;
            let n = N::Binary {
                op,
                l: term,
                r: term_right,
            };
            let block_ni = ast.len();
            ast.push(n);
            return Ok(block_ni);
        }
    }

    Ok(term)
}

pub fn parse_term(ast: &mut AST, i: &mut usize, code: &[char], pad: usize) -> Result<NI, Error> {
    if *i >= code.len() {
        return Err("EOF");
    }

    info!(
        "{}parse_term {:?}",
        pa(pad),
        &code[*i..(*i + 5).min(code.len() - 1)]
    );

    let factor = parse_factor(ast, i, code, pad + 1)?;
    let mut j = *i;
    let token = next_token(&mut j, code);
    match token {
        Token::Bin(BinOp::Mul) | Token::Bin(BinOp::Div) | Token::ParStart => {
            *i = j;
            let factor_right = parse_factor(ast, i, code, pad + 1)?;
            match token {
                Token::Bin(op) => {
                    let n = N::Binary {
                        op,
                        l: factor,
                        r: factor_right,
                    };
                    let block_ni = ast.len();
                    ast.push(n);
                    return Ok(block_ni);
                }

                Token::ParStart => {}
                _ => {}
            }
        }

        _ => {}
    }

    Ok(factor)
}

pub fn parse_factor(ast: &mut AST, i: &mut usize, code: &[char], pad: usize) -> Result<NI, Error> {
    if *i >= code.len() {
        return Err("EOF");
    }

    info!(
        "{}parse_factor {:?}",
        pa(pad),
        &code[*i..(*i + 5).min(code.len() - 1)]
    );

    let token = next_token(i, code);
    info!("{}{:?}", pa(pad), token);
    if let Token::BlockStart = token {
        let block_ni = ast.len();
        ast.push(N::Block(Vec::new()));

        loop {
            let mut j = *i;
            let e = parse_expr(ast, &mut j, code, pad + 1);
            match e {
                Ok(expr) => {
                    *i = j;
                    insert_in_parent(ast, block_ni, expr)
                }
                Err(_) => {
                    break;
                }
            }
        }
        let token = next_token(i, code);
        if let Token::BlockEnd = token {
            return Ok(block_ni);
        } else {
            return Err("No block end");
        }
    }

    if let Token::FuncDefStart { args } = token {
        let scope = parse_expr(ast, i, code, pad + 1)?;

        let n = N::FuncDef {
            args_name: args,
            scope,
        };
        let block_ni: usize = ast.len();
        ast.push(n);
        return Ok(block_ni);
    }

    if let Token::Quoted(s) = token {
        let n = N::Str(s);
        let node_ni = ast.len();
        ast.push(n);
        return Ok(node_ni);
    }

    if let Token::While = token {
        let condition = parse_expr(ast, i, code, pad + 1)?;
        let body = parse_expr(ast, i, code, pad + 1)?;
        let n = N::While { condition, body };
        let node_ni = ast.len();
        ast.push(n);
        return Ok(node_ni);
    }

    if let Token::If = token {
        let cond_expr = parse_expr(ast, i, code, pad + 1)?;
        let true_expr = parse_expr(ast, i, code, pad + 1)?;
        let mut j = *i;
        let token = next_token(&mut j, code);
        let else_expr;
        if let Token::Else = token {
            *i = j;
            else_expr = parse_expr(ast, i, code, pad + 1)?;
        } else {
            let n = N::Unit;
            else_expr = ast.len();
            ast.push(n);
        }
        let n = N::If {
            condition: cond_expr,
            path_true: true_expr,
            path_false: else_expr,
        };
        let ifn = ast.len();
        ast.push(n);
        return Ok(ifn);
    }

    if let Token::Let(name) = token {
        let val = parse_expr(ast, i, code, pad + 1)?;
        let n = N::Set { name, val };
        let set_expr_ni: usize = ast.len();
        ast.push(n);
        return Ok(set_expr_ni);
    }

    if let Token::N(N::Num(num)) = token {
        let n = N::Num(num);
        let expr_ni: usize = ast.len();
        ast.push(n);
        return Ok(expr_ni);
    }

    if let Token::N(N::Get { name }) = token {
        let n = N::Get { name };
        let expr_ni: usize = ast.len();
        ast.push(n);
        return Ok(expr_ni);
    }

    if let Token::FuncCallStart(name) = token {
        let get_ni = {
            let get = N::Get { name };
            let expr_ni: usize = ast.len();
            ast.push(get);
            expr_ni
        };

        let n = N::FuncCall {
            func: get_ni,
            args: Vec::new(),
        };
        let expr_ni: usize = ast.len();
        ast.push(n);
        loop {
            let mut j = *i;
            let e = parse_expr(ast, &mut j, code, pad + 1);
            match e {
                Ok(expr) => {
                    *i = j;
                    insert_in_parent(ast, expr_ni, expr);
                    let mut k = *i;
                    let token = next_token(&mut k, code);
                    if let Token::Comma = token {
                        *i = k
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }
        let token = next_token(i, code);
        if let Token::ParEnd = token {
            return Ok(expr_ni);
        } else {
            return Err("No parenthesis close");
        }
    }

    Err("No term found")
}

#[cfg(test)]
mod test;
