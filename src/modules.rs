//! Module System - Imports, Namespaces, and Cross-Module Dependencies
//!
//! This module provides support for organizing code into modules with proper
//! namespacing, imports, and dependency management.

use crate::error::{SlvrError, SlvrResult};

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Represents a module in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    /// Unique module identifier
    pub id: String,
    /// Module name (fully qualified)
    pub name: String,
    /// Module namespace
    pub namespace: String,
    /// Module version
    pub version: String,
    /// Module source code
    pub source: String,
    /// Exported symbols
    pub exports: Vec<ExportedSymbol>,
    /// Module dependencies
    pub dependencies: Vec<ModuleDependency>,
    /// Module metadata
    pub metadata: HashMap<String, String>,
}

/// Exported symbol from a module
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ExportedSymbol {
    /// Symbol name
    pub name: String,
    /// Symbol type (function, constant, type, etc.)
    pub symbol_type: SymbolType,
    /// Whether symbol is public
    pub public: bool,
    /// Symbol documentation
    pub doc: Option<String>,
}

/// Type of exported symbol
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SymbolType {
    /// Function symbol
    Function,
    /// Constant symbol
    Constant,
    /// Type symbol
    Type,
    /// Schema symbol
    Schema,
    /// Table symbol
    Table,
    /// Module symbol
    Module,
}

impl std::fmt::Display for SymbolType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SymbolType::Function => write!(f, "function"),
            SymbolType::Constant => write!(f, "constant"),
            SymbolType::Type => write!(f, "type"),
            SymbolType::Schema => write!(f, "schema"),
            SymbolType::Table => write!(f, "table"),
            SymbolType::Module => write!(f, "module"),
        }
    }
}

/// Module dependency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleDependency {
    /// Dependency module name
    pub module_name: String,
    /// Version constraint (e.g., "1.0.0", "^1.0.0", "~1.0.0")
    pub version_constraint: String,
    /// Imported symbols (None = import all)
    pub imported_symbols: Option<Vec<String>>,
    /// Alias for the module
    pub alias: Option<String>,
}

/// Import statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStatement {
    /// Module to import from
    pub module: String,
    /// Symbols to import (None = import all)
    pub symbols: Option<Vec<String>>,
    /// Alias for the import
    pub alias: Option<String>,
}

/// Module registry for managing modules and their dependencies
#[derive(Debug, Clone)]
pub struct ModuleRegistry {
    /// Registered modules indexed by fully qualified name
    modules: HashMap<String, Module>,
    /// Module versions indexed by name
    module_versions: HashMap<String, Vec<String>>,
    /// Import resolution cache
    import_cache: HashMap<String, Vec<ExportedSymbol>>,
    /// Dependency graph
    dependency_graph: HashMap<String, Vec<String>>,
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            module_versions: HashMap::new(),
            import_cache: HashMap::new(),
            dependency_graph: HashMap::new(),
        }
    }

    /// Register a new module
    pub fn register_module(
        &mut self,
        name: String,
        namespace: String,
        version: String,
        source: String,
        exports: Vec<ExportedSymbol>,
        dependencies: Vec<ModuleDependency>,
    ) -> SlvrResult<String> {
        let module_id = Uuid::new_v4().to_string();
        let fully_qualified_name = format!("{}::{}", namespace, name);

        // Check for circular dependencies
        self.check_circular_dependencies(&fully_qualified_name, &dependencies)?;

        let module = Module {
            id: module_id.clone(),
            name,
            namespace,
            version: version.clone(),
            source,
            exports,
            dependencies,
            metadata: HashMap::new(),
        };

        self.modules.insert(fully_qualified_name.clone(), module);

        // Update version tracking
        self.module_versions
            .entry(fully_qualified_name.clone())
            .or_default()
            .push(version);

        // Update dependency graph
        self.update_dependency_graph(&fully_qualified_name)?;

        Ok(module_id)
    }

    /// Get a module by name
    pub fn get_module(&self, fully_qualified_name: &str) -> SlvrResult<Module> {
        self.modules
            .get(fully_qualified_name)
            .cloned()
            .ok_or_else(|| SlvrError::RuntimeError {
                message: format!("Module not found: {}", fully_qualified_name),
            })
    }

    /// Get all versions of a module
    pub fn get_module_versions(&self, fully_qualified_name: &str) -> Vec<String> {
        self.module_versions
            .get(fully_qualified_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Resolve an import statement
    pub fn resolve_import(&mut self, import: &ImportStatement) -> SlvrResult<Vec<ExportedSymbol>> {
        let cache_key = format!("{}::{:?}", import.module, import.symbols);

        // Check cache
        if let Some(cached) = self.import_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Get module
        let module = self.get_module(&import.module)?;

        // Filter exports based on import statement
        let exported: Vec<ExportedSymbol> = if let Some(symbols) = &import.symbols {
            module
                .exports
                .iter()
                .filter(|e| symbols.contains(&e.name) && e.public)
                .cloned()
                .collect()
        } else {
            module.exports.iter().filter(|e| e.public).cloned().collect()
        };

        // Cache result
        self.import_cache.insert(cache_key, exported.clone());

        Ok(exported)
    }

    /// Check if a symbol is accessible from a module
    pub fn is_symbol_accessible(
        &self,
        from_module: &str,
        target_module: &str,
        symbol_name: &str,
    ) -> SlvrResult<bool> {
        // Get target module
        let target = self.get_module(target_module)?;

        // Find symbol
        if let Some(symbol) = target.exports.iter().find(|s| s.name == symbol_name) {
            // Check if symbol is public
            if !symbol.public {
                return Ok(false);
            }

            // Check if from_module imports target_module
            let from = self.get_module(from_module)?;
            let imports_target = from
                .dependencies
                .iter()
                .any(|d| d.module_name == target_module);

            Ok(imports_target)
        } else {
            Ok(false)
        }
    }

    /// Get all modules in a namespace
    pub fn get_namespace_modules(&self, namespace: &str) -> Vec<Module> {
        self.modules
            .values()
            .filter(|m| m.namespace == namespace)
            .cloned()
            .collect()
    }

    /// Get module dependencies
    pub fn get_dependencies(&self, fully_qualified_name: &str) -> SlvrResult<Vec<Module>> {
        let module = self.get_module(fully_qualified_name)?;

        let mut deps = Vec::new();
        for dep in &module.dependencies {
            if let Ok(dep_module) = self.get_module(&dep.module_name) {
                deps.push(dep_module);
            }
        }

        Ok(deps)
    }

    /// Get modules that depend on a given module
    pub fn get_dependents(&self, fully_qualified_name: &str) -> Vec<Module> {
        self.modules
            .values()
            .filter(|m| {
                m.dependencies
                    .iter()
                    .any(|d| d.module_name == fully_qualified_name)
            })
            .cloned()
            .collect()
    }

    /// Check for circular dependencies
    fn check_circular_dependencies(
        &self,
        module_name: &str,
        dependencies: &[ModuleDependency],
    ) -> SlvrResult<()> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for dep in dependencies {
            if self.has_circular_dep(&dep.module_name, &mut visited, &mut rec_stack)? {
                return Err(SlvrError::RuntimeError {
                    message: format!("Circular dependency detected involving {}", module_name),
                });
            }
        }

        Ok(())
    }

    /// Helper to detect circular dependencies
    fn has_circular_dep(
        &self,
        module_name: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> SlvrResult<bool> {
        visited.insert(module_name.to_string());
        rec_stack.insert(module_name.to_string());

        if let Ok(module) = self.get_module(module_name) {
            for dep in &module.dependencies {
                if !visited.contains(&dep.module_name) {
                    if self.has_circular_dep(&dep.module_name, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.contains(&dep.module_name) {
                    return Ok(true);
                }
            }
        }

        rec_stack.remove(module_name);
        Ok(false)
    }

    /// Update dependency graph
    fn update_dependency_graph(&mut self, module_name: &str) -> SlvrResult<()> {
        let module = self.get_module(module_name)?;

        let deps: Vec<String> = module
            .dependencies
            .iter()
            .map(|d| d.module_name.clone())
            .collect();

        self.dependency_graph.insert(module_name.to_string(), deps);

        Ok(())
    }

    /// Get module statistics
    pub fn get_stats(&self) -> ModuleStats {
        let total_modules = self.modules.len();
        let total_namespaces = self
            .modules
            .values()
            .map(|m| m.namespace.clone())
            .collect::<HashSet<_>>()
            .len();

        let total_exports: usize = self.modules.values().map(|m| m.exports.len()).sum();
        let total_dependencies: usize = self.modules.values().map(|m| m.dependencies.len()).sum();

        ModuleStats {
            total_modules,
            total_namespaces,
            total_exports,
            total_dependencies,
        }
    }
}

/// Statistics for module registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleStats {
    pub total_modules: usize,
    pub total_namespaces: usize,
    pub total_exports: usize,
    pub total_dependencies: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_module() {
        let mut registry = ModuleRegistry::new();
        let _module_id = registry
            .register_module(
                "token".to_string(),
                "contracts".to_string(),
                "1.0.0".to_string(),
                "code".to_string(),
                vec![ExportedSymbol {
                    name: "transfer".to_string(),
                    symbol_type: SymbolType::Function,
                    public: true,
                    doc: None,
                }],
                vec![],
            )
            .unwrap();

        let module = registry.get_module("contracts::token").unwrap();
        assert_eq!(module.name, "token");
    }

    #[test]
    fn test_resolve_import() {
        let mut registry = ModuleRegistry::new();
        registry
            .register_module(
                "token".to_string(),
                "contracts".to_string(),
                "1.0.0".to_string(),
                "code".to_string(),
                vec![ExportedSymbol {
                    name: "transfer".to_string(),
                    symbol_type: SymbolType::Function,
                    public: true,
                    doc: None,
                }],
                vec![],
            )
            .unwrap();

        let import = ImportStatement {
            module: "contracts::token".to_string(),
            symbols: None,
            alias: None,
        };

        let symbols = registry.resolve_import(&import).unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "transfer");
    }

    #[test]
    fn test_module_dependencies() {
        let mut registry = ModuleRegistry::new();
        registry
            .register_module(
                "base".to_string(),
                "contracts".to_string(),
                "1.0.0".to_string(),
                "code".to_string(),
                vec![],
                vec![],
            )
            .unwrap();

        registry
            .register_module(
                "token".to_string(),
                "contracts".to_string(),
                "1.0.0".to_string(),
                "code".to_string(),
                vec![],
                vec![ModuleDependency {
                    module_name: "contracts::base".to_string(),
                    version_constraint: "1.0.0".to_string(),
                    imported_symbols: None,
                    alias: None,
                }],
            )
            .unwrap();

        let deps = registry.get_dependencies("contracts::token").unwrap();
        assert_eq!(deps.len(), 1);
    }

    #[test]
    fn test_namespace_modules() {
        let mut registry = ModuleRegistry::new();
        registry
            .register_module(
                "token".to_string(),
                "contracts".to_string(),
                "1.0.0".to_string(),
                "code".to_string(),
                vec![],
                vec![],
            )
            .unwrap();

        registry
            .register_module(
                "swap".to_string(),
                "contracts".to_string(),
                "1.0.0".to_string(),
                "code".to_string(),
                vec![],
                vec![],
            )
            .unwrap();

        let modules = registry.get_namespace_modules("contracts");
        assert_eq!(modules.len(), 2);
    }

    #[test]
    fn test_module_stats() {
        let mut registry = ModuleRegistry::new();
        registry
            .register_module(
                "token".to_string(),
                "contracts".to_string(),
                "1.0.0".to_string(),
                "code".to_string(),
                vec![ExportedSymbol {
                    name: "transfer".to_string(),
                    symbol_type: SymbolType::Function,
                    public: true,
                    doc: None,
                }],
                vec![],
            )
            .unwrap();

        let stats = registry.get_stats();
        assert_eq!(stats.total_modules, 1);
        assert_eq!(stats.total_exports, 1);
    }
}
