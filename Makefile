# Kafka-RS Development Tasks
# Common development tasks for the Kafka-RS project

.PHONY: help build test format lint clean dev install-hooks integration-test audit coverage

# Default target
help: ## Show help message
	@echo "🦀 Kafka-RS Development Tasks"
	@echo ""
	@echo "Build & Test:"
	@echo "  make build           - Build the project"
	@echo "  make test            - Run all tests"
	@echo "  make dev             - Build and run the server"
	@echo "  make integration     - Run integration tests"
	@echo ""
	@echo "Code Quality:"
	@echo "  make format          - Auto-format all code"
	@echo "  make lint            - Run linting checks"
	@echo "  make audit           - Security audit"
	@echo "  make coverage        - Generate code coverage"
	@echo ""
	@echo "Setup:"
	@echo "  make install-hooks   - Install git pre-commit hooks"
	@echo "  make clean           - Clean build artifacts"

# Build the project
build: ## Build the project
	@echo "🔨 Building Kafka-RS..."
	@cargo build --all-targets

# Build for release
build-release: ## Build for release
	@echo "🚀 Building Kafka-RS for release..."
	@cargo build --release --all-targets

# Run tests
test: ## Run all tests
	@echo "🧪 Running tests..."
	@cargo test --verbose --all-features
	@cargo test --doc --verbose

# Auto-format code
format: ## Auto-format all code
	@echo "🎨 Formatting code..."
	@./scripts/format.sh

# Run linting checks
lint: ## Run linting checks
	@echo "📋 Running linting checks..."
	@cargo fmt --all -- --check
	@cargo clippy --all-targets --all-features -- -D warnings

# Clean build artifacts
clean: ## Clean build artifacts
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean
	@rm -rf target/
	@rm -rf integration/kafka-client-test/node_modules/

# Run development server
dev: build-release ## Run development server
	@echo "🚀 Starting Kafka-RS development server..."
	@cargo run --release -- --port 9092

# Install git hooks
install-hooks: ## Install git pre-commit hooks
	@echo "🔗 Installing git pre-commit hooks..."
	@ln -sf ../../scripts/pre-commit.sh .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "✅ Pre-commit hook installed"

# Run integration tests
integration: build-release ## Run integration tests
	@echo "🔌 Running Kafka client integration tests..."
	@cd integration/kafka-client-test && npm ci
	@cargo run --release -- --port 9092 &
	@sleep 5
	@cd integration/kafka-client-test && npm test
	@pkill -f kafka-rs || true

# Security audit
audit: ## Run security audit
	@echo "� Running security audit..."
	@cargo audit

# Code coverage (requires cargo-tarpaulin)
coverage: ## Generate code coverage
	@echo "📊 Generating code coverage..."
	@cargo tarpaulin --verbose --all-features --workspace --timeout 120 --out html
	@echo "Coverage report generated in tarpaulin-report.html"

# Check everything (used by CI)
check-all: format lint test ## Run all checks
	@echo "✅ All checks passed!"

# Quick development cycle
quick: format build test ## Quick development check
	@echo "🏃 Quick development check complete!"

# Pre-commit workflow
pre-commit: format lint test ## Run pre-commit checks (format, lint, test)
	@echo "✅ Pre-commit checks passed!"

# CI workflow  
ci: format lint test ## Run CI checks locally
	@echo "✅ CI checks passed!"
