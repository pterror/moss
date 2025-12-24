//! Trait-based external import resolution.
//!
//! Extends LanguageSupport with filesystem-based import resolution.

use std::path::{Path, PathBuf};

use moss_languages::LanguageSupport;

pub use crate::external_packages::ResolvedPackage;

/// Extension trait for languages that support external import resolution.
///
/// This is separate from LanguageSupport because it requires filesystem access.
pub trait ImportResolver: LanguageSupport {
    /// Resolve an external import to its source location.
    ///
    /// Returns None if:
    /// - The import can't be resolved
    /// - The import is internal to the project
    /// - The language doesn't support external imports
    fn resolve_import(&self, import_name: &str, project_root: &Path) -> Option<ResolvedPackage>;

    /// Check if an import is from the standard library.
    fn is_stdlib_import(&self, import_name: &str, project_root: &Path) -> bool {
        let _ = (import_name, project_root);
        false
    }

    /// Get the language/runtime version (for package index versioning).
    fn get_version(&self, project_root: &Path) -> Option<String> {
        let _ = project_root;
        None
    }

    /// Find package cache/installation directory.
    fn find_package_cache(&self, project_root: &Path) -> Option<PathBuf> {
        let _ = project_root;
        None
    }
}

// =============================================================================
// Python
// =============================================================================

impl ImportResolver for moss_languages::Python {
    fn resolve_import(&self, import_name: &str, project_root: &Path) -> Option<ResolvedPackage> {
        use crate::external_packages;

        // Check stdlib first
        if let Some(stdlib) = external_packages::find_python_stdlib(project_root) {
            if let Some(pkg) = external_packages::resolve_python_stdlib_import(import_name, &stdlib) {
                return Some(pkg);
            }
        }

        // Then site-packages
        if let Some(site_packages) = external_packages::find_python_site_packages(project_root) {
            return external_packages::resolve_python_import(import_name, &site_packages);
        }

        None
    }

    fn is_stdlib_import(&self, import_name: &str, project_root: &Path) -> bool {
        use crate::external_packages;

        if let Some(stdlib) = external_packages::find_python_stdlib(project_root) {
            external_packages::is_python_stdlib_module(import_name, &stdlib)
        } else {
            false
        }
    }

    fn get_version(&self, project_root: &Path) -> Option<String> {
        crate::external_packages::get_python_version(project_root)
    }

    fn find_package_cache(&self, project_root: &Path) -> Option<PathBuf> {
        crate::external_packages::find_python_site_packages(project_root)
    }
}

// =============================================================================
// Go
// =============================================================================

impl ImportResolver for moss_languages::Go {
    fn resolve_import(&self, import_name: &str, project_root: &Path) -> Option<ResolvedPackage> {
        use crate::external_packages;
        let _ = project_root;

        // Check stdlib first
        if external_packages::is_go_stdlib_import(import_name) {
            if let Some(stdlib) = external_packages::find_go_stdlib() {
                if let Some(pkg) = external_packages::resolve_go_stdlib_import(import_name, &stdlib) {
                    return Some(pkg);
                }
            }
        }

        // Then mod cache
        if let Some(mod_cache) = external_packages::find_go_mod_cache() {
            return external_packages::resolve_go_import(import_name, &mod_cache);
        }

        None
    }

    fn is_stdlib_import(&self, import_name: &str, _project_root: &Path) -> bool {
        crate::external_packages::is_go_stdlib_import(import_name)
    }

    fn get_version(&self, _project_root: &Path) -> Option<String> {
        crate::external_packages::get_go_version()
    }

    fn find_package_cache(&self, _project_root: &Path) -> Option<PathBuf> {
        crate::external_packages::find_go_mod_cache()
    }
}

// =============================================================================
// JavaScript / TypeScript
// =============================================================================

impl ImportResolver for moss_languages::JavaScript {
    fn resolve_import(&self, import_name: &str, project_root: &Path) -> Option<ResolvedPackage> {
        use crate::external_packages;

        // Skip relative imports
        if import_name.starts_with('.') || import_name.starts_with('/') {
            return None;
        }

        let node_modules = external_packages::find_node_modules(project_root)?;
        external_packages::resolve_node_import(import_name, &node_modules)
    }

    fn get_version(&self, _project_root: &Path) -> Option<String> {
        crate::external_packages::get_node_version()
    }

    fn find_package_cache(&self, project_root: &Path) -> Option<PathBuf> {
        crate::external_packages::find_node_modules(project_root)
    }
}

impl ImportResolver for moss_languages::TypeScript {
    fn resolve_import(&self, import_name: &str, project_root: &Path) -> Option<ResolvedPackage> {
        use crate::external_packages;

        if import_name.starts_with('.') || import_name.starts_with('/') {
            return None;
        }

        let node_modules = external_packages::find_node_modules(project_root)?;
        external_packages::resolve_node_import(import_name, &node_modules)
    }

    fn get_version(&self, _project_root: &Path) -> Option<String> {
        crate::external_packages::get_node_version()
    }

    fn find_package_cache(&self, project_root: &Path) -> Option<PathBuf> {
        crate::external_packages::find_node_modules(project_root)
    }
}

// =============================================================================
// Rust
// =============================================================================

impl ImportResolver for moss_languages::Rust {
    fn resolve_import(&self, crate_name: &str, _project_root: &Path) -> Option<ResolvedPackage> {
        use crate::external_packages;

        let registry = external_packages::find_cargo_registry()?;
        external_packages::resolve_rust_crate(crate_name, &registry)
    }

    fn get_version(&self, _project_root: &Path) -> Option<String> {
        crate::external_packages::get_rust_version()
    }

    fn find_package_cache(&self, _project_root: &Path) -> Option<PathBuf> {
        crate::external_packages::find_cargo_registry()
    }
}

// =============================================================================
// C / C++
// =============================================================================

impl ImportResolver for moss_languages::C {
    fn resolve_import(&self, include: &str, _project_root: &Path) -> Option<ResolvedPackage> {
        use crate::external_packages;

        let include_paths = external_packages::find_cpp_include_paths();
        external_packages::resolve_cpp_include(include, &include_paths)
    }

    fn get_version(&self, _project_root: &Path) -> Option<String> {
        crate::external_packages::get_gcc_version()
    }
}

impl ImportResolver for moss_languages::Cpp {
    fn resolve_import(&self, include: &str, _project_root: &Path) -> Option<ResolvedPackage> {
        use crate::external_packages;

        let include_paths = external_packages::find_cpp_include_paths();
        external_packages::resolve_cpp_include(include, &include_paths)
    }

    fn get_version(&self, _project_root: &Path) -> Option<String> {
        crate::external_packages::get_gcc_version()
    }
}

// =============================================================================
// Java
// =============================================================================

impl ImportResolver for moss_languages::Java {
    fn resolve_import(&self, import_name: &str, _project_root: &Path) -> Option<ResolvedPackage> {
        use crate::external_packages;

        let maven_repo = external_packages::find_maven_repository();
        let gradle_cache = external_packages::find_gradle_cache();

        external_packages::resolve_java_import(
            import_name,
            maven_repo.as_deref(),
            gradle_cache.as_deref(),
        )
    }

    fn get_version(&self, _project_root: &Path) -> Option<String> {
        crate::external_packages::get_java_version()
    }

    fn find_package_cache(&self, _project_root: &Path) -> Option<PathBuf> {
        crate::external_packages::find_maven_repository()
            .or_else(crate::external_packages::find_gradle_cache)
    }
}

// =============================================================================
// Dispatch helper
// =============================================================================

/// Resolve an import for any supported language.
///
/// Uses the file extension to determine the language and dispatch to the
/// appropriate resolver.
pub fn resolve_import(
    file_path: &Path,
    import_name: &str,
    project_root: &Path,
) -> Option<ResolvedPackage> {
    let ext = file_path.extension()?.to_str()?;

    match ext {
        "py" | "pyi" | "pyw" => moss_languages::Python.resolve_import(import_name, project_root),
        "go" => moss_languages::Go.resolve_import(import_name, project_root),
        "js" | "mjs" | "cjs" | "jsx" => {
            moss_languages::JavaScript.resolve_import(import_name, project_root)
        }
        "ts" | "mts" | "cts" | "tsx" => {
            moss_languages::TypeScript.resolve_import(import_name, project_root)
        }
        "rs" => moss_languages::Rust.resolve_import(import_name, project_root),
        "c" | "h" => moss_languages::C.resolve_import(import_name, project_root),
        "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => {
            moss_languages::Cpp.resolve_import(import_name, project_root)
        }
        "java" => moss_languages::Java.resolve_import(import_name, project_root),
        _ => None,
    }
}

/// Check if an import is from the standard library.
pub fn is_stdlib_import(file_path: &Path, import_name: &str, project_root: &Path) -> bool {
    let ext = match file_path.extension().and_then(|e| e.to_str()) {
        Some(e) => e,
        None => return false,
    };

    match ext {
        "py" | "pyi" | "pyw" => moss_languages::Python.is_stdlib_import(import_name, project_root),
        "go" => moss_languages::Go.is_stdlib_import(import_name, project_root),
        _ => false,
    }
}

/// Get the language/runtime version for a file.
pub fn get_language_version(file_path: &Path, project_root: &Path) -> Option<String> {
    let ext = file_path.extension()?.to_str()?;

    match ext {
        "py" | "pyi" | "pyw" => moss_languages::Python.get_version(project_root),
        "go" => moss_languages::Go.get_version(project_root),
        "js" | "mjs" | "cjs" | "jsx" => moss_languages::JavaScript.get_version(project_root),
        "ts" | "mts" | "cts" | "tsx" => moss_languages::TypeScript.get_version(project_root),
        "rs" => moss_languages::Rust.get_version(project_root),
        "c" | "h" => moss_languages::C.get_version(project_root),
        "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => moss_languages::Cpp.get_version(project_root),
        "java" => moss_languages::Java.get_version(project_root),
        _ => None,
    }
}
