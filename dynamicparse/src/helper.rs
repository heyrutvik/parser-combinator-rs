use crate::combinator::*;
use crate::Parser;
use std::ops::RangeBounds;
use std::rc::Rc;

pub struct CharParser;
impl CharParser {
    pub fn make<'a>() -> Parser<'a, char> {
        Rc::new(|s| {
            let mut chars = s.chars();
            match chars.next() {
                Some(c) => Some((c, chars.as_str())),
                _ => None,
            }
        })
    }
}
pub fn character<'a>(c: char) -> Parser<'a, char> {
    filter(CharParser::make(), move |v| *v == c)
}
pub fn character_range<'a, R>(r: R) -> Parser<'a, char>
where
    R: RangeBounds<char> + 'a,
{
    filter(CharParser::make(), move |parsed| r.contains(parsed))
}
pub fn digit<'a>() -> Parser<'a, u8> {
    map(
        filter(CharParser::make(), |parsed| parsed.is_ascii_digit()),
        |c| c.to_digit(10).unwrap() as u8,
    )
}

pub struct Token {
    token: String,
}
impl<'a> Token {
    pub fn new(self) -> Parser<'a, String> {
        Rc::new(move |s| {
            if let Some(r) = s.strip_prefix(self.token.as_str()) {
                let v = s[..self.token.len()].to_owned();
                Some((v, r))
            } else {
                None
            }
        })
    }
}
pub fn token(s: &str) -> Parser<String> {
    Token {
        token: s.to_owned(),
    }
    .new()
}
