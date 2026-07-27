//! Language support registry with extension-based lookup.

use crate::Language;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{OnceLock, RwLock};

/// Global language registry.
static LANGUAGES: RwLock<Vec<&'static dyn Language>> = RwLock::new(Vec::new());
static INITIALIZED: OnceLock<()> = OnceLock::new();

/// Cached extension → language lookup table.
static EXTENSION_MAP: OnceLock<HashMap<&'static str, &'static dyn Language>> = OnceLock::new();

/// Cached grammar_name → language lookup table.
static GRAMMAR_MAP: OnceLock<HashMap<&'static str, &'static dyn Language>> = OnceLock::new();

/// Register a language in the global registry.
/// Called internally by language modules.
pub fn register(lang: &'static dyn Language) {
    LANGUAGES
        .write()
        .unwrap_or_else(|e| e.into_inner())
        .push(lang);
}

/// Initialize built-in languages (called once).
fn init_builtin() {
    INITIALIZED.get_or_init(|| {
        #[cfg(feature = "lang-python")]
        register(&crate::python::Python);
        #[cfg(feature = "lang-rust")]
        register(&crate::rust::Rust);
        #[cfg(feature = "lang-javascript")]
        register(&crate::javascript::JavaScript);
        #[cfg(feature = "lang-typescript")]
        {
            register(&crate::typescript::TypeScript);
            register(&crate::typescript::Tsx);
        }
        #[cfg(feature = "lang-go")]
        register(&crate::go::Go);
        #[cfg(feature = "lang-java")]
        register(&crate::java::Java);
        #[cfg(feature = "lang-kotlin")]
        register(&crate::kotlin::Kotlin);
        #[cfg(feature = "lang-csharp")]
        register(&crate::csharp::CSharp);
        #[cfg(feature = "lang-swift")]
        register(&crate::swift::Swift);
        #[cfg(feature = "lang-php")]
        register(&crate::php::Php);
        #[cfg(feature = "lang-dockerfile")]
        register(&crate::dockerfile::Dockerfile);
        #[cfg(feature = "lang-c")]
        register(&crate::c::C);
        #[cfg(feature = "lang-cpp")]
        register(&crate::cpp::Cpp);
        #[cfg(feature = "lang-ruby")]
        register(&crate::ruby::Ruby);
        #[cfg(feature = "lang-scala")]
        register(&crate::scala::Scala);
        #[cfg(feature = "lang-vue")]
        register(&crate::vue::Vue);
        #[cfg(feature = "lang-markdown")]
        register(&crate::markdown::Markdown);
        #[cfg(feature = "lang-json")]
        register(&crate::json::Json);
        #[cfg(feature = "lang-yaml")]
        register(&crate::yaml::Yaml);
        #[cfg(feature = "lang-toml")]
        register(&crate::toml::Toml);
        #[cfg(feature = "lang-html")]
        register(&crate::html::Html);
        #[cfg(feature = "lang-css")]
        register(&crate::css::Css);
        #[cfg(feature = "lang-bash")]
        register(&crate::bash::Bash);
        #[cfg(feature = "lang-lua")]
        register(&crate::lua::Lua);
        #[cfg(feature = "lang-zig")]
        register(&crate::zig::Zig);
        #[cfg(feature = "lang-elixir")]
        register(&crate::elixir::Elixir);
        #[cfg(feature = "lang-erlang")]
        register(&crate::erlang::Erlang);
        #[cfg(feature = "lang-dart")]
        register(&crate::dart::Dart);
        #[cfg(feature = "lang-fsharp")]
        register(&crate::fsharp::FSharp);
        #[cfg(feature = "lang-sql")]
        register(&crate::sql::Sql);
        #[cfg(feature = "lang-graphql")]
        register(&crate::graphql::GraphQL);
        #[cfg(feature = "lang-hcl")]
        register(&crate::hcl::Hcl);
        #[cfg(feature = "lang-scss")]
        register(&crate::scss::Scss);
        #[cfg(feature = "lang-svelte")]
        register(&crate::svelte::Svelte);
        #[cfg(feature = "lang-xml")]
        register(&crate::xml::Xml);
        #[cfg(feature = "lang-clojure")]
        register(&crate::clojure::Clojure);
        #[cfg(feature = "lang-haskell")]
        register(&crate::haskell::Haskell);
        #[cfg(feature = "lang-ocaml")]
        register(&crate::ocaml::OCaml);
        #[cfg(feature = "lang-nix")]
        register(&crate::nix::Nix);
        #[cfg(feature = "lang-perl")]
        register(&crate::perl::Perl);
        #[cfg(feature = "lang-r")]
        register(&crate::r::R);
        #[cfg(feature = "lang-julia")]
        register(&crate::julia::Julia);
        #[cfg(feature = "lang-elm")]
        register(&crate::elm::Elm);
        #[cfg(feature = "lang-cmake")]
        register(&crate::cmake::CMake);
        #[cfg(feature = "lang-vim")]
        register(&crate::vim::Vim);
        #[cfg(feature = "lang-awk")]
        register(&crate::awk::Awk);
        #[cfg(feature = "lang-fish")]
        register(&crate::fish::Fish);
        #[cfg(feature = "lang-jq")]
        register(&crate::jq::Jq);
        #[cfg(feature = "lang-powershell")]
        register(&crate::powershell::PowerShell);
        #[cfg(feature = "lang-zsh")]
        register(&crate::zsh::Zsh);
        #[cfg(feature = "lang-groovy")]
        register(&crate::groovy::Groovy);
        #[cfg(feature = "lang-glsl")]
        register(&crate::glsl::Glsl);
        #[cfg(feature = "lang-hlsl")]
        register(&crate::hlsl::Hlsl);
        #[cfg(feature = "lang-commonlisp")]
        register(&crate::commonlisp::CommonLisp);
        #[cfg(feature = "lang-elisp")]
        register(&crate::elisp::Elisp);
        #[cfg(feature = "lang-gleam")]
        register(&crate::gleam::Gleam);
        #[cfg(feature = "lang-ini")]
        register(&crate::ini::Ini);
        #[cfg(feature = "lang-diff")]
        register(&crate::diff::Diff);
        #[cfg(feature = "lang-dot")]
        register(&crate::dot::Dot);
        #[cfg(feature = "lang-kdl")]
        register(&crate::kdl::Kdl);
        #[cfg(feature = "lang-ada")]
        register(&crate::ada::Ada);
        #[cfg(feature = "lang-agda")]
        register(&crate::agda::Agda);
        #[cfg(feature = "lang-d")]
        register(&crate::d::D);
        #[cfg(feature = "lang-matlab")]
        register(&crate::matlab::Matlab);
        #[cfg(feature = "lang-meson")]
        register(&crate::meson::Meson);
        #[cfg(feature = "lang-nginx")]
        register(&crate::nginx::Nginx);
        #[cfg(feature = "lang-prolog")]
        register(&crate::prolog::Prolog);
        #[cfg(feature = "lang-batch")]
        register(&crate::batch::Batch);
        #[cfg(feature = "lang-asm")]
        register(&crate::asm::Asm);
        #[cfg(feature = "lang-objc")]
        register(&crate::objc::ObjC);
        #[cfg(feature = "lang-typst")]
        register(&crate::typst::Typst);
        #[cfg(feature = "lang-asciidoc")]
        register(&crate::asciidoc::AsciiDoc);
        #[cfg(feature = "lang-vb")]
        register(&crate::vb::VB);
        #[cfg(feature = "lang-idris")]
        register(&crate::idris::Idris);
        #[cfg(feature = "lang-rescript")]
        register(&crate::rescript::ReScript);
        #[cfg(feature = "lang-lean")]
        register(&crate::lean::Lean);
        #[cfg(feature = "lang-caddy")]
        register(&crate::caddy::Caddy);
        #[cfg(feature = "lang-capnp")]
        register(&crate::capnp::Capnp);
        #[cfg(feature = "lang-devicetree")]
        register(&crate::devicetree::DeviceTree);
        #[cfg(feature = "lang-jinja2")]
        register(&crate::jinja2::Jinja2);
        #[cfg(feature = "lang-ninja")]
        register(&crate::ninja::Ninja);
        #[cfg(feature = "lang-postscript")]
        register(&crate::postscript::PostScript);
        #[cfg(feature = "lang-query")]
        register(&crate::query::Query);
        // Scheme registered after Query so .scm → Scheme (not Query) in extension_map
        #[cfg(feature = "lang-scheme")]
        register(&crate::scheme::Scheme);
        #[cfg(feature = "lang-ron")]
        register(&crate::ron::Ron);
        #[cfg(feature = "lang-sparql")]
        register(&crate::sparql::Sparql);
        #[cfg(feature = "lang-sshconfig")]
        register(&crate::sshconfig::SshConfig);
        #[cfg(feature = "lang-starlark")]
        register(&crate::starlark::Starlark);
        #[cfg(feature = "lang-textproto")]
        register(&crate::textproto::TextProto);
        #[cfg(feature = "lang-thrift")]
        register(&crate::thrift::Thrift);
        #[cfg(feature = "lang-tlaplus")]
        register(&crate::tlaplus::TlaPlus);
        #[cfg(feature = "lang-uiua")]
        register(&crate::uiua::Uiua);
        #[cfg(feature = "lang-verilog")]
        register(&crate::verilog::Verilog);
        #[cfg(feature = "lang-vhdl")]
        register(&crate::vhdl::Vhdl);
        #[cfg(feature = "lang-wit")]
        register(&crate::wit::Wit);
        #[cfg(feature = "lang-x86asm")]
        register(&crate::x86asm::X86Asm);
        #[cfg(feature = "lang-yuri")]
        register(&crate::yuri::Yuri);
    });
}

fn extension_map() -> &'static HashMap<&'static str, &'static dyn Language> {
    init_builtin();
    EXTENSION_MAP.get_or_init(|| {
        let mut map = HashMap::new();
        let langs = LANGUAGES.read().unwrap_or_else(|e| e.into_inner());
        for lang in langs.iter() {
            for ext in lang.extensions() {
                map.insert(*ext, *lang);
            }
        }
        map
    })
}

/// Cached extension → *all* registered languages for that extension, in
/// registration order. Unlike [`extension_map`] (which keeps only the
/// last-registered "winner"), this preserves every candidate so ambiguous
/// extensions (`.m`, `.pl`, `.s`/`.S`/`.asm`, `.conf`, `.scm`, ...) can be
/// disambiguated by [`resolve_language`] instead of silently picking one.
static EXTENSION_CANDIDATES: OnceLock<HashMap<&'static str, Vec<&'static dyn Language>>> =
    OnceLock::new();

fn extension_candidates_map() -> &'static HashMap<&'static str, Vec<&'static dyn Language>> {
    init_builtin();
    EXTENSION_CANDIDATES.get_or_init(|| {
        let mut map: HashMap<&'static str, Vec<&'static dyn Language>> = HashMap::new();
        let langs = LANGUAGES.read().unwrap_or_else(|e| e.into_inner());
        for lang in langs.iter() {
            for ext in lang.extensions() {
                map.entry(*ext).or_default().push(*lang);
            }
        }
        map
    })
}

/// All languages registered for a given extension, in registration order.
///
/// Empty if the extension isn't recognized. Length 1 for the overwhelming
/// majority of extensions; length > 1 marks a genuine collision (two
/// languages both claim the extension) that [`resolve_language`] must
/// disambiguate rather than silently picking the last-registered one.
pub fn candidates_for_extension(ext: &str) -> Vec<&'static dyn Language> {
    extension_candidates_map()
        .get(ext)
        .or_else(|| extension_candidates_map().get(ext.to_lowercase().as_str()))
        .cloned()
        .unwrap_or_default()
}

/// All extensions that currently resolve to more than one registered
/// language — i.e. the set of collisions `support_for_path` silently
/// resolves via last-registration-wins. Derived from live registration data
/// so a newly added language that collides with an existing extension shows
/// up here automatically instead of requiring a hardcoded list to be kept in
/// sync by hand.
pub fn ambiguous_extensions() -> Vec<&'static str> {
    let mut exts: Vec<&'static str> = extension_candidates_map()
        .iter()
        .filter(|(_, langs)| langs.len() > 1)
        .map(|(ext, _)| *ext)
        .collect();
    exts.sort_unstable();
    exts
}

fn grammar_map() -> &'static HashMap<&'static str, &'static dyn Language> {
    init_builtin();
    GRAMMAR_MAP.get_or_init(|| {
        let mut map = HashMap::new();
        let langs = LANGUAGES.read().unwrap_or_else(|e| e.into_inner());
        for lang in langs.iter() {
            map.insert(lang.grammar_name(), *lang);
        }
        map
    })
}

/// Get language support for a file extension.
///
/// Returns `None` if the extension is not recognized or the feature is not enabled.
pub fn support_for_extension(ext: &str) -> Option<&'static dyn Language> {
    extension_map()
        .get(ext)
        .or_else(|| extension_map().get(ext.to_lowercase().as_str()))
        .copied()
}

/// Get language support by grammar name.
///
/// Returns `None` if the grammar is not recognized or the feature is not enabled.
pub fn support_for_grammar(grammar: &str) -> Option<&'static dyn Language> {
    grammar_map().get(grammar).copied()
}

/// Get language support from a file path.
///
/// Returns `None` if the file has no extension, the extension is not recognized,
/// or the feature is not enabled.
pub fn support_for_path(path: &Path) -> Option<&'static dyn Language> {
    path.extension()
        .and_then(|e| e.to_str())
        .and_then(support_for_extension)
}

/// Project-level extension-to-language overrides (`.normalize/config.toml`
/// `[languages]` table), consulted by [`resolve_language`] before content
/// sniffing. Patterns are gitignore/glob-style (`*.m`, `src/**/*.pl`,
/// matched against the path as given to [`resolve_language`]) and are tried
/// in the order supplied; the first match wins.
pub struct LanguageOverrides {
    patterns: Vec<(globset::GlobMatcher, String)>,
}

impl LanguageOverrides {
    /// Build from an ordered list of `(glob pattern, language name or
    /// grammar name)` pairs, e.g. as loaded from `.normalize/config.toml`'s
    /// `[languages]` table. Invalid glob patterns are skipped rather than
    /// erroring the whole set, since one bad entry in a project config
    /// shouldn't break language detection for every other file.
    pub fn new(patterns: impl IntoIterator<Item = (String, String)>) -> Self {
        let compiled = patterns
            .into_iter()
            .filter_map(|(pattern, lang)| {
                globset::Glob::new(&pattern)
                    .ok()
                    .map(|g| (g.compile_matcher(), lang))
            })
            .collect();
        Self { patterns: compiled }
    }

    /// No overrides configured.
    pub fn empty() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    /// The configured language name for `path`, if any override glob
    /// matches (first match in configured order wins).
    pub fn resolve(&self, path: &Path) -> Option<&str> {
        self.patterns
            .iter()
            .find(|(matcher, _)| matcher.is_match(path))
            .map(|(_, lang)| lang.as_str())
    }
}

/// Why [`resolve_language`] settled on (or failed to settle on) a language.
/// Carried alongside the result so callers can log/print an explanation
/// rather than silently picking a winner for an ambiguous extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionReason {
    /// Caller passed an explicit `--lang`/`--language` name; used outright.
    ExplicitFlag,
    /// `--lang` named a language `resolve_language` doesn't recognize.
    ExplicitFlagUnknown,
    /// `.normalize/config.toml` `[languages]` glob matched.
    ConfigOverride,
    /// `.normalize/config.toml` `[languages]` glob matched but named an
    /// unrecognized language.
    ConfigOverrideUnknown,
    /// Only one language is registered for this extension — no ambiguity.
    Unambiguous,
    /// A `#!` shebang line unambiguously named one candidate.
    ShebangSniff,
    /// No shebang; keyword/syntax heuristics unambiguously favored one
    /// candidate.
    HeuristicSniff,
    /// Multiple candidates registered for this extension and neither a
    /// shebang nor heuristics could decide; fell back to the deterministic
    /// (last-registered-wins) default.
    AmbiguousDefault,
    /// Extension not recognized by any registered language.
    NotFound,
}

/// Result of [`resolve_language`]: the chosen language (if any), why it was
/// chosen, and — for the ambiguous case — the full set of candidates that
/// were in contention, so a caller can surface them (e.g. "could be MATLAB
/// or Objective-C; pass --lang or set .normalize/config.toml").
#[derive(Clone)]
pub struct LanguageResolutionResult {
    pub language: Option<&'static dyn Language>,
    pub reason: ResolutionReason,
    /// Non-empty only when `reason == AmbiguousDefault`: every candidate
    /// that was in contention for this extension.
    pub ambiguous_candidates: Vec<&'static dyn Language>,
}

impl LanguageResolutionResult {
    /// Human-readable explanation suitable for a log line or stderr note,
    /// e.g. `"resolved .m to matlab via --lang"` or `"resolved .conf to
    /// nginx: ambiguous with ini, no override configured, defaulting per
    /// registration order — set --lang or .normalize/config.toml to
    /// override"`.
    pub fn describe(&self, ext: &str) -> String {
        let lang_name = self.language.map(|l| l.grammar_name());
        match self.reason {
            ResolutionReason::ExplicitFlag => {
                format!("resolved .{ext} to {} via --lang", lang_name.unwrap_or("?"))
            }
            ResolutionReason::ExplicitFlagUnknown => {
                "--lang named a language that isn't registered".to_string()
            }
            ResolutionReason::ConfigOverride => format!(
                "resolved .{ext} to {} via .normalize/config.toml [languages] override",
                lang_name.unwrap_or("?")
            ),
            ResolutionReason::ConfigOverrideUnknown => {
                ".normalize/config.toml [languages] override named a language that isn't registered"
                    .to_string()
            }
            ResolutionReason::Unambiguous => {
                format!("resolved .{ext} to {}", lang_name.unwrap_or("?"))
            }
            ResolutionReason::ShebangSniff => format!(
                "resolved .{ext} to {} via shebang",
                lang_name.unwrap_or("?")
            ),
            ResolutionReason::HeuristicSniff => format!(
                "resolved .{ext} to {} via content heuristics",
                lang_name.unwrap_or("?")
            ),
            ResolutionReason::AmbiguousDefault => {
                let others: Vec<&str> = self
                    .ambiguous_candidates
                    .iter()
                    .map(|l| l.grammar_name())
                    .filter(|n| Some(*n) != lang_name)
                    .collect();
                format!(
                    "resolved .{ext} to {}: ambiguous with {}, no override configured, defaulting per registration order — set --lang or .normalize/config.toml to override",
                    lang_name.unwrap_or("?"),
                    others.join(", ")
                )
            }
            ResolutionReason::NotFound => format!("no language registered for .{ext}"),
        }
    }
}

fn lookup_by_name(name: &str) -> Option<&'static dyn Language> {
    support_for_grammar(name).or_else(|| {
        let langs = LANGUAGES.read().unwrap_or_else(|e| e.into_inner());
        langs
            .iter()
            .find(|l| l.name().eq_ignore_ascii_case(name))
            .copied()
    })
}

/// Score a single candidate's [`crate::SniffHints::content_signals`] against
/// `content`.
fn heuristic_score(lang: &'static dyn Language, content: &str) -> i32 {
    lang.sniff_hints()
        .content_signals
        .iter()
        .filter(|(needle, _)| content.contains(needle))
        .map(|(_, weight)| *weight)
        .sum()
}

/// Layered language resolver: explicit `--lang` flag, then project config
/// overrides, then (only if the extension is genuinely ambiguous) content
/// sniffing via shebang and keyword heuristics, then the legacy
/// deterministic default. See module docs / CLAUDE.md for the full design.
///
/// `content` is optional — pass `None` when only a path is available (e.g.
/// batch-walking a tree without reading every file); sniffing is skipped and
/// an ambiguous extension falls straight through to `AmbiguousDefault`.
pub fn resolve_language(
    path: &Path,
    content: Option<&str>,
    explicit_lang: Option<&str>,
    overrides: &LanguageOverrides,
) -> LanguageResolutionResult {
    // 1. Explicit --lang flag wins outright.
    if let Some(name) = explicit_lang {
        return match lookup_by_name(name) {
            Some(lang) => LanguageResolutionResult {
                language: Some(lang),
                reason: ResolutionReason::ExplicitFlag,
                ambiguous_candidates: Vec::new(),
            },
            None => LanguageResolutionResult {
                language: None,
                reason: ResolutionReason::ExplicitFlagUnknown,
                ambiguous_candidates: Vec::new(),
            },
        };
    }

    // 2. Project-level glob config override.
    if let Some(name) = overrides.resolve(path) {
        return match lookup_by_name(name) {
            Some(lang) => LanguageResolutionResult {
                language: Some(lang),
                reason: ResolutionReason::ConfigOverride,
                ambiguous_candidates: Vec::new(),
            },
            None => LanguageResolutionResult {
                language: None,
                reason: ResolutionReason::ConfigOverrideUnknown,
                ambiguous_candidates: Vec::new(),
            },
        };
    }

    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return LanguageResolutionResult {
            language: None,
            reason: ResolutionReason::NotFound,
            ambiguous_candidates: Vec::new(),
        };
    };
    let candidates = candidates_for_extension(ext);

    if candidates.is_empty() {
        return LanguageResolutionResult {
            language: None,
            reason: ResolutionReason::NotFound,
            ambiguous_candidates: Vec::new(),
        };
    }
    if candidates.len() == 1 {
        return LanguageResolutionResult {
            language: Some(candidates[0]),
            reason: ResolutionReason::Unambiguous,
            ambiguous_candidates: Vec::new(),
        };
    }

    // 3. Content sniffing — only reached for a genuinely ambiguous extension.
    if let Some(content) = content {
        // 3a. Shebang line: high confidence, decisive if it names exactly
        // one candidate.
        if let Some(first_line) = content.lines().next()
            && let Some(shebang) = first_line.strip_prefix("#!")
        {
            let shebang_lower = shebang.to_lowercase();
            let matches: Vec<&'static dyn Language> = candidates
                .iter()
                .filter(|l| {
                    l.sniff_hints()
                        .shebang_patterns
                        .iter()
                        .any(|p| shebang_lower.contains(&p.to_lowercase()))
                })
                .copied()
                .collect();
            if matches.len() == 1 {
                return LanguageResolutionResult {
                    language: Some(matches[0]),
                    reason: ResolutionReason::ShebangSniff,
                    ambiguous_candidates: Vec::new(),
                };
            }
        }

        // 3b. Low-confidence keyword/syntax heuristics. Only decisive when
        // exactly one candidate has a uniquely-highest, strictly-positive
        // score — a tie or all-zero scores means the heuristics genuinely
        // couldn't decide, so we do NOT guess and fall through instead.
        let scored: Vec<(i32, &'static dyn Language)> = candidates
            .iter()
            .copied()
            .map(|l| (heuristic_score(l, content), l))
            .collect();
        let max_score = scored.iter().map(|(s, _)| *s).max().unwrap_or(0);
        if max_score > 0 {
            let winners: Vec<&'static dyn Language> = scored
                .iter()
                .filter(|(s, _)| *s == max_score)
                .map(|(_, l)| *l)
                .collect();
            if winners.len() == 1 {
                return LanguageResolutionResult {
                    language: Some(winners[0]),
                    reason: ResolutionReason::HeuristicSniff,
                    ambiguous_candidates: Vec::new(),
                };
            }
        }
    }

    // 4. Still ambiguous — documented deterministic fallback (matches
    // legacy `support_for_path` behavior: last-registered candidate wins).
    let default = candidates[candidates.len() - 1];
    LanguageResolutionResult {
        language: Some(default),
        reason: ResolutionReason::AmbiguousDefault,
        ambiguous_candidates: candidates,
    }
}

/// Check if a file path is a dedicated test file for its language.
///
/// Returns false for unknown file types or languages that use inline tests.
/// Matches against the language's `test_file_globs()` patterns.
pub fn is_test_path(path: &Path) -> bool {
    let lang = match support_for_path(path) {
        Some(l) => l,
        None => return false,
    };
    let globs = lang.test_file_globs();
    if globs.is_empty() {
        return false;
    }
    let mut builder = globset::GlobSetBuilder::new();
    for g in globs {
        if let Ok(glob) = globset::Glob::new(g) {
            builder.add(glob);
        }
    }
    let Ok(set) = builder.build() else {
        return false;
    };
    set.is_match(path)
}

/// Get all glob patterns that identify test files for a given language extension.
pub fn test_file_globs_for_path(path: &Path) -> &'static [&'static str] {
    support_for_path(path)
        .map(|lang| lang.test_file_globs())
        .unwrap_or(&[])
}

/// Get all supported languages.
pub fn supported_languages() -> Vec<&'static dyn Language> {
    init_builtin();
    LANGUAGES.read().unwrap_or_else(|e| e.into_inner()).clone()
}

/// Check if a path is a programming language (not a data/config format).
///
/// Returns false for data formats like JSON, YAML, TOML, Markdown, etc.
/// even though normalize-languages can parse them for syntax highlighting.
///
/// Useful for architecture analysis where only "code" files are relevant.
/// Uses `normalize_language_meta::capabilities_for()` to determine if a
/// language is executable code.
pub fn is_programming_language(path: &Path) -> bool {
    let lang = match support_for_path(path) {
        Some(l) => l,
        None => return false,
    };

    let caps = normalize_language_meta::capabilities_for(lang.name());
    caps.executable
}

/// Validate that a language's unused node kinds audit is complete and accurate.
///
/// This function checks:
/// 1. All kinds in `documented_unused` actually exist in the grammar
/// 2. All potentially useful kinds from the grammar are either used or documented
///
/// Call this from each language's `unused_node_kinds_audit` test.
pub fn validate_unused_kinds_audit(
    lang: &dyn Language,
    documented_unused: &[&str],
) -> Result<(), String> {
    use crate::GrammarLoader;
    use crate::grammar_loader::GrammarLoadError;
    use std::collections::HashSet;

    let loader = GrammarLoader::new();
    let ts_lang = match loader.get(lang.grammar_name()) {
        Ok(l) => l,
        // Grammar `.so` not present — typical in `cargo test` without
        // `cargo xtask build-grammars`. Skip the audit instead of panicking.
        // Real audit failures (loaded but mismatched) still surface.
        Err(GrammarLoadError::NotFound(_)) => return Ok(()),
        Err(e) => return Err(format!("Grammar '{}' not found: {e}", lang.grammar_name())),
    };

    // Keywords that suggest a node kind might be useful (same as cross_check_node_kinds)
    let interesting_patterns = [
        "statement",
        "expression",
        "definition",
        "declaration",
        "clause",
        "block",
        "body",
        "import",
        "export",
        "function",
        "method",
        "class",
        "struct",
        "enum",
        "interface",
        "trait",
        "module",
        "type",
        "return",
        "if",
        "else",
        "for",
        "while",
        "loop",
        "match",
        "case",
        "try",
        "catch",
        "except",
        "throw",
        "raise",
        "with",
        "async",
        "await",
        "yield",
        "lambda",
        "comprehension",
        "generator",
        "operator",
    ];

    // Collect kinds referenced in tags.scm
    let tags_kinds: HashSet<String> = {
        let mut kinds = HashSet::new();
        if let Some(tags_content) = loader.get_tags(lang.grammar_name()) {
            // Extract top-level node kind names: lines starting with "(<identifier>"
            // These are the patterns like "(function_definition ..." in the query
            for line in tags_content.lines() {
                let trimmed = line.trim_start();
                if trimmed.starts_with('(')
                    && !trimmed.starts_with(";;")
                    && !trimmed.starts_with(";")
                {
                    // Extract the first word after the opening paren
                    let inner = &trimmed[1..];
                    let kind_name: String = inner
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                        .collect();
                    if !kind_name.is_empty() && !kind_name.starts_with('@') {
                        kinds.insert(kind_name);
                    }
                }
            }
        }
        kinds
    };

    let documented_set: HashSet<&str> = documented_unused.iter().copied().collect();

    // Get all valid named node kinds from grammar
    let mut grammar_kinds: HashSet<&str> = HashSet::new();
    let count = ts_lang.node_kind_count();
    for id in 0..count as u16 {
        if let Some(kind) = ts_lang.node_kind_for_id(id) {
            let named = ts_lang.node_kind_is_named(id);
            if named && !kind.starts_with('_') {
                grammar_kinds.insert(kind);
            }
        }
    }

    let mut errors: Vec<String> = Vec::new();

    // Check 1: All documented unused kinds must exist in grammar
    for kind in documented_unused {
        if !grammar_kinds.contains(*kind) {
            errors.push(format!(
                "Documented kind '{}' doesn't exist in grammar",
                kind
            ));
        }
        // Also check it's not actually being used (in tags.scm)
        if tags_kinds.contains(*kind) {
            errors.push(format!(
                "Documented kind '{}' is actually used in tags.scm",
                kind
            ));
        }
    }

    // Check 2: All potentially useful grammar kinds must be used or documented
    for kind in &grammar_kinds {
        let lower = kind.to_lowercase();
        let is_interesting = interesting_patterns.iter().any(|p| lower.contains(p));

        if is_interesting && !tags_kinds.contains(*kind) && !documented_set.contains(*kind) {
            errors.push(format!(
                "Potentially useful kind '{}' is neither used nor documented",
                kind
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{} validation errors:\n  - {}",
            errors.len(),
            errors.join("\n  - ")
        ))
    }
}

#[cfg(test)]
mod resolver_tests {
    use super::*;
    use std::path::Path;

    /// The 5 documented extension collisions must still exist (i.e. this
    /// test guards against someone "fixing" a collision by deleting an
    /// extension, which would silently change behavior without going
    /// through the resolver).
    #[test]
    fn known_collisions_are_still_ambiguous() {
        let ambiguous = ambiguous_extensions();
        for ext in ["m", "s", "S", "asm", "pl", "conf", "scm"] {
            assert!(
                ambiguous.contains(&ext),
                "expected .{ext} to be an ambiguous extension, got {ambiguous:?}"
            );
        }
    }

    #[test]
    fn unambiguous_extension_resolves_directly() {
        let result = resolve_language(
            Path::new("main.rs"),
            None,
            None,
            &LanguageOverrides::empty(),
        );
        assert_eq!(result.reason, ResolutionReason::Unambiguous);
        assert_eq!(result.language.map(|l| l.grammar_name()), Some("rust"));
    }

    #[test]
    fn explicit_lang_flag_wins_over_everything() {
        let overrides = LanguageOverrides::new([("*.m".to_string(), "objc".to_string())]);
        let result = resolve_language(
            Path::new("script.m"),
            Some("function y = f(x)\nend\n"),
            Some("matlab"),
            &overrides,
        );
        assert_eq!(result.reason, ResolutionReason::ExplicitFlag);
        assert_eq!(result.language.map(|l| l.grammar_name()), Some("matlab"));
    }

    #[test]
    fn config_override_wins_over_sniffing() {
        // Content strongly suggests Objective-C, but the config override
        // for this glob names MATLAB — override must win.
        let overrides = LanguageOverrides::new([("*.m".to_string(), "matlab".to_string())]);
        let result = resolve_language(
            Path::new("widget.m"),
            Some("#import <Foundation/Foundation.h>\n@interface Foo\n@end\n"),
            None,
            &overrides,
        );
        assert_eq!(result.reason, ResolutionReason::ConfigOverride);
        assert_eq!(result.language.map(|l| l.grammar_name()), Some("matlab"));
    }

    #[test]
    fn config_override_glob_matches_by_path_pattern() {
        let overrides =
            LanguageOverrides::new([("src/**/*.pl".to_string(), "prolog".to_string())]);
        let result = resolve_language(
            Path::new("src/rules/facts.pl"),
            Some("use strict;\nmy $x = 1;\n"),
            None,
            &overrides,
        );
        assert_eq!(result.reason, ResolutionReason::ConfigOverride);
        assert_eq!(result.language.map(|l| l.grammar_name()), Some("prolog"));
    }

    #[test]
    fn shebang_resolves_perl_over_prolog() {
        let result = resolve_language(
            Path::new("script.pl"),
            Some("#!/usr/bin/perl\nuse strict;\n"),
            None,
            &LanguageOverrides::empty(),
        );
        assert_eq!(result.reason, ResolutionReason::ShebangSniff);
        assert_eq!(result.language.map(|l| l.grammar_name()), Some("perl"));
    }

    #[test]
    fn shebang_resolves_prolog_over_perl() {
        let result = resolve_language(
            Path::new("rules.pl"),
            Some("#!/usr/bin/env swipl\n:- initialization(main).\n"),
            None,
            &LanguageOverrides::empty(),
        );
        assert_eq!(result.reason, ResolutionReason::ShebangSniff);
        assert_eq!(result.language.map(|l| l.grammar_name()), Some("prolog"));
    }

    #[test]
    fn heuristic_resolves_matlab_without_shebang() {
        let result = resolve_language(
            Path::new("solve.m"),
            Some("function y = solve(x)\n  y = x + 1;\nendfunction\n"),
            None,
            &LanguageOverrides::empty(),
        );
        assert_eq!(result.reason, ResolutionReason::HeuristicSniff);
        assert_eq!(result.language.map(|l| l.grammar_name()), Some("matlab"));
    }

    #[test]
    fn heuristic_resolves_objc_without_shebang() {
        let result = resolve_language(
            Path::new("Widget.m"),
            Some("#import <Foundation/Foundation.h>\n@interface Widget : NSObject\n@end\n"),
            None,
            &LanguageOverrides::empty(),
        );
        assert_eq!(result.reason, ResolutionReason::HeuristicSniff);
        assert_eq!(result.language.map(|l| l.grammar_name()), Some("objc"));
    }

    /// An empty/near-empty file gives no shebang and no heuristic signal —
    /// the resolver must NOT silently guess one of the two candidates. It
    /// falls back to the documented deterministic default and reports
    /// `AmbiguousDefault` with every candidate that was in contention, so a
    /// caller can surface the ambiguity instead of trusting a blind pick.
    #[test]
    fn genuinely_ambiguous_content_does_not_silently_guess() {
        let result = resolve_language(
            Path::new("mystery.m"),
            Some("\n\n"),
            None,
            &LanguageOverrides::empty(),
        );
        assert_eq!(result.reason, ResolutionReason::AmbiguousDefault);
        assert!(result.language.is_some(), "documented fallback still picks a language");
        let names: Vec<&str> = result
            .ambiguous_candidates
            .iter()
            .map(|l| l.grammar_name())
            .collect();
        assert!(names.contains(&"matlab") && names.contains(&"objc"));
        // The message must name the ambiguity, not just assert a language.
        let msg = result.describe("m");
        assert!(msg.contains("ambiguous"));
    }

    #[test]
    fn no_content_available_falls_back_to_ambiguous_default() {
        let result = resolve_language(
            Path::new("legacy.conf"),
            None,
            None,
            &LanguageOverrides::empty(),
        );
        assert_eq!(result.reason, ResolutionReason::AmbiguousDefault);
        assert_eq!(result.language.map(|l| l.grammar_name()), Some("nginx"));
    }

    #[test]
    fn ambiguous_default_matches_legacy_support_for_path_for_documented_pairs() {
        // The 5 documented collisions must keep resolving to the same
        // "winner" as the old last-registered-wins `support_for_path`, so
        // the fallback path is a behavior-preserving default, not a
        // silent regression, for callers that don't opt into the resolver.
        let cases: &[(&str, &str)] = &[
            ("foo.m", "objc"),
            ("foo.s", "x86asm"),
            ("foo.S", "x86asm"),
            ("foo.pl", "prolog"),
            ("foo.conf", "nginx"),
        ];
        for (path, expected) in cases {
            let legacy = support_for_path(Path::new(path)).map(|l| l.grammar_name());
            assert_eq!(legacy, Some(*expected), "legacy winner changed for {path}");

            let resolved = resolve_language(
                Path::new(path),
                None,
                None,
                &LanguageOverrides::empty(),
            );
            assert_eq!(
                resolved.language.map(|l| l.grammar_name()),
                Some(*expected),
                "resolver fallback diverged from legacy for {path}"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GrammarLoader;

    /// Dump all valid node kinds for a grammar (useful for fixing invalid kinds).
    /// Run with: cargo test -p rhizome-normalize-languages dump_node_kinds -- --nocapture
    #[test]
    #[ignore]
    fn dump_node_kinds() {
        let loader = GrammarLoader::new();
        // Change this to the grammar you want to inspect
        let grammar_name = std::env::var("DUMP_GRAMMAR").unwrap_or_else(|_| "python".to_string());

        let ts_lang = loader.get(&grammar_name).expect("grammar not found");

        println!("\n=== Valid node kinds for '{}' ===\n", grammar_name);
        let count = ts_lang.node_kind_count();
        for id in 0..count as u16 {
            if let Some(kind) = ts_lang.node_kind_for_id(id) {
                let named = ts_lang.node_kind_is_named(id);
                if named && !kind.starts_with('_') {
                    println!("{}", kind);
                }
            }
        }
    }

    /// Validate that all node kinds returned by Language trait methods
    /// actually exist in the tree-sitter grammar.
    ///
    /// No trait methods return node kind lists any more —
    /// export detection now uses tags.scm queries exclusively.
    /// This test is intentionally empty.
    #[test]
    fn validate_node_kinds() {
        // Nothing to validate — node kind lists were removed from the Language trait.
    }

    /// Cross-check grammar node kinds against Language implementations.
    /// Finds potentially useful kinds that exist in the grammar but aren't used.
    /// Run with: cargo test -p rhizome-normalize-languages cross_check_node_kinds -- --nocapture --ignored
    #[test]
    #[ignore]
    fn cross_check_node_kinds() {
        use std::collections::HashSet;

        let loader = GrammarLoader::new();

        // Keywords that suggest a node kind might be useful
        let interesting_patterns = [
            "statement",
            "expression",
            "definition",
            "declaration",
            "clause",
            "block",
            "body",
            "import",
            "export",
            "function",
            "method",
            "class",
            "struct",
            "enum",
            "interface",
            "trait",
            "module",
            "type",
            "return",
            "if",
            "else",
            "for",
            "while",
            "loop",
            "match",
            "case",
            "try",
            "catch",
            "except",
            "throw",
            "raise",
            "with",
            "async",
            "await",
            "yield",
            "lambda",
            "comprehension",
            "generator",
            "operator",
        ];

        for lang in supported_languages() {
            let grammar_name = lang.grammar_name();
            let ts_lang = match loader.get(grammar_name).ok() {
                Some(l) => l,
                None => continue,
            };

            // Collect all kinds currently used by the language
            // public_symbol_kinds() removed — export detection uses tags.scm exclusively.
            let used_kinds: HashSet<&str> = HashSet::new();

            // Get all valid named node kinds from grammar
            let mut all_kinds: Vec<&str> = Vec::new();
            let count = ts_lang.node_kind_count();
            for id in 0..count as u16 {
                if let Some(kind) = ts_lang.node_kind_for_id(id) {
                    let named = ts_lang.node_kind_is_named(id);
                    if named && !kind.starts_with('_') {
                        all_kinds.push(kind);
                    }
                }
            }

            // Find unused but potentially interesting kinds
            let mut unused_interesting: Vec<&str> = all_kinds
                .into_iter()
                .filter(|kind| !used_kinds.contains(*kind))
                .filter(|kind| {
                    let lower = kind.to_lowercase();
                    interesting_patterns.iter().any(|p| lower.contains(p))
                })
                .collect();

            unused_interesting.sort();

            if !unused_interesting.is_empty() {
                println!(
                    "\n=== {} ({}) - {} potentially useful unused kinds ===",
                    lang.name(),
                    grammar_name,
                    unused_interesting.len()
                );
                for kind in &unused_interesting {
                    println!("  {}", kind);
                }
            }
        }
    }
}
