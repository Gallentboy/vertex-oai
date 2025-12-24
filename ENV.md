# 环境变量配置指南

Vertex-OAI 支持通过 `.env` 文件配置环境变量,简化部署和配置管理。

## 🚀 快速开始

### 1. 创建 .env 文件

```bash
# 复制示例文件
cp .env.example .env

# 编辑配置
nano .env
```

### 2. 配置环境变量

`.env` 文件示例:

```bash
# GCP 项目配置
GCP_PROJECT_ID=your-gcp-project-id
GCP_LOCATION=global

# 服务端口
PORT=8087

# 日志级别
RUST_LOG=info
```

### 3. 启动服务

```bash
# .env 文件会自动加载
./vertex-oai start

# 或前台运行
./vertex-oai
```

---

## 📋 可用环境变量

### 必填变量

| 变量名 | 说明 | 示例 |
|--------|------|------|
| `GCP_PROJECT_ID` | GCP 项目 ID | `my-gcp-project` |

### 可选变量

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `GCP_LOCATION` | `global` | Vertex AI 区域 |
| `PORT` | `8087` | 服务监听端口 |
| `RUST_LOG` | - | 日志级别 |

---

## 🔧 日志级别配置

`RUST_LOG` 环境变量控制日志输出级别:

```bash
# 仅错误
RUST_LOG=error

# 警告和错误
RUST_LOG=warn

# 信息、警告和错误(推荐)
RUST_LOG=info

# 调试信息
RUST_LOG=debug

# 所有信息(包括追踪)
RUST_LOG=trace
```

### 模块级别日志

```bash
# 仅显示特定模块的调试信息
RUST_LOG=vertex_oai=debug,info

# 多个模块
RUST_LOG=vertex_oai::handlers=debug,vertex_oai::gcp=trace,info
```

---

## 📝 完整配置示例

### 开发环境

`.env`:
```bash
GCP_PROJECT_ID=dev-project-123
GCP_LOCATION=us-central1
PORT=8087
RUST_LOG=debug
```

### 生产环境

`.env`:
```bash
GCP_PROJECT_ID=prod-project-456
GCP_LOCATION=global
PORT=8087
RUST_LOG=info
```

### 测试环境

`.env`:
```bash
GCP_PROJECT_ID=test-project-789
GCP_LOCATION=us-west1
PORT=8088
RUST_LOG=trace
```

---

## 🔐 安全最佳实践

### 1. 不要提交 .env 文件

`.env` 文件已自动添加到 `.gitignore`,确保不会被提交到版本控制:

```bash
# .gitignore
.env
```

### 2. 使用 .env.example

提供一个示例文件供其他开发者参考:

```bash
# .env.example (可以提交)
GCP_PROJECT_ID=your-gcp-project-id
GCP_LOCATION=global
PORT=8087
RUST_LOG=info
```

### 3. 限制文件权限

```bash
# 限制 .env 文件权限
chmod 600 .env
```

### 4. 使用不同的环境文件

```bash
# 开发环境
cp .env.development .env

# 生产环境
cp .env.production .env

# 测试环境
cp .env.test .env
```

---

## 🆚 环境变量优先级

环境变量的优先级(从高到低):

1. **系统环境变量** - 直接在 shell 中设置
2. **.env 文件** - 项目根目录的 .env 文件
3. **默认值** - 代码中的默认值

### 示例

```bash
# .env 文件
PORT=8087

# 系统环境变量会覆盖 .env
export PORT=9000

# 最终使用 9000
./vertex-oai start
```

---

## 💡 使用技巧

### 1. 多环境管理

创建多个环境文件:

```bash
.env.development
.env.staging
.env.production
```

使用脚本切换:

```bash
#!/bin/bash
# switch-env.sh

ENV=${1:-development}
cp .env.$ENV .env
echo "切换到 $ENV 环境"
```

### 2. 验证配置

创建验证脚本:

```bash
#!/bin/bash
# verify-env.sh

if [ ! -f .env ]; then
    echo "错误: .env 文件不存在"
    exit 1
fi

source .env

if [ -z "$GCP_PROJECT_ID" ]; then
    echo "错误: GCP_PROJECT_ID 未设置"
    exit 1
fi

echo "✓ 配置验证通过"
```

### 3. Docker 集成

`Dockerfile`:
```dockerfile
FROM rust:1.83 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/vertex-oai /usr/local/bin/
COPY .env.example /app/.env.example
WORKDIR /app
CMD ["vertex-oai", "start"]
```

`docker-compose.yml`:
```yaml
version: '3.8'
services:
  vertex-oai:
    build: .
    env_file:
      - .env
    ports:
      - "${PORT:-8087}:${PORT:-8087}"
```

---

## 🐛 故障排查

### .env 文件未加载

**症状:**
```
✗ 服务启动失败
Error: 请设置GCP_PROJECT_ID
```

**解决方案:**
1. 确认 .env 文件存在于项目根目录
2. 检查文件权限
3. 验证文件内容格式正确

### 环境变量未生效

**检查加载情况:**

程序启动时会显示:
```
✓ 已加载环境变量文件: /path/to/.env
```

如果没有显示,说明 .env 文件不存在或加载失败。

### 变量值错误

**验证环境变量:**

```bash
# 在程序中打印环境变量
RUST_LOG=debug ./vertex-oai
```

---

## 📖 相关文档

- [README.md](README.md) - 项目总览
- [CLI.md](CLI.md) - 命令行使用指南
- [DAEMON.md](DAEMON.md) - 守护进程指南

---

## 🔗 参考资源

- [dotenvy 文档](https://docs.rs/dotenvy/)
- [环境变量最佳实践](https://12factor.net/config)
