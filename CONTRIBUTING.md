# Contributing to Velocity

Thank you for your interest in contributing to Velocity! This document provides guidelines and information for contributors.

## Code of Conduct

Please be respectful and considerate of others. We want Velocity to be a welcoming project for everyone.

## Getting Started

### Prerequisites

- **Rust 1.70+** (stable toolchain)
- **Git**
- **Node.js** (for testing package installation)

### Setting Up Development Environment

```bash
# Clone the repository
git clone https://github.com/justbytecode/velocity.git
cd velocity

# Build the project
cargo build

# Run tests
cargo test

# Run with verbose logging
RUST_LOG=debug cargo run -- --help
```

## Development Workflow

### Branch Naming

- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation changes
- `refactor/description` - Code refactoring

### Making Changes

1. **Fork** the repository
2. **Create a branch** from `main`
3. **Make your changes**
4. **Write/update tests**
5. **Run the test suite**
6. **Submit a pull request**

### Code Style

We follow standard Rust conventions:

```bash
# Format code
cargo fmt

# Check linting
cargo clippy --all-targets --all-features

# Run all checks
cargo fmt --check && cargo clippy && cargo test
```

### Commit Messages

Use conventional commits:

```
feat: add support for private registries
fix: correct version constraint parsing
docs: update README with examples
refactor: simplify cache implementation
test: add tests for workspace detection
```

## Project Structure

```
src/
├── cli/           # Command-line interface
│   └── commands/  # Individual command implementations
├── core/          # Core types and engine
├── resolver/      # Dependency resolution
├── installer/     # Package installation
├── cache/         # Caching layer
├── security/      # Security features
├── workspace/     # Monorepo support
├── registry/      # npm registry client
├── templates/     # Project scaffolding
├── permissions/   # Permission system
└── utils/         # Utility functions
```

## Adding a New Command

1. Create `src/cli/commands/mycommand.rs`:

```rust
use clap::Args;
use crate::core::VelocityResult;

#[derive(Args)]
pub struct MyCommandArgs {
    #[arg(short, long)]
    pub flag: bool,
}

pub async fn execute(args: MyCommandArgs, json_output: bool) -> VelocityResult<()> {
    // Implementation
    Ok(())
}
```

2. Add to `src/cli/commands/mod.rs`:

```rust
pub mod mycommand;
```

3. Add to `src/cli/mod.rs`:

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ...
    MyCommand(mycommand::MyCommandArgs),
}
```

4. Add to `src/main.rs`:

```rust
Commands::MyCommand(args) => cli::commands::mycommand::execute(args, json_output).await,
```

## Adding a New Framework Template

1. Create `src/templates/myframework.rs`:

```rust
use std::path::Path;
use crate::core::VelocityResult;
use crate::templates::Template;

pub struct MyFrameworkTemplate {
    typescript: bool,
}

impl MyFrameworkTemplate {
    pub fn new(typescript: bool) -> Self {
        Self { typescript }
    }
}

impl Template for MyFrameworkTemplate {
    fn name(&self) -> &str {
        "myframework"
    }

    fn generate(&self, target: &Path) -> VelocityResult<()> {
        // Generate files
        Ok(())
    }
}
```

2. Add to `src/templates/mod.rs`

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_function() {
        assert_eq!(my_function(), expected_result);
    }
}
```

### Integration Tests

Place in `tests/` directory:

```rust
use assert_cmd::Command;

#[test]
fn test_init_command() {
    let mut cmd = Command::cargo_bin("velocity").unwrap();
    cmd.arg("init")
        .arg("--yes")
        .assert()
        .success();
}
```

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_name

# With output
cargo test -- --nocapture

# Integration tests only
cargo test --test '*'
```

## Documentation

- Update README.md for user-facing changes
- Add doc comments to public APIs:

```rust
/// Resolves dependencies from a dependency map.
///
/// # Arguments
/// * `dependencies` - Map of package names to version constraints
///
/// # Returns
/// A `Resolution` containing the dependency graph and lockfile
pub async fn resolve(&self, dependencies: &HashMap<String, String>) -> VelocityResult<Resolution> {
    // ...
}
```

## Performance Considerations

- Use async I/O for network operations
- Prefer streaming over loading entire files
- Use content-addressable caching
- Profile hot paths with `cargo flamegraph`

## Security Considerations

- All package data must be verified
- Never trust user input or package contents
- Path traversal checks on all file operations
- No implicit script execution



## Questions?

Open an issue or discussion on GitHub!
