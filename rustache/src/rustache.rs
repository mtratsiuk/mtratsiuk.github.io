use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::iter::Peekable;
use std::path::Path;
use std::str::Chars;

const TEMPLATE_NAME: &str = "index.html";
const VARIABLE_OPEN: (u8, u8) = (b'{', b'{');
const VARIABLE_CLOSE: (u8, u8) = (b'}', b'}');

pub fn render(input: &Path, output: &Path) -> Result<(), Box<dyn Error>> {
    let mut parser = Parser::from(input);
    parser.run()?;
    let result = parser.result()?;

    fs::write(output, result)?;

    Ok(())
}

struct Parser {
    in_bytes: Vec<u8>,
    out_bytes: Vec<u8>,
    pos: usize,
    scopes: Vec<HashMap<String, String>>,
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
            scopes: vec![HashMap::from([("name".to_string(), "Misha".to_string())])],
        }
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        while self.pos < self.in_bytes.len() {
            self.html()?;
        }

        Ok(())
    }

    fn result(self) -> Result<String, Box<dyn Error>> {
        Ok(String::from_utf8(self.out_bytes)?)
    }

    fn html(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            match self.peek_pair() {
                Some(pair) => match pair {
                    VARIABLE_OPEN => {
                        self.skip(2);
                        return self.variable();
                    }
                    _ => self.consume(2),
                },
                None => {
                    self.consume(1);
                    break;
                }
            }
        }

        Ok(())
    }

    fn variable(&mut self) -> Result<(), Box<dyn Error>> {
        let name = self.skip_until_pair(VARIABLE_CLOSE)?;
        let value = self.get_value(&name).ok_or(format!("Undefined variable {}", name))?;

        self.emit(&mut value.into_bytes());
        self.skip(2);

        Ok(())
    }

    fn peek_pair(&self) -> Option<(u8, u8)> {
        return if self.pos + 2 > self.in_bytes.len() {
            None
        } else {
            Some((self.in_bytes[self.pos], self.in_bytes[self.pos + 1]))
        };
    }

    fn skip(&mut self, n: usize) -> () {
        self.pos += n;
    }

    fn skip_until_pair(&mut self, pair: (u8, u8)) -> Result<String, Box<dyn Error>> {
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

    fn get_value(&mut self, key: &str) -> Option<String> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(key) {
                return Some(value.clone())
            }
        }

        None
    }
}

// enum TemplateNode {
//     Text(&str),
//     Variable(&str),
//     Loop(&str, &str, Vec<TemplateNode>),
// }

// struct Parser2 {
//   in_bytes: Vec<u8>,
//   pos: usize,
//   current_parser: fn(&mut Parser) -> Result<(), Box<dyn Error>>,
// }
