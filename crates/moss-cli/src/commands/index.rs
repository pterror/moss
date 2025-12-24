//! Index-related commands for moss CLI.

use crate::{index, skeleton};
use moss_core::get_moss_dir;
use moss_languages::external_packages;
use std::path::{Path, PathBuf};

/// Refresh the file index
pub fn cmd_reindex(root: Option<&Path>, call_graph: bool) -> i32 {
    let root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    match index::FileIndex::open(&root) {
        Ok(mut idx) => {
            match idx.refresh() {
                Ok(count) => {
                    println!("Indexed {} files", count);

                    // Optionally rebuild call graph
                    if call_graph {
                        match idx.refresh_call_graph() {
                            Ok((symbols, calls, imports)) => {
                                println!(
                                    "Indexed {} symbols, {} calls, {} imports",
                                    symbols, calls, imports
                                );
                            }
                            Err(e) => {
                                eprintln!("Error indexing call graph: {}", e);
                                return 1;
                            }
                        }
                    }
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

/// Check if a file is binary by looking for null bytes
fn is_binary_file(path: &Path) -> bool {
    use std::io::Read;

    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };

    let mut buffer = [0u8; 8192];
    let Ok(bytes_read) = file.read(&mut buffer) else {
        return false;
    };

    // Check for null bytes (common in binary files)
    buffer[..bytes_read].contains(&0)
}

/// Show index statistics
pub fn cmd_index_stats(root: Option<&Path>, json: bool) -> i32 {
    let root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let moss_dir = get_moss_dir(&root);
    let db_path = moss_dir.join("index.sqlite");

    // Get DB file size
    let db_size = std::fs::metadata(&db_path)
        .map(|m| m.len())
        .unwrap_or(0);

    // Open index and get stats
    let idx = match index::FileIndex::open(&root) {
        Ok(idx) => idx,
        Err(e) => {
            eprintln!("Failed to open index: {}", e);
            return 1;
        }
    };

    // Get file stats from index
    let files = match idx.all_files() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to read files: {}", e);
            return 1;
        }
    };

    let file_count = files.iter().filter(|f| !f.is_dir).count();
    let dir_count = files.iter().filter(|f| f.is_dir).count();

    // Count by extension (detect binary files)
    let mut ext_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for f in &files {
        if f.is_dir {
            continue;
        }
        let path = std::path::Path::new(&f.path);
        let ext = match path.extension().and_then(|e| e.to_str()) {
            Some(e) => e.to_string(),
            None => {
                // No extension - check if binary
                let full_path = root.join(&f.path);
                if is_binary_file(&full_path) {
                    "(binary)".to_string()
                } else {
                    "(no ext)".to_string()
                }
            }
        };
        *ext_counts.entry(ext).or_insert(0) += 1;
    }

    // Sort by count descending
    let mut ext_list: Vec<_> = ext_counts.into_iter().collect();
    ext_list.sort_by(|a, b| b.1.cmp(&a.1));

    // Get call graph stats
    let (symbol_count, call_count, import_count) = idx.call_graph_stats().unwrap_or((0, 0, 0));

    // Calculate codebase size (sum of file sizes)
    let mut codebase_size: u64 = 0;
    for f in &files {
        if !f.is_dir {
            let full_path = root.join(&f.path);
            if let Ok(meta) = std::fs::metadata(&full_path) {
                codebase_size += meta.len();
            }
        }
    }

    if json {
        let output = serde_json::json!({
            "db_size_bytes": db_size,
            "codebase_size_bytes": codebase_size,
            "ratio": if codebase_size > 0 { db_size as f64 / codebase_size as f64 } else { 0.0 },
            "file_count": file_count,
            "dir_count": dir_count,
            "symbol_count": symbol_count,
            "call_count": call_count,
            "import_count": import_count,
            "extensions": ext_list.iter().take(20).map(|(e, c)| serde_json::json!({"ext": e, "count": c})).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        println!("Index Statistics");
        println!("================");
        println!();
        println!("Database:     {} ({:.1} KB)", db_path.display(), db_size as f64 / 1024.0);
        println!("Codebase:     {:.1} MB", codebase_size as f64 / 1024.0 / 1024.0);
        println!("Ratio:        {:.2}%", if codebase_size > 0 { db_size as f64 / codebase_size as f64 * 100.0 } else { 0.0 });
        println!();
        println!("Files:        {} ({} dirs)", file_count, dir_count);
        println!("Symbols:      {}", symbol_count);
        println!("Calls:        {}", call_count);
        println!("Imports:      {}", import_count);
        println!();
        println!("Top extensions:");
        for (ext, count) in ext_list.iter().take(15) {
            println!("  {:12} {:>6}", ext, count);
        }
    }

    0
}

/// List files in the index
pub fn cmd_list_files(prefix: Option<&str>, root: Option<&Path>, limit: usize, json: bool) -> i32 {
    let root = root
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let idx = match index::FileIndex::open(&root) {
        Ok(idx) => idx,
        Err(e) => {
            eprintln!("Failed to open index: {}", e);
            return 1;
        }
    };

    let files = match idx.all_files() {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to read files: {}", e);
            return 1;
        }
    };

    // Filter by prefix and exclude directories
    let prefix_str = prefix.unwrap_or("");
    let filtered: Vec<&str> = files
        .iter()
        .filter(|f| !f.is_dir && f.path.starts_with(prefix_str))
        .take(limit)
        .map(|f| f.path.as_str())
        .collect();

    if json {
        println!("{}", serde_json::to_string(&filtered).unwrap());
    } else {
        for path in &filtered {
            println!("{}", path);
        }
    }

    0
}

/// Index external packages into the global cache.
pub fn cmd_index_packages(only: &[String], clear: bool, root: Option<&Path>, json: bool) -> i32 {
    let root = root.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    });

    // Open or create the index
    let index = match external_packages::PackageIndex::open() {
        Ok(idx) => idx,
        Err(e) => {
            eprintln!("Failed to open package index: {}", e);
            return 1;
        }
    };

    if clear {
        if let Err(e) = index.clear() {
            eprintln!("Failed to clear index: {}", e);
            return 1;
        }
        if !json {
            println!("Cleared existing index");
        }
    }

    // Collect results per language
    let mut results: std::collections::HashMap<&str, (usize, usize)> = std::collections::HashMap::new();

    // Get all available lang_keys from registered languages
    let available: Vec<&str> = moss_languages::supported_languages()
        .iter()
        .map(|l| l.lang_key())
        .filter(|k| !k.is_empty())
        .collect();

    // Filter to requested ecosystems
    let ecosystems: Vec<&str> = if only.is_empty() {
        available.clone()
    } else {
        only.iter()
            .map(|s| s.as_str())
            .filter(|s| available.contains(s))
            .collect()
    };

    // Log error for unknown ecosystems
    for eco in only {
        if !available.contains(&eco.as_str()) {
            eprintln!("Error: unknown ecosystem '{}', valid options: {}", eco, available.join(", "));
        }
    }

    // Index each language using the generic indexer
    for lang in moss_languages::supported_languages() {
        let lang_key = lang.lang_key();
        if lang_key.is_empty() || !ecosystems.contains(&lang_key) {
            continue;
        }
        // Skip if we already indexed this lang_key (e.g., TypeScript shares "js" with JavaScript)
        if results.contains_key(lang_key) {
            continue;
        }
        let (pkgs, syms) = index_language_packages(lang, &index, &root, json);
        results.insert(lang_key, (pkgs, syms));
    }

    // Output results
    if json {
        let mut json_obj = serde_json::Map::new();
        for (key, (pkgs, syms)) in &results {
            json_obj.insert(format!("{}_packages", key), serde_json::json!(pkgs));
            json_obj.insert(format!("{}_symbols", key), serde_json::json!(syms));
        }
        println!("{}", serde_json::Value::Object(json_obj));
    } else {
        println!("\nIndexing complete:");
        for (key, (pkgs, syms)) in &results {
            println!("  {}: {} packages, {} symbols", key, pkgs, syms);
        }
    }

    0
}

// ============================================================================
// Generic package indexer using Language trait
// ============================================================================

/// Count symbols and insert them into the index.
fn count_and_insert_symbols(
    index: &external_packages::PackageIndex,
    pkg_id: i64,
    symbols: &[skeleton::SkeletonSymbol],
) -> usize {
    let mut count = 0;
    for sym in symbols {
        let _ = index.insert_symbol(
            pkg_id,
            &sym.name,
            sym.kind,
            &sym.signature,
            sym.start_line as u32,
        );
        count += 1;
        count += count_and_insert_symbols(index, pkg_id, &sym.children);
    }
    count
}

/// Index packages for a language using its package_sources().
fn index_language_packages(
    lang: &dyn moss_languages::Language,
    index: &external_packages::PackageIndex,
    project_root: &Path,
    json: bool,
) -> (usize, usize) {
    let version = lang.get_version(project_root)
        .and_then(|v| external_packages::Version::parse(&v));

    let lang_key = lang.lang_key();
    if lang_key.is_empty() {
        return (0, 0);
    }

    if !json {
        println!("Indexing {} packages (version {:?})...", lang.name(), version);
    }

    let sources = lang.package_sources(project_root);
    if sources.is_empty() {
        if !json {
            println!("  No package sources found");
        }
        return (0, 0);
    }

    let min_version = version.unwrap_or(external_packages::Version { major: 0, minor: 0 });
    let mut extractor = skeleton::SkeletonExtractor::new();
    let mut total_packages = 0;
    let mut total_symbols = 0;

    for source in sources {
        if !json {
            println!("  {}: {}", source.name, source.path.display());
        }

        let max_version = if source.version_specific { version } else { None };

        // Use the trait's discover_packages method - no kind-specific dispatch here
        let discovered = lang.discover_packages(&source);

        for (pkg_name, pkg_path) in discovered {
            if let Ok(true) = index.is_indexed(lang_key, &pkg_name) {
                continue;
            }

            let pkg_id = match index.insert_package(
                lang_key,
                &pkg_name,
                &pkg_path.to_string_lossy(),
                min_version,
                max_version,
            ) {
                Ok(id) => id,
                Err(_) => continue,
            };

            total_packages += 1;
            total_symbols += index_package_symbols(lang, index, &mut extractor, pkg_id, &pkg_path);
        }
    }

    (total_packages, total_symbols)
}

/// Index symbols from a package path (file or directory).
fn index_package_symbols(
    lang: &dyn moss_languages::Language,
    index: &external_packages::PackageIndex,
    extractor: &mut skeleton::SkeletonExtractor,
    pkg_id: i64,
    path: &Path,
) -> usize {
    // Use trait method to find entry point
    let entry = match lang.find_package_entry(path) {
        Some(e) => e,
        None => return 0,
    };

    if let Ok(content) = std::fs::read_to_string(&entry) {
        let result = extractor.extract(&entry, &content);
        return count_and_insert_symbols(index, pkg_id, &result.symbols);
    }

    0
}
