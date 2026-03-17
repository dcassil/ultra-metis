//! Ultra-Metis CLI
//!
//! Command-line interface for Ultra-Metis document management.

use clap::{Parser, Subcommand};
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

        /// Project root directory
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
    },

    /// Search documents by text query
    Search {
        /// Search query
        query: String,

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
            path,
        } => cmd_edit(&path, &short_code, &search, &replace),
        Commands::Transition {
            short_code,
            phase,
            path,
        } => cmd_transition(&path, &short_code, phase.as_deref()),
        Commands::Search { query, path } => cmd_search(&path, &query),
        Commands::Archive { short_code, path } => cmd_archive(&path, &short_code),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn cmd_init(path: &PathBuf, prefix: &str) -> Result<(), String> {
    let store = DocumentStore::new(path);
    store.initialize(prefix).map_err(|e| e.to_string())?;
    println!("Initialized Ultra-Metis project at {} with prefix {}", path.display(), prefix);
    Ok(())
}

fn cmd_list(path: &PathBuf, include_archived: bool) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let docs = store.list_documents(include_archived).map_err(|e| e.to_string())?;

    if docs.is_empty() {
        println!("No documents found.");
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

fn cmd_read(path: &PathBuf, short_code: &str) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let raw = store.read_document_raw(short_code).map_err(|e| e.to_string())?;
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
        .map_err(|e| e.to_string())?;
    println!("Created {} ({}: {})", short_code, doc_type, title);
    Ok(())
}

fn cmd_edit(path: &PathBuf, short_code: &str, search: &str, replace: &str) -> Result<(), String> {
    let store = DocumentStore::new(path);
    store
        .edit_document(short_code, search, replace)
        .map_err(|e| e.to_string())?;
    println!("Updated {}", short_code);
    Ok(())
}

fn cmd_transition(
    path: &PathBuf,
    short_code: &str,
    phase: Option<&str>,
) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let result = store
        .transition_phase(short_code, phase)
        .map_err(|e| e.to_string())?;
    println!("{}: {}", short_code, result);
    Ok(())
}

fn cmd_search(path: &PathBuf, query: &str) -> Result<(), String> {
    let store = DocumentStore::new(path);
    let docs = store.search_documents(query).map_err(|e| e.to_string())?;

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
        .map_err(|e| e.to_string())?;
    println!("Archived {}", short_code);
    Ok(())
}
