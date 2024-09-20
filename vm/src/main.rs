use vm::chunk::Chunk;
use vm::compiler::Compiler;
use vm::vm::Vm;
fn main() {
    /*let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let mut vm = Vm::new(&input).unwrap();
    vm.run().unwrap();*/

    // let mut c = Chunk::new();
    // println!("{:?}", Compiler::compile("\"ciao\" + 5;", &mut c));
    // println!("{}", c);
    let mut vm = Vm::new("var a = 5; var b = 6; a + b = 7;").unwrap();
    println!("{:?}", vm.run());
}
