![Crates.io](https://img.shields.io/crates/v/fomoscript?link=https%3A%2F%2Fcrates.io%2Fcrates%2Ffomoscript)

# fomoscript

Toy scripting language, built with Rust

- 0 dependencies\*
- 1 [file](/src/lib.rs)
- no_std with alloc

Only a few days old. **Not** production ready. The goal is to use it in [Fomos](https://github.com/Ruddle/Fomos) as a shell.

\* except log, doesn't count ;)

# Examples

#### Simple script

```java
{
    let x = 0
    while x<5 {
        x = x+1
    }
    x
}
```

`returns 5`

#### Support of higher order functions

```java
{
    let x = 0
    let f = (e) => {e+1}
    let g = (f,e) => f(e)
    g(f,x)
}
```

`returns 1`

# Usage

Add this to your Cargo.toml:

```toml
[dependencies]
fomoscript = "0.2.1"
```

Parse and evaluate a script:

```rust
let result = fomoscript::parse_eval("{
    let x = 0
    while x<5 {
        x = x+1
    }
    x
}");
```

The result will have [this type](/src/lib.rs#L36).

Go see the [tests](/src/test.rs) for more examples.

`parse_eval` is a high level function hiding the lower level `Ctx`.

You can explicitly instantiate an interpreter called `Ctx` to implement a REPL or explore/modify the state during execution.

By default, there is **no side effect** possible from the script during eval (except inside ctx)

You can **insert native** rust closure with (or without) **side effects** into the `Ctx`, and use it from inside the script.
Example with the print function:

```rust
use fomoscript::*;
let code = r#"
{
    my_print(1+1)
}
"#;

let mut ctx = Ctx::new();
ctx.insert_code(code);

let print_closure = Rc::new(|a: N, _, _, _| {
    println!("{}", a.to_str());
    N::Unit
});
ctx.set_var_absolute("my_print", N::FuncNativeDef(Native(print_closure)));

let expr = ctx.parse_next_expr().unwrap();
let _ = eval(&expr, &mut ctx);
```

### REPL

For simplicity, std is used here, but you can replace it with any input and output impl.

```rust
use fomoscript::*;
let mut ctx = Ctx::new();
let mut buffer = String::new();
loop {
    buffer.clear();
    std::io::stdin().read_line(&mut buffer).unwrap();
    ctx.insert_code(&buffer);
    while let Ok(parent) = ctx.parse_next_expr() {
        let res = eval(&parent, &mut ctx);
        println!("> {:?}", res);
    }
}
```

# Cruelly missing

- Arrays
- Javascript-like objects (we just have Number and String 😱)
- Error handling (now it just UB if something goes wrong)
- Escape characters in quoted strings
- Months of work
- Pattern matching

Also the inner workings are not very rust-like, no unsafe though ;)
Should be panic free during eval. Don't trust the parser just yet.

# Features

- [x] String type
- [x] Number type (f64)
- [x] Scoped variable assignment
- [x] Binary operators +,-,/,\*,>,<,==,!=,&,|
- [x] Operator precedence
- [x] Higher order function
- [x] Control flow if/else/while
- [x] Custom native function
- [x] Anonymous function call$
- [x] REPL example

# Performance

Parsing is instantaneous (50+GB/sec).

Evaluation is slow, but reasonable for scripting:

- Worst case 1:1000 compared to native
- Common case 1:20 when using native functions reasonably.

See for yourself with `cargo bench`

Unstructured number crunching will stay slow.
Typed arrays (like in js) could be added in the future for fast structured operation.

# Fun facts

Everything is an expression in fomoscript. For instance if/else acts as a ternary operator.

`let x= if 1 99 else 45`

now x is 99

When there is a doubt, the interpreter defaults to `N::Unit`. For instance let's not put an else branch:

`let x=  if 0 1`

x is now `N::Unit`

No parenthesis needed for the `if` condition or body, the previous expression is equivalent to:

```rust
let x = if 0 {
    1
} else {
    N::Unit
}
```

Same goes for while, it returns `N::Unit` if it never runs the body, or the last body expression if it runs at least once.

Same goes for brackets :

`let x = {1 2 3}` is equivalent to `let x = {3}` or `let x = 3` or

```
let x = {
    1
    2
    3
}
```

There is no parenthesis, use brackets to force factorization and precedence.

`\n` is just a whitespace like space. It doesn't separate statements more than space, unlike most languages.

Boolean operation automatically cast operand to bool (lookup `to_bool` to see how)

the (and,or) operators are (`&`,`|`)

`1 & 0` evaluate to 0

`1 | 0` evaluate to 1

No bitwise operation yet.
