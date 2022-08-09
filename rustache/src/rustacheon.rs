use std::collections::HashMap;
use std::error::Error;
use std::result;

const OBJECT_OPEN: u8 = b'{';
const OBJECT_CLOSE: u8 = b'}';
const ARRAY_OPEN: u8 = b'[';
const ARRAY_CLOSE: u8 = b']';
const ID_CLOSE: u8 = b':';

pub type Result<T> = result::Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub enum Value {
    Text(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

pub fn parse(value: String) -> Result<Value> {
    let bytes = value.into_bytes();
    let mut lexer = Lexer::from(&bytes);
    let tokens = dbg!(lexer.run()?);

    Ok(Value::Text("".to_string()))
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
                        is_id_close = self.bytes[end] == ID_CLOSE;
                        is_text_close = self.bytes[end].is_ascii_control();

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

                    self.emit(token, 0)
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
