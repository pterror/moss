//! Language support registry with extension-based lookup.

use crate::LanguageSupport;
use std::path::Path;

/// Get language support for a file extension.
///
/// Returns `None` if the extension is not recognized or the feature is not enabled.
pub fn support_for_extension(ext: &str) -> Option<&'static dyn LanguageSupport> {
    match ext.to_lowercase().as_str() {
        #[cfg(feature = "lang-python")]
        "py" | "pyi" | "pyw" => Some(&crate::python::Python),

        #[cfg(feature = "lang-rust")]
        "rs" => Some(&crate::rust::Rust),

        #[cfg(feature = "lang-javascript")]
        "js" | "mjs" | "cjs" | "jsx" => Some(&crate::javascript::JavaScript),

        #[cfg(feature = "lang-typescript")]
        "ts" | "mts" | "cts" => Some(&crate::typescript::TypeScript),

        #[cfg(feature = "lang-typescript")]
        "tsx" => Some(&crate::typescript::Tsx),

        #[cfg(feature = "lang-go")]
        "go" => Some(&crate::go::Go),

        #[cfg(feature = "lang-java")]
        "java" => Some(&crate::java::Java),

        #[cfg(feature = "lang-c")]
        "c" | "h" => Some(&crate::c::C),

        #[cfg(feature = "lang-cpp")]
        "cpp" | "cc" | "cxx" | "hpp" | "hh" | "hxx" => Some(&crate::cpp::Cpp),

        #[cfg(feature = "lang-ruby")]
        "rb" => Some(&crate::ruby::Ruby),

        #[cfg(feature = "lang-scala")]
        "scala" | "sc" => Some(&crate::scala::Scala),

        #[cfg(feature = "lang-vue")]
        "vue" => Some(&crate::vue::Vue),

        #[cfg(feature = "lang-markdown")]
        "md" | "markdown" => Some(&crate::markdown::Markdown),

        #[cfg(feature = "lang-json")]
        "json" | "jsonc" => Some(&crate::json::Json),

        #[cfg(feature = "lang-yaml")]
        "yaml" | "yml" => Some(&crate::yaml::Yaml),

        #[cfg(feature = "lang-toml")]
        "toml" => Some(&crate::toml::Toml),

        #[cfg(feature = "lang-html")]
        "html" | "htm" => Some(&crate::html::Html),

        #[cfg(feature = "lang-css")]
        "css" | "scss" => Some(&crate::css::Css),

        #[cfg(feature = "lang-bash")]
        "sh" | "bash" | "zsh" => Some(&crate::bash::Bash),

        _ => None,
    }
}

/// Get language support from a file path.
///
/// Returns `None` if the file has no extension, the extension is not recognized,
/// or the feature is not enabled.
pub fn support_for_path(path: &Path) -> Option<&'static dyn LanguageSupport> {
    path.extension()
        .and_then(|e| e.to_str())
        .and_then(support_for_extension)
}

/// Get all supported languages (based on enabled features).
pub fn supported_languages() -> Vec<&'static dyn LanguageSupport> {
    let mut langs: Vec<&'static dyn LanguageSupport> = Vec::new();

    #[cfg(feature = "lang-python")]
    langs.push(&crate::python::Python);

    #[cfg(feature = "lang-rust")]
    langs.push(&crate::rust::Rust);

    #[cfg(feature = "lang-javascript")]
    langs.push(&crate::javascript::JavaScript);

    #[cfg(feature = "lang-typescript")]
    {
        langs.push(&crate::typescript::TypeScript);
        langs.push(&crate::typescript::Tsx);
    }

    #[cfg(feature = "lang-go")]
    langs.push(&crate::go::Go);

    #[cfg(feature = "lang-java")]
    langs.push(&crate::java::Java);

    #[cfg(feature = "lang-c")]
    langs.push(&crate::c::C);

    #[cfg(feature = "lang-cpp")]
    langs.push(&crate::cpp::Cpp);

    #[cfg(feature = "lang-ruby")]
    langs.push(&crate::ruby::Ruby);

    #[cfg(feature = "lang-scala")]
    langs.push(&crate::scala::Scala);

    #[cfg(feature = "lang-vue")]
    langs.push(&crate::vue::Vue);

    #[cfg(feature = "lang-markdown")]
    langs.push(&crate::markdown::Markdown);

    #[cfg(feature = "lang-json")]
    langs.push(&crate::json::Json);

    #[cfg(feature = "lang-yaml")]
    langs.push(&crate::yaml::Yaml);

    #[cfg(feature = "lang-toml")]
    langs.push(&crate::toml::Toml);

    #[cfg(feature = "lang-html")]
    langs.push(&crate::html::Html);

    #[cfg(feature = "lang-css")]
    langs.push(&crate::css::Css);

    #[cfg(feature = "lang-bash")]
    langs.push(&crate::bash::Bash);

    langs
}
