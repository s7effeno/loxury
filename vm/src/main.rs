use vm::chunk::Chunk;
use vm::compiler::Compiler;
use vm::vm::Vm;
fn main() {
    /*let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let mut vm = Vm::new(&input).unwrap();
    vm.run().unwrap();*/

    let mut vm = Vm::new();
    let a = "for (var i = 0; i < 10; i = i + 1) for (var j = 0; j < i; j = j + 1) print j;";
    vm.run(a);
}
