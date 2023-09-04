use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fomoscript::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("counter", |b| {
        b.iter(|| {
            let code = r#"{
                let x = 0
                while x<1000 
                    x = x+1
                x
                }"#;

            let mut ctx = Ctx::new();
            ctx.insert_code(code);
            let parent = &mut ctx.parse_next_expr().unwrap();
            black_box(eval(parent, &mut ctx));
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

    c.bench_function("counter_deep", |b| {
        b.iter(|| {
            let code = r#"{
                let x = 0
                while x<1000 {
                    {
                        {
                            x = x+1
                        }
                    }
                }
                x
                }"#;
            let mut ctx = Ctx::new();
            ctx.insert_code(code);
            let parent = &mut ctx.parse_next_expr().unwrap();
            black_box(eval(parent, &mut ctx));
        })
    });

    c.bench_function("counter_parse", |b| {
        b.iter(|| {
            let mut ctx = Ctx::new();
            let code = r#"{
                let x = 0
                while x<1000 {
                    {
                        {
                            x = x+1
                        }
                    }
                }
                x
                }"#;
            ctx.insert_code(code);
            let _ = black_box(ctx.parse_next_expr().unwrap());
        })
    });

    c.bench_function("fib_20", |b| {
        b.iter(|| {
            let code = r#"{let fib = (e)=> if e<2 e else fib(e-1)+fib(e-2)
                fib(20)}"#;
            let mut ctx = Ctx::new();
            ctx.insert_code(code);
            let parent = &mut ctx.parse_next_expr().unwrap();
            black_box(eval(parent, &mut ctx));
        })
    });

    c.bench_function("fib_20_native", |b| {
        b.iter(|| {
            fn fib(e: f64) -> f64 {
                if e < 2.0 {
                    e
                } else {
                    fib(e - 1.) + fib(e - 2.)
                }
            }
            black_box(fib(black_box(20.0)))
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
