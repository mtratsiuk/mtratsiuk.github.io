mod rustache;

use std::path::Path;

fn main() {
    let input = Path::new("./src");
    let output = Path::new("./index.html");

    match rustache::render(input, output) {
      Ok(_) => println!("Done!"),
      Err(error) => println!("Failed to rustache: {}", error)
    }
}
