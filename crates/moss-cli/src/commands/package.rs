//! Package registry queries.

use moss_packages::{detect_ecosystem, all_ecosystems, PackageInfo, PackageError};
use std::path::Path;

pub fn cmd_package(
    package: &str,
    ecosystem: Option<&str>,
    root: Option<&Path>,
    json: bool,
) -> i32 {
    let project_root = root.unwrap_or(Path::new("."));

    // Get ecosystem either by name or by detection
    let eco: &dyn moss_packages::Ecosystem = if let Some(name) = ecosystem {
        match find_ecosystem_by_name(name) {
            Some(e) => e,
            None => {
                eprintln!("error: unknown ecosystem '{}'", name);
                eprintln!("available: {}", available_ecosystems().join(", "));
                return 1;
            }
        }
    } else {
        match detect_ecosystem(project_root) {
            Some(e) => e,
            None => {
                eprintln!("error: could not detect ecosystem from project files");
                eprintln!("hint: use --ecosystem to specify explicitly");
                eprintln!("available: {}", available_ecosystems().join(", "));
                return 1;
            }
        }
    };

    // Query package info
    match eco.query(package, project_root) {
        Ok(info) => {
            if json {
                print_json(&info);
            } else {
                print_human(&info, eco.name());
            }
            0
        }
        Err(e) => {
            match e {
                PackageError::NotFound(name) => {
                    eprintln!("error: package '{}' not found in {} registry", name, eco.name());
                }
                PackageError::NoToolFound => {
                    eprintln!("error: no {} tools found in PATH", eco.name());
                    eprintln!("hint: install one of: {:?}", eco.tools());
                }
                _ => {
                    eprintln!("error: {}", e);
                }
            }
            1
        }
    }
}

fn find_ecosystem_by_name(name: &str) -> Option<&'static dyn moss_packages::Ecosystem> {
    all_ecosystems()
        .iter()
        .find(|e| e.name() == name)
        .copied()
}

fn available_ecosystems() -> Vec<&'static str> {
    all_ecosystems().iter().map(|e| e.name()).collect()
}

fn print_json(info: &PackageInfo) {
    if let Ok(json) = serde_json::to_string_pretty(info) {
        println!("{}", json);
    }
}

fn print_human(info: &PackageInfo, ecosystem: &str) {
    println!("{} {} ({})", info.name, info.version, ecosystem);

    if let Some(desc) = &info.description {
        println!("{}", desc);
    }

    println!();

    if let Some(license) = &info.license {
        println!("license: {}", license);
    }

    if let Some(homepage) = &info.homepage {
        println!("homepage: {}", homepage);
    }

    if let Some(repo) = &info.repository {
        println!("repository: {}", repo);
    }

    if !info.features.is_empty() {
        println!();
        println!("features:");
        for feature in &info.features {
            if feature.dependencies.is_empty() {
                println!("  {}", feature.name);
            } else {
                println!("  {} = [{}]", feature.name, feature.dependencies.join(", "));
            }
        }
    }

    if !info.dependencies.is_empty() {
        println!();
        println!("dependencies:");
        for dep in &info.dependencies {
            let version = dep.version_req.as_deref().unwrap_or("*");
            let optional = if dep.optional { " (optional)" } else { "" };
            println!("  {} {}{}", dep.name, version, optional);
        }
    }
}
