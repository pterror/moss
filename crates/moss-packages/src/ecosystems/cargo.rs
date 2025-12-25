//! Cargo (Rust) ecosystem.

use crate::{Ecosystem, Feature, LockfileManager, PackageError, PackageInfo};
use std::process::Command;

pub struct Cargo;

impl Ecosystem for Cargo {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn manifest_files(&self) -> &'static [&'static str] {
        &["Cargo.toml"]
    }

    fn lockfiles(&self) -> &'static [LockfileManager] {
        &[LockfileManager {
            filename: "Cargo.lock",
            manager: "cargo",
        }]
    }

    fn tools(&self) -> &'static [&'static str] {
        &["cargo"]
    }

    fn fetch_info(&self, package: &str, tool: &str) -> Result<PackageInfo, PackageError> {
        match tool {
            "cargo" => fetch_cargo_info(package),
            _ => Err(PackageError::ToolFailed(format!("unknown tool: {}", tool))),
        }
    }
}

fn fetch_cargo_info(package: &str) -> Result<PackageInfo, PackageError> {
    let output = Command::new("cargo")
        .args(["info", package])
        .output()
        .map_err(|e| PackageError::ToolFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("could not find") {
            return Err(PackageError::NotFound(package.to_string()));
        }
        return Err(PackageError::ToolFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_cargo_info(&stdout, package)
}

fn parse_cargo_info(output: &str, package: &str) -> Result<PackageInfo, PackageError> {
    let mut info = PackageInfo {
        name: package.to_string(),
        version: String::new(),
        description: None,
        license: None,
        homepage: None,
        repository: None,
        features: Vec::new(),
        dependencies: Vec::new(),
    };

    let mut in_features = false;
    let mut lines = output.lines().peekable();

    // First line: package name and tags (skip)
    if let Some(first) = lines.next() {
        // "serde #serde #serialization #no_std"
        // Name is already set from the query
        let _ = first;
    }

    // Second line: description
    if let Some(desc) = lines.next() {
        if !desc.is_empty() && !desc.starts_with("version:") {
            info.description = Some(desc.to_string());
        }
    }

    for line in lines {
        if line.starts_with("version:") {
            info.version = line.trim_start_matches("version:").trim().to_string();
        } else if line.starts_with("license:") {
            info.license = Some(line.trim_start_matches("license:").trim().to_string());
        } else if line.starts_with("homepage:") {
            info.homepage = Some(line.trim_start_matches("homepage:").trim().to_string());
        } else if line.starts_with("repository:") {
            info.repository = Some(line.trim_start_matches("repository:").trim().to_string());
        } else if line.starts_with("features:") {
            in_features = true;
        } else if line.starts_with("note:") {
            in_features = false;
        } else if in_features && line.starts_with(' ') {
            // Parse feature line: " +default      = [std]" or "  std          = [serde_core/std]"
            if let Some(feature) = parse_feature_line(line) {
                info.features.push(feature);
            }
        }
    }

    if info.version.is_empty() {
        return Err(PackageError::ParseError(
            "could not find version in output".to_string(),
        ));
    }

    Ok(info)
}

fn parse_feature_line(line: &str) -> Option<Feature> {
    // " +default      = [std]"
    // "  derive       = [serde_derive]"
    let trimmed = line.trim();

    // Remove leading + if present (indicates default feature)
    let trimmed = trimmed.strip_prefix('+').unwrap_or(trimmed);

    // Split on '='
    let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
    if parts.len() != 2 {
        return None;
    }

    let name = parts[0].trim().to_string();
    let deps_str = parts[1].trim();

    // Parse [dep1, dep2] or [dep:name]
    let deps = if deps_str.starts_with('[') && deps_str.ends_with(']') {
        let inner = &deps_str[1..deps_str.len() - 1];
        inner
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        Vec::new()
    };

    Some(Feature {
        name,
        description: None,
        dependencies: deps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cargo_info() {
        let output = r#"serde #serde #serialization #no_std
A generic serialization/deserialization framework
version: 1.0.228
license: MIT OR Apache-2.0
rust-version: 1.56
documentation: https://docs.rs/serde
homepage: https://serde.rs
repository: https://github.com/serde-rs/serde
crates.io: https://crates.io/crates/serde/1.0.228
features:
 +default      = [std]
  std          = [serde_core/std]
  alloc        = [serde_core/alloc]
  derive       = [serde_derive]
note: to see how you depend on serde, run `cargo tree --invert --package serde@1.0.228`"#;

        let info = parse_cargo_info(output, "serde").unwrap();
        assert_eq!(info.name, "serde");
        assert_eq!(info.version, "1.0.228");
        assert_eq!(
            info.description,
            Some("A generic serialization/deserialization framework".to_string())
        );
        assert_eq!(info.license, Some("MIT OR Apache-2.0".to_string()));
        assert_eq!(info.homepage, Some("https://serde.rs".to_string()));
        assert_eq!(
            info.repository,
            Some("https://github.com/serde-rs/serde".to_string())
        );
        assert_eq!(info.features.len(), 4);
        assert_eq!(info.features[0].name, "default");
        assert_eq!(info.features[0].dependencies, vec!["std"]);
    }

    #[test]
    fn test_parse_feature_line() {
        let f = parse_feature_line(" +default      = [std]").unwrap();
        assert_eq!(f.name, "default");
        assert_eq!(f.dependencies, vec!["std"]);

        let f = parse_feature_line("  derive       = [serde_derive]").unwrap();
        assert_eq!(f.name, "derive");
        assert_eq!(f.dependencies, vec!["serde_derive"]);
    }
}
