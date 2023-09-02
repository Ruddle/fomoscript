use criterion::{black_box, criterion_group, criterion_main, Criterion};
use fomoscript::*;

pub fn criterion_benchmark(c: &mut Criterion) {
    let code = r#"{
        let x = 0
        while x<1000 
            x = x+1
        x
        }"#;

    let mut ctx = Ctx::new();
    ctx.insert_code(code);
    let parent = ctx.parse_next_expr().unwrap();

    c.bench_function("counter", |b| {
        b.iter(|| {
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
    let parent = ctx.parse_next_expr().unwrap();
    c.bench_function("counter_deep", |b| {
        b.iter(|| {
            black_box(eval(&parent, &mut ctx));
        })
    });

    let mut ctx = Ctx::new();
    c.bench_function("counter_parse", |b| {
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
            ctx.insert_code(code);
            let _ = black_box(ctx.parse_next_expr().unwrap());
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
