use std::error::Error;
use std::fmt::Debug;
use std::ops::{RangeBounds};
use std::result;

use crate::ron::Value;

pub type Result<T> = result::Result<T, Box<dyn Error>>;

pub fn parse(value: &str) -> Result<Box<dyn Pipe>> {
    let (name, params) = match value.split_once(' ') {
        None => (value.to_string(), "".to_string()),
        Some((name, params)) => (name.to_string(), params.to_string()),
    };

    match name.as_str() {
        "$reverse" => Ok(Box::new(ReversePipe::from_string(params)?)),
        "$sort" => Ok(Box::new(SortPipe::from_string(params)?)),
        _ => Err(format!("Unexpected pipe name: {:?}", name))?,
    }
}

pub trait Pipe {
    fn from_string(params: String) -> Result<Self>
    where
        Self: Sized;

    fn apply(&self, value: &Value) -> Result<Value>;
}

#[derive(Debug)]
pub struct ReversePipe;

impl Pipe for ReversePipe {
    fn from_string(_params: String) -> Result<Self> {
        Ok(ReversePipe {})
    }

    fn apply(&self, value: &Value) -> Result<Value> {
        match value {
            Value::Text(text) => Ok(Value::Text(text.chars().rev().collect())), // Note: broken for surrogate pairs
            Value::Array(array) => Ok(Value::Array(array.clone().into_iter().rev().collect())),
            _ => Err(format!("Can't apply ReversePipe to {:?}", value))?,
        }
    }
}

#[derive(Debug)]
pub struct SortPipe {
    op: Op,
    left: Vec<String>,
    right: Vec<String>,
}

impl Pipe for SortPipe {
    fn from_string(params: String) -> Result<Self> {
        let sort_expr_parser = create_expr_parser();

        match sort_expr_parser(&mut ParserState::from_string(&params)) {
            ParserResult::Some(Expr::Call(op, args)) => match &args[..] {
                [Expr::Id(left), Expr::Id(right)] => {
                    let (left, right) = (
                        left.clone()
                            .split('.')
                            .map(str::to_string)
                            .collect::<Vec<_>>(),
                        right.split('.').map(str::to_string).collect::<Vec<_>>(),
                    );

                    match (
                        left.first().unwrap().as_str(),
                        right.first().unwrap().as_str(),
                    ) {
                        ("$1", "$2") | ("$2", "$1") => (),
                        _ => Err(format!("Expected argument names to start with $1 or $2"))?,
                    }

                    Ok(Self { op, left, right })
                }
                _ => Err(format!(
                    "Unexpected number of $sort arguments: {:?}, expected 2",
                    args.len()
                ))?,
            },
            ParserResult::Err(err) => Err(format!("Can't parse $sort expression {:?}", err))?,
            ParserResult::None => Err(format!("Can't parse $sort expression"))?,
            _ => Err(format!("Unexpected $sort expression type"))?,
        }
    }

    fn apply(&self, value: &Value) -> Result<Value> {
        match value {
            Value::Array(array) => {
                let mut result = array.clone();

                result.sort_by(|l, r| {
                    let mut l_value = self.get_value(l, &self.left[1..]).unwrap();
                    let mut r_value = self.get_value(r, &self.right[1..]).unwrap();

                    if self.left.first().unwrap() == "$2" {
                        (l_value, r_value) = (r_value, l_value);
                    }

                    match self.op {
                        Op::Sub => u32::from_str_radix(&l_value, 10)
                            .unwrap()
                            .cmp(&u32::from_str_radix(&r_value, 10).unwrap()),
                        Op::StrCmp => l_value.cmp(&r_value),
                        _ => panic!("Unexpected operation"),
                    }
                });

                Ok(Value::Array(result))
            }
            _ => Err(format!(
                "Can't apply SortPipe to {:?} (expected array)",
                value
            ))?,
        }
    }
}

impl SortPipe {
    fn get_value(&self, value: &Value, path: &[String]) -> Result<String> {
        match (value, &path[..]) {
            (Value::Object(v), [key, rest @ ..]) => self.get_value(
                v.get(key)
                    .ok_or(format!("Property {} is undefined at {:?}", key, value))?,
                rest,
            ),
            (Value::Text(v), []) => Ok(v.to_string()),
            _ => Err(format!("Can't read {:?} at {:?}", path, value))?,
        }
    }
}

struct ParserState {
    in_bytes: Vec<u8>,
    pos: usize,
}

type PS = ParserState;

impl ParserState {
    fn from_string(value: &str) -> Self {
        ParserState {
            in_bytes: value.as_bytes().to_vec(),
            pos: 0,
        }
    }

    fn peek(&self) -> u8 {
        self.in_bytes[self.pos]
    }

    fn advance(&mut self) -> () {
        self.pos += 1
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.in_bytes.len()
    }
}

#[allow(dead_code)]
enum ParserResult<T> {
    Some(T),
    None,
    Err(Box<dyn Error>),
}

type PR<T> = ParserResult<T>;

#[derive(Debug, PartialEq, Eq)]
enum Op {
    Sub,
    StrCmp,
    Unknown,
}

#[derive(Debug, PartialEq, Eq)]
enum Expr {
    Id(String),
    Call(Op, Vec<Expr>),
}

fn parser_byte_ranges(
    ranges: Vec<impl RangeBounds<u8> + 'static>,
) -> Box<dyn Fn(&mut PS) -> PR<u8>> {
    Box::new(move |state| {
        if state.is_at_end() {
            return ParserResult::None;
        }

        let char = state.peek();

        if ranges.iter().any(|range| range.contains(&char)) {
            state.advance();
            ParserResult::Some(char)
        } else {
            ParserResult::None
        }
    })
}

fn parser_byte(ch: u8) -> Box<dyn Fn(&mut PS) -> PR<u8>> {
    parser_byte_ranges(vec![(ch..=ch)])
}

fn parser_seq<T, P>(parsers: Vec<P>) -> Box<dyn Fn(&mut PS) -> PR<Vec<T>>>
where
    P: Fn(&mut PS) -> PR<T> + 'static,
{
    Box::new(move |state| {
        let initial_pos = state.pos;
        let mut result: Vec<T> = vec![];

        for parser in &parsers {
            match parser(state) {
                ParserResult::Some(r) => {
                    result.push(r);
                }
                ParserResult::None => {
                    state.pos = initial_pos;
                    return ParserResult::None;
                }
                ParserResult::Err(err) => {
                    return ParserResult::Err(err);
                }
            }
        }

        ParserResult::Some(result)
    })
}

fn parser_or<T, P>(parsers: Vec<P>) -> Box<dyn Fn(&mut PS) -> PR<T>>
where
    P: Fn(&mut PS) -> PR<T> + 'static,
{
    Box::new(move |state| {
        let initial_pos = state.pos;

        for parser in &parsers {
            match parser(state) {
                ParserResult::Some(r) => {
                    return ParserResult::Some(r);
                }
                ParserResult::None => {
                    state.pos = initial_pos;
                }
                ParserResult::Err(err) => {
                    return ParserResult::Err(err);
                }
            }
        }

        ParserResult::None
    })
}

fn parser_many<T, P>(parser: P) -> Box<dyn Fn(&mut PS) -> PR<Vec<T>>>
where
    P: Fn(&mut PS) -> PR<T> + 'static,
{
    Box::new(move |state| {
        let mut results = vec![];

        loop {
            let initial_pos = state.pos;

            match parser(state) {
                ParserResult::Some(r) => {
                    results.push(r);
                }
                ParserResult::None => {
                    state.pos = initial_pos;
                    break;
                }
                ParserResult::Err(err) => {
                    return ParserResult::Err(err);
                }
            }
        }

        ParserResult::Some(results)
    })
}

fn parser_map<T, F, P, R>(parser: P, map: F) -> Box<dyn Fn(&mut PS) -> PR<R>>
where
    F: Fn(T) -> R + 'static,
    P: Fn(&mut PS) -> PR<T> + 'static,
{
    Box::new(move |state| match parser(state) {
        ParserResult::Some(r) => ParserResult::Some(map(r)),
        ParserResult::None => ParserResult::None,
        ParserResult::Err(err) => ParserResult::Err(err),
    })
}

macro_rules! try_parse(
    ($e:expr) => ({
        match $e {
            ParserResult::Some(x) => x,
            ParserResult::None => {
                return ParserResult::None;
            },
            ParserResult::Err(err) => {
                return ParserResult::Err(err);
            },
        }
    });
);

fn create_expr_parser() -> Box<dyn Fn(&mut PS) -> PR<Expr>> {
    let call_open = parser_byte(b'(');
    let call_close = parser_byte(b')');
    let space = parser_byte(b' ');
    let op_sub = parser_seq("$sub".as_bytes().iter().map(|x| parser_byte(*x)).collect());
    let op_str_cmp = parser_seq(
        "$str_cmp"
            .as_bytes()
            .iter()
            .map(|x| parser_byte(*x))
            .collect(),
    );
    let id = parser_many(parser_or(vec![
        parser_byte_ranges(vec![(b'0'..=b'9'), (b'a'..=b'z'), (b'A'..=b'Z')]),
        parser_byte(b'$'),
        parser_byte(b'_'),
        parser_byte(b'.'),
    ]));
    let op = parser_map(
        parser_or(vec![op_sub, op_str_cmp]),
        |x| match &String::from_utf8(x).unwrap()[..] {
            "$sub" => Op::Sub,
            "$str_cmp" => Op::StrCmp,
            _ => Op::Unknown,
        },
    );

    let expression = move |state: &mut ParserState| -> ParserResult<Expr> {
        try_parse!(call_open(state));
        let operation = try_parse!(op(state));
        try_parse!(space(state));
        let left = try_parse!(id(state));
        try_parse!(space(state));
        let right = try_parse!(id(state));
        try_parse!(call_close(state));

        ParserResult::Some(Expr::Call(
            operation,
            vec![
                Expr::Id(String::from_utf8(left).unwrap()),
                Expr::Id(String::from_utf8(right).unwrap()),
            ],
        ))
    };

    Box::new(expression)
}

#[cfg(test)]
mod tests {
    use crate::ron;
    use super::*;

    #[test]
    fn reverse_pipe_should_reverse_text() {
        let pipe = parse("$reverse").unwrap();
        let value = Value::Text("123456".to_string());
        let expected = Value::Text("654321".to_string());

        assert_eq!(pipe.apply(&value).unwrap(), expected)
    }

    #[test]
    fn test_parser_byte() {
        let mut state = ParserState::from_string("123abc!$#");
        let parser = parser_byte_ranges(vec![(b'0'..=b'9'), (b'a'..=b'z')]);

        let result = parser(&mut state);

        if let ParserResult::Some(v) = result {
            assert_eq!(v, b'1');
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parser_seq() {
        let mut state = ParserState::from_string("1234abc!$#");
        let parser = parser_seq(vec![
            parser_byte(b'1'),
            parser_byte(b'2'),
            parser_byte(b'3'),
            parser_or(vec![parser_byte(b'4')]),
        ]);

        let result = parser(&mut state);

        if let ParserResult::Some(v) = result {
            assert_eq!(v, vec![b'1', b'2', b'3', b'4']);
            assert_eq!(state.pos, 4);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parser_seq_failed() {
        let mut state = ParserState::from_string("123abc!$#");
        let parser = parser_seq(vec![
            parser_byte(b'3'),
            parser_byte(b'2'),
            parser_byte(b'1'),
        ]);

        let result = parser(&mut state);

        if let ParserResult::None = result {
            assert_eq!(state.pos, 0);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parser_map() {
        let mut state = ParserState::from_string("123abc!$#");
        let parser = parser_map(
            parser_seq(vec![
                parser_byte(b'1'),
                parser_byte(b'2'),
                parser_byte(b'3'),
            ]),
            |x| String::from_utf8(x).unwrap(),
        );

        let result = parser(&mut state);

        if let ParserResult::Some(v) = result {
            assert_eq!(v, "123".to_string());
            assert_eq!(state.pos, 3);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_parser_or() {
        let mut state = ParserState::from_string("123abc!$#");
        let parser = parser_or(vec![
            parser_byte(b'a'),
            parser_byte(b'3'),
            parser_byte(b'2'),
            parser_byte(b'1'),
        ]);

        let result = parser(&mut state);

        if let ParserResult::Some(v) = result {
            assert_eq!(v, b'1');
            assert_eq!(state.pos, 1);
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_expr() {
        let mut state = ParserState::from_string("($sub $2.count $1.count)");
        let parser = create_expr_parser();

        let result = parser(&mut state);

        if let ParserResult::Some(v) = result {
            assert_eq!(
                v,
                Expr::Call(
                    Op::Sub,
                    vec![
                        Expr::Id("$2.count".to_string()),
                        Expr::Id("$1.count".to_string())
                    ]
                )
            )
        } else {
            assert!(false);
        }
    }

    #[test]
    fn test_sort_pipe_sub() {
        let pipe = parse("$sort ($sub $2.count.value $1.count.value)").unwrap();

        let value = ron::parse(
            "\
[
    {
        count: {
            value: 10
        }
    }
    {
         count: {
            value: 5
        }
    }
    {
         count: {
            value: 55
        }
    }
    {
         count: {
            value: 18
        }
    }
]"
            .to_string(),
        )
        .unwrap();

        let expected = ron::parse(
            "\
[
    {
        count: {
            value: 55
        }
    }
    {
        count: {
            value: 18
        }
    }
    {
        count: {
            value: 10
        }
    }
    {
        count: {
            value: 5
        }
    }
]"
            .to_string(),
        )
        .unwrap();

        assert_eq!(pipe.apply(&value).unwrap(), expected)
    }

    #[test]
    fn test_sort_pipe_str_cmp() {
        let pipe = parse("$sort ($str_cmp $2.count.value $1.count.value)").unwrap();

        let value = ron::parse(
            "\
[
    {
        count: {
            value: c#
        }
    }
    {
         count: {
            value: g#
        }
    }
    {
         count: {
            value: a#
        }
    }
    {
         count: {
            value: b
        }
    }
]"
            .to_string(),
        )
        .unwrap();

        let expected = ron::parse(
            "\
[
    {
        count: {
            value: g#
        }
    }
    {
        count: {
            value: c#
        }
    }
    {
        count: {
            value: b
        }
    }
    {
        count: {
            value: a#
        }
    }
]"
            .to_string(),
        )
        .unwrap();

        assert_eq!(pipe.apply(&value).unwrap(), expected)
    }
}
