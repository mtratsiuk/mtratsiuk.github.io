use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::{fs, result};

use crate::ron;
use crate::ron::Value as RonValue;

pub type Result<T> = result::Result<T, Box<dyn Error>>;

type TemplatePair = (u8, u8);

const TEMPLATE_NAME: &str = "index.rustache.html";
const VARIABLES_NAME: &str = "index.ron";
const LOOP_ITEM_VARIABLE: &str = "$it";
const VARIABLE_OPEN: TemplatePair = (b'{', b'{');
const VARIABLE_CLOSE: TemplatePair = (b'}', b'}');
const LOOP_OPEN: TemplatePair = (b'{', b'*');
const LOOP_CLOSE: TemplatePair = (b'*', b'}');
const BLOCK_END: TemplatePair = (b'{', b'}');

pub fn render(input: &Path, output: &Path) -> Result<()> {
    let mut parser = Parser::from(input);
    parser.run()?;
    let result = parser.result()?;

    fs::write(output, result)?;

    Ok(())
}

#[derive(Debug)]
struct Parser {
    in_bytes: Vec<u8>,
    out_bytes: Vec<u8>,
    pos: usize,
    scopes: Vec<RonValue>,
}

impl Parser {
    fn from(input: &Path) -> Self {
        let template_path = input.join(TEMPLATE_NAME);
        let template = fs::read_to_string(&template_path).expect(&format!(
            "Expected template file at {}",
            template_path.display()
        ));

        let variables_path = input.join(VARIABLES_NAME);
        let variables_string = fs::read_to_string(&variables_path).expect(&format!(
            "Expected variables file at {}",
            variables_path.display()
        ));

        let variables = ron::parse(variables_string).expect("Failed to parse variables file");

        let in_bytes = template.into_bytes();
        let out_bytes = Vec::with_capacity(in_bytes.len());

        Self {
            in_bytes,
            out_bytes,
            pos: 0,
            scopes: vec![variables],
        }
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
                    self.consume(1);
                    break;
                }
            }
        }

        Ok(())
    }

    fn run_variable(&mut self) -> Result<()> {
        let name = self.skip_until_pair(VARIABLE_CLOSE)?;
        self.skip(2);

        let variable = self.get_value(&name)?;

        let value = match variable {
            RonValue::Text(x) => x.clone(),
            _ => return Err(format!("Expected {} to be variable", name))?,
        };

        self.emit(&mut value.into_bytes());

        Ok(())
    }

    fn run_loop(&mut self) -> Result<()> {
        let name = self.skip_until_pair(LOOP_CLOSE)?;
        self.skip(2);

        let variable = self.get_value(&name)?;

        let items = match variable {
            RonValue::Array(x) => x.clone(),
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

        let mut name = String::from_utf8(name)?;
        name.retain(|c| !c.is_whitespace());

        Ok(name)
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
        let mut path = key.split('.').into_iter();

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
}
