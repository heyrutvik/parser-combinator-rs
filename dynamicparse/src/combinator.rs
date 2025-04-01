use crate::Parser;
use std::rc::Rc;

pub fn and<'a, T: 'a, U: 'a>(parser1: Parser<'a, T>, parser2: Parser<'a, U>) -> Parser<'a, (T, U)> {
    Rc::new(move |s| parser1(s).and_then(|(v1, r1)| parser2(r1).map(|(v2, r2)| ((v1, v2), r2))))
}

pub fn or<'a, T: 'a>(parser1: Parser<'a, T>, parser2: Parser<'a, T>) -> Parser<'a, T> {
    Rc::new(move |s| match parser1(s) {
        r @ Some(_) => r,
        _ => parser2(s),
    })
}

pub fn many<'a, T: 'a>(parser: Parser<'a, T>) -> Parser<'a, Vec<T>> {
    Rc::new(move |s| {
        let mut input = s;
        let mut vs = Vec::new();
        while let Some((v, r)) = parser(input) {
            vs.push(v);
            input = r;
        }
        Some((vs, input))
    })
}

pub fn many1<'a, T: 'a>(parser: Parser<'a, T>) -> Parser<'a, Vec<T>> {
    filter(many(parser.clone()), |vs| !vs.is_empty())
}

pub fn skip<'a, T: 'a>(parser: Parser<'a, T>) -> Parser<'a, ()> {
    map(parser, |_| ())
}

pub fn between<'a, T: 'a, U: 'a, V: 'a>(
    parser: Parser<'a, T>,
    start: Parser<'a, U>,
    end: Parser<'a, V>,
) -> Parser<'a, T> {
    Rc::new(move |s| {
        let (_, r) = start(s)?;
        let (v, r) = parser(r)?;
        let (_, r) = end(r)?;
        Some((v, r))
    })
}

pub fn sep_by<'a, T: 'a, U: 'a>(parser: Parser<'a, T>, sep: Parser<'a, U>) -> Parser<'a, Vec<T>> {
    Rc::new(move |s| {
        let mut vs = Vec::new();
        let mut input = s;

        if let Some((v, r)) = parser(input) {
            vs.push(v);
            input = r;
        } else {
            return None;
        }

        while let Some((_, r1)) = sep(input) {
            if let Some((v, r2)) = parser(r1) {
                vs.push(v);
                input = r2;
            } else {
                break;
            }
        }

        Some((vs, input))
    })
}

pub fn filter<'a, T: 'a, F>(parser: Parser<'a, T>, pred: F) -> Parser<'a, T>
where
    F: Fn(&T) -> bool + 'a,
{
    Rc::new(move |s| parser(s).filter(|(v, _)| pred(v)))
}

pub fn map<'a, T: 'a, U, F>(parser: Parser<'a, T>, func: F) -> Parser<'a, U>
where
    F: Fn(T) -> U + 'a,
{
    Rc::new(move |s| parser(s).map(|(v, r)| (func(v), r)))
}

pub fn optional<'a, T: 'a>(parser: Parser<'a, T>) -> Parser<'a, Option<T>> {
    Rc::new(move |s| {
        parser(s)
            .map(|(v, r)| (Some(v), r))
            .or_else(|| Some((None, s)))
    })
}

pub fn except<'a, T: PartialEq + 'a>(
    parser1: Parser<'a, T>,
    parser2: Parser<'a, T>,
) -> Parser<'a, T> {
    Rc::new(move |s| {
        let (v1, r) = parser1(s)?;
        if parser2(s).map_or(false, |(v2, _)| v1 == v2) {
            return None;
        }
        Some((v1, r))
    })
}
