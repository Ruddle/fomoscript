use super::*;

#[test]
fn binary_opt() {
    let code: Vec<char> = r#"{1+1}"#.chars().collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 2.0)
    } else {
        assert!(false)
    }
}

#[test]
fn flow_while_0() {
    let code: Vec<char> = r#"{
        let x = 0
        while x<5 {
            x = x+1
        }
        x
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 5.0)
    } else {
        assert!(false)
    }
}

#[test]
fn string_concat() {
    let code: Vec<char> = r#"{"hello" +" "+"world"}"#.chars().collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Str(x) = res {
        assert_eq!(x, "hello world".to_owned())
    } else {
        assert!(false)
    }
}

#[test]
fn higher_order_func() {
    let code: Vec<char> = r#"{
        let x = 0
        let f = (e) => {e+1}
        let g=  (f,e)=> f(e)
        g(f,x)
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 1.0)
    } else {
        assert!(false)
    }
}

#[test]
fn scope() {
    let code: Vec<char> = r#"{
        let x = 0
        if x<1 {
            let y = 1
            if y<2 {
                let z= 2
                x+y+z
            }
        }
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 3.0)
    } else {
        assert!(false)
    }
}
