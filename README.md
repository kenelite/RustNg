## RustNg — Cloud-Native Reverse Proxy

An extensible reverse proxy written in Rust. The project uses a Cargo workspace for modularity and focuses on dynamic configuration, service discovery, observability, and evolvable hot upgrade capabilities. The repository currently contains a minimal runnable/testable scaffold; proxy features will be iteratively implemented.

### Features
- Implemented (baseline):
  - Multi-crate workspace with minimal compilable implementations
  - Core abstractions: `Filter`, `Router`, `Upstream`, `Metrics`, etc.
  - Basic unit tests (`core`/`http`/`config`/`sd`/`admin`/`control`/`e2e-tests`)
  - Runnable placeholder binaries: `rustng-cli`, `rustng-operator`, `rustng-examples`
- Planned:
  - HTTP/1.1 and HTTP/2 proxying, gRPC (`hyper`/`tonic`)
  - HTTP/3/QUIC support (`quinn`/`quiche`)
  - Dynamic config and service discovery (file/Consul/K8s/DNS)
  - Admin API, Prometheus metrics, OpenTelemetry traces
  - Hot upgrade (FD passing or K8s rolling update)

---

### Directory Layout
```
RustNg/
├─ Cargo.toml
├─ core/        # Core abstractions and interfaces
├─ transport/   # Transport layer abstractions (TCP/TLS/QUIC)
├─ http/        # HTTP layer (to integrate hyper/tonic/quinn)
├─ control/     # Control-plane placeholder
├─ config/      # Config event model and fan-out
├─ sd/          # Service discovery placeholder
├─ admin/       # Admin API placeholder
├─ plugins/     # Plugin/Filter placeholder
├─ cli/         # CLI (runnable)
├─ operator/    # K8s Operator placeholder
├─ examples/    # Example entrypoint (runnable)
└─ e2e-tests/   # End-to-end tests (placeholder)
```

---

### Quickstart
Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup component add clippy rustfmt
```

Build and test:
```bash
cargo build --workspace
cargo test --workspace
```

Code quality checks:
```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

Generate docs:
```bash
cargo doc --workspace --no-deps --open
```

---

### Run examples and binaries
```bash
# CLI (placeholder)
cargo run -p rustng-cli

# Operator (placeholder)
cargo run -p rustng-operator

# Examples (placeholder)
cargo run -p rustng-examples
```

Release build example:
```bash
cargo build -p rustng-cli --release
```

---

### Development Conventions
- Place shared abstractions and cross-module contracts in `core`
- Register any new crate under `[workspace]` in the root `Cargo.toml`
- Update or add unit tests when changing public interfaces
- Before committing, ensure: `cargo fmt`, `cargo clippy -D warnings`, `cargo test`

---

### Roadmap (short)
- Phase 0: Scaffold and basic tests (current)
- Phase 1: HTTP/1.1 and HTTP/2 proxy and connection management; initial Config/SD; basic Admin API
- Phase 2: gRPC and HTTP/3; health checks and more load-balancing strategies; WASM plugins
- Phase 3: Robust hot upgrades, production-grade observability, deeper K8s integration and Operator enhancements

Contributions are welcome — please open issues/PRs.

