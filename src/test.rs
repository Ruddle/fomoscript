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
fn flow_if_no_else_0() {
    let code: Vec<char> = r#"{
        let x = 0
        if x<5 {
            x = 6
            x
        }
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 6.0)
    } else {
        assert!(false)
    }
}
#[test]
fn flow_if_no_else_1() {
    let code: Vec<char> = r#"{
        let x = 0
        if x>5 {
            x = 6
            x
        }
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Unit = res {
        assert!(true)
    } else {
        assert!(false)
    }
}

#[test]
fn flow_if_else_0() {
    let code: Vec<char> = r#"{
        let x = 0
        let y = if x>5 {
            x = 6
            x
        }else 10
        y
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 10.0)
    } else {
        assert!(false)
    }
}

#[test]
fn flow_if_else_1() {
    let code: Vec<char> = r#"{
        let x = 0
        let y = if x<5 {
            x = 6
            x
        }else 10
        y
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 6.0)
    } else {
        assert!(false)
    }
}

#[test]
fn flow_if_else_mini() {
    let code: Vec<char> = r#"{
        let true = 1
        let y = if true 10 else 20
        y
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 10.0)
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
        assert_eq!(x, String::from("hello world"))
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

#[test]
fn big_function() {
    let code: Vec<char> = r#"{
        let x = 0
        let f = (a,b,c) => {
            let d = 1000
            a+b+c+d
        }
        f(0,1,2)
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 1003.0)
    } else {
        assert!(false)
    }
}

#[test]
fn smol_function() {
    let code: Vec<char> = r#"{
        let f = (a) => 1+a
        f(10)
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 11.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_equals() {
    let code: Vec<char> = r#"{
        let x = 0
        if x==0{
            x =x+1
        }
        x
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
fn op_greater() {
    let code: Vec<char> = r#"{
        let x = 0
        if 1>x{
            x =x+1
        }
        x
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
fn missing_args() {
    let code: Vec<char> = r#"{
        let f = (a,b,c)=> a+b+c
        f(1,2)
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Unit = res {
        assert!(true)
    } else {
        assert!(false)
    }
}

#[test]
fn over_args() {
    let code: Vec<char> = r#"{
        let f = (a,b,c)=> a+b+c
        f(1,2,3,4)
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 6.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_precedence_0() {
    let code: Vec<char> = r#"{
        2+3*4
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 14.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_precedence_1() {
    let code: Vec<char> = r#"{
        3*4+2
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 14.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_boolean_0() {
    let code: Vec<char> = r#"{
        0&1
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 0.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_boolean_1() {
    let code: Vec<char> = r#"{
        1&0
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 0.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_boolean_2() {
    let code: Vec<char> = r#"{
        1&1
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
fn op_boolean_3() {
    let code: Vec<char> = r#"{
        0|1
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
fn op_boolean_4() {
    let code: Vec<char> = r#"{
        1|0
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
fn op_boolean_5() {
    let code: Vec<char> = r#"{
        1|1
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
fn op_boolean_6() {
    let code: Vec<char> = r#"{
        0|0
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 0.0)
    } else {
        assert!(false)
    }
}

#[test]
fn func_returns_func() {
    let code: Vec<char> = r#"{
        let f = (e) => {(a)=> a+e}
        let g = f(1)
        g(2)
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

#[test]
fn anonymous_func() {
    let code: Vec<char> = r#"{
        {(a,b,c)=> a+b+c}(1,2,3)
        }"#
    .chars()
    .collect();
    let ast = parse_ast(&code).expect("parse ok");
    let mut ctx = Ctx::new(ast);
    let res = eval(&0, &mut ctx);
    if let N::Num(x) = res {
        assert_eq!(x, 6.0)
    } else {
        assert!(false)
    }
}
