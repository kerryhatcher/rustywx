# rustywx — just commands for local development and CI parity
#
# Usage:
#   just          # run all checks (same as CI)
#   just fmt      # format code
#   just lint     # run clippy
#   just test     # run all tests
#   just ci       # run full CI suite locally

default: ci

# ── Formatting ──────────────────────────────────────────────────
fmt:
    cargo fmt --all -- --check

fmt-fix:
    cargo fmt --all

# ── Linting ─────────────────────────────────────────────────────
lint:
    cargo clippy --all-targets --all-features -- -D warnings

lint-fix:
    cargo clippy --all-targets --all-features --fix --allow-dirty --allow-staged

# ── Compilation ─────────────────────────────────────────────────
check:
    cargo check --all-targets --all-features

# ── Testing ─────────────────────────────────────────────────────
test:
    cargo test --all-features

test-doc:
    cargo test --doc

# ── Security ────────────────────────────────────────────────────
audit:
    # RUSTSEC-2026-0192: ttf-parser 0.21.1 unmaintained, transitive via
    # ply-engine -> macroquad-ply -> fontdue. No upstream fix available.
    cargo audit --ignore RUSTSEC-2026-0192

deny:
    cargo deny check

# ── Secrets detection ───────────────────────────────────────────
gitleaks:
    gitleaks detect --source . --verbose

# ── Vulnerability scanning ───────────────────────────────────────
trivy:
    trivy filesystem --scanners vuln,secret,misconfig --severity HIGH,CRITICAL --exit-code 1 .

# ── Spell checking ──────────────────────────────────────────────
typos:
    typos --config typos.toml

# ── Link checking ───────────────────────────────────────────────
lychee:
    lychee --no-progress --exclude 'localhost|127\.0\.0\.1|crates\.io' .

# ── Secrets scan (kingfisher) ───────────────────────────────────
kingfisher:
    kingfisher scan --exclude target --exclude .git --exclude Cargo.lock .

# ── Pre-commit ──────────────────────────────────────────────────
pre-commit:
    pre-commit run --all-files

pre-commit-install:
    pre-commit install
    pre-commit install --hook-type pre-push

# ── Full CI suite (all checks in order) ─────────────────────────
ci: fmt lint check test audit deny gitleaks typos
    @echo ""
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "  All CI checks passed ✓"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ── Full CI suite including slow checks ─────────────────────────
ci-full: ci test-doc trivy lychee kingfisher
    @echo ""
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "  Full CI suite passed ✓"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ── Build ───────────────────────────────────────────────────────
build:
    cargo build --release

# ── Run (release build) ─────────────────────────────────────────
run:
    cd ply-spike && cargo run --release

# ── Clean ───────────────────────────────────────────────────────
clean:
    cargo clean

# ── Setup (install all required tools) ──────────────────────────
setup:
    rustup component add clippy rustfmt
    cargo install cargo-audit cargo-deny just typos-cli lychee
    pip install pre-commit
    pre-commit install
    pre-commit install --hook-type pre-push
    @echo "Run 'just setup-tools' to install external scanners"

setup-tools:
    @echo "Install these tools manually:"
    @echo "  gitleaks:  https://github.com/gitleaks/gitleaks#installing"
    @echo "           or use Docker: source ~/.zshrc.d/gitleaks.zsh"
    @echo "  trivy:     https://trivy.dev/latest/getting-started/installation/"
    @echo "  shellcheck: apt install shellcheck"
