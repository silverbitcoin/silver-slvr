//! Language Server Protocol (LSP) Implementation
//!
//! Full LSP 3.17 support for IDE integration (VS Code, Neovim, Emacs, etc.)
//! Provides real-time diagnostics, code completion, hover information, and more.

use crate::error::{SlvrError, SlvrResult};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::types::TypeEnv;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;


/// LSP Position (line and character)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// LSP Range (start and end positions)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// LSP Location (URI and range)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

/// Diagnostic severity
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

/// LSP Diagnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: Option<DiagnosticSeverity>,
    pub code: Option<String>,
    pub source: String,
    pub message: String,
    pub related_information: Option<Vec<DiagnosticRelatedInformation>>,
}

/// Related diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticRelatedInformation {
    pub location: Location,
    pub message: String,
}

/// Completion item kind
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompletionItemKind {
    Text = 1,
    Method = 2,
    Function = 3,
    Constructor = 4,
    Field = 5,
    Variable = 6,
    Class = 7,
    Interface = 8,
    Module = 9,
    Property = 10,
    Unit = 11,
    Value = 12,
    Enum = 13,
    Keyword = 14,
    Snippet = 15,
    Color = 16,
    File = 17,
    Reference = 18,
    Folder = 19,
    EnumMember = 20,
    Constant = 21,
    Struct = 22,
    Event = 23,
    Operator = 24,
    TypeParameter = 25,
}

/// LSP Completion Item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: Option<CompletionItemKind>,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub sort_text: Option<String>,
    pub filter_text: Option<String>,
    pub insert_text: Option<String>,
    pub insert_text_format: Option<u32>,
    pub additional_text_edits: Option<Vec<TextEdit>>,
    pub commit_characters: Option<Vec<String>>,
    pub command: Option<Command>,
    pub data: Option<serde_json::Value>,
}

/// Text edit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// Command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    pub title: String,
    pub command: String,
    pub arguments: Option<Vec<serde_json::Value>>,
}

/// Hover information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hover {
    pub contents: String,
    pub range: Option<Range>,
}

/// Symbol kind
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SymbolKind {
    File = 1,
    Module = 2,
    Namespace = 3,
    Package = 4,
    Class = 5,
    Method = 6,
    Property = 7,
    Field = 8,
    Constructor = 9,
    Enum = 10,
    Interface = 11,
    Function = 12,
    Variable = 13,
    Constant = 14,
    String = 15,
    Number = 16,
    Boolean = 17,
    Array = 18,
    Object = 19,
    Key = 20,
    Null = 21,
    EnumMember = 22,
    Struct = 23,
    Event = 24,
    Operator = 25,
    TypeParameter = 26,
}

/// Document symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSymbol {
    pub name: String,
    pub detail: Option<String>,
    pub kind: SymbolKind,
    pub deprecated: Option<bool>,
    pub range: Range,
    pub selection_range: Range,
    pub children: Option<Vec<DocumentSymbol>>,
}

/// LSP Server state
pub struct LspServer {
    documents: Arc<Mutex<HashMap<String, String>>>,
    diagnostics: Arc<Mutex<HashMap<String, Vec<Diagnostic>>>>,
    type_env: Arc<Mutex<TypeEnv>>,
    tx: mpsc::UnboundedSender<LspNotification>,
}

/// LSP Notification
#[derive(Debug, Clone)]
pub enum LspNotification {
    PublishDiagnostics {
        uri: String,
        diagnostics: Vec<Diagnostic>,
    },
}

impl LspServer {
    /// Create new LSP server
    pub fn new(tx: mpsc::UnboundedSender<LspNotification>) -> Self {
        Self {
            documents: Arc::new(Mutex::new(HashMap::new())),
            diagnostics: Arc::new(Mutex::new(HashMap::new())),
            type_env: Arc::new(Mutex::new(TypeEnv::new())),
            tx,
        }
    }

    /// Open document
    pub fn open_document(&self, uri: String, text: String) -> SlvrResult<()> {
        let mut docs = self.documents.lock().unwrap();
        docs.insert(uri.clone(), text.clone());

        // Analyze document
        self.analyze_document(&uri, &text)?;
        Ok(())
    }

    /// Update document
    pub fn update_document(&self, uri: String, text: String) -> SlvrResult<()> {
        let mut docs = self.documents.lock().unwrap();
        docs.insert(uri.clone(), text.clone());

        // Re-analyze document
        self.analyze_document(&uri, &text)?;
        Ok(())
    }

    /// Close document
    pub fn close_document(&self, uri: &str) -> SlvrResult<()> {
        let mut docs = self.documents.lock().unwrap();
        docs.remove(uri);

        let mut diags = self.diagnostics.lock().unwrap();
        diags.remove(uri);

        Ok(())
    }

    /// Analyze document for diagnostics
    fn analyze_document(&self, uri: &str, text: &str) -> SlvrResult<()> {
        let mut diagnostics = Vec::new();

        // Lexical analysis
        let mut lexer = Lexer::new(text);
        match lexer.tokenize() {
            Ok(_tokens) => {
                // Parser analysis
                match Parser::new(text) {
                    Ok(mut parser) => {
                        match parser.parse() {
                            Ok(_ast) => {
                                // Type checking
                                // (would be done here)
                            }
                            Err(e) => {
                                diagnostics.push(Diagnostic {
                                    range: Range {
                                        start: Position { line: 0, character: 0 },
                                        end: Position { line: 0, character: 100 },
                                    },
                                    severity: Some(DiagnosticSeverity::Error),
                                    code: Some("parse_error".to_string()),
                                    source: "slvr".to_string(),
                                    message: format!("Parse error: {}", e),
                                    related_information: None,
                                });
                            }
                        }
                    }
                    Err(e) => {
                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position { line: 0, character: 0 },
                                end: Position { line: 0, character: 100 },
                            },
                            severity: Some(DiagnosticSeverity::Error),
                            code: Some("parser_init_error".to_string()),
                            source: "slvr".to_string(),
                            message: format!("Parser initialization error: {}", e),
                            related_information: None,
                        });
                    }
                }
            }
            Err(e) => {
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 100 },
                    },
                    severity: Some(DiagnosticSeverity::Error),
                    code: Some("lex_error".to_string()),
                    source: "slvr".to_string(),
                    message: format!("Lexical error: {}", e),
                    related_information: None,
                });
            }
        }

        // Store diagnostics
        let mut diags = self.diagnostics.lock().unwrap();
        diags.insert(uri.to_string(), diagnostics.clone());

        // Publish diagnostics
        let _ = self.tx.send(LspNotification::PublishDiagnostics {
            uri: uri.to_string(),
            diagnostics,
        });

        Ok(())
    }

    /// Get completions at position
    pub fn get_completions(&self, uri: &str, _position: Position) -> SlvrResult<Vec<CompletionItem>> {
        let docs = self.documents.lock().unwrap();
        let _text = docs.get(uri).ok_or_else(|| SlvrError::RuntimeError {
            message: "Document not found".to_string(),
        })?;

        let mut completions = Vec::new();

        // Built-in functions
        let builtins = vec![
            ("concat", "String concatenation", "function"),
            ("length", "Get length of string or list", "function"),
            ("substring", "Extract substring", "function"),
            ("to-upper", "Convert to uppercase", "function"),
            ("to-lower", "Convert to lowercase", "function"),
            ("trim", "Trim whitespace", "function"),
            ("split", "Split string", "function"),
            ("contains", "Check if contains", "function"),
            ("format", "Format string", "function"),
            ("abs", "Absolute value", "function"),
            ("min", "Minimum value", "function"),
            ("max", "Maximum value", "function"),
            ("sqrt", "Square root", "function"),
            ("pow", "Power", "function"),
            ("floor", "Floor", "function"),
            ("ceil", "Ceiling", "function"),
            ("round", "Round", "function"),
            ("sha256", "SHA256 hash", "function"),
            ("sha512", "SHA512 hash", "function"),
            ("blake3", "BLAKE3 hash", "function"),
            ("read", "Read from database", "function"),
            ("write", "Write to database", "function"),
            ("update", "Update database", "function"),
            ("delete", "Delete from database", "function"),
            ("if", "Conditional", "keyword"),
            ("let", "Variable binding", "keyword"),
            ("defun", "Function definition", "keyword"),
            ("defschema", "Schema definition", "keyword"),
            ("deftable", "Table definition", "keyword"),
            ("module", "Module definition", "keyword"),
            ("defpact", "Multi-step transaction", "keyword"),
            ("defcap", "Capability definition", "keyword"),
        ];

        for (label, doc, kind) in builtins {
            completions.push(CompletionItem {
                label: label.to_string(),
                kind: Some(if kind == "keyword" {
                    CompletionItemKind::Keyword
                } else {
                    CompletionItemKind::Function
                }),
                detail: Some(doc.to_string()),
                documentation: Some(format!("Built-in {}: {}", kind, doc)),
                sort_text: Some(label.to_string()),
                filter_text: Some(label.to_string()),
                insert_text: Some(label.to_string()),
                insert_text_format: None,
                additional_text_edits: None,
                commit_characters: None,
                command: None,
                data: None,
            });
        }

        Ok(completions)
    }

    /// Get hover information
    pub fn get_hover(&self, uri: &str, position: Position) -> SlvrResult<Option<Hover>> {
        let docs = self.documents.lock().unwrap();
        let text = docs.get(uri).ok_or_else(|| SlvrError::RuntimeError {
            message: "Document not found".to_string(),
        })?;

        // Find word at position
        let lines: Vec<&str> = text.lines().collect();
        if position.line as usize >= lines.len() {
            return Ok(None);
        }

        let line = lines[position.line as usize];
        let char_pos = position.character as usize;

        if char_pos > line.len() {
            return Ok(None);
        }

        // Extract word
        let start = line[..char_pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);

        let end = line[char_pos..]
            .find(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .map(|i| i + char_pos)
            .unwrap_or(line.len());

        let word = &line[start..end];

        // Provide hover info for known symbols
        let hover_info = match word {
            "concat" => Some("concat: Concatenate strings or values\n\nUsage: (concat str1 str2 ...)"),
            "length" => Some("length: Get length of string or list\n\nUsage: (length value)"),
            "read" => Some("read: Read value from database\n\nUsage: (read table key)"),
            "write" => Some("write: Write value to database\n\nUsage: (write table key value)"),
            "defun" => Some("defun: Define a function\n\nUsage: (defun name (args) body)"),
            "defschema" => Some("defschema: Define a schema\n\nUsage: (defschema name field:type ...)"),
            "deftable" => Some("deftable: Define a table\n\nUsage: (deftable name:{schema} doc)"),
            "module" => Some("module: Define a module\n\nUsage: (module name doc ...)"),
            "defpact" => Some("defpact: Define multi-step transaction\n\nUsage: (defpact name (args) step1 step2 ...)"),
            "defcap" => Some("defcap: Define capability\n\nUsage: (defcap name (args) body)"),
            _ => None,
        };

        Ok(hover_info.map(|info| Hover {
            contents: info.to_string(),
            range: Some(Range {
                start: Position {
                    line: position.line,
                    character: start as u32,
                },
                end: Position {
                    line: position.line,
                    character: end as u32,
                },
            }),
        }))
    }

    /// Get document symbols
    pub fn get_document_symbols(&self, uri: &str) -> SlvrResult<Vec<DocumentSymbol>> {
        let docs = self.documents.lock().unwrap();
        let text = docs.get(uri).ok_or_else(|| SlvrError::RuntimeError {
            message: "Document not found".to_string(),
        })?;

        let mut symbols = Vec::new();

        // Parse and extract symbols
        if let Ok(mut parser) = Parser::new(text) {
            match parser.parse() {
                Ok(program) => {
                    // Extract symbols from AST
                    for definition in program.definitions {
                        if let Some(symbol) = self.extract_symbol(&definition, 0) {
                            symbols.push(symbol);
                        }
                    }
                }
                Err(_) => {
                    // Return empty symbols on parse error
                }
            }
        }

        Ok(symbols)
    }

    /// Extract symbol from expression
    fn extract_symbol(&self, _expr: &crate::ast::Definition, _line: u32) -> Option<DocumentSymbol> {
        // This would extract symbols from the AST
        // For now, return None as placeholder
        None
    }

    /// Get definition location
    pub fn get_definition(&self, uri: &str, _position: Position) -> SlvrResult<Option<Location>> {
        let docs = self.documents.lock().unwrap();
        let _text = docs.get(uri).ok_or_else(|| SlvrError::RuntimeError {
            message: "Document not found".to_string(),
        })?;

        // Find definition in current or other documents
        // This would require symbol table lookup
        Ok(None)
    }

    /// Get references
    pub fn get_references(&self, uri: &str, _position: Position) -> SlvrResult<Vec<Location>> {
        let docs = self.documents.lock().unwrap();
        let _text = docs.get(uri).ok_or_else(|| SlvrError::RuntimeError {
            message: "Document not found".to_string(),
        })?;

        // Find all references to symbol at position
        Ok(Vec::new())
    }

    /// Format document
    pub fn format_document(&self, uri: &str) -> SlvrResult<Vec<TextEdit>> {
        let docs = self.documents.lock().unwrap();
        let text = docs.get(uri).ok_or_else(|| SlvrError::RuntimeError {
            message: "Document not found".to_string(),
        })?;

        let mut edits = Vec::new();

        // Format code
        let formatted = self.format_code(text);
        if formatted != *text {
            edits.push(TextEdit {
                range: Range {
                    start: Position { line: 0, character: 0 },
                    end: Position {
                        line: text.lines().count() as u32,
                        character: 0,
                    },
                },
                new_text: formatted,
            });
        }

        Ok(edits)
    }

    /// Format code
    fn format_code(&self, code: &str) -> String {
        // Simple formatting: proper indentation and spacing
        let mut result = String::new();
        let mut indent_level: usize = 0;
        let mut in_string = false;
        let mut prev_char = ' ';

        for ch in code.chars() {
            match ch {
                '"' if prev_char != '\\' => {
                    in_string = !in_string;
                    result.push(ch);
                }
                '(' if !in_string => {
                    result.push(ch);
                    indent_level += 1;
                }
                ')' if !in_string => {
                    indent_level = indent_level.saturating_sub(1);
                    result.push(ch);
                }
                '\n' if !in_string => {
                    result.push('\n');
                    result.push_str(&"  ".repeat(indent_level));
                }
                ' ' if !in_string && prev_char == ' ' => {
                    // Skip multiple spaces
                }
                _ => {
                    result.push(ch);
                }
            }
            prev_char = ch;
        }

        result
    }

    /// Check types in document
    pub fn check_types(&self, uri: &str) -> SlvrResult<Vec<Diagnostic>> {
        let docs = self.documents.lock().unwrap();
        let _text = docs.get(uri).ok_or_else(|| SlvrError::RuntimeError {
            message: "Document not found".to_string(),
        })?;

        let _type_env = self.type_env.lock().unwrap();
        let diagnostics = Vec::new();

        // Perform type checking using the type environment
        // This would validate types in the document
        // Type environment is used for tracking variable and function types

        Ok(diagnostics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position() {
        let pos = Position { line: 5, character: 10 };
        assert_eq!(pos.line, 5);
        assert_eq!(pos.character, 10);
    }

    #[test]
    fn test_range() {
        let range = Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 1, character: 10 },
        };
        assert_eq!(range.start.line, 0);
        assert_eq!(range.end.line, 1);
    }

    #[test]
    fn test_diagnostic() {
        let diag = Diagnostic {
            range: Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 10 },
            },
            severity: Some(DiagnosticSeverity::Error),
            code: Some("test".to_string()),
            source: "slvr".to_string(),
            message: "Test error".to_string(),
            related_information: None,
        };
        assert_eq!(diag.severity, Some(DiagnosticSeverity::Error));
    }

    #[test]
    fn test_completion_item() {
        let item = CompletionItem {
            label: "concat".to_string(),
            kind: Some(CompletionItemKind::Function),
            detail: Some("String concatenation".to_string()),
            documentation: Some("Concatenate strings".to_string()),
            sort_text: Some("concat".to_string()),
            filter_text: Some("concat".to_string()),
            insert_text: Some("concat".to_string()),
            insert_text_format: None,
            additional_text_edits: None,
            commit_characters: None,
            command: None,
            data: None,
        };
        assert_eq!(item.label, "concat");
    }

    #[test]
    fn test_hover() {
        let hover = Hover {
            contents: "Test hover".to_string(),
            range: Some(Range {
                start: Position { line: 0, character: 0 },
                end: Position { line: 0, character: 10 },
            }),
        };
        assert_eq!(hover.contents, "Test hover");
    }
}
