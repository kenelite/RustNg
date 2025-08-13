# Rust 云原生反向代理 — 项目结构与设计说明

> 该文档给出一个可扩展、模块化的 Rust 项目结构（workspace + 多 crate），以及每个子模块的设计要点、交互模式、选型建议与实现要点。目标特性：
> 1. 动态配置
> 2. 服务发现
> 3. 在线热升级（零停机切换）
> 4. 控制面管理接口
> 5. 支持 HTTP/2, HTTP/3, gRPC

---

## 概览

核心思想：把系统拆成若干职能明确的 crate（微模块），通过定义良好的契约（protobuf / serde JSON/YAML）和异步消息/事件总线解耦。运行时以 Tokio 为主异步 runtime，HTTP 底层使用 `hyper`/`tonic`（HTTP/1/2/gRPC），QUIC/HTTP3 使用 `quinn` 或 `quiche` 封装层；TLS 使用 `rustls`。

工作流（高层）：
- 控制面（Control Plane）或配置源（K8s/Consul/etcd/文件）变更 -> 更新配置存储
- 配置管理器（Config Manager）下发增量配置事件 -> 转发到路由/上游管理器
- 上游管理器（Upstream Manager）维护后端列表（支持健康检查 + 权重）
- 代理核心（Proxy Core）基于最新配置进行流量转发
- 管理 API 提供观察/管理/手动热重载接口

---

## 推荐 Cargo Workspace（顶层 `Cargo.toml`）

```toml
[workspace]
members = [
  "core",
  "transport",
  "http",
  "control",
  "config",
  "sd",
  "admin",
  "cli",
  "plugins",
  "examples",
  "operator",
  "e2e-tests",
]
```

每个成员为一个 crate（库或二进制）。下面详细说明。

---

## 目录结构示例

```
RustNg/
├─ Cargo.toml  # workspace
├─ README.md
├─ core/                # 核心抽象：请求上下文、filter chain、router、metrics API
├─ transport/           # 低层 transport：tcp/udp/quic listener 封装、socket 接管、socket activation
├─ http/                # http 层实现：hyper/tonic 适配、http/2/1 的 connection 管理
├─ control/             # 控制面（control-plane）实现：xDS-like 或 自定义 gRPC/REST API
├─ config/              # 配置管理：schema、热加载、delta apply
├─ sd/                  # 服务发现插件：k8s、consul、dns、static
├─ admin/               # admin server（HTTP REST + gRPC）用于管理与监控
├─ plugins/             # 插件宿主（例如 WASM runtime 接口）
├─ cli/                 # 可选：启动/管理 CLI
├─ operator/            # k8s operator（可选）
├─ examples/            # example configs and demo servers
└─ e2e-tests/           # end-to-end tests
```

---

## 各 crate 设计要点

### 1) `core`
职责：定义系统的核心抽象、接口与契约。
内容建议：
- `RequestContext`、`ResponseContext`：跨模块传递的结构
- `Filter` trait：类似 Nginx 的 filter chain（请求/响应/流量控制）
- `Router` 接口：给定请求 -> 选择 route/upstream
- `Upstream` 抽象：封装单个后端节点的元信息与运行时状态
- `HealthChecker` trait
- `metrics` 抽象（prometheus client wrapper）

示例 trait：
```rust
pub trait Filter: Send + Sync {
    fn on_request(&self, ctx: &mut RequestContext) -> FilterResult;
    fn on_response(&self, ctx: &mut ResponseContext) -> FilterResult;
}
```

---

### 2) `transport`
职责：监听器管理、socket 接管、低层网络 I/O
要点：
- 提供统一的 `Listener` 抽象（支持 TCP + TLS + QUIC）
- 支持 socket activation & fd passing（实现热升级时的 listener 传递）
- 连接/流限制、accept 循环、SO_REUSEPORT 支持
- 使用 `socket2`、`tokio::net`、`quinn` 等库


---

### 3) `http`
职责：实现 HTTP/1.1、HTTP/2、HTTP/3 及 gRPC 转发
要点：
- 使用 `hyper` 作为 HTTP/1.1/2 server & client（或 `h2` 直接使用）
- gRPC：使用 `tonic` 作为 gRPC server/client（基于 hyper/h2）
- HTTP/3：封装 `quinn`（或 `quiche`）并提供与 core 的统一连接/stream 接口
- 提供反向代理逻辑：请求重写、头部注入/删除、流控、连接池管理
- 连接池/HTTP Keepalive/流并发限制


---

### 4) `config`
职责：配置 schema、热加载与增量下发
要点：
- 使用 protobuf 或 serde 定义配置 schema（建议用 proto + proto JSON for wire）
- 支持多种 config 源：文件、gRPC push（control plane）、k8s CRD、etcd watch
- 提供 `ConfigManager`：将配置变化转换为事件（增量/全量）分发给 `core` 与 `sd`
- 配置验证与回滚：当新配置校验失败时自动回滚

事件模型示例：
```
ConfigManager -> emits: RouteAdded/RouteRemoved/UpstreamUpdated/PluginReload
Subscribers: core.router, upstream.manager, plugins.host
```

---

### 5) `sd`（Service Discovery）
职责：从不同注册中心获取后端实例
插件化实现：
- `k8s`（watch Service/Endpoints/Ingress/CRD）
- `consul`（catalog/services + health checks）
- `dns`（A/AAAA/SRV）
- `static`（手工配置）

行为：发现变更 -> 归一化为 `Upstream` 节点 -> 发送事件给 `core`/`config`。

---

### 6) `control`
职责：控制面实现（可选：支持 xDS）
要点：
- 提供 gRPC/config API（供 CI/CD 或控制台调用），也可以实现 Envoy xDS compatibility（可选）
- 支持 push 模式（服务器推送配置更新）与 pull 模式（代理定期拉取）
- 对接认证（mTLS、JWT）

示例 proto（高层概念）：
```proto
service ControlPlane {
  rpc StreamConfig(stream ConfigRequest) returns (stream ConfigResponse);
  rpc ApplyConfig(ApplyConfigRequest) returns (ApplyConfigResponse);
}
```

---

### 7) `admin`
职责：运行时管理面板与诊断接口
- 管理 API（REST/gRPC）：查询路由、后端状态、metrics、主动下线、触发热重载
- 支持 web UI（可选）或与外部 UI 集成
- 认证/权限（basic token / mTLS / OAuth）

---

### 8) `plugins`
职责：提供扩展能力：WASM 或 native plugins
- 推荐使用 WASM（wasmtime / wasi）以获得安全隔离与热加载能力
- 提供 Filter API 的插件宿主（在请求/响应生命周期被调用）

---

### 9) `cli`, `operator`, `examples`, `e2e-tests`
- `cli`：管理和调试工具（比如发起 graceful reload, show config）
- `operator`：K8s Operator，管理 CRD/Deployment 与 rollout
- `examples`：示例服务、示例配置
- `e2e-tests`：集成测试，模拟 config push、SD 变更、流量切换、升级验证

---

## 动态配置策略

两种主流方式：
1. **Push 模式（Control Plane 推送）**：控制面通过 gRPC/HTTP 将增量配置推送到代理（可用 stream）——延迟小、易回滚。类似 Envoy xDS。
2. **Pull/Watch 模式**：代理订阅 k8s watch 或 etctd/consul 的 watch 接口，或轮询配置文件。

实现要点：
- 统一事件总线（例如 `tokio::sync::broadcast` 或 `async-broadcast` / `flume`）分发配置变更
- 增量更新优先，避免重建所有运行时对象
- 保证配置应用的原子性 & 回滚策略

---

## 服务发现（SD）与健康检查

- SD 插件负责将 upstream 节点变更映射为 `Upstream` 实例并上报
- 上游管理器执行主动健康检查（HTTP / gRPC health / TCP）并根据健康情况调整权重或剔除
- 支持权重、故障速率（circuit breaker）、延迟/容量感知的负载均衡策略（least connections, ring-hash, consistent-hash）

---

## 在线热升级（Zero-downtime rolling / hot replace）

可选技术方案（按复杂度与适配性排序）：

1. **进程接管（Socket FD Passing）**（推荐）
   - 新进程在启动时从旧进程接收已打开的监听 socket（Unix domain socket 或者 systemd socket activation）
   - 老进程停止接受新连接，但继续处理现有连接（优雅关闭），当所有连接完成或超时后退出
   - Rust 实现要点：使用 `socket2` / `nix` 进行 fd 传递；在启动中支持 `--inherit-fds` 或 ENV 约定

   优点：二进制可替换，最低层 listener 状态保持，适合本地/VM 部署

2. **K8s 的 Rolling Update**（实际生产中常用）
   - 在 Kubernetes 中通过 Deployment rolling update 实现零停机；结合 readinessProbe + gracefulShutdownSeconds
   - 设计集成 operator 来协调 pub/sub 配置与 graceful shutdown

   优点：无需 FD 传递，借助 k8s 已有能力

3. **WASM/插件热替换（灰度或功能切换）**
   - 把业务逻辑或 filter 做成 WASM 模块，在运行时热替换模块（不替换代理二进制）

   优点：更细粒度的热更，快速回滚

实现注意事项：
- 为避免内存泄露，要保证旧进程在给定超时后强制退出
- 所有长期运行的 state（连接池、缓存）应有明确序列化/反序列化策略（若需要迁移）

---

## 控制面管理接口（Admin / Control Plane）

建议组合：
- 管理 API（REST）供 UI/运维调用（查询、手动下线、触发 reload）
- 推送 API（gRPC stream）供 CI/CD 或 control plane 推送配置
- 提供审计日志、变更历史、版本化配置
- 身份认证：mTLS 或 OAuth2/JWT

---

## HTTP/2, HTTP/3, gRPC 选型与实现提示

- HTTP/1.1 & HTTP/2: `hyper` + `h2`（`tonic` 基于此实现 gRPC）
- gRPC: `tonic`（server/client）
- HTTP/3/QUIC: `quinn` 或 `quiche`（`quinn` 更易用并与 tokio 集成良好）
- TLS: `rustls`（与 `quinn` / `hyper` 组合）

统一抽象：为 connection/stream 提供统一 trait，使 core 能无感切换：
```rust
pub trait StreamConn: Send + Sync {
    async fn read(&mut self) -> Result<bytes::Bytes, Box<dyn std::error::Error + Send + Sync>>;
    async fn write(&mut self, bytes: bytes::Bytes) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    fn remote_addr(&self) -> std::net::SocketAddr;
}
```

---

## 可扩展的 Filter / 插件 设计

- 提供同步/异步 Filter API
- 插件执行时机（on_request, on_response, on_route_decision, on_stream_event）
- 推荐使用 WASM：安全、可语言无关、支持热加载
- 为 native plugin 提供 ABI（慎用，兼容性开销大）

---

## Observability（可观测性）

必须：
- Prometheus metrics（请求数、延迟、上游健康、conn count）
- OpenTelemetry traces（支持 gRPC trace context & W3C traceparent）
- Structured logs（JSON）并支持 log level 热切换
- Diagnostic endpoints（/health, /ready, /metrics, /debug/vars）

---

## 测试 & CI

- 单元测试覆盖核心逻辑（router、filter、config parsing）
- integration tests：使用 `e2e-tests` crate，启动真实监听，注入配置，做流量断言
- 压力测试：wrk/ghz/fortio
- CI pipeline：cargo fmt, clippy, cargo test, cross-compile artifacts

---

## 示例：热重启（FD Passing）流程（伪代码说明）

1. **旧进程**接收到 SIGHUP/USR2 表示要替换：
   - 停止 accept（或将 accept 放到单独线程并停止），把 listener fds 发送到新进程的 unix socket
   - 继续处理现有连接，设置 graceful timeout
2. **新进程**启动时检查是否有传入的 fds：
   - 使用这些 fds 作为 listener 继续 accept
   - 加载新二进制的配置并逐步接管

伪代码：
```text
old -> send fds -> new
old: stop accepting; wait until active_connections == 0 or timeout -> exit
new: receive fds -> start accept loop
```

---

## 性能与安全考虑

- 内存分配：prefer `mimalloc` 或 `jemalloc`（benchmark 评估）
- CPU 缓存友好型数据结构：避免全局锁，采用 lock-free 或 shard locks
- DoS 防护：限制 header 大小、请求速率、并发流量
- TLS 卸载：支持外部 TLS 终止器（但仍需内置 rustls）

---

## Roadmap 提议（迭代开发计划）

阶段 0：PoC
- minimal proxy 支持 HTTP/1.1, HTTP/2 转发
- 基础配置（文件）与热重载（进程内）

阶段 1：核心功能
- service discovery（static + dns + k8s endpoints）
- control plane stub（gRPC push）
- admin API + metrics

阶段 2：扩展性
- WASM 插件
- HTTP/3 支持
- more LB 策略 & health checks

阶段 3：成熟
- Envoy xDS 兼容性（可选）
- k8s operator
- 安全审计 & 商业特性