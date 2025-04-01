use crate::Parser;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct AndThen<P1, P2> {
    pub(crate) parser1: P1,
    pub(crate) parser2: P2,
}

impl<P1, P2, T, U> Parser<(T, U)> for AndThen<P1, P2>
where
    P1: Parser<T>,
    P2: Parser<U>,
{
    fn parse<'a>(&self, s: &'a str) -> Option<((T, U), &'a str)> {
        self.parser1
            .parse(s)
            .and_then(|(v1, r1)| self.parser2.parse(r1).map(|(v2, r2)| ((v1, v2), r2)))
    }
}

#[derive(Clone)]
pub struct OrElse<P1, P2> {
    pub(crate) parser1: P1,
    pub(crate) parser2: P2,
}

impl<P1, P2, T> Parser<T> for OrElse<P1, P2>
where
    P1: Parser<T>,
    P2: Parser<T>,
{
    fn parse<'a>(&self, s: &'a str) -> Option<(T, &'a str)> {
        match self.parser1.parse(s) {
            r @ Some(_) => r,
            _ => self.parser2.parse(s),
        }
    }
}

#[derive(Clone)]
pub struct Optional<P> {
    pub(crate) parser: P,
}
impl<P, T> Parser<Option<T>> for Optional<P>
where
    P: Parser<T>,
{
    fn parse<'a>(&self, s: &'a str) -> Option<(Option<T>, &'a str)> {
        self.parser
            .parse(s)
            .map(|(v, r)| (Some(v), r))
            .or_else(|| Some((None, s)))
    }
}

#[derive(Clone)]
pub struct Map<P, F, T> {
    pub(crate) parser: P,
    pub(crate) func: F,
    pub(crate) _marker: PhantomData<T>,
}
impl<P, F, T, U> Parser<U> for Map<P, F, T>
where
    P: Parser<T>,
    F: Fn(T) -> U,
{
    fn parse<'a>(&self, s: &'a str) -> Option<(U, &'a str)> {
        self.parser.parse(s).map(|(v, r)| ((self.func)(v), r))
    }
}

#[derive(Clone)]
pub struct Filter<P, F> {
    pub(crate) parser: P,
    pub(crate) pred: F,
}
impl<P, F, T> Parser<T> for Filter<P, F>
where
    P: Parser<T>,
    F: Fn(&T) -> bool,
{
    fn parse<'a>(&self, s: &'a str) -> Option<(T, &'a str)> {
        self.parser.parse(s).filter(|(v, _)| (self.pred)(v))
    }
}

#[derive(Clone)]
pub struct Many<P> {
    pub(crate) parser: P,
}
impl<P, T> Parser<Vec<T>> for Many<P>
where
    P: Parser<T>,
{
    fn parse<'a>(&self, s: &'a str) -> Option<(Vec<T>, &'a str)> {
        let mut input = s;
        let mut vs = Vec::new();
        while let Some((v, r)) = self.parser.parse(input) {
            vs.push(v);
            input = r;
        }
        Some((vs, input))
    }
}

#[derive(Clone)]
pub struct Many1<P> {
    pub(crate) parser: P,
}
impl<P, T> Parser<Vec<T>> for Many1<P>
where
    P: Parser<T> + Clone,
{
    fn parse<'a>(&self, s: &'a str) -> Option<(Vec<T>, &'a str)> {
        self.parser
            .clone()
            .many()
            .filter(|vs| !vs.is_empty())
            .parse(s)
    }
}

#[derive(Clone)]
pub struct Skip<P, T> {
    pub(crate) parser: P,
    pub(crate) _marker: PhantomData<T>,
}
impl<P, T> Parser<()> for Skip<P, T>
where
    P: Parser<T> + Clone,
{
    fn parse<'a>(&self, s: &'a str) -> Option<((), &'a str)> {
        self.parser.clone().map(|_| ()).parse(s)
    }
}

#[derive(Clone)]
pub struct Except<P1, P2> {
    pub(crate) parser1: P1,
    pub(crate) parser2: P2,
}
impl<P1, P2, T> Parser<T> for Except<P1, P2>
where
    T: PartialEq,
    P1: Parser<T>,
    P2: Parser<T>,
{
    fn parse<'a>(&self, s: &'a str) -> Option<(T, &'a str)> {
        let (v1, r) = self.parser1.parse(s)?;
        if self.parser2.parse(s).map_or(false, |(v2, _)| v1 == v2) {
            return None;
        }
        Some((v1, r))
    }
}

#[derive(Clone)]
pub struct SepBy<P1, P2, U> {
    pub(crate) parser: P1,
    pub(crate) sep: P2,
    pub(crate) _marker: PhantomData<U>,
}
impl<P1, P2, T, U> Parser<Vec<T>> for SepBy<P1, P2, U>
where
    P1: Parser<T>,
    P2: Parser<U>,
{
    fn parse<'a>(&self, s: &'a str) -> Option<(Vec<T>, &'a str)> {
        let mut vs = Vec::new();
        let mut input = s;

        if let Some((v, r)) = self.parser.parse(input) {
            vs.push(v);
            input = r;
        } else {
            return None;
        }

        while let Some((_, r1)) = self.sep.parse(input) {
            if let Some((v, r2)) = self.parser.parse(r1) {
                vs.push(v);
                input = r2;
            } else {
                break;
            }
        }

        Some((vs, input))
    }
}

#[derive(Clone)]
pub struct Between<P1, P2, P3, U, V> {
    pub(crate) parser: P1,
    pub(crate) start: P2,
    pub(crate) end: P3,
    pub(crate) _marker: PhantomData<(U, V)>,
}
impl<P1, P2, P3, T, U, V> Parser<T> for Between<P1, P2, P3, U, V>
where
    P1: Parser<T>,
    P2: Parser<U>,
    P3: Parser<V>,
{
    fn parse<'a>(&self, s: &'a str) -> Option<(T, &'a str)> {
        let (_, r) = self.start.parse(s)?;
        let (v, r) = self.parser.parse(r)?;
        let (_, r) = self.end.parse(r)?;
        Some((v, r))
    }
}
