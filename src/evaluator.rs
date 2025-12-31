//! Evaluator for the Slvr language
//!
//! Interprets AST directly without compilation.

use crate::ast::*;
use crate::error::{SlvrError, SlvrResult};
use crate::value::Value;
use dashmap::DashMap;
use indexmap::IndexMap;
use std::sync::Arc;

/// Evaluator for Slvr language
pub struct Evaluator {
    /// Global variables
    globals: Arc<DashMap<String, Value>>,
    /// Local variable scopes
    locals: Vec<IndexMap<String, Value>>,
    /// Recursion depth tracking
    recursion_depth: usize,
    /// Maximum recursion depth
    max_recursion_depth: usize,
}

impl Evaluator {
    /// Create a new evaluator
    pub fn new() -> Self {
        Self {
            globals: Arc::new(DashMap::new()),
            locals: vec![IndexMap::new()],
            recursion_depth: 0,
            max_recursion_depth: 1024,
        }
    }

    /// Create evaluator with custom recursion limit
    pub fn with_recursion_limit(max_depth: usize) -> Self {
        Self {
            globals: Arc::new(DashMap::new()),
            locals: vec![IndexMap::new()],
            recursion_depth: 0,
            max_recursion_depth: max_depth,
        }
    }

    /// Evaluate an expression
    pub fn eval(&mut self, expr: &Expr) -> SlvrResult<Value> {
        self.eval_expr(expr)
    }

    /// Evaluate a program
    pub fn eval_program(&mut self, program: &Program) -> SlvrResult<Value> {
        let mut result = Value::Unit;
        for def in &program.definitions {
            result = self.eval_definition(def)?;
        }
        Ok(result)
    }

    fn eval_definition(&mut self, def: &Definition) -> SlvrResult<Value> {
        match def {
            Definition::Module { body, .. } => {
                self.push_scope();
                let mut result = Value::Unit;
                for inner_def in body {
                    result = self.eval_definition(inner_def)?;
                }
                self.pop_scope();
                Ok(result)
            }
            Definition::Constant { name, value, .. } => {
                let val = self.eval_expr(value)?;
                self.set_global(name.clone(), val.clone());
                Ok(val)
            }
            _ => Ok(Value::Unit),
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> SlvrResult<Value> {
        // Check recursion depth
        if self.recursion_depth >= self.max_recursion_depth {
            return Err(SlvrError::runtime(format!(
                "Maximum recursion depth ({}) exceeded",
                self.max_recursion_depth
            )));
        }

        self.recursion_depth += 1;
        let result = match expr {
            Expr::Literal(lit) => self.eval_literal(lit),
            Expr::Variable(name) => self.get_variable(name),
            Expr::BinOp { op, left, right } => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                self.eval_binop(*op, left_val, right_val)
            }
            Expr::UnaryOp { op, operand } => {
                let val = self.eval_expr(operand)?;
                self.eval_unaryop(*op, val)
            }
            Expr::Call { function, args } => {
                if let Expr::Variable(func_name) = &**function {
                    let arg_vals: SlvrResult<Vec<_>> =
                        args.iter().map(|a| self.eval_expr(a)).collect();
                    self.call_function(func_name, arg_vals?)
                } else {
                    Err(SlvrError::runtime("Invalid function call"))
                }
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.eval_expr(condition)?;
                if cond_val.is_truthy() {
                    self.eval_expr(then_branch)
                } else if let Some(else_expr) = else_branch {
                    self.eval_expr(else_expr)
                } else {
                    Ok(Value::Unit)
                }
            }
            Expr::Let { name, value, body } => {
                let val = self.eval_expr(value)?;
                self.push_scope();
                self.set_local(name.clone(), val);
                let result = self.eval_expr(body)?;
                self.pop_scope();
                Ok(result)
            }
            Expr::List(elements) => {
                let vals: SlvrResult<Vec<_>> = elements.iter().map(|e| self.eval_expr(e)).collect();
                Ok(Value::List(vals?))
            }
            Expr::Object(fields) => {
                let mut obj = std::collections::HashMap::new();
                for (key, value) in fields {
                    obj.insert(key.clone(), self.eval_expr(value)?);
                }
                Ok(Value::Object(obj))
            }
            Expr::FieldAccess { object, field } => {
                let obj_val = self.eval_expr(object)?;
                obj_val.get_field(field)
            }
            Expr::Index { object, index } => {
                let obj_val = self.eval_expr(object)?;
                let idx_val = self.eval_expr(index)?;
                let idx = idx_val.to_integer()? as usize;
                obj_val.get_list_element(idx)
            }
            Expr::Block(exprs) => {
                let mut result = Value::Unit;
                for expr in exprs {
                    result = self.eval_expr(expr)?;
                }
                Ok(result)
            }
            Expr::Read { table, key } => {
                let key_val = self.eval_expr(key)?;
                let key_str = key_val.to_string_value()?;
                let table_key = format!("{}:{}", table, key_str);
                Ok(self
                    .globals
                    .get(&table_key)
                    .map(|v| v.clone())
                    .unwrap_or(Value::Null))
            }
            Expr::Write { table, key, value } => {
                let key_val = self.eval_expr(key)?;
                let key_str = key_val.to_string_value()?;
                let val = self.eval_expr(value)?;
                let table_key = format!("{}:{}", table, key_str);
                self.globals.insert(table_key, val.clone());
                Ok(val)
            }
            Expr::Update {
                table,
                key,
                updates,
            } => {
                let key_val = self.eval_expr(key)?;
                let key_str = key_val.to_string_value()?;
                let table_key = format!("{}:{}", table, key_str);

                // Evaluate all field values first
                let mut field_values = Vec::new();
                for (field_name, field_expr) in updates {
                    let field_val = self.eval_expr(field_expr)?;
                    field_values.push((field_name.clone(), field_val));
                }

                // Then update the object
                if let Some(mut current) = self.globals.get_mut(&table_key) {
                    if let Value::Object(ref mut obj) = *current {
                        for (field_name, field_val) in field_values {
                            obj.insert(field_name, field_val);
                        }
                    }
                }

                Ok(self
                    .globals
                    .get(&table_key)
                    .map(|v| v.clone())
                    .unwrap_or(Value::Null))
            }
            Expr::Delete { table, key } => {
                let key_val = self.eval_expr(key)?;
                let key_str = key_val.to_string_value()?;
                let table_key = format!("{}:{}", table, key_str);
                Ok(self
                    .globals
                    .remove(&table_key)
                    .map(|(_, v)| v)
                    .unwrap_or(Value::Null))
            }
        };

        self.recursion_depth -= 1;
        result
    }

    fn eval_literal(&self, lit: &Literal) -> SlvrResult<Value> {
        Ok(match lit {
            Literal::Integer(n) => Value::Integer(*n),
            Literal::Decimal(d) => Value::Decimal(*d),
            Literal::String(s) => Value::String(s.clone()),
            Literal::Boolean(b) => Value::Boolean(*b),
            Literal::Unit => Value::Unit,
            Literal::Null => Value::Null,
        })
    }

    fn eval_binop(&self, op: BinOp, left: Value, right: Value) -> SlvrResult<Value> {
        match op {
            BinOp::Add => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
                (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Decimal(a + b)),
                (Value::Integer(a), Value::Decimal(b)) => Ok(Value::Decimal(a as f64 + b)),
                (Value::Decimal(a), Value::Integer(b)) => Ok(Value::Decimal(a + b as f64)),
                _ => Err(SlvrError::type_mismatch("numeric", "non-numeric")),
            },
            BinOp::Subtract => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
                (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Decimal(a - b)),
                (Value::Integer(a), Value::Decimal(b)) => Ok(Value::Decimal(a as f64 - b)),
                (Value::Decimal(a), Value::Integer(b)) => Ok(Value::Decimal(a - b as f64)),
                _ => Err(SlvrError::type_mismatch("numeric", "non-numeric")),
            },
            BinOp::Multiply => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Decimal(a * b)),
                (Value::Integer(a), Value::Decimal(b)) => Ok(Value::Decimal(a as f64 * b)),
                (Value::Decimal(a), Value::Integer(b)) => Ok(Value::Decimal(a * b as f64)),
                _ => Err(SlvrError::type_mismatch("numeric", "non-numeric")),
            },
            BinOp::Divide => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => {
                    if b == 0 {
                        Err(SlvrError::DivisionByZero)
                    } else {
                        Ok(Value::Integer(a / b))
                    }
                }
                (Value::Decimal(a), Value::Decimal(b)) => {
                    if b == 0.0 {
                        Err(SlvrError::DivisionByZero)
                    } else {
                        Ok(Value::Decimal(a / b))
                    }
                }
                (Value::Integer(a), Value::Decimal(b)) => {
                    if b == 0.0 {
                        Err(SlvrError::DivisionByZero)
                    } else {
                        Ok(Value::Decimal(a as f64 / b))
                    }
                }
                (Value::Decimal(a), Value::Integer(b)) => {
                    if b == 0 {
                        Err(SlvrError::DivisionByZero)
                    } else {
                        Ok(Value::Decimal(a / b as f64))
                    }
                }
                _ => Err(SlvrError::type_mismatch("numeric", "non-numeric")),
            },
            BinOp::Modulo => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => {
                    if b == 0 {
                        Err(SlvrError::DivisionByZero)
                    } else {
                        Ok(Value::Integer(a % b))
                    }
                }
                _ => Err(SlvrError::type_mismatch("integer", "non-integer")),
            },
            BinOp::Power => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => {
                    if b < 0 {
                        Ok(Value::Decimal((a as f64).powf(b as f64)))
                    } else {
                        Ok(Value::Integer(a.pow(b as u32)))
                    }
                }
                (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Decimal(a.powf(b))),
                (Value::Integer(a), Value::Decimal(b)) => Ok(Value::Decimal((a as f64).powf(b))),
                (Value::Decimal(a), Value::Integer(b)) => Ok(Value::Decimal(a.powf(b as f64))),
                _ => Err(SlvrError::type_mismatch("numeric", "non-numeric")),
            },
            BinOp::Equal => Ok(Value::Boolean(left == right)),
            BinOp::NotEqual => Ok(Value::Boolean(left != right)),
            BinOp::Less => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a < b)),
                (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Boolean(a < b)),
                (Value::Integer(a), Value::Decimal(b)) => Ok(Value::Boolean((a as f64) < b)),
                (Value::Decimal(a), Value::Integer(b)) => Ok(Value::Boolean(a < (b as f64))),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a < b)),
                _ => Err(SlvrError::type_mismatch("comparable", "non-comparable")),
            },
            BinOp::LessEqual => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a <= b)),
                (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Boolean(a <= b)),
                (Value::Integer(a), Value::Decimal(b)) => Ok(Value::Boolean((a as f64) <= b)),
                (Value::Decimal(a), Value::Integer(b)) => Ok(Value::Boolean(a <= (b as f64))),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a <= b)),
                _ => Err(SlvrError::type_mismatch("comparable", "non-comparable")),
            },
            BinOp::Greater => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a > b)),
                (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Boolean(a > b)),
                (Value::Integer(a), Value::Decimal(b)) => Ok(Value::Boolean((a as f64) > b)),
                (Value::Decimal(a), Value::Integer(b)) => Ok(Value::Boolean(a > (b as f64))),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a > b)),
                _ => Err(SlvrError::type_mismatch("comparable", "non-comparable")),
            },
            BinOp::GreaterEqual => match (left, right) {
                (Value::Integer(a), Value::Integer(b)) => Ok(Value::Boolean(a >= b)),
                (Value::Decimal(a), Value::Decimal(b)) => Ok(Value::Boolean(a >= b)),
                (Value::Integer(a), Value::Decimal(b)) => Ok(Value::Boolean((a as f64) >= b)),
                (Value::Decimal(a), Value::Integer(b)) => Ok(Value::Boolean(a >= (b as f64))),
                (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a >= b)),
                _ => Err(SlvrError::type_mismatch("comparable", "non-comparable")),
            },
            BinOp::And => Ok(Value::Boolean(left.is_truthy() && right.is_truthy())),
            BinOp::Or => Ok(Value::Boolean(left.is_truthy() || right.is_truthy())),
            BinOp::Concat => {
                let left_str = left.to_string_value()?;
                let right_str = right.to_string_value()?;
                Ok(Value::String(format!("{}{}", left_str, right_str)))
            }
        }
    }

    fn eval_unaryop(&self, op: UnaryOp, operand: Value) -> SlvrResult<Value> {
        match op {
            UnaryOp::Not => Ok(Value::Boolean(!operand.is_truthy())),
            UnaryOp::Negate => match operand {
                Value::Integer(n) => Ok(Value::Integer(-n)),
                Value::Decimal(d) => Ok(Value::Decimal(-d)),
                _ => Err(SlvrError::type_mismatch("numeric", operand.type_name())),
            },
        }
    }

    fn call_function(&mut self, _name: &str, _args: Vec<Value>) -> SlvrResult<Value> {
        // Built-in functions would be handled here
        Err(SlvrError::undefined_func(_name))
    }

    fn get_variable(&self, name: &str) -> SlvrResult<Value> {
        // Check local scopes from innermost to outermost
        for scope in self.locals.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Ok(val.clone());
            }
        }

        // Check globals
        if let Some(val) = self.globals.get(name) {
            return Ok(val.clone());
        }

        Err(SlvrError::undefined_var(name))
    }

    fn set_local(&mut self, name: String, value: Value) {
        if let Some(scope) = self.locals.last_mut() {
            scope.insert(name, value);
        }
    }

    fn set_global(&self, name: String, value: Value) {
        self.globals.insert(name, value);
    }

    fn push_scope(&mut self) {
        self.locals.push(IndexMap::new());
    }

    fn pop_scope(&mut self) {
        if self.locals.len() > 1 {
            self.locals.pop();
        }
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluator_creation() {
        let evaluator = Evaluator::new();
        assert_eq!(evaluator.recursion_depth, 0);
    }

    #[test]
    fn test_eval_literal() {
        let mut evaluator = Evaluator::new();
        let expr = Expr::Literal(Literal::Integer(42));
        let result = evaluator.eval(&expr).unwrap();
        assert_eq!(result, Value::Integer(42));
    }

    #[test]
    fn test_eval_arithmetic() {
        let mut evaluator = Evaluator::new();
        let expr = Expr::BinOp {
            op: BinOp::Add,
            left: Box::new(Expr::Literal(Literal::Integer(2))),
            right: Box::new(Expr::Literal(Literal::Integer(3))),
        };
        let result = evaluator.eval(&expr).unwrap();
        assert_eq!(result, Value::Integer(5));
    }
}
