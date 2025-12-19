use ignore::WalkBuilder;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct PathMatch {
    pub path: String,
    pub kind: String,
    pub score: u32,
}

/// Resolve a fuzzy query to matching paths.
///
/// Handles:
/// - Exact paths: src/moss/dwim.py
/// - Partial filenames: dwim.py, dwim
/// - Directory names: moss, src
pub fn resolve(query: &str, root: &Path) -> Vec<PathMatch> {
    // Handle file:symbol syntax (defer symbol resolution to Python for now)
    if query.contains(':') {
        let file_part = query.split(':').next().unwrap();
        return resolve(file_part, root);
    }

    let query_lower = query.to_lowercase();

    // Collect all files using gitignore-aware walker
    let mut all_paths: Vec<(String, bool)> = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if let Ok(rel) = path.strip_prefix(root) {
            let rel_str = rel.to_string_lossy().to_string();
            if !rel_str.is_empty() {
                let is_dir = path.is_dir();
                all_paths.push((rel_str, is_dir));
            }
        }
    }

    // Try exact match first
    for (path, is_dir) in &all_paths {
        if path == query {
            return vec![PathMatch {
                path: path.clone(),
                kind: if *is_dir { "directory" } else { "file" }.to_string(),
                score: u32::MAX,
            }];
        }
    }

    // Try exact filename/dirname match (case-insensitive)
    let mut exact_matches: Vec<PathMatch> = Vec::new();
    for (path, is_dir) in &all_paths {
        let name = Path::new(path)
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let stem = Path::new(path)
            .file_stem()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if name == query_lower || stem == query_lower {
            exact_matches.push(PathMatch {
                path: path.clone(),
                kind: if *is_dir { "directory" } else { "file" }.to_string(),
                score: u32::MAX - 1,
            });
        }
    }

    if !exact_matches.is_empty() {
        return exact_matches;
    }

    // Fuzzy match using nucleo
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);

    let mut fuzzy_matches: Vec<PathMatch> = Vec::new();

    for (path, is_dir) in &all_paths {
        let mut buf = Vec::new();
        if let Some(score) = pattern.score(nucleo_matcher::Utf32Str::new(path, &mut buf), &mut matcher) {
            fuzzy_matches.push(PathMatch {
                path: path.clone(),
                kind: if *is_dir { "directory" } else { "file" }.to_string(),
                score,
            });
        }
    }

    // Sort by score descending, take top 10
    fuzzy_matches.sort_by(|a, b| b.score.cmp(&a.score));
    fuzzy_matches.truncate(10);

    fuzzy_matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_exact_match() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src/moss")).unwrap();
        fs::write(dir.path().join("src/moss/cli.py"), "").unwrap();

        let matches = resolve("src/moss/cli.py", dir.path());
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].path, "src/moss/cli.py");
    }

    #[test]
    fn test_filename_match() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src/moss")).unwrap();
        fs::write(dir.path().join("src/moss/dwim.py"), "").unwrap();

        let matches = resolve("dwim.py", dir.path());
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].path, "src/moss/dwim.py");
    }

    #[test]
    fn test_stem_match() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src/moss")).unwrap();
        fs::write(dir.path().join("src/moss/dwim.py"), "").unwrap();

        let matches = resolve("dwim", dir.path());
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].path, "src/moss/dwim.py");
    }
}
