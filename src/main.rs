use std::fs::File;
use std::io::{prelude::*, BufReader, Error, ErrorKind, Result};

fn extract_field(s: &str, offset: usize) -> Result<(String, usize)> {
    let s = &s[offset..];
    if s.len() == 0 {
        return Ok((String::with_capacity(0), 0));
    }

    let mut v = String::new();
    let mut chars = s.chars().enumerate().peekable();
    let mut quoted = false;
    let mut n = 0;

    while let Some((i, c)) = chars.next() {
        n += 1;
        if i == 0 && c == '"' {
            quoted = true;
        } else if !quoted {
            if c == '"' {
                quoted = true;
                break;
            } else if c != ',' {
                v.push(c);
            } else {
                break;
            }
        } else if c != '"' {
            v.push(c);
        } else {
            match chars.peek() {
                None => {
                    quoted = false;
                    break;
                }
                Some((_, nc)) => {
                    if *nc == '"' {
                        v.push('"');
                        chars.next();
                        n += 1;
                    } else if *nc != ',' {
                        break;
                    } else {
                        n += 1;
                        quoted = false;
                        break;
                    }
                }
            }
        }
    }

    if quoted {
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!("{}", offset + n),
        ));
    }

    Ok((v, n))
}

fn parse_line(n: usize, line: &str) -> Result<Vec<String>> {
    println!("{:4}: {}", n, line);

    let mut start = 0;
    let mut vs = Vec::new();

    loop {
        let (v, n) = match extract_field(&line, start) {
            Ok(tup) => tup,
            Err(e) => {
                return Err(Error::new(
                    e.kind(),
                    format!("invalid character at line {} column {}", n, e),
                ))
            }
        };
        if n == 0 {
            println!("{:?}", vs);
            break;
        }
        start += n;
        vs.push(v);
    }

    Ok(vs)
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 2 {
        return Err(Error::new(ErrorKind::InvalidInput, "expecting a file"));
    }

    let f = File::options().read(true).open(&args[1])?;
    let lines = BufReader::new(f)
        .lines()
        .filter_map(|line| line.ok())
        .enumerate();

    for (i, line) in lines {
        parse_line(i + 1, &line)?;
    }

    Ok(())
}

#[test]
fn valid_input() {
    let lines = vec![
        "foo,\"bar,baz\"",
        "\"aaa\",\"b\"\"bb\",\"ccc\"",
        "\"aaa\",,,\"b\"\",\"\"bb\",ccc",
    ];
    let expected = vec![
        vec!["foo".to_string(), "bar,baz".to_string()],
        vec!["aaa".to_string(), "b\"bb".to_string(), "ccc".to_string()],
        vec![
            "aaa".to_string(),
            "".to_string(),
            "".to_string(),
            "b\",\"bb".to_string(),
            "ccc".to_string(),
        ],
    ];

    lines
        .iter()
        .zip(expected.into_iter())
        .enumerate()
        .for_each(|(i, (line, ans))| assert_eq!(parse_line(i + 1, line).ok(), Some(ans)))
}

#[test]
fn invalid_input() {
    let lines = vec!["foo,\"bar\",\"baz\"\"", "\"aaa\",bbb,\"c\"cc\""];
    let expected = vec![
        "invalid character at line 1 column 16".to_string(),
        "invalid character at line 2 column 13".to_string(),
    ];

    for (i, (line, ans)) in lines.iter().zip(expected.into_iter()).enumerate() {
        let res = parse_line(i + 1, line).err().map(|e| format!("{}", e));
        assert_eq!(res, Some(ans));
    }
}
