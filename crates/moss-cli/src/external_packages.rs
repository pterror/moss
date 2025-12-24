//! External package resolution for Python and Go.
//!
//! Finds installed packages, stdlib, and resolves import paths to their source files.
//! Uses a global cache at ~/.cache/moss/ for indexed packages.

use std::path::{Path, PathBuf};
use std::process::Command;

// =============================================================================
// Global Cache
// =============================================================================

/// Get the global moss cache directory (~/.cache/moss/).
pub fn get_global_cache_dir() -> Option<PathBuf> {
    // XDG_CACHE_HOME or ~/.cache
    let cache_base = if let Ok(xdg) = std::env::var("XDG_CACHE_HOME") {
        PathBuf::from(xdg)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cache")
    } else if let Ok(home) = std::env::var("USERPROFILE") {
        // Windows
        PathBuf::from(home).join(".cache")
    } else {
        return None;
    };

    let moss_cache = cache_base.join("moss");

    // Create if doesn't exist
    if !moss_cache.exists() {
        std::fs::create_dir_all(&moss_cache).ok()?;
    }

    Some(moss_cache)
}

/// Get the path to the unified global package index database.
/// e.g., ~/.cache/moss/packages.db
///
/// Schema:
/// - packages(id, language, name, path, min_major, min_minor, max_major, max_minor, indexed_at)
/// - symbols(id, package_id, name, kind, signature, line)
///
/// Version stored as (major, minor) integers for proper comparison.
/// max_major/max_minor NULL means "any version".
pub fn get_global_packages_db() -> Option<PathBuf> {
    let cache = get_global_cache_dir()?;
    Some(cache.join("packages.db"))
}

/// Get Python version from the project's interpreter.
pub fn get_python_version(project_root: &Path) -> Option<String> {
    let python = if project_root.join(".venv/bin/python").exists() {
        project_root.join(".venv/bin/python")
    } else if project_root.join("venv/bin/python").exists() {
        project_root.join("venv/bin/python")
    } else {
        PathBuf::from("python3")
    };

    let output = Command::new(&python)
        .args(["-c", "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')"])
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

/// Get Go version.
pub fn get_go_version() -> Option<String> {
    let output = Command::new("go").args(["version"]).output().ok()?;

    if output.status.success() {
        let version_str = String::from_utf8_lossy(&output.stdout);
        // "go version go1.21.0 linux/amd64" -> "1.21"
        for part in version_str.split_whitespace() {
            if part.starts_with("go") && part.len() > 2 {
                let ver = part.trim_start_matches("go");
                // Take major.minor only
                let parts: Vec<&str> = ver.split('.').collect();
                if parts.len() >= 2 {
                    return Some(format!("{}.{}", parts[0], parts[1]));
                }
            }
        }
    }

    None
}

/// Result of resolving an external package
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    /// Path to the package source
    pub path: PathBuf,
    /// Package name as imported
    pub name: String,
    /// Whether this is a namespace package (no __init__.py)
    pub is_namespace: bool,
}

// =============================================================================
// Python
// =============================================================================

/// Find Python stdlib directory.
///
/// Uses `python -c "import sys; print(sys.prefix)"` to find the prefix,
/// then looks for lib/pythonX.Y/ underneath.
pub fn find_python_stdlib(project_root: &Path) -> Option<PathBuf> {
    // Try to use the project's Python first (from venv)
    let python = if project_root.join(".venv/bin/python").exists() {
        project_root.join(".venv/bin/python")
    } else if project_root.join("venv/bin/python").exists() {
        project_root.join("venv/bin/python")
    } else {
        PathBuf::from("python3")
    };

    // Get sys.prefix and sys.version_info
    let output = Command::new(&python)
        .args(["-c", "import sys; print(sys.prefix); print(f'{sys.version_info.major}.{sys.version_info.minor}')"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines();
    let prefix = lines.next()?.trim();
    let version = lines.next()?.trim();

    // Unix: lib/pythonX.Y
    let stdlib = PathBuf::from(prefix).join("lib").join(format!("python{}", version));
    if stdlib.is_dir() {
        return Some(stdlib);
    }

    // Windows: Lib
    let stdlib = PathBuf::from(prefix).join("Lib");
    if stdlib.is_dir() {
        return Some(stdlib);
    }

    None
}

/// Check if a module name is a Python stdlib module.
pub fn is_python_stdlib_module(module_name: &str, stdlib_path: &Path) -> bool {
    let top_level = module_name.split('.').next().unwrap_or(module_name);

    // Check for package
    let pkg_dir = stdlib_path.join(top_level);
    if pkg_dir.is_dir() {
        return true;
    }

    // Check for module
    let py_file = stdlib_path.join(format!("{}.py", top_level));
    if py_file.is_file() {
        return true;
    }

    false
}

/// Resolve a Python stdlib import to its source location.
pub fn resolve_python_stdlib_import(import_name: &str, stdlib_path: &Path) -> Option<ResolvedPackage> {
    let parts: Vec<&str> = import_name.split('.').collect();
    let top_level = parts[0];

    // Check for package (directory)
    let pkg_dir = stdlib_path.join(top_level);
    if pkg_dir.is_dir() {
        if parts.len() == 1 {
            let init = pkg_dir.join("__init__.py");
            if init.is_file() {
                return Some(ResolvedPackage {
                    path: pkg_dir,
                    name: import_name.to_string(),
                    is_namespace: false,
                });
            }
            // Some stdlib packages don't have __init__.py in newer Python
            return Some(ResolvedPackage {
                path: pkg_dir,
                name: import_name.to_string(),
                is_namespace: true,
            });
        } else {
            // Submodule
            let mut path = pkg_dir.clone();
            for part in &parts[1..] {
                path = path.join(part);
            }

            if path.is_dir() {
                let init = path.join("__init__.py");
                return Some(ResolvedPackage {
                    path: path.clone(),
                    name: import_name.to_string(),
                    is_namespace: !init.is_file(),
                });
            }

            let py_file = path.with_extension("py");
            if py_file.is_file() {
                return Some(ResolvedPackage {
                    path: py_file,
                    name: import_name.to_string(),
                    is_namespace: false,
                });
            }

            return None;
        }
    }

    // Check for single-file module
    let py_file = stdlib_path.join(format!("{}.py", top_level));
    if py_file.is_file() {
        return Some(ResolvedPackage {
            path: py_file,
            name: import_name.to_string(),
            is_namespace: false,
        });
    }

    None
}

/// Find Python site-packages directory for a project.
///
/// Search order:
/// 1. .venv/lib/pythonX.Y/site-packages/ (uv, poetry, standard venv)
/// 2. Walk up looking for venv directories
pub fn find_python_site_packages(project_root: &Path) -> Option<PathBuf> {
    // Check .venv in project root first (most common with uv/poetry)
    let venv_dir = project_root.join(".venv");
    if venv_dir.is_dir() {
        if let Some(site_packages) = find_site_packages_in_venv(&venv_dir) {
            return Some(site_packages);
        }
    }

    // Check venv (alternative name)
    let venv_dir = project_root.join("venv");
    if venv_dir.is_dir() {
        if let Some(site_packages) = find_site_packages_in_venv(&venv_dir) {
            return Some(site_packages);
        }
    }

    // Check .venv in parent directories
    let mut current = project_root.to_path_buf();
    while let Some(parent) = current.parent() {
        let venv_dir = parent.join(".venv");
        if venv_dir.is_dir() {
            if let Some(site_packages) = find_site_packages_in_venv(&venv_dir) {
                return Some(site_packages);
            }
        }
        current = parent.to_path_buf();
    }

    None
}

/// Find site-packages within a venv directory.
fn find_site_packages_in_venv(venv: &Path) -> Option<PathBuf> {
    // Unix: lib/pythonX.Y/site-packages
    let lib_dir = venv.join("lib");
    if lib_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&lib_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.starts_with("python") {
                    let site_packages = entry.path().join("site-packages");
                    if site_packages.is_dir() {
                        return Some(site_packages);
                    }
                }
            }
        }
    }

    // Windows: Lib/site-packages
    let lib_dir = venv.join("Lib").join("site-packages");
    if lib_dir.is_dir() {
        return Some(lib_dir);
    }

    None
}

/// Resolve a Python import to its source location.
///
/// Handles:
/// - Package imports (requests -> requests/__init__.py)
/// - Module imports (six -> six.py)
/// - Submodule imports (requests.api -> requests/api.py)
/// - Namespace packages (no __init__.py)
pub fn resolve_python_import(import_name: &str, site_packages: &Path) -> Option<ResolvedPackage> {
    // Split on dots for submodule resolution
    let parts: Vec<&str> = import_name.split('.').collect();
    let top_level = parts[0];

    // Check for package (directory)
    let pkg_dir = site_packages.join(top_level);
    if pkg_dir.is_dir() {
        if parts.len() == 1 {
            // Just the package - look for __init__.py
            let init = pkg_dir.join("__init__.py");
            if init.is_file() {
                return Some(ResolvedPackage {
                    path: pkg_dir,
                    name: import_name.to_string(),
                    is_namespace: false,
                });
            }
            // Namespace package (no __init__.py)
            return Some(ResolvedPackage {
                path: pkg_dir,
                name: import_name.to_string(),
                is_namespace: true,
            });
        } else {
            // Submodule - build path
            let mut path = pkg_dir.clone();
            for part in &parts[1..] {
                path = path.join(part);
            }

            // Try as package first
            if path.is_dir() {
                let init = path.join("__init__.py");
                return Some(ResolvedPackage {
                    path: path.clone(),
                    name: import_name.to_string(),
                    is_namespace: !init.is_file(),
                });
            }

            // Try as module
            let py_file = path.with_extension("py");
            if py_file.is_file() {
                return Some(ResolvedPackage {
                    path: py_file,
                    name: import_name.to_string(),
                    is_namespace: false,
                });
            }

            return None;
        }
    }

    // Check for single-file module
    let py_file = site_packages.join(format!("{}.py", top_level));
    if py_file.is_file() {
        return Some(ResolvedPackage {
            path: py_file,
            name: import_name.to_string(),
            is_namespace: false,
        });
    }

    None
}

// =============================================================================
// Go
// =============================================================================

/// Find Go stdlib directory (GOROOT/src).
pub fn find_go_stdlib() -> Option<PathBuf> {
    // Try GOROOT env var
    if let Ok(goroot) = std::env::var("GOROOT") {
        let src = PathBuf::from(goroot).join("src");
        if src.is_dir() {
            return Some(src);
        }
    }

    // Try `go env GOROOT`
    if let Ok(output) = Command::new("go").args(["env", "GOROOT"]).output() {
        if output.status.success() {
            let goroot = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let src = PathBuf::from(goroot).join("src");
            if src.is_dir() {
                return Some(src);
            }
        }
    }

    // Common locations
    for path in &["/usr/local/go/src", "/usr/lib/go/src", "/opt/go/src"] {
        let src = PathBuf::from(path);
        if src.is_dir() {
            return Some(src);
        }
    }

    None
}

/// Check if a Go import is a stdlib import (no dots in first path segment).
pub fn is_go_stdlib_import(import_path: &str) -> bool {
    let first_segment = import_path.split('/').next().unwrap_or(import_path);
    !first_segment.contains('.')
}

/// Resolve a Go stdlib import to its source location.
pub fn resolve_go_stdlib_import(import_path: &str, stdlib_path: &Path) -> Option<ResolvedPackage> {
    if !is_go_stdlib_import(import_path) {
        return None;
    }

    let pkg_dir = stdlib_path.join(import_path);
    if pkg_dir.is_dir() {
        return Some(ResolvedPackage {
            path: pkg_dir,
            name: import_path.to_string(),
            is_namespace: false,
        });
    }

    None
}

/// Find Go module cache directory.
///
/// Uses GOMODCACHE env var, falls back to ~/go/pkg/mod
pub fn find_go_mod_cache() -> Option<PathBuf> {
    // Check GOMODCACHE env var
    if let Ok(cache) = std::env::var("GOMODCACHE") {
        let path = PathBuf::from(cache);
        if path.is_dir() {
            return Some(path);
        }
    }

    // Fall back to ~/go/pkg/mod using HOME env var
    if let Ok(home) = std::env::var("HOME") {
        let mod_cache = PathBuf::from(home).join("go").join("pkg").join("mod");
        if mod_cache.is_dir() {
            return Some(mod_cache);
        }
    }

    // Windows fallback
    if let Ok(home) = std::env::var("USERPROFILE") {
        let mod_cache = PathBuf::from(home).join("go").join("pkg").join("mod");
        if mod_cache.is_dir() {
            return Some(mod_cache);
        }
    }

    None
}

/// Resolve a Go import to its source location.
///
/// Import paths like "github.com/user/repo/pkg" are mapped to
/// $GOMODCACHE/github.com/user/repo@version/pkg
pub fn resolve_go_import(import_path: &str, mod_cache: &Path) -> Option<ResolvedPackage> {
    // Skip standard library imports (no dots in first segment)
    let first_segment = import_path.split('/').next()?;
    if !first_segment.contains('.') {
        // This is stdlib (fmt, os, etc.) - not in mod cache
        return None;
    }

    // Find the module in cache
    // Import path: github.com/user/repo/internal/pkg
    // Cache path: github.com/user/repo@v1.2.3/internal/pkg

    // We need to find the right version directory
    // Start with the full path and try progressively shorter prefixes
    let parts: Vec<&str> = import_path.split('/').collect();

    for i in (2..=parts.len()).rev() {
        let module_prefix = parts[..i].join("/");
        let module_dir = mod_cache.join(&module_prefix);

        // The parent directory might contain version directories
        if let Some(parent) = module_dir.parent() {
            if parent.is_dir() {
                // Look for versioned directories matching this module
                let module_name = module_dir.file_name()?.to_string_lossy();
                if let Ok(entries) = std::fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        let name = entry.file_name();
                        let name_str = name.to_string_lossy();
                        // Match module@version pattern
                        if name_str.starts_with(&format!("{}@", module_name)) {
                            let versioned_path = entry.path();
                            // Add remaining path components
                            let remainder = if i < parts.len() {
                                parts[i..].join("/")
                            } else {
                                String::new()
                            };
                            let full_path = if remainder.is_empty() {
                                versioned_path.clone()
                            } else {
                                versioned_path.join(&remainder)
                            };

                            if full_path.is_dir() {
                                return Some(ResolvedPackage {
                                    path: full_path,
                                    name: import_path.to_string(),
                                    is_namespace: false,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

// =============================================================================
// Global Package Index Database
// =============================================================================

use rusqlite::{Connection, params};

/// Parsed version as (major, minor).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
}

impl Version {
    /// Parse "3.11" into Version { major: 3, minor: 11 }.
    pub fn parse(s: &str) -> Option<Version> {
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() >= 2 {
            Some(Version {
                major: parts[0].parse().ok()?,
                minor: parts[1].parse().ok()?,
            })
        } else {
            None
        }
    }

    /// Check if this version is within a range [min, max].
    /// If max is None, only checks >= min.
    pub fn in_range(&self, min: Version, max: Option<Version>) -> bool {
        if *self < min {
            return false;
        }
        if let Some(max) = max {
            if *self > max {
                return false;
            }
        }
        true
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.major.cmp(&other.major) {
            std::cmp::Ordering::Equal => self.minor.cmp(&other.minor),
            ord => ord,
        }
    }
}

/// A package record in the index.
#[derive(Debug, Clone)]
pub struct PackageRecord {
    pub id: i64,
    pub language: String,
    pub name: String,
    pub path: String,
    pub min_major: u32,
    pub min_minor: u32,
    pub max_major: Option<u32>,
    pub max_minor: Option<u32>,
}

impl PackageRecord {
    pub fn min_version(&self) -> Version {
        Version { major: self.min_major, minor: self.min_minor }
    }

    pub fn max_version(&self) -> Option<Version> {
        match (self.max_major, self.max_minor) {
            (Some(major), Some(minor)) => Some(Version { major, minor }),
            _ => None,
        }
    }
}

/// A symbol record in the index.
#[derive(Debug, Clone)]
pub struct SymbolRecord {
    pub id: i64,
    pub package_id: i64,
    pub name: String,
    pub kind: String,
    pub signature: String,
    pub line: u32,
}

/// Global package index backed by SQLite.
pub struct PackageIndex {
    conn: Connection,
}

impl PackageIndex {
    /// Open or create the global package index.
    pub fn open() -> Result<Self, rusqlite::Error> {
        let db_path = get_global_packages_db()
            .ok_or_else(|| rusqlite::Error::InvalidPath("Cannot determine cache directory".into()))?;

        let conn = Connection::open(&db_path)?;
        let index = PackageIndex { conn };
        index.init_schema()?;
        Ok(index)
    }

    /// Open an in-memory database (for testing).
    pub fn open_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        let index = PackageIndex { conn };
        index.init_schema()?;
        Ok(index)
    }

    /// Initialize database schema.
    fn init_schema(&self) -> Result<(), rusqlite::Error> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS packages (
                id INTEGER PRIMARY KEY,
                language TEXT NOT NULL,
                name TEXT NOT NULL,
                path TEXT NOT NULL,
                min_major INTEGER NOT NULL,
                min_minor INTEGER NOT NULL,
                max_major INTEGER,
                max_minor INTEGER,
                indexed_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_packages_lang_name ON packages(language, name);

            CREATE TABLE IF NOT EXISTS symbols (
                id INTEGER PRIMARY KEY,
                package_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                signature TEXT NOT NULL,
                line INTEGER NOT NULL,
                FOREIGN KEY (package_id) REFERENCES packages(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_symbols_package ON symbols(package_id);
            CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
            "
        )?;
        Ok(())
    }

    /// Insert a package and return its ID.
    pub fn insert_package(
        &self,
        language: &str,
        name: &str,
        path: &str,
        min_version: Version,
        max_version: Option<Version>,
    ) -> Result<i64, rusqlite::Error> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.conn.execute(
            "INSERT INTO packages (language, name, path, min_major, min_minor, max_major, max_minor, indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                language,
                name,
                path,
                min_version.major,
                min_version.minor,
                max_version.map(|v| v.major),
                max_version.map(|v| v.minor),
                now,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Insert a symbol for a package.
    pub fn insert_symbol(
        &self,
        package_id: i64,
        name: &str,
        kind: &str,
        signature: &str,
        line: u32,
    ) -> Result<i64, rusqlite::Error> {
        self.conn.execute(
            "INSERT INTO symbols (package_id, name, kind, signature, line)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![package_id, name, kind, signature, line],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Find a package by language and name, optionally filtering by version.
    pub fn find_package(
        &self,
        language: &str,
        name: &str,
        version: Option<Version>,
    ) -> Result<Option<PackageRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, language, name, path, min_major, min_minor, max_major, max_minor
             FROM packages WHERE language = ?1 AND name = ?2"
        )?;

        let packages: Vec<PackageRecord> = stmt.query_map(params![language, name], |row| {
            Ok(PackageRecord {
                id: row.get(0)?,
                language: row.get(1)?,
                name: row.get(2)?,
                path: row.get(3)?,
                min_major: row.get(4)?,
                min_minor: row.get(5)?,
                max_major: row.get(6)?,
                max_minor: row.get(7)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        // Filter by version in Rust (simpler than complex SQL)
        if let Some(ver) = version {
            for pkg in packages {
                if ver.in_range(pkg.min_version(), pkg.max_version()) {
                    return Ok(Some(pkg));
                }
            }
            Ok(None)
        } else {
            Ok(packages.into_iter().next())
        }
    }

    /// Get all symbols for a package.
    pub fn get_symbols(&self, package_id: i64) -> Result<Vec<SymbolRecord>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT id, package_id, name, kind, signature, line
             FROM symbols WHERE package_id = ?1 ORDER BY line"
        )?;

        let symbols = stmt.query_map(params![package_id], |row| {
            Ok(SymbolRecord {
                id: row.get(0)?,
                package_id: row.get(1)?,
                name: row.get(2)?,
                kind: row.get(3)?,
                signature: row.get(4)?,
                line: row.get(5)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(symbols)
    }

    /// Find a symbol by name across all packages for a language.
    pub fn find_symbol(
        &self,
        language: &str,
        symbol_name: &str,
        version: Option<Version>,
    ) -> Result<Vec<(PackageRecord, SymbolRecord)>, rusqlite::Error> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.language, p.name, p.path, p.min_major, p.min_minor, p.max_major, p.max_minor,
                    s.id, s.package_id, s.name, s.kind, s.signature, s.line
             FROM symbols s
             JOIN packages p ON s.package_id = p.id
             WHERE p.language = ?1 AND s.name = ?2"
        )?;

        let results: Vec<(PackageRecord, SymbolRecord)> = stmt.query_map(params![language, symbol_name], |row| {
            Ok((
                PackageRecord {
                    id: row.get(0)?,
                    language: row.get(1)?,
                    name: row.get(2)?,
                    path: row.get(3)?,
                    min_major: row.get(4)?,
                    min_minor: row.get(5)?,
                    max_major: row.get(6)?,
                    max_minor: row.get(7)?,
                },
                SymbolRecord {
                    id: row.get(8)?,
                    package_id: row.get(9)?,
                    name: row.get(10)?,
                    kind: row.get(11)?,
                    signature: row.get(12)?,
                    line: row.get(13)?,
                },
            ))
        })?.collect::<Result<Vec<_>, _>>()?;

        // Filter by version
        if let Some(ver) = version {
            Ok(results.into_iter()
                .filter(|(pkg, _)| ver.in_range(pkg.min_version(), pkg.max_version()))
                .collect())
        } else {
            Ok(results)
        }
    }

    /// Check if a package is already indexed.
    pub fn is_indexed(&self, language: &str, name: &str) -> Result<bool, rusqlite::Error> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM packages WHERE language = ?1 AND name = ?2",
            params![language, name],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Delete a package and its symbols.
    pub fn delete_package(&self, package_id: i64) -> Result<(), rusqlite::Error> {
        self.conn.execute("DELETE FROM symbols WHERE package_id = ?1", params![package_id])?;
        self.conn.execute("DELETE FROM packages WHERE id = ?1", params![package_id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parsing() {
        assert_eq!(Version::parse("3.11"), Some(Version { major: 3, minor: 11 }));
        assert_eq!(Version::parse("1.21"), Some(Version { major: 1, minor: 21 }));
        assert_eq!(Version::parse("invalid"), None);
    }

    #[test]
    fn test_version_comparison() {
        let v39 = Version { major: 3, minor: 9 };
        let v310 = Version { major: 3, minor: 10 };
        let v311 = Version { major: 3, minor: 11 };

        assert!(v39 < v310);
        assert!(v310 < v311);
        assert!(v311 > v39);
    }

    #[test]
    fn test_version_in_range() {
        let v310 = Version { major: 3, minor: 10 };
        let min = Version { major: 3, minor: 9 };
        let max = Version { major: 3, minor: 12 };

        assert!(v310.in_range(min, Some(max)));
        assert!(v310.in_range(min, None));
        assert!(!Version { major: 3, minor: 8 }.in_range(min, Some(max)));
        assert!(!Version { major: 3, minor: 13 }.in_range(min, Some(max)));
    }

    #[test]
    fn test_package_index() {
        let index = PackageIndex::open_in_memory().unwrap();

        // Insert a package
        let pkg_id = index.insert_package(
            "python",
            "requests",
            "/path/to/requests",
            Version { major: 3, minor: 8 },
            Some(Version { major: 3, minor: 12 }),
        ).unwrap();

        // Insert symbols
        index.insert_symbol(pkg_id, "get", "function", "def get(url, **kwargs) -> Response", 42).unwrap();
        index.insert_symbol(pkg_id, "post", "function", "def post(url, **kwargs) -> Response", 100).unwrap();

        // Find package
        let found = index.find_package("python", "requests", Some(Version { major: 3, minor: 10 })).unwrap();
        assert!(found.is_some());
        let pkg = found.unwrap();
        assert_eq!(pkg.name, "requests");

        // Find with wrong version
        let found = index.find_package("python", "requests", Some(Version { major: 2, minor: 7 })).unwrap();
        assert!(found.is_none());

        // Get symbols
        let symbols = index.get_symbols(pkg_id).unwrap();
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "get");

        // Find symbol by name
        let results = index.find_symbol("python", "get", None).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.name, "requests");
        assert_eq!(results[0].1.name, "get");
    }

    #[test]
    fn test_find_site_packages() {
        // Test with current project (has .venv)
        let root = std::env::current_dir().unwrap();
        let site_packages = find_python_site_packages(&root);
        // This test assumes we're running from moss project root with .venv
        if root.join(".venv").exists() {
            assert!(site_packages.is_some());
            let sp = site_packages.unwrap();
            assert!(sp.to_string_lossy().contains("site-packages"));
        }
    }

    #[test]
    fn test_resolve_python_import() {
        let root = std::env::current_dir().unwrap();
        if let Some(site_packages) = find_python_site_packages(&root) {
            // Try to resolve a common package
            if let Some(pkg) = resolve_python_import("pathlib", &site_packages) {
                // pathlib might be stdlib, skip
                let _ = pkg;
            }

            // Try requests if installed
            if let Some(pkg) = resolve_python_import("ruff", &site_packages) {
                assert!(pkg.path.exists());
                assert_eq!(pkg.name, "ruff");
            }
        }
    }
}
