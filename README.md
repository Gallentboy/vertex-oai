# Vertex-OAI

<div align="center">

**🚀 Google Cloud Vertex AI 的 OpenAI 兼容网关**

[![Rust](https://img.shields.io/badge/Rust-1.83+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

[English](README.md) | [简体中文](README_CN.md)

</div>

---

## 📖 简介

Vertex-OAI 是一个高性能的 Rust 应用程序,作为 Google Cloud Vertex AI 和 OpenAI API 之间的桥梁。它允许您使用标准的 OpenAI SDK 和工具来访问 Google 的 Gemini 模型,无需修改现有代码。

### ✨ 核心特性

- 🔄 **完全兼容 OpenAI API** - 支持 `/v1/chat/completions` 和 `/v1/models` 端点
- ⚡ **高性能** - 使用 Rust 和 Axum 框架构建,支持异步处理和 HTTP/2
- 🔐 **自动认证** - 自动管理 GCP 访问令牌,无需手动处理
- 💾 **智能缓存** - 使用 Moka 缓存模型列表,减少 API 调用
- 🌊 **流式支持** - 完整支持流式响应(SSE)
- 📦 **单一二进制** - 编译为独立可执行文件,无需运行时依赖
- 🔒 **安全优化** - 使用 rustls 替代 OpenSSL,减少安全风险
- 🎛️ **命令行控制** - 类似 Redis/Nginx 的进程管理(Unix)
- 🔧 **环境变量配置** - 支持 .env 文件,简化配置管理
- 🌍 **跨平台支持** - Unix/Linux/macOS/Windows

---

## 🎯 使用场景

- 在现有 OpenAI 应用中使用 Google Gemini 模型
- 统一多个 AI 提供商的 API 接口
- 为 Vertex AI 提供 OpenAI 兼容的访问层
- 构建多模型 AI 应用的统一网关

---

## 🚀 快速开始

### 前置要求

- **Rust** 1.83 或更高版本
- **Google Cloud Platform** 账户
- 已启用 Vertex AI API 的 GCP 项目
- 配置好的 GCP 认证凭据

### 安装

#### 方法 1: 从源码编译

```bash
# 克隆仓库
git clone https://github.com/Gallentboy/vertex-oai.git
cd vertex-oai

# 编译 Release 版本
cargo build --release

# 二进制文件位于 target/release/vertex-oai
```

#### 方法 2: 使用 Cargo 安装

```bash
cargo install --path .
```

### 配置

#### 1. 设置 GCP 认证

确保您已配置 GCP 认证凭据:

```bash
# 方法 1: 使用 gcloud CLI 登录
gcloud auth application-default login

# 方法 2: 设置服务账号密钥
export GOOGLE_APPLICATION_CREDENTIALS="/path/to/service-account-key.json"
```

#### 2. 配置环境变量

**推荐方式 - 使用 .env 文件:**

```bash
# 复制示例文件
cp .env.example .env

# 编辑配置
nano .env
```

`.env` 文件内容:
```bash
GCP_PROJECT_ID=your-gcp-project-id
GCP_LOCATION=global
PORT=8087
RUST_LOG=info
```

**或使用环境变量:**

```bash
export GCP_PROJECT_ID="your-gcp-project-id"
export PORT="8087"
```

### 运行

#### Unix/Linux/macOS

```bash
# 启动守护进程
./vertex-oai start

# 查看状态
./vertex-oai status

# 停止服务
./vertex-oai stop

# 重启服务
./vertex-oai restart

# 前台运行(开发模式)
./vertex-oai
```

#### Windows

```bash
# 前台运行
.\vertex-oai.exe

# 守护进程功能不可用,请使用 NSSM 或 WinSW
# 详见 WINDOWS.md
```

---

## 📚 API 使用

### 健康检查

```bash
curl http://localhost:8087/
```

### 获取可用模型

```bash
curl http://localhost:8087/v1/models
```

### 聊天补全(非流式)

```bash
curl http://localhost:8087/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gemini-2.0-flash-exp",
    "messages": [
      {
        "role": "user",
        "content": "你好,请介绍一下你自己"
      }
    ]
  }'
```

### 聊天补全(流式)

```bash
curl http://localhost:8087/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "gemini-2.0-flash-exp",
    "messages": [
      {
        "role": "user",
        "content": "写一首关于 Rust 的诗"
      }
    ],
    "stream": true
  }'
```

### 使用 OpenAI Python SDK

```python
from openai import OpenAI

# 配置客户端指向本地网关
client = OpenAI(
    base_url="http://localhost:8087/v1",
    api_key="dummy-key"  # 网关不需要 API key
)

# 使用 Gemini 模型
response = client.chat.completions.create(
    model="gemini-2.0-flash-exp",
    messages=[
        {"role": "user", "content": "Hello!"}
    ]
)

print(response.choices[0].message.content)
```

### 使用 OpenAI Node.js SDK

```javascript
import OpenAI from 'openai';

const client = new OpenAI({
  baseURL: 'http://localhost:8087/v1',
  apiKey: 'dummy-key'
});

const response = await client.chat.completions.create({
  model: 'gemini-2.0-flash-exp',
  messages: [
    { role: 'user', content: 'Hello!' }
  ]
});

console.log(response.choices[0].message.content);
```

---

## 🏗️ 架构设计

### 项目结构

```
vertex-oai/
├── src/
│   ├── main.rs           # 应用入口和进程管理
│   ├── state.rs          # 应用状态管理
│   ├── routes.rs         # 路由配置
│   ├── handlers/         # 请求处理器
│   │   └── mod.rs        # 聊天补全和模型列表处理
│   ├── models/           # 数据模型
│   │   └── mod.rs        # OpenAI 和 Vertex AI 模型定义
│   └── gcp/              # GCP 集成
│       └── mod.rs        # 令牌管理
├── Cargo.toml            # 项目依赖
├── .env.example          # 环境变量示例
├── CLI.md                # 命令行使用指南
├── ENV.md                # 环境变量配置指南
├── WINDOWS.md            # Windows 平台指南
└── README.md             # 本文件
```

### 核心组件

#### 1. 令牌管理器 (`TokenManager`)

- 自动获取和刷新 GCP 访问令牌
- 使用 `google-cloud-auth` 库处理认证
- 支持多种认证方式(ADC、服务账号等)

#### 2. 模型缓存

- 使用 Moka 实现内存缓存
- 缓存时间: 1 小时
- 自动过期和刷新

#### 3. 请求转发

- 透传所有请求头和响应头
- 支持流式和非流式响应
- 自动处理区域路由(Gemini 3.x 使用 global 端点)

#### 4. 错误处理

- 详细的错误日志
- 优雅的错误响应
- 网络超时保护(30秒)

---

## ⚙️ 配置说明

### 环境变量

| 变量名 | 必填 | 默认值 | 说明 |
|--------|------|--------|------|
| `GCP_PROJECT_ID` | ✅ | - | GCP 项目 ID |
| `GCP_LOCATION` | ❌ | `global` | Vertex AI 区域 |
| `PORT` | ❌ | `8087` | 服务监听端口 |

### 编译优化

`Cargo.toml` 中的 Release 配置:

```toml
[profile.release]
opt-level = 3          # 最高优化级别
lto = true             # 启用链接时优化
codegen-units = 1      # 更好的优化
strip = true           # 移除调试符号
panic = "abort"        # 减小二进制大小
```

---

## 🔧 开发指南

### 本地开发

```bash
# 安装依赖
cargo build

# 运行开发版本
cargo run

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 代码检查
cargo clippy
```

### 日志级别

使用 `RUST_LOG` 环境变量控制日志级别:

```bash
# 调试模式
RUST_LOG=debug cargo run

# 仅显示错误
RUST_LOG=error cargo run

# 详细追踪
RUST_LOG=trace cargo run
```

---

## 📊 性能优化

### 已实现的优化

- ✅ 使用 `rustls` 替代 OpenSSL
- ✅ 启用 LTO(链接时优化)
- ✅ 移除调试符号
- ✅ 异步 I/O 处理
- ✅ 模型列表缓存
- ✅ HTTP 连接复用

### 性能指标

- **启动时间**: < 100ms
- **内存占用**: ~10MB(空闲)
- **响应延迟**: < 5ms(不含 Vertex AI 延迟)
- **并发支持**: 数千并发连接

---

## 🐛 故障排查

### 常见问题

#### 1. 认证失败

```
Error: Failed to get authorization token
```

**解决方案**:
- 确保已运行 `gcloud auth application-default login`
- 或设置 `GOOGLE_APPLICATION_CREDENTIALS` 环境变量
- 检查服务账号是否有 Vertex AI 权限

#### 2. 模型不可用

```
Error: 404 NOT_FOUND
```

**解决方案**:
- 确认模型名称正确(如 `gemini-2.0-flash-exp`)
- 检查 GCP 项目是否启用了 Vertex AI API
- 验证区域设置是否正确

#### 3. 连接超时

```
Error: Connection timeout
```

**解决方案**:
- 检查网络连接
- 验证防火墙设置
- 确认 GCP 服务状态

---

## 📚 完整文档

- **[CLI.md](CLI.md)** - 命令行进程控制详细指南
- **[ENV.md](ENV.md)** - 环境变量配置完整说明
- **[DAEMON.md](DAEMON.md)** - 守护进程使用指南(已弃用,推荐使用 CLI)
- **[WINDOWS.md](WINDOWS.md)** - Windows 平台部署指南

---

## 🤝 贡献指南

我们欢迎所有形式的贡献!

### 如何贡献

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### 代码规范

- 遵循 Rust 官方代码风格
- 运行 `cargo fmt` 格式化代码
- 运行 `cargo clippy` 检查代码质量
- 添加必要的测试和文档

---

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

---

## 🙏 致谢

- [Axum](https://github.com/tokio-rs/axum) - 优秀的 Web 框架
- [Tokio](https://tokio.rs/) - 异步运行时
- [google-cloud-auth](https://github.com/yoshidan/google-cloud-rust) - GCP 认证库
- [Moka](https://github.com/moka-rs/moka) - 高性能缓存库

---

## 📞 联系方式

- **作者**: zhangyue
- **项目主页**: [GitHub](https://github.com/Gallentboy/vertex-oai)
- **问题反馈**: [Issues](https://github.com/Gallentboy/vertex-oai/issues)

---

<div align="center">

**如果这个项目对您有帮助,请给一个 ⭐️ Star!**

Made with ❤️ by zhangyue

</div>
