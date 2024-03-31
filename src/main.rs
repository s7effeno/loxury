use loxury::{Interpreter, Lexer, Parser, Resolver};
use std::env;
use std::{
    fs,
    io::{self, Write},
};

fn main() -> Result<(), ()> {
    let mut args = env::args();
    args.next();
    match (args.next(), args.next()) {
        (None, None) => run_prompt(),
        (Some(filename), None) => run_file(&filename),
        _ => {
            eprintln!("usage: loxury [script]");
            Err(())
        }
    }
}

fn run_prompt() -> ! {
    let mut interpreter = Interpreter::new();
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let _ = run(&input, &mut interpreter);
    }
}

fn run_file(filename: &str) -> Result<(), ()> {
    match fs::read_to_string(filename) {
        Ok(source) => run(&source, &mut Interpreter::new()),
        Err(e) => {
            eprintln!("{}", e);
            Err(())
        }
    }
}

fn run(source: &str, interpreter: &mut Interpreter) -> Result<(), ()> {
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let statements = match parser.parse() {
        Ok(statements) => statements,
        Err(()) => {
            for e in parser.errors() {
                eprintln!("{e}");
            }
            return Err(());
        }
    };
    let mut resolver = Resolver::new(interpreter);
    resolver.resolve(&statements);
    if let Err(e) = interpreter.interpret(statements) {
        eprintln!("{e}");
        return Err(());
    }
    Ok(())
}
