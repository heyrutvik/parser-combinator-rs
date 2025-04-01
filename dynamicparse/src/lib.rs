use std::rc::Rc;

pub mod combinator;
pub mod helper;

pub type Parser<'a, T> = Rc<dyn Fn(&'a str) -> Option<(T, &'a str)> + 'a>;
