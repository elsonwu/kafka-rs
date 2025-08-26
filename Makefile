# Kafka-RS Development Tasks
# 
# Common development tasks for the Kafka-RS project

.PHONY: help format check test build clean lint fix

help: ## Show this help message
	@echo "Kafka-RS Development Tasks"
	@echo ""
	@echo "Available commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

format: ## Format code with cargo fmt
	@echo "🎨 Formatting code..."
	cargo fmt --all

check: ## Check formatting without making changes
	@echo "🔍 Checking code formatting..."
	cargo fmt --all -- --check

lint: ## Run clippy linter
	@echo "📎 Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings

test: ## Run all tests
	@echo "🧪 Running tests..."
	cargo test --verbose --all-features

build: ## Build the project
	@echo "🔨 Building project..."
	cargo build --verbose --all-targets

build-release: ## Build in release mode
	@echo "🚀 Building release..."
	cargo build --release

integration-test: ## Run integration tests with KafkaJS client
	@echo "🔌 Running integration tests..."
	cd integration/kafka-client-test && npm test

server: ## Start the Kafka server
	@echo "⚡ Starting Kafka-RS server..."
	cargo run --release

fix: format lint ## Fix formatting and linting issues

clean: ## Clean build artifacts
	@echo "🧹 Cleaning..."
	cargo clean

pre-commit: format lint test ## Run pre-commit checks (format, lint, test)
	@echo "✅ Pre-commit checks passed!"

ci: check lint test ## Run CI checks locally
	@echo "✅ CI checks passed!"
