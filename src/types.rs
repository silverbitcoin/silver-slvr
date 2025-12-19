//! Type system for the Slvr language
//!
//! Provides type definitions, type checking, and type inference capabilities.
//! The type system is designed to catch errors at compile time and ensure
//! safe execution on the blockchain.

use crate::error::{SlvrError, SlvrResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Represents a type in the Slvr language
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    /// Integer type (arbitrary precision)
    Integer,
    /// Decimal/fixed-point type
    Decimal,
    /// String type
    String,
    /// Boolean type
    Boolean,
    /// List type with element type
    List(Box<Type>),
    /// Object/map type with field types
    Object(HashMap<String, Type>),
    /// Function type: (arg_types) -> return_type
    Function(Vec<Type>, Box<Type>),
    /// Unit type (void)
    Unit,
    /// Any type (for dynamic typing)
    Any,
    /// Custom user-defined type
    Custom(String),
    /// Table type for database operations
    Table(Box<Type>),
    /// Schema type for table definitions
    Schema(HashMap<String, Type>),
}

impl std::hash::Hash for Type {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Type::Integer => 0.hash(state),
            Type::Decimal => 1.hash(state),
            Type::String => 2.hash(state),
            Type::Boolean => 3.hash(state),
            Type::List(t) => {
                4.hash(state);
                t.hash(state);
            }
            Type::Object(_) => 5.hash(state),
            Type::Function(_, _) => 6.hash(state),
            Type::Unit => 7.hash(state),
            Type::Any => 8.hash(state),
            Type::Custom(s) => {
                9.hash(state);
                s.hash(state);
            }
            Type::Table(t) => {
                10.hash(state);
                t.hash(state);
            }
            Type::Schema(_) => 11.hash(state),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Integer => write!(f, "integer"),
            Type::Decimal => write!(f, "decimal"),
            Type::String => write!(f, "string"),
            Type::Boolean => write!(f, "boolean"),
            Type::List(inner) => write!(f, "[{}]", inner),
            Type::Object(_) => write!(f, "object"),
            Type::Function(args, ret) => {
                write!(f, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ") -> {}", ret)
            }
            Type::Unit => write!(f, "unit"),
            Type::Any => write!(f, "any"),
            Type::Custom(name) => write!(f, "{}", name),
            Type::Table(inner) => write!(f, "table<{}>", inner),
            Type::Schema(_) => write!(f, "schema"),
        }
    }
}

impl Type {
    /// Check if this type is compatible with another type
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Any, _) | (_, Type::Any) => true,
            (a, b) => a == b,
        }
    }

    /// Get the default value for this type
    pub fn default_value(&self) -> String {
        match self {
            Type::Integer => "0".to_string(),
            Type::Decimal => "0.0".to_string(),
            Type::String => "\"\"".to_string(),
            Type::Boolean => "false".to_string(),
            Type::List(_) => "[]".to_string(),
            Type::Unit => "()".to_string(),
            Type::Object(_) => "{}".to_string(),
            _ => "null".to_string(),
        }
    }

    /// Check if this type is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Integer | Type::Decimal)
    }

    /// Check if this type is comparable
    pub fn is_comparable(&self) -> bool {
        matches!(
            self,
            Type::Integer | Type::Decimal | Type::String | Type::Boolean
        )
    }

    /// Check if this type is a collection
    pub fn is_collection(&self) -> bool {
        matches!(self, Type::List(_) | Type::Object(_))
    }
}

/// Type environment for tracking variable and function types
#[derive(Debug, Clone)]
pub struct TypeEnv {
    /// Stack of scopes for variable types
    var_scopes: Vec<HashMap<String, Type>>,
    /// Function signatures
    functions: HashMap<String, (Vec<Type>, Type)>,
    /// Custom type definitions
    custom_types: HashMap<String, Type>,
    /// Table definitions
    tables: HashMap<String, Type>,
}

impl TypeEnv {
    /// Create a new type environment
    pub fn new() -> Self {
        Self {
            var_scopes: vec![HashMap::new()],
            functions: HashMap::new(),
            custom_types: HashMap::new(),
            tables: HashMap::new(),
        }
    }

    /// Push a new scope
    pub fn push_scope(&mut self) {
        self.var_scopes.push(HashMap::new());
    }

    /// Pop the current scope
    pub fn pop_scope(&mut self) -> SlvrResult<()> {
        if self.var_scopes.len() > 1 {
            self.var_scopes.pop();
            Ok(())
        } else {
            Err(SlvrError::internal("Cannot pop global scope"))
        }
    }

    /// Define a variable in the current scope
    pub fn define_var(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.var_scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    /// Look up a variable's type
    pub fn lookup_var(&self, name: &str) -> Option<Type> {
        for scope in self.var_scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    /// Check if a variable is defined
    pub fn is_var_defined(&self, name: &str) -> bool {
        self.lookup_var(name).is_some()
    }

    /// Define a function
    pub fn define_function(&mut self, name: String, args: Vec<Type>, ret: Type) {
        self.functions.insert(name, (args, ret));
    }

    /// Look up a function signature
    pub fn lookup_function(&self, name: &str) -> Option<(Vec<Type>, Type)> {
        self.functions.get(name).cloned()
    }

    /// Define a custom type
    pub fn define_custom_type(&mut self, name: String, ty: Type) {
        self.custom_types.insert(name, ty);
    }

    /// Look up a custom type
    pub fn lookup_custom_type(&self, name: &str) -> Option<Type> {
        self.custom_types.get(name).cloned()
    }

    /// Define a table
    pub fn define_table(&mut self, name: String, schema: Type) {
        self.tables.insert(name, schema);
    }

    /// Look up a table definition
    pub fn lookup_table(&self, name: &str) -> Option<Type> {
        self.tables.get(name).cloned()
    }

    /// Get all defined functions
    pub fn functions(&self) -> &HashMap<String, (Vec<Type>, Type)> {
        &self.functions
    }

    /// Get all defined custom types
    pub fn custom_types(&self) -> &HashMap<String, Type> {
        &self.custom_types
    }

    /// Get all defined tables
    pub fn tables(&self) -> &HashMap<String, Type> {
        &self.tables
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
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
    fn test_type_compatibility() {
        assert!(Type::Integer.is_compatible_with(&Type::Integer));
        assert!(Type::Any.is_compatible_with(&Type::Integer));
        assert!(!Type::Integer.is_compatible_with(&Type::String));
    }

    #[test]
    fn test_type_properties() {
        assert!(Type::Integer.is_numeric());
        assert!(Type::Decimal.is_numeric());
        assert!(!Type::String.is_numeric());

        assert!(Type::Integer.is_comparable());
        assert!(Type::String.is_comparable());
        assert!(!Type::List(Box::new(Type::Integer)).is_comparable());

        assert!(Type::List(Box::new(Type::Integer)).is_collection());
        assert!(Type::Object(HashMap::new()).is_collection());
    }

    #[test]
    fn test_type_env() {
        let mut env = TypeEnv::new();
        env.define_var("x".to_string(), Type::Integer);
        assert_eq!(env.lookup_var("x"), Some(Type::Integer));
        assert_eq!(env.lookup_var("y"), None);
    }

    #[test]
    fn test_type_env_scopes() {
        let mut env = TypeEnv::new();
        env.define_var("x".to_string(), Type::Integer);

        env.push_scope();
        env.define_var("y".to_string(), Type::String);

        assert_eq!(env.lookup_var("x"), Some(Type::Integer));
        assert_eq!(env.lookup_var("y"), Some(Type::String));

        env.pop_scope().unwrap();
        assert_eq!(env.lookup_var("x"), Some(Type::Integer));
        assert_eq!(env.lookup_var("y"), None);
    }

    #[test]
    fn test_function_definitions() {
        let mut env = TypeEnv::new();
        env.define_function(
            "add".to_string(),
            vec![Type::Integer, Type::Integer],
            Type::Integer,
        );

        let sig = env.lookup_function("add");
        assert!(sig.is_some());
        let (args, ret) = sig.unwrap();
        assert_eq!(args.len(), 2);
        assert_eq!(ret, Type::Integer);
    }
}
