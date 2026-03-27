---
name: cadre:cadre-setup
description: "This skill should be used when the user asks to 'initialize cadre', 'set up cadre project', 'cadre setup', 'create product doc for this project', 'guided setup', 'analyze this project', 'onboard this repo', 'bootstrap cadre', or needs guided post-initialization project setup."
---

# Cadre Setup

This skill guides users through post-initialization project setup, analyzing the project and creating foundational documents through a conversational flow.

## Prerequisites

The project must already have a Cadre workspace initialized (`.cadre/` directory exists). If not, initialize first:
```
mcp__plugin_cadre_cadre__initialize_project(project_path="/path/to/project/.cadre", prefix="PROJ")
```

## Step 1: Assess the Project

Call the analysis tool to understand what we're working with:

```
mcp__plugin_cadre_cadre__analyze_project(project_path="/path/to/project")
```

Examine the response:
- If the analysis indicates **"Brownfield"** (existing code detected) -> proceed to **Step 2A**
- If the analysis indicates **"Greenfield"** (no existing code) -> proceed to **Step 2B**

## Step 2A: Brownfield Path

For projects with existing code.

### 2A.1: Present Analysis

Present the bootstrap analysis to the user clearly:
- Detected languages and frameworks
- Project type (web app, CLI, library, etc.)
- Build tools and configuration found
- Dev tools and CI/CD setup detected

### 2A.2: Evaluate Brownfield Context

Call the brownfield evaluation tool with detected project characteristics:

```
mcp__plugin_cadre_cadre__evaluate_brownfield(language="<detected>", project_type="<detected>")
```

### 2A.3: Query Architecture Catalog

Search for matching architecture patterns:

```
mcp__plugin_cadre_cadre__query_architecture_catalog(query="<language> <project_type>")
```

### 2A.4: Present Findings

Summarize everything discovered:

> Based on my analysis, this is a **[project type]** using **[languages/frameworks]**. Here's what I found:
> - [Key structural observations]
> - [Build/dev tooling summary]
> - [Architecture pattern match, if any]

### 2A.5: Create ProductDoc

Ask the user:

> Let's create a ProductDoc to anchor all future planning. What is the product vision and purpose? What problem does it solve, and who are the target users?

Wait for the user's response. Then create the ProductDoc:

```
mcp__plugin_cadre_cadre__create_document(type="product_doc", title="<product name>")
```

### 2A.6: Populate ProductDoc

Immediately populate the ProductDoc with real content derived from the user's response and the code analysis. Do NOT leave template placeholders:

```
mcp__plugin_cadre_cadre__edit_document(short_code="<product_doc_short_code>", search="<template placeholder text>", replace="<real content>")
```

Include:
- Product intent and scope (from user input)
- Technical context (from code analysis)
- Target audience and key benefits (from user input)
- Current state summary (from brownfield evaluation)

### 2A.7: Offer ReferenceArchitecture (if applicable)

If the architecture catalog query returned a matching pattern, ask:

> I found a matching architecture pattern: **'[pattern name]'**. Would you like me to create a ReferenceArchitecture document from it? This will define the target structure and conformance rules for the project.

If confirmed:

```
mcp__plugin_cadre_cadre__create_document(type="reference_architecture", title="<architecture name>")
```

Then populate it with the catalog pattern details using `mcp__plugin_cadre_cadre__edit_document`.

### 2A.8: Offer Code Indexing and Quality Baseline

Ask the user:

> Would you like me to index the codebase and capture a quality baseline? This gives Cadre structural awareness of your code and establishes a starting point for tracking quality.

If confirmed, run both:

```
mcp__plugin_cadre_cadre__index_code(project_path="/path/to/project")
```

```
mcp__plugin_cadre_cadre__capture_quality_baseline(project_path="/path/to/project")
```

## Step 2B: Greenfield Path

For new projects with no existing code.

### 2B.1: Acknowledge Empty Project

Tell the user:

> This is a new project with no existing code. Let's define what you're building before anything else.

### 2B.2: Gather Product Vision

Ask:

> What problem is this project solving? Who are the target users?

Wait for the user's response before proceeding.

### 2B.3: Create and Populate ProductDoc

Use the user's response to create a ProductDoc:

```
mcp__plugin_cadre_cadre__create_document(type="product_doc", title="<product name>")
```

Immediately populate it with real content using `mcp__plugin_cadre_cadre__edit_document`:
- Product intent derived from the user's problem statement
- Target audience from the user's description
- Success criteria (ask if not provided)
- Scope boundaries

### 2B.4: Gather Technology Choices

Ask:

> What languages or frameworks are you planning to use? Any architectural preferences (monolith, microservices, serverless, etc.)?

### 2B.5: Find Matching Architecture Patterns

Using the user's technology choices, query for available patterns:

```
mcp__plugin_cadre_cadre__list_catalog_languages()
```

```
mcp__plugin_cadre_cadre__query_architecture_catalog(query="<language> <framework> <architecture style>")
```

### 2B.6: Offer ReferenceArchitecture (if applicable)

If a matching pattern was found, ask:

> I found an architecture pattern that matches your choices: **'[pattern name]'**. Would you like me to create a ReferenceArchitecture from it? This will give you a recommended project structure and conformance rules.

If confirmed, create and populate via `mcp__plugin_cadre_cadre__create_document` and `mcp__plugin_cadre_cadre__edit_document`.

## Step 3: Wrap Up

Summarize everything that was created:

> Here's what we set up:
> - **ProductDoc**: [short code] - [title]
> - **ReferenceArchitecture**: [short code] - [title] *(if created)*
> - **Code index**: *(if captured)*
> - **Quality baseline**: *(if captured)*

Then suggest next steps:

> Your project is set up. Next steps:
> - Create an **Epic** to start planning your first piece of work
> - Run `/cadre-setup` again anytime to update the baseline
> - Use `mcp__plugin_cadre_cadre__create_document(type="epic", parent_id="<product_doc_short_code>")` to get started

## Key Principles

- **Human-in-the-loop**: Always ASK before creating each document. Never create documents without user confirmation.
- **Present before prompting**: Show what you found, then ask for input. Don't ask questions you could answer from analysis.
- **Real content only**: Every document created must be populated with actual content. No template placeholders left behind.
- **Conversational flow**: Each step is a natural conversation beat. Don't rush through or dump everything at once.
- **Respect the hierarchy**: ProductDoc first, then supporting documents. Everything traces back to product intent.
