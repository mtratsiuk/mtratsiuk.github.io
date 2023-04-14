use std::error::Error;
use std::fmt::Debug;
use std::result;

use crate::ron::Value;

pub type Result<T> = result::Result<T, Box<dyn Error>>;

pub fn parse(value: String) -> Result<impl Pipe> {
    let (name, params) = match value.split_once(' ') {
        None => (value, "".to_string()),
        Some((name, params)) => (name.to_string(), params.to_string())
    };

    match name.as_str() {
        "$reverse" => ReversePipe::from_string(params),
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn reverse_pipe_should_reverse_text() {
        let pipe = parse("$reverse".to_string()).unwrap();
        let value = Value::Text("123456".to_string());
        let expected = Value::Text("654321".to_string());

        assert_eq!(pipe.apply(&value).unwrap(), expected)
    }
}
