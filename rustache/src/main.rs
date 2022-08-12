mod ron;
mod rustache;

use std::{env, path::Path};

fn main() {
    let args: Vec<String> = env::args().collect();

    let input = read_path_arg(
        &args,
        "in",
        "Expected input path argument e.g.: `--in=./src`",
    );
    let output = read_path_arg(
        &args,
        "out",
        "Expected output path argument e.g.: `--out=./build/index.html`",
    );

    match rustache::render(input, output) {
        Ok(_) => println!("Done!"),
        Err(error) => println!("Failed to rustache: {error}"),
    }
}

fn read_path_arg<'a>(args: &'a Vec<String>, name: &'a str, err_msg: &'a str) -> &'a Path {
    args.iter()
        .find(|a| a.starts_with(&format!("--{name}")))
        .and_then(|a| a.split('=').last())
        .map(Path::new)
        .expect(err_msg)
}
