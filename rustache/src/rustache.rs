use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

type TemplatePair = (u8, u8);
type TemplateError = Box<dyn Error>;

const TEMPLATE_NAME: &str = "index.html";
const LOOP_ITEM_VARIABLE: &str = "$it";
const VARIABLE_OPEN: TemplatePair = (b'{', b'{');
const VARIABLE_CLOSE: TemplatePair = (b'}', b'}');
const LOOP_OPEN: TemplatePair = (b'{', b'*');
const LOOP_CLOSE: TemplatePair = (b'*', b'}');
const BLOCK_END: TemplatePair = (b'{', b'}');

pub fn render(input: &Path, output: &Path) -> Result<(), TemplateError> {
    let mut parser = Parser::from(input);
    parser.run()?;
    let result = parser.result()?;

    fs::write(output, result)?;

    Ok(())
}

#[derive(Debug)]
enum Variable {
    Value(String),
    Array(Vec<String>),
}

#[derive(Debug)]
struct Parser {
    in_bytes: Vec<u8>,
    out_bytes: Vec<u8>,
    pos: usize,
    scopes: Vec<HashMap<String, Variable>>,
}

impl Parser {
    fn from(input: &Path) -> Self {
        let template_path = input.join(TEMPLATE_NAME);
        let template = fs::read_to_string(&template_path).expect(&format!(
            "Expected template file at {}",
            template_path.display()
        ));
        let in_bytes = template.into_bytes();
        let out_bytes = Vec::with_capacity(in_bytes.len());

        Self {
            in_bytes,
            out_bytes,
            pos: 0,
            scopes: vec![HashMap::from([
                ("name".to_string(), Variable::Value("Misha".to_string())),
                (
                    "items".to_string(),
                    Variable::Array(vec!["first!".to_string(), "second!".to_string()]),
                ),
            ])],
        }
    }

    fn run(&mut self) -> Result<(), TemplateError> {
        while self.pos < self.in_bytes.len() {
            self.run_html()?;
        }

        Ok(())
    }

    fn result(self) -> Result<String, TemplateError> {
        Ok(String::from_utf8(self.out_bytes)?)
    }

    fn run_html(&mut self) -> Result<(), TemplateError> {
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

    fn run_variable(&mut self) -> Result<(), TemplateError> {
        let name = self.skip_until_pair(VARIABLE_CLOSE)?;
        self.skip(2);

        let variable = self
            .get_value(&name)
            .ok_or(format!("Undefined variable {}", name))?;

        let value = match variable {
            Variable::Value(x) => x.clone(),
            _ => return Err(format!("Expected {} to be variable", name))?,
        };

        self.emit(&mut value.into_bytes());

        Ok(())
    }

    fn run_loop(&mut self) -> Result<(), TemplateError> {
        let name = self.skip_until_pair(LOOP_CLOSE)?;
        self.skip(2);

        let variable = self
            .get_value(&name)
            .ok_or(format!("Undefined variable {}", name))?;

        let items = match variable {
            Variable::Array(x) => x.clone(),
            _ => return Err(format!("Expected {} to be array", name))?,
        };

        let return_pos = self.pos;

        for item in items {
            self.pos = return_pos;

            let scope = HashMap::from([(LOOP_ITEM_VARIABLE.to_string(), Variable::Value(item))]);
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

    fn skip_until_pair(&mut self, pair: TemplatePair) -> Result<String, TemplateError> {
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

    fn get_value(&mut self, key: &str) -> Option<&Variable> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(key) {
                return Some(value);
            }
        }

        None
    }
}
