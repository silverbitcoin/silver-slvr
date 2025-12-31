//! Runtime values for the Slvr language
//!
//! Represents values that can be computed and stored during execution.

use crate::error::{SlvrError, SlvrResult};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A runtime value in the Slvr language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    /// Integer value (arbitrary precision)
    Integer(i128),
    /// Decimal value (fixed-point)
    Decimal(f64),
    /// String value
    String(String),
    /// Boolean value
    Boolean(bool),
    /// List value
    List(Vec<Value>),
    /// Object/map value
    Object(HashMap<String, Value>),
    /// Unit value
    Unit,
    /// Null value
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::Decimal(d) => write!(f, "{}", d),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Object(map) => {
                write!(f, "{{")?;
                let mut items: Vec<_> = map.iter().collect();
                items.sort_by_key(|(k, _)| k.as_str());
                for (i, (k, v)) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Unit => write!(f, "()"),
            Value::Null => write!(f, "null"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Integer(a), Value::Integer(b)) => a == b,
            (Value::Decimal(a), Value::Decimal(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }
}

impl Value {
    /// Check if this value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            Value::Unit => false,
            Value::Integer(n) => *n != 0,
            Value::Decimal(d) => *d != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Object(o) => !o.is_empty(),
        }
    }

    /// Convert to integer
    pub fn to_integer(&self) -> SlvrResult<i128> {
        match self {
            Value::Integer(n) => Ok(*n),
            Value::Decimal(d) => Ok(*d as i128),
            Value::Boolean(b) => Ok(if *b { 1 } else { 0 }),
            Value::String(s) => s
                .parse::<i128>()
                .map_err(|_| SlvrError::type_mismatch("integer", "string")),
            _ => Err(SlvrError::type_mismatch("integer", self.type_name())),
        }
    }

    /// Convert to decimal
    pub fn to_decimal(&self) -> SlvrResult<f64> {
        match self {
            Value::Integer(n) => Ok(*n as f64),
            Value::Decimal(d) => Ok(*d),
            Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            Value::String(s) => s
                .parse::<f64>()
                .map_err(|_| SlvrError::type_mismatch("decimal", "string")),
            _ => Err(SlvrError::type_mismatch("decimal", self.type_name())),
        }
    }

    /// Convert to string
    pub fn to_string_value(&self) -> SlvrResult<String> {
        match self {
            Value::String(s) => Ok(s.clone()),
            _ => Ok(self.to_string()),
        }
    }

    /// Convert to boolean
    pub fn to_boolean(&self) -> SlvrResult<bool> {
        match self {
            Value::Boolean(b) => Ok(*b),
            Value::Integer(n) => Ok(*n != 0),
            Value::Decimal(d) => Ok(*d != 0.0),
            Value::String(s) => Ok(!s.is_empty()),
            Value::Null => Ok(false),
            Value::Unit => Ok(false),
            _ => Err(SlvrError::type_mismatch("boolean", self.type_name())),
        }
    }

    /// Get the type name as a string
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "integer",
            Value::Decimal(_) => "decimal",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::List(_) => "list",
            Value::Object(_) => "object",
            Value::Unit => "unit",
            Value::Null => "null",
        }
    }

    /// Get the length of a collection
    pub fn len(&self) -> SlvrResult<usize> {
        match self {
            Value::String(s) => Ok(s.len()),
            Value::List(l) => Ok(l.len()),
            Value::Object(o) => Ok(o.len()),
            _ => Err(SlvrError::invalid_arg(format!(
                "Cannot get length of {}",
                self.type_name()
            ))),
        }
    }

    /// Check if a collection is empty
    pub fn is_empty(&self) -> SlvrResult<bool> {
        Ok(self.len()? == 0)
    }

    /// Get an element from a list by index
    pub fn get_list_element(&self, index: usize) -> SlvrResult<Value> {
        match self {
            Value::List(l) => l.get(index).cloned().ok_or(SlvrError::IndexOutOfBounds {
                index: index as i64,
                length: l.len(),
            }),
            _ => Err(SlvrError::invalid_arg(format!(
                "Cannot index {}",
                self.type_name()
            ))),
        }
    }

    /// Get a field from an object
    pub fn get_field(&self, key: &str) -> SlvrResult<Value> {
        match self {
            Value::Object(o) => o.get(key).cloned().ok_or_else(|| SlvrError::KeyNotFound {
                key: key.to_string(),
            }),
            _ => Err(SlvrError::invalid_arg(format!(
                "Cannot access field on {}",
                self.type_name()
            ))),
        }
    }

    /// Set a field in an object
    pub fn set_field(&mut self, key: String, value: Value) -> SlvrResult<()> {
        match self {
            Value::Object(o) => {
                o.insert(key, value);
                Ok(())
            }
            _ => Err(SlvrError::invalid_arg(format!(
                "Cannot set field on {}",
                self.type_name()
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_display() {
        assert_eq!(Value::Integer(42).to_string(), "42");
        assert_eq!(Value::String("hello".to_string()).to_string(), "\"hello\"");
        assert_eq!(Value::Boolean(true).to_string(), "true");
    }

    #[test]
    fn test_value_truthiness() {
        assert!(Value::Boolean(true).is_truthy());
        assert!(!Value::Boolean(false).is_truthy());
        assert!(Value::Integer(1).is_truthy());
        assert!(!Value::Integer(0).is_truthy());
        assert!(!Value::Null.is_truthy());
    }

    #[test]
    fn test_value_conversions() {
        assert_eq!(Value::Integer(42).to_integer().unwrap(), 42);
        let test_decimal = 2.71_f64; // Use e instead of pi
        assert_eq!(
            Value::Decimal(test_decimal).to_decimal().unwrap(),
            test_decimal
        );
        assert_eq!(
            Value::String("hello".to_string())
                .to_string_value()
                .unwrap(),
            "hello"
        );
    }

    #[test]
    fn test_value_equality() {
        assert_eq!(Value::Integer(42), Value::Integer(42));
        assert_ne!(Value::Integer(42), Value::Integer(43));
        assert_eq!(
            Value::String("hello".to_string()),
            Value::String("hello".to_string())
        );
    }

    #[test]
    fn test_list_operations() {
        let list = Value::List(vec![
            Value::Integer(1),
            Value::Integer(2),
            Value::Integer(3),
        ]);

        assert_eq!(list.len().unwrap(), 3);
        assert_eq!(list.get_list_element(0).unwrap(), Value::Integer(1));
        assert!(list.get_list_element(10).is_err());
    }

    #[test]
    fn test_object_operations() {
        let mut obj = Value::Object(HashMap::new());
        obj.set_field("name".to_string(), Value::String("Alice".to_string()))
            .unwrap();

        assert_eq!(
            obj.get_field("name").unwrap(),
            Value::String("Alice".to_string())
        );
        assert!(obj.get_field("age").is_err());
    }
}
