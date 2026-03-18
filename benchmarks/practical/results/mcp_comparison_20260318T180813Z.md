# MCP Comparison Report

**Date**: 2026-03-18 18:08:13 UTC  
**Scenario**: File Processing Toolkit (file-processing-toolkit)

## Shared MCP Workflow

| Operation | original-metis (ms) | ultra-metis-mcp (ms) | Winner |
|-----------|---------------------:|----------------------:|--------|
| initialize_project | 15.72 | 3.70 | ultra-metis-mcp |
| create_vision | 0.00 | 21.80 | original-metis |
| create_initiative | 15.15 | 4.05 | ultra-metis-mcp |
| list_documents | 14.97 | 1.90 | ultra-metis-mcp |
| read_document | 15.48 | 0.48 | ultra-metis-mcp |
| search_documents | 9.68 | 1.24 | ultra-metis-mcp |

## Tool Surface

- original-metis shared tools: initialize_project, list_documents, search_documents, read_document, create_document, edit_document, transition_phase, archive_document, reassign_parent
- ultra-metis-mcp shared tools: initialize_project, create_document, read_document, list_documents, edit_document, transition_phase, search_documents, archive_document, index_code, reassign_parent
