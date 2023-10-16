
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
    pub fn disassemble(&self, name: &str) {
        println!("=== {name} chunk ===");
        let mut iter = self.instructions.iter().enumerate();
        let mut elem = iter.next();
        while !elem.is_none() {
            let (idx, byte) = elem.unwrap();
            let (consumed, disassembled, line) = match *byte {
                RETURN => {
                    let (c, inst) = Return::marshal(&self, idx);
                    let (d, l) = inst.disassemble();
                    (c, d, l)
                }
                CONSTANT => {
                    let (c, inst) = Constant::marshal(&self, idx);
                    let (d, l) = inst.disassemble();
                    (c, d, l)
                }
                _ => panic!("unknown op code"),
            };
            for _ in 1..consumed {
                iter.next();
            }
            println!("{:0>4} {:0>4} {}", idx, line, disassembled);
            elem = iter.next();
        }
    }
    fn add_line(&mut self, line: u32, length: u8) {
        let exists = match self.lines.last() {
            Some(l) => l.number == line,
            None => false,
        };
        if exists {
            self.lines.last_mut().unwrap().length += length as u32;
        } else {
            self.lines.push(Line { number: line, length: length as u32 });
        }
    }
    fn get_line(&self, mut offset: usize) -> u32 {
        for line in self.lines.iter() {
            if offset >= line.length as usize {
                offset -= line.length as usize;
            } else {
                return line.number;
            }
        }
        return 0;
    }
}

type Value = f64;

struct Pool {
    constants: Vec<Value>,
}

impl Pool {
    fn new() -> Pool {
        return Pool { constants: Vec::new() };
    }
    fn add(&mut self, constant: Value) -> usize {
        self.constants.push(constant);
        return self.constants.len() - 1;
    }
    fn get(&self, location: usize) -> f64 {
        return self.constants[location];
    }
}

struct Line {
    number: u32,
    length: u32
}

const RETURN: u8 = 1;
const CONSTANT: u8 = 2;

pub trait Instruction {
    /// Marshals a new instance of this instruction from a `Chunk` starting at a
    /// byte offset and returns the number of bytes consumed and the new
    /// instance.
    ///
    /// The offset must point to the op code byte.
    fn marshal(chunk: &Chunk, offset: usize) -> (usize, Self)
    where
        Self: Sized;
    /// Unmarshals this instruction into bytes in a `Chunk`.
    fn unmarshal(&self, chunk: &mut Chunk);
    /// Returns a debug string that describes this instruction and an assocaited
    /// line number.
    fn disassemble(&self) -> (String, u32);
}

pub struct Return {
  line: u32
}

impl Return {
    pub fn new(line: u32) -> Return {
        return Return { line: line };
    }
    pub fn write(&self, chunk: &mut Chunk) {
        self.unmarshal(chunk);
    }
}

impl Instruction for Return {
    fn marshal(chunk: &Chunk, offset: usize) -> (usize, Self) {
        return (1, Return { line: chunk.get_line(offset) });
    }
    fn unmarshal(&self, chunk: &mut Chunk) {
        chunk.instructions.push(RETURN);
        chunk.add_line(self.line, 1);
    }
    fn disassemble(&self) -> (String, u32) {
        ("RETURN".to_string(), self.line)
    }
}

pub struct Constant {
    value: f64,
    line: u32,
}

impl Constant {
    pub fn new(value: f64, line: u32) -> Constant {
        return Constant { value: value, line: line };
    }
    pub fn write(&self, chunk: &mut Chunk) {
        self.unmarshal(chunk);
    }
}

impl Instruction for Constant {
    fn marshal(chunk: &Chunk, offset: usize) -> (usize, Self) {
        let loc = chunk.instructions[offset + 1];
        return (2, Constant { value: chunk.pool.get(loc.into()), line: chunk.get_line(offset) });
    }
    fn unmarshal(&self, chunk: &mut Chunk) {
        chunk.instructions.push(CONSTANT);
        chunk.instructions.push(chunk.pool.add(self.value) as u8);
        chunk.add_line(self.line, 2);
    }
    fn disassemble(&self) -> (String, u32) {
        (format!("CONSTANT: {}", self.value), self.line)
    }
}

