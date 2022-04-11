use criterion::*;
use divvy::Str;
use std::iter::*;
use table::*;
use Entry::*;

fn n<T, N: Into<Number>>(n: N) -> Entry<T> {
    Num(n.into())
}
fn vec_of_vecs() -> Vec<Vec<Entry<Str>>> {
    // 1000 cols by 10000 rows
    let rowseed = vec![
        n(101),
        Nil,
        Obj(Str::new("Hello, world!!!")),
        n(1.23456789e54),
    ];
    let row = repeat(rowseed).take(250).flatten().collect();
    repeat(row).take(10_000).collect()
}

fn adding_rows(c: &mut Criterion) {
    let mut c = c.benchmark_group("Adding Rows");

    c.bench_function("adding one row at a time", |b| {
        let seed = vec_of_vecs();
        b.iter(|| {
            let mut x = Table::new();
            for row in &seed {
                x.add_row(row.iter().cloned());
            }
        });
    });

    c.bench_function("adding rows all at once", |b| {
        let seed = vec_of_vecs();
        b.iter(|| {
            let mut x = Table::new();
            x.add_rows(seed.iter().map(|row| row.iter().cloned()));
        });
    });

    c.bench_function("using From vector of vectors", |b| {
        let seed = vec_of_vecs();
        b.iter(|| {
            let seed = seed.clone();
            black_box(Table::from(seed));
        });
    });
}

fn cloning(c: &mut Criterion) {
    let mut c = c.benchmark_group("Table Replicating");

    c.bench_function("clone", |b| {
        let table = Table::from(vec_of_vecs());
        b.iter(|| black_box(table.clone()));
    });

    c.bench_function("using add_rows and rows", |b| {
        let table = Table::from(vec_of_vecs());
        b.iter(|| {
            let mut x = ::table::Table::new();
            x.add_rows(table.rows().map(|x| x.cloned()));
            x
        });
    });

    c.bench_function("Table::map", |b| {
        let table = Table::from(vec_of_vecs());
        b.iter(|| {
            let table = table.clone();
            black_box(table.map(|_| Entry::<()>::Num(3.14.into())));
        })
    });

    c.bench_function("Table::map_obj", |b| {
        let table = Table::from(vec_of_vecs());
        b.iter(|| {
            let table = table.clone();
            black_box(table.map_obj(|e| Obj(e.to_string())));
        })
    });
}

fn parse_csv(c: &mut Criterion) {
    let mut c = c.benchmark_group("Parse CSV");

    let file = &std::fs::read_to_string("diamonds.csv").unwrap();
    c.bench_function("parse_dsv diamonds.csv", |b| {
        b.iter(|| black_box(parse_dsv(',', file)))
    });

    let file = &std::fs::read_to_string("aus-energy-2020.csv").unwrap();
    c.bench_function("parse_dsv aus-energy-2020.csv", |b| {
        b.iter(|| black_box(parse_dsv(',', file)))
    });
}

criterion_group!(benches, adding_rows, cloning, parse_csv);
criterion_main!(benches);
