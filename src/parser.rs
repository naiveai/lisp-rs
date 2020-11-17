use std::{error::Error, fmt};

pub fn parse_sexpr(code: &str) -> Result<Node, SexprSyntaxError> {
    parse_tokens(&tokenize(code))
}

fn parse_tokens(tokens: &[String]) -> Result<Node, SexprSyntaxError> {
    if let Some(first) = tokens.first() {
        if first != "(" {
            return if tokens.len() == 1 && first != ")" {
                Ok(Node::Atom(parse_atom(first)))
            } else {
                Err(SexprSyntaxError::UnmatchedParen)
            };
        }
    } else {
        return Err(SexprSyntaxError::Empty);
    }

    if tokens.last().unwrap() != ")" {
        return Err(SexprSyntaxError::UnmatchedParen);
    }

    let inner_sexpr_tokens = &tokens[1..tokens.len() - 1];

    let mut parsed_list = vec![];
    let mut element_start = 0;

    while element_start < inner_sexpr_tokens.len() {
        let token = &inner_sexpr_tokens[element_start];

        let element_end = element_start
            + if token == "(" {
                find_matching_paren(&inner_sexpr_tokens[element_start..])
                    .ok_or(SexprSyntaxError::UnmatchedParen)?
            } else {
                0
            };

        parsed_list.push(parse_tokens(
            &inner_sexpr_tokens[element_start..=element_end],
        )?);

        element_start = element_end + 1;
    }

    Ok(Node::List(parsed_list))
}

fn find_matching_paren(tokens: &[String]) -> Option<usize> {
    let mut num_parens = 0;

    for (idx, token) in tokens.iter().enumerate() {
        if token == "(" {
            num_parens += 1;
        } else if token == ")" {
            if num_parens == 1 {
                return Some(idx);
            }

            num_parens -= 1;
        }
    }

    None
}

fn parse_atom(atom: &str) -> Atom {
    if let Ok(integer) = atom.parse() {
        Atom::Int(integer)
    } else if let Ok(float) = atom.parse() {
        Atom::Float(float)
    } else {
        Atom::Symbol(atom.to_owned())
    }
}

fn tokenize(code: &str) -> Vec<String> {
    code.trim()
        .replace("(", " ( ")
        .replace(")", " ) ")
        .split_whitespace()
        .map(|token| token.to_owned())
        .collect()
}

#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum SexprSyntaxError {
    Empty,
    UnmatchedParen,
}

impl Error for SexprSyntaxError {}

impl fmt::Display for SexprSyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty S-expression provided"),
            Self::UnmatchedParen => write!(f, "Unmatched parentheses found"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Node {
    List(Vec<Node>),
    Atom(Atom),
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::List(v) => write!(
                f,
                "({})",
                v.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
            Self::Atom(a) => write!(f, "{}", a),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Atom {
    Int(isize),
    Float(f64),
    Symbol(String),
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Int(i) => write!(f, "{}", i),
            Self::Float(fl) => write!(f, "f{}", fl),
            Self::Symbol(s) => write!(f, "\"{}\"", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_sexpr, SexprSyntaxError, Atom::*, Node::*};

    #[test]
    fn empty_err() {
        assert_eq!(parse_sexpr("").unwrap_err(), SexprSyntaxError::Empty);
    }

    #[test]
    fn atom_int() {
        assert_eq!(parse_sexpr("1").unwrap(), Atom(Int(1)));
    }

    #[test]
    fn atom_float() {
        assert_eq!(parse_sexpr("1.5").unwrap(), Atom(Float(1.5)));
    }

    #[test]
    fn atom_str() {
        assert_eq!(
            parse_sexpr("hello").unwrap(),
            Atom(Symbol("hello".to_owned())),
        )
    }

    #[test]
    fn empty_list() {
        assert_eq!(parse_sexpr("()").unwrap(), List(vec![]),)
    }

    #[test]
    fn basic() {
        assert_eq!(
            parse_sexpr("(+ 1 2)").unwrap(),
            List(vec![
                Atom(Symbol("+".to_owned())),
                Atom(Int(1)),
                Atom(Int(2))
            ]),
        );
    }

    #[test]
    fn single_unmatched_paren_err() {
        assert_eq!(
            parse_sexpr("(").unwrap_err(),
            SexprSyntaxError::UnmatchedParen,
        )
    }

    #[test]
    fn single_unmatched_back_paren_err() {
        assert_eq!(
            parse_sexpr(")").unwrap_err(),
            SexprSyntaxError::UnmatchedParen,
        )
    }

    #[test]
    fn unmatched_paren_err() {
        assert_eq!(
            parse_sexpr("(+ 1 2").unwrap_err(),
            SexprSyntaxError::UnmatchedParen,
        );
    }

    #[test]
    fn unmatched_front_paren_err() {
        assert_eq!(
            parse_sexpr("+ 1 2)").unwrap_err(),
            SexprSyntaxError::UnmatchedParen,
        )
    }

    #[test]
    fn unmatched_extra_paren_err() {
        assert_eq!(
            parse_sexpr("(+ 1 2))").unwrap_err(),
            SexprSyntaxError::UnmatchedParen,
        )
    }

    #[test]
    fn unmatched_paren_nested_err() {
        assert_eq!(
            parse_sexpr("(+ (+ 3 4 2)").unwrap_err(),
            SexprSyntaxError::UnmatchedParen,
        );
    }

    #[test]
    fn unmatched_extra_paren_nested_err() {
        assert_eq!(
            parse_sexpr("(+ (+ 3 4 2)))").unwrap_err(),
            SexprSyntaxError::UnmatchedParen,
        )
    }

    #[test]
    fn basic_nested() {
        assert_eq!(
            parse_sexpr("(+ (+ 1 2) 3)").unwrap(),
            List(vec![
                Atom(Symbol("+".to_owned())),
                List(vec![
                    Atom(Symbol("+".to_owned())),
                    Atom(Int(1)),
                    Atom(Int(2))
                ]),
                Atom(Int(3))
            ],),
        );
    }

    #[test]
    fn multiple_nested() {
        assert_eq!(
            parse_sexpr("(+ (+ 1 2) (+ 3 4))").unwrap(),
            List(vec![
                Atom(Symbol("+".to_owned())),
                List(vec![
                    Atom(Symbol("+".to_owned())),
                    Atom(Int(1)),
                    Atom(Int(2))
                ]),
                List(vec![
                    Atom(Symbol("+".to_owned())),
                    Atom(Int(3)),
                    Atom(Int(4))
                ]),
            ],),
        );
    }

    #[test]
    fn long_opname_multiple_nested() {
        assert_eq!(
            parse_sexpr("(add (add 1 2) (add 3 4))").unwrap(),
            List(vec![
                Atom(Symbol("add".to_owned())),
                List(vec![
                    Atom(Symbol("add".to_owned())),
                    Atom(Int(1)),
                    Atom(Int(2)),
                ]),
                List(vec![
                    Atom(Symbol("add".to_owned())),
                    Atom(Int(3)),
                    Atom(Int(4))
                ]),
            ],),
        );
    }

    #[test]
    fn deeply_nested() {
        let sexpr = "(car (list 1 (+ 2 3) (* (+ 4 5) 6)))";

        let expected_inner = List(vec![
            Atom(Symbol("list".to_owned())),
            Atom(Int(1)),
            List(vec![
                Atom(Symbol("+".to_owned())),
                Atom(Int(2)),
                Atom(Int(3)),
            ]),
            List(vec![
                Atom(Symbol("*".to_owned())),
                List(vec![
                    Atom(Symbol("+".to_owned())),
                    Atom(Int(4)),
                    Atom(Int(5)),
                ]),
                Atom(Int(6)),
            ]),
        ]);

        assert_eq!(
            parse_sexpr(sexpr).unwrap(),
            List(vec![Atom(Symbol("car".to_owned())), expected_inner,])
        );
    }
}
