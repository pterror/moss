"""Go Import Resolution: fisheye for Go codebases.

This module provides import resolution for Go codebases by:
1. Parsing go.mod to understand the module path
2. Scanning packages in the project
3. Resolving import paths to local files

Similar to Python's expand_import_context but for Go.
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


@dataclass
class GoModule:
    """Information about a Go module from go.mod."""

    path: str  # e.g., "github.com/user/project"
    go_version: str | None = None
    require: list[tuple[str, str]] = field(default_factory=list)  # (module, version)
    replace: dict[str, str] = field(default_factory=dict)  # old -> new


@dataclass
class GoPackage:
    """Information about a Go package."""

    name: str  # Package name (from package declaration)
    import_path: str  # Full import path
    dir: Path  # Directory containing the package
    files: list[Path] = field(default_factory=list)
    exports: list[str] = field(default_factory=list)  # Exported symbols (capitalized)


@dataclass
class GoImport:
    """A Go import statement."""

    path: str  # Import path
    alias: str | None = None  # Alias if using named import
    lineno: int = 0


@dataclass
class GoImportReport:
    """Report of Go imports and resolution for a project."""

    module: GoModule | None = None
    packages: list[GoPackage] = field(default_factory=list)
    external_imports: list[str] = field(default_factory=list)

    def to_compact(self) -> str:
        """Return compact format for display."""
        lines = []

        if self.module:
            lines.append(f"# Go Module: {self.module.path}")
            if self.module.go_version:
                lines.append(f"Go version: {self.module.go_version}")

        if self.packages:
            lines.append(f"Packages: {len(self.packages)}")
            for pkg in self.packages[:10]:
                export_count = len(pkg.exports)
                files_count = len(pkg.files)
                lines.append(f"  - {pkg.import_path}: {files_count} files, {export_count} exports")
            if len(self.packages) > 10:
                lines.append(f"  ... and {len(self.packages) - 10} more")

        if self.external_imports:
            unique = sorted(set(self.external_imports))
            lines.append(f"External imports: {len(unique)}")
            for imp in unique[:5]:
                lines.append(f"  - {imp}")
            if len(unique) > 5:
                lines.append(f"  ... and {len(unique) - 5} more")

        return "\n".join(lines) if lines else "No Go module found"

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for JSON output."""
        return {
            "module": {
                "path": self.module.path if self.module else None,
                "go_version": self.module.go_version if self.module else None,
            },
            "packages": [
                {
                    "name": pkg.name,
                    "import_path": pkg.import_path,
                    "dir": str(pkg.dir),
                    "file_count": len(pkg.files),
                    "exports": pkg.exports[:20],
                }
                for pkg in self.packages
            ],
            "external_imports": sorted(set(self.external_imports))[:30],
        }


# Directories to skip
SKIP_DIRS = {
    ".git",
    "vendor",
    "node_modules",
    "testdata",
    ".idea",
}


def parse_go_mod(go_mod_path: Path) -> GoModule | None:
    """Parse a go.mod file.

    Args:
        go_mod_path: Path to go.mod file

    Returns:
        GoModule with parsed information, or None if parsing fails
    """
    try:
        content = go_mod_path.read_text()
    except OSError:
        return None

    module_path = None
    go_version = None
    require: list[tuple[str, str]] = []
    replace: dict[str, str] = {}

    # Parse module path
    module_match = re.search(r'^module\s+(\S+)', content, re.MULTILINE)
    if module_match:
        module_path = module_match.group(1)

    # Parse go version
    go_match = re.search(r'^go\s+(\d+\.\d+(?:\.\d+)?)', content, re.MULTILINE)
    if go_match:
        go_version = go_match.group(1)

    # Parse require block
    require_block = re.search(r'require\s*\((.*?)\)', content, re.DOTALL)
    if require_block:
        for line in require_block.group(1).split('\n'):
            line = line.strip()
            if line and not line.startswith('//'):
                parts = line.split()
                if len(parts) >= 2:
                    require.append((parts[0], parts[1]))

    # Parse single-line requires
    for match in re.finditer(r'^require\s+(\S+)\s+(\S+)', content, re.MULTILINE):
        require.append((match.group(1), match.group(2)))

    # Parse replace directives
    for match in re.finditer(r'^replace\s+(\S+)\s+=>\s+(\S+)', content, re.MULTILINE):
        replace[match.group(1)] = match.group(2)

    if module_path is None:
        return None

    return GoModule(
        path=module_path,
        go_version=go_version,
        require=require,
        replace=replace,
    )


def extract_go_imports(go_file: Path) -> list[GoImport]:
    """Extract imports from a Go source file.

    Args:
        go_file: Path to a .go file

    Returns:
        List of GoImport objects
    """
    try:
        content = go_file.read_text()
    except OSError:
        return []

    imports: list[GoImport] = []

    # Match import blocks: import ( ... )
    import_block = re.search(r'import\s*\((.*?)\)', content, re.DOTALL)
    if import_block:
        lineno = content[:import_block.start()].count('\n') + 1
        for line in import_block.group(1).split('\n'):
            line = line.strip()
            if not line or line.startswith('//'):
                continue

            # Handle aliased imports: alias "path"
            alias_match = re.match(r'(\w+)\s+"([^"]+)"', line)
            if alias_match:
                imports.append(GoImport(
                    path=alias_match.group(2),
                    alias=alias_match.group(1),
                    lineno=lineno,
                ))
            else:
                # Regular import: "path"
                path_match = re.match(r'"([^"]+)"', line)
                if path_match:
                    imports.append(GoImport(
                        path=path_match.group(1),
                        lineno=lineno,
                    ))
            lineno += 1

    # Match single-line imports: import "path"
    for match in re.finditer(r'^import\s+"([^"]+)"', content, re.MULTILINE):
        lineno = content[:match.start()].count('\n') + 1
        imports.append(GoImport(path=match.group(1), lineno=lineno))

    return imports


def extract_go_exports(go_file: Path) -> list[str]:
    """Extract exported symbols from a Go source file.

    Exported symbols in Go are those starting with an uppercase letter.

    Args:
        go_file: Path to a .go file

    Returns:
        List of exported symbol names
    """
    try:
        content = go_file.read_text()
    except OSError:
        return []

    exports: list[str] = []

    # Functions: func Name(...
    func_pattern = r'^func\s+(\([^)]*\)\s+)?([A-Z][a-zA-Z0-9_]*)\s*\('
    for match in re.finditer(func_pattern, content, re.MULTILINE):
        exports.append(match.group(2))

    # Types: type Name ...
    for match in re.finditer(r'^type\s+([A-Z][a-zA-Z0-9_]*)\s', content, re.MULTILINE):
        exports.append(match.group(1))

    # Constants: const Name = ...
    for match in re.finditer(r'^const\s+([A-Z][a-zA-Z0-9_]*)\s', content, re.MULTILINE):
        exports.append(match.group(1))

    # Variables: var Name ...
    for match in re.finditer(r'^var\s+([A-Z][a-zA-Z0-9_]*)\s', content, re.MULTILINE):
        exports.append(match.group(1))

    return list(set(exports))


def get_package_name(go_file: Path) -> str | None:
    """Get the package name from a Go source file.

    Args:
        go_file: Path to a .go file

    Returns:
        Package name, or None if not found
    """
    try:
        content = go_file.read_text()
    except OSError:
        return None

    match = re.search(r'^package\s+(\w+)', content, re.MULTILINE)
    return match.group(1) if match else None


def scan_go_packages(root: Path, module: GoModule) -> list[GoPackage]:
    """Scan a Go project for packages.

    Args:
        root: Project root directory (containing go.mod)
        module: Parsed GoModule

    Returns:
        List of GoPackage objects
    """
    packages: dict[str, GoPackage] = {}

    for go_file in root.rglob("*.go"):
        # Skip excluded directories
        rel_path = go_file.relative_to(root)
        if any(part in SKIP_DIRS for part in rel_path.parts):
            continue

        # Skip test files for now
        if go_file.name.endswith("_test.go"):
            continue

        pkg_dir = go_file.parent
        pkg_dir_rel = pkg_dir.relative_to(root)

        # Calculate import path
        if pkg_dir_rel == Path("."):
            import_path = module.path
        else:
            import_path = f"{module.path}/{pkg_dir_rel}".replace("\\", "/")

        # Get or create package
        if import_path not in packages:
            pkg_name = get_package_name(go_file)
            if pkg_name is None:
                continue

            packages[import_path] = GoPackage(
                name=pkg_name,
                import_path=import_path,
                dir=pkg_dir,
            )

        packages[import_path].files.append(go_file)
        packages[import_path].exports.extend(extract_go_exports(go_file))

    # Deduplicate exports
    for pkg in packages.values():
        pkg.exports = sorted(set(pkg.exports))

    return list(packages.values())


def analyze_go_imports(root: Path) -> GoImportReport:
    """Analyze Go imports for a project.

    Args:
        root: Project root directory

    Returns:
        GoImportReport with module info, packages, and external imports
    """
    # Find go.mod
    go_mod_path = root / "go.mod"
    if not go_mod_path.exists():
        return GoImportReport()

    # Parse go.mod
    module = parse_go_mod(go_mod_path)
    if module is None:
        return GoImportReport()

    # Scan packages
    packages = scan_go_packages(root, module)

    # Collect all imports
    all_imports: list[str] = []
    internal_prefixes = {module.path}

    for go_file in root.rglob("*.go"):
        rel_path = go_file.relative_to(root)
        if any(part in SKIP_DIRS for part in rel_path.parts):
            continue

        for imp in extract_go_imports(go_file):
            all_imports.append(imp.path)

    # Filter to external imports
    external_imports = [
        imp for imp in all_imports
        if not any(imp.startswith(prefix) for prefix in internal_prefixes)
        and not imp.startswith(".")  # Skip relative imports
    ]

    return GoImportReport(
        module=module,
        packages=packages,
        external_imports=external_imports,
    )


def resolve_go_import(
    import_path: str,
    root: Path,
    module: GoModule,
) -> Path | None:
    """Resolve a Go import path to a local directory.

    Args:
        import_path: Import path to resolve
        root: Project root directory
        module: Parsed GoModule

    Returns:
        Path to the package directory, or None if external/not found
    """
    # Check if it's an internal import
    if not import_path.startswith(module.path):
        return None

    # Calculate relative path
    rel_import = import_path[len(module.path):]
    if rel_import.startswith("/"):
        rel_import = rel_import[1:]

    if rel_import:
        target = root / rel_import
    else:
        target = root

    if target.is_dir():
        return target

    return None


def get_go_package_exports(
    import_path: str,
    root: Path,
    module: GoModule,
) -> list[str]:
    """Get exported symbols from a Go package.

    Args:
        import_path: Import path of the package
        root: Project root directory
        module: Parsed GoModule

    Returns:
        List of exported symbol names
    """
    pkg_dir = resolve_go_import(import_path, root, module)
    if pkg_dir is None:
        return []

    exports: list[str] = []
    for go_file in pkg_dir.glob("*.go"):
        if go_file.name.endswith("_test.go"):
            continue
        exports.extend(extract_go_exports(go_file))

    return sorted(set(exports))
