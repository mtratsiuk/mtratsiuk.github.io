use std::collections::HashMap;
use std::error::Error;
use std::iter::Peekable;
use std::slice::Iter;
use std::{result, vec};

const OBJECT_OPEN: u8 = b'{';
const OBJECT_CLOSE: u8 = b'}';
const ARRAY_OPEN: u8 = b'[';
const ARRAY_CLOSE: u8 = b']';
const ID_CLOSE: u8 = b':';
const ID_CLOSE_ESCAPE: &str = r"\:";

pub type Result<T> = result::Result<T, Box<dyn Error>>;

/// Parses RON (Rustache Object Notation) string into AST
pub fn parse(value: String) -> Result<Value> {
    let bytes = value.into_bytes();
    let mut lexer = Lexer::from(&bytes);
    let tokens = lexer.run()?;
    let mut parser = Parser::from(tokens);
    let result = parser.run()?;

    Ok(result)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    Text(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

#[derive(Debug)]
struct Parser<'a> {
    tokens: Peekable<Iter<'a, Token<'a>>>,
}

impl<'a> Parser<'a> {
    fn from(tokens: &'a Vec<Token>) -> Self {
        Self {
            tokens: tokens.iter().peekable(),
        }
    }

    fn run(&mut self) -> Result<Value> {
        let token = self
            .tokens
            .next()
            .ok_or("Unexpected end of tokens stream")?;

        return match token {
            Token::ObjectOpen => self.run_object(),
            Token::ArrayOpen => self.run_array(),
            Token::Text(value) => self.run_text(value),
            _ => Err(format!("Expected Object, Array or Text, got: {:?}", token))?,
        };
    }

    fn run_text(&mut self, value: &[u8]) -> Result<Value> {
        let string = String::from_utf8(value.to_vec())?
            .replace(ID_CLOSE_ESCAPE, &(ID_CLOSE as char).to_string());

        Ok(Value::Text(string))
    }

    fn run_object(&mut self) -> Result<Value> {
        let mut object = HashMap::new();

        loop {
            let token = self.tokens.next();

            match token {
                Some(value) => match value {
                    Token::Id(object_key) => {
                        let object_value = self.run()?;
                        object.insert(String::from_utf8(object_key.to_vec())?, object_value);
                    }
                    Token::ObjectClose => break,
                    _ => Err(format!("Expected Id, got: {:?}", value))?,
                },
                None => Err(format!("Expected {} closing Object", OBJECT_CLOSE))?,
            }
        }

        Ok(Value::Object(object))
    }

    fn run_array(&mut self) -> Result<Value> {
        let mut array = vec![];

        loop {
            let token = self.tokens.peek();

            match token {
                Some(value) => match value {
                    Token::ArrayClose => {
                        self.tokens.next();
                        break;
                    }
                    _ => {
                        array.push(self.run()?);
                    }
                },
                None => Err(format!("Expected {} closing Array", ARRAY_CLOSE))?,
            }
        }

        Ok(Value::Array(array))
    }
}

#[derive(Debug)]
enum Token<'a> {
    Id(&'a [u8]),
    Text(&'a [u8]),
    ObjectOpen,
    ObjectClose,
    ArrayOpen,
    ArrayClose,
}

#[derive(Debug)]
struct Lexer<'a> {
    pos: usize,
    bytes: &'a Vec<u8>,
    tokens: Vec<Token<'a>>,
}

impl<'a> Lexer<'a> {
    fn from(bytes: &'a Vec<u8>) -> Self {
        Self {
            pos: 0,
            bytes,
            tokens: vec![],
        }
    }

    fn run(&mut self) -> Result<&Vec<Token<'a>>> {
        while self.pos < self.bytes.len() {
            let current = self.bytes[self.pos];

            match current {
                OBJECT_OPEN => self.emit(Token::ObjectOpen, 1),
                OBJECT_CLOSE => self.emit(Token::ObjectClose, 1),
                ARRAY_OPEN => self.emit(Token::ArrayOpen, 1),
                ARRAY_CLOSE => self.emit(Token::ArrayClose, 1),
                byte if (!byte.is_ascii_whitespace()) => {
                    let start = self.pos;
                    let mut end;
                    let mut is_eof;
                    let mut is_id_close;
                    let mut is_text_close;

                    loop {
                        end = self.pos + 1;
                        is_eof = end >= self.bytes.len();
                        is_id_close = !is_eof
                            && self.bytes[end] == ID_CLOSE
                            && &self.bytes[end - 1..end + 1] != ID_CLOSE_ESCAPE.as_bytes();
                        is_text_close = !is_eof && self.bytes[end].is_ascii_control();

                        self.advance(1);

                        if is_eof || is_id_close || is_text_close {
                            break;
                        }
                    }

                    let value = &self.bytes[start..end];
                    let token = if is_id_close {
                        Token::Id(value)
                    } else {
                        Token::Text(value)
                    };

                    self.emit(token, 1)
                }
                _ => self.advance(1),
            }
        }

        Ok(&self.tokens)
    }

    fn advance(&mut self, n: usize) -> () {
        self.pos += n;
    }

    fn emit(&mut self, token: Token<'a>, advance_n: usize) -> () {
        self.tokens.push(token);
        self.advance(advance_n);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_text() {
        assert_eq!(
            parse("text".to_string()).unwrap(),
            Value::Text("text".to_string())
        );
    }

    #[test]
    fn parse_array() {
        assert_eq!(parse("[]".to_string()).unwrap(), Value::Array(vec![]));
    }

    #[test]
    fn parse_escaped_id_clise() {
        assert_eq!(
            parse(r"https\://test.com".to_string()).unwrap(),
            Value::Text("https://test.com".to_string())
        );
    }

    #[test]
    fn parse_object() {
        assert_eq!(
            parse("{}".to_string()).unwrap(),
            Value::Object(HashMap::new())
        );
    }

    #[test]
    fn parse_nested() {
        let string = "
{
    text: Name

    array: [
        ArrayText
    ]

    object: {
        field: ObjectText
    }
}
"
        .to_string();

        let expected = Value::Object(HashMap::from([
            ("text".to_string(), Value::Text("Name".to_string())),
            (
                "array".to_string(),
                Value::Array(vec![Value::Text("ArrayText".to_string())]),
            ),
            (
                "object".to_string(),
                Value::Object(HashMap::from([(
                    "field".to_string(),
                    Value::Text("ObjectText".to_string()),
                )])),
            ),
        ]));

        assert_eq!(parse(string).unwrap(), expected);
    }
}
