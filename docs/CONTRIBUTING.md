# AAMN Contributing Guide

Thank you for your interest in contributing to AAMN! 🎉

---

## Ways to Contribute

- 🐛 **Bug reports** — Open an issue describing the bug and how to reproduce it
- 💡 **Feature requests** — Open an issue describing the feature and its use case
- 🔧 **Pull requests** — See the workflow below
- 📖 **Documentation** — Fix typos, improve clarity, add examples
- 🔐 **Security** — See [SECURITY.md](SECURITY.md) for vulnerability disclosure

---

## Development Workflow

### Prerequisites

- Rust 1.75+ (`rustup toolchain install stable`)
- `rustfmt` and `clippy` (`rustup component add rustfmt clippy`)

### Setup

```bash
git clone https://github.com/Maustral/aamn.git
cd aamn
cargo build
cargo test
```

### Before Opening a PR

All of the following must pass:

```bash
# 1. Format code
cargo fmt

# 2. Lint (no warnings allowed)
cargo clippy -- -D warnings

# 3. Run all tests
cargo test --lib
cargo test --doc

# 4. Security audit
cargo audit
```

### Branch Naming

| Type | Format | Example |
|---|---|---|
| Feature | `feat/description` | `feat/session-rotation` |
| Bug fix | `fix/description` | `fix/dht-panic-on-short-msg` |
| Docs | `docs/description` | `docs/improve-protocol-spec` |
| Chore | `chore/description` | `chore/upgrade-quinn` |

---

## Code Style

- Follow the existing module structure (`src/*.rs`)
- All public items must have `///` doc comments
- Prefer `anyhow::Result` for error propagation
- No `unsafe` blocks without explicit justification
- Keep functions small and focused

---

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add session key rotation
fix: prevent DHT panic on short messages
docs: improve protocol specification
test: add integration test for onion wrap/unwrap
chore: update dependencies
```

---

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](../LICENSE).
