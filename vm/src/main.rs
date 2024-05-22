use std::io;
use vm::vm::Vm;
fn main() {
    /*let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let mut vm = Vm::new(&input).unwrap();
    vm.run().unwrap();*/

    // let mut vm = Vm::new("\"AAAA\" + \"CCCC\"").unwrap();
    let mut vm = Vm::new("1 + 5").unwrap();
    vm.run().unwrap();
}
