//! Ultra-Metis CLI
//!
//! Command-line interface for Ultra-Metis document management.

use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;
use ultra_metis_store::DocumentStore;

#[derive(Parser)]
#[command(name = "ultra-metis")]
#[command(about = "Ultra-Metis: Repo-Native AI Engineering Orchestration")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Ultra-Metis project
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
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { path, prefix } => cmd_init(&path, &prefix),
        Commands::List {
            path,
            include_archived,
        } => cmd_list(&path, include_archived),
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
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn cmd_init(path: &PathBuf, prefix: &str) -> Result<(), String> {
    let store = DocumentStore::new(path);
    store.initialize(prefix).map_err(|e| e.user_message())?;
    println!("Initialized Ultra-Metis project at {} with prefix {}", path.display(), prefix);
    Ok(())
}

fn cmd_list(path: &PathBuf, include_archived: bool) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let docs = store.list_documents(include_archived).map_err(|e| e.user_message())?;

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

fn cmd_read(path: &PathBuf, short_code: &str) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let raw = store.read_document_raw(short_code).map_err(|e| e.user_message())?;
    println!("{}", raw);
    Ok(())
}

fn cmd_create(
    path: &PathBuf,
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
    path: &PathBuf,
    short_code: &str,
    search: &str,
    replace: &str,
    replace_all: bool,
) -> Result<(), String> {
    let store = DocumentStore::new(path);
    store
        .edit_document_with_options(short_code, search, replace, replace_all)
        .map_err(|e| e.user_message())?;
    let mode = if replace_all { " (all occurrences)" } else { "" };
    println!("Updated {}{}", short_code, mode);
    Ok(())
}

fn cmd_transition(
    path: &PathBuf,
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
    path: &PathBuf,
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

fn cmd_archive(path: &PathBuf, short_code: &str) -> Result<(), String> {
    let store = DocumentStore::new(path);
    store
        .archive_document(short_code)
        .map_err(|e| e.user_message())?;
    println!("Archived {}", short_code);
    Ok(())
}

fn cmd_reassign(
    path: &PathBuf,
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

fn cmd_status(path: &PathBuf) -> Result<(), String> {
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
            let todo = task_docs.iter().filter(|t| t.parent_id.as_deref() == Some(&init.short_code) && t.phase == "todo").count();
            let active = task_docs.iter().filter(|t| t.parent_id.as_deref() == Some(&init.short_code) && t.phase == "active").count();
            let done = task_docs.iter().filter(|t| t.parent_id.as_deref() == Some(&init.short_code) && t.phase == "completed").count();
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

fn cmd_validate(
    path: &PathBuf,
    short_code: Option<&str>,
    all: bool,
) -> Result<(), String> {
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
                    let parent_str = parent_id.to_string();
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
