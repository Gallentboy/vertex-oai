# GitHub Actions 工作流说明

本项目包含两个 GitHub Actions 工作流:

## 🔄 CI 工作流 (ci.yml)

**触发条件:**
- Push 到 `main` 或 `develop` 分支
- Pull Request 到 `main` 或 `develop` 分支

**任务:**
1. **测试** - 运行所有单元测试
2. **代码检查** - 运行 `cargo fmt` 和 `cargo clippy`
3. **构建验证** - 在 Linux、macOS、Windows 上构建

---

## 🚀 Release 工作流 (release.yml)

**触发条件:**
- 推送 tag (格式: `v*`,如 `v0.1.0`)
- 手动触发 (workflow_dispatch)

**支持平台:**

| 平台 | 架构 | 文件名 |
|------|------|--------|
| Linux | x86_64 | `vertex-oai-linux-amd64.tar.gz` |
| Linux | ARM64 | `vertex-oai-linux-arm64.tar.gz` |
| macOS | x86_64 | `vertex-oai-macos-amd64.tar.gz` |
| macOS | ARM64 | `vertex-oai-macos-arm64.tar.gz` |
| Windows | x86_64 | `vertex-oai-windows-amd64.exe.zip` |
| Windows | ARM64 | `vertex-oai-windows-arm64.exe.zip` |

**构建产物:**
- 压缩包 (`.tar.gz` 或 `.zip`)
- SHA256 校验文件

---

## 📦 发布新版本

### 1. 更新版本号

编辑 `Cargo.toml`:
```toml
[package]
version = "0.1.0"  # 更新版本号
```

### 2. 提交更改

```bash
git add Cargo.toml
git commit -m "chore: bump version to 0.1.0"
git push
```

### 3. 创建并推送 tag

```bash
# 创建 tag
git tag -a v0.1.0 -m "Release v0.1.0"

# 推送 tag
git push origin v0.1.0
```

### 4. 等待构建完成

GitHub Actions 会自动:
1. 为所有平台构建二进制文件
2. 创建 GitHub Release
3. 上传所有构建产物

### 5. 编辑 Release 说明

访问 GitHub Releases 页面,编辑自动生成的 Release 说明。

---

## 🔧 手动触发构建

1. 访问 GitHub Actions 页面
2. 选择 "Release" 工作流
3. 点击 "Run workflow"
4. 选择分支
5. 点击 "Run workflow" 按钮

---

## 📥 下载构建产物

### 从 GitHub Releases

```bash
# Linux AMD64
wget https://github.com/yourusername/vertex-oai/releases/download/v0.1.0/vertex-oai-linux-amd64.tar.gz

# macOS ARM64 (Apple Silicon)
wget https://github.com/yourusername/vertex-oai/releases/download/v0.1.0/vertex-oai-macos-arm64.tar.gz

# Windows AMD64
wget https://github.com/yourusername/vertex-oai/releases/download/v0.1.0/vertex-oai-windows-amd64.exe.zip
```

### 验证校验和

```bash
# 下载校验文件
wget https://github.com/yourusername/vertex-oai/releases/download/v0.1.0/vertex-oai-linux-amd64.tar.gz.sha256

# 验证
sha256sum -c vertex-oai-linux-amd64.tar.gz.sha256
```

---

## 🐛 故障排查

### 构建失败

1. 检查 GitHub Actions 日志
2. 确保 `Cargo.toml` 配置正确
3. 验证所有依赖都支持目标平台

### 交叉编译问题

**Linux ARM64:**
- 工作流会自动安装 `gcc-aarch64-linux-gnu`
- 如果失败,检查 Ubuntu 软件源

**Windows ARM64:**
- 需要 Rust 1.64+ 支持
- 某些依赖可能不支持 ARM64

### Release 创建失败

确保:
1. Tag 格式正确 (`v*`)
2. `GITHUB_TOKEN` 有足够权限
3. Repository 设置允许创建 Release

---

## 💡 优化建议

### 加速构建

1. **使用缓存** - 已启用 cargo 缓存
2. **并行构建** - 使用 matrix 策略
3. **增量编译** - 缓存 target 目录

### 减小二进制大小

已在 `Cargo.toml` 中配置:
```toml
[profile.release]
opt-level = 3
lto = true
strip = true
```

### 添加更多平台

编辑 `.github/workflows/release.yml`,在 `matrix.include` 中添加:

```yaml
# FreeBSD x86_64
- os: ubuntu-latest
  target: x86_64-unknown-freebsd
  artifact_name: vertex-oai
  asset_name: vertex-oai-freebsd-amd64
```

---

## 📊 构建状态

在 README.md 中添加徽章:

```markdown
[![CI](https://github.com/yourusername/vertex-oai/actions/workflows/ci.yml/badge.svg)](https://github.com/yourusername/vertex-oai/actions/workflows/ci.yml)
[![Release](https://github.com/yourusername/vertex-oai/actions/workflows/release.yml/badge.svg)](https://github.com/yourusername/vertex-oai/actions/workflows/release.yml)
```
