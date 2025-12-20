//! Advanced Query Engine - Indexing, Filtering, and Pagination
//!
//! This module provides advanced database query capabilities including
//! indexing, complex filtering, and pagination support.

use crate::value::Value;
use crate::error::{SlvrError, SlvrResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query filter condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterCondition {
    Equals(String, Value),
    NotEquals(String, Value),
    GreaterThan(String, Value),
    LessThan(String, Value),
    GreaterThanOrEqual(String, Value),
    LessThanOrEqual(String, Value),
    Contains(String, String),
    In(String, Vec<Value>),
    And(Box<FilterCondition>, Box<FilterCondition>),
    Or(Box<FilterCondition>, Box<FilterCondition>),
    Not(Box<FilterCondition>),
}

impl FilterCondition {
    /// Evaluate filter condition against a value
    pub fn evaluate(&self, record: &HashMap<String, Value>) -> bool {
        match self {
            FilterCondition::Equals(field, value) => {
                record.get(field) == Some(value)
            }
            FilterCondition::NotEquals(field, value) => {
                record.get(field) != Some(value)
            }
            FilterCondition::GreaterThan(field, value) => {
                if let Some(v) = record.get(field) {
                    Self::compare_values(v, value) > 0
                } else {
                    false
                }
            }
            FilterCondition::LessThan(field, value) => {
                if let Some(v) = record.get(field) {
                    Self::compare_values(v, value) < 0
                } else {
                    false
                }
            }
            FilterCondition::GreaterThanOrEqual(field, value) => {
                if let Some(v) = record.get(field) {
                    Self::compare_values(v, value) >= 0
                } else {
                    false
                }
            }
            FilterCondition::LessThanOrEqual(field, value) => {
                if let Some(v) = record.get(field) {
                    Self::compare_values(v, value) <= 0
                } else {
                    false
                }
            }
            FilterCondition::Contains(field, substring) => {
                if let Some(Value::String(s)) = record.get(field) {
                    s.contains(substring)
                } else {
                    false
                }
            }
            FilterCondition::In(field, values) => {
                if let Some(v) = record.get(field) {
                    values.iter().any(|val| val == v)
                } else {
                    false
                }
            }
            FilterCondition::And(left, right) => {
                left.evaluate(record) && right.evaluate(record)
            }
            FilterCondition::Or(left, right) => {
                left.evaluate(record) || right.evaluate(record)
            }
            FilterCondition::Not(condition) => !condition.evaluate(record),
        }
    }

    /// Compare two values
    fn compare_values(a: &Value, b: &Value) -> i32 {
        match (a, b) {
            (Value::Integer(x), Value::Integer(y)) => {
                if x < y {
                    -1
                } else if x > y {
                    1
                } else {
                    0
                }
            }
            (Value::Decimal(x), Value::Decimal(y)) => {
                if x < y {
                    -1
                } else if x > y {
                    1
                } else {
                    0
                }
            }
            (Value::String(x), Value::String(y)) => {
                if x < y {
                    -1
                } else if x > y {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
}

/// Sort order
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Sort specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortSpec {
    pub field: String,
    pub order: SortOrder,
}

/// Query builder for complex queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub filters: Vec<FilterCondition>,
    pub sort: Vec<SortSpec>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl Query {
    /// Create a new query
    pub fn new() -> Self {
        Query {
            filters: Vec::new(),
            sort: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    /// Add a filter condition
    pub fn filter(mut self, condition: FilterCondition) -> Self {
        self.filters.push(condition);
        self
    }

    /// Add a sort specification
    pub fn sort(mut self, field: String, order: SortOrder) -> Self {
        self.sort.push(SortSpec { field, order });
        self
    }

    /// Set limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Execute query on records
    pub fn execute(&self, records: Vec<HashMap<String, Value>>) -> SlvrResult<Vec<HashMap<String, Value>>> {
        let mut result = records;

        // Apply filters
        for filter in &self.filters {
            result.retain(|record| filter.evaluate(record));
        }

        // Apply sorting
        for sort_spec in self.sort.iter().rev() {
            result.sort_by(|a, b| {
                let a_val = a.get(&sort_spec.field);
                let b_val = b.get(&sort_spec.field);

                let cmp = match (a_val, b_val) {
                    (Some(Value::Integer(x)), Some(Value::Integer(y))) => x.cmp(y),
                    (Some(Value::Decimal(x)), Some(Value::Decimal(y))) => {
                        x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Some(Value::String(x)), Some(Value::String(y))) => x.cmp(y),
                    _ => std::cmp::Ordering::Equal,
                };

                match sort_spec.order {
                    SortOrder::Ascending => cmp,
                    SortOrder::Descending => cmp.reverse(),
                }
            });
        }

        // Apply offset and limit
        let offset = self.offset.unwrap_or(0);
        let limit = self.limit.unwrap_or(result.len());

        if offset >= result.len() {
            Ok(Vec::new())
        } else {
            Ok(result[offset..std::cmp::min(offset + limit, result.len())].to_vec())
        }
    }
}

impl Default for Query {
    fn default() -> Self {
        Self::new()
    }
}

/// Database index for fast lookups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    pub name: String,
    pub field: String,
    pub entries: HashMap<String, Vec<String>>,
}

impl Index {
    /// Create a new index
    pub fn new(name: String, field: String) -> Self {
        Index {
            name,
            field,
            entries: HashMap::new(),
        }
    }

    /// Add entry to index
    pub fn add(&mut self, key: String, record_id: String) {
        self.entries
            .entry(key)
            .or_default()
            .push(record_id);
    }

    /// Remove entry from index
    pub fn remove(&mut self, key: &str, record_id: &str) {
        if let Some(ids) = self.entries.get_mut(key) {
            ids.retain(|id| id != record_id);
            if ids.is_empty() {
                self.entries.remove(key);
            }
        }
    }

    /// Lookup records by key
    pub fn lookup(&self, key: &str) -> Vec<String> {
        self.entries
            .get(key).cloned()
            .unwrap_or_default()
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }
}

/// Index manager for managing multiple indexes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexManager {
    indexes: HashMap<String, Index>,
}

impl IndexManager {
    /// Create a new index manager
    pub fn new() -> Self {
        IndexManager {
            indexes: HashMap::new(),
        }
    }

    /// Create an index
    pub fn create_index(&mut self, name: String, field: String) -> SlvrResult<()> {
        if self.indexes.contains_key(&name) {
            return Err(SlvrError::RuntimeError {
                message: format!("index {} already exists", name),
            });
        }
        let index_name = name.clone();
        self.indexes.insert(name, Index::new(index_name, field));
        Ok(())
    }

    /// Get an index
    pub fn get_index(&self, name: &str) -> SlvrResult<Index> {
        self.indexes
            .get(name)
            .cloned()
            .ok_or_else(|| SlvrError::RuntimeError {
                message: format!("index {} not found", name),
            })
    }

    /// Drop an index
    pub fn drop_index(&mut self, name: &str) -> SlvrResult<()> {
        if self.indexes.remove(name).is_none() {
            return Err(SlvrError::RuntimeError {
                message: format!("index {} not found", name),
            });
        }
        Ok(())
    }

    /// List all indexes
    pub fn list_indexes(&self) -> Vec<String> {
        self.indexes.keys().cloned().collect()
    }
}

impl Default for IndexManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Pagination helper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: usize,
    pub page_size: usize,
    pub total: usize,
}

impl Pagination {
    /// Create a new pagination
    pub fn new(page: usize, page_size: usize, total: usize) -> Self {
        Pagination {
            page,
            page_size,
            total,
        }
    }

    /// Get offset
    pub fn offset(&self) -> usize {
        (self.page - 1) * self.page_size
    }

    /// Get total pages
    pub fn total_pages(&self) -> usize {
        self.total.div_ceil(self.page_size)
    }

    /// Check if has next page
    pub fn has_next(&self) -> bool {
        self.page < self.total_pages()
    }

    /// Check if has previous page
    pub fn has_previous(&self) -> bool {
        self.page > 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_equals() {
        let mut record = HashMap::new();
        record.insert("name".to_string(), Value::String("Alice".to_string()));

        let filter = FilterCondition::Equals(
            "name".to_string(),
            Value::String("Alice".to_string()),
        );

        assert!(filter.evaluate(&record));
    }

    #[test]
    fn test_filter_greater_than() {
        let mut record = HashMap::new();
        record.insert("age".to_string(), Value::Integer(30));

        let filter = FilterCondition::GreaterThan(
            "age".to_string(),
            Value::Integer(25),
        );

        assert!(filter.evaluate(&record));
    }

    #[test]
    fn test_query_builder() {
        let query = Query::new()
            .filter(FilterCondition::Equals(
                "status".to_string(),
                Value::String("active".to_string()),
            ))
            .limit(10)
            .offset(0);

        assert_eq!(query.filters.len(), 1);
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(0));
    }

    #[test]
    fn test_pagination() {
        let pagination = Pagination::new(2, 10, 50);
        assert_eq!(pagination.offset(), 10);
        assert_eq!(pagination.total_pages(), 5);
        assert!(pagination.has_next());
        assert!(pagination.has_previous());
    }

    #[test]
    fn test_index() {
        let mut index = Index::new("idx_name".to_string(), "name".to_string());
        index.add("Alice".to_string(), "rec1".to_string());
        index.add("Alice".to_string(), "rec2".to_string());

        let results = index.lookup("Alice");
        assert_eq!(results.len(), 2);
    }
}
