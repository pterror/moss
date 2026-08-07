# Getting Started

Welcome to the project documentation. This guide walks you through setup and usage.

## Installation

Install the tool using your package manager:

```bash
cargo install normalize
```

Or clone and build from source:

```bash
git clone https://github.com/example/normalize
cd normalize
cargo build --release
```

## Configuration

Create a `normalize.toml` in your project root:

```toml
[rules]
enabled = true

[[rules.overrides]]
rule = "rust/unwrap-in-impl"
severity = "warning"
```

### Environment Variables

| Variable      | Description                    | Default |
|---------------|--------------------------------|---------|
| `NO_COLOR`    | Disable color output           | unset   |
| `NORMALIZE_DB`| Path to the SQLite index       | `.normalize/index.db` |

## Usage

### Analyzing Code

Run the analyzer on your project:

```bash
normalize analyze
normalize analyze --complexity src/
normalize rules run crates/
```

### Viewing Symbols

```bash
normalize view src/lib.rs
normalize view src/lib.rs/MyStruct
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

See `CONTRIBUTING.md` for details.

## Roadmap

- [x] Ship the CLI
- [x] Publish to crates.io
- [ ] Add an LSP server
  - [ ] Hover support
  - [ ] Go-to-definition
- [ ] Add a VS Code extension

> **Note:** roadmap items are tracked in `TODO.md`, not here — this list is
> illustrative only.

## Support

Questions and bug reports go to the [issue tracker](https://github.com/example/normalize/issues)
or the [`#normalize` channel][chat]. You can also email <support@example.com>
directly, or browse <https://rhi.zone/normalize/> for the full docs.

See the ![project logo](logo.png "normalize") in the repo root.

[chat]: https://chat.example.com/normalize "Project chat"

License
=======

Distributed under the MIT license. See `LICENSE` for the full text.
