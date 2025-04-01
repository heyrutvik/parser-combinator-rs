use crate::json::Json;
use dynamicparse::combinator::*;
use dynamicparse::helper::*;
use dynamicparse::Parser;
use std::collections::HashMap;
use std::rc::Rc;

fn lazy<'a, T: 'a>(f: impl Fn() -> Parser<'a, T> + 'a) -> Parser<'a, T> {
    Rc::new(move |s| f()(s))
}

pub fn parse(s: &str) -> Option<Json> {
    element()(s).map(|(json, _)| json)
}

fn element<'a>() -> Parser<'a, Json> {
    lazy(|| between(value(), ws(), ws()))
}

pub fn value<'a>() -> Parser<'a, Json> {
    or(
        or(or(or(or(null(), boolean()), number()), string()), array()),
        object(),
    )
}

pub fn object<'a>() -> Parser<'a, Json> {
    let pair = map(
        and(
            and(and(and(ws(), string()), ws()), character(':')),
            element(),
        ),
        |((((_, key), _), _), value)| match key {
            Json::String(key) => (key, value),
            _ => panic!("we shouldn't be here."),
        },
    );
    let members = sep_by(pair, character(','));
    let empty_obj = map(between(ws(), character('{'), character('}')), |_| {
        HashMap::<String, Json>::new()
    });
    let non_empty_obj = map(between(members, character('{'), character('}')), |x| {
        x.into_iter().collect::<HashMap<String, Json>>()
    });
    map(or(empty_obj, non_empty_obj), |v| Json::Object(v))
}

pub fn array<'a>() -> Parser<'a, Json> {
    let empty_arr = map(between(ws(), character('['), character(']')), |_| {
        Vec::<Json>::new()
    });
    let non_empty_arr = between(
        sep_by(element(), character(',')),
        character('['),
        character(']'),
    );
    map(or(empty_arr, non_empty_arr), |vs| Json::Array(vs))
}

pub fn null<'a>() -> Parser<'a, Json> {
    map(token("null"), |_| Json::Null)
}

pub fn boolean<'a>() -> Parser<'a, Json> {
    map(or(token("true"), token("false")), |parsed| {
        Json::Bool(parsed.parse::<bool>().expect("couldn't parse bool"))
    })
}

pub fn string<'a>() -> Parser<'a, Json> {
    let hex = or(
        or(digit(), map(character_range('a'..='f'), |c| c as u8)),
        map(character_range('A'..='F'), |c| c as u8),
    );

    let unicode = map(
        and(
            and(
                and(and(character('u'), hex.clone()), hex.clone()),
                hex.clone(),
            ),
            hex.clone(),
        ),
        |((((_, a), b), c), d)| {
            let unicode = format!("{:X}{:X}{:X}{:X}", a, b, c, d);
            let unicode = u32::from_str_radix(&unicode, 16).unwrap();
            char::from_u32(unicode).unwrap()
        },
    );
    let escape = or(
        or(
            or(
                or(
                    or(
                        or(
                            or(or(character('"'), character('\\')), character('/')),
                            character('b'),
                        ),
                        character('f'),
                    ),
                    character('n'),
                ),
                character('r'),
            ),
            character('t'),
        ),
        unicode,
    );

    let json_valid_chars = except(
        character_range('\u{0020}'..='\u{10FFFF}'),
        or(character('"'), character('\\')),
    );
    let json_valid_escape = map(and(character('\\'), escape), |(_, b)| match b {
        'b' => '\u{0008}',
        'f' => '\u{000C}',
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        _ => b,
    });
    let json_character = or(json_valid_chars, json_valid_escape);
    let characters = many(json_character);

    map(
        and(and(character('"'), characters), character('"')),
        |((_, vs), _)| Json::String(String::from_iter(vs)),
    )
}

pub fn number<'a>() -> Parser<'a, Json> {
    let digits = map(many1(digit()), |ds| {
        ds.iter().fold(0_i32, |acc, &digit| acc * 10 + digit as i32)
    });
    let sign = or(character('+'), character('-'));
    let integer = or(
        digits.clone(),
        map(and(character('-'), digits.clone()), |(_, number)| -number),
    );
    let fraction = map(and(skip(character('.')), digits.clone()), |(_, num)| num);
    let exponent = and(
        skip(or(character('E'), character('e'))),
        map(and(sign, digits.clone()), |(s, n)| match s {
            '-' => -n,
            _ => n,
        }),
    );
    let p = map(
        and(and(integer, optional(fraction)), optional(exponent)),
        |((n, f), e)| match (n, f, e) {
            (n, Some(f), Some((_, e))) => format!("{}.{}E{}", n, f, e)
                .parse::<f64>()
                .expect("couldn't parse number"),
            (n, Some(f), _) => format!("{}.{}", n, f)
                .parse::<f64>()
                .expect("couldn't parse number"),
            (n, _, _) => f64::from(n),
        },
    );
    map(p, |v| Json::Number(v))
}

fn ws<'a>() -> Parser<'a, ()> {
    skip(many(or(
        or(or(character(' '), character('\n')), character('\r')),
        character('\t'),
    )))
}
