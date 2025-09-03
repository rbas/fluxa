# Fluxa - URL Monitoring Service

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

Fluxa is a Rust-based lightweight monitoring tool that continuously checks URL health and sends Pushover notifications when services go down or recover. It runs as a web service with a health check endpoint.

## Working Effectively

**CRITICAL - NEVER CANCEL BUILDS OR TESTS**: Builds may take up to 50+ minutes, tests take ~2 seconds. Always set timeouts appropriately and wait for completion.

### Bootstrap and Build
- Install Rust if not present: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- `cargo clippy` -- NEVER CANCEL: Takes ~40 seconds first run, ~5 seconds subsequent runs. Set timeout to 120+ seconds.
- `cargo build` -- NEVER CANCEL: Takes ~25 seconds. Set timeout to 60+ seconds.
- `cargo build --release` -- NEVER CANCEL: Takes ~50 seconds. Set timeout to 120+ seconds.

### Testing
- `cargo test` -- Takes ~2 seconds. Tests validate configuration parsing, error handling, and URL validation.
- `cargo fmt --check` -- Format validation. Takes <1 second.

### Running the Application
- **ALWAYS run the build steps first.**
- Create config file based on `config.sample.toml` with your Pushover API credentials
- Debug mode: `RUST_LOG=info ./target/debug/fluxa --config /path/to/config.toml` 
- Production mode: `RUST_LOG=info ./target/release/fluxa --config /path/to/config.toml`
- Health check: `curl http://127.0.0.1:8080/` should return "Ok" with 200 status

### Configuration Requirements
- Must provide `pushover_api_key` and `pushover_user_key` 
- Must define `[fluxa]` section with `listen` address (default: "127.0.0.1:8080")
- Must define at least one `[[services]]` section with `url`, `interval_seconds`, `max_retries`, `retry_interval`

## Manual Validation Requirements

**ALWAYS perform these validation steps after making changes:**

1. **Build Validation**: Run `cargo build --release` and ensure it completes successfully
2. **Test Validation**: Run `cargo test` and ensure all 7 tests pass
3. **Functionality Validation**: 
   - Create test config with valid TOML syntax
   - Run `./target/release/fluxa --config test-config.toml`
   - Verify startup logs show "Spawning monitoring" and "Listening on [address]"
   - Test health endpoint: `curl http://127.0.0.1:8080/` returns "Ok"
   - Let run for 30+ seconds to verify monitoring attempts (will show network errors in sandbox)

## Known Issues and Workarounds

- **Makefile Bug**: `make build` fails because Makefile uses `-release` instead of `--release`. Use `cargo build --release` directly.
- **Network Limitations**: In sandboxed environments, external HTTP requests and DNS resolution will fail. This is expected and does not indicate application problems.
- **Pushover API**: Will show connection errors in sandbox environment, but this validates the monitoring logic is working.

## CI Integration

Always run these before committing or CI will fail:
- `cargo clippy` -- linting validation
- `cargo fmt --check` -- format validation  
- `cargo build --verbose` -- build validation
- `cargo test` -- test validation

The GitHub Actions workflow (`.github/workflows/on.pr.push.yml`) runs these checks on Ubuntu and macOS.

## Key Project Structure

### Frequently Modified Files
- `src/main.rs` -- CLI entry point and main application logic
- `src/settings.rs` -- Configuration parsing and validation (modify when changing config format)
- `src/service.rs` -- URL monitoring and health check logic
- `src/notification.rs` -- Pushover notification handling
- `Cargo.toml` -- Dependencies and project metadata

### Important Config Files
- `config.sample.toml` -- Template showing required configuration format
- `rust-toolchain.toml` -- Rust version specification (stable channel)
- `.github/workflows/` -- CI/CD pipeline definitions

## Common Tasks

### Repository root structure
```
.
├── .github/           # GitHub Actions workflows
├── src/              # Rust source code modules  
├── assets/           # Static assets
├── Cargo.toml        # Rust project configuration
├── Cargo.lock        # Dependency lock file
├── Makefile          # Build shortcuts (HAS BUG - use cargo directly)
├── README.md         # Project documentation
├── config.sample.toml # Configuration template
└── rust-toolchain.toml # Rust version spec
```

### Essential Commands Reference
```bash
# Build and validate (in sequence)
cargo clippy           # ~40s first run, ~5s subsequent
cargo build --release # ~50s 
cargo test            # ~2s
cargo fmt --check     # <1s

# Run application
./target/release/fluxa --config config.toml

# Health check
curl http://127.0.0.1:8080/  # Returns "Ok"

# Get help and version
./target/release/fluxa --help
./target/release/fluxa --version  # Shows current version
```

### Example Working Configuration
```toml
# Pushover API credentials (required)
pushover_api_key = "your-api-key"
pushover_user_key = "your-user-key"

[fluxa]
listen = "127.0.0.1:8080"

[[services]]
url = "https://example.com"
interval_seconds = 300
max_retries = 3
retry_interval = 5
```