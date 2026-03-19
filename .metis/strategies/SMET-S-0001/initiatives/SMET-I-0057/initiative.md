---
id: persistence-search-implement-code
level: initiative
title: "Persistence & Search: Implement Code Indexing and SQLite FTS Backend"
short_code: "SMET-I-0057"
created_at: 2026-03-17T22:43:37.907549+00:00
updated_at: 2026-03-17T22:43:37.907549+00:00
parent: SMET-S-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: SMET-S-0001
initiative_id: persistence-search-implement-code
---

# Persistence & Search: Harden Code Indexing and Improve Search Performance

## Strategy Update (2026-03-18)

**Revised approach**: Scaled down significantly. The `index_code` MCP tool already exists with tree-sitter support (SMET-T-0120). Claude Code provides Grep/Glob for general codebase search. Full SQLite FTS5 migration is over-engineering for current needs.

**Key decisions:**
- Skip SQLite FTS5 migration — file-based search is sufficient for typical project sizes (< 500 docs)
- Skip database-backed relationships — parent-child queries work fine with file scan at current scale
- Focus on hardening existing `index_code` tool and improving `search_documents` for larger doc sets
- Use Claude Code's built-in Grep/Glob for code search — don't duplicate this in ultra-metis
- Re-evaluate SQLite backend only if projects regularly exceed 500+ documents

## Context

Ultra-Metis has a working `index_code` MCP tool (tree-sitter based, SMET-T-0120) and file-based `search_documents`. Claude Code provides powerful Grep/Glob tools for codebase search. The original XL scope (full SQLite FTS backend) is unnecessary given these capabilities.

**What exists:**
- `index_code` tool with tree-sitter symbol extraction (Rust, JS, Python, Go)
- `search_documents` with file-scan search
- Claude Code Grep/Glob for codebase search

**Remaining gaps:**
- `index_code` needs hardening (error handling, edge cases, larger codebases)
- `search_documents` could be faster for 200+ document projects
- No relationship queries (e.g., "all tasks under initiative X") — but this is low-priority

## Goals & Non-Goals

**Goals:**
- Harden `index_code` tool: better error handling, support for more file types, performance on larger codebases
- Improve `search_documents` performance for projects with 200+ documents (simple caching or indexing)
- Add parent-child filtering to `list_documents` (e.g., list all tasks under an initiative)
- Validate tree-sitter symbol extraction quality across supported languages

**Non-Goals:**
- Full SQLite FTS5 migration (over-engineering for current scale)
- Database-backed relationship schema (file scan is sufficient)
- Code-to-documentation cross-referencing (low priority)
- Real-time incremental indexing

## Requirements

### Database Migration
- REQ-001: SQLite database (.metis/metis.db) stores all documents and metadata
- REQ-002: FTS5 virtual table indexes document content for full-text search
- REQ-003: Relational schema with document, parent_child, cross_reference tables
- REQ-004: Lazy migration: documents remain in .metis/docs/ files until accessed/modified
- REQ-005: Query operations check both database (primary) and file system (fallback)

### Code Indexing (tree-sitter)
- REQ-006: Extract symbols from source files (functions, types, structs, traits, classes)
- REQ-007: Support multiple languages: Rust, JavaScript, Python, Go (configurable)
- REQ-008: Store symbols in code_symbols table with language, scope, file path
- REQ-009: Create code_to_doc cross-references linking symbols to documents

### Search Performance
- REQ-010: search_documents with FTS5 query returns results in O(log n) time
- REQ-011: FTS5 query sanitization escapes special chars (prevent injection)
- REQ-012: Support phrase queries ("exact phrase"), fuzzy matching, ranking
- REQ-013: Parent-child queries optimized via relational foreign keys

### Backward Compatibility
- REQ-014: Existing file-based workflow continues to work during transition
- REQ-015: Sync command migrates documents from files to database on demand
- REQ-016: Both file and database sources queried transparently

## Detailed Design

### Database Schema

```sql
-- Documents table
CREATE TABLE documents (
  id TEXT PRIMARY KEY,
  short_code TEXT UNIQUE NOT NULL,
  doc_type TEXT NOT NULL,  -- vision, initiative, task, etc.
  title TEXT NOT NULL,
  phase TEXT NOT NULL,
  parent_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  archived BOOLEAN DEFAULT FALSE,
  FOREIGN KEY(parent_id) REFERENCES documents(id)
);

-- Full-text search index
CREATE VIRTUAL TABLE documents_fts USING fts5(
  short_code,
  title,
  content,
  content=documents,
  content_rowid=rowid
);

-- Parent-child relationships (for fast hierarchy queries)
CREATE TABLE parent_child (
  parent_id TEXT NOT NULL,
  child_id TEXT NOT NULL,
  PRIMARY KEY(parent_id, child_id),
  FOREIGN KEY(parent_id) REFERENCES documents(id),
  FOREIGN KEY(child_id) REFERENCES documents(id)
);

-- Code symbol indexing
CREATE TABLE code_symbols (
  id INTEGER PRIMARY KEY,
  file_path TEXT NOT NULL,
  language TEXT NOT NULL,  -- rust, javascript, python, go
  symbol_name TEXT NOT NULL,
  symbol_type TEXT NOT NULL,  -- function, type, struct, trait, class
  scope TEXT,  -- module/namespace where symbol is defined
  line_number INTEGER,
  created_at TEXT NOT NULL
);

-- Cross-references between documents and code symbols
CREATE TABLE code_to_doc (
  document_id TEXT NOT NULL,
  symbol_id INTEGER NOT NULL,
  reference_type TEXT,  -- implements, uses, documents, etc.
  PRIMARY KEY(document_id, symbol_id),
  FOREIGN KEY(document_id) REFERENCES documents(id),
  FOREIGN KEY(symbol_id) REFERENCES code_symbols(id)
);
```

### Code Indexing (tree-sitter Integration)

- Create CodeIndexer struct that uses tree-sitter for each supported language
- For each source file, extract symbols using language-specific queries
- Store results in code_symbols table
- Tie symbols to documents via code_to_doc table based on filename/module references

### Search Implementation

1. **search_documents with FTS5**:
   - Parse query, sanitize special chars
   - SELECT * FROM documents_fts WHERE documents_fts MATCH ?
   - Return results ranked by FTS5 relevance
   
2. **Fast parent queries**:
   - SELECT * FROM documents WHERE parent_id = ? (via foreign key index)
   
3. **Symbol search**:
   - SELECT * FROM code_symbols WHERE symbol_name LIKE ? OR symbol_type = ?
   - Join with code_to_doc to find related documents

### Lazy Migration Strategy

- DocumentStore::load checks database first; if not found, loads from file and caches in DB
- sync command explicitly migrates all files to database
- Both sources queried transparently during transition period

## Testing Strategy

### Unit Tests
- Database schema validation (tables created correctly)
- FTS5 query parsing and sanitization
- tree-sitter symbol extraction for each language
- Lazy migration logic (file vs DB precedence)
- Code-to-doc cross-reference creation

### Integration Tests
- Full database migration: file → DB via sync command
- Search performance: FTS5 vs file scan (1000+ docs)
- Code indexing end-to-end: source files → symbols → docs
- Backward compatibility: old file-based operations still work
- Lazy migration: seamless switch between file and DB sources

### Performance Tests
- search_documents with 10,000+ documents (target: <100ms FTS5 vs 5s+ file scan)
- Code indexing on large codebase (10,000+ symbols)
- Database size growth and index maintenance

## Alternatives Considered

**Alternative 1: Immediate hard migration (all files → DB at once)**
- Rejected: Risk of data loss if migration fails; backward compatibility broken
- Lazy migration safer: can always fall back to files

**Alternative 2: Keep file-only, optimize file search**
- Rejected: No optimization can match O(log n) FTS5 performance
- Would still lack code indexing and relationship queries

**Selected Approach: SQLite backend with lazy migration + tree-sitter code indexing**
- Database provides FTS5 and relational capabilities
- Lazy migration minimizes disruption
- tree-sitter enables multi-language code analysis
- Transparent fallback to file system during transition

## Implementation Plan

### Phase 1: SQLite Backend & Schema (Critical path)
- Add rusqlite or sqlx dependency
- Create database initialization logic
- Implement documents table with FTS5 virtual table
- Add parent_child and cross_reference tables
- Migrate search_documents to use FTS5
- Unit and integration tests

### Phase 2: Lazy Migration & Transparent Querying
- Implement dual-source query logic (DB first, files second)
- Add document caching in DB on access
- Update sync command to migrate files to DB
- Ensure backward compatibility during transition
- Integration tests with mixed file/DB sources

### Phase 3: tree-sitter Code Indexing
- Add tree-sitter dependency (rust, javascript, python, go)
- Implement CodeIndexer struct with language-specific extractors
- Create code_symbols and code_to_doc tables
- Implement symbol search and code-to-doc cross-referencing
- Integration tests with real source files

### Phase 4: Performance Validation & Optimization
- Benchmark FTS5 vs file scan with 1000+ documents
- Profile code indexing performance
- Add indexes on frequently queried columns
- Optimize query plans for parent-child lookups

### Exit Criteria
- SQLite database fully functional with FTS5 search
- Lazy migration working (files auto-move to DB on access)
- Code indexing extracting symbols from source files
- Search performance meets O(log n) target
- Backward compatibility maintained with existing file-based workflows
- All integration tests passing
- Performance benchmarks documented