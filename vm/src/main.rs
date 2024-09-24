use vm::chunk::Chunk;
use vm::compiler::Compiler;
use vm::vm::Vm;
fn main() {
    /*let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let mut vm = Vm::new(&input).unwrap();
    vm.run().unwrap();*/


    let mut c = Chunk::new();
    println!(
        "{:?}",
        Compiler::compile("{var a =  7; {var b = 9;}}", &mut c)
    );
    println!("{}", c);
    println!("compilation done");
    let mut vm = Vm::new("{ var a = a }").unwrap();
    println!("{:?}", vm.run());
}
