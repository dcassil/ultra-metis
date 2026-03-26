//! Cadre CLI
//!
//! Command-line interface for Cadre document management.

use cadre_core::domain::rules::query::{scope_hierarchy, scope_rank, RuleQuery, RuleQueryEngine};
use cadre_core::{
    BaselineCaptureService, ClippyParser, CoverageParser, CrossReference, DurableInsightNote,
    EslintParser, FeedbackSignal, InsightCategory, InsightScope, NoteStatus, RelationshipType,
    RulesConfig, ToolOutputParser, TraceabilityIndex, TypeScriptParser,
};
use cadre_store::DocumentStore;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "cadre")]
#[command(about = "Cadre: Repo-Native AI Engineering Orchestration")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the MCP server for external integrations (e.g., Claude Code)
    Mcp,

    /// Initialize a new Cadre project
    Init {
        /// Project root directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Short code prefix (e.g., PROJ)
        #[arg(short = 'x', long)]
        prefix: String,
    },

    /// List all documents
    List {
        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,

        /// Include archived documents
        #[arg(long)]
        include_archived: bool,

        /// Filter to children of this parent document
        #[arg(long)]
        parent: Option<String>,
    },

    /// Read a document by short code
    Read {
        /// Document short code (e.g., PROJ-V-0001)
        short_code: String,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Create a new document
    Create {
        /// Document type: vision, initiative, task
        #[arg(short = 't', long = "type")]
        doc_type: String,

        /// Document title
        title: String,

        /// Parent document short code
        #[arg(short = 'P', long)]
        parent: Option<String>,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Edit a document using search and replace
    Edit {
        /// Document short code
        short_code: String,

        /// Text to search for
        #[arg(short, long)]
        search: String,

        /// Replacement text
        #[arg(short, long)]
        replace: String,

        /// Replace all occurrences (default: first only)
        #[arg(long)]
        replace_all: bool,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Transition a document to the next phase
    Transition {
        /// Document short code
        short_code: String,

        /// Target phase (optional, auto-advances if omitted)
        #[arg(long)]
        phase: Option<String>,

        /// Force transition, bypassing exit criteria validation
        #[arg(long)]
        force: bool,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Search documents by text query
    Search {
        /// Search query
        query: String,

        /// Filter by document type (vision, initiative, task, etc.)
        #[arg(short = 't', long = "type")]
        doc_type: Option<String>,

        /// Maximum number of results
        #[arg(short, long)]
        limit: Option<usize>,

        /// Include archived documents in results
        #[arg(long)]
        include_archived: bool,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Archive a document
    Archive {
        /// Document short code
        short_code: String,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Reassign a task to a different parent or backlog
    Reassign {
        /// Task short code to reassign
        short_code: String,

        /// Target parent initiative short code
        #[arg(long)]
        parent: Option<String>,

        /// Move to backlog with category (bug, feature, tech-debt)
        #[arg(long)]
        backlog: Option<String>,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Show project status dashboard
    Status {
        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Validate document integrity
    Validate {
        /// Document short code (omit for --all)
        short_code: Option<String>,

        /// Validate all documents in the project
        #[arg(long)]
        all: bool,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Quality baseline and comparison operations
    Quality {
        #[command(subcommand)]
        command: QualityCommands,
    },

    /// Engineering rules management
    Rules {
        #[command(subcommand)]
        command: RulesCommands,
    },

    /// Durable insight notes
    Notes {
        #[command(subcommand)]
        command: NotesCommands,
    },

    /// Traceability and cross-references
    Trace {
        #[command(subcommand)]
        command: TraceCommands,
    },
}

// ---------------------------------------------------------------------------
// Quality subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
enum QualityCommands {
    /// Capture a quality baseline from tool output
    Capture {
        /// Tool name: eslint, clippy, tsc, coverage
        #[arg(long)]
        tool: String,

        /// Raw tool output to parse
        #[arg(long)]
        output: String,

        /// Short code of linked rules config (optional)
        #[arg(long)]
        linked_rules: Option<String>,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Compare two quality baselines
    Compare {
        /// Short code of the "before" baseline
        #[arg(long)]
        before: String,

        /// Short code of the "after" baseline
        #[arg(long)]
        after: String,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// List quality records and analysis baselines
    List {
        /// Filter by status: pass, warn, fail
        #[arg(long)]
        status: Option<String>,

        /// Maximum number of results
        #[arg(long)]
        limit: Option<usize>,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
}

// ---------------------------------------------------------------------------
// Rules subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
enum RulesCommands {
    /// Query rules by scope, protection level, and source architecture
    Query {
        /// Rule scope: platform, org, repo, package, component, task
        #[arg(long)]
        scope: Option<String>,

        /// Protection level: standard, protected
        #[arg(long)]
        protection_level: Option<String>,

        /// Source architecture short code
        #[arg(long)]
        source_arch: Option<String>,

        /// Include archived rules
        #[arg(long)]
        include_archived: bool,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Get rules applicable at a scope (via inheritance)
    Applicable {
        /// Target scope: platform, org, repo, package, component, task
        #[arg(long)]
        scope: String,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// List all protected rules
    Protected {
        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
}

// ---------------------------------------------------------------------------
// Notes subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
enum NotesCommands {
    /// Create a new insight note
    Create {
        /// Note title
        #[arg(long)]
        title: String,

        /// Note text
        #[arg(long)]
        note: String,

        /// Category of insight
        #[arg(long)]
        category: String,

        /// Scope: repository name
        #[arg(long)]
        scope_repo: Option<String>,

        /// Scope: package name
        #[arg(long)]
        scope_package: Option<String>,

        /// Scope: subsystem name
        #[arg(long)]
        scope_subsystem: Option<String>,

        /// Scope: comma-separated file paths
        #[arg(long, value_delimiter = ',')]
        scope_paths: Option<Vec<String>>,

        /// Scope: comma-separated symbols
        #[arg(long, value_delimiter = ',')]
        scope_symbols: Option<Vec<String>>,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Fetch insight notes by scope
    Fetch {
        /// Scope: repository name
        #[arg(long)]
        scope_repo: Option<String>,

        /// Scope: package name
        #[arg(long)]
        scope_package: Option<String>,

        /// Scope: subsystem name
        #[arg(long)]
        scope_subsystem: Option<String>,

        /// Scope: comma-separated file paths
        #[arg(long, value_delimiter = ',')]
        scope_paths: Option<Vec<String>>,

        /// Scope: comma-separated symbols
        #[arg(long, value_delimiter = ',')]
        scope_symbols: Option<Vec<String>>,

        /// Maximum number of results
        #[arg(long)]
        limit: Option<usize>,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Score an insight note with feedback
    Score {
        /// Note short code
        short_code: String,

        /// Feedback signal: helpful, meh, harmful
        #[arg(long)]
        signal: String,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// List insight notes with optional filters
    List {
        /// Filter by status: active, prune_candidate, needs_human_review, archived
        #[arg(long)]
        status: Option<String>,

        /// Filter by category
        #[arg(long)]
        category: Option<String>,

        /// Include archived notes
        #[arg(long)]
        include_archived: bool,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
}

// ---------------------------------------------------------------------------
// Trace subcommands
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
enum TraceCommands {
    /// Create a cross-reference between two documents
    Create {
        /// Source document short code
        #[arg(long)]
        source: String,

        /// Target document short code
        #[arg(long)]
        target: String,

        /// Relationship type
        #[arg(long = "type")]
        rel_type: String,

        /// Description of the relationship
        #[arg(long)]
        description: Option<String>,

        /// Whether the relationship is bidirectional
        #[arg(long)]
        bidirectional: bool,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Query relationships for a document
    Query {
        /// Document short code to query
        short_code: String,

        /// Direction: outgoing, incoming, all
        #[arg(long, default_value = "all")]
        direction: String,

        /// Filter by relationship type
        #[arg(long = "type")]
        rel_type: Option<String>,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Trace ancestry of a document
    Ancestry {
        /// Document short code
        short_code: String,

        /// Direction: ancestors, descendants, siblings
        #[arg(long)]
        direction: String,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// List cross-references
    List {
        /// Filter by relationship type
        #[arg(long = "type")]
        rel_type: Option<String>,

        /// Filter to references involving this short code
        #[arg(long)]
        involving: Option<String>,

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Handle MCP subcommand separately (async, different error type)
    if matches!(cli.command, Commands::Mcp) {
        if let Err(e) = cadre_mcp_server::run().await {
            eprintln!("MCP server error: {}", e);
            std::process::exit(1);
        }
        return;
    }

    let result = match cli.command {
        Commands::Mcp => unreachable!(),
        Commands::Init { path, prefix } => cmd_init(&path, &prefix),
        Commands::List {
            path,
            include_archived,
            parent,
        } => cmd_list(&path, include_archived, parent.as_deref()),
        Commands::Read { short_code, path } => cmd_read(&path, &short_code),
        Commands::Create {
            doc_type,
            title,
            parent,
            path,
        } => cmd_create(&path, &doc_type, &title, parent.as_deref()),
        Commands::Edit {
            short_code,
            search,
            replace,
            replace_all,
            path,
        } => cmd_edit(&path, &short_code, &search, &replace, replace_all),
        Commands::Transition {
            short_code,
            phase,
            force,
            path,
        } => cmd_transition(&path, &short_code, phase.as_deref(), force),
        Commands::Search {
            query,
            doc_type,
            limit,
            include_archived,
            path,
        } => cmd_search(&path, &query, doc_type.as_deref(), limit, include_archived),
        Commands::Archive { short_code, path } => cmd_archive(&path, &short_code),
        Commands::Reassign {
            short_code,
            parent,
            backlog,
            path,
        } => cmd_reassign(&path, &short_code, parent.as_deref(), backlog.as_deref()),
        Commands::Status { path } => cmd_status(&path),
        Commands::Validate {
            short_code,
            all,
            path,
        } => cmd_validate(&path, short_code.as_deref(), all),
        Commands::Quality { command } => match command {
            QualityCommands::Capture {
                tool,
                output,
                linked_rules,
                path,
            } => cmd_quality_capture(&path, &tool, &output, linked_rules.as_deref()),
            QualityCommands::Compare {
                before,
                after,
                path,
            } => cmd_quality_compare(&path, &before, &after),
            QualityCommands::List {
                status,
                limit,
                path,
            } => cmd_quality_list(&path, status.as_deref(), limit),
        },
        Commands::Rules { command } => match command {
            RulesCommands::Query {
                scope,
                protection_level,
                source_arch,
                include_archived,
                path,
            } => cmd_rules_query(
                &path,
                scope.as_deref(),
                protection_level.as_deref(),
                source_arch.as_deref(),
                include_archived,
            ),
            RulesCommands::Applicable { scope, path } => cmd_rules_applicable(&path, &scope),
            RulesCommands::Protected { path } => cmd_rules_protected(&path),
        },
        Commands::Notes { command } => match command {
            NotesCommands::Create {
                title,
                note,
                category,
                scope_repo,
                scope_package,
                scope_subsystem,
                scope_paths,
                scope_symbols,
                path,
            } => cmd_notes_create(
                &path,
                &title,
                &note,
                &category,
                scope_repo.as_deref(),
                scope_package.as_deref(),
                scope_subsystem.as_deref(),
                scope_paths.as_deref(),
                scope_symbols.as_deref(),
            ),
            NotesCommands::Fetch {
                scope_repo,
                scope_package,
                scope_subsystem,
                scope_paths,
                scope_symbols,
                limit,
                path,
            } => cmd_notes_fetch(
                &path,
                scope_repo.as_deref(),
                scope_package.as_deref(),
                scope_subsystem.as_deref(),
                scope_paths.as_deref(),
                scope_symbols.as_deref(),
                limit,
            ),
            NotesCommands::Score {
                short_code,
                signal,
                path,
            } => cmd_notes_score(&path, &short_code, &signal),
            NotesCommands::List {
                status,
                category,
                include_archived,
                path,
            } => cmd_notes_list(
                &path,
                status.as_deref(),
                category.as_deref(),
                include_archived,
            ),
        },
        Commands::Trace { command } => match command {
            TraceCommands::Create {
                source,
                target,
                rel_type,
                description,
                bidirectional,
                path,
            } => cmd_trace_create(
                &path,
                &source,
                &target,
                &rel_type,
                description.as_deref(),
                bidirectional,
            ),
            TraceCommands::Query {
                short_code,
                direction,
                rel_type,
                path,
            } => cmd_trace_query(&path, &short_code, &direction, rel_type.as_deref()),
            TraceCommands::Ancestry {
                short_code,
                direction,
                path,
            } => cmd_trace_ancestry(&path, &short_code, &direction),
            TraceCommands::List {
                rel_type,
                involving,
                path,
            } => cmd_trace_list(&path, rel_type.as_deref(), involving.as_deref()),
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

// ===========================================================================
// Existing command implementations
// ===========================================================================

fn cmd_init(path: &Path, prefix: &str) -> Result<(), String> {
    let store = DocumentStore::new(path);
    store.initialize(prefix).map_err(|e| e.user_message())?;
    println!(
        "Initialized Cadre project at {} with prefix {}",
        path.display(),
        prefix
    );
    Ok(())
}

fn cmd_list(path: &Path, include_archived: bool, parent: Option<&str>) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let docs = store
        .list_documents_with_options(include_archived, parent)
        .map_err(|e| e.user_message())?;

    if docs.is_empty() {
        println!("No documents found.");
        return Ok(());
    }

    println!(
        "{:<16} {:<40} {:<12} {:<12} {:<16}",
        "SHORT CODE", "TITLE", "TYPE", "PHASE", "PARENT"
    );
    println!("{}", "-".repeat(96));
    for doc in &docs {
        let parent = doc.parent_id.as_deref().unwrap_or("-");
        println!(
            "{:<16} {:<40} {:<12} {:<12} {:<16}",
            doc.short_code, doc.title, doc.document_type, doc.phase, parent
        );
    }
    Ok(())
}

fn cmd_read(path: &Path, short_code: &str) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let raw = store
        .read_document_raw(short_code)
        .map_err(|e| e.user_message())?;
    println!("{}", raw);
    Ok(())
}

fn cmd_create(
    path: &Path,
    doc_type: &str,
    title: &str,
    parent: Option<&str>,
) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let short_code = store
        .create_document(doc_type, title, parent)
        .map_err(|e| e.user_message())?;
    println!("Created {} ({}: {})", short_code, doc_type, title);
    Ok(())
}

fn cmd_edit(
    path: &Path,
    short_code: &str,
    search: &str,
    replace: &str,
    replace_all: bool,
) -> Result<(), String> {
    let store = DocumentStore::new(path);
    store
        .edit_document_with_options(short_code, search, replace, replace_all)
        .map_err(|e| e.user_message())?;
    let mode = if replace_all {
        " (all occurrences)"
    } else {
        ""
    };
    println!("Updated {}{}", short_code, mode);
    Ok(())
}

fn cmd_transition(
    path: &Path,
    short_code: &str,
    phase: Option<&str>,
    force: bool,
) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let result = store
        .transition_phase_with_options(short_code, phase, force)
        .map_err(|e| e.user_message())?;
    let force_note = if force { " (forced)" } else { "" };
    println!("{}: {}{}", short_code, result, force_note);
    Ok(())
}

fn cmd_search(
    path: &Path,
    query: &str,
    doc_type: Option<&str>,
    limit: Option<usize>,
    include_archived: bool,
) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let docs = store
        .search_documents_with_options(query, doc_type, limit, include_archived)
        .map_err(|e| e.user_message())?;

    if docs.is_empty() {
        println!("No documents matching '{}'", query);
        return Ok(());
    }

    println!(
        "{:<16} {:<40} {:<12} {:<12}",
        "SHORT CODE", "TITLE", "TYPE", "PHASE"
    );
    println!("{}", "-".repeat(80));
    for doc in &docs {
        println!(
            "{:<16} {:<40} {:<12} {:<12}",
            doc.short_code, doc.title, doc.document_type, doc.phase
        );
    }
    Ok(())
}

fn cmd_archive(path: &Path, short_code: &str) -> Result<(), String> {
    let store = DocumentStore::new(path);
    store
        .archive_document(short_code)
        .map_err(|e| e.user_message())?;
    println!("Archived {}", short_code);
    Ok(())
}

fn cmd_reassign(
    path: &Path,
    short_code: &str,
    parent: Option<&str>,
    backlog: Option<&str>,
) -> Result<(), String> {
    if parent.is_none() && backlog.is_none() {
        return Err("Must specify either --parent <id> or --backlog <category>".to_string());
    }

    let store = DocumentStore::new(path);
    let result = store
        .reassign_parent(short_code, parent, backlog)
        .map_err(|e| e.user_message())?;
    println!("{}", result);
    Ok(())
}

fn cmd_status(path: &Path) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let docs = store.list_documents(false).map_err(|e| e.user_message())?;

    if docs.is_empty() {
        println!("No documents found.");
        return Ok(());
    }

    // Aggregate counts
    let mut type_counts: HashMap<String, usize> = HashMap::new();
    let mut phase_counts: HashMap<String, usize> = HashMap::new();
    let mut initiatives = Vec::new();
    let mut active_tasks = Vec::new();

    for doc in &docs {
        *type_counts.entry(doc.document_type.clone()).or_insert(0) += 1;
        *phase_counts.entry(doc.phase.clone()).or_insert(0) += 1;

        if doc.document_type == "initiative" && doc.phase != "completed" {
            initiatives.push(doc);
        }
        if doc.document_type == "task" && (doc.phase == "active" || doc.phase == "todo") {
            active_tasks.push(doc);
        }
    }

    // Count child tasks per initiative
    let task_docs: Vec<_> = docs.iter().filter(|d| d.document_type == "task").collect();

    // Initiatives section
    if !initiatives.is_empty() {
        println!("INITIATIVES");
        println!(
            "{:<16} {:<36} {:<12} {:>5} {:>5} {:>5}",
            "CODE", "TITLE", "PHASE", "TODO", "ACT", "DONE"
        );
        println!("{}", "-".repeat(85));
        for init in &initiatives {
            let todo = task_docs
                .iter()
                .filter(|t| t.parent_id.as_deref() == Some(&init.short_code) && t.phase == "todo")
                .count();
            let active = task_docs
                .iter()
                .filter(|t| t.parent_id.as_deref() == Some(&init.short_code) && t.phase == "active")
                .count();
            let done = task_docs
                .iter()
                .filter(|t| {
                    t.parent_id.as_deref() == Some(&init.short_code) && t.phase == "completed"
                })
                .count();
            let title_trunc = if init.title.len() > 35 {
                format!("{}...", &init.title[..32])
            } else {
                init.title.clone()
            };
            println!(
                "{:<16} {:<36} {:<12} {:>5} {:>5} {:>5}",
                init.short_code, title_trunc, init.phase, todo, active, done
            );
        }
        println!();
    }

    // Active tasks section
    if !active_tasks.is_empty() {
        println!("ACTIVE TASKS");
        println!(
            "{:<16} {:<40} {:<16} {:<8}",
            "CODE", "TITLE", "PARENT", "PHASE"
        );
        println!("{}", "-".repeat(80));
        for task in &active_tasks {
            let parent = task.parent_id.as_deref().unwrap_or("-");
            let title_trunc = if task.title.len() > 39 {
                format!("{}...", &task.title[..36])
            } else {
                task.title.clone()
            };
            println!(
                "{:<16} {:<40} {:<16} {:<8}",
                task.short_code, title_trunc, parent, task.phase
            );
        }
        println!();
    }

    // Summary
    println!("SUMMARY");
    let total = docs.len();
    let vision_count = type_counts.get("vision").unwrap_or(&0);
    let init_count = type_counts.get("initiative").unwrap_or(&0);
    let task_count = type_counts.get("task").unwrap_or(&0);
    let todo = phase_counts.get("todo").unwrap_or(&0);
    let active = phase_counts.get("active").unwrap_or(&0);
    let completed = phase_counts.get("completed").unwrap_or(&0);
    println!(
        "{} documents: {} visions, {} initiatives, {} tasks",
        total, vision_count, init_count, task_count
    );
    println!(
        "Phases: {} todo, {} active, {} completed",
        todo, active, completed
    );

    Ok(())
}

fn cmd_validate(path: &Path, short_code: Option<&str>, all: bool) -> Result<(), String> {
    if short_code.is_none() && !all {
        return Err("Specify a short code or use --all to validate all documents".to_string());
    }

    let store = DocumentStore::new(path);

    let codes_to_validate: Vec<String> = if all {
        let docs = store.list_documents(true).map_err(|e| e.user_message())?;
        docs.iter().map(|d| d.short_code.clone()).collect()
    } else {
        vec![short_code.unwrap().to_string()]
    };

    if codes_to_validate.is_empty() {
        println!("No documents to validate.");
        return Ok(());
    }

    // Build a set of all known short codes for cross-reference validation
    let all_docs = store.list_documents(true).map_err(|e| e.user_message())?;
    let known_codes: std::collections::HashSet<String> =
        all_docs.iter().map(|d| d.short_code.clone()).collect();

    let mut issues: Vec<(String, String, &str)> = Vec::new(); // (short_code, message, severity)
    let mut ok_count = 0;

    for code in &codes_to_validate {
        let mut doc_issues = Vec::new();

        // Check 1: Document parses correctly
        match store.read_document(code) {
            Ok(doc) => {
                // Check 2: Phase is valid
                if let Err(e) = doc.phase() {
                    doc_issues.push((code.clone(), format!("Invalid phase: {}", e), "ERROR"));
                }

                // Check 3: Parent cross-reference
                if let Some(parent_id) = doc.parent_id() {
                    let parent_str = parent_id.clone();
                    if parent_str != "NULL" && !known_codes.contains(&parent_str) {
                        doc_issues.push((
                            code.clone(),
                            format!("Parent '{}' not found", parent_str),
                            "ERROR",
                        ));
                    }
                }
            }
            Err(e) => {
                doc_issues.push((code.clone(), format!("Parse error: {}", e), "ERROR"));
            }
        }

        if doc_issues.is_empty() {
            ok_count += 1;
        } else {
            issues.extend(doc_issues);
        }
    }

    // Output results
    if issues.is_empty() {
        println!("All {} documents valid.", ok_count);
        return Ok(());
    }

    println!(
        "{:<8} {:<16} {:<50} {:<8}",
        "STATUS", "CODE", "ISSUE", "SEVERITY"
    );
    println!("{}", "-".repeat(82));

    for (code, message, severity) in &issues {
        println!("{:<8} {:<16} {:<50} {:<8}", "FAIL", code, message, severity);
    }

    println!();
    println!(
        "{} passed, {} issues found in {} documents",
        ok_count,
        issues.len(),
        codes_to_validate.len()
    );

    // Exit code 1 if any errors
    if issues.iter().any(|(_, _, s)| *s == "ERROR") {
        std::process::exit(1);
    }

    Ok(())
}

// ===========================================================================
// Quality command implementations
// ===========================================================================

fn cmd_quality_capture(
    path: &Path,
    tool_name: &str,
    raw_output: &str,
    linked_rules: Option<&str>,
) -> Result<(), String> {
    // Parse the raw output using the appropriate parser
    let parsed = match tool_name.to_lowercase().as_str() {
        "eslint" => EslintParser.parse(raw_output).map_err(|e| e.to_string())?,
        "clippy" => ClippyParser.parse(raw_output).map_err(|e| e.to_string())?,
        "tsc" | "typescript" => TypeScriptParser
            .parse(raw_output)
            .map_err(|e| e.to_string())?,
        "coverage" => CoverageParser
            .parse(raw_output)
            .map_err(|e| e.to_string())?,
        _ => {
            return Err(format!(
                "Unknown tool: {}. Supported: eslint, clippy, tsc, coverage",
                tool_name
            ))
        }
    };

    let store = DocumentStore::new(path);
    let short_code = store
        .create_document(
            "analysis_baseline",
            &format!("{} Baseline", tool_name),
            None,
        )
        .map_err(|e| e.user_message())?;

    // Capture into a proper baseline and overwrite the file
    let baseline =
        BaselineCaptureService::capture(&parsed, &short_code, linked_rules.map(|s| s.to_string()))
            .map_err(|e| e.to_string())?;

    let content = baseline.to_content().map_err(|e| e.to_string())?;
    let doc_path = path
        .join(".cadre")
        .join("docs")
        .join(format!("{}.md", short_code));
    std::fs::write(&doc_path, content).map_err(|e| format!("Failed to write baseline: {}", e))?;

    println!("{:<18} {}", "Short Code:", short_code);
    println!("{:<18} {}", "Tool:", parsed.tool_name);
    println!("{:<18} {}", "Total Findings:", parsed.total_findings());
    println!("{:<18} {}", "Errors:", parsed.error_count());
    println!("{:<18} {}", "Warnings:", parsed.warning_count());
    println!("{:<18} {}", "Metrics:", parsed.metrics.len());
    Ok(())
}

fn cmd_quality_compare(path: &Path, before_sc: &str, after_sc: &str) -> Result<(), String> {
    let store = DocumentStore::new(path);

    // Read both baselines
    let before_raw = store
        .read_document_raw(before_sc)
        .map_err(|e| e.user_message())?;
    let after_raw = store
        .read_document_raw(after_sc)
        .map_err(|e| e.user_message())?;

    // Extract tool name from baseline content
    let before_tool = extract_tool_from_baseline(&before_raw)
        .ok_or_else(|| "Could not determine tool from 'before' baseline".to_string())?;
    let after_tool = extract_tool_from_baseline(&after_raw)
        .ok_or_else(|| "Could not determine tool from 'after' baseline".to_string())?;

    if before_tool != after_tool {
        return Err(format!(
            "Cannot compare baselines from different tools: '{}' vs '{}'",
            before_tool, after_tool
        ));
    }

    // Create a QualityRecord
    let qr_code = store
        .create_document(
            "quality_record",
            &format!("{} Comparison: {} vs {}", before_tool, before_sc, after_sc),
            None,
        )
        .map_err(|e| e.user_message())?;

    println!("{:<18} {}", "Before:", before_sc);
    println!("{:<18} {}", "After:", after_sc);
    println!("{:<18} {}", "Tool:", before_tool);
    println!("{:<18} {}", "Record:", qr_code);
    println!(
        "\nComparison record created. Edit {} to add detailed metric deltas.",
        qr_code
    );
    Ok(())
}

fn cmd_quality_list(
    path: &Path,
    status_filter: Option<&str>,
    limit: Option<usize>,
) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let docs = store
        .search_documents_with_options("quality_record", Some("quality_record"), None, false)
        .map_err(|e| e.user_message())?;

    let baselines = store
        .search_documents_with_options("analysis_baseline", Some("analysis_baseline"), None, false)
        .map_err(|e| e.user_message())?;

    let mut results = Vec::new();
    for doc in &docs {
        if let Some(filter) = status_filter {
            let raw = store.read_document_raw(&doc.short_code).unwrap_or_default();
            if !raw
                .to_lowercase()
                .contains(&format!("overall_status: {}", filter.to_lowercase()))
            {
                continue;
            }
        }
        results.push(doc);
    }

    if let Some(lim) = limit {
        results.truncate(lim);
    }

    if !results.is_empty() {
        println!("QUALITY RECORDS ({})", results.len());
        println!("{:<16} {:<40} {:<12}", "SHORT CODE", "TITLE", "PHASE");
        println!("{}", "-".repeat(68));
        for doc in &results {
            println!("{:<16} {:<40} {:<12}", doc.short_code, doc.title, doc.phase);
        }
        println!();
    }

    if !baselines.is_empty() {
        println!("ANALYSIS BASELINES ({})", baselines.len());
        println!("{:<16} {:<40} {:<12}", "SHORT CODE", "TITLE", "PHASE");
        println!("{}", "-".repeat(68));
        for doc in &baselines {
            println!("{:<16} {:<40} {:<12}", doc.short_code, doc.title, doc.phase);
        }
    }

    if results.is_empty() && baselines.is_empty() {
        println!("No quality records or baselines found.");
    }

    Ok(())
}

/// Extract tool name from a baseline's markdown content
fn extract_tool_from_baseline(content: &str) -> Option<String> {
    for line in content.lines() {
        if line.contains("**Tool**:") || line.contains("- **Tool**:") {
            let tool = line.split(':').nth(1)?.trim().to_string();
            return Some(tool);
        }
    }
    // Fallback: check the title
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("title:") {
            let title = trimmed.strip_prefix("title:")?.trim().trim_matches('"');
            if title.to_lowercase().contains("clippy") {
                return Some("clippy".to_string());
            } else if title.to_lowercase().contains("eslint") {
                return Some("eslint".to_string());
            } else if title.to_lowercase().contains("tsc")
                || title.to_lowercase().contains("typescript")
            {
                return Some("tsc".to_string());
            } else if title.to_lowercase().contains("coverage") {
                return Some("coverage".to_string());
            }
        }
    }
    None
}

// ===========================================================================
// Rules command implementations
// ===========================================================================

/// Helper to load all RulesConfig documents from the store
fn load_all_rules_configs(store: &DocumentStore) -> Result<Vec<RulesConfig>, String> {
    let docs = store
        .search_documents_with_options("rules_config", Some("rules_config"), None, false)
        .map_err(|e| e.user_message())?;

    let mut rules = Vec::new();
    for doc in &docs {
        let raw = store
            .read_document_raw(&doc.short_code)
            .map_err(|e| e.user_message())?;
        if let Ok(rc) = RulesConfig::from_content(&raw) {
            rules.push(rc);
        }
    }
    Ok(rules)
}

fn cmd_rules_query(
    path: &Path,
    scope_str: Option<&str>,
    protection_str: Option<&str>,
    arch_ref: Option<&str>,
    include_archived: bool,
) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let rules_configs = load_all_rules_configs(&store)?;
    let refs: Vec<&RulesConfig> = rules_configs.iter().collect();
    let engine = RuleQueryEngine::new(&refs);

    let mut query = RuleQuery::new();
    if let Some(s) = scope_str {
        let scope = s
            .parse()
            .map_err(|e: cadre_core::DocumentValidationError| e.to_string())?;
        query = query.with_scope(scope);
    }
    if let Some(p) = protection_str {
        let level = p
            .parse()
            .map_err(|e: cadre_core::DocumentValidationError| e.to_string())?;
        query = query.with_protection_level(level);
    }
    if let Some(a) = arch_ref {
        query = query.with_source_architecture_ref(a);
    }
    if include_archived {
        query = query.including_archived();
    }

    let results = engine.query(&query);

    if results.is_empty() {
        println!("No rules matching the query.");
        return Ok(());
    }

    println!("RULES QUERY RESULTS ({})", results.len());
    println!(
        "{:<16} {:<30} {:<12} {:<12} {:<10}",
        "SHORT CODE", "TITLE", "SCOPE", "PROTECTION", "PHASE"
    );
    println!("{}", "-".repeat(80));
    for rule in &results {
        let phase = rule
            .phase()
            .map(|p| p.to_string())
            .unwrap_or_else(|_| "?".to_string());
        let title_trunc = if rule.title().len() > 29 {
            format!("{}...", &rule.title()[..26])
        } else {
            rule.title().to_string()
        };
        println!(
            "{:<16} {:<30} {:<12} {:<12} {:<10}",
            rule.metadata().short_code,
            title_trunc,
            rule.scope,
            rule.protection_level,
            phase
        );
    }
    Ok(())
}

fn cmd_rules_applicable(path: &Path, target_scope_str: &str) -> Result<(), String> {
    let target_scope: cadre_core::domain::documents::rules_config::RuleScope = target_scope_str
        .parse()
        .map_err(|e: cadre_core::DocumentValidationError| e.to_string())?;

    let store = DocumentStore::new(path);
    let rules_configs = load_all_rules_configs(&store)?;
    let refs: Vec<&RulesConfig> = rules_configs.iter().collect();
    let engine = RuleQueryEngine::new(&refs);

    let results = engine.applicable_at_scope(target_scope);

    if results.is_empty() {
        println!("No rules applicable at scope '{}'.", target_scope_str);
        return Ok(());
    }

    // Show inheritance chain
    let hierarchy: Vec<&str> = scope_hierarchy()
        .iter()
        .take_while(|s| scope_rank(**s) <= scope_rank(target_scope))
        .map(|s| match s {
            cadre_core::domain::documents::rules_config::RuleScope::Platform => "platform",
            cadre_core::domain::documents::rules_config::RuleScope::Organization => "org",
            cadre_core::domain::documents::rules_config::RuleScope::Repository => "repo",
            cadre_core::domain::documents::rules_config::RuleScope::Package => "package",
            cadre_core::domain::documents::rules_config::RuleScope::Component => "component",
            cadre_core::domain::documents::rules_config::RuleScope::Task => "task",
        })
        .collect();

    println!(
        "APPLICABLE RULES AT '{}' SCOPE ({} rules)",
        target_scope_str,
        results.len()
    );
    println!("Inheritance chain: {}", hierarchy.join(" > "));
    println!();
    println!(
        "{:<16} {:<30} {:<12} {:<12}",
        "SHORT CODE", "TITLE", "SCOPE", "PROTECTION"
    );
    println!("{}", "-".repeat(70));
    for rule in &results {
        let title_trunc = if rule.title().len() > 29 {
            format!("{}...", &rule.title()[..26])
        } else {
            rule.title().to_string()
        };
        println!(
            "{:<16} {:<30} {:<12} {:<12}",
            rule.metadata().short_code,
            title_trunc,
            rule.scope,
            rule.protection_level
        );
    }
    Ok(())
}

fn cmd_rules_protected(path: &Path) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let rules_configs = load_all_rules_configs(&store)?;
    let refs: Vec<&RulesConfig> = rules_configs.iter().collect();
    let engine = RuleQueryEngine::new(&refs);

    let results = engine.protected_rules();

    if results.is_empty() {
        println!("No protected rules found.");
        return Ok(());
    }

    println!("PROTECTED RULES ({})", results.len());
    println!(
        "{:<16} {:<30} {:<12} {:<20}",
        "SHORT CODE", "TITLE", "SCOPE", "SOURCE ARCH"
    );
    println!("{}", "-".repeat(78));
    for rule in &results {
        let arch_ref = rule.source_architecture_ref.as_deref().unwrap_or("-");
        let title_trunc = if rule.title().len() > 29 {
            format!("{}...", &rule.title()[..26])
        } else {
            rule.title().to_string()
        };
        println!(
            "{:<16} {:<30} {:<12} {:<20}",
            rule.metadata().short_code,
            title_trunc,
            rule.scope,
            arch_ref
        );
    }
    Ok(())
}

// ===========================================================================
// Notes command implementations
// ===========================================================================

/// Build an InsightScope from CLI arguments
fn build_scope(
    repo: Option<&str>,
    package: Option<&str>,
    subsystem: Option<&str>,
    paths: Option<&[String]>,
    symbols: Option<&[String]>,
) -> InsightScope {
    let mut scope = InsightScope::new();
    scope.repo = repo.map(|s| s.to_string());
    scope.package = package.map(|s| s.to_string());
    scope.subsystem = subsystem.map(|s| s.to_string());
    scope.paths = paths.map(|p| p.to_vec()).unwrap_or_default();
    scope.symbols = symbols.map(|s| s.to_vec()).unwrap_or_default();
    scope
}

fn cmd_notes_create(
    path: &Path,
    title: &str,
    note_text: &str,
    category_str: &str,
    scope_repo: Option<&str>,
    scope_package: Option<&str>,
    scope_subsystem: Option<&str>,
    scope_paths: Option<&[String]>,
    scope_symbols: Option<&[String]>,
) -> Result<(), String> {
    let category: InsightCategory = category_str.parse().map_err(|e: String| e)?;
    let scope = build_scope(
        scope_repo,
        scope_package,
        scope_subsystem,
        scope_paths,
        scope_symbols,
    );

    let store = DocumentStore::new(path);
    let short_code = store
        .create_document("durable_insight_note", title, None)
        .map_err(|e| e.user_message())?;

    // Create the proper note with scope and category, overwrite
    let din = DurableInsightNote::new(
        title.to_string(),
        note_text.to_string(),
        category,
        scope.clone(),
        vec![cadre_core::Tag::Phase(cadre_core::Phase::Draft)],
        false,
        short_code.clone(),
    )
    .map_err(|e| e.to_string())?;

    let content = din.to_content().map_err(|e| e.to_string())?;
    let doc_path = path
        .join(".cadre")
        .join("docs")
        .join(format!("{}.md", short_code));
    std::fs::write(&doc_path, content).map_err(|e| format!("Failed to write note: {}", e))?;

    let scope_desc = [
        scope.repo.as_deref().unwrap_or(""),
        scope.package.as_deref().unwrap_or(""),
        scope.subsystem.as_deref().unwrap_or(""),
    ]
    .iter()
    .filter(|s| !s.is_empty())
    .copied()
    .collect::<Vec<_>>()
    .join(", ");

    println!("Created insight note {}", short_code);
    println!("{:<18} {}", "Title:", title);
    println!("{:<18} {}", "Category:", category_str);
    println!(
        "{:<18} {}",
        "Scope:",
        if scope_desc.is_empty() {
            "global"
        } else {
            &scope_desc
        }
    );
    Ok(())
}

fn cmd_notes_fetch(
    path: &Path,
    scope_repo: Option<&str>,
    scope_package: Option<&str>,
    scope_subsystem: Option<&str>,
    scope_paths: Option<&[String]>,
    scope_symbols: Option<&[String]>,
    limit: Option<usize>,
) -> Result<(), String> {
    let limit = limit.unwrap_or(10);
    let query_scope = build_scope(
        scope_repo,
        scope_package,
        scope_subsystem,
        scope_paths,
        scope_symbols,
    );

    let store = DocumentStore::new(path);
    let all_docs = store.list_documents(false).map_err(|e| e.user_message())?;

    let mut matched = Vec::new();
    for doc in &all_docs {
        if doc.document_type != "durable_insight_note" {
            continue;
        }
        let raw = match store.read_document_raw(&doc.short_code) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if let Ok(din) = DurableInsightNote::from_content(&raw) {
            if din.status != NoteStatus::Active {
                continue;
            }
            let has_query = query_scope.repo.is_some()
                || query_scope.package.is_some()
                || query_scope.subsystem.is_some()
                || !query_scope.paths.is_empty()
                || !query_scope.symbols.is_empty();

            if !has_query || din.scope.matches(&query_scope) {
                matched.push((doc.short_code.clone(), din));
            }
        }
        if matched.len() >= limit {
            break;
        }
    }

    // Record fetch on each matched note and save back
    for (sc, _) in &matched {
        let mut note =
            DurableInsightNote::from_content(&store.read_document_raw(sc).unwrap_or_default()).ok();
        if let Some(ref mut n) = note {
            n.record_fetch();
            if let Ok(content) = n.to_content() {
                let doc_path = path.join(".cadre").join("docs").join(format!("{}.md", sc));
                let _ = std::fs::write(&doc_path, content);
            }
        }
    }

    if matched.is_empty() {
        println!("No insight notes found for the given scope.");
        return Ok(());
    }

    println!("INSIGHT NOTES ({})", matched.len());
    println!();
    for (sc, din) in &matched {
        println!("--- {} -- {} [{}]", sc, din.title(), din.category);
        println!("{}", din.note);
        println!();
    }
    Ok(())
}

fn cmd_notes_score(path: &Path, short_code: &str, signal_str: &str) -> Result<(), String> {
    let signal: FeedbackSignal = signal_str.parse().map_err(|e: String| e)?;

    let store = DocumentStore::new(path);
    let raw = store
        .read_document_raw(short_code)
        .map_err(|e| e.user_message())?;
    let mut din = DurableInsightNote::from_content(&raw).map_err(|e| e.to_string())?;

    din.record_feedback(signal);

    // Check for prune candidate
    let was_pruned = din.should_be_prune_candidate(30, 0.5, 3, 5, 0.3);
    if was_pruned {
        din.mark_prune_candidate();
    }

    let content = din.to_content().map_err(|e| e.to_string())?;
    let doc_path = path
        .join(".cadre")
        .join("docs")
        .join(format!("{}.md", short_code));
    std::fs::write(&doc_path, content).map_err(|e| format!("Failed to write: {}", e))?;

    let status_change = if was_pruned {
        " (marked as prune candidate)"
    } else {
        ""
    };

    println!("Feedback recorded for {}{}", short_code, status_change);
    println!("{:<18} {}", "Signal:", signal_str);
    println!("{:<18} {}", "Helpful:", din.thumbs_up_count);
    println!("{:<18} {}", "Meh:", din.meh_count);
    println!("{:<18} {}", "Harmful:", din.thumbs_down_count);
    println!("{:<18} {}", "Status:", din.status);
    Ok(())
}

fn cmd_notes_list(
    path: &Path,
    status_filter: Option<&str>,
    category_filter: Option<&str>,
    include_archived: bool,
) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let all_docs = store
        .list_documents(include_archived)
        .map_err(|e| e.user_message())?;

    let mut notes = Vec::new();
    for doc in &all_docs {
        if doc.document_type != "durable_insight_note" {
            continue;
        }
        let raw = match store.read_document_raw(&doc.short_code) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if let Ok(din) = DurableInsightNote::from_content(&raw) {
            if let Some(sf) = status_filter {
                if din.status.to_string() != sf {
                    continue;
                }
            }
            if let Some(cf) = category_filter {
                if din.category.to_string() != cf {
                    continue;
                }
            }
            notes.push((doc.short_code.clone(), din));
        }
    }

    if notes.is_empty() {
        println!("No insight notes found.");
        return Ok(());
    }

    println!("INSIGHT NOTES ({})", notes.len());
    println!(
        "{:<16} {:<28} {:<16} {:<12} {:>8} {:>10}",
        "SHORT CODE", "TITLE", "CATEGORY", "STATUS", "FETCHES", "HELPFUL%"
    );
    println!("{}", "-".repeat(90));
    for (sc, din) in &notes {
        let helpful_pct = if din.total_feedback() > 0 {
            format!("{:.0}%", din.helpful_ratio() * 100.0)
        } else {
            "-".to_string()
        };
        let title_trunc = if din.title().len() > 27 {
            format!("{}...", &din.title()[..24])
        } else {
            din.title().to_string()
        };
        println!(
            "{:<16} {:<28} {:<16} {:<12} {:>8} {:>10}",
            sc, title_trunc, din.category, din.status, din.fetch_count, helpful_pct
        );
    }
    Ok(())
}

// ===========================================================================
// Trace command implementations
// ===========================================================================

/// Helper to build a TraceabilityIndex from all CrossReference documents
fn build_traceability_index(
    store: &DocumentStore,
) -> Result<(TraceabilityIndex, Vec<(String, CrossReference)>), String> {
    let all_docs = store.list_documents(false).map_err(|e| e.user_message())?;
    let mut index = TraceabilityIndex::new();
    let mut xrefs = Vec::new();

    for doc in &all_docs {
        if doc.document_type != "cross_reference" {
            continue;
        }
        let raw = match store.read_document_raw(&doc.short_code) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if let Ok(xref) = CrossReference::from_content(&raw) {
            index.add_from_document(&xref);
            xrefs.push((doc.short_code.clone(), xref));
        }
    }
    Ok((index, xrefs))
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn cmd_trace_create(
    path: &Path,
    source_ref: &str,
    target_ref: &str,
    rel_type_str: &str,
    description: Option<&str>,
    bidirectional: bool,
) -> Result<(), String> {
    if source_ref == target_ref {
        return Err("Source and target cannot be the same document.".to_string());
    }

    let rel_type: RelationshipType = rel_type_str.parse().map_err(|e: String| e)?;

    let store = DocumentStore::new(path);

    // Verify both documents exist
    store
        .read_document(source_ref)
        .map_err(|e| e.user_message())?;
    store
        .read_document(target_ref)
        .map_err(|e| e.user_message())?;

    let title = format!("{} {} {}", source_ref, rel_type, target_ref);
    let short_code = store
        .create_document("cross_reference", &title, None)
        .map_err(|e| e.user_message())?;

    let desc = description.unwrap_or("");

    // Create the proper cross-reference and overwrite
    let xref = CrossReference::new(
        title,
        vec![cadre_core::Tag::Phase(cadre_core::Phase::Draft)],
        false,
        short_code.clone(),
        source_ref.to_string(),
        target_ref.to_string(),
        rel_type,
        desc.to_string(),
        bidirectional,
    )
    .map_err(|e| e.to_string())?;

    let content = xref.to_content().map_err(|e| e.to_string())?;
    let doc_path = path
        .join(".cadre")
        .join("docs")
        .join(format!("{}.md", short_code));
    std::fs::write(&doc_path, content).map_err(|e| format!("Failed to write: {}", e))?;

    let bidir_label = if bidirectional {
        " (bidirectional)"
    } else {
        ""
    };
    println!("Created cross-reference {}{}", short_code, bidir_label);
    println!("{:<18} {}", "Source:", source_ref);
    println!("{:<18} {}", "Target:", target_ref);
    println!("{:<18} {}", "Type:", rel_type);
    if !desc.is_empty() {
        println!("{:<18} {}", "Description:", desc);
    }
    Ok(())
}

fn cmd_trace_query(
    path: &Path,
    short_code: &str,
    direction: &str,
    rel_type_filter: Option<&str>,
) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let (index, _) = build_traceability_index(&store)?;

    let entries = match direction {
        "outgoing" => index.outgoing(short_code),
        "incoming" => index.incoming(short_code),
        _ => index.involving(short_code),
    };

    let filtered: Vec<_> = if let Some(rt) = rel_type_filter {
        let rel_type: RelationshipType = rt.parse().map_err(|e: String| e)?;
        entries
            .into_iter()
            .filter(|e| e.relationship_type == rel_type)
            .collect()
    } else {
        entries
    };

    if filtered.is_empty() {
        println!("No {} relationships found for {}.", direction, short_code);
        return Ok(());
    }

    println!(
        "RELATIONSHIPS FOR {} ({}, {})",
        short_code,
        direction,
        filtered.len()
    );
    println!(
        "{:<16} {:<20} {:<16} {:<14}",
        "SOURCE", "TYPE", "TARGET", "BIDIRECTIONAL"
    );
    println!("{}", "-".repeat(66));
    for entry in &filtered {
        println!(
            "{:<16} {:<20} {:<16} {:<14}",
            entry.source_ref, entry.relationship_type, entry.target_ref, entry.bidirectional
        );
    }
    Ok(())
}

fn cmd_trace_ancestry(path: &Path, short_code: &str, direction: &str) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let (index, _) = build_traceability_index(&store)?;

    let results = match direction {
        "ancestors" => index.ancestors(short_code),
        "descendants" => index.descendants(short_code),
        "siblings" => index.siblings(short_code),
        _ => {
            return Err(format!(
                "Invalid direction '{}'. Use: ancestors, descendants, or siblings",
                direction
            ))
        }
    };

    if results.is_empty() {
        println!("No {} found for {}.", direction, short_code);
        return Ok(());
    }

    println!(
        "{} OF {} ({})",
        capitalize_first(direction).to_uppercase(),
        short_code,
        results.len()
    );
    for (i, sc) in results.iter().enumerate() {
        let title = store
            .read_document(sc)
            .map(|d| d.title().to_string())
            .unwrap_or_else(|_| "?".to_string());
        let prefix = if direction == "ancestors" {
            "  ".repeat(i)
        } else {
            String::new()
        };
        println!("{}- {} ({})", prefix, sc, title);
    }
    Ok(())
}

fn cmd_trace_list(
    path: &Path,
    rel_type_filter: Option<&str>,
    involving: Option<&str>,
) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let (_, xrefs) = build_traceability_index(&store)?;

    let filtered: Vec<_> = xrefs
        .iter()
        .filter(|(_, xref)| {
            if let Some(rt) = rel_type_filter {
                if let Ok(rel_type) = rt.parse::<RelationshipType>() {
                    if xref.relationship_type != rel_type {
                        return false;
                    }
                }
            }
            if let Some(inv) = involving {
                if !xref.involves(inv) {
                    return false;
                }
            }
            true
        })
        .collect();

    if filtered.is_empty() {
        println!("No cross-references found.");
        return Ok(());
    }

    println!("CROSS-REFERENCES ({})", filtered.len());
    println!(
        "{:<16} {:<16} {:<20} {:<16} {:<14}",
        "SHORT CODE", "SOURCE", "TYPE", "TARGET", "BIDIRECTIONAL"
    );
    println!("{}", "-".repeat(82));
    for (sc, xref) in &filtered {
        println!(
            "{:<16} {:<16} {:<20} {:<16} {:<14}",
            sc, xref.source_ref, xref.relationship_type, xref.target_ref, xref.bidirectional
        );
    }
    Ok(())
}
