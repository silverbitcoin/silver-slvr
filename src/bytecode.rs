//! Bytecode representation for the Slvr language
//!
//! Complete bytecode instruction set for the Slvr virtual machine.

use crate::ast::Type as AstType;
use crate::types::Type;
use serde::{Deserialize, Serialize};

/// Bytecode instruction set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    // Stack operations
    PushInt(i128),
    PushDecimal(f64),
    PushString(String),
    PushBool(bool),
    PushUnit,
    PushNull,
    Pop,
    Dup,

    // Arithmetic operations
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    Negate,

    // Comparison operations
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Logical operations
    And,
    Or,
    Not,

    // String operations
    Concat,

    // Control flow
    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),
    Return,

    // Variable operations
    LoadLocal(usize),
    StoreLocal(usize),
    LoadGlobal(String),
    StoreGlobal(String),

    // Function operations
    Call(String, usize), // function name, arg count

    // Collection operations
    MakeList(usize),
    MakeObject(usize),
    GetField(String),
    GetIndex,
    SetField(String),
    SetIndex,

    // Database operations
    Read(String),
    Write(String),
    Update(String, usize), // table name, field count
    Delete(String),

    // Type operations
    TypeOf,
    Cast(Type),

    // Error handling
    Throw(String),

    // Fuel operations
    ConsumeFuel(u64),
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::PushInt(n) => write!(f, "PUSH_INT {}", n),
            Instruction::PushDecimal(d) => write!(f, "PUSH_DECIMAL {}", d),
            Instruction::PushString(s) => write!(f, "PUSH_STRING \"{}\"", s),
            Instruction::PushBool(b) => write!(f, "PUSH_BOOL {}", b),
            Instruction::PushUnit => write!(f, "PUSH_UNIT"),
            Instruction::PushNull => write!(f, "PUSH_NULL"),
            Instruction::Pop => write!(f, "POP"),
            Instruction::Dup => write!(f, "DUP"),
            Instruction::Add => write!(f, "ADD"),
            Instruction::Subtract => write!(f, "SUB"),
            Instruction::Multiply => write!(f, "MUL"),
            Instruction::Divide => write!(f, "DIV"),
            Instruction::Modulo => write!(f, "MOD"),
            Instruction::Power => write!(f, "POW"),
            Instruction::Negate => write!(f, "NEG"),
            Instruction::Equal => write!(f, "EQ"),
            Instruction::NotEqual => write!(f, "NE"),
            Instruction::Less => write!(f, "LT"),
            Instruction::LessEqual => write!(f, "LE"),
            Instruction::Greater => write!(f, "GT"),
            Instruction::GreaterEqual => write!(f, "GE"),
            Instruction::And => write!(f, "AND"),
            Instruction::Or => write!(f, "OR"),
            Instruction::Not => write!(f, "NOT"),
            Instruction::Concat => write!(f, "CONCAT"),
            Instruction::Jump(addr) => write!(f, "JMP {}", addr),
            Instruction::JumpIfFalse(addr) => write!(f, "JMP_FALSE {}", addr),
            Instruction::JumpIfTrue(addr) => write!(f, "JMP_TRUE {}", addr),
            Instruction::Return => write!(f, "RET"),
            Instruction::LoadLocal(idx) => write!(f, "LOAD_LOCAL {}", idx),
            Instruction::StoreLocal(idx) => write!(f, "STORE_LOCAL {}", idx),
            Instruction::LoadGlobal(name) => write!(f, "LOAD_GLOBAL {}", name),
            Instruction::StoreGlobal(name) => write!(f, "STORE_GLOBAL {}", name),
            Instruction::Call(name, argc) => write!(f, "CALL {} ({})", name, argc),
            Instruction::MakeList(len) => write!(f, "MAKE_LIST {}", len),
            Instruction::MakeObject(len) => write!(f, "MAKE_OBJECT {}", len),
            Instruction::GetField(name) => write!(f, "GET_FIELD {}", name),
            Instruction::GetIndex => write!(f, "GET_INDEX"),
            Instruction::SetField(name) => write!(f, "SET_FIELD {}", name),
            Instruction::SetIndex => write!(f, "SET_INDEX"),
            Instruction::Read(table) => write!(f, "READ {}", table),
            Instruction::Write(table) => write!(f, "WRITE {}", table),
            Instruction::Update(table, count) => write!(f, "UPDATE {} ({})", table, count),
            Instruction::Delete(table) => write!(f, "DELETE {}", table),
            Instruction::TypeOf => write!(f, "TYPEOF"),
            Instruction::Cast(ty) => write!(f, "CAST {}", ty),
            Instruction::Throw(msg) => write!(f, "THROW \"{}\"", msg),
            Instruction::ConsumeFuel(amount) => write!(f, "CONSUME_FUEL {}", amount),
        }
    }
}

/// Function definition in bytecode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<(String, AstType)>,
    pub return_type: Type,
    pub bytecode: Bytecode,
}

/// Compiled bytecode program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
}

impl Bytecode {
    /// Create new bytecode
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
        }
    }

    /// Add an instruction
    pub fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    /// Add multiple instructions
    pub fn extend(&mut self, other: Bytecode) {
        self.instructions.extend(other.instructions);
    }

    /// Get current instruction pointer
    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    /// Check if bytecode is empty
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    /// Get instruction at index
    pub fn get(&self, index: usize) -> Option<&Instruction> {
        self.instructions.get(index)
    }

    /// Get mutable instruction at index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Instruction> {
        self.instructions.get_mut(index)
    }

    /// Disassemble bytecode to string
    pub fn disassemble(&self) -> String {
        let mut output = String::new();
        for (i, instr) in self.instructions.iter().enumerate() {
            output.push_str(&format!("{:04}: {}\n", i, instr));
        }
        output
    }
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for Bytecode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.disassemble())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytecode_creation() {
        let bytecode = Bytecode::new();
        assert!(bytecode.is_empty());
        assert_eq!(bytecode.len(), 0);
    }

    #[test]
    fn test_bytecode_push() {
        let mut bytecode = Bytecode::new();
        bytecode.push(Instruction::PushInt(42));
        bytecode.push(Instruction::Return);

        assert_eq!(bytecode.len(), 2);
        assert!(matches!(bytecode.get(0), Some(Instruction::PushInt(42))));
    }

    #[test]
    fn test_bytecode_extend() {
        let mut bytecode1 = Bytecode::new();
        bytecode1.push(Instruction::PushInt(1));

        let mut bytecode2 = Bytecode::new();
        bytecode2.push(Instruction::PushInt(2));

        bytecode1.extend(bytecode2);
        assert_eq!(bytecode1.len(), 2);
    }

    #[test]
    fn test_instruction_display() {
        let instr = Instruction::PushInt(42);
        assert_eq!(instr.to_string(), "PUSH_INT 42");

        let instr = Instruction::Call("add".to_string(), 2);
        assert_eq!(instr.to_string(), "CALL add (2)");
    }

    #[test]
    fn test_bytecode_disassemble() {
        let mut bytecode = Bytecode::new();
        bytecode.push(Instruction::PushInt(42));
        bytecode.push(Instruction::Return);

        let disasm = bytecode.disassemble();
        assert!(disasm.contains("PUSH_INT 42"));
        assert!(disasm.contains("RET"));
    }
}
