//! Virtual Machine for executing Slvr bytecode
//!
//! Executes compiled bytecode with fuel metering and state management.

use crate::bytecode::{Bytecode, Instruction};
use crate::error::{SlvrError, SlvrResult};
use crate::runtime::Runtime;
use crate::value::Value;

use std::collections::HashMap;

/// Virtual Machine for Slvr bytecode execution
pub struct VirtualMachine {
    /// Bytecode to execute
    bytecode: Bytecode,
    /// Instruction pointer
    ip: usize,
    /// Execution stack
    stack: Vec<Value>,
    /// Local variable stack
    locals: Vec<Vec<Value>>,
    /// Global variables
    globals: HashMap<String, Value>,
    /// Runtime environment
    runtime: Runtime,
    /// Call stack for debugging
    call_stack: Vec<String>,
}

impl VirtualMachine {
    /// Create a new VM
    pub fn new(bytecode: Bytecode, runtime: Runtime) -> Self {
        Self {
            bytecode,
            ip: 0,
            stack: Vec::new(),
            locals: vec![Vec::new()],
            globals: HashMap::new(),
            runtime,
            call_stack: Vec::new(),
        }
    }

    /// Execute the bytecode
    pub fn execute(&mut self) -> SlvrResult<Value> {
        while self.ip < self.bytecode.instructions.len() {
            let instruction = self.bytecode.instructions[self.ip].clone();
            self.execute_instruction(&instruction)?;
            self.ip += 1;
        }

        if self.stack.is_empty() {
            Ok(Value::Unit)
        } else {
            self.stack
                .pop()
                .ok_or_else(|| SlvrError::runtime("Stack pop failed unexpectedly"))
        }
    }

    /// Get current stack
    pub fn stack(&self) -> &[Value] {
        &self.stack
    }

    /// Get runtime reference
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    fn execute_instruction(&mut self, instruction: &Instruction) -> SlvrResult<()> {
        match instruction {
            // Stack operations
            Instruction::PushInt(n) => self.stack.push(Value::Integer(*n)),
            Instruction::PushDecimal(d) => self.stack.push(Value::Decimal(*d)),
            Instruction::PushString(s) => self.stack.push(Value::String(s.clone())),
            Instruction::PushBool(b) => self.stack.push(Value::Boolean(*b)),
            Instruction::PushUnit => self.stack.push(Value::Unit),
            Instruction::PushNull => self.stack.push(Value::Null),
            Instruction::Pop => {
                if self.stack.is_empty() {
                    return Err(SlvrError::runtime("Stack underflow"));
                }
                self.stack.pop();
            }
            Instruction::Dup => {
                if self.stack.is_empty() {
                    return Err(SlvrError::runtime("Stack underflow"));
                }
                let val = self
                    .stack
                    .last()
                    .ok_or_else(|| SlvrError::runtime("Stack access failed"))?
                    .clone();
                self.stack.push(val);
            }

            // Arithmetic operations
            Instruction::Add => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = Self::add_values(a, b)?;
                self.stack.push(result);
            }
            Instruction::Subtract => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = Self::subtract_values(a, b)?;
                self.stack.push(result);
            }
            Instruction::Multiply => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = Self::multiply_values(a, b)?;
                self.stack.push(result);
            }
            Instruction::Divide => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = Self::divide_values(a, b)?;
                self.stack.push(result);
            }
            Instruction::Modulo => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = Self::modulo_values(a, b)?;
                self.stack.push(result);
            }
            Instruction::Power => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let result = Self::power_values(a, b)?;
                self.stack.push(result);
            }
            Instruction::Negate => {
                let val = self.pop_stack()?;
                match val {
                    Value::Integer(n) => self.stack.push(Value::Integer(-n)),
                    Value::Decimal(d) => self.stack.push(Value::Decimal(-d)),
                    _ => return Err(SlvrError::type_mismatch("numeric", val.type_name())),
                }
            }

            // Comparison operations
            Instruction::Equal => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.stack.push(Value::Boolean(a == b));
            }
            Instruction::NotEqual => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.stack.push(Value::Boolean(a != b));
            }
            Instruction::Less => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.stack
                    .push(Value::Boolean(Self::compare_values(&a, &b)? < 0));
            }
            Instruction::LessEqual => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.stack
                    .push(Value::Boolean(Self::compare_values(&a, &b)? <= 0));
            }
            Instruction::Greater => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.stack
                    .push(Value::Boolean(Self::compare_values(&a, &b)? > 0));
            }
            Instruction::GreaterEqual => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.stack
                    .push(Value::Boolean(Self::compare_values(&a, &b)? >= 0));
            }

            // Logical operations
            Instruction::And => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.stack
                    .push(Value::Boolean(a.is_truthy() && b.is_truthy()));
            }
            Instruction::Or => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                self.stack
                    .push(Value::Boolean(a.is_truthy() || b.is_truthy()));
            }
            Instruction::Not => {
                let val = self.pop_stack()?;
                self.stack.push(Value::Boolean(!val.is_truthy()));
            }

            // String operations
            Instruction::Concat => {
                let b = self.pop_stack()?;
                let a = self.pop_stack()?;
                let a_str = a.to_string_value()?;
                let b_str = b.to_string_value()?;
                self.stack
                    .push(Value::String(format!("{}{}", a_str, b_str)));
            }

            // Control flow
            Instruction::Jump(addr) => self.ip = *addr - 1,
            Instruction::JumpIfFalse(addr) => {
                let val = self.pop_stack()?;
                if !val.is_truthy() {
                    self.ip = *addr - 1;
                }
            }
            Instruction::JumpIfTrue(addr) => {
                let val = self.pop_stack()?;
                if val.is_truthy() {
                    self.ip = *addr - 1;
                }
            }
            Instruction::Return => {
                if !self.call_stack.is_empty() {
                    self.call_stack.pop();
                }
            }

            // Variable operations
            Instruction::LoadLocal(idx) => {
                if let Some(locals) = self.locals.last() {
                    if let Some(val) = locals.get(*idx) {
                        self.stack.push(val.clone());
                    } else {
                        return Err(SlvrError::runtime(format!(
                            "Local variable {} not found",
                            idx
                        )));
                    }
                }
            }
            Instruction::StoreLocal(idx) => {
                let val = self.pop_stack()?;
                if let Some(locals) = self.locals.last_mut() {
                    if *idx < locals.len() {
                        locals[*idx] = val;
                    } else {
                        locals.push(val);
                    }
                }
            }
            Instruction::LoadGlobal(name) => {
                if let Some(val) = self.globals.get(name) {
                    self.stack.push(val.clone());
                } else if let Some(val) = self.runtime.read(name) {
                    self.stack.push(val);
                } else {
                    return Err(SlvrError::undefined_var(name));
                }
            }
            Instruction::StoreGlobal(name) => {
                let val = self.pop_stack()?;
                self.globals.insert(name.clone(), val);
            }

            // Collection operations
            Instruction::MakeList(len) => {
                let mut list = Vec::new();
                for _ in 0..*len {
                    list.push(self.pop_stack()?);
                }
                list.reverse();
                self.stack.push(Value::List(list));
            }
            Instruction::MakeObject(len) => {
                let mut obj = std::collections::HashMap::new();
                for _ in 0..*len {
                    let val = self.pop_stack()?;
                    let key = self.pop_stack()?.to_string_value()?;
                    obj.insert(key, val);
                }
                self.stack.push(Value::Object(obj));
            }
            Instruction::GetField(name) => {
                let obj = self.pop_stack()?;
                let val = obj.get_field(name)?;
                self.stack.push(val);
            }
            Instruction::GetIndex => {
                let idx = self.pop_stack()?;
                let obj = self.pop_stack()?;
                let idx_usize = idx.to_integer()? as usize;
                let val = obj.get_list_element(idx_usize)?;
                self.stack.push(val);
            }
            Instruction::SetField(name) => {
                let val = self.pop_stack()?;
                let mut obj = self.pop_stack()?;
                obj.set_field(name.clone(), val)?;
                self.stack.push(obj);
            }
            Instruction::SetIndex => {
                let val = self.pop_stack()?;
                let idx = self.pop_stack()?;
                let mut obj = self.pop_stack()?;
                let idx_usize = idx.to_integer()? as usize;
                if let Value::List(ref mut list) = obj {
                    if idx_usize < list.len() {
                        list[idx_usize] = val;
                    } else {
                        return Err(SlvrError::IndexOutOfBounds {
                            index: idx_usize as i64,
                            length: list.len(),
                        });
                    }
                }
                self.stack.push(obj);
            }

            // Database operations
            Instruction::Read(table) => {
                let key = self.pop_stack()?.to_string_value()?;
                let table_key = format!("{}:{}", table, key);
                let val = self.runtime.read(&table_key).unwrap_or(Value::Null);
                self.stack.push(val);
            }
            Instruction::Write(table) => {
                let val = self.pop_stack()?;
                let key = self.pop_stack()?.to_string_value()?;
                let table_key = format!("{}:{}", table, key);
                self.runtime.write(table_key, val.clone())?;
                self.stack.push(val);
            }
            Instruction::Update(table, _count) => {
                let val = self.pop_stack()?;
                let key = self.pop_stack()?.to_string_value()?;
                let table_key = format!("{}:{}", table, key);
                self.runtime.update(&table_key, val.clone())?;
                self.stack.push(val);
            }
            Instruction::Delete(table) => {
                let key = self.pop_stack()?.to_string_value()?;
                let table_key = format!("{}:{}", table, key);
                let val = self.runtime.delete(&table_key)?;
                self.stack.push(val.unwrap_or(Value::Null));
            }

            // Type operations
            Instruction::TypeOf => {
                let val = self.pop_stack()?;
                self.stack.push(Value::String(val.type_name().to_string()));
            }
            Instruction::Cast(_ty) => {
                // Type casting - keep value as is
            }

            // Error handling
            Instruction::Throw(msg) => {
                return Err(SlvrError::runtime(msg));
            }

            // Fuel operations
            Instruction::ConsumeFuel(amount) => {
                self.runtime.consume_fuel(*amount)?;
            }

            // Function calls
            Instruction::Call(name, _argc) => {
                return Err(SlvrError::undefined_func(name));
            }
        }
        Ok(())
    }

    fn pop_stack(&mut self) -> SlvrResult<Value> {
        self.stack
            .pop()
            .ok_or_else(|| SlvrError::runtime("Stack underflow"))
    }

    fn add_values(a: Value, b: Value) -> SlvrResult<Value> {
        match (a, b) {
            (Value::Integer(x), Value::Integer(y)) => Ok(Value::Integer(x + y)),
            (Value::Decimal(x), Value::Decimal(y)) => Ok(Value::Decimal(x + y)),
            (Value::Integer(x), Value::Decimal(y)) => Ok(Value::Decimal(x as f64 + y)),
            (Value::Decimal(x), Value::Integer(y)) => Ok(Value::Decimal(x + y as f64)),
            _ => Err(SlvrError::type_mismatch("numeric", "non-numeric")),
        }
    }

    fn subtract_values(a: Value, b: Value) -> SlvrResult<Value> {
        match (a, b) {
            (Value::Integer(x), Value::Integer(y)) => Ok(Value::Integer(x - y)),
            (Value::Decimal(x), Value::Decimal(y)) => Ok(Value::Decimal(x - y)),
            (Value::Integer(x), Value::Decimal(y)) => Ok(Value::Decimal(x as f64 - y)),
            (Value::Decimal(x), Value::Integer(y)) => Ok(Value::Decimal(x - y as f64)),
            _ => Err(SlvrError::type_mismatch("numeric", "non-numeric")),
        }
    }

    fn multiply_values(a: Value, b: Value) -> SlvrResult<Value> {
        match (a, b) {
            (Value::Integer(x), Value::Integer(y)) => Ok(Value::Integer(x * y)),
            (Value::Decimal(x), Value::Decimal(y)) => Ok(Value::Decimal(x * y)),
            (Value::Integer(x), Value::Decimal(y)) => Ok(Value::Decimal(x as f64 * y)),
            (Value::Decimal(x), Value::Integer(y)) => Ok(Value::Decimal(x * y as f64)),
            _ => Err(SlvrError::type_mismatch("numeric", "non-numeric")),
        }
    }

    fn divide_values(a: Value, b: Value) -> SlvrResult<Value> {
        match (a, b) {
            (Value::Integer(x), Value::Integer(y)) => {
                if y == 0 {
                    Err(SlvrError::DivisionByZero)
                } else {
                    Ok(Value::Integer(x / y))
                }
            }
            (Value::Decimal(x), Value::Decimal(y)) => {
                if y == 0.0 {
                    Err(SlvrError::DivisionByZero)
                } else {
                    Ok(Value::Decimal(x / y))
                }
            }
            (Value::Integer(x), Value::Decimal(y)) => {
                if y == 0.0 {
                    Err(SlvrError::DivisionByZero)
                } else {
                    Ok(Value::Decimal(x as f64 / y))
                }
            }
            (Value::Decimal(x), Value::Integer(y)) => {
                if y == 0 {
                    Err(SlvrError::DivisionByZero)
                } else {
                    Ok(Value::Decimal(x / y as f64))
                }
            }
            _ => Err(SlvrError::type_mismatch("numeric", "non-numeric")),
        }
    }

    fn modulo_values(a: Value, b: Value) -> SlvrResult<Value> {
        match (a, b) {
            (Value::Integer(x), Value::Integer(y)) => {
                if y == 0 {
                    Err(SlvrError::DivisionByZero)
                } else {
                    Ok(Value::Integer(x % y))
                }
            }
            _ => Err(SlvrError::type_mismatch("integer", "non-integer")),
        }
    }

    fn power_values(a: Value, b: Value) -> SlvrResult<Value> {
        match (a, b) {
            (Value::Integer(x), Value::Integer(y)) => {
                if y < 0 {
                    Ok(Value::Decimal((x as f64).powf(y as f64)))
                } else {
                    Ok(Value::Integer(x.pow(y as u32)))
                }
            }
            (Value::Decimal(x), Value::Decimal(y)) => Ok(Value::Decimal(x.powf(y))),
            (Value::Integer(x), Value::Decimal(y)) => Ok(Value::Decimal((x as f64).powf(y))),
            (Value::Decimal(x), Value::Integer(y)) => Ok(Value::Decimal(x.powf(y as f64))),
            _ => Err(SlvrError::type_mismatch("numeric", "non-numeric")),
        }
    }

    fn compare_values(a: &Value, b: &Value) -> SlvrResult<i32> {
        match (a, b) {
            (Value::Integer(x), Value::Integer(y)) => Ok(if x < y {
                -1
            } else if x > y {
                1
            } else {
                0
            }),
            (Value::Decimal(x), Value::Decimal(y)) => Ok(if x < y {
                -1
            } else if x > y {
                1
            } else {
                0
            }),
            (Value::Integer(x), Value::Decimal(y)) => {
                let x_f = *x as f64;
                Ok(if x_f < *y {
                    -1
                } else if x_f > *y {
                    1
                } else {
                    0
                })
            }
            (Value::Decimal(x), Value::Integer(y)) => {
                let y_f = *y as f64;
                Ok(if x < &y_f {
                    -1
                } else if x > &y_f {
                    1
                } else {
                    0
                })
            }
            (Value::String(x), Value::String(y)) => Ok(if x < y {
                -1
            } else if x > y {
                1
            } else {
                0
            }),
            _ => Err(SlvrError::type_mismatch("comparable", "non-comparable")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_creation() {
        let bytecode = Bytecode::new();
        let runtime = Runtime::new(1_000_000);
        let vm = VirtualMachine::new(bytecode, runtime);
        assert!(vm.stack().is_empty());
    }

    #[test]
    fn test_vm_push_pop() {
        let mut bytecode = Bytecode::new();
        bytecode.push(Instruction::PushInt(42));
        bytecode.push(Instruction::PushInt(8));
        bytecode.push(Instruction::Add);

        let runtime = Runtime::new(1_000_000);
        let mut vm = VirtualMachine::new(bytecode, runtime);
        let result = vm.execute().unwrap();
        assert_eq!(result, Value::Integer(50));
    }
}
