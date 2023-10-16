mod vm;

fn main() {
    let mut chunk = vm::Chunk::new();
    vm::Return::new(100).write(&mut chunk);
    vm::Constant::new(1.2, 101).write(&mut chunk);
    chunk.disassemble("test");
}
