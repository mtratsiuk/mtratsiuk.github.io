use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::{fs, result};

use crate::pipe::{self, Pipe};
use crate::ron;
use crate::ron::Value as RonValue;

pub type Result<T> = result::Result<T, Box<dyn Error>>;

type TemplatePair = (u8, u8);

const TEMPLATE_NAME: &str = "index.rustache";
const VARIABLES_NAME: &str = "index.ron";
const CSS_NAME: &str = "index.css";
const JS_NAME: &str = "index.js";
const LOOP_ITEM_VARIABLE: &str = "$it";
const VARIABLE_OPEN: TemplatePair = (b'{', b'{');
const VARIABLE_CLOSE: TemplatePair = (b'}', b'}');
const LOOP_OPEN: TemplatePair = (b'{', b'*');
const LOOP_CLOSE: TemplatePair = (b'*', b'}');
const OPTIONAL_OPEN: TemplatePair = (b'{', b'?');
const OPTIONAL_CLOSE: TemplatePair = (b'?', b'}');
const INLINE_OPEN: TemplatePair = (b'{', b'>');
const INLINE_CLOSE: TemplatePair = (b'<', b'}');
const BLOCK_END: TemplatePair = (b'{', b'}');
const VARIABLE_PATH_SEPARATOR: char = '.';
const PIPE_SEPARATOR: char = '|';

const BLOCK_OPENING_PAIRS: [TemplatePair; 2] = [LOOP_OPEN, OPTIONAL_OPEN];

pub fn render(input: &Path, output: &Path) -> Result<()> {
    let mut parser = Parser::from(input)?;
    parser.run()?;
    let result = parser.result()?;

    fs::write(output, result)?;

    Ok(())
}

#[derive(Debug)]
struct Parser<'a> {
    input: &'a Path,
    in_bytes: Vec<u8>,
    out_bytes: Vec<u8>,
    pos: usize,
    scopes: Vec<RonValue>,
}

impl<'a> Parser<'a> {
    fn from(input: &'a Path) -> Result<Self> {
        let template_path = input.join(TEMPLATE_NAME);
        let template = fs::read_to_string(&template_path)?;
        let variables_path = input.join(VARIABLES_NAME);
        let variables_string = fs::read_to_string(&variables_path)?;

        let variables = ron::parse(variables_string)?;

        let in_bytes = template.into_bytes();
        let out_bytes = Vec::with_capacity(in_bytes.len());

        Ok(Self {
            input,
            in_bytes,
            out_bytes,
            pos: 0,
            scopes: vec![variables],
        })
    }

    fn __broken_from_string(template_str: String, variables_str: String) -> Result<Self> {
        let input = Path::new("fake.txt");
        let in_bytes = template_str.into_bytes();
        let out_bytes = Vec::with_capacity(in_bytes.len());
        let variables = ron::parse(variables_str)?;

        Ok(Self {
            input,
            in_bytes,
            out_bytes,
            pos: 0,
            scopes: vec![variables],
        })
    }

    fn run(&mut self) -> Result<()> {
        while self.pos < self.in_bytes.len() {
            self.run_html()?;
        }

        Ok(())
    }

    fn result(self) -> Result<String> {
        Ok(String::from_utf8(self.out_bytes)?)
    }

    fn run_html(&mut self) -> Result<()> {
        loop {
            match self.peek_pair() {
                Some(pair) => match pair {
                    VARIABLE_OPEN => {
                        self.skip(2);
                        self.run_variable()?;
                    }
                    LOOP_OPEN => {
                        self.skip(2);
                        self.run_loop()?;
                    }
                    OPTIONAL_OPEN => {
                        self.skip(2);
                        self.run_optional()?;
                    }
                    INLINE_OPEN => {
                        self.skip(2);
                        self.run_inline()?;
                    }
                    BLOCK_END => {
                        if self.scopes.len() > 1 {
                            // if we are inside the block scope,
                            // stop and give control back to previous parser
                            // it will take care of the closing characters
                            break;
                        } else {
                            // skip otherwise
                            self.consume(2);
                        }
                    }
                    _ => self.consume(1),
                },
                None => {
                    // Only single char left, consume and stop
                    //
                    // Bound check handles the case when template ends with
                    // block closing chars (e.g. `{}`) - then they would be already
                    // consumed by last parser
                    if self.in_bytes.len() > self.pos {
                        self.consume(1);
                    }

                    break;
                }
            }
        }

        Ok(())
    }

    fn run_variable(&mut self) -> Result<()> {
        let variable_string = self.skip_until_pair(VARIABLE_CLOSE)?;

        self.skip(2);

        let (name, apply_pipe) = self.get_name_and_pipe(&variable_string)?;
        let variable = self.get_value(&name)?;

        let value = match variable {
            value @ RonValue::Text(_) => match apply_pipe(value)? {
                RonValue::Text(x) => x.clone(),
                _ => return Err("Expected pipe to return text")?,
            },
            _ => return Err(format!("Expected {} to be variable", name))?,
        };

        self.emit(&mut value.into_bytes());

        Ok(())
    }

    fn run_inline(&mut self) -> Result<()> {
        let name = self.skip_until_pair(INLINE_CLOSE)?;
        self.skip(2);

        match name.as_str() {
            "css" => {
                let css_path = self.input.join(CSS_NAME);
                let css_string = fs::read_to_string(&css_path)?;

                self.emit(&mut "<style>\n".to_string().into_bytes());
                self.emit(&mut css_string.into_bytes());
                self.emit(&mut "</style>".to_string().into_bytes());
            }
            "js" => {
                let js_path = self.input.join(JS_NAME);
                let js_string = fs::read_to_string(&js_path)?;

                self.emit(&mut "<script>\n".to_string().into_bytes());
                self.emit(&mut js_string.into_bytes());
                self.emit(&mut "</script>".to_string().into_bytes());
            }
            _ => return Err(format!("Unexpected inline asset: {}", name))?,
        }

        Ok(())
    }

    fn run_loop(&mut self) -> Result<()> {
        let variable_string = self.skip_until_pair(LOOP_CLOSE)?;
        self.skip(2);

        let (name, apply_pipe) = self.get_name_and_pipe(&variable_string)?;
        let variable = self.get_value(&name)?;

        let items = match variable {
            value @ RonValue::Array(_) => match apply_pipe(value)? {
                RonValue::Array(x) => x.clone(),
                _ => return Err("Expected pipe to return array")?,
            },
            _ => return Err(format!("Expected {} to be array", name))?,
        };

        let return_pos = self.pos;

        for item in items {
            self.pos = return_pos;

            let scope = RonValue::Object(HashMap::from([(LOOP_ITEM_VARIABLE.to_string(), item)]));
            self.scopes.push(scope);

            self.run_html()?;

            self.scopes.pop();
        }

        self.skip(2);

        Ok(())
    }

    fn run_optional(&mut self) -> Result<()> {
        let name = self.skip_until_pair(OPTIONAL_CLOSE)?;
        self.skip(2);

        let variable = self.get_value(&name);
        let mut inner_blocks = 0;

        match variable {
            Ok(_) => {
                self.run_html()?;
                self.skip(2);
            }
            Err(_) => loop {
                match self.peek_pair() {
                    Some(pair) => match pair {
                        pair if BLOCK_OPENING_PAIRS.contains(&pair) => {
                            self.skip(2);
                            inner_blocks += 1;
                        }
                        BLOCK_END => {
                            self.skip(2);
                            if inner_blocks > 0 {
                                inner_blocks -= 1;
                            } else {
                                break;
                            }
                        }
                        _ => self.skip(1),
                    },
                    None => {
                        return Err(format!("Expected {:?} closing Optional block", BLOCK_END))?
                    }
                }
            },
        }

        Ok(())
    }

    fn peek_pair(&self) -> Option<TemplatePair> {
        return if self.pos + 2 > self.in_bytes.len() {
            None
        } else {
            Some((self.in_bytes[self.pos], self.in_bytes[self.pos + 1]))
        };
    }

    fn skip(&mut self, n: usize) -> () {
        self.pos += n;
    }

    fn skip_until_pair(&mut self, pair: TemplatePair) -> Result<String> {
        let mut name = vec![];

        while self
            .peek_pair()
            .expect(&format!("Expected closing {:?}", pair))
            != pair
        {
            name.push(self.in_bytes[self.pos]);
            self.skip(1);
        }

        let name = String::from_utf8(name)?;

        Ok(name.trim().to_string())
    }

    fn consume(&mut self, n: usize) -> () {
        for _ in 0..n {
            self.out_bytes.push(self.in_bytes[self.pos]);
            self.pos += 1;
        }
    }

    fn emit(&mut self, bytes: &mut Vec<u8>) -> () {
        self.out_bytes.append(bytes);
    }

    fn get_value(&mut self, key: &str) -> Result<&RonValue> {
        let mut path = key.split(VARIABLE_PATH_SEPARATOR).into_iter();

        for scope in self.scopes.iter().rev() {
            let variables = match scope {
                RonValue::Object(x) => x,
                _ => {
                    return Err(format!(
                        "Expected root scope to be Object, got: {:?}",
                        scope
                    ))?
                }
            };

            let root_key = path
                .next()
                .ok_or(format!("Unexpected variable name {}", key))?;
            let root_value = variables.get(root_key);

            if let Some(mut value) = root_value {
                for next_key in path {
                    match value {
                        RonValue::Object(object) => {
                            value = object.get(next_key).ok_or(format!(
                                "Property {} is undefined at {:?}",
                                next_key, value
                            ))?;
                        }
                        _ => Err(format!("Cannot read property {} of {:?}", next_key, value))?,
                    }
                }

                return Ok(value);
            }
        }

        Err(format!("Variable {} is undefined", key))?
    }

    fn get_name_and_pipe(
        &self,
        var_str: &str,
    ) -> Result<(String, impl FnOnce(&RonValue) -> Result<RonValue>)> {
        let (name, pipe) = match var_str.split_once(PIPE_SEPARATOR) {
            None => (var_str.to_string(), None),
            Some((name, pipe_str)) => {
                (name.trim().to_string(), Some(pipe::parse(pipe_str.trim())?))
            }
        };

        Ok((name, |val: &RonValue| match pipe {
            None => Ok(val.clone()),
            Some(pipe) => pipe.apply(val),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_should_handle_template_variable() {
        let template = "\
<div>{{ name }}</div>\
"
        .to_string();

        let variables = "
{
    name: Test name
}
"
        .to_string();

        let mut parser = Parser::__broken_from_string(template, variables).unwrap();
        parser.run().unwrap();
        let result = parser.result().unwrap();

        assert_eq!(result, "<div>Test name</div>");
    }

    #[test]
    fn parser_should_handle_template_variable_with_reverse_pipe() {
        let template = "\
<div>{{ name | $reverse }}</div>\
"
        .to_string();

        let variables = "
{
    name: 12345
}
"
        .to_string();

        let mut parser = Parser::__broken_from_string(template, variables).unwrap();
        parser.run().unwrap();
        let result = parser.result().unwrap();

        assert_eq!(result, "<div>54321</div>");
    }

    #[test]
    fn parser_should_handle_template_loop_with_reverse_pipe() {
        let template = "\
{* items | $reverse *}<div>{{ $it.name }}</div>{}\
"
        .to_string();

        let variables = "
{
    items: [
        {
            name: One
        }
        {
            name: Two
        }
        {
            name: Three
        }
    ]
}
"
        .to_string();

        let mut parser = Parser::__broken_from_string(template, variables).unwrap();
        parser.run().unwrap();
        let result = parser.result().unwrap();

        assert_eq!(result, "<div>Three</div><div>Two</div><div>One</div>");
    }
}
