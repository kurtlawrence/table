use super::{Entry, Table};
use ::kserd::Number;
use rayon::prelude::*;

/// Parse a string and split on `delimiter` and new lines.
///
/// New lines represent new rows, splits on delimiter are the columns.
///
/// # Escaping
/// _Delimiters_ can be escaped by using quotes around cell values. The parsing is set up to be
/// **line-prioritised** (for performance reasons), this means that new lines take priority over
/// qutoes and are always respected.
///
/// # Panics
/// Panics is delimiter is not an ascii character.
pub fn parse_dsv(delimiter: char, data: &str) -> Table<&str> {
    let delimiter = {
        if !delimiter.is_ascii() {
            panic!("delimiter is expected to be an ascii character");
        }
        let mut b = [0];
        delimiter.encode_utf8(&mut b);
        b[0]
    };

    let mut lines = Vec::new();
    let mut s = data;
    while !s.is_empty() {
        let (line, rem) = parse_line2(delimiter, s);
        lines.push(line);
        s = rem;
    }

    //     let x: Vec<Vec<Entry<&str>>> = data
    //         .par_lines()
    //         .map(|line| parse_line(delimiter, line))
    //         .collect();

    lines.into()
}

fn parse_line(delimiter: u8, line: &str) -> Vec<Entry<&str>> {
    fn to_str(bytes: &[u8]) -> &str {
        // we know this is safe as we are converting _from_ a utf8 str (and the delimiter is a byte)
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }

    let mut entries = Vec::new();
    let mut line = line.as_bytes();

    let quote_byte = b'"';
    let quote_ch = '"';

    while !line.is_empty() {
        let (entry, remaining) = quoted_str(line, delimiter, quote_byte);

        line = if remaining.get(0) == Some(&delimiter) {
            &remaining[1..]
        } else {
            remaining
        };

        let entry = to_str(entry).trim_end().trim_matches(quote_ch);
        entries.push(map_entry(entry));
    }

    entries
}

fn parse_line2(delimiter: u8, s: &str) -> (Vec<Entry<&str>>, &str) {
    fn to_str(bytes: &[u8]) -> &str {
        // we know this is safe as we are converting _from_ a utf8 str (and the delimiter is a byte)
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }

    let mut entries = Vec::new();

    let quote_byte = b'"';
    let quote_ch = '"';

    let mut s = s.as_bytes();

    loop {
        if s.is_empty() {
            break (entries, "");
        }

        if let Some(rem) = strip_nl(s) {
            break (entries, to_str(rem));
        }

        let (entry, remaining) = quoted_str(s, delimiter, quote_byte);

        s = if remaining.get(0) == Some(&delimiter) {
            &remaining[1..]
        } else {
            remaining
        };

        let entry = to_str(entry).trim_end().trim_matches(quote_ch);
        entries.push(map_entry(entry));
    }
}

fn strip_nl(string: &[u8]) -> Option<&[u8]> {
    string
        .strip_prefix(b"\n")
        .or_else(|| string.strip_prefix(b"\r\n"))
}

/// Assumes `delimiter` and `quot` are valid characters.
/// Returns the slice up _until the first **unquoted** delimiter_, and the remaining slice.
fn quoted_str(line: &[u8], delimiter: u8, quot: u8) -> (&[u8], &[u8]) {
    let mut i = {
        let start = line
            .iter()
            .take_while(|&&b| b.is_ascii_whitespace() && b != b'\n' && b != b'\r')
            .count();
        &line[start..]
    };

    let mut escaped = i.get(0) == Some(&quot);
    if escaped {
        i = &i[1..];
    }

    for (idx, &ch) in i.iter().enumerate() {
        if !escaped && (ch == delimiter || ch == b'\n' || ch == b'\r') {
            return (&i[..idx], &i[idx..]);
        } else if ch == quot {
            escaped = false;
        }
    }

    (i, &[])
}

fn map_entry(s: &str) -> Entry<&str> {
    if s.is_empty() {
        Entry::Nil
    } else if let Ok(x) = s.parse::<Number>() {
        Entry::Num(x)
    } else {
        Entry::Obj(s)
    }
}

#[cfg(test)]
mod tests {
    use super::parse_dsv as parse;
    use super::*;
    use Entry::*;

    fn all_str(rows: Vec<Vec<&str>>) -> Table<&str> {
        let mut repr: Table<&str> = Default::default();
        repr.add_rows(rows.into_iter().map(|x| x.into_iter().map(Obj)));
        repr
    }

    #[test]
    fn empty() {
        let s = "";
        assert_eq!(parse(',', s), Table::new());
        let s = "   \t    \t    \n       ";
        let mut repr = Table::new();
        repr.add_row(vec![Nil].into_iter());
        repr.add_row(vec![Nil].into_iter());
        assert_eq!(parse(',', s), repr);
    }

    #[test]
    fn simple() {
        let repr = all_str(vec![
            vec!["dog", "cat", "mouse"],
            vec!["lion", "hyena", "elephant"],
        ]);

        let s = "dog,cat,mouse
lion,hyena,elephant";
        assert_eq!(parse(',', s), repr);

        let s = "dog|cat|mouse
lion|hyena|elephant";
        assert_eq!(parse('|', s), repr);

        let s = "dog cat mouse
lion hyena elephant";
        assert_eq!(parse(' ', s), repr);
    }

    #[test]
    fn numbers() {
        let mut repr = Table::new();
        repr.add_row(vec![Num(1.into()), Num((-2i8).into())].into_iter());
        repr.add_row(vec![Num(3.14e7.into()), Num((-1.1).into())].into_iter());

        let s = " 1 ,  -2   
   3.14e7   , -1.1  ";
        assert_eq!(parse(',', s), repr);
    }

    #[test]
    fn test_blanks() {
        let s = "Hello,,world

        whats,,up

        ";

        let mut repr = Table::new();
        repr.add_rows(
            vec![
                vec![Obj("Hello".into()), Nil, Obj("world".into())],
                vec![],
                vec![Obj("whats".into()), Nil, Obj("up".into())],
                vec![],
                vec![],
            ]
            .into_iter()
            .map(|x| x.into_iter()),
        );

        assert_eq!(parse(',', s), repr);
    }

    #[test]
    fn complex() {
        let s = r#""Hello, world!", 101, , "Nested " Quote"
        1,2
        ,,,     "last""#;

        let mut repr = Table::new();
        repr.add_rows(
            vec![
                vec![
                    Obj("Hello, world!".into()),
                    Num(101.into()),
                    Nil,
                    Obj("Nested \" Quote".into()),
                ],
                vec![Num(1.into()), Num(2.into())],
                vec![Nil, Nil, Nil, Obj("last".into())],
            ]
            .into_iter()
            .map(|x| x.into_iter()),
        );

        assert_eq!(parse(',', s), repr);
    }

    #[test]
    fn leading_blanks() {
        let s = ",Missing,Heading
,One,
Two,Three,Four";
        let mut table = Table::new();
        let o = |i| Obj(i);
        table.add_rows(
            vec![
                vec![Nil, o("Missing"), o("Heading")],
                vec![Nil, o("One"), Nil],
                vec![o("Two"), o("Three"), o("Four")],
            ]
            .into_iter()
            .map(|x| x.into_iter()),
        );

        assert_eq!(parse(',', s), table);
    }

    #[test]
    fn quoted_new_lines() {
        let s = "\"Hello
world\",Yo";
        let mut table = Table::new();
        let o = |i| Obj(i);
        table.add_rows(
            vec![vec![o("Hello\nworld"), o("Yo")]]
                .into_iter()
                .map(|x| x.into_iter()),
        );

        assert_eq!(parse(',', s), table);

        let s = "\"Hello\r\nworld\",Yo";
        let mut table = Table::new();
        let o = |i| Obj(i);
        table.add_rows(
            vec![vec![o("Hello\r\nworld"), o("Yo")]]
                .into_iter()
                .map(|x| x.into_iter()),
        );

        assert_eq!(parse(',', s), table);
    }
}
