//! Query fixture tests for cmake.
//!
//! Split out of the former monolithic `query_fixtures.rs`; see
//! `tests/common/mod.rs` for the shared helpers.
//!
//! These tests require compiled grammar `.so` files in `target/grammars/`.
//! Build them with `cargo xtask build-grammars`. Without grammars present the
//! tests skip gracefully.

mod common;

#[allow(unused_imports)]
use common::*;
use normalize_languages::GrammarLoader;
use tree_sitter::Parser;

// ---------------------------------------------------------------------------
// CMake
// ---------------------------------------------------------------------------

const CMAKE_SAMPLE: &str = include_str!("fixtures/cmake/CMakeLists.txt");

#[test]
fn cmake_tags_finds_functions_and_macros() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cmake_tags: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cmake").ok() else {
        eprintln!("Skipping cmake_tags: cmake grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("cmake").expect("cmake tags query missing");
    let names = collect_captures(&lang, CMAKE_SAMPLE, &query_str, "name");
    assert!(
        names.contains(&"add_component".to_string()),
        "expected 'add_component' function in cmake tags, got: {names:?}"
    );
    assert!(
        names
            .iter()
            .any(|n| n == "setup_target" || n == "install_component"),
        "expected 'setup_target' or 'install_component' in cmake tags, got: {names:?}"
    );
}

#[test]
fn cmake_calls_finds_command_calls() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cmake_calls: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cmake").ok() else {
        eprintln!("Skipping cmake_calls: cmake grammar .so not found");
        return;
    };
    let query_str = loader
        .get_calls("cmake")
        .expect("cmake calls query missing");
    let calls = collect_captures(&lang, CMAKE_SAMPLE, &query_str, "call");
    assert!(
        calls
            .iter()
            .any(|c| c == "find_package" || c == "add_library" || c == "target_link_libraries"),
        "expected cmake command calls in sample, got: {calls:?}"
    );
}

#[test]
fn cmake_complexity_finds_control_flow() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cmake_complexity: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cmake").ok() else {
        eprintln!("Skipping cmake_complexity: cmake grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("cmake")
        .expect("cmake complexity query missing");
    let complexity = collect_captures(&lang, CMAKE_SAMPLE, &query_str, "complexity");
    assert!(
        !complexity.is_empty(),
        "expected at least 1 complexity node in cmake sample, got: {complexity:?}"
    );
}

#[test]
fn cmake_imports_finds_includes_and_find_package() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cmake_imports: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cmake").ok() else {
        eprintln!("Skipping cmake_imports: cmake grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("cmake")
        .expect("cmake imports query missing");
    let paths = collect_captures(&lang, CMAKE_SAMPLE, &query_str, "import.path");
    assert!(
        paths
            .iter()
            .any(|p| p == "Threads" || p == "OpenSSL" || p == "GNUInstallDirs"),
        "expected 'Threads'/'OpenSSL'/'GNUInstallDirs' in cmake import paths, got: {paths:?}"
    );
}

const CMAKE_VARIANTS: &str = include_str!("fixtures/cmake/variants.cmake");

#[test]
fn cmake_tags_completeness_case_insensitive_definitions() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cmake_tags_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cmake").ok() else {
        eprintln!("Skipping cmake_tags_completeness: cmake grammar .so not found");
        return;
    };
    let query_str = loader.get_tags("cmake").expect("cmake tags query missing");
    let pairs = collect_tag_pairs(&lang, CMAKE_VARIANTS, &query_str);
    // function()/FUNCTION()/macro() all produce definition.function regardless
    // of command-name casing (CMake commands are case-insensitive; the grammar
    // assigns the same function_def/macro_def node type either way).
    for name in ["plain_function", "upper_function", "plain_macro"] {
        assert!(
            pairs
                .iter()
                .any(|(k, n)| k == "definition.function" && n == name),
            "expected (definition.function, {name}) in cmake tags variants, got: {pairs:?}"
        );
    }
}

#[test]
fn cmake_imports_completeness_case_insensitive_and_command_families() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cmake_imports_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cmake").ok() else {
        eprintln!("Skipping cmake_imports_completeness: cmake grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("cmake")
        .expect("cmake imports query missing");
    let paths = collect_captures(&lang, CMAKE_VARIANTS, &query_str, "import.path");

    // include()/INCLUDE() — case-insensitivity. The original query used a
    // case-sensitive #match? and matched 0 uppercase commands.
    assert!(
        paths.contains(&"LowercaseModule".to_string()),
        "expected lowercase include() path, got: {paths:?}"
    );
    assert!(
        paths.contains(&"UppercaseModule".to_string()),
        "expected uppercase INCLUDE() path (case-insensitive match), got: {paths:?}"
    );
    // find_package with a trailing REQUIRED keyword argument — only the
    // package name is a path, not the keyword.
    assert!(
        paths.contains(&"SomePackage".to_string()),
        "expected find_package path, got: {paths:?}"
    );
    // find_library/find_path/find_program — second argument, not the output
    // variable name (first argument).
    for name in ["foo_lib_name", "foo_header.h", "foo_program"] {
        assert!(
            paths.contains(&name.to_string()),
            "expected find_{{library,path,program}} search target '{name}', got: {paths:?}"
        );
    }
    // add_subdirectory — CMake's module/file-import analog, previously
    // entirely unhandled.
    for name in ["some_subdir", "other_subdir"] {
        assert!(
            paths.contains(&name.to_string()),
            "expected add_subdirectory path '{name}', got: {paths:?}"
        );
    }
    // FetchContent_MakeAvailable — modern external-dependency import idiom,
    // previously entirely unhandled.
    for name in ["some_dep", "other_dep"] {
        assert!(
            paths.contains(&name.to_string()),
            "expected FetchContent_MakeAvailable dep '{name}', got: {paths:?}"
        );
    }
}

#[test]
fn cmake_imports_negative_keyword_and_output_var_args_not_paths() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cmake_imports_negative: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cmake").ok() else {
        eprintln!("Skipping cmake_imports_negative: cmake grammar .so not found");
        return;
    };
    let query_str = loader
        .get_imports("cmake")
        .expect("cmake imports query missing");
    let paths = collect_captures(&lang, CMAKE_VARIANTS, &query_str, "import.path");

    // The original bug: an unanchored `(argument) @import.path` inside
    // find_package's argument_list matched every argument, so the trailing
    // REQUIRED keyword argument was captured as a bogus second path.
    assert!(
        !paths.contains(&"REQUIRED".to_string()),
        "'REQUIRED' keyword argument must not appear as @import.path, got: {paths:?}"
    );
    assert!(
        !paths.contains(&"EXCLUDE_FROM_ALL".to_string()),
        "'EXCLUDE_FROM_ALL' keyword argument must not appear as @import.path, got: {paths:?}"
    );
    // find_library/find_path/find_program's FIRST argument is the output
    // variable name, not the search target — must not be captured.
    for output_var in ["FOO_LIB_VAR", "FOO_INCLUDE_VAR", "FOO_PROGRAM_VAR"] {
        assert!(
            !paths.contains(&output_var.to_string()),
            "output variable '{output_var}' must not appear as @import.path, got: {paths:?}"
        );
    }
}

#[test]
fn cmake_complexity_completeness_scope_nesting() {
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cmake_complexity_completeness: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cmake").ok() else {
        eprintln!("Skipping cmake_complexity_completeness: cmake grammar .so not found");
        return;
    };
    let query_str = loader
        .get_complexity("cmake")
        .expect("cmake complexity query missing");
    let full = collect_captures_full(&lang, CMAKE_VARIANTS, &query_str);
    // function_def/macro_def/block_def all count as @nesting (scope
    // introduction), matching the cross-language convention already used by
    // rust.complexity.scm (function_item/impl_item/mod_item) and
    // python.complexity.scm (function_definition/class_definition). Assert
    // on capture *kind*, not just text, since block_def's text overlaps with
    // nothing else in this fixture but function_def/macro_def do share a
    // "definition" shape worth distinguishing.
    let nesting_kinds: Vec<&str> = full
        .iter()
        .filter(|(cap, ..)| cap == "nesting")
        .map(|(_, kind, ..)| kind.as_str())
        .collect();
    for expected_kind in ["function_def", "macro_def", "block_def"] {
        assert!(
            nesting_kinds.contains(&expected_kind),
            "expected a @nesting capture of kind '{expected_kind}', got kinds: {nesting_kinds:?}"
        );
    }
}

#[test]
fn cmake_node_name_finds_nested_function_and_macro_names() {
    // `node_name()` in cmake.rs is what the actual symbol-extraction pipeline
    // uses for a definition's name (not the tags query's own @name capture
    // text) — see normalize-facts::extract::build_symbol_from_def. The
    // previous implementation scanned only `function_def`'s DIRECT children
    // for an `argument` node, but arborium-cmake 2.17.0's node-types.json
    // shows `function_def`'s children are only
    // function_command/body/endfunction_command — the name argument is two
    // levels deeper (function_def -> function_command -> argument_list ->
    // argument). That made node_name() return None for every CMake
    // function/macro, so `normalize view <file>.cmake` reported zero symbols
    // for any file with function/macro definitions. Verified live via
    // `normalize view` against tests/fixtures/cmake/CMakeLists.txt before
    // and after the fix.
    let Some(gdir) = grammar_dir() else {
        eprintln!("Skipping cmake_node_name: run `cargo xtask build-grammars` first");
        return;
    };
    let loader = GrammarLoader::with_paths(vec![gdir]);
    let Some(lang) = loader.get("cmake").ok() else {
        eprintln!("Skipping cmake_node_name: cmake grammar .so not found");
        return;
    };
    let mut parser = Parser::new();
    parser.set_language(&lang).expect("set_language failed");
    let tree = parser.parse(CMAKE_VARIANTS, None).expect("parse failed");
    let support = normalize_languages::CMake;
    let root = tree.root_node();
    let mut cursor = root.walk();
    let mut found_function = false;
    let mut found_macro = false;
    for child in root.children(&mut cursor) {
        match child.kind() {
            "function_def" => {
                let name =
                    normalize_languages::Language::node_name(&support, &child, CMAKE_VARIANTS);
                if name == Some("plain_function") {
                    found_function = true;
                }
            }
            "macro_def" => {
                let name =
                    normalize_languages::Language::node_name(&support, &child, CMAKE_VARIANTS);
                assert_eq!(
                    name,
                    Some("plain_macro"),
                    "expected node_name to find 'plain_macro' via function_command -> \
                     argument_list -> argument, got: {name:?}"
                );
                found_macro = true;
            }
            _ => {}
        }
    }
    assert!(
        found_function,
        "no function_def with name 'plain_function' found"
    );
    assert!(found_macro, "no macro_def found in variants.cmake");
}
