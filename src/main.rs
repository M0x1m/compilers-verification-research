use std::fs;
use std::io::Read;
use std::collections::HashMap;

mod parser;
mod lexer;

fn verify(ast: parser::AST) {
    println!("{ast:?}");
    todo!()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ast = parser::parse_file("input.dsl")?;

    verify(ast);
    Ok(())
}
