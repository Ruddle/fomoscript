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

The result will have [this type](/src/lib.rs#L52).

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
- Operator precedence

Also the inner workings are not very rust-like, no unsafe though ;)
