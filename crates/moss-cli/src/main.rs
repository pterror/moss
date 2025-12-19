use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

mod index;
mod path_resolve;

#[derive(Parser)]
#[command(name = "moss")]
#[command(about = "Fast code intelligence CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output as JSON
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Resolve a fuzzy path to exact location(s)
    Path {
        /// Query to resolve (file path, partial name, or symbol)
        query: String,

        /// Root directory to search (defaults to current directory)
        #[arg(short, long)]
        root: Option<PathBuf>,
    },

    /// View a file or symbol (shows content)
    View {
        /// Target to view (file path or symbol name)
        target: String,

        /// Root directory (defaults to current directory)
        #[arg(short, long)]
        root: Option<PathBuf>,

        /// Show line numbers
        #[arg(short = 'n', long)]
        line_numbers: bool,
    },

    /// Search for files/symbols matching a pattern
    SearchTree {
        /// Search query
        query: String,

        /// Root directory (defaults to current directory)
        #[arg(short, long)]
        root: Option<PathBuf>,

        /// Limit results
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Rebuild the file index
    Reindex {
        /// Root directory (defaults to current directory)
        #[arg(short, long)]
        root: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    let exit_code = match cli.command {
        Commands::Path { query, root } => cmd_path(&query, root.as_deref(), cli.json),
        Commands::View { target, root, line_numbers } => {
            cmd_view(&target, root.as_deref(), line_numbers, cli.json)
        }
        Commands::SearchTree { query, root, limit } => {
            cmd_search_tree(&query, root.as_deref(), limit, cli.json)
        }
        Commands::Reindex { root } => cmd_reindex(root.as_deref()),
    };

    std::process::exit(exit_code);
}

fn cmd_path(query: &str, root: Option<&Path>, json: bool) -> i32 {
    let root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let matches = path_resolve::resolve(&query, &root);

    if matches.is_empty() {
        if json {
            println!("[]");
        } else {
            eprintln!("No matches for: {}", query);
        }
        return 1;
    }

    if json {
        let output: Vec<_> = matches
            .iter()
            .map(|m| serde_json::json!({"path": m.path, "kind": m.kind}))
            .collect();
        println!("{}", serde_json::to_string(&output).unwrap());
    } else {
        for m in &matches {
            println!("{} ({})", m.path, m.kind);
        }
    }

    0
}

fn cmd_view(target: &str, root: Option<&Path>, line_numbers: bool, json: bool) -> i32 {
    let root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    // Resolve the target to a file
    let matches = path_resolve::resolve(target, &root);

    if matches.is_empty() {
        eprintln!("No matches for: {}", target);
        return 1;
    }

    // Take the first file match
    let file_match = matches
        .iter()
        .find(|m| m.kind == "file")
        .unwrap_or(&matches[0]);

    let file_path = root.join(&file_match.path);

    match std::fs::read_to_string(&file_path) {
        Ok(content) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "path": file_match.path,
                        "content": content
                    })
                );
            } else if line_numbers {
                for (i, line) in content.lines().enumerate() {
                    println!("{:4} {}", i + 1, line);
                }
            } else {
                print!("{}", content);
            }
            0
        }
        Err(e) => {
            eprintln!("Error reading {}: {}", file_match.path, e);
            1
        }
    }
}

fn cmd_search_tree(query: &str, root: Option<&Path>, limit: usize, json: bool) -> i32 {
    let root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    // Use fuzzy matching to find all matches
    let matches = path_resolve::resolve(query, &root);
    let limited: Vec<_> = matches.into_iter().take(limit).collect();

    if json {
        let output: Vec<_> = limited
            .iter()
            .map(|m| serde_json::json!({"path": m.path, "kind": m.kind, "score": m.score}))
            .collect();
        println!("{}", serde_json::to_string(&output).unwrap());
    } else {
        for m in &limited {
            println!("{} ({})", m.path, m.kind);
        }
    }

    0
}

fn cmd_reindex(root: Option<&Path>) -> i32 {
    let root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    match index::FileIndex::open(&root) {
        Ok(mut idx) => {
            match idx.refresh() {
                Ok(count) => {
                    println!("Indexed {} files", count);
                    0
                }
                Err(e) => {
                    eprintln!("Error refreshing index: {}", e);
                    1
                }
            }
        }
        Err(e) => {
            eprintln!("Error opening index: {}", e);
            1
        }
    }
}
