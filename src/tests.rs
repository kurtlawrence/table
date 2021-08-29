use super::*;
use ::divvy::Str;

#[test]
fn entry_default() {
    assert_eq!(Entry::<()>::default(), Nil);
}

#[test]
fn table_setting_header() {
    let mut x: Table<()> = Table::new();
    x.set_header(false);
    assert_eq!(x.header, false);
    x.set_header(true);
    assert_eq!(x.header, true);
}

#[test]
fn table_is_empty() {
    let mut r: Table<()> = Table::default();
    assert_eq!(r.is_empty(), true);

    r.add_row(vec![Entry::Nil].into_iter());
    assert_eq!(r.is_empty(), false);
}

#[test]
fn new_table_repr() {
    let x: Table<()> = Table::new();
    assert_eq!(x.data.is_empty(), true);
    assert_eq!(x.header, true);
    assert_eq!(x.cols, 0);
}

#[test]
fn table_repr_add() {
    let mut x: Table<()> = Table::new();

    // add a row
    x.add_row(vec![Nil, Num(101.into()), Obj(())].into_iter());
    assert_eq!(x.data, vec![vec![Nil, Num(101.into()), Obj(())]]);

    // add a short row
    x.add_row(vec![Obj(())].into_iter());
    assert_eq!(
        x.data,
        vec![vec![Nil, Num(101.into()), Obj(())], vec![Obj(()), Nil, Nil]]
    );

    // add a long row
    x.add_row(repeat(Obj(())).take(4));
    assert_eq!(
        x.data,
        vec![
            vec![Nil, Num(101.into()), Obj(()), Nil],
            vec![Obj(()), Nil, Nil, Nil],
            vec![Obj(()); 4]
        ]
    );
}

#[test]
fn table_add_rows() {
    let mut x: Table<()> = Table::new();

    let mut v = Vec::new();
    for i in 1..=100 {
        v.push(vec![Obj(()); i]);
    }

    x.add_rows(v.into_iter().map(|x| x.into_iter())); // borrowed version

    let ans: Vec<_> = (1..=100)
        .map(|i| {
            let mut v = vec![Obj(()); i];
            v.resize_with(100, Default::default);
            v
        })
        .collect();

    assert_eq!(x.data, ans);
}

#[test]
fn table_add_col() {
    let mut x: Table<()> = Table::default();

    x.add_col(once(Obj(())));
    assert_eq!(x.data, vec![vec![Obj(())]]);

    // longer col
    x.add_col(vec![Nil, Num(101.into()), Num(202.into())].into_iter());
    assert_eq!(
        x.data,
        vec![
            vec![Obj(()), Nil],
            vec![Nil, Num(101.into())],
            vec![Nil, Num(202.into())]
        ]
    );

    // shorter col
    x.add_col(once(Num(303.into())));
    assert_eq!(
        x.data,
        vec![
            vec![Obj(()), Nil, Num(303.into())],
            vec![Nil, Num(101.into()), Nil],
            vec![Nil, Num(202.into()), Nil]
        ]
    );
}

#[test]
fn table_add_cols() {
    let mut x = Table::new();

    let v: Vec<_> = (1..=100).map(|i| vec![Obj(()); i]).collect();
    x.add_cols(v.into_iter().map(|x| x.into_iter()));

    let ans: Vec<_> = (0..100)
        .map(|i| {
            let mut v = vec![Nil; i];
            v.resize(100, Obj(()));
            v
        })
        .collect();
    assert_eq!(x.data, ans);
}

#[test]
fn test_is_data_empty() {
    let mut r: Table<()> = Table::default();
    assert_eq!(r.is_data_empty(), true);

    r.add_row(vec![Entry::Nil].into_iter());
    assert_eq!(r.is_empty(), false);
    assert_eq!(r.is_data_empty(), true);

    r.set_header(false);
    assert_eq!(r.is_data_empty(), false);

    r.set_header(true);
    r.add_row(vec![Entry::Nil].into_iter());
    assert_eq!(r.is_data_empty(), false);
}

#[test]
fn test_lens() {
    let mut r: Table<()> = Table::default();
    assert_eq!(r.rows_len(), 0);
    assert_eq!(r.cols_len(), 0);

    r.add_row(vec![Nil, Nil].into_iter());
    assert_eq!(r.rows_len(), 1);
    assert_eq!(r.cols_len(), 2);
}

#[test]
fn test_iteration() {
    let mut r: Table<()> = Table::new();
    r.add_rows(
        vec![
            vec![Nil, Num(101.into())].into_iter(),
            vec![Num(202.into()), Obj(())].into_iter(),
        ]
        .into_iter(),
    );

    let mut row = r.row(0).unwrap();
    assert_eq!(row.next(), Some(&Nil));
    assert_eq!(row.next(), Some(&Num(101.into())));
    let mut row = r.row(1).unwrap();
    assert_eq!(row.next(), Some(&Num(202.into())));
    assert_eq!(row.next(), Some(&Obj(())));
    assert_eq!(r.col(2).is_none(), true);

    let mut col = r.col(0).unwrap();
    assert_eq!(col.next(), Some(&Nil));
    assert_eq!(col.next(), Some(&Num(202.into())));
    let mut col = r.col(1).unwrap();
    assert_eq!(col.next(), Some(&Num(101.into())));
    assert_eq!(col.next(), Some(&Obj(())));
    assert_eq!(r.col(2).is_none(), true);

    // iteration
    let mut rows = r.rows();
    let mut row = rows.next().unwrap();
    assert_eq!(row.next(), Some(&Nil));
    assert_eq!(row.next(), Some(&Num(101.into())));
    let mut row = rows.next().unwrap();
    assert_eq!(row.next(), Some(&Num(202.into())));
    assert_eq!(row.next(), Some(&Obj(())));
    assert_eq!(row.next().is_none(), true);

    let mut cols = r.cols();
    let mut col = cols.next().unwrap();
    assert_eq!(col.next(), Some(&Nil));
    assert_eq!(col.next(), Some(&Num(202.into())));
    let mut col = cols.next().unwrap();
    assert_eq!(col.next(), Some(&Num(101.into())));
    assert_eq!(col.next(), Some(&Obj(())));
    assert_eq!(cols.next().is_none(), true);
}

#[test]
fn entry_helpers() {
    let e = Entry::<()>::Nil;
    assert_eq!(e.is_nil(), true);
    assert_eq!(e.is_num(), false);

    let e = Entry::<()>::Num(101.into());
    assert_eq!(e.is_num(), true);
    assert_eq!(e.is_obj(), false);

    let e = Entry::Obj(());
    assert_eq!(e.is_obj(), true);
    assert_eq!(e.is_nil(), false);
}

#[test]
fn test_borrowing_example() {
    let mut x = Table::new();
    x.add_rows(
        vec![vec![Nil, Nil, Obj("Hello")], vec![Num(101.into())], vec![]]
            .into_iter()
            .map(|x| x.into_iter()),
    );

    let mut brw = Table::new();
    brw.add_rows(x.rows());

    assert_eq!(brw, x);
}

#[test]
fn inserting_rows_columns() {
    let mut repr = <Table<()>>::new();
    repr.insert_row(0, [Obj(()), Obj(())].iter());

    let mut exp = Table::new();
    exp.add_row([Obj(()), Obj(())].iter());
    assert_eq!(repr, exp);

    repr.insert_row(0, [Num(1.into())].iter());
    let mut exp = Table::new();
    exp.add_rows(vec![[Num(1.into()), Nil].iter(), [Obj(()), Obj(())].iter()].into_iter());
    assert_eq!(repr, exp);

    repr.insert_col(2, [Num(2.into())].iter());
    let mut exp = Table::new();
    exp.add_rows(
        vec![
            [Num(1.into()), Nil, Num(2.into())].iter(),
            [Obj(()), Obj(()), Nil].iter(),
        ]
        .into_iter(),
    );
    assert_eq!(repr, exp);

    repr.insert_col(1, [Obj(()), Num(3.into()), Obj(())].iter());
    let mut exp = Table::new();
    exp.add_rows(
        vec![
            [Num(1.into()), Obj(()), Nil, Num(2.into())].iter(),
            [Obj(()), Num(3.into()), Obj(()), Nil].iter(),
            [Nil, Obj(()), Nil, Nil].iter(),
        ]
        .into_iter(),
    );
    assert_eq!(repr, exp);

    repr.insert_row(
        1,
        [
            Num(0.into()),
            Num(1.into()),
            Num(2.into()),
            Num(3.into()),
            Num(4.into()),
        ]
        .iter(),
    );
    let mut exp = Table::new();
    exp.add_rows(
        vec![
            [Num(1.into()), Obj(()), Nil, Num(2.into()), Nil].iter(),
            [
                Num(0.into()),
                Num(1.into()),
                Num(2.into()),
                Num(3.into()),
                Num(4.into()),
            ]
            .iter(),
            [Obj(()), Num(3.into()), Obj(()), Nil, Nil].iter(),
            [Nil, Obj(()), Nil, Nil, Nil].iter(),
        ]
        .into_iter(),
    );
    assert_eq!(repr, exp);
}

#[test]
#[should_panic]
fn insert_row_panic() {
    let mut repr: Table<()> = Table::new();
    repr.insert_row(1, [Nil, Nil].iter());
}

#[test]
#[should_panic]
fn insert_col_panic() {
    let mut repr: Table<()> = Table::new();
    repr.insert_col(1, [Nil, Nil].iter());
}

#[test]
fn entry_as_str() {
    assert_eq!(Entry::<Str>::Nil.as_str(), "-");
    assert_eq!(Entry::<Str>::Num(3.14.into()).as_str(), "3.14");
    assert_eq!(Entry::<Str>::Obj("what".into()).as_str(), "what");
}

#[test]
fn remove_rows_columns() {
    let mut table = <Table<()>>::new();
    table.add_rows(
        vec![
            vec![Obj(()), Num(1.into()), Nil],
            vec![Num(2.into()), Nil, Obj(())],
            vec![Nil, Obj(()), Nil],
        ]
        .into_iter()
        .map(|x| x.into_iter()),
    );

    assert_eq!((table.rows_len(), table.cols_len()), (3, 3));

    table.remove_row(1);

    assert_eq!((table.rows_len(), table.cols_len()), (2, 3));
    assert_eq!(
        table.data,
        vec![vec![Obj(()), Num(1.into()), Nil], vec![Nil, Obj(()), Nil]]
    );

    table.remove_col(1);

    assert_eq!((table.rows_len(), table.cols_len()), (2, 2));
    assert_eq!(table.data, vec![vec![Obj(()), Nil], vec![Nil, Nil]]);

    table.remove_col(1);

    assert_eq!((table.rows_len(), table.cols_len()), (2, 1));
    // check that cols gets updated
    let mut t = Table::new();
    t.add_rows(
        vec![vec![Obj(())], vec![Nil]]
            .into_iter()
            .map(|x| x.into_iter()),
    );
    assert_eq!(table, t);
}

#[test]
#[should_panic]
fn remove_row_panic() {
    let mut table = <Table<()>>::new();
    table.remove_row(0);
}

#[test]
#[should_panic]
fn remove_col_panic() {
    let mut table = <Table<()>>::new();
    table.remove_col(0);
}

#[test]
fn entry_ordering() {
    use std::cmp::{Ordering::*, *};

    // Nil LHS
    let lhs: Entry<()> = Nil;
    let rhs = Nil;
    assert_eq!(lhs.partial_cmp(&rhs), Some(Equal));
    assert!(lhs == rhs);
    assert!(!(lhs > rhs));
    assert!(!(lhs < rhs));

    let lhs = Nil;
    let rhs = Obj("hello");
    assert_eq!(lhs.partial_cmp(&rhs), None);
    assert!(lhs != rhs);
    assert!(!(lhs > rhs));
    assert!(!(lhs < rhs));

    let lhs: Entry<&str> = Nil;
    let rhs = Num(101.into());
    assert_eq!(lhs.partial_cmp(&rhs), None);
    assert!(lhs != rhs);
    assert!(!(lhs > rhs));
    assert!(!(lhs < rhs));

    // Num LHS
    let lhs: Entry<&str> = Num(101.into());
    let rhs = Nil;
    assert_eq!(lhs.partial_cmp(&rhs), None);
    assert!(lhs != rhs);
    assert!(!(lhs > rhs));
    assert!(!(lhs < rhs));

    let lhs: Entry<&str> = Num(101.into());
    let rhs = Num(102.into());
    assert_eq!(lhs.partial_cmp(&rhs), Some(Less));
    assert!(lhs != rhs);
    assert!(!(lhs > rhs));
    assert!(lhs < rhs);

    let lhs: Entry<&str> = Num(101.into());
    let rhs = Obj("hello");
    assert_eq!(lhs.partial_cmp(&rhs), None);
    assert!(lhs != rhs);
    assert!(!(lhs > rhs));
    assert!(!(lhs < rhs));

    // Obj LHS
    let lhs: Entry<&str> = Obj("a");
    let rhs = Nil;
    assert_eq!(lhs.partial_cmp(&rhs), None);
    assert!(lhs != rhs);
    assert!(!(lhs > rhs));
    assert!(!(lhs < rhs));

    let lhs: Entry<&str> = Obj("a");
    let rhs = Num(102.into());
    assert_eq!(lhs.partial_cmp(&rhs), None);
    assert!(lhs != rhs);
    assert!(!(lhs > rhs));
    assert!(!(lhs < rhs));

    let lhs: Entry<&str> = Obj("a");
    let rhs = Obj("b");
    assert_eq!(lhs.partial_cmp(&rhs), Some(Less));
    assert!(lhs != rhs);
    assert!(!(lhs > rhs));
    assert!(lhs < rhs);
}

#[test]
fn table_from_vector_of_vectors() {
    let vs = vec![
        vec![Nil, Num(101.into()), Obj(())],
        vec![],
        vec![Num(202.into())],
    ];

    let table = Table::from(vs);

    assert_eq!(table.cols, 3);
    assert_eq!(table.header, true);
    assert_eq!(
        table.data,
        vec![
            vec![Nil, Num(101.into()), Obj(())],
            vec![Nil, Nil, Nil],
            vec![Num(202.into()), Nil, Nil]
        ]
    );
}

#[test]
fn test_cloning() {
    use Entry::*;

    let vecs = vec![
        vec![Nil, Obj(()), Num(101.into())],
        vec![Obj(()), Obj(()), Num(303.into())],
        vec![Obj(()), Num(303.into())],
    ];

    let table = Table::from(vecs.clone());

    let table_clone = table.clone();
    let table_2 = Table::from(vecs);

    assert_eq!(table, table_clone);
    assert_eq!(table, table_2);
    assert_eq!(table_2, table_clone);
}
