use crate::combinator::*;
use std::marker::PhantomData;

mod combinator;
pub mod helper;

pub trait Parser<T>: Sized {
    fn parse<'a>(&self, s: &'a str) -> Option<(T, &'a str)>;

    fn and_then<P, U>(self, next: P) -> AndThen<Self, P>
    where
        P: Parser<U>,
    {
        AndThen {
            parser1: self,
            parser2: next,
        }
    }

    fn or_else<P>(self, alternative: P) -> OrElse<Self, P>
    where
        P: Parser<T>,
    {
        OrElse {
            parser1: self,
            parser2: alternative,
        }
    }

    fn optional(self) -> Optional<Self> {
        Optional { parser: self }
    }

    fn map<F, U>(self, f: F) -> Map<Self, F, T>
    where
        F: Fn(T) -> U,
    {
        Map {
            parser: self,
            func: f,
            _marker: PhantomData,
        }
    }

    fn filter<F>(self, f: F) -> Filter<Self, F>
    where
        F: Fn(&T) -> bool,
    {
        Filter {
            parser: self,
            pred: f,
        }
    }

    fn many(self) -> Many<Self> {
        Many { parser: self }
    }

    fn many1(self) -> Many1<Self> {
        Many1 { parser: self }
    }

    fn skip(self) -> Skip<Self, T> {
        Skip {
            parser: self,
            _marker: PhantomData,
        }
    }

    fn except<P>(self, p: P) -> Except<Self, P>
    where
        P: Parser<T>,
    {
        Except {
            parser1: self,
            parser2: p,
        }
    }

    fn sep_by<P, U>(self, sep: P) -> SepBy<Self, P, U>
    where
        P: Parser<U>,
    {
        SepBy {
            parser: self,
            sep,
            _marker: PhantomData,
        }
    }

    fn between<P2, P3, U, V>(self, start: P2, end: P3) -> Between<Self, P2, P3, U, V>
    where
        P2: Parser<U>,
        P3: Parser<V>,
    {
        Between {
            parser: self,
            start,
            end,
            _marker: PhantomData,
        }
    }
}
