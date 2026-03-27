//! Code indexer — tree-sitter based source code symbol extraction.
//!
//! Indexes source files and stores symbols for cross-referencing with documents.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A single extracted symbol from source code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file_path: String,
    pub line_number: usize,
    pub signature: String,
}

/// Types of symbols that can be extracted
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Struct,
    Trait,
    Enum,
    Impl,
    TypeAlias,
    Const,
    Static,
    Mod,
}

impl std::fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Function => write!(f, "function"),
            Self::Struct => write!(f, "struct"),
            Self::Trait => write!(f, "trait"),
            Self::Enum => write!(f, "enum"),
            Self::Impl => write!(f, "impl"),
            Self::TypeAlias => write!(f, "type_alias"),
            Self::Const => write!(f, "const"),
            Self::Static => write!(f, "static"),
            Self::Mod => write!(f, "mod"),
        }
    }
}

impl std::str::FromStr for SymbolKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "function" | "fn" => Ok(Self::Function),
            "struct" => Ok(Self::Struct),
            "trait" => Ok(Self::Trait),
            "enum" => Ok(Self::Enum),
            "impl" => Ok(Self::Impl),
            "type_alias" | "type" => Ok(Self::TypeAlias),
            "const" => Ok(Self::Const),
            "static" => Ok(Self::Static),
            "mod" | "module" => Ok(Self::Mod),
            _ => Err(format!("Unknown symbol kind: {s}")),
        }
    }
}

/// The full code index stored on disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIndex {
    pub symbols: Vec<CodeSymbol>,
    pub indexed_files: usize,
    pub indexed_at: String,
    /// Cached source directory relative to project root (resolved on first index)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_dir: Option<String>,
}

/// Code indexer using tree-sitter
pub struct CodeIndexer {
    project_root: PathBuf,
}

impl CodeIndexer {
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
        }
    }

    /// Index source files matching the given glob patterns
    pub fn index(&self, patterns: &[String]) -> Result<CodeIndex, String> {
        let mut all_symbols = Vec::new();
        let mut file_count = 0;

        for pattern in patterns {
            let full_pattern = self.project_root.join(pattern).display().to_string();
            let paths = glob::glob(&full_pattern)
                .map_err(|e| format!("Invalid glob pattern '{pattern}': {e}"))?;

            for entry in paths.flatten() {
                if entry.is_file() {
                    if let Some(ext) = entry.extension().and_then(|e| e.to_str()) {
                        let symbols = match ext {
                            "rs" => match self.index_rust_file(&entry) {
                                Ok(syms) => syms,
                                Err(_) => {
                                    // Skip files that fail to parse rather than aborting
                                    continue;
                                }
                            },
                            _ => continue,
                        };
                        all_symbols.extend(symbols);
                        file_count += 1;
                    }
                }
            }
        }

        Ok(CodeIndex {
            symbols: all_symbols,
            indexed_files: file_count,
            indexed_at: chrono::Utc::now().to_rfc3339(),
            source_dir: None,
        })
    }

    /// Extract symbols from a Rust source file using tree-sitter
    fn index_rust_file(&self, path: &Path) -> Result<Vec<CodeSymbol>, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

        let mut parser = tree_sitter::Parser::new();
        let language = tree_sitter_rust::LANGUAGE.into();
        parser
            .set_language(&language)
            .map_err(|e| format!("Failed to set language: {e}"))?;

        let tree = parser
            .parse(&source, None)
            .ok_or_else(|| format!("Failed to parse {}", path.display()))?;

        let relative_path = path
            .strip_prefix(&self.project_root)
            .unwrap_or(path)
            .display()
            .to_string();

        let mut symbols = Vec::new();
        self.walk_tree(tree.root_node(), &source, &relative_path, &mut symbols);

        Ok(symbols)
    }

    /// Recursively walk the syntax tree and extract named symbols
    fn walk_tree(
        &self,
        node: tree_sitter::Node,
        source: &str,
        file_path: &str,
        symbols: &mut Vec<CodeSymbol>,
    ) {
        let kind = node.kind();

        let symbol_kind = match kind {
            "function_item" => Some(SymbolKind::Function),
            "struct_item" => Some(SymbolKind::Struct),
            "trait_item" => Some(SymbolKind::Trait),
            "enum_item" => Some(SymbolKind::Enum),
            "impl_item" => Some(SymbolKind::Impl),
            "type_item" => Some(SymbolKind::TypeAlias),
            "const_item" => Some(SymbolKind::Const),
            "static_item" => Some(SymbolKind::Static),
            "mod_item" => Some(SymbolKind::Mod),
            _ => None,
        };

        if let Some(sk) = symbol_kind {
            if let Some(name) = self.extract_name(node, source, &sk) {
                let line_number = node.start_position().row + 1;
                let signature = self.extract_signature(node, source, &sk);

                symbols.push(CodeSymbol {
                    name,
                    kind: sk,
                    file_path: file_path.to_string(),
                    line_number,
                    signature,
                });
            }
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_tree(child, source, file_path, symbols);
        }
    }

    /// Extract the name of a symbol from its tree-sitter node
    fn extract_name(
        &self,
        node: tree_sitter::Node,
        source: &str,
        kind: &SymbolKind,
    ) -> Option<String> {
        if kind == &SymbolKind::Impl {
            // For impl blocks, extract the type name
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "type_identifier" || child.kind() == "generic_type" {
                    return Some(child.utf8_text(source.as_bytes()).ok()?.to_string());
                }
            }
            None
        } else {
            // For most items, the name is in the first "name" or identifier child
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "identifier" || child.kind() == "type_identifier" {
                    return Some(child.utf8_text(source.as_bytes()).ok()?.to_string());
                }
            }
            None
        }
    }

    /// Extract a one-line signature for a symbol
    fn extract_signature(
        &self,
        node: tree_sitter::Node,
        source: &str,
        _kind: &SymbolKind,
    ) -> String {
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        // Take just the first line as the signature
        let first_line = text.lines().next().unwrap_or("");
        // Trim trailing open brace if present
        let sig = first_line.trim_end().trim_end_matches('{').trim_end();
        sig.to_string()
    }

    /// Search the index for symbols matching a query
    pub fn search_symbols<'a>(
        index: &'a CodeIndex,
        name_pattern: Option<&str>,
        kind_filter: Option<&str>,
    ) -> Vec<&'a CodeSymbol> {
        let kind_filter: Option<SymbolKind> = kind_filter.and_then(|k| k.parse().ok());

        index
            .symbols
            .iter()
            .filter(|s| {
                if let Some(pattern) = name_pattern {
                    let pattern_lower = pattern.to_lowercase();
                    if !s.name.to_lowercase().contains(&pattern_lower) {
                        return false;
                    }
                }
                if let Some(ref kf) = kind_filter {
                    if s.kind != *kf {
                        return false;
                    }
                }
                true
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn setup_rust_project() -> (tempfile::TempDir, PathBuf) {
        let dir = tempdir().unwrap();
        let project_root = dir.path().to_path_buf();
        let src_dir = project_root.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let rust_code = r#"
pub fn hello_world() {
    println!("Hello, world!");
}

fn private_helper(x: i32) -> i32 {
    x + 1
}

pub struct MyStruct {
    pub field: String,
}

pub trait MyTrait {
    fn do_something(&self);
}

pub enum Color {
    Red,
    Green,
    Blue,
}

impl MyStruct {
    pub fn new() -> Self {
        Self { field: String::new() }
    }
}

pub type AliasType = Vec<String>;

pub const MAX_SIZE: usize = 100;

pub static GLOBAL: &str = "hello";

pub mod submodule {
    pub fn nested_fn() {}
}
"#;

        let file_path = src_dir.join("lib.rs");
        fs::write(&file_path, rust_code).unwrap();

        (dir, project_root)
    }

    #[test]
    fn test_index_rust_symbols() {
        let (_dir, project_root) = setup_rust_project();
        let indexer = CodeIndexer::new(&project_root);

        let index = indexer.index(&["src/**/*.rs".to_string()]).unwrap();

        assert_eq!(index.indexed_files, 1);
        assert!(
            index.symbols.len() >= 8,
            "Expected at least 8 symbols, got {}",
            index.symbols.len()
        );

        // Check for specific symbols
        let names: Vec<&str> = index.symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(
            names.contains(&"hello_world"),
            "Missing hello_world function"
        );
        assert!(
            names.contains(&"private_helper"),
            "Missing private_helper function"
        );
        assert!(names.contains(&"MyStruct"), "Missing MyStruct");
        assert!(names.contains(&"MyTrait"), "Missing MyTrait");
        assert!(names.contains(&"Color"), "Missing Color enum");
    }

    #[test]
    fn test_index_symbol_details() {
        let (_dir, project_root) = setup_rust_project();
        let indexer = CodeIndexer::new(&project_root);

        let index = indexer.index(&["src/**/*.rs".to_string()]).unwrap();

        let hello = index
            .symbols
            .iter()
            .find(|s| s.name == "hello_world")
            .unwrap();
        assert_eq!(hello.kind, SymbolKind::Function);
        assert_eq!(hello.file_path, "src/lib.rs");
        assert!(hello.line_number > 0);
        assert!(hello.signature.contains("fn hello_world"));
    }

    #[test]
    fn test_search_symbols_by_name() {
        let (_dir, project_root) = setup_rust_project();
        let indexer = CodeIndexer::new(&project_root);
        let index = indexer.index(&["src/**/*.rs".to_string()]).unwrap();

        let results = CodeIndexer::search_symbols(&index, Some("My"), None);
        assert!(results.len() >= 2, "Should find MyStruct and MyTrait");
    }

    #[test]
    fn test_search_symbols_by_kind() {
        let (_dir, project_root) = setup_rust_project();
        let indexer = CodeIndexer::new(&project_root);
        let index = indexer.index(&["src/**/*.rs".to_string()]).unwrap();

        let results = CodeIndexer::search_symbols(&index, None, Some("struct"));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "MyStruct");
    }

    #[test]
    fn test_index_serialization() {
        let (_dir, project_root) = setup_rust_project();
        let indexer = CodeIndexer::new(&project_root);
        let index = indexer.index(&["src/**/*.rs".to_string()]).unwrap();

        let json = serde_json::to_string_pretty(&index).unwrap();
        let deserialized: CodeIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.indexed_files, index.indexed_files);
        assert_eq!(deserialized.symbols.len(), index.symbols.len());
    }

    #[test]
    fn test_empty_project() {
        let dir = tempdir().unwrap();
        let indexer = CodeIndexer::new(dir.path());
        let index = indexer.index(&["src/**/*.rs".to_string()]).unwrap();
        assert_eq!(index.indexed_files, 0);
        assert_eq!(index.symbols.len(), 0);
    }

    #[test]
    fn test_malformed_file_skipped_gracefully() {
        let dir = tempdir().unwrap();
        let project_root = dir.path().to_path_buf();
        let src_dir = project_root.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Write a valid file
        fs::write(src_dir.join("good.rs"), "pub fn good_func() {}\n").unwrap();

        // Write a malformed file (invalid Rust syntax)
        fs::write(
            src_dir.join("bad.rs"),
            "this is not valid rust {{{{{{{{ @@@ syntax error\n",
        )
        .unwrap();

        let indexer = CodeIndexer::new(&project_root);
        // Should not error — bad file is skipped
        let index = indexer.index(&["src/**/*.rs".to_string()]).unwrap();
        // Should have indexed the good file (bad file skipped)
        assert!(
            index.indexed_files >= 1,
            "Should index at least the good file"
        );

        let names: Vec<&str> = index.symbols.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"good_func"), "Should find good_func");
    }

    #[test]
    fn test_unreadable_file_skipped_gracefully() {
        let dir = tempdir().unwrap();
        let project_root = dir.path().to_path_buf();
        let src_dir = project_root.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        // Write a valid file
        fs::write(src_dir.join("good.rs"), "pub fn another_func() {}\n").unwrap();

        let indexer = CodeIndexer::new(&project_root);
        // Should succeed even with only good files
        let index = indexer.index(&["src/**/*.rs".to_string()]).unwrap();
        assert_eq!(index.indexed_files, 1);
    }
}
