use super::*;

#[test]
fn binary_opt() {
    let code = r#"{1+1}"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 2.0)
    } else {
        assert!(false)
    }
}

#[test]
fn flow_while_0() {
    let code = r#"{
        let x = 0
        while x<5 {
            x = x+1
        }
        x
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 5.0)
    } else {
        assert!(false)
    }
}

#[test]
fn flow_if_no_else_0() {
    let code = r#"{
        let x = 0
        if x<5 {
            x = 6
            x
        }
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 6.0)
    } else {
        assert!(false)
    }
}
#[test]
fn flow_if_no_else_1() {
    let code = r#"{
        let x = 0
        if x>5 {
            x = 6
            x
        }
        }"#;
    let res = parse_eval(&code);
    if let N::Unit = res {
        assert!(true)
    } else {
        assert!(false)
    }
}

#[test]
fn flow_if_else_0() {
    let code = r#"{
        let x = 0
        let y = if x>5 {
            x = 6
            x
        }else 10
        y
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 10.0)
    } else {
        assert!(false)
    }
}

#[test]
fn flow_if_else_1() {
    let code = r#"{
        let x = 0
        let y = if x<5 {
            x = 6
            x
        }else 10
        y
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 6.0)
    } else {
        assert!(false)
    }
}

#[test]
fn flow_if_else_mini() {
    let code = r#"{
        let true = 1
        let y = if true 10 else 20
        y
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 10.0)
    } else {
        assert!(false)
    }
}

#[test]
fn string_concat() {
    let code = r#"{"hello" +" "+"world"}"#;
    let res = parse_eval(&code);
    if let N::Str(x) = res {
        assert_eq!(x, String::from("hello world"))
    } else {
        assert!(false)
    }
}

#[test]
fn higher_order_func() {
    let code = r#"{
        let x = 0
        let f = (e) => {e+1}
        let g=  (f,e)=> f(e)
        g(f,x)
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 1.0)
    } else {
        assert!(false)
    }
}

#[test]
fn scope() {
    let code = r#"{
        let x = 0
        if x<1 {
            let y = 1
            if y<2 {
                let z= 2
                x+y+z
            }
        }
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 3.0)
    } else {
        assert!(false)
    }
}

#[test]
fn big_function() {
    let code = r#"{
        let x = 0
        let f = (a,b,c) => {
            let d = 1000
            a+b+c+d
        }
        f(0,1,2)
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 1003.0)
    } else {
        assert!(false)
    }
}

#[test]
fn smol_function() {
    let code = r#"{
        let f = (a) => 1+a
        f(10)
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 11.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_equals() {
    let code = r#"{
        let x = 0
        if x==0{
            x =x+1
        }
        x
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 1.0)
    } else {
        assert!(false)
    }
}
#[test]
fn op_greater() {
    let code = r#"{
        let x = 0
        if 1>x{
            x =x+1
        }
        x
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 1.0)
    } else {
        assert!(false)
    }
}

#[test]
fn missing_args() {
    let code = r#"{
        let f = (a,b,c)=> a+b+c
        f(1,2)
        }"#;
    let res = parse_eval(&code);
    if let N::Unit = res {
        assert!(true)
    } else {
        assert!(false)
    }
}

#[test]
fn over_args() {
    let code = r#"{
        let f = (a,b,c)=> a+b+c
        f(1,2,3,4)
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 6.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_precedence_0() {
    let code = r#"{
        2+3*4
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 14.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_precedence_1() {
    let code = r#"{
        3*4+2
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 14.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_boolean_0() {
    let code = r#"{
        0&1
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 0.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_boolean_1() {
    let code = r#"{
        1&parent
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 0.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_boolean_2() {
    let code = r#"{
        1&1
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 1.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_boolean_3() {
    let code = r#"{
        0|1
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 1.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_boolean_4() {
    let code = r#"{
        1|0
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 1.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_boolean_5() {
    let code = r#"{
        1|1
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 1.0)
    } else {
        assert!(false)
    }
}

#[test]
fn op_boolean_6() {
    let code = r#"{
        0|0
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 0.0)
    } else {
        assert!(false)
    }
}

#[test]
fn func_returns_func() {
    let code = r#"{
        let f = (e) => {(a)=> a+e}
        let g = f(1)
        g(2)
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 3.0)
    } else {
        assert!(false)
    }
}

#[test]
fn anonymous_func() {
    let code = r#"{
        {(a,b,c)=> a+b+c}(1,2,3)
        }"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 6.0)
    } else {
        assert!(false)
    }
}

#[test]
fn multiple_expr_no_block() {
    let code = r#"let x = 5
    x+2"#;
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 7.0)
    } else {
        assert!(false)
    }
}

#[test]
fn fibonacci() {
    let code = "
let fib = (e)=> if e<2 e else fib(e-1)+fib(e-2)
fib(10)
";
    let res = parse_eval(&code);
    if let N::Num(x) = res {
        assert_eq!(x, 55.0)
    } else {
        assert!(false)
    }
}
