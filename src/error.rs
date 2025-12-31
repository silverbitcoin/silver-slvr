//! Error types for the Slvr language

use thiserror::Error;

/// Result type for Slvr operations
pub type SlvrResult<T> = Result<T, SlvrError>;

/// Errors that can occur during Slvr compilation and execution
#[derive(Debug, Clone, Error)]
pub enum SlvrError {
    /// Lexical analysis error
    #[error("Lexer error at line {line}, column {column}: {message}")]
    LexerError {
        line: usize,
        column: usize,
        message: String,
    },

    /// Parsing error
    #[error("Parse error at line {line}, column {column}: {message}")]
    ParseError {
        line: usize,
        column: usize,
        message: String,
    },

    /// Type checking error
    #[error("Type error: {message}")]
    TypeError { message: String },

    /// Runtime error
    #[error("Runtime error: {message}")]
    RuntimeError { message: String },

    /// Execution exceeded fuel limit
    #[error("Execution exceeded fuel limit: used {used}, limit {limit}")]
    FuelExceeded { used: u64, limit: u64 },

    /// Recursion depth exceeded
    #[error("Recursion depth exceeded: {depth}")]
    RecursionDepthExceeded { depth: usize },

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Index out of bounds
    #[error("Index {index} out of bounds for sequence of length {length}")]
    IndexOutOfBounds { index: i64, length: usize },

    /// Key not found in map
    #[error("Key not found: {key}")]
    KeyNotFound { key: String },

    /// Variable not defined
    #[error("Variable not defined: {name}")]
    UndefinedVariable { name: String },

    /// Function not defined
    #[error("Function not defined: {name}")]
    UndefinedFunction { name: String },

    /// Module not found
    #[error("Module not found: {name}")]
    ModuleNotFound { name: String },

    /// Invalid argument
    #[error("Invalid argument: {message}")]
    InvalidArgument { message: String },

    /// Type mismatch
    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    /// Compilation error
    #[error("Compilation error: {message}")]
    CompilationError { message: String },

    /// IO error
    #[error("IO error: {message}")]
    IoError { message: String },

    /// Internal error
    #[error("Internal error: {message}")]
    InternalError { message: String },

    /// Lock error (mutex poisoning)
    #[error("Lock error: {0}")]
    LockError(String),
}

impl SlvrError {
    /// Create a lexer error
    pub fn lexer(line: usize, column: usize, message: impl Into<String>) -> Self {
        SlvrError::LexerError {
            line,
            column,
            message: message.into(),
        }
    }

    /// Create a parse error
    pub fn parse(line: usize, column: usize, message: impl Into<String>) -> Self {
        SlvrError::ParseError {
            line,
            column,
            message: message.into(),
        }
    }

    /// Create a type error
    pub fn type_error(message: impl Into<String>) -> Self {
        SlvrError::TypeError {
            message: message.into(),
        }
    }

    /// Create a runtime error
    pub fn runtime(message: impl Into<String>) -> Self {
        SlvrError::RuntimeError {
            message: message.into(),
        }
    }

    /// Create an undefined variable error
    pub fn undefined_var(name: impl Into<String>) -> Self {
        SlvrError::UndefinedVariable { name: name.into() }
    }

    /// Create an undefined function error
    pub fn undefined_func(name: impl Into<String>) -> Self {
        SlvrError::UndefinedFunction { name: name.into() }
    }

    /// Create a type mismatch error
    pub fn type_mismatch(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        SlvrError::TypeMismatch {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create an invalid argument error
    pub fn invalid_arg(message: impl Into<String>) -> Self {
        SlvrError::InvalidArgument {
            message: message.into(),
        }
    }

    /// Create a compilation error
    pub fn compilation(message: impl Into<String>) -> Self {
        SlvrError::CompilationError {
            message: message.into(),
        }
    }

    /// Create an IO error
    pub fn io(message: impl Into<String>) -> Self {
        SlvrError::IoError {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        SlvrError::InternalError {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = SlvrError::lexer(1, 5, "unexpected token");
        assert!(err.to_string().contains("Lexer error"));

        let err = SlvrError::type_error("type mismatch");
        assert!(err.to_string().contains("Type error"));

        let err = SlvrError::undefined_var("x");
        assert!(err.to_string().contains("Variable not defined"));
    }

    #[test]
    fn test_result_type() {
        let result: SlvrResult<i64> = Err(SlvrError::runtime("test error"));
        assert!(result.is_err());

        let result: SlvrResult<i64> = Ok(42);
        assert!(result.is_ok());
    }
}
