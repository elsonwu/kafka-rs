#!/bin/bash
# Auto-format script for Kafka-RS project
# This script formats both Rust code and Markdown documentation

set -e

echo "🔧 Auto-formatting Kafka-RS project..."

# Format Rust code
echo "📝 Formatting Rust code..."
if cargo fmt --all; then
    echo "✅ Rust code formatted successfully"
else
    echo "❌ Rust formatting failed"
    exit 1
fi

# Format Markdown files if markdownlint-cli2 is available
if command -v markdownlint-cli2 &> /dev/null; then
    echo "📑 Formatting Markdown files..."
    if markdownlint-cli2-fix "**/*.md" "#target" "#node_modules"; then
        echo "✅ Markdown files formatted successfully"
    else
        echo "⚠️  Some markdown formatting issues couldn't be auto-fixed"
        echo "💡 Run 'markdownlint-cli2 \"**/*.md\" \"#target\" \"#node_modules\"' to see remaining issues"
    fi
else
    echo "⚠️  markdownlint-cli2 not found, skipping markdown formatting"
    echo "💡 Install with: npm install -g markdownlint-cli2"
fi

# Check if any files were modified
if git diff --quiet; then
    echo "✨ No formatting changes needed"
else
    echo "📋 Files formatted and ready to commit:"
    git diff --name-only
    echo ""
    echo "💡 Review the changes and commit them:"
    echo "   git add ."
    echo "   git commit -m 'style: auto-format code and documentation'"
fi
