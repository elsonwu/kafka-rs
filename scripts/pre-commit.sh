#!/bin/bash
# Pre-commit hook to auto-format code before committing
# Install this with: ln -sf ../../scripts/pre-commit.sh .git/hooks/pre-commit

set -e

echo "🔍 Running pre-commit formatting..."

# Check if we're in the project root
if [ ! -f "Cargo.toml" ] || [ ! -d "src" ]; then
    echo "❌ Not in project root directory"
    exit 1
fi

# Format Rust code
echo "📝 Formatting Rust code..."
cargo fmt --all

# Format Markdown if available
if command -v markdownlint-cli2 &> /dev/null; then
    echo "📑 Formatting Markdown files..."
    markdownlint-cli2-fix "**/*.md" "#target" "#node_modules" || true
fi

# Check if formatting changed anything
if ! git diff --quiet --cached; then
    echo "✨ Code formatted successfully"
    # Re-stage formatted files
    git add -u
else
    echo "✨ No formatting changes needed"
fi

echo "✅ Pre-commit formatting complete"
