//! Generic data table abstraction.
#![warn(missing_docs)]

mod entry;
mod parsing;
#[cfg(test)]
mod tests;

pub use ::kserd::Number;
pub use entry::Entry;
use rayon::prelude::*;
use std::{cmp::*, iter::*};
use Entry::*;

pub use crate::parsing::parse_dsv;

/// The main data table type.
///
/// Most representations of a [`Table`] use a string based type for the object entry.
/// This type alias provides a convenience type for use throughout `daedalus`.
pub type DataTable = Table<::divvy::Str>;

// ########### TABLE #####################################################################
/// A table of data.
#[derive(Debug, PartialEq, Hash)]
pub struct Table<T> {
    data: Vec<Vec<Entry<T>>>,
    /// Treat the first row as a header row. Defaults to `true`.
    pub header: bool,
    cols: usize,
}

impl<T> Table<T> {
    /// Construct a new table representation.
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            header: true,
            cols: 0,
        }
    }

    /// Set whether to treat the first row as a header row.
    ///
    /// ```rust
    /// # use table::*;
    /// let mut x: Table<()> = Table::new();
    /// assert_eq!(x.header, true);
    /// assert_eq!(x.set_header(false).header, false);
    /// ```
    pub fn set_header(&mut self, has: bool) -> &mut Self {
        self.header = has;
        self
    }

    /// Add a row of entries into the table.
    ///
    /// If the length of entries is less than the current number of columns, the entries will be
    /// padded out with [`Nil`]. If there are _more_ entries, all other rows will be padded.
    pub fn add_row<I, E>(&mut self, entries: I) -> &mut Self
    where
        I: Iterator<Item = E>,
        E: Into<Entry<T>>,
    {
        self.add_rows(once(entries))
    }

    /// Add rows of entries into the table.
    ///
    /// Similarly to [`Table::add_row`], rows will be padded to ensure consistent column length.
    pub fn add_rows<I, J, E>(&mut self, rows: I) -> &mut Self
    where
        I: Iterator<Item = J>,
        J: Iterator<Item = E>,
        E: Into<Entry<T>>,
    {
        let mut resize = false;
        let mut cols = self.cols;
        let map = rows.map(|row| {
            let mut vec = Vec::with_capacity(max(cols, row.size_hint().0));
            vec.extend(row.map(Into::into));

            let len = vec.len();
            resize |= len != cols;
            cols = max(len, cols);

            vec
        });

        self.data.extend(map);
        self.cols = cols; // this should be at the maximum length now (after map runs)!

        if resize {
            self.resize_cols();
        }

        self
    }

    fn resize_cols(&mut self) {
        let len = self.cols;
        self.data
            .iter_mut()
            .filter(|x| x.len() != len)
            .for_each(|x| x.resize_with(len, Default::default));
    }

    /// Insert a row of entries at the specified row index, shifting trailing rows down.
    ///
    /// # Panics
    /// Panics if `index` is outside the rows bounds.
    pub fn insert_row<I, E>(&mut self, index: usize, entries: I) -> &mut Self
    where
        I: Iterator<Item = E>,
        E: Into<Entry<T>>,
    {
        if index > self.rows_len() {
            panic_rows(index, self.rows_len());
        }

        let mut row: Vec<_> = entries.map(Into::into).collect();

        if row.len() < self.cols {
            row.resize_with(self.cols, Default::default);
        }

        let resize = row.len() != self.cols;

        self.cols = max(row.len(), self.cols);
        self.data.insert(index, row);

        if resize {
            self.resize_cols();
        }

        self
    }

    /// Add a column of entries into the table.
    ///
    /// If the length of entries is less than the current number of rows, the entries will be
    /// padded out with [`Nil`]. If there are _more_ entries, additional rows padded with [`Nil`]
    /// are added.
    pub fn add_col<I, E>(&mut self, entries: I) -> &mut Self
    where
        I: Iterator<Item = E>,
        E: Into<Entry<T>>,
    {
        self.add_cols(once(entries))
    }

    /// Add columns of entries into the table.
    ///
    /// Similarly to [`Table::add_col`], rows will be padded to ensure consistent row length.
    pub fn add_cols<I, J, E>(&mut self, cols: I) -> &mut Self
    where
        I: Iterator<Item = J>,
        J: Iterator<Item = E>,
        E: Into<Entry<T>>,
    {
        let mut cols = cols.collect::<Vec<_>>();

        let mut has_some = true;

        // map into a _row_ of entries
        // as a macro as closure can't capture the has_some variable
        macro_rules! next_row {
            () => {{
                cols.iter_mut().map(|x| {
                    let entry = x.next().map(Into::into);
                    has_some |= entry.is_some();
                    entry.unwrap_or_default()
                })
            }};
        }

        for row in self.data.iter_mut() {
            has_some = false;
            // if the col group is shorter than data, this
            // will just return defaults, which is to be expected
            // (padding out lower rows)
            row.extend(next_row!());
        }

        let newcols = self.cols + cols.len();

        if has_some {
            // the last data row had new cols,
            // this means there _may_ be more data, and the
            // only way to tell is to allocate a new vector =/
            loop {
                has_some = false;
                let mut v = Vec::with_capacity(newcols);
                v.resize_with(self.cols, Default::default);
                v.extend(next_row!());
                if has_some {
                    self.data.push(v);
                } else {
                    break;
                }
            }
        }

        self.cols = newcols;

        self
    }

    /// Insert a column of entries at the specified column index, shifting trailing columns right.
    ///
    /// # Panics
    /// Panics if `index` is outside the columns bounds.
    pub fn insert_col<I, E>(&mut self, index: usize, mut entries: I) -> &mut Self
    where
        I: Iterator<Item = E>,
        E: Into<Entry<T>>,
    {
        if index > self.cols_len() {
            panic_cols(index, self.cols_len());
        }

        for row in &mut self.data {
            row.insert(
                index,
                if let Some(entry) = entries.next() {
                    entry.into()
                } else {
                    Nil
                },
            );
        }

        self.cols += 1; // always inserts a column, even if empty

        for entry in entries {
            let mut v = Vec::with_capacity(self.cols);
            v.resize_with(index, Default::default);
            v.push(entry.into());
            v.resize_with(self.cols, Default::default);
            self.data.push(v);
        }

        self
    }

    /// Remove a row of entries at the specified row index, shifting trailing rows up.
    ///
    /// # Panics
    /// Panics if `index` is outside the rows bounds.
    pub fn remove_row(&mut self, index: usize) -> &mut Self {
        if index >= self.rows_len() {
            panic_rows(index, self.rows_len());
        }
        self.data.remove(index);
        if self.is_empty() {
            self.cols = 0;
        }
        self
    }

    /// Remove a column of entries at the specified column index, shifting trailing columns left.
    ///
    /// # Panics
    /// Panics if `index` is outside the columns bounds.
    pub fn remove_col(&mut self, index: usize) -> &mut Self {
        if index >= self.cols_len() {
            panic_cols(index, self.cols_len());
        }
        for row in &mut self.data {
            row.remove(index);
        }
        self.cols -= 1;
        self.remove_empty_row_entries();
        self
    }

    /// Remove a column of entries at the specified column index, shifting trailing columns left.
    ///
    /// # Panics
    /// Panics if `index` is outside the columns bounds.
    ///
    /// # Parallelisation
    /// Parallelised over the rows.
    pub fn remove_col_par(&mut self, index: usize) -> &mut Self
    where
        T: Send,
    {
        if index >= self.cols_len() {
            panic_cols(index, self.cols_len());
        }
        self.data
            .par_iter_mut()
            .for_each(|row| (row.remove(index), ()).1);
        self.cols -= 1;
        self.remove_empty_row_entries();
        self
    }

    /// Table has no data in it.
    ///
    /// ```rust
    /// # use table::*;
    /// let mut r: Table<()> = Table::default();
    /// assert_eq!(r.is_empty(), true);
    ///
    /// r.add_row(vec![Entry::Nil].into_iter());
    /// assert_eq!(r.is_empty(), false);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() || self.data.iter().all(|x| x.is_empty())
    }

    /// Table has no data below the header row.
    ///
    /// ```rust
    /// # use table::*;
    /// let mut r: Table<()> = Table::default();
    /// assert_eq!(r.is_data_empty(), true);
    ///
    /// r.add_row(vec![Entry::Nil].into_iter());
    /// assert_eq!(r.is_data_empty(), true);
    ///
    /// r.set_header(false);
    /// assert_eq!(r.is_data_empty(), false);
    ///
    /// r.set_header(true);
    /// r.add_row(vec![Entry::Nil].into_iter());
    /// assert_eq!(r.is_data_empty(), false);
    /// ```
    pub fn is_data_empty(&self) -> bool {
        match (self.is_empty(), self.header) {
            (true, _) => true,
            (false, true) => self.data.len() == 1,
            (false, false) => false,
        }
    }

    /// The number of rows of data, including the header.
    ///
    /// ```rust
    /// # use table::*;
    /// let mut r: Table<()> = Table::default();
    /// assert_eq!(r.rows_len(), 0);
    /// assert_eq!(r.cols_len(), 0);
    ///
    /// r.add_row(vec![Entry::Nil, Entry::Nil].into_iter());
    /// assert_eq!(r.rows_len(), 1);
    /// assert_eq!(r.cols_len(), 2);
    /// ```
    pub fn rows_len(&self) -> usize {
        self.data.len()
    }

    /// The number of columns of data.
    ///
    /// ```rust
    /// # use table::*;
    /// let mut r: Table<()> = Table::default();
    /// assert_eq!(r.rows_len(), 0);
    /// assert_eq!(r.cols_len(), 0);
    ///
    /// r.add_row(vec![Entry::Nil, Entry::Nil].into_iter());
    /// assert_eq!(r.rows_len(), 1);
    /// assert_eq!(r.cols_len(), 2);
    /// ```
    pub fn cols_len(&self) -> usize {
        self.cols
    }

    /// Retrieve a row of entries.
    ///
    /// ```rust
    /// # use table::*;
    /// use Entry::*;
    /// let mut r: Table<()> = Table::new();
    /// r.add_rows(
    ///     vec![
    ///         vec![Nil, Num(101.into())].into_iter(),
    ///         vec![Num(202.into()), Obj(())].into_iter()
    ///     ].into_iter()
    /// );
    ///
    /// let mut row = r.row(0).unwrap();
    /// assert_eq!(row.next(), Some(&Nil));
    /// assert_eq!(row.next(), Some(&Num(101.into())));
    /// ```
    pub fn row(&self, index: usize) -> Option<impl Iterator<Item = &Entry<T>>> {
        self.data.get(index).map(|x| x.iter())
    }

    /// Retrieve a row of mutable entries.
    pub fn row_mut(&mut self, index: usize) -> Option<impl Iterator<Item = &mut Entry<T>>> {
        self.data.get_mut(index).map(|x| x.iter_mut())
    }

    /// Retrieve a column of entries.
    ///
    /// ```rust
    /// # use table::*;
    /// use Entry::*;
    /// let mut r: Table<()> = Table::new();
    /// r.add_rows(
    ///     vec![
    ///         vec![Nil, Num(101.into())].into_iter(),
    ///         vec![Num(202.into()), Obj(())].into_iter()
    ///     ].into_iter()
    /// );
    ///
    /// let mut col = r.col(1).unwrap();
    /// assert_eq!(col.next(), Some(&Num(101.into())));
    /// assert_eq!(col.next(), Some(&Obj(())));
    /// ```
    pub fn col(&self, index: usize) -> Option<impl Iterator<Item = &Entry<T>>> {
        if index < self.cols_len() {
            Some(self.data.iter().filter_map(move |x| x.get(index)))
        } else {
            None
        }
    }

    /// Retrieve a column of mutable entries.
    pub fn col_mut(&mut self, index: usize) -> Option<impl Iterator<Item = &mut Entry<T>>> {
        if index < self.cols_len() {
            Some(self.data.iter_mut().filter_map(move |x| x.get_mut(index)))
        } else {
            None
        }
    }

    /// Iterate over the rows.
    ///
    /// ```rust
    /// # use table::*;
    /// use Entry::*;
    /// let mut r: Table<()> = Table::new();
    /// r.add_rows(
    ///     vec![
    ///         vec![Nil, Num(101.into())].into_iter(),
    ///         vec![Num(202.into()), Obj(())].into_iter()
    ///     ].into_iter()
    /// );
    ///
    /// let mut row = r.rows().next().unwrap();
    /// assert_eq!(row.next(), Some(&Nil));
    /// assert_eq!(row.next(), Some(&Num(101.into())));
    /// ```
    pub fn rows(&self) -> impl Iterator<Item = impl Iterator<Item = &Entry<T>>> {
        self.data.iter().map(|x| x.iter())
    }

    /// Iterator over columns.
    ///
    /// ```rust
    /// # use table::*;
    /// use Entry::*;
    /// let mut r: Table<()> = Table::new();
    /// r.add_rows(
    ///     vec![
    ///         vec![Nil, Num(101.into())].into_iter(),
    ///         vec![Num(202.into()), Obj(())].into_iter()
    ///     ].into_iter()
    /// );
    ///
    /// let mut col = r.cols().next().unwrap();
    /// assert_eq!(col.next(), Some(&Nil));
    /// assert_eq!(col.next(), Some(&Num(202.into())));
    /// ```
    pub fn cols(&self) -> impl Iterator<Item = impl Iterator<Item = &Entry<T>>> {
        (0..self.cols_len()).map(move |i| self.col(i).unwrap())
    }

    /// Map each entry `Entry<T> -> Entry<U>` using a morphism `Entry<T> -> Entry<U>`.
    ///
    /// The morphism can arbitarily change the variants of `Entry<T>`.
    ///
    /// # Parallelisation
    /// `map` preallocates the output table and does the mapping in a parallel fashion by row. This
    /// places extra constraints on generic parameters to gain performance benefits.
    ///
    /// Where possible, `map` and `map_obj` should be the preferred way of mutating the table.
    pub fn map<U, F>(self, f: F) -> Table<U>
    where
        T: Send,
        U: Send,
        F: Fn(Entry<T>) -> Entry<U> + Sync,
    {
        let mut data = alloc(self.rows_len(), self.cols_len());
        data.par_iter_mut()
            .zip_eq(self.data)
            .for_each(|(v, r)| v.extend(r.into_iter().map(&f)));

        Table {
            data,
            header: self.header,
            cols: self.cols,
        }
    }

    /// Map each entry `Entry<T> -> Entry<U>` using a morphism `T -> U`.
    ///
    /// The morphism only maps `Entry::Obj` variants; `Entry::Obj(T) -> Entry::Obj(U)`. `Nil` and
    /// `Num` entries remain the same. For a mapping from an entry see [`Table::map`].
    ///
    /// # Parallelisation
    /// `map_obj` preallocates the output table and does the mapping in a parallel fashion by row. This
    /// places extra constraints on generic parameters to gain performance benefits.
    ///
    /// Where possible, `map` and `map_obj` should be the preferred way of mutating the table.
    pub fn map_obj<U, F>(self, f: F) -> Table<U>
    where
        T: Send,
        U: Send,
        F: Fn(T) -> U + Sync,
    {
        self.map(|e| match e {
            Nil => Nil,
            Num(x) => Num(x),
            Obj(t) => Obj(f(t)),
        })
    }

    /// Map each entry `&Entry<T> -> Entry<U>` using a morphism `&Entry<T> -> Entry<U>`.
    ///
    /// # Parallelisation
    /// `map_ref` preallocates the output table and does the mapping in a parallel fashion by row. This
    /// places extra constraints on generic parameters to gain performance benefits.
    pub fn map_ref<U, F>(&self, f: F) -> Table<U>
    where
        T: Sync,
        U: Send,
        F: Fn(&Entry<T>) -> Entry<U> + Sync,
    {
        let mut data = alloc(self.rows_len(), self.cols_len());
        data.par_iter_mut()
            .zip_eq(&self.data)
            .for_each(|(v, r)| v.extend(r.iter().map(&f)));

        Table {
            data,
            header: self.header,
            cols: self.cols,
        }
    }

    /// Map each entry `&Entry<T> -> Entry<U>` using a morphism `&T -> U`.
    ///
    /// The morphism only maps `Entry::Obj` variants; `Entry::Obj(T) -> Entry::Obj(U)`. `Nil` and
    /// `Num` entries remain the same. For a mapping from an entry see [`Table::map_ref`].
    ///
    /// # Parallelisation
    /// `map_ref_obj` preallocates the output table and does the mapping in a parallel fashion by row.
    /// This places extra constraints on generic parameters to gain performance benefits.
    pub fn map_ref_obj<U, F>(&self, f: F) -> Table<U>
    where
        T: Sync,
        U: Send,
        F: Fn(&T) -> U + Sync,
    {
        self.map_ref(|e| match e {
            Nil => Nil,
            Num(x) => Num(*x),
            Obj(t) => Obj(f(t)),
        })
    }

    /// Retain rows that match the predicate `p`.
    ///
    /// The predicate supplies the row index and the row entries.
    pub fn retain_rows<P>(&mut self, p: P)
    where
        P: Fn(usize, std::slice::Iter<Entry<T>>) -> bool,
    {
        let mut idx = 0;
        self.data.retain(|row| {
            let keep = p(idx, row.iter());
            idx += 1;
            keep
        });
        if self.is_empty() {
            self.cols = 0;
        }
    }

    /// Clone the table with the explicit capacity used for columns (ie the backing rows vector
    /// size).
    ///
    /// This is useful for when the table _has_ to be cloned, _and_ it is known that more columns
    /// are to be added onto it.
    pub fn clone_with_col_capacity(&self, cap: usize) -> Self
    where
        T: Clone + Send + Sync,
    {
        let mut data = alloc(self.rows_len(), cap);
        let chunk_size = std::cmp::max(self.rows_len() / 4, 1);
        data.par_chunks_mut(chunk_size)
            .zip_eq(self.data.par_chunks(chunk_size))
            .for_each(|(a, b)| {
                a.iter_mut()
                    .zip(b)
                    .for_each(|(v, r)| v.extend(r.iter().cloned()))
            });

        Table {
            data,
            cols: self.cols,
            header: self.header,
        }
    }

    /// Sort _data_ rows by comparing entries in a column.
    ///
    /// Since [`Entry`] does not implement [`Ord`] (as there is no ordering between variants), the
    /// caller must define the ordering between entries.
    ///
    /// [`Table::sort`] is a _stable_ sort.
    ///
    /// # Panics
    /// Panics if `col` is outside the columns bounds.
    ///
    /// # Parallelisation
    /// `sort` uses parallelisation to efficiently sort table.
    pub fn sort<F>(&mut self, col: usize, ordering: F)
    where
        T: Send,
        F: Fn(&Entry<T>, &Entry<T>) -> std::cmp::Ordering + Sync,
    {
        if col >= self.cols_len() {
            panic_cols(col, self.cols_len());
        }

        let s = if self.header { 1 } else { 0 };
        let e = self.rows_len();
        self.data[s..e].par_sort_by(|a, b| ordering(&a[col], &b[col]));
    }

    /// Reverse the order of the _data_ rows, in place.
    pub fn reverse_rows(&mut self) {
        let s = if self.header { 1 } else { 0 };
        self.data[s..].reverse();
    }

    /// Reverse the order of the columns, in place.
    pub fn reverse_cols_par(&mut self)
    where
        T: Send,
    {
        self.data.par_iter_mut().for_each(|x| x.reverse());
    }

    /// Extracts the backing vector of the Table.
    pub fn into_raw(self) -> Vec<Vec<Entry<T>>> {
        self.data
    }

    /// Removes rows which contain an empty vector, this is needed on column removals.
    fn remove_empty_row_entries(&mut self) {
        self.data.retain(|r| !r.is_empty());
    }
}

impl<T> Default for Table<T> {
    fn default() -> Self {
        Self::new()
    }
}

// TODO once stabilisation lands, have a specialised drop implementation that drop the backing vec
// into another thread.
// default impl<T> Drop for Table<T> {
//     fn drop(&mut self) { }
// }
// /// Implementation of [`Drop`] swaps the backing vector into another thread to dispose of.
// ///
// /// This is done as even moderately size tables (say 10,000 rows) can have millisecond `drop` time.
// /// By spawning this into another thread, _large tables_ can be disposed of with impacting
// /// performance.
// ///
// /// This is only implemented on table objects that implement `Send + 'static`.
// impl<T> Drop for Table<T> where
// T: Send + 'static,
// {
//     fn drop(&mut self) {
//         let data = std::mem::take(&mut self.data);
//         std::thread::spawn(|| drop(data));
//     }
// }

/// Specialisation of turning a vector of vectors of [`Entry`]s into a `Table`.
///
/// This is useful when preallocation of the table structure can be done.
impl<T> From<Vec<Vec<Entry<T>>>> for Table<T> {
    fn from(vecs: Vec<Vec<Entry<T>>>) -> Self {
        let mut table = Table::default();
        table.data = vecs;
        table.cols = table.data.iter().map(|x| x.len()).max().unwrap_or_default();
        table.resize_cols();
        table
    }
}

/// # Parallelisation
/// `clone` preallocates the output table, and batches the clone operations into a parallel system.
impl<T: Clone + Send + Sync> Clone for Table<T> {
    fn clone(&self) -> Self {
        self.clone_with_col_capacity(self.cols_len())
    }
}

fn alloc<U>(rows: usize, cols: usize) -> Vec<Vec<Entry<U>>> {
    use std::iter::*;
    repeat_with(|| Vec::with_capacity(cols))
        .take(rows)
        .collect()
}

fn panic_rows(idx: usize, rows: usize) {
    panic!("index {} is outside bounds of table rows {}", idx, rows);
}

fn panic_cols(idx: usize, cols: usize) {
    panic!("index {} is outside bounds of table columns {}", idx, cols);
}
