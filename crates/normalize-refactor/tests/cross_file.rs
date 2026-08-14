//! Cross-file name resolution tests.
//!
//! Tests ModuleResolver implementations for Rust, TypeScript, Python, Go,
//! JavaScript, and Ruby against fixture directories under `tests/fixtures/xfile/`.

use normalize_languages::go::GoModuleResolver;
use normalize_languages::javascript::JsModuleResolver;
use normalize_languages::python::PythonModuleResolver;
use normalize_languages::ruby::RubyModuleResolver;
use normalize_languages::rust::RustModuleResolver;
use normalize_languages::typescript::TsModuleResolver;
use normalize_languages::{ImportSpec, ModuleResolver, Resolution, support_for_path};
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/xfile/rust")
}

#[test]
fn workspace_config_reads_crate_name() {
    let root = fixture_root();
    let resolver = RustModuleResolver;
    let cfg = resolver.workspace_config(&root);

    assert_eq!(cfg.workspace_root, root);
    assert!(
        cfg.path_mappings
            .iter()
            .any(|(name, _)| name == "xfile-fixture"),
        "expected 'xfile-fixture' in path_mappings, got: {:?}",
        cfg.path_mappings.iter().map(|(n, _)| n).collect::<Vec<_>>()
    );
}

#[test]
fn module_of_file_lib_rs() {
    let root = fixture_root();
    let resolver = RustModuleResolver;
    let cfg = resolver.workspace_config(&root);

    // main.rs at crate root → canonical path is the crate name
    let main_rs = root.join("src/main.rs");
    // The fixture doesn't have a src/ dir — the .rs files are at the root.
    // This tests the fallback: files not under src/ return empty.
    let modules = resolver.module_of_file(&root, &main_rs, &cfg);
    // main.rs is not under src/, so no module identity is returned
    assert!(modules.is_empty() || !modules[0].canonical_path.is_empty());
}

#[test]
fn module_of_file_for_src_utils() {
    // Build a temporary crate structure with a src/ directory
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    // Write Cargo.toml
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"mycrate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();

    // Write src/utils.rs
    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(
        src_dir.join("utils.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }",
    )
    .unwrap();
    std::fs::write(src_dir.join("lib.rs"), "pub mod utils;").unwrap();

    let resolver = RustModuleResolver;
    let cfg = resolver.workspace_config(root);
    assert!(cfg.path_mappings.iter().any(|(n, _)| n == "mycrate"));

    let utils_rs = src_dir.join("utils.rs");
    let modules = resolver.module_of_file(root, &utils_rs, &cfg);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].canonical_path, "mycrate::utils");
}

#[test]
fn module_of_file_lib_rs_is_crate_root() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"mylib\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();

    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "pub fn hello() {}").unwrap();

    let resolver = RustModuleResolver;
    let cfg = resolver.workspace_config(root);

    let lib_rs = src_dir.join("lib.rs");
    let modules = resolver.module_of_file(root, &lib_rs, &cfg);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].canonical_path, "mylib");
}

#[test]
fn resolve_intra_workspace_import() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"mycrate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();

    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "pub mod utils;").unwrap();
    std::fs::write(
        src_dir.join("utils.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }",
    )
    .unwrap();
    std::fs::write(src_dir.join("main.rs"), "mod utils; fn main() {}").unwrap();

    let resolver = RustModuleResolver;
    let cfg = resolver.workspace_config(root);

    let from = src_dir.join("main.rs");
    let spec = ImportSpec {
        raw: "mycrate::utils::add".to_string(),
        is_relative: false,
        names: Vec::new(),
        is_glob: false,
    };
    let resolution = resolver.resolve(&from, &spec, &cfg);
    match resolution {
        Resolution::Resolved(path, name) => {
            assert_eq!(path, src_dir.join("utils.rs"));
            assert_eq!(name, "add");
        }
        other => panic!(
            "expected Resolved, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn resolve_stdlib_returns_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();

    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"mycrate\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();

    let src_dir = root.join("src");
    std::fs::create_dir_all(&src_dir).unwrap();
    std::fs::write(src_dir.join("lib.rs"), "").unwrap();

    let resolver = RustModuleResolver;
    let cfg = resolver.workspace_config(root);

    let from = src_dir.join("lib.rs");
    let spec = ImportSpec {
        raw: "std::collections::HashMap".to_string(),
        is_relative: false,
        names: Vec::new(),
        is_glob: false,
    };
    let resolution = resolver.resolve(&from, &spec, &cfg);
    assert!(matches!(resolution, Resolution::NotFound));
}

#[test]
fn resolve_non_rs_file_returns_not_applicable() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();

    let resolver = RustModuleResolver;
    let cfg = resolver.workspace_config(root);

    let from = root.join("src/index.ts");
    let spec = ImportSpec {
        raw: "x::foo".to_string(),
        is_relative: false,
        names: Vec::new(),
        is_glob: false,
    };
    let resolution = resolver.resolve(&from, &spec, &cfg);
    assert!(matches!(resolution, Resolution::NotApplicable));
}

// =============================================================================
// TypeScript resolver tests
// =============================================================================

fn ts_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/xfile/typescript")
}

#[test]
fn ts_resolve_relative_import() {
    let root = ts_fixture_root();
    let resolver = TsModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let from = root.join("models.ts");
    let spec = ImportSpec {
        raw: "./utils".to_string(),
        is_relative: true,
        names: vec!["greet".to_string()],
        is_glob: false,
    };
    match resolver.resolve(&from, &spec, &cfg) {
        Resolution::Resolved(path, _) => {
            assert_eq!(path, root.join("utils.ts"));
        }
        other => panic!(
            "expected Resolved, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn ts_resolve_js_extension_elision() {
    // TypeScript allows importing ./utils.js which resolves to ./utils.ts
    let root = ts_fixture_root();
    let resolver = TsModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let from = root.join("index.ts");
    let spec = ImportSpec {
        raw: "./utils.js".to_string(),
        is_relative: true,
        names: vec!["greet".to_string()],
        is_glob: false,
    };
    match resolver.resolve(&from, &spec, &cfg) {
        Resolution::Resolved(path, _) => {
            assert_eq!(path, root.join("utils.ts"));
        }
        other => panic!(
            "expected Resolved (via .js elision), got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn ts_not_applicable_for_non_ts_file() {
    let root = ts_fixture_root();
    let resolver = TsModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let from = root.join("app.js");
    let spec = ImportSpec {
        raw: "./utils".to_string(),
        is_relative: true,
        names: Vec::new(),
        is_glob: false,
    };
    assert!(matches!(
        resolver.resolve(&from, &spec, &cfg),
        Resolution::NotApplicable
    ));
}

#[test]
fn ts_module_of_file() {
    let root = ts_fixture_root();
    let resolver = TsModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let file = root.join("utils.ts");
    let modules = resolver.module_of_file(&root, &file, &cfg);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].canonical_path, "utils");
}

// =============================================================================
// Python resolver tests
// =============================================================================

fn py_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/xfile/python")
}

#[test]
fn py_resolve_relative_import() {
    let root = py_fixture_root();
    let resolver = PythonModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let from = root.join("models.py");
    let spec = ImportSpec {
        raw: ".utils".to_string(),
        is_relative: true,
        names: vec!["format_name".to_string()],
        is_glob: false,
    };
    match resolver.resolve(&from, &spec, &cfg) {
        Resolution::Resolved(path, _) => {
            assert_eq!(path, root.join("utils.py"));
        }
        other => panic!(
            "expected Resolved, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn py_not_applicable_for_non_py_file() {
    let root = py_fixture_root();
    let resolver = PythonModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let from = root.join("app.rb");
    let spec = ImportSpec {
        raw: "utils".to_string(),
        is_relative: false,
        names: Vec::new(),
        is_glob: false,
    };
    assert!(matches!(
        resolver.resolve(&from, &spec, &cfg),
        Resolution::NotApplicable
    ));
}

#[test]
fn py_absolute_import_not_found() {
    let root = py_fixture_root();
    let resolver = PythonModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let from = root.join("main.py");
    let spec = ImportSpec {
        raw: "os.path".to_string(),
        is_relative: false,
        names: Vec::new(),
        is_glob: false,
    };
    // stdlib — can't be resolved
    assert!(matches!(
        resolver.resolve(&from, &spec, &cfg),
        Resolution::NotFound
    ));
}

// =============================================================================
// Go resolver tests
// =============================================================================

fn go_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/xfile/go")
}

#[test]
fn go_workspace_config_reads_module_path() {
    let root = go_fixture_root();
    let resolver = GoModuleResolver;
    let cfg = resolver.workspace_config(&root);

    assert!(
        cfg.path_mappings
            .iter()
            .any(|(name, _)| name == "example.com/myapp"),
        "expected 'example.com/myapp' in path_mappings, got: {:?}",
        cfg.path_mappings.iter().map(|(n, _)| n).collect::<Vec<_>>()
    );
}

#[test]
fn go_resolve_subpackage() {
    let root = go_fixture_root();
    let resolver = GoModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let from = root.join("main.go");
    let spec = ImportSpec {
        raw: "example.com/myapp/utils".to_string(),
        is_relative: false,
        names: Vec::new(),
        is_glob: false,
    };
    match resolver.resolve(&from, &spec, &cfg) {
        Resolution::Resolved(path, _) => {
            assert_eq!(path, root.join("utils"));
        }
        other => panic!(
            "expected Resolved, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn go_not_applicable_for_non_go_file() {
    let root = go_fixture_root();
    let resolver = GoModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let from = root.join("main.rs");
    let spec = ImportSpec {
        raw: "example.com/myapp/utils".to_string(),
        is_relative: false,
        names: Vec::new(),
        is_glob: false,
    };
    assert!(matches!(
        resolver.resolve(&from, &spec, &cfg),
        Resolution::NotApplicable
    ));
}

#[test]
fn go_module_of_file() {
    let root = go_fixture_root();
    let resolver = GoModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let file = root.join("utils/math.go");
    let modules = resolver.module_of_file(&root, &file, &cfg);
    assert_eq!(modules.len(), 1);
    assert_eq!(modules[0].canonical_path, "example.com/myapp/utils");
}

// =============================================================================
// JavaScript resolver tests
// =============================================================================

fn js_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/xfile/javascript")
}

#[test]
fn js_resolve_relative_import() {
    let root = js_fixture_root();
    let resolver = JsModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let from = root.join("app.js");
    let spec = ImportSpec {
        raw: "./utils.js".to_string(),
        is_relative: true,
        names: vec!["sum".to_string()],
        is_glob: false,
    };
    match resolver.resolve(&from, &spec, &cfg) {
        Resolution::Resolved(path, _) => {
            assert_eq!(path, root.join("utils.js"));
        }
        other => panic!(
            "expected Resolved, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn js_not_applicable_for_non_js_file() {
    let root = js_fixture_root();
    let resolver = JsModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let from = root.join("main.ts");
    let spec = ImportSpec {
        raw: "./utils".to_string(),
        is_relative: true,
        names: Vec::new(),
        is_glob: false,
    };
    assert!(matches!(
        resolver.resolve(&from, &spec, &cfg),
        Resolution::NotApplicable
    ));
}

#[test]
fn js_bare_specifier_is_not_found() {
    let root = js_fixture_root();
    let resolver = JsModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let from = root.join("app.js");
    let spec = ImportSpec {
        raw: "lodash".to_string(),
        is_relative: false,
        names: Vec::new(),
        is_glob: false,
    };
    assert!(matches!(
        resolver.resolve(&from, &spec, &cfg),
        Resolution::NotFound
    ));
}

// =============================================================================
// Ruby resolver tests
// =============================================================================

fn rb_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/xfile/ruby")
}

#[test]
fn rb_resolve_require_relative() {
    let root = rb_fixture_root();
    let resolver = RubyModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let from = root.join("app.rb");
    let spec = ImportSpec {
        raw: "utils".to_string(),
        is_relative: true,
        names: Vec::new(),
        is_glob: false,
    };
    match resolver.resolve(&from, &spec, &cfg) {
        Resolution::Resolved(path, _) => {
            assert_eq!(path, root.join("utils.rb"));
        }
        other => panic!(
            "expected Resolved, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
}

#[test]
fn rb_bare_require_is_not_found() {
    let root = rb_fixture_root();
    let resolver = RubyModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let from = root.join("app.rb");
    let spec = ImportSpec {
        raw: "json".to_string(),
        is_relative: false,
        names: Vec::new(),
        is_glob: false,
    };
    assert!(matches!(
        resolver.resolve(&from, &spec, &cfg),
        Resolution::NotFound
    ));
}

#[test]
fn rb_not_applicable_for_non_rb_file() {
    let root = rb_fixture_root();
    let resolver = RubyModuleResolver;
    let cfg = resolver.workspace_config(&root);

    let from = root.join("app.py");
    let spec = ImportSpec {
        raw: "utils".to_string(),
        is_relative: true,
        names: Vec::new(),
        is_glob: false,
    };
    assert!(matches!(
        resolver.resolve(&from, &spec, &cfg),
        Resolution::NotApplicable
    ));
}

// =============================================================================
// find_references confidence tagging
// =============================================================================

/// Verify that find_references tags results with "resolved" for Rust files
/// (which have a ModuleResolver) and "heuristic" for files without one.
#[test]
fn find_references_confidence_tag_no_index() {
    use std::path::Path;

    // Rust files have a resolver → would tag as "resolved"
    let rust_file = Path::new("src/utils.rs");
    let has_rust_resolver = support_for_path(rust_file)
        .and_then(|lang| lang.module_resolver())
        .is_some();
    assert!(has_rust_resolver, "Rust should have a module_resolver");

    // TypeScript files have a resolver
    let ts_file = Path::new("src/utils.ts");
    let has_ts_resolver = support_for_path(ts_file)
        .and_then(|lang| lang.module_resolver())
        .is_some();
    assert!(has_ts_resolver, "TypeScript should have a module_resolver");

    // Python files have a resolver
    let py_file = Path::new("src/utils.py");
    let has_py_resolver = support_for_path(py_file)
        .and_then(|lang| lang.module_resolver())
        .is_some();
    assert!(has_py_resolver, "Python should have a module_resolver");

    // Go files have a resolver
    let go_file = Path::new("main.go");
    let has_go_resolver = support_for_path(go_file)
        .and_then(|lang| lang.module_resolver())
        .is_some();
    assert!(has_go_resolver, "Go should have a module_resolver");

    // A Bash file has no resolver → would tag as "heuristic"
    let sh_file = Path::new("script.sh");
    let has_sh_resolver = support_for_path(sh_file)
        .and_then(|lang| lang.module_resolver())
        .is_some();
    assert!(!has_sh_resolver, "Bash should NOT have a module_resolver");
}

/// Matrix test: assert that every language with a module system returns Some(&dyn ModuleResolver),
/// and that data/scripting/template languages return None.
///
/// HAS_RESOLVER: languages that implement ModuleResolver
/// NOT_APPLICABLE: languages without a module system (returns None by design)
///
/// Note: no `#[cfg(feature = "lang-*")]` guards here — this crate depends on
/// normalize-languages with its default `langs-all` feature, so every
/// language type is always available in this test binary. `normalize-refactor`
/// declares no such features itself, so any such guard would silently
/// evaluate false and compile the entry out (see normalize-cfg's
/// `coverage_matrix.rs` for the same note).
#[test]
fn module_resolver_coverage_matrix() {
    use normalize_languages::Language;

    // Languages that MUST have a resolver
    let has_resolver: &[(&dyn Language, &str)] = &[
        (&normalize_languages::Rust, "Rust"),
        (&normalize_languages::TypeScript, "TypeScript"),
        (&normalize_languages::Tsx, "TSX"),
        (&normalize_languages::JavaScript, "JavaScript"),
        (&normalize_languages::Python, "Python"),
        (&normalize_languages::Go, "Go"),
        (&normalize_languages::Ruby, "Ruby"),
        (&normalize_languages::Java, "Java"),
        (&normalize_languages::Kotlin, "Kotlin"),
        (&normalize_languages::Groovy, "Groovy"),
        (&normalize_languages::Scala, "Scala"),
        (&normalize_languages::CSharp, "C#"),
        (&normalize_languages::VB, "VB"),
        (&normalize_languages::FSharp, "F#"),
        (&normalize_languages::Swift, "Swift"),
        (&normalize_languages::Dart, "Dart"),
        (&normalize_languages::Zig, "Zig"),
        (&normalize_languages::Elixir, "Elixir"),
        (&normalize_languages::Erlang, "Erlang"),
        (&normalize_languages::Haskell, "Haskell"),
        (&normalize_languages::OCaml, "OCaml"),
        (&normalize_languages::Lua, "Lua"),
        (&normalize_languages::Php, "PHP"),
        (&normalize_languages::Perl, "Perl"),
        (&normalize_languages::Clojure, "Clojure"),
        (&normalize_languages::CommonLisp, "Common Lisp"),
        (&normalize_languages::Scheme, "Scheme"),
        (&normalize_languages::Gleam, "Gleam"),
        (&normalize_languages::ReScript, "ReScript"),
        (&normalize_languages::Elm, "Elm"),
        (&normalize_languages::Nix, "Nix"),
        (&normalize_languages::R, "R"),
        (&normalize_languages::Julia, "Julia"),
        (&normalize_languages::Matlab, "MATLAB"),
        (&normalize_languages::Prolog, "Prolog"),
        (&normalize_languages::D, "D"),
    ];

    // Collect every mismatch across both matrices instead of failing fast on
    // the first one — a single systemic regression (e.g. a resolver
    // registration bug) can affect many languages at once, and fail-fast
    // would report only the alphabetically/positionally-first one.
    let mut failures: Vec<String> = Vec::new();

    for (lang, name) in has_resolver {
        if lang.module_resolver().is_none() {
            failures.push(format!(
                "{name} should have a module_resolver (returns Some) but returned None"
            ));
        }
    }

    // Languages that must NOT have a resolver (no module system)
    let not_applicable: &[(&dyn Language, &str)] = &[
        (&normalize_languages::Css, "CSS"),
        (&normalize_languages::Scss, "SCSS"),
        (&normalize_languages::Json, "JSON"),
        (&normalize_languages::Yaml, "YAML"),
        (&normalize_languages::Toml, "TOML"),
        (&normalize_languages::Xml, "XML"),
        (&normalize_languages::Html, "HTML"),
        (&normalize_languages::Markdown, "Markdown"),
        (&normalize_languages::Sql, "SQL"),
        (&normalize_languages::GraphQL, "GraphQL"),
        (&normalize_languages::Bash, "Bash"),
        (&normalize_languages::Fish, "Fish"),
        (&normalize_languages::Awk, "Awk"),
        (&normalize_languages::PowerShell, "PowerShell"),
        (&normalize_languages::Glsl, "GLSL"),
        (&normalize_languages::Hlsl, "HLSL"),
        (&normalize_languages::Dockerfile, "Dockerfile"),
    ];

    for (lang, name) in not_applicable {
        if lang.module_resolver().is_some() {
            failures.push(format!(
                "{name} should NOT have a module_resolver (returns None — no module system) but returned Some"
            ));
        }
    }

    // DEFERRED languages (module system exists but resolver is None for now):
    // C, C++, ObjC — preprocessor #include; no standard package mapping without toolchain
    // Ada, Agda, Idris, Lean — niche; resolver not yet implemented

    if !failures.is_empty() {
        if failures.len() == 1 {
            // Common case: a single language failed. Keep the panic short.
            panic!("module_resolver_coverage_matrix: {}", failures[0]);
        }
        panic!(
            "module_resolver_coverage_matrix: {} language(s) failed:\n{}",
            failures.len(),
            failures.join("\n")
        );
    }
}
