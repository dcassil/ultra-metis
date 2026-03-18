# MCP Comparison Report

**Date**: 2026-03-18 18:00:31 UTC  
**Scenario**: File Processing Toolkit (file-processing-toolkit)

## Shared MCP Workflow

| Operation | original-metis (ms) | ultra-metis-mcp (ms) | Winner |
|-----------|---------------------:|----------------------:|--------|
| initialize_project | 9.91 | 1.18 | ultra-metis-mcp |
| create_vision | 5.84 | 2.80 | ultra-metis-mcp |
| create_initiative | 4.66 | 1.11 | ultra-metis-mcp |
| list_documents | 4.56 | 0.21 | ultra-metis-mcp |
| read_document | 8.18 | 0.10 | ultra-metis-mcp |
| search_documents | 4.59 | 0.10 | ultra-metis-mcp |

## Tool Surface

- original-metis shared tools: initialize_project, list_documents, search_documents, read_document, create_document, edit_document, transition_phase, archive_document, reassign_parent
- ultra-metis-mcp shared tools: initialize_project, create_document, read_document, list_documents, edit_document, transition_phase, search_documents, archive_document, index_code, reassign_parent
