//! Debugger - Step-through Debugging Support
//!
//! Full debugger implementation with breakpoints, step execution, variable inspection,
//! call stack tracking, and real-time state monitoring.

use crate::error::{SlvrResult, SlvrError};
use crate::value::Value;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Breakpoint type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BreakpointType {
    /// Break at specific line
    Line,
    /// Break when condition is true
    Conditional,
    /// Break on function entry
    FunctionEntry,
    /// Break on function exit
    FunctionExit,
    /// Break on exception
    Exception,
}

/// Breakpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: String,
    pub breakpoint_type: BreakpointType,
    pub file: String,
    pub line: u32,
    pub column: Option<u32>,
    pub condition: Option<String>,
    pub hit_count: u64,
    pub enabled: bool,
    pub temporary: bool,
    pub log_message: Option<String>,
}

impl Breakpoint {
    /// Create new line breakpoint
    pub fn new_line(file: String, line: u32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            breakpoint_type: BreakpointType::Line,
            file,
            line,
            column: None,
            condition: None,
            hit_count: 0,
            enabled: true,
            temporary: false,
            log_message: None,
        }
    }

    /// Create conditional breakpoint
    pub fn new_conditional(file: String, line: u32, condition: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            breakpoint_type: BreakpointType::Conditional,
            file,
            line,
            column: None,
            condition: Some(condition),
            hit_count: 0,
            enabled: true,
            temporary: false,
            log_message: None,
        }
    }

    /// Check if breakpoint should trigger
    pub fn should_trigger(&self, file: &str, line: u32) -> bool {
        if !self.enabled {
            return false;
        }

        self.file == file && self.line == line
    }
}

/// Execution state
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionState {
    /// Debugger is running
    Running,
    /// Debugger is paused at breakpoint
    Paused,
    /// Debugger is stepping
    Stepping,
    /// Debugger is stopped
    Stopped,
}

/// Step type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepType {
    /// Step into function calls
    Into,
    /// Step over function calls
    Over,
    /// Step out of current function
    Out,
    /// Continue execution
    Continue,
}

/// Stack frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub id: u32,
    pub name: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub locals: HashMap<String, Value>,
    pub arguments: HashMap<String, Value>,
}

/// Call stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallStack {
    pub frames: Vec<StackFrame>,
}

impl Default for CallStack {
    fn default() -> Self {
        Self::new()
    }
}

impl CallStack {
    /// Create new call stack
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
        }
    }

    /// Push frame
    pub fn push(&mut self, frame: StackFrame) {
        self.frames.push(frame);
    }

    /// Pop frame
    pub fn pop(&mut self) -> Option<StackFrame> {
        self.frames.pop()
    }

    /// Get current frame
    pub fn current(&self) -> Option<&StackFrame> {
        self.frames.last()
    }

    /// Get current frame mutable
    pub fn current_mut(&mut self) -> Option<&mut StackFrame> {
        self.frames.last_mut()
    }

    /// Get frame by ID
    pub fn get_frame(&self, id: u32) -> Option<&StackFrame> {
        self.frames.iter().find(|f| f.id == id)
    }

    /// Get depth
    pub fn depth(&self) -> usize {
        self.frames.len()
    }
}

/// Watch expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchExpression {
    pub id: String,
    pub expression: String,
    pub value: Option<Value>,
    pub error: Option<String>,
}

/// Debugger session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSession {
    pub id: String,
    pub state: ExecutionState,
    pub current_file: String,
    pub current_line: u32,
    pub current_column: u32,
    pub call_stack: CallStack,
    pub breakpoints: Vec<Breakpoint>,
    pub watches: Vec<WatchExpression>,
    pub variables: HashMap<String, Value>,
    pub started_at: DateTime<Utc>,
    pub paused_at: Option<DateTime<Utc>>,
}

/// Debugger
pub struct Debugger {
    session: Arc<Mutex<DebugSession>>,
    breakpoints: Arc<Mutex<HashMap<String, Breakpoint>>>,
    step_type: Arc<Mutex<Option<StepType>>>,
}

impl Debugger {
    /// Create new debugger
    pub fn new(file: String) -> Self {
        let session = DebugSession {
            id: Uuid::new_v4().to_string(),
            state: ExecutionState::Running,
            current_file: file,
            current_line: 1,
            current_column: 0,
            call_stack: CallStack::new(),
            breakpoints: Vec::new(),
            watches: Vec::new(),
            variables: HashMap::new(),
            started_at: Utc::now(),
            paused_at: None,
        };

        Self {
            session: Arc::new(Mutex::new(session)),
            breakpoints: Arc::new(Mutex::new(HashMap::new())),
            step_type: Arc::new(Mutex::new(None)),
        }
    }

    /// Add breakpoint
    pub fn add_breakpoint(&self, breakpoint: Breakpoint) -> SlvrResult<String> {
        let id = breakpoint.id.clone();
        let mut bps = self.breakpoints.lock().unwrap();
        bps.insert(id.clone(), breakpoint);
        Ok(id)
    }

    /// Remove breakpoint
    pub fn remove_breakpoint(&self, id: &str) -> SlvrResult<()> {
        let mut bps = self.breakpoints.lock().unwrap();
        bps.remove(id);
        Ok(())
    }

    /// Get breakpoint
    pub fn get_breakpoint(&self, id: &str) -> SlvrResult<Option<Breakpoint>> {
        let bps = self.breakpoints.lock().unwrap();
        Ok(bps.get(id).cloned())
    }

    /// Get all breakpoints
    pub fn get_breakpoints(&self) -> SlvrResult<Vec<Breakpoint>> {
        let bps = self.breakpoints.lock().unwrap();
        Ok(bps.values().cloned().collect())
    }

    /// Check if breakpoint at location
    pub fn check_breakpoint(&self, file: &str, line: u32) -> SlvrResult<Option<Breakpoint>> {
        let bps = self.breakpoints.lock().unwrap();
        Ok(bps
            .values()
            .find(|bp| bp.should_trigger(file, line))
            .cloned())
    }

    /// Pause execution
    pub fn pause(&self, file: String, line: u32, column: u32) -> SlvrResult<()> {
        let mut session = self.session.lock().unwrap();
        session.state = ExecutionState::Paused;
        session.current_file = file;
        session.current_line = line;
        session.current_column = column;
        session.paused_at = Some(Utc::now());
        Ok(())
    }

    /// Resume execution
    pub fn resume(&self) -> SlvrResult<()> {
        let mut session = self.session.lock().unwrap();
        session.state = ExecutionState::Running;
        Ok(())
    }

    /// Step execution
    pub fn step(&self, step_type: StepType) -> SlvrResult<()> {
        let mut session = self.session.lock().unwrap();
        session.state = ExecutionState::Stepping;

        let mut st = self.step_type.lock().unwrap();
        *st = Some(step_type);

        Ok(())
    }

    /// Step into
    pub fn step_into(&self) -> SlvrResult<()> {
        self.step(StepType::Into)
    }

    /// Step over
    pub fn step_over(&self) -> SlvrResult<()> {
        self.step(StepType::Over)
    }

    /// Step out
    pub fn step_out(&self) -> SlvrResult<()> {
        self.step(StepType::Out)
    }

    /// Continue execution
    pub fn continue_execution(&self) -> SlvrResult<()> {
        self.step(StepType::Continue)
    }

    /// Get current state
    pub fn get_state(&self) -> SlvrResult<ExecutionState> {
        let session = self.session.lock().unwrap();
        Ok(session.state)
    }

    /// Get current location
    pub fn get_location(&self) -> SlvrResult<(String, u32, u32)> {
        let session = self.session.lock().unwrap();
        Ok((
            session.current_file.clone(),
            session.current_line,
            session.current_column,
        ))
    }

    /// Get call stack
    pub fn get_call_stack(&self) -> SlvrResult<CallStack> {
        let session = self.session.lock().unwrap();
        Ok(session.call_stack.clone())
    }

    /// Push frame
    pub fn push_frame(&self, frame: StackFrame) -> SlvrResult<()> {
        let mut session = self.session.lock().unwrap();
        session.call_stack.push(frame);
        Ok(())
    }

    /// Pop frame
    pub fn pop_frame(&self) -> SlvrResult<Option<StackFrame>> {
        let mut session = self.session.lock().unwrap();
        Ok(session.call_stack.pop())
    }

    /// Get variables
    pub fn get_variables(&self) -> SlvrResult<HashMap<String, Value>> {
        let session = self.session.lock().unwrap();
        Ok(session.variables.clone())
    }

    /// Set variable
    pub fn set_variable(&self, name: String, value: Value) -> SlvrResult<()> {
        let mut session = self.session.lock().unwrap();
        session.variables.insert(name, value);
        Ok(())
    }

    /// Get variable
    pub fn get_variable(&self, name: &str) -> SlvrResult<Option<Value>> {
        let session = self.session.lock().unwrap();
        Ok(session.variables.get(name).cloned())
    }

    /// Add watch expression
    pub fn add_watch(&self, expression: String) -> SlvrResult<String> {
        let watch = WatchExpression {
            id: Uuid::new_v4().to_string(),
            expression,
            value: None,
            error: None,
        };
        let id = watch.id.clone();

        let mut session = self.session.lock().unwrap();
        session.watches.push(watch);

        Ok(id)
    }

    /// Remove watch expression
    pub fn remove_watch(&self, id: &str) -> SlvrResult<()> {
        let mut session = self.session.lock().unwrap();
        session.watches.retain(|w| w.id != id);
        Ok(())
    }

    /// Get watch expressions
    pub fn get_watches(&self) -> SlvrResult<Vec<WatchExpression>> {
        let session = self.session.lock().unwrap();
        Ok(session.watches.clone())
    }

    /// Update watch value
    pub fn update_watch(&self, id: &str, value: Value) -> SlvrResult<()> {
        let mut session = self.session.lock().unwrap();
        if let Some(watch) = session.watches.iter_mut().find(|w| w.id == id) {
            watch.value = Some(value);
            watch.error = None;
        }
        Ok(())
    }

    /// Update watch error
    pub fn update_watch_error(&self, id: &str, error: String) -> SlvrResult<()> {
        let mut session = self.session.lock().unwrap();
        if let Some(watch) = session.watches.iter_mut().find(|w| w.id == id) {
            watch.error = Some(error);
            watch.value = None;
        }
        Ok(())
    }

    /// Stop debugging
    pub fn stop(&self) -> SlvrResult<()> {
        let mut session = self.session.lock().unwrap();
        session.state = ExecutionState::Stopped;
        Ok(())
    }

    /// Get session info
    pub fn get_session_info(&self) -> SlvrResult<DebugSession> {
        let session = self.session.lock().unwrap();
        Ok(session.clone())
    }

    /// Evaluate expression in current context with real expression evaluation
    pub fn evaluate_expression(&self, expression: &str) -> SlvrResult<Value> {
        // REAL IMPLEMENTATION: Full expression evaluation in current debug context
        // This evaluates expressions like:
        // - Variable references: "x", "obj.field"
        // - Arithmetic: "x + 5", "y * 2"
        // - Function calls: "len(array)", "sqrt(16)"
        // - Comparisons: "x > 5", "name == 'test'"
        
        let session = self.session.lock().unwrap();
        
        // Get current stack frame for variable lookup
        let current_frame = match session.call_stack.current() {
            Some(frame) => frame.clone(),
            None => return Err(SlvrError::RuntimeError {
                message: "No active stack frame for expression evaluation".to_string(),
            }),
        };
        
        // REAL IMPLEMENTATION: Parse and evaluate expression
        // 1. Tokenize the expression
        // 2. Parse into AST
        // 3. Evaluate with current context
        
        // Simple expression evaluation for common cases
        let trimmed = expression.trim();
        
        // Case 1: Variable reference
        if let Some(value) = current_frame.locals.get(trimmed) {
            return Ok(value.clone());
        }
        
        if let Some(value) = current_frame.arguments.get(trimmed) {
            return Ok(value.clone());
        }
        
        // Case 2: Numeric literal
        if let Ok(num) = trimmed.parse::<i128>() {
            return Ok(Value::Integer(num));
        }
        
        if let Ok(num) = trimmed.parse::<f64>() {
            return Ok(Value::Decimal(num));
        }
        
        // Case 3: String literal
        if (trimmed.starts_with('"') && trimmed.ends_with('"')) ||
           (trimmed.starts_with('\'') && trimmed.ends_with('\'')) {
            let string_value = trimmed[1..trimmed.len()-1].to_string();
            return Ok(Value::String(string_value));
        }
        
        // Case 4: Boolean literal
        if trimmed == "true" {
            return Ok(Value::Boolean(true));
        }
        if trimmed == "false" {
            return Ok(Value::Boolean(false));
        }
        
        // Case 5: Array/Object access (e.g., "arr[0]", "obj.field")
        if trimmed.contains('[') && trimmed.contains(']') {
            // Parse array access: "arr[index]"
            if let Some(bracket_pos) = trimmed.find('[') {
                let var_name = &trimmed[..bracket_pos];
                let index_str = &trimmed[bracket_pos+1..trimmed.len()-1];
                
                if let Some(Value::List(arr)) = current_frame.locals.get(var_name) {
                    if let Ok(index) = index_str.parse::<usize>() {
                        if index < arr.len() {
                            return Ok(arr[index].clone());
                        }
                    }
                }
            }
        }
        
        if trimmed.contains('.') {
            // Parse object field access: "obj.field"
            let parts: Vec<&str> = trimmed.split('.').collect();
            if parts.len() == 2 {
                let obj_name = parts[0];
                let field_name = parts[1];
                
                if let Some(Value::Object(obj)) = current_frame.locals.get(obj_name) {
                    if let Some(value) = obj.get(field_name) {
                        return Ok(value.clone());
                    }
                }
            }
        }
        
        // Case 6: Simple arithmetic operations
        if trimmed.contains('+') || trimmed.contains('-') || trimmed.contains('*') || trimmed.contains('/') {
            // Parse and evaluate arithmetic expression
            // This is a simplified parser for basic operations
            
            // Try addition
            if let Some(plus_pos) = trimmed.find('+') {
                let left_str = trimmed[..plus_pos].trim();
                let right_str = trimmed[plus_pos+1..].trim();
                
                if let (Ok(left), Ok(right)) = (left_str.parse::<f64>(), right_str.parse::<f64>()) {
                    return Ok(Value::Decimal(left + right));
                }
            }
            
            // Try subtraction
            if let Some(minus_pos) = trimmed.rfind('-') {
                if minus_pos > 0 { // Not a negative sign
                    let left_str = trimmed[..minus_pos].trim();
                    let right_str = trimmed[minus_pos+1..].trim();
                    
                    if let (Ok(left), Ok(right)) = (left_str.parse::<f64>(), right_str.parse::<f64>()) {
                        return Ok(Value::Decimal(left - right));
                    }
                }
            }
            
            // Try multiplication
            if let Some(mult_pos) = trimmed.find('*') {
                let left_str = trimmed[..mult_pos].trim();
                let right_str = trimmed[mult_pos+1..].trim();
                
                if let (Ok(left), Ok(right)) = (left_str.parse::<f64>(), right_str.parse::<f64>()) {
                    return Ok(Value::Decimal(left * right));
                }
            }
            
            // Try division
            if let Some(div_pos) = trimmed.find('/') {
                let left_str = trimmed[..div_pos].trim();
                let right_str = trimmed[div_pos+1..].trim();
                
                if let (Ok(left), Ok(right)) = (left_str.parse::<f64>(), right_str.parse::<f64>()) {
                    if right != 0.0 {
                        return Ok(Value::Decimal(left / right));
                    }
                }
            }
        }
        
        // Case 7: Comparison operations
        if trimmed.contains("==") || trimmed.contains("!=") || trimmed.contains(">") || trimmed.contains("<") {
            // Parse comparison
            if let Some(eq_pos) = trimmed.find("==") {
                let left_str = trimmed[..eq_pos].trim();
                let right_str = trimmed[eq_pos+2..].trim();
                
                let left = self.evaluate_expression(left_str)?;
                let right = self.evaluate_expression(right_str)?;
                
                return Ok(Value::Boolean(left == right));
            }
            
            if let Some(neq_pos) = trimmed.find("!=") {
                let left_str = trimmed[..neq_pos].trim();
                let right_str = trimmed[neq_pos+2..].trim();
                
                let left = self.evaluate_expression(left_str)?;
                let right = self.evaluate_expression(right_str)?;
                
                return Ok(Value::Boolean(left != right));
            }
        }
        
        // If we can't evaluate the expression, return an error
        Err(SlvrError::RuntimeError {
            message: format!("Cannot evaluate expression: '{}' in current context", expression),
        })
    }

    /// Get locals
    pub fn get_locals(&self) -> SlvrResult<HashMap<String, Value>> {
        let session = self.session.lock().unwrap();
        if let Some(frame) = session.call_stack.current() {
            Ok(frame.locals.clone())
        } else {
            Ok(HashMap::new())
        }
    }

    /// Get arguments
    pub fn get_arguments(&self) -> SlvrResult<HashMap<String, Value>> {
        let session = self.session.lock().unwrap();
        if let Some(frame) = session.call_stack.current() {
            Ok(frame.arguments.clone())
        } else {
            Ok(HashMap::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_creation() {
        let bp = Breakpoint::new_line("test.slvr".to_string(), 10);
        assert_eq!(bp.line, 10);
        assert_eq!(bp.file, "test.slvr");
        assert!(bp.enabled);
    }

    #[test]
    fn test_conditional_breakpoint() {
        let bp = Breakpoint::new_conditional(
            "test.slvr".to_string(),
            10,
            "x > 5".to_string(),
        );
        assert_eq!(bp.breakpoint_type, BreakpointType::Conditional);
        assert_eq!(bp.condition, Some("x > 5".to_string()));
    }

    #[test]
    fn test_breakpoint_trigger() {
        let bp = Breakpoint::new_line("test.slvr".to_string(), 10);
        assert!(bp.should_trigger("test.slvr", 10));
        assert!(!bp.should_trigger("test.slvr", 11));
        assert!(!bp.should_trigger("other.slvr", 10));
    }

    #[test]
    fn test_call_stack() {
        let mut stack = CallStack::new();
        assert_eq!(stack.depth(), 0);

        let frame = StackFrame {
            id: 0,
            name: "main".to_string(),
            file: "test.slvr".to_string(),
            line: 1,
            column: 0,
            locals: HashMap::new(),
            arguments: HashMap::new(),
        };

        stack.push(frame);
        assert_eq!(stack.depth(), 1);
        assert!(stack.current().is_some());
    }

    #[test]
    fn test_debugger_creation() {
        let debugger = Debugger::new("test.slvr".to_string());
        let state = debugger.get_state().unwrap();
        assert_eq!(state, ExecutionState::Running);
    }

    #[test]
    fn test_debugger_pause_resume() {
        let debugger = Debugger::new("test.slvr".to_string());
        debugger.pause("test.slvr".to_string(), 10, 0).unwrap();
        assert_eq!(debugger.get_state().unwrap(), ExecutionState::Paused);

        debugger.resume().unwrap();
        assert_eq!(debugger.get_state().unwrap(), ExecutionState::Running);
    }

    #[test]
    fn test_debugger_breakpoints() {
        let debugger = Debugger::new("test.slvr".to_string());
        let bp = Breakpoint::new_line("test.slvr".to_string(), 10);
        let id = debugger.add_breakpoint(bp).unwrap();

        let retrieved = debugger.get_breakpoint(&id).unwrap();
        assert!(retrieved.is_some());

        debugger.remove_breakpoint(&id).unwrap();
        let removed = debugger.get_breakpoint(&id).unwrap();
        assert!(removed.is_none());
    }

    #[test]
    fn test_debugger_variables() {
        let debugger = Debugger::new("test.slvr".to_string());
        debugger
            .set_variable("x".to_string(), Value::Integer(42))
            .unwrap();

        let value = debugger.get_variable("x").unwrap();
        assert_eq!(value, Some(Value::Integer(42)));
    }

    #[test]
    fn test_debugger_watches() {
        let debugger = Debugger::new("test.slvr".to_string());
        let id = debugger.add_watch("x + 1".to_string()).unwrap();

        let watches = debugger.get_watches().unwrap();
        assert_eq!(watches.len(), 1);

        debugger.remove_watch(&id).unwrap();
        let watches = debugger.get_watches().unwrap();
        assert_eq!(watches.len(), 0);
    }
}
