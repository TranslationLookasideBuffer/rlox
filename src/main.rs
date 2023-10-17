mod vm;

fn main() {
    let mut chunk = vm::Chunk::new();
    vm::Constant::new(1.2).write(&mut chunk, 100);
    vm::Constant::new(3.4).write(&mut chunk, 100);
    vm::Add::new().write(&mut chunk, 100);
    vm::Constant::new(5.6).write(&mut chunk, 100);
    vm::Divide::new().write(&mut chunk, 100);
    vm::Negate::new().write(&mut chunk, 100);
    vm::Return::new().write(&mut chunk, 101);
    let mut vm = vm::VM::new(chunk);
    vm.interpret();
}
