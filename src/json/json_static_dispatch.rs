use crate::json::Json;
use staticparse::helper::*;
use staticparse::Parser;
use std::collections::HashMap;

pub fn parse(s: &str) -> Option<Json> {
    element().parse(s).map(|(json, _)| json)
}

#[derive(Clone)]
pub struct ElementParser;
impl Parser<Json> for ElementParser {
    fn parse<'a>(&self, s: &'a str) -> Option<(Json, &'a str)> {
        ws().and_then(value())
            .and_then(ws())
            .map(|((_, v), _)| v)
            .parse(s)
    }
}
pub fn element() -> ElementParser {
    ElementParser
}

#[derive(Clone)]
pub struct ValueParser;
impl Parser<Json> for ValueParser {
    fn parse<'a>(&self, s: &'a str) -> Option<(Json, &'a str)> {
        token("null")
            .map(|_| Json::Null)
            .or_else(boolean().map(Json::Bool))
            .or_else(number().map(Json::Number))
            .or_else(string().map(Json::String))
            .or_else(array().map(Json::Array))
            .or_else(object().map(Json::Object))
            .parse(s)
    }
}
pub fn value() -> ValueParser {
    ValueParser
}

pub fn object() -> impl Parser<HashMap<String, Json>> + Clone {
    let pair = ws()
        .and_then(string())
        .and_then(ws())
        .and_then(character(':'))
        .and_then(element())
        .map(|((((_, key), _), _), value)| (key, value));
    let members = pair.sep_by(character(','));
    let empty_obj = ws()
        .between(character('{'), character('}'))
        .map(|_| HashMap::<String, Json>::new());
    let non_empty_obj = members
        .between(character('{'), character('}'))
        .map(|x| x.into_iter().collect());
    empty_obj.or_else(non_empty_obj)
}

pub fn array() -> impl Parser<Vec<Json>> + Clone {
    let empty_arr = ws()
        .between(character('['), character(']'))
        .map(|_| Vec::<Json>::new());
    let non_empty_arr = element()
        .sep_by(character(','))
        .between(character('['), character(']'));
    empty_arr.or_else(non_empty_arr)
}

pub fn boolean() -> impl Parser<bool> + Clone {
    token("true")
        .or_else(token("false"))
        .map(|parsed| parsed.parse::<bool>().expect("couldn't parse bool"))
}

pub fn string() -> impl Parser<String> + Clone {
    let hex = digit()
        .or_else(character_range('a'..='f').map(|c| c as u8))
        .or_else(character_range('A'..='F').map(|c| c as u8));

    let escape = character('"')
        .or_else(character('\\'))
        .or_else(character('/'))
        .or_else(character('b'))
        .or_else(character('f'))
        .or_else(character('n'))
        .or_else(character('r'))
        .or_else(character('t'))
        .or_else(
            character('u')
                .and_then(hex.clone())
                .and_then(hex.clone())
                .and_then(hex.clone())
                .and_then(hex.clone())
                .map(|((((_, a), b), c), d)| {
                    let unicode = format!("{:X}{:X}{:X}{:X}", a, b, c, d);
                    let unicode = u32::from_str_radix(&unicode, 16).unwrap();
                    char::from_u32(unicode).unwrap()
                }),
        );

    let json_valid_chars =
        character_range('\u{0020}'..='\u{10FFFF}').except(character('"').or_else(character('\\')));

    let json_valid_escape = character('\\').and_then(escape).map(|(_, b)| match b {
        'b' => '\u{0008}',
        'f' => '\u{000C}',
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        _ => b,
    });

    let json_character = json_valid_chars.or_else(json_valid_escape);

    let characters = json_character.many();
    character('"')
        .and_then(characters)
        .and_then(character('"'))
        .map(|((_, vs), _)| String::from_iter(vs))
}

pub fn number() -> impl Parser<f64> + Clone {
    let digits = digit()
        .many1()
        .map(|ds| ds.iter().fold(0_i32, |acc, &digit| acc * 10 + digit as i32));

    let sign = character('+').or_else(character('-'));
    let integer = digits.clone().or_else(
        character('-')
            .and_then(digits.clone())
            .map(|(_, number)| -number),
    );
    let fraction = character('.')
        .skip()
        .and_then(digits.clone())
        .map(|(_, num)| num);
    let exponent =
        character('E')
            .or_else(character('e'))
            .skip()
            .and_then(sign.and_then(digits.clone()).map(|(s, n)| match s {
                '-' => -n,
                _ => n,
            }));

    integer
        .and_then(fraction.optional())
        .and_then(exponent.optional())
        .map(|((n, f), e)| match (n, f, e) {
            (n, Some(f), Some((_, e))) => format!("{}.{}E{}", n, f, e)
                .parse::<f64>()
                .expect("couldn't parse number"),
            (n, Some(f), _) => format!("{}.{}", n, f)
                .parse::<f64>()
                .expect("couldn't parse number"),
            (n, _, _) => f64::from(n),
        })
}

fn ws() -> impl Parser<()> + Clone {
    character(' ')
        .or_else(character('\n'))
        .or_else(character('\r'))
        .or_else(character('\t'))
        .many()
        .skip()
}

#[cfg(test)]
mod tests {
    use crate::json::json_static_dispatch::*;
    use staticparse::Parser;

    #[test]
    fn test_parse_ws() {
        let s = " \n\r\thello";
        let (_, r) = ws().parse(s).unwrap();
        assert_eq!("hello", r);
    }

    #[test]
    fn test_parse_number() {
        let s = "1";
        let (v, _) = number().parse(s).unwrap();
        assert_eq!(1_f64, v);
        let s = "1.2";
        let (v, _) = number().parse(s).unwrap();
        assert_eq!(1.2_f64, v);
        let s = "1.234E-567";
        let (v, _) = number().parse(s).unwrap();
        assert_eq!(1.234E-567_f64, v);
    }

    #[test]
    fn test_parse_string() {
        let s = "\"hello\"1";
        let (v, r) = string().parse(s).unwrap();
        assert_eq!("hello", v.as_str());
        assert_eq!("1", r);
    }

    #[test]
    fn test_parse_bool() {
        let s = "true";
        let (v, _) = boolean().parse(s).unwrap();
        assert_eq!(true, v);
        let s = "false";
        let (v, _) = boolean().parse(s).unwrap();
        assert_eq!(false, v);
    }

    #[test]
    fn test_parse_array() {
        let s = "[1, [\"a\", false], null]";
        let (vs, _) = array().parse(s).unwrap();
        assert_eq!(Json::Number(1_f64), vs[0]);
        assert_eq!(
            Json::Array(vec![Json::String("a".to_string()), Json::Bool(false)]),
            vs[1]
        );
        assert_eq!(Json::Null, vs[2]);
    }

    #[test]
    fn test_parse_object() {
        let s = "{\"a\": true}";
        let (obj, _) = object().parse(s).unwrap();
        let expected = {
            let mut m = HashMap::new();
            m.insert("a".to_string(), Json::Bool(true));
            m
        };
        assert_eq!(expected, obj);
    }
}
