//! Abstract Syntax Tree (AST) for the Slvr language
//!
//! Represents the structure of Slvr programs after parsing.

use serde::{Deserialize, Serialize};

/// A complete Slvr program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    /// Top-level definitions
    pub definitions: Vec<Definition>,
}

/// Top-level definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Definition {
    /// Module definition
    Module {
        name: String,
        doc: Option<String>,
        body: Vec<Definition>,
    },
    /// Function definition
    Function {
        name: String,
        params: Vec<(String, Type)>,
        return_type: Type,
        doc: Option<String>,
        body: Expr,
    },
    /// Schema definition
    Schema {
        name: String,
        fields: Vec<(String, Type)>,
        doc: Option<String>,
    },
    /// Table definition
    Table {
        name: String,
        schema: String,
        doc: Option<String>,
    },
    /// Constant definition
    Constant {
        name: String,
        ty: Type,
        value: Expr,
    },
}

/// Type annotations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    /// Integer type
    Integer,
    /// Decimal type
    Decimal,
    /// String type
    String,
    /// Boolean type
    Boolean,
    /// List type
    List(Box<Type>),
    /// Object type
    Object,
    /// Custom type
    Custom(String),
    /// Unit type
    Unit,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Integer => write!(f, "integer"),
            Type::Decimal => write!(f, "decimal"),
            Type::String => write!(f, "string"),
            Type::Boolean => write!(f, "boolean"),
            Type::List(inner) => write!(f, "[{}]", inner),
            Type::Object => write!(f, "object"),
            Type::Custom(name) => write!(f, "{}", name),
            Type::Unit => write!(f, "unit"),
        }
    }
}

/// Expressions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    /// Literal value
    Literal(Literal),
    /// Variable reference
    Variable(String),
    /// Function call
    Call {
        function: Box<Expr>,
        args: Vec<Expr>,
    },
    /// Binary operation
    BinOp {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// Unary operation
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    /// If expression
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    /// Let binding
    Let {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    /// List literal
    List(Vec<Expr>),
    /// Object literal
    Object(Vec<(String, Expr)>),
    /// Field access
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    /// Index access
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    /// Block expression
    Block(Vec<Expr>),
    /// Database read
    Read {
        table: String,
        key: Box<Expr>,
    },
    /// Database write
    Write {
        table: String,
        key: Box<Expr>,
        value: Box<Expr>,
    },
    /// Database update
    Update {
        table: String,
        key: Box<Expr>,
        updates: Vec<(String, Expr)>,
    },
    /// Database delete
    Delete {
        table: String,
        key: Box<Expr>,
    },
}

/// Literal values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    /// Integer literal
    Integer(i128),
    /// Decimal literal
    Decimal(f64),
    /// String literal
    String(String),
    /// Boolean literal
    Boolean(bool),
    /// Unit literal
    Unit,
    /// Null literal
    Null,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinOp {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,

    // Comparison
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    // Logical
    And,
    Or,

    // String
    Concat,
}

impl std::fmt::Display for BinOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinOp::Add => write!(f, "+"),
            BinOp::Subtract => write!(f, "-"),
            BinOp::Multiply => write!(f, "*"),
            BinOp::Divide => write!(f, "/"),
            BinOp::Modulo => write!(f, "%"),
            BinOp::Power => write!(f, "^"),
            BinOp::Equal => write!(f, "=="),
            BinOp::NotEqual => write!(f, "!="),
            BinOp::Less => write!(f, "<"),
            BinOp::LessEqual => write!(f, "<="),
            BinOp::Greater => write!(f, ">"),
            BinOp::GreaterEqual => write!(f, ">="),
            BinOp::And => write!(f, "&&"),
            BinOp::Or => write!(f, "||"),
            BinOp::Concat => write!(f, "++"),
        }
    }
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOp {
    /// Logical negation
    Not,
    /// Arithmetic negation
    Negate,
}

impl std::fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Not => write!(f, "!"),
            UnaryOp::Negate => write!(f, "-"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_display() {
        assert_eq!(Type::Integer.to_string(), "integer");
        assert_eq!(Type::String.to_string(), "string");
        assert_eq!(Type::List(Box::new(Type::Integer)).to_string(), "[integer]");
    }

    #[test]
    fn test_binop_display() {
        assert_eq!(BinOp::Add.to_string(), "+");
        assert_eq!(BinOp::Equal.to_string(), "==");
        assert_eq!(BinOp::And.to_string(), "&&");
    }

    #[test]
    fn test_unaryop_display() {
        assert_eq!(UnaryOp::Not.to_string(), "!");
        assert_eq!(UnaryOp::Negate.to_string(), "-");
    }
}
