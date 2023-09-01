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

# Use in Rust

**Step 1**: have some code

```rust
let code: Vec<char> = r#"
{
    let x = 0
    while x<5 {
        x = x+1
    }
    x
}"#
    .chars()
    .collect();
```

**Step 2**: parse it

```rust
let ast = parse_ast(&code).expect("parse ok");
```

**Step 3**: run it

```rust
let mut ctx = Ctx::new(ast);
let result = eval(&0, &mut ctx);
```

The result will have [this type](/src/lib.rs#L36).

Go see the [tests](/src/test.rs) for more examples.

By default, there is no side effect possible from the script during eval (except inside ctx)

You can insert native rust closure with (or without) side effects into the script, and use it from there.
Example with the print function:

```rust
let code: Vec<char> = r#"
{
    my_print(1+1)
}
"#
.chars()
.collect();
let ast = parse_ast(&code).unwrap();
let mut ctx = Ctx::new(ast);

let print_closure = Rc::new(|a: N, _, _, _| {
    println!("{}", a.to_str());
    N::Unit
});
ctx.set_var_absolute("my_print", N::FuncNativeDef(Native(print_closure)));

let _ = eval(&0, &mut ctx);
```

# Cruelly missing

- Arrays
- Months of work
- Pattern matching
- Javascript-like objects (we just have Number and String 😱)
- Read-Eval-Print Loop
- Error handling (now it just UB if something goes wrong)
- Escape characters in quoted strings

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
- [x] Anonymous function call

# Performance

Parsing is instantaneous (50+GB/sec).

Evaluation is slow, but reasonable for scripting:

- Worst case 1:1000 compared to native
- Common case 1:20 when using native functions reasonably.

See for yourself with `cargo bench`

Unstructured number crunching will stay slow.
Typed arrays (like in js) could be added in the future for fast structured operation.

# Fun facts

Everything is an expression. For instance if/else acts as a ternary operator.

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
