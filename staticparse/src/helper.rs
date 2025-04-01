use crate::Parser;
use std::ops::RangeBounds;

#[derive(Clone)]
pub struct CharParser;
impl Parser<char> for CharParser {
    fn parse<'a>(&self, s: &'a str) -> Option<(char, &'a str)> {
        let mut chars = s.chars();
        match chars.next() {
            Some(c) => Some((c, chars.as_str())),
            _ => None,
        }
    }
}

pub fn digit() -> impl Parser<u8> + Clone {
    CharParser
        .filter(|parsed| parsed.is_ascii_digit())
        .map(|c| c.to_digit(10).unwrap() as u8)
}

pub fn letter() -> impl Parser<char> + Clone {
    CharParser.filter(|parsed| parsed.is_ascii_alphabetic())
}

pub fn character(c: char) -> impl Parser<char> + Clone {
    CharParser.filter(move |&parsed| parsed == c)
}

pub fn character_range<R>(r: R) -> impl Parser<char> + Clone
where
    R: RangeBounds<char> + Clone,
{
    CharParser.filter(move |parsed| r.contains(parsed))
}

#[derive(Clone)]
pub struct TokenParser<'b> {
    token: &'b str,
}

impl<'b> Parser<String> for TokenParser<'b> {
    fn parse<'a>(&self, s: &'a str) -> Option<(String, &'a str)> {
        if let Some(r) = s.strip_prefix(self.token) {
            let v = s[..self.token.len()].to_owned();
            Some((v, r))
        } else {
            None
        }
    }
}

pub fn token<'a>(s: &'a str) -> impl Parser<String> + Clone + 'a {
    TokenParser { token: s }
}
