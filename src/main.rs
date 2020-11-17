use std::{error::Error, io::{self, Read}};

mod parser;
use parser::parse_sexpr;

fn main() -> Result<(), Box<dyn Error>> {
    let mut program = String::new();
    io::stdin().read_to_string(&mut program)?;

    let ast = parse_sexpr(&program)?;

    println!("AST: {:#?}", ast);
    println!("AST prettyprinted: {}", ast);

    Ok(())
}
