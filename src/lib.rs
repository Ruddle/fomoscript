#![no_std]
use log::info;

extern crate alloc;
use alloc::borrow::ToOwned;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use core::ops::Rem;
use core::result::Result;
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
        op: BinaryOp,
        l: NI,
        r: NI,
    },
    //Terminal nodes, the following nodes can be output by eval
    FuncDef {
        args_name: Vec<String>,
        scope: Vec<NI>,
    },
    FuncNativeDef(Native),
    Array(Vec<NI>),
    Num(f64),
    Str(String),
    Unit,
}
#[derive(Clone)]
pub struct Native(pub alloc::rc::Rc<dyn Fn(N, N, N, N) -> N>);

impl core::fmt::Debug for Native {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "Native")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Plus,
    Minus,
    Mul,
    Div,
    Assign,
    Lesser,
    Greater,
    Equals,
    Modulus,
}

impl N {
    pub fn as_f64(&self) -> f64 {
        match self {
            N::Num(x) => *x,
            _ => 0.0,
        }
    }
    ///Cast to to boolean. The equivalent of js: self == true
    pub fn is_truthy(&self) -> bool {
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
    ParEnd,
    While,
    Quoted(String),
    Bin(BinaryOp),
    N(N),
    Let(String),
    Err(String),
    FuncCallStart(String),
    FuncDefStart { args: Vec<String> },
}

/// Interpreter context
/// Holds all variables during execution
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
            path: "_".to_string(),
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
        self.variables.insert(path.to_owned(), n);
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

pub fn eval(ni: &NI, ctx: &mut Ctx) -> N {
    match ctx.get_n(*ni) {
        N::If {
            condition,
            path_true,
            path_false,
        } => {
            if eval(&condition, ctx).is_truthy() {
                eval(&path_true, ctx)
            } else {
                eval(&path_false, ctx)
            }
        }

        N::While { condition, body } => {
            let mut res = N::Unit;
            while eval(&condition, ctx).is_truthy() {
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
        N::Get { name } => match ctx.find_var(&name) {
            Some((_, n)) => n,
            _ => N::Unit,
        },
        N::FuncCall { func, args } => {
            //
            match eval(&func, ctx) {
                N::FuncNativeDef(native) => {
                    let func = native.0;

                    match args.len() {
                        0 => func(N::Unit, N::Unit, N::Unit, N::Unit),
                        1 => func(eval(&args[0], ctx), N::Unit, N::Unit, N::Unit),
                        2 => func(eval(&args[0], ctx), eval(&args[1], ctx), N::Unit, N::Unit),
                        3 => func(
                            eval(&args[0], ctx),
                            eval(&args[1], ctx),
                            eval(&args[2], ctx),
                            N::Unit,
                        ),
                        _ => func(
                            eval(&args[0], ctx),
                            eval(&args[1], ctx),
                            eval(&args[2], ctx),
                            eval(&args[3], ctx),
                        ),
                    }
                }
                N::FuncDef { args_name, scope } => {
                    for (i, arg) in args.iter().enumerate() {
                        let val = eval(arg, ctx);
                        info!("fun call arg{}: {:?}", i, val);
                        ctx.path.push('_');
                        ctx.set_var_scoped(&args_name[i], val);
                        ctx.path.pop();
                    }
                    ctx.path.push('_');
                    let mut res = N::Unit;
                    for a in scope.iter() {
                        res = eval(a, ctx);
                    }
                    ctx.path.pop();
                    res
                }
                _ => N::Unit,
            }
        }

        N::Binary { op, l, r } => {
            if let BinaryOp::Assign = op {
                let n = ctx.get_n(l);
                if let N::Get { name } = n {
                    if let Some((key, _)) = ctx.find_var(&name) {
                        let v = eval(&r, ctx);
                        ctx.set_var_absolute(&key, v);
                    }
                }
                N::Unit
            } else {
                let lt = eval(&l, ctx);
                let rt = eval(&r, ctx);
                match (op, &lt, &rt) {
                    (BinaryOp::Plus, N::Num(li), N::Num(ri)) => N::Num(li + ri),
                    (BinaryOp::Greater, N::Num(li), N::Num(ri)) => {
                        N::Num(if li > ri { 1.0 } else { 0.0 })
                    }
                    (BinaryOp::Lesser, N::Num(li), N::Num(ri)) => {
                        N::Num(if li < ri { 1.0 } else { 0.0 })
                    }
                    (BinaryOp::Equals, N::Num(li), N::Num(ri)) => {
                        N::Num(if li == ri { 1.0 } else { 0.0 })
                    }
                    (BinaryOp::Equals, N::Str(li), N::Str(ri)) => {
                        N::Num(if li == ri { 1.0 } else { 0.0 })
                    }
                    (BinaryOp::Minus, N::Num(li), N::Num(ri)) => N::Num(li - ri),
                    (BinaryOp::Mul, N::Num(li), N::Num(ri)) => N::Num(li * ri),
                    (BinaryOp::Div, N::Num(li), N::Num(ri)) => N::Num(li / ri),
                    (BinaryOp::Modulus, N::Num(li), N::Num(ri)) => N::Num(li.rem(ri)),
                    (BinaryOp::Plus, N::Str(li), ri) => N::Str(format!("{}{}", li, ri.to_str())),
                    (BinaryOp::Plus, li, N::Str(ri)) => N::Str(format!("{}{}", li.to_str(), ri)),
                    _ => {
                        info!("ERROR: bin {:?} {:?}", lt, rt);
                        N::Unit
                    }
                }
            }
        }
        e => e.clone(),
    }
}

pub fn next_token(i: &mut usize, code: &[char]) -> Token {
    let skip_whitespace = |i: &mut usize| {
        while *i < code.len() && (code[*i] == ' ' || code[*i] == '\n') {
            *i += 1;
        }
    };
    let skip_comma = |i: &mut usize| {
        if code[*i] == ',' {
            *i += 1;
        }
    };

    let parse_number = |i: &mut usize| {
        let backup_i = *i;
        let mut id = "".to_owned();
        while code[*i].is_ascii_digit() || code[*i] == '.' {
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
        let mut id = "".to_owned();
        while code[*i].is_alphanumeric() || code[*i] == '_' {
            id = format!("{}{}", id, code[*i]);
            *i += 1;
        }
        if !id.is_empty() {
            Some(id)
        } else {
            None
        }
    };
    loop {
        skip_whitespace(i);
        if *i >= code.len() {
            break Token::Err("i>code".to_owned());
        }
        let c = code[*i];

        {
            if c == '{' {
                *i += 1;
                break Token::BlockStart;
            }
            if let '}' = c {
                *i += 1;
                break Token::BlockEnd;
            }

            if c == '"' {
                let mut builder = "".to_owned();
                loop {
                    *i += 1;
                    let c = code[*i];
                    if c != '"' {
                        builder.push(c);
                    } else {
                        *i += 1;
                        return Token::Quoted(builder);
                    }
                }
            }

            if c == 'i' && *i + 2 < code.len() && code[*i + 1] == 'f' {
                let c2 = code[*i + 2];
                if c2 == ' ' || c2 == '{' {
                    *i += 2;
                    break Token::If;
                }
            }
            if c == 'e'
                && *i + 4 < code.len()
                && code[*i + 1] == 'l'
                && code[*i + 2] == 's'
                && code[*i + 3] == 'e'
            {
                let c2 = code[*i + 4];
                if c2 == ' ' || c2 == '{' {
                    *i += 4;
                    break Token::Else;
                }
            }

            if c == 'w'
                && *i + 5 < code.len()
                && code[*i + 1] == 'h'
                && code[*i + 2] == 'i'
                && code[*i + 3] == 'l'
                && code[*i + 4] == 'e'
            {
                let c2 = code[*i + 5];
                if c2 == ' ' || c2 == '{' {
                    *i += 5;
                    break Token::While;
                }
            }

            if c == 'l'
                && *i + 3 < code.len()
                && code[*i + 1] == 'e'
                && code[*i + 2] == 't'
                && code[*i + 3] == ' '
            {
                *i += 4;
                skip_whitespace(i);
                let id = match parse_ident(i) {
                    Some(id) => id,
                    None => break Token::Err("no id after let # ".to_owned()),
                };
                skip_whitespace(i);

                if code[*i] != '=' {
                    break Token::Err("no equal after let 'id' # ".to_owned());
                }
                *i += 1;

                break Token::Let(id);
            }

            if c == '(' {
                //let i_backup = *i;
                *i += 1;

                let mut idents = vec![];
                loop {
                    skip_whitespace(i);
                    match parse_ident(i) {
                        Some(id) => idents.push(id),
                        None => break,
                    };
                    skip_comma(i);
                }

                skip_whitespace(i);

                if code[*i] != ')' {
                    break Token::Err("no end parenthesis after args".to_owned());
                }
                *i += 1;
                skip_whitespace(i);

                if code[*i] != '=' || code[*i + 1] != '>' {
                    break Token::Err("no => after args".to_owned());
                }
                *i += 2;

                break Token::FuncDefStart { args: idents };
            }

            if let Some(num) = parse_number(i) {
                break Token::N(N::Num(num));
            }

            if let Some(id) = parse_ident(i) {
                skip_whitespace(i);
                if code[*i] == '(' {
                    *i += 1;
                    break Token::FuncCallStart(id);
                }

                break Token::N(N::Get { name: id });
            }
            if c == ',' {
                *i += 1;
                break Token::Comma;
            }
            if c == ')' {
                *i += 1;
                break Token::ParEnd;
            }
            if c == '+' {
                *i += 1;
                break Token::Bin(BinaryOp::Plus);
            }
            if c == '-' {
                *i += 1;
                break Token::Bin(BinaryOp::Minus);
            }
            if c == '*' {
                *i += 1;
                break Token::Bin(BinaryOp::Mul);
            }
            if c == '/' {
                *i += 1;
                break Token::Bin(BinaryOp::Div);
            }
            if c == '=' {
                *i += 1;
                break Token::Bin(BinaryOp::Assign);
            }
            if c == '>' {
                *i += 1;
                break Token::Bin(BinaryOp::Greater);
            }
            if c == '<' {
                *i += 1;
                break Token::Bin(BinaryOp::Lesser);
            }
            if c == '%' {
                *i += 1;
                break Token::Bin(BinaryOp::Modulus);
            }
            if c == '=' && *i + 1 < code.len() && code[*i + 1] == '=' {
                *i += 2;
                break Token::Bin(BinaryOp::Equals);
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
        N::FuncDef { scope, .. } => {
            scope.push(child);
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
    let mut ast = vec![];
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
    let token = next_token(i, code);
    info!("{}p{:?}", pa(pad), token);
    if let Token::BlockStart = token {
        let block_ni = ast.len();
        ast.push(N::Block(vec![]));

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
            scope: vec![scope],
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
            args: vec![],
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
