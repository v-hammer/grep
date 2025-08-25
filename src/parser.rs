use std::iter::Peekable;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("dangling quantifier: '{0}' has no preceding expression at position {1}")]
    DanglingQuantifier(char, usize),
    #[error("invalid escape sequence: '\\' at position {0}")]
    InvalidEscape(usize),
    #[error("unterminated escape: '\\' at position {0}")]
    LackEscape(usize),
    #[error("unterminated character class, missing ']' starting at position {0}")]
    UnterminatedCharClass(usize),
    #[error("unterminated group, missing ')' starting at position {0}")]
    UnterminatedGroup(usize),
}

#[derive(Debug)]
pub enum Quantifier {
    ZeroOrOne,
    OneOrMore,
    ZeroOrMore,
}

#[derive(Debug)]
pub enum RE {
    Literal(char),
    Dot,
    Digit,
    Word,
    CharClass(Vec<char>),
    NegClass(Vec<char>),
    Repeat(Box<RE>, Quantifier),
    Alt(Vec<RE>),
    Group(usize, Box<RE>),
    BackRef(usize),
    AnchorStart,
    AnchorEnd,
    Seq(Vec<RE>),
}

pub fn parse(raw_re: &str) -> Result<RE, ParseError> {
    let mut iter = raw_re.char_indices().peekable();
    let mut group_index = 0;
    parse_seq(&mut iter, &mut group_index)
}

fn parse_seq<I>(iter: &mut Peekable<I>, group_index: &mut usize) -> Result<RE, ParseError>
where
    I: Iterator<Item = (usize, char)>,
{
    let mut seq: Vec<RE> = Vec::new();

    while let Some(&(pos, c)) = iter.peek() {
        let re = match c {
            '^' => { iter.next(); RE::AnchorStart }
            '$' => { iter.next(); RE::AnchorEnd }
            '.' => { iter.next(); RE::Dot }
            '*' | '+' | '?' => {
                iter.next();
                if let Some(last) = seq.pop() {
                    let q = match c {
                        '*' => Quantifier::ZeroOrMore,
                        '+' => Quantifier::OneOrMore,
                        '?' => Quantifier::ZeroOrOne,
                        _ => unreachable!(),
                    };
                    RE::Repeat(Box::new(last), q)
                } else {
                    return Err(ParseError::DanglingQuantifier(c, pos));
                }
            }
            '\\' => {
                let start_pos = pos;
                iter.next();
                if let Some(&(_, next)) = iter.peek() {
                    let r = match next {
                        'd' => RE::Digit,
                        'w' => RE::Word,
                        '1'..='9' => RE::BackRef(next.to_digit(10).unwrap() as usize),
                        _ => return Err(ParseError::InvalidEscape(pos + 1)),
                    };
                    iter.next();
                    r
                } else {
                    return Err(ParseError::LackEscape(start_pos));
                }
            }
            '[' => {
                let start_pos = pos;
                iter.next();
                let mut chars = Vec::new();
                let mut neg = false;

                if let Some(&(_, c2)) = iter.peek() {
                    if c2 == '^' {
                        neg = true;
                        iter.next();
                    }
                }

                loop {
                    match iter.peek() {
                        Some(&(_, ']')) => { iter.next(); break; }
                        Some(&(_, ch)) => { chars.push(ch); iter.next(); }
                        None => return Err(ParseError::UnterminatedCharClass(start_pos)),
                    }
                }

                if neg { RE::NegClass(chars) } else { RE::CharClass(chars) }
            }
            '(' => {
                iter.next();
                parse_group(iter, group_index)?
            }
            ')' | '|' => break,
            _ => { iter.next(); RE::Literal(c) },
        };
        seq.push(re);
    }

    Ok(RE::Seq(seq))
}

fn parse_group<I>(iter: &mut Peekable<I>, group_index: &mut usize) -> Result<RE, ParseError>
where
    I: Iterator<Item = (usize, char)>,
{
    *group_index += 1;
    let my_idx = *group_index;
    let mut alts: Vec<RE> = Vec::new();

    loop {
        let seq = parse_seq(iter, group_index)?;
        alts.push(seq);

        match iter.peek() {
            Some(&(_, '|')) => { iter.next(); }
            Some(&(_, ')')) => { iter.next(); break; }
            None => return Err(ParseError::UnterminatedGroup(0)),
            _ => break,
        }
    }

    let inner = if alts.len() == 1 { alts.remove(0) } else { RE::Alt(alts) };
    Ok(RE::Group(my_idx, Box::new(inner)))
}