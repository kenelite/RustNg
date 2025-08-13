## RustNg — 云原生反向代理

Rust 编写的可扩展反向代理，采用 Cargo Workspace 模块化，聚焦于动态配置、服务发现、可观测性与可演进的热升级能力。当前仓库包含最小可运行/可测试的脚手架，后续将逐步完善代理能力。

### 特性
- 已具备：
  - 多 crate 工作区与最小可编译实现
  - 核心抽象接口：`Filter`、`Router`、`Upstream`、`Metrics` 等
  - 基础单元测试（`core`/`http`/`config`/`sd`/`admin`/`control`/`e2e-tests`）
  - 可运行占位二进制：`rustng-cli`、`rustng-operator`、`rustng-examples`
- 规划中：
  - HTTP/1.1、HTTP/2、gRPC 转发（`hyper`/`tonic`）
  - HTTP/3/QUIC 支持（`quinn`/`quiche`）
  - 动态配置与服务发现（文件/Consul/K8s/DNS）
  - Admin 管理接口、Prometheus 指标、OpenTelemetry 追踪
  - 热升级（FD 传递或 K8s Rolling Update）

---

### 目录结构
```
RustNg/
├─ Cargo.toml
├─ core/        # 核心抽象与接口
├─ transport/   # 传输层抽象（TCP/TLS/QUIC 等）
├─ http/        # HTTP 层（后续集成 hyper/tonic/quinn）
├─ control/     # 控制面占位
├─ config/      # 配置事件模型与分发
├─ sd/          # 服务发现占位
├─ admin/       # 管理 API 占位
├─ plugins/     # 插件/Filter 占位
├─ cli/         # 命令行工具（可运行）
├─ operator/    # K8s Operator 占位
├─ examples/    # 示例入口（可运行）
└─ e2e-tests/   # 端到端测试（占位）
```

---

### 快速开始
安装 Rust：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup component add clippy rustfmt
```

构建与测试：
```bash
cargo build --workspace
cargo test --workspace
```

代码质量检查：
```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
```

生成文档：
```bash
cargo doc --workspace --no-deps --open
```

---

### 运行示例与二进制
```bash
# CLI（占位）
cargo run -p rustng-cli

# Operator（占位）
cargo run -p rustng-operator

# Examples（占位）
cargo run -p rustng-examples
```

发布构建示例：
```bash
cargo build -p rustng-cli --release
```

---

### 开发约定
- 公共抽象与跨模块契约放入 `core`
- 新增 crate 需在根 `Cargo.toml` 的 `[workspace]` 中登记
- 修改公共接口时附带或更新单元测试
- 提交前确保通过：`cargo fmt`、`cargo clippy -D warnings`、`cargo test`

---

### 路线图（简版）
- 阶段 0：脚手架与基础测试（当前）
- 阶段 1：HTTP/1.1、HTTP/2 代理与连接管理；配置/SD 初版；基本 Admin API
- 阶段 2：gRPC 与 HTTP/3；健康检查与更多负载均衡；WASM 插件
- 阶段 3：热升级完善、生产级可观测性、K8s 集成与 Operator 增强

如需提交问题或建议，欢迎提 Issue/PR。

