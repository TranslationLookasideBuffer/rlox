use std::fmt::Write;

pub struct VM {
    chunk: Chunk,
    stack: Stack,
}

impl VM {
    pub fn new(chunk: Chunk) -> VM {
        return VM {
            chunk: chunk,
            stack: Stack::new(),
        };
    }

    pub fn interpret(&mut self) -> InterpretResult {
        return self.chunk.interpret(&mut self.stack);
    }

    pub fn close(&self) {}
}

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

struct Stack {
    values: [Value; 256],
    cursor: usize,
}

impl Stack {
    fn new() -> Stack {
        return Stack {
            values: [0.0; 256],
            cursor: 0,
        };
    }

    fn push(&mut self, value: Value) {
        self.values[self.cursor] = value;
        self.cursor += 1;
    }

    fn pop(&mut self) -> Value {
        self.cursor -= 1;
        return self.values[self.cursor];
    }

    fn debug(&self) -> String {
        let mut debug = String::new();
        for idx in 0..self.cursor {
            write!(&mut debug, "{} ", self.values[idx]).unwrap();
        }
        if debug.len() > 0 {
            debug.pop();
        }
        format!("[{}]", debug)
    }
}

pub struct Chunk {
    instructions: Vec<u8>,
    lines: Vec<Line>,
    pool: Pool,
}

impl Chunk {
    pub fn new() -> Chunk {
        return Chunk {
            instructions: Vec::new(),
            lines: Vec::new(),
            pool: Pool::new(),
        };
    }

    fn interpret(&self, stack: &mut Stack) -> InterpretResult {
        let mut idx = 0;
        while idx < self.instructions.len() {
            let consumed: usize;
            if cfg!(debug_assertions) {
                println!("{}", stack.debug());
            }
            match self.instructions[idx] {
                CONSTANT => {
                    let inst;
                    (consumed, inst) = Constant::marshal(self, idx);
                    if cfg!(debug_assertions) {
                        println!("{:0>4} {:0>4} {}", idx, self.lines[idx], inst.disassemble());
                    }
                    stack.push(inst.value);
                }
                ADD => {
                    let inst;
                    (consumed, inst) = Add::marshal(self, idx);
                    if cfg!(debug_assertions) {
                        println!("{:0>4} {:0>4} {}", idx, self.lines[idx], inst.disassemble());
                    }
                    let b = stack.pop();
                    let a = stack.pop();
                    stack.push(a + b);
                }
                SUBTRACT => {
                    let inst;
                    (consumed, inst) = Subtract::marshal(self, idx);
                    if cfg!(debug_assertions) {
                        println!("{:0>4} {:0>4} {}", idx, self.lines[idx], inst.disassemble());
                    }
                    let b = stack.pop();
                    let a = stack.pop();
                    stack.push(a - b);
                }
                MULTIPLY => {
                    let inst;
                    (consumed, inst) = Multiply::marshal(self, idx);
                    if cfg!(debug_assertions) {
                        println!("{:0>4} {:0>4} {}", idx, self.lines[idx], inst.disassemble());
                    }
                    let b = stack.pop();
                    let a = stack.pop();
                    stack.push(a * b);
                }
                DIVIDE => {
                    let inst;
                    (consumed, inst) = Divide::marshal(self, idx);
                    if cfg!(debug_assertions) {
                        println!("{:0>4} {:0>4} {}", idx, self.lines[idx], inst.disassemble());
                    }
                    let b = stack.pop();
                    let a = stack.pop();
                    stack.push(a / b);
                }
                NEGATE => {
                    let inst;
                    (consumed, inst) = Negate::marshal(self, idx);
                    if cfg!(debug_assertions) {
                        println!("{:0>4} {:0>4} {}", idx, self.lines[idx], inst.disassemble());
                    }
                    let value = -stack.pop();
                    stack.push(value);
                }
                RETURN => {
                    let inst;
                    (_, inst) = Return::marshal(self, idx);
                    if cfg!(debug_assertions) {
                        println!("{:0>4} {:0>4} {}", idx, self.lines[idx], inst.disassemble());
                    }
                    println!("{}", stack.pop());
                    return InterpretResult::Ok;
                }
                _ => return InterpretResult::RuntimeError,
            }
            idx += consumed;
        }
        return InterpretResult::RuntimeError;
    }

    fn disassemble(&self, name: &str) {
        println!("=== {name} chunk ===");
        let mut idx = 0;
        while idx < self.instructions.len() {
            let (consumed, disassembled) = match self.instructions[idx] {
                CONSTANT => {
                    let (c, inst) = Constant::marshal(&self, idx);
                    (c, inst.disassemble())
                }
                ADD => {
                    let (c, inst) = Add::marshal(&self, idx);
                    (c, inst.disassemble())
                }
                SUBTRACT => {
                    let (c, inst) = Subtract::marshal(&self, idx);
                    (c, inst.disassemble())
                }
                MULTIPLY => {
                    let (c, inst) = Multiply::marshal(&self, idx);
                    (c, inst.disassemble())
                }
                DIVIDE => {
                    let (c, inst) = Divide::marshal(&self, idx);
                    (c, inst.disassemble())
                }
                NEGATE => {
                    let (c, inst) = Negate::marshal(&self, idx);
                    (c, inst.disassemble())
                }
                RETURN => {
                    let (c, inst) = Return::marshal(&self, idx);
                    (c, inst.disassemble())
                }
                _ => panic!("unknown op code"),
            };
            println!("{:0>4} {:0>4} {}", idx, self.lines[idx], disassembled);
            idx += consumed;
        }
    }
}

type Value = f64;

struct Pool {
    constants: Vec<Value>,
}

impl Pool {
    fn new() -> Pool {
        return Pool {
            constants: Vec::new(),
        };
    }

    fn add(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        return self.constants.len() - 1;
    }

    fn get(&self, location: usize) -> f64 {
        return self.constants[location];
    }
}

pub type Line = u16;

const RETURN: u8 = 1;
const CONSTANT: u8 = 2;
const NEGATE: u8 = 3;
const ADD: u8 = 4;
const SUBTRACT: u8 = 5;
const MULTIPLY: u8 = 6;
const DIVIDE: u8 = 7;

pub trait Instruction {
    /// Marshals a new instance of this instruction from a `Chunk` starting at a
    /// byte offset and returns the number of bytes consumed and the new
    /// instance.
    ///
    /// The offset must point to the op code byte.
    fn marshal(chunk: &Chunk, offset: usize) -> (usize, Self)
    where
        Self: Sized;

    /// Unmarshals this instruction into bytes in a `Chunk` with an associated
    /// line number.
    fn unmarshal(&self, chunk: &mut Chunk, line: Line);

    /// Returns a debug string that describes this instruction and an assocaited
    /// line number.
    fn disassemble(&self) -> String;
}

pub struct Return {}

impl Return {
    pub fn new() -> Return {
        return Return {};
    }

    pub fn write(&self, chunk: &mut Chunk, line: Line) {
        self.unmarshal(chunk, line);
    }
}

impl Instruction for Return {
    fn marshal(_chunk: &Chunk, _offset: usize) -> (usize, Self) {
        return (1, Return {});
    }

    fn unmarshal(&self, chunk: &mut Chunk, line: Line) {
        chunk.instructions.push(RETURN);
        chunk.lines.push(line);
    }

    fn disassemble(&self) -> String {
        "RETURN".to_string()
    }
}

pub struct Negate {}

impl Negate {
    pub fn new() -> Negate {
        return Negate {};
    }

    pub fn write(&self, chunk: &mut Chunk, line: Line) {
        self.unmarshal(chunk, line);
    }
}

impl Instruction for Negate {
    fn marshal(_chunk: &Chunk, _offset: usize) -> (usize, Self) {
        return (1, Negate {});
    }

    fn unmarshal(&self, chunk: &mut Chunk, line: Line) {
        chunk.instructions.push(NEGATE);
        chunk.lines.push(line);
    }

    fn disassemble(&self) -> String {
        "NEGATE".to_string()
    }
}

pub struct Add {}

impl Add {
    pub fn new() -> Add {
        return Add {};
    }

    pub fn write(&self, chunk: &mut Chunk, line: Line) {
        self.unmarshal(chunk, line);
    }
}

impl Instruction for Add {
    fn marshal(_chunk: &Chunk, _offset: usize) -> (usize, Self) {
        return (1, Add {});
    }

    fn unmarshal(&self, chunk: &mut Chunk, line: Line) {
        chunk.instructions.push(ADD);
        chunk.lines.push(line);
    }

    fn disassemble(&self) -> String {
        "ADD".to_string()
    }
}

pub struct Subtract {}

impl Subtract {
    pub fn new() -> Subtract {
        return Subtract {};
    }

    pub fn write(&self, chunk: &mut Chunk, line: Line) {
        self.unmarshal(chunk, line);
    }
}

impl Instruction for Subtract {
    fn marshal(_chunk: &Chunk, _offset: usize) -> (usize, Self) {
        return (1, Subtract {});
    }

    fn unmarshal(&self, chunk: &mut Chunk, line: Line) {
        chunk.instructions.push(SUBTRACT);
        chunk.lines.push(line);
    }

    fn disassemble(&self) -> String {
        "SUBTRACT".to_string()
    }
}

pub struct Multiply {}

impl Multiply {
    pub fn new() -> Multiply {
        return Multiply {};
    }

    pub fn write(&self, chunk: &mut Chunk, line: Line) {
        self.unmarshal(chunk, line);
    }
}

impl Instruction for Multiply {
    fn marshal(_chunk: &Chunk, _offset: usize) -> (usize, Self) {
        return (1, Multiply {});
    }

    fn unmarshal(&self, chunk: &mut Chunk, line: Line) {
        chunk.instructions.push(MULTIPLY);
        chunk.lines.push(line);
    }

    fn disassemble(&self) -> String {
        "MULTIPLY".to_string()
    }
}

pub struct Divide {}

impl Divide {
    pub fn new() -> Divide {
        return Divide {};
    }

    pub fn write(&self, chunk: &mut Chunk, line: Line) {
        self.unmarshal(chunk, line);
    }
}

impl Instruction for Divide {
    fn marshal(_chunk: &Chunk, _offset: usize) -> (usize, Self) {
        return (1, Divide {});
    }

    fn unmarshal(&self, chunk: &mut Chunk, line: Line) {
        chunk.instructions.push(DIVIDE);
        chunk.lines.push(line);
    }

    fn disassemble(&self) -> String {
        "DIVIDE".to_string()
    }
}

pub struct Constant {
    value: f64,
}

impl Constant {
    pub fn new(value: f64) -> Constant {
        return Constant { value: value };
    }

    pub fn write(&self, chunk: &mut Chunk, line: Line) {
        self.unmarshal(chunk, line);
    }
}

impl Instruction for Constant {
    fn marshal(chunk: &Chunk, offset: usize) -> (usize, Self) {
        let loc = chunk.instructions[offset + 1];
        return (
            2,
            Constant {
                value: chunk.pool.get(loc.into()),
            },
        );
    }

    fn unmarshal(&self, chunk: &mut Chunk, line: Line) {
        chunk.instructions.push(CONSTANT);
        chunk.instructions.push(chunk.pool.add(self.value) as u8);
        chunk.lines.push(line);
        chunk.lines.push(0);
    }

    fn disassemble(&self) -> String {
        format!("CONSTANT: {}", self.value)
    }
}
