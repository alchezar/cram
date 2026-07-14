SHELL := /bin/bash
LOAD_ENV := set -eo pipefail; set -a; source ./.env; set +a;

# Where 'make install' puts the self-contained bundle (override: PREFIX=path)
PREFIX ?= $(HOME)/.cargo/cram

.PHONY: help
# Environment
.PHONY: migrate migrate-reset
# Code quality
.PHONY: check ci validate fmt lint prepare
# Testing
.PHONY: test test-one test-in test-not
# Development
.PHONY: build release install dev run clean

# Help -------------------------------------------------------------------------

help: ## Show available targets
	@grep -E '^[a-zA-Z0-9_.-]+:.*?## ' Makefile                                \
	| sort                                                                     \
	| awk 'BEGIN {FS = ":.*?## "}; {printf "  make %-14s %s\n", $$1, $$2}'

# Database ---------------------------------------------------------------------

migrate: ## Apply pending migrations (non-destructive)
	@echo "[*] Applying pending migrations..."
	@$(LOAD_ENV) cargo sqlx migrate run --source migrations

migrate-reset: ## Drop and recreate database, then run all migrations (DESTRUCTIVE)
	@echo "[!] Resetting database - all data will be lost."
	@$(LOAD_ENV) cargo sqlx database reset -y --source migrations


# Code Quality -----------------------------------------------------------------

ci: fmt lint build ## CI pipeline (GitHub Actions)

validate: check test ## Validate full local pipeline (requires running environment)

check: fmt prepare lint ## Full code quality check (requires .env and db)

fmt: ## Check and fix formatting if needed
	@echo "[*] Checking formatting..."
	@cargo fmt --all -- --check \
		|| (echo "[*] Formatting code..." && cargo fmt --all)

lint: ## Run clippy in strict mode
	@echo "[*] Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings

prepare: ## Generate SQLx offline query metadata for CI builds (bash/zsh)
	@echo "[*] Generating SQLx offline query metadata..."
	@test -f .env || (echo "Error: .env file not found" && exit 1)
	@$(LOAD_ENV)                                                               \
		SQLX_OFFLINE=false                                                     \
		CARGO_TERM_COLOR=always cargo                                          \
		sqlx prepare -- --all-features --tests 2>&1                            \
		| grep -v 'query data written'

# Testing ----------------------------------------------------------------------

test: ## Run nextest (use ARGS="..." for extra arguments)
	@echo "[*] Running tests..."
	@$(LOAD_ENV) cargo nextest run --all-features --no-fail-fast $(ARGS)

test-one: ## Run single test: `make test_one <test_name>`
	@$(MAKE) test ARGS="$(filter-out $@,$(MAKECMDGOALS))"

test-in: ## Run tests in module: `make test_in <module_name>`
	@$(MAKE) test ARGS="$(patsubst %,--test %,$(filter-out $@,$(MAKECMDGOALS)))"

test-not: ## Exclude tests: `make test_not <test1> <test2> ...`
	@args="$(filter-out $@,$(MAKECMDGOALS))";                                  \
	expr=$$(printf '%s\n' $$args                                               \
		| sed 's/.*/not test(&)/'                                              \
		| paste -sd' and ' -);                                                 \
	$(MAKE) test ARGS="-E '$$expr'"

# Development ------------------------------------------------------------------

build: ## Build all workspace crates
	@echo "[*] Building workspace..."
	@cargo build

release: ## Build release binary and copy to project root
	@echo "[*] Building (release)..."
	@cargo build --bin cram --release
	@cp target/release/cram .
	@echo "[+] Built."

install: ## Install a self-contained bundle to $(PREFIX) and symlink cram into ~/.cargo/bin
	@echo "[*] Building (release)..."
	@test -f cram.toml || (echo "[!] Error: cram.toml not found" && exit 1)
	@cargo build --bin cram --release
	@echo "[*] Installing bundle into $(PREFIX)/ ..."
	@rm -rf "$(PREFIX)"
	@mkdir -p "$(PREFIX)"
	@cp target/release/cram "$(PREFIX)/"
	@cp cram.toml "$(PREFIX)/cram.toml"
	@cp roadmap.toml "$(PREFIX)/"
	@cp -R quizzes web "$(PREFIX)/"
	@mkdir -p "$(HOME)/.cargo/bin"
	@ln -sf "$(PREFIX)/cram" "$(HOME)/.cargo/bin/cram"
	@echo "[+] Installed. Run 'cram' from anywhere (needs ~/.cargo/bin on PATH)."

dev: ## Run in debug mode with .env loaded
	@echo "[*] Running (debug)..."
	@set -a && . ./.env && set +a && cargo run --bin cram

run: ## Run in release mode
	@echo "[*] Running (release)..."
	@cargo run --bin cram --release

clean: ## Clean build artifacts
	@echo "[*] Cleaning build artifacts..."
	@cargo clean

# Prevent "No rule to make target" error for arguments
#   %: — catch-all target, matches any unknown target (e.g. arguments like "test_name")
#   @: — no-op command, does nothing silently
%:
	@:
