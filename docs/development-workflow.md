# Development Workflow Guide

This document outlines the development workflow for the Kafka-RS project, ensuring code quality and preventing CI failures.

## Quick Setup

To get started with automatic code formatting:

```bash
# Install pre-commit hook for automatic formatting
make install-hooks

# Or manually:
ln -sf ../../scripts/pre-commit.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

## Development Commands

### Essential Commands

```bash
# Format all code (Rust + Markdown)
make format

# Check formatting and linting
make lint

# Run all tests
make test

# Run all CI checks locally (recommended before push)
make ci
```

### Pre-commit Workflow

The pre-commit hook automatically formats code when you commit:

```bash
git add .
git commit -m "your message"  # Automatically formats code
git push
```

### Manual Formatting

If you prefer manual control:

```bash
# Format Rust code only
cargo fmt --all

# Check Rust formatting
cargo fmt --all -- --check

# Run Clippy linter
cargo clippy --all-targets --all-features -- -D warnings
```

## CI Pipeline

Our CI pipeline runs these checks:

1. **Formatting Check**: `cargo fmt --all -- --check`
2. **Linting**: `cargo clippy --all-targets --all-features -- -D warnings`
3. **Build**: `cargo build --verbose --all-targets`
4. **Tests**: `cargo test --verbose --all-features`
5. **Doc Tests**: `cargo test --doc --verbose`
6. **Integration Tests**: KafkaJS client compatibility tests
7. **Security Audit**: `cargo audit`
8. **Code Coverage**: `cargo tarpaulin`

## Preventing CI Failures

### 1. Always Run CI Checks Locally

Before pushing to the remote repository:

```bash
make ci
```

This runs the same checks as the CI pipeline.

### 2. Use Pre-commit Hooks

The pre-commit hook is already set up to:

- Auto-format Rust code with `cargo fmt`
- Auto-format Markdown files (if markdownlint-cli2 is installed)
- Re-stage formatted files

### 3. Address Common Issues

**Formatting Issues:**

```bash
# Fix: Run formatter
cargo fmt --all
git add -u && git commit --amend --no-edit
```

**Linting Issues:**

```bash
# Fix: Address clippy warnings
cargo clippy --all-targets --all-features -- -D warnings
# Fix issues and commit
```

**Test Failures:**

```bash
# Fix: Run tests locally first
cargo test --verbose --all-features
```

## IDE Setup

### VS Code

Add to your `.vscode/settings.json`:

```json
{
    "rust-analyzer.rustfmt.enableRangeFormatting": true,
    "editor.formatOnSave": true,
    "[rust]": {
        "editor.defaultFormatter": "rust-lang.rust-analyzer"
    }
}
```

### Vim/Neovim

Add to your config:

```vim
" Auto-format on save
autocmd BufWritePre *.rs lua vim.lsp.buf.format()
```

## Common Workflow

### Feature Development

```bash
# 1. Create feature branch
git checkout -b feature/my-feature

# 2. Make changes and format
make format

# 3. Run local CI checks
make ci

# 4. Commit (pre-commit hook runs automatically)
git add .
git commit -m "feat: add new feature"

# 5. Push
git push origin feature/my-feature
```

### Bug Fixes

```bash
# 1. Create fix branch
git checkout -b fix/issue-description

# 2. Make changes and test
make test

# 3. Format and lint
make format
make lint

# 4. Run full CI check
make ci

# 5. Commit and push
git add .
git commit -m "fix: resolve issue with XYZ"
git push origin fix/issue-description
```

## Troubleshooting

### Pre-commit Hook Not Working

```bash
# Reinstall hook
make install-hooks

# Or manually
chmod +x .git/hooks/pre-commit
```

### CI Formatting Failures

```bash
# Fix locally and push
make format
git add -u
git commit -m "style: auto-format code"
git push
```

### Markdown Formatting (Optional)

Install markdownlint for better Markdown formatting:

```bash
npm install -g markdownlint-cli2
```

## Integration Tests

For KafkaJS integration tests:

```bash
# Install dependencies
cd integration/kafka-client-test
npm ci

# Run integration test (starts server automatically)
make integration

# Manual testing
make dev &  # Start server in background
cd integration/kafka-client-test && npm test
pkill -f kafka-rs  # Stop server
```

## Best Practices

1. **Always use the pre-commit hook** - prevents most CI failures
2. **Run `make ci` before pushing** - catches issues early
3. **Keep commits focused** - easier to debug CI failures
4. **Write tests for new features** - ensures code quality
5. **Update documentation** - keep docs in sync with code

## Additional Resources

- [Rust formatting guide](https://doc.rust-lang.org/rustfmt/)
- [Clippy lints](https://rust-lang.github.io/rust-clippy/master/)
- [KafkaJS documentation](https://kafka.js.org/)

---

*This workflow ensures high code quality and prevents CI failures. Questions? Check the Makefile for available commands or the CI configuration in `.github/workflows/ci.yml`.*
