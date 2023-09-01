use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fomoscript::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    let code: Vec<char> = r#"{
        let x = 0
        while x<1000 
            x = x+1
        x
        }"#
    .chars()
    .collect();
    let (ast, parent) = parse_ast(&code).expect("parse ok");
    c.bench_function("counter", |b| {
        b.iter(|| {
            let mut ctx = Ctx::new(ast.clone());
            black_box(eval(&parent, &mut ctx));
        })
    });

    c.bench_function("counter_native", |b| {
        b.iter(|| {
            let mut x = 0;
            while x < 1000 {
                x = black_box(x) + 1
            }
            x
        })
    });

    let code: Vec<char> = r#"{
        let x = 0
        while x<1000 {
            {
                {
                    x = x+1
                }
            }
        }
        x
        }"#
    .chars()
    .collect();
    let (ast, parent) = parse_ast(&code).expect("parse ok");
    c.bench_function("counter_deep", |b| {
        b.iter(|| {
            let mut ctx = Ctx::new(ast.clone());
            black_box(eval(&parent, &mut ctx));
        })
    });

    c.bench_function("counter_parse", |b| {
        b.iter(|| {
            let code: Vec<char> = r#"{
                let x = 0
                while x<1000 {
                    {
                        {
                            x = x+1
                        }
                    }
                }
                x
                }"#
            .chars()
            .collect();
            parse_ast(&black_box(code)).expect("parse ok");
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
