//! Standard Library - 100+ Built-in Functions for Slvr
//! 
//! This module provides comprehensive built-in functions for string manipulation,
//! mathematical operations, cryptographic functions, list operations, and more.

use crate::value::Value;
use crate::error::{SlvrError, SlvrResult};
use std::collections::HashMap;
use sha2::{Sha256, Sha512, Digest};
use blake3;

/// String manipulation functions
pub mod string {
    use super::*;

    pub fn concat(args: Vec<Value>) -> SlvrResult<Value> {
        let mut result = String::new();
        for arg in args {
            match arg {
                Value::String(s) => result.push_str(&s),
                Value::Integer(i) => result.push_str(&i.to_string()),
                Value::Decimal(d) => result.push_str(&d.to_string()),
                Value::Boolean(b) => result.push_str(if b { "true" } else { "false" }),
                _ => return Err(SlvrError::TypeError {
                    message: "concat requires string-convertible values".to_string(),
                }),
            }
        }
        Ok(Value::String(result))
    }

    pub fn length(s: Value) -> SlvrResult<Value> {
        match s {
            Value::String(s) => Ok(Value::Integer(s.len() as i128)),
            _ => Err(SlvrError::TypeError {
                message: "length requires a string".to_string(),
            }),
        }
    }

    pub fn substring(s: Value, start: Value, end: Value) -> SlvrResult<Value> {
        let string = match s {
            Value::String(s) => s,
            _ => return Err(SlvrError::TypeError {
                message: "substring requires a string".to_string(),
            }),
        };

        let start_idx = match start {
            Value::Integer(i) => i as usize,
            _ => return Err(SlvrError::TypeError {
                message: "substring start must be an integer".to_string(),
            }),
        };

        let end_idx = match end {
            Value::Integer(i) => i as usize,
            _ => return Err(SlvrError::TypeError {
                message: "substring end must be an integer".to_string(),
            }),
        };

        if start_idx > string.len() || end_idx > string.len() || start_idx > end_idx {
            return Err(SlvrError::IndexOutOfBounds {
                index: start_idx as i64,
                length: string.len(),
            });
        }

        Ok(Value::String(string[start_idx..end_idx].to_string()))
    }

    pub fn to_upper(s: Value) -> SlvrResult<Value> {
        match s {
            Value::String(s) => Ok(Value::String(s.to_uppercase())),
            _ => Err(SlvrError::TypeError {
                message: "to-upper requires a string".to_string(),
            }),
        }
    }

    pub fn to_lower(s: Value) -> SlvrResult<Value> {
        match s {
            Value::String(s) => Ok(Value::String(s.to_lowercase())),
            _ => Err(SlvrError::TypeError {
                message: "to-lower requires a string".to_string(),
            }),
        }
    }

    pub fn trim(s: Value) -> SlvrResult<Value> {
        match s {
            Value::String(s) => Ok(Value::String(s.trim().to_string())),
            _ => Err(SlvrError::TypeError {
                message: "trim requires a string".to_string(),
            }),
        }
    }

    pub fn split(s: Value, delimiter: Value) -> SlvrResult<Value> {
        let string = match s {
            Value::String(s) => s,
            _ => return Err(SlvrError::TypeError {
                message: "split requires a string".to_string(),
            }),
        };

        let delim = match delimiter {
            Value::String(d) => d,
            _ => return Err(SlvrError::TypeError {
                message: "split delimiter must be a string".to_string(),
            }),
        };

        let parts: Vec<Value> = string
            .split(&delim)
            .map(|s| Value::String(s.to_string()))
            .collect();

        Ok(Value::List(parts))
    }

    pub fn contains(s: Value, substring: Value) -> SlvrResult<Value> {
        let string = match s {
            Value::String(s) => s,
            _ => return Err(SlvrError::TypeError {
                message: "contains requires a string".to_string(),
            }),
        };

        let substr = match substring {
            Value::String(s) => s,
            _ => return Err(SlvrError::TypeError {
                message: "contains substring must be a string".to_string(),
            }),
        };

        Ok(Value::Boolean(string.contains(&substr)))
    }

    pub fn format(template: Value, args: Vec<Value>) -> SlvrResult<Value> {
        let template_str = match template {
            Value::String(s) => s,
            _ => return Err(SlvrError::TypeError {
                message: "format requires a string template".to_string(),
            }),
        };

        let mut result = template_str.clone();
        for (i, arg) in args.iter().enumerate() {
            let placeholder = format!("{{{}}}", i);
            let replacement = match arg {
                Value::String(s) => s.clone(),
                Value::Integer(i) => i.to_string(),
                Value::Decimal(d) => d.to_string(),
                Value::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
                _ => format!("{:?}", arg),
            };
            result = result.replace(&placeholder, &replacement);
        }

        Ok(Value::String(result))
    }
}

/// Mathematical functions
pub mod math {
    use super::*;

    pub fn abs(n: Value) -> SlvrResult<Value> {
        match n {
            Value::Integer(i) => Ok(Value::Integer(i.abs())),
            Value::Decimal(d) => Ok(Value::Decimal(d.abs())),
            _ => Err(SlvrError::TypeError {
                message: "abs requires a number".to_string(),
            }),
        }
    }

    pub fn min(a: Value, b: Value) -> SlvrResult<Value> {
        match (a, b) {
            (Value::Integer(x), Value::Integer(y)) => Ok(Value::Integer(x.min(y))),
            (Value::Decimal(x), Value::Decimal(y)) => Ok(Value::Decimal(x.min(y))),
            (Value::Integer(x), Value::Decimal(y)) => {
                Ok(Value::Decimal((x as f64).min(y)))
            }
            (Value::Decimal(x), Value::Integer(y)) => {
                Ok(Value::Decimal(x.min(y as f64)))
            }
            _ => Err(SlvrError::TypeError {
                message: "min requires numbers".to_string(),
            }),
        }
    }

    pub fn max(a: Value, b: Value) -> SlvrResult<Value> {
        match (a, b) {
            (Value::Integer(x), Value::Integer(y)) => Ok(Value::Integer(x.max(y))),
            (Value::Decimal(x), Value::Decimal(y)) => Ok(Value::Decimal(x.max(y))),
            (Value::Integer(x), Value::Decimal(y)) => {
                Ok(Value::Decimal((x as f64).max(y)))
            }
            (Value::Decimal(x), Value::Integer(y)) => {
                Ok(Value::Decimal(x.max(y as f64)))
            }
            _ => Err(SlvrError::TypeError {
                message: "max requires numbers".to_string(),
            }),
        }
    }

    pub fn sqrt(n: Value) -> SlvrResult<Value> {
        match n {
            Value::Integer(i) => {
                if i < 0 {
                    return Err(SlvrError::RuntimeError {
                        message: "sqrt of negative number".to_string(),
                    });
                }
                Ok(Value::Decimal((i as f64).sqrt()))
            }
            Value::Decimal(d) => {
                if d < 0.0 {
                    return Err(SlvrError::RuntimeError {
                        message: "sqrt of negative number".to_string(),
                    });
                }
                Ok(Value::Decimal(d.sqrt()))
            }
            _ => Err(SlvrError::TypeError {
                message: "sqrt requires a number".to_string(),
            }),
        }
    }

    pub fn ln(n: Value) -> SlvrResult<Value> {
        match n {
            Value::Integer(i) => {
                if i <= 0 {
                    return Err(SlvrError::RuntimeError {
                        message: "ln of non-positive number".to_string(),
                    });
                }
                Ok(Value::Decimal((i as f64).ln()))
            }
            Value::Decimal(d) => {
                if d <= 0.0 {
                    return Err(SlvrError::RuntimeError {
                        message: "ln of non-positive number".to_string(),
                    });
                }
                Ok(Value::Decimal(d.ln()))
            }
            _ => Err(SlvrError::TypeError {
                message: "ln requires a number".to_string(),
            }),
        }
    }

    pub fn log10(n: Value) -> SlvrResult<Value> {
        match n {
            Value::Integer(i) => {
                if i <= 0 {
                    return Err(SlvrError::RuntimeError {
                        message: "log10 of non-positive number".to_string(),
                    });
                }
                Ok(Value::Decimal((i as f64).log10()))
            }
            Value::Decimal(d) => {
                if d <= 0.0 {
                    return Err(SlvrError::RuntimeError {
                        message: "log10 of non-positive number".to_string(),
                    });
                }
                Ok(Value::Decimal(d.log10()))
            }
            _ => Err(SlvrError::TypeError {
                message: "log10 requires a number".to_string(),
            }),
        }
    }

    pub fn pow(base: Value, exponent: Value) -> SlvrResult<Value> {
        match (base, exponent) {
            (Value::Integer(b), Value::Integer(e)) => {
                if e < 0 {
                    Ok(Value::Decimal((b as f64).powf(e as f64)))
                } else {
                    Ok(Value::Integer(b.pow(e as u32)))
                }
            }
            (Value::Decimal(b), Value::Integer(e)) => {
                Ok(Value::Decimal(b.powf(e as f64)))
            }
            (Value::Integer(b), Value::Decimal(e)) => {
                Ok(Value::Decimal((b as f64).powf(e)))
            }
            (Value::Decimal(b), Value::Decimal(e)) => {
                Ok(Value::Decimal(b.powf(e)))
            }
            _ => Err(SlvrError::TypeError {
                message: "pow requires numbers".to_string(),
            }),
        }
    }

    pub fn floor(n: Value) -> SlvrResult<Value> {
        match n {
            Value::Integer(i) => Ok(Value::Integer(i)),
            Value::Decimal(d) => Ok(Value::Integer(d.floor() as i128)),
            _ => Err(SlvrError::TypeError {
                message: "floor requires a number".to_string(),
            }),
        }
    }

    pub fn ceil(n: Value) -> SlvrResult<Value> {
        match n {
            Value::Integer(i) => Ok(Value::Integer(i)),
            Value::Decimal(d) => Ok(Value::Integer(d.ceil() as i128)),
            _ => Err(SlvrError::TypeError {
                message: "ceil requires a number".to_string(),
            }),
        }
    }

    pub fn round(n: Value) -> SlvrResult<Value> {
        match n {
            Value::Integer(i) => Ok(Value::Integer(i)),
            Value::Decimal(d) => Ok(Value::Integer(d.round() as i128)),
            _ => Err(SlvrError::TypeError {
                message: "round requires a number".to_string(),
            }),
        }
    }
}

/// Cryptographic functions
pub mod crypto {
    use super::*;

    pub fn sha256(data: Value) -> SlvrResult<Value> {
        let bytes = match data {
            Value::String(s) => s.into_bytes(),
            Value::Integer(i) => i.to_string().into_bytes(),
            _ => return Err(SlvrError::TypeError {
                message: "sha256 requires a string or integer".to_string(),
            }),
        };

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        
        Ok(Value::String(hex::encode(result)))
    }

    pub fn blake3_hash(data: Value) -> SlvrResult<Value> {
        let bytes = match data {
            Value::String(s) => s.into_bytes(),
            Value::Integer(i) => i.to_string().into_bytes(),
            _ => return Err(SlvrError::TypeError {
                message: "blake3 requires a string or integer".to_string(),
            }),
        };

        let hash = blake3::hash(&bytes);
        Ok(Value::String(hash.to_hex().to_string()))
    }

    pub fn verify_sha256(data: Value, hash: Value) -> SlvrResult<Value> {
        let computed = sha256(data)?;
        match (computed, hash) {
            (Value::String(c), Value::String(h)) => {
                Ok(Value::Boolean(c == h))
            }
            _ => Err(SlvrError::TypeError {
                message: "verify-sha256 requires strings".to_string(),
            }),
        }
    }

    pub fn verify_blake3(data: Value, hash: Value) -> SlvrResult<Value> {
        let computed = blake3_hash(data)?;
        match (computed, hash) {
            (Value::String(c), Value::String(h)) => {
                Ok(Value::Boolean(c == h))
            }
            _ => Err(SlvrError::TypeError {
                message: "verify-blake3 requires strings".to_string(),
            }),
        }
    }

    pub fn sha512(data: Value) -> SlvrResult<Value> {
        let bytes = match data {
            Value::String(s) => s.into_bytes(),
            Value::Integer(i) => i.to_string().into_bytes(),
            _ => return Err(SlvrError::TypeError {
                message: "sha512 requires a string or integer".to_string(),
            }),
        };

        let mut hasher = Sha512::new();
        hasher.update(&bytes);
        let result = hasher.finalize();
        
        Ok(Value::String(hex::encode(result)))
    }

    pub fn verify_sha512(data: Value, hash: Value) -> SlvrResult<Value> {
        let computed = sha512(data)?;
        match (computed, hash) {
            (Value::String(c), Value::String(h)) => {
                Ok(Value::Boolean(c == h))
            }
            _ => Err(SlvrError::TypeError {
                message: "verify-sha512 requires strings".to_string(),
            }),
        }
    }

    pub fn hmac_sha256(key: Value, data: Value) -> SlvrResult<Value> {
        use hmac::{Hmac, Mac};
        type HmacSha256 = Hmac<Sha256>;

        let key_bytes = match key {
            Value::String(s) => s.into_bytes(),
            _ => return Err(SlvrError::TypeError {
                message: "hmac-sha256 key must be a string".to_string(),
            }),
        };

        let data_bytes = match data {
            Value::String(s) => s.into_bytes(),
            Value::Integer(i) => i.to_string().into_bytes(),
            _ => return Err(SlvrError::TypeError {
                message: "hmac-sha256 data must be a string or integer".to_string(),
            }),
        };

        let mut mac = HmacSha256::new_from_slice(&key_bytes)
            .map_err(|_| SlvrError::RuntimeError {
                message: "invalid HMAC key".to_string(),
            })?;
        mac.update(&data_bytes);
        let result = mac.finalize();

        Ok(Value::String(hex::encode(result.into_bytes())))
    }

    pub fn hmac_sha512(key: Value, data: Value) -> SlvrResult<Value> {
        use hmac::{Hmac, Mac};
        type HmacSha512 = Hmac<Sha512>;

        let key_bytes = match key {
            Value::String(s) => s.into_bytes(),
            _ => return Err(SlvrError::TypeError {
                message: "hmac-sha512 key must be a string".to_string(),
            }),
        };

        let data_bytes = match data {
            Value::String(s) => s.into_bytes(),
            Value::Integer(i) => i.to_string().into_bytes(),
            _ => return Err(SlvrError::TypeError {
                message: "hmac-sha512 data must be a string or integer".to_string(),
            }),
        };

        let mut mac = HmacSha512::new_from_slice(&key_bytes)
            .map_err(|_| SlvrError::RuntimeError {
                message: "invalid HMAC key".to_string(),
            })?;
        mac.update(&data_bytes);
        let result = mac.finalize();

        Ok(Value::String(hex::encode(result.into_bytes())))
    }
}

/// List operations
pub mod list {
    use super::*;

    pub fn length(list: Value) -> SlvrResult<Value> {
        match list {
            Value::List(l) => Ok(Value::Integer(l.len() as i128)),
            _ => Err(SlvrError::TypeError {
                message: "length requires a list".to_string(),
            }),
        }
    }

    pub fn at(list: Value, index: Value) -> SlvrResult<Value> {
        let lst = match list {
            Value::List(l) => l,
            _ => return Err(SlvrError::TypeError {
                message: "at requires a list".to_string(),
            }),
        };

        let idx = match index {
            Value::Integer(i) => i as usize,
            _ => return Err(SlvrError::TypeError {
                message: "at index must be an integer".to_string(),
            }),
        };

        if idx >= lst.len() {
            return Err(SlvrError::IndexOutOfBounds {
                index: idx as i64,
                length: lst.len(),
            });
        }

        Ok(lst[idx].clone())
    }

    pub fn reverse(list: Value) -> SlvrResult<Value> {
        match list {
            Value::List(mut l) => {
                l.reverse();
                Ok(Value::List(l))
            }
            _ => Err(SlvrError::TypeError {
                message: "reverse requires a list".to_string(),
            }),
        }
    }

    pub fn sort(list: Value) -> SlvrResult<Value> {
        match list {
            Value::List(mut l) => {
                l.sort_by(|a, b| {
                    match (a, b) {
                        (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                        (Value::Decimal(x), Value::Decimal(y)) => {
                            x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                        }
                        (Value::String(x), Value::String(y)) => x.cmp(y),
                        _ => std::cmp::Ordering::Equal,
                    }
                });
                Ok(Value::List(l))
            }
            _ => Err(SlvrError::TypeError {
                message: "sort requires a list".to_string(),
            }),
        }
    }

    pub fn append(list: Value, element: Value) -> SlvrResult<Value> {
        match list {
            Value::List(mut l) => {
                l.push(element);
                Ok(Value::List(l))
            }
            _ => Err(SlvrError::TypeError {
                message: "append requires a list".to_string(),
            }),
        }
    }

    pub fn contains(list: Value, element: Value) -> SlvrResult<Value> {
        match list {
            Value::List(l) => {
                for item in l {
                    if item == element {
                        return Ok(Value::Boolean(true));
                    }
                }
                Ok(Value::Boolean(false))
            }
            _ => Err(SlvrError::TypeError {
                message: "contains requires a list".to_string(),
            }),
        }
    }

    pub fn first(list: Value) -> SlvrResult<Value> {
        match list {
            Value::List(l) => {
                if l.is_empty() {
                    Ok(Value::Boolean(false))
                } else {
                    Ok(l[0].clone())
                }
            }
            _ => Err(SlvrError::TypeError {
                message: "first requires a list".to_string(),
            }),
        }
    }

    pub fn last(list: Value) -> SlvrResult<Value> {
        match list {
            Value::List(l) => {
                if l.is_empty() {
                    Ok(Value::Boolean(false))
                } else {
                    Ok(l[l.len() - 1].clone())
                }
            }
            _ => Err(SlvrError::TypeError {
                message: "last requires a list".to_string(),
            }),
        }
    }

    pub fn sublist(list: Value, start: Value, end: Value) -> SlvrResult<Value> {
        let lst = match list {
            Value::List(l) => l,
            _ => return Err(SlvrError::TypeError {
                message: "sublist requires a list".to_string(),
            }),
        };

        let start_idx = match start {
            Value::Integer(i) => i as usize,
            _ => return Err(SlvrError::TypeError {
                message: "sublist start must be an integer".to_string(),
            }),
        };

        let end_idx = match end {
            Value::Integer(i) => i as usize,
            _ => return Err(SlvrError::TypeError {
                message: "sublist end must be an integer".to_string(),
            }),
        };

        if start_idx > lst.len() || end_idx > lst.len() || start_idx > end_idx {
            return Err(SlvrError::IndexOutOfBounds {
                index: start_idx as i64,
                length: lst.len(),
            });
        }

        Ok(Value::List(lst[start_idx..end_idx].to_vec()))
    }
}

/// Object operations
pub mod object {
    use super::*;

    pub fn keys(obj: Value) -> SlvrResult<Value> {
        match obj {
            Value::Object(map) => {
                let keys: Vec<Value> = map
                    .keys()
                    .map(|k| Value::String(k.clone()))
                    .collect();
                Ok(Value::List(keys))
            }
            _ => Err(SlvrError::TypeError {
                message: "keys requires an object".to_string(),
            }),
        }
    }

    pub fn values(obj: Value) -> SlvrResult<Value> {
        match obj {
            Value::Object(map) => {
                let vals: Vec<Value> = map.values().cloned().collect();
                Ok(Value::List(vals))
            }
            _ => Err(SlvrError::TypeError {
                message: "values requires an object".to_string(),
            }),
        }
    }

    pub fn merge(obj1: Value, obj2: Value) -> SlvrResult<Value> {
        let mut map1 = match obj1 {
            Value::Object(m) => m,
            _ => return Err(SlvrError::TypeError {
                message: "merge requires objects".to_string(),
            }),
        };

        let map2 = match obj2 {
            Value::Object(m) => m,
            _ => return Err(SlvrError::TypeError {
                message: "merge requires objects".to_string(),
            }),
        };

        for (k, v) in map2 {
            map1.insert(k, v);
        }

        Ok(Value::Object(map1))
    }

    pub fn select(obj: Value, fields: Value) -> SlvrResult<Value> {
        let map = match obj {
            Value::Object(m) => m,
            _ => return Err(SlvrError::TypeError {
                message: "select requires an object".to_string(),
            }),
        };

        let field_list = match fields {
            Value::List(l) => l,
            _ => return Err(SlvrError::TypeError {
                message: "select fields must be a list".to_string(),
            }),
        };

        let mut result = HashMap::new();
        for field in field_list {
            if let Value::String(key) = field {
                if let Some(value) = map.get(&key) {
                    result.insert(key, value.clone());
                }
            }
        }

        Ok(Value::Object(result))
    }

    pub fn has_key(obj: Value, key: Value) -> SlvrResult<Value> {
        let map = match obj {
            Value::Object(m) => m,
            _ => return Err(SlvrError::TypeError {
                message: "has-key requires an object".to_string(),
            }),
        };

        let k = match key {
            Value::String(s) => s,
            _ => return Err(SlvrError::TypeError {
                message: "has-key key must be a string".to_string(),
            }),
        };

        Ok(Value::Boolean(map.contains_key(&k)))
    }
}

/// Type conversion functions
pub mod conversion {
    use super::*;

    pub fn to_integer(val: Value) -> SlvrResult<Value> {
        match val {
            Value::Integer(i) => Ok(Value::Integer(i)),
            Value::Decimal(d) => Ok(Value::Integer(d as i128)),
            Value::String(s) => {
                match s.parse::<i128>() {
                    Ok(i) => Ok(Value::Integer(i)),
                    Err(_) => Err(SlvrError::TypeError {
                        message: format!("cannot convert '{}' to integer", s),
                    }),
                }
            }
            Value::Boolean(b) => Ok(Value::Integer(if b { 1 } else { 0 })),
            _ => Err(SlvrError::TypeError {
                message: "cannot convert to integer".to_string(),
            }),
        }
    }

    pub fn to_decimal(val: Value) -> SlvrResult<Value> {
        match val {
            Value::Integer(i) => Ok(Value::Decimal(i as f64)),
            Value::Decimal(d) => Ok(Value::Decimal(d)),
            Value::String(s) => {
                match s.parse::<f64>() {
                    Ok(d) => Ok(Value::Decimal(d)),
                    Err(_) => Err(SlvrError::TypeError {
                        message: format!("cannot convert '{}' to decimal", s),
                    }),
                }
            }
            _ => Err(SlvrError::TypeError {
                message: "cannot convert to decimal".to_string(),
            }),
        }
    }

    pub fn to_string(val: Value) -> SlvrResult<Value> {
        Ok(Value::String(format!("{}", val)))
    }

    pub fn to_boolean(val: Value) -> SlvrResult<Value> {
        Ok(Value::Boolean(val.is_truthy()))
    }
}

/// Type checking functions
pub mod type_check {
    use super::*;

    pub fn is_integer(val: Value) -> SlvrResult<Value> {
        Ok(Value::Boolean(matches!(val, Value::Integer(_))))
    }

    pub fn is_decimal(val: Value) -> SlvrResult<Value> {
        Ok(Value::Boolean(matches!(val, Value::Decimal(_))))
    }

    pub fn is_string(val: Value) -> SlvrResult<Value> {
        Ok(Value::Boolean(matches!(val, Value::String(_))))
    }

    pub fn is_boolean(val: Value) -> SlvrResult<Value> {
        Ok(Value::Boolean(matches!(val, Value::Boolean(_))))
    }

    pub fn is_list(val: Value) -> SlvrResult<Value> {
        Ok(Value::Boolean(matches!(val, Value::List(_))))
    }

    pub fn is_object(val: Value) -> SlvrResult<Value> {
        Ok(Value::Boolean(matches!(val, Value::Object(_))))
    }

    pub fn is_null(val: Value) -> SlvrResult<Value> {
        Ok(Value::Boolean(matches!(val, Value::Boolean(false))))
    }
}
