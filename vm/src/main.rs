use std::io::Write;
use std::{env, fs, io, process};
use vm::vm::Vm;

fn main() {
    let mut args = env::args();
    args.next();

    let mut vm = Vm::new();
    if let Err(()) = match (args.next(), args.next()) {
        (None, None) => run_prompt(&mut vm),
        (Some(filename), None) => run_file(&mut vm, &filename),
        _ => Err(()),
    } {
        process::exit(1);
    };
}

fn run_prompt(vm: &mut Vm) -> ! {
    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let _ = vm.run(&input);
    }
}

fn run_file(vm: &mut Vm, filename: &str) -> Result<(), ()> {
    match fs::read_to_string(filename) {
        Ok(source) => vm.run(&source),
        Err(e) => {
            eprintln!("{}", e.to_string().to_lowercase());
            Err(())
        }
    }
}
