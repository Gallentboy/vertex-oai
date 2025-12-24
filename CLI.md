# 命令行进程控制使用指南

Vertex-OAI 现在支持类似 Redis/Nginx 的命令行进程控制方式!

## 🚀 快速开始

### 启动服务(守护进程)
```bash
./vertex-oai start
```

### 停止服务
```bash
./vertex-oai stop
```

### 重启服务
```bash
./vertex-oai restart
```

### 查看状态
```bash
./vertex-oai status
```

### 前台运行(开发模式)
```bash
./vertex-oai
```

---

## 📖 详细说明

### `start` - 启动守护进程

以守护进程模式启动服务:

```bash
$ ./vertex-oai start
正在启动 vertex-oai 守护进程...
```

**特点:**
- 自动后台运行
- 自动创建 PID 文件(`.pid`)
- 日志输出到文件(`./logs/vertex-oai.log`)
- 如果服务已运行,会提示错误

**环境变量:**
```bash
export GCP_PROJECT_ID="your-project-id"
export PORT="8087"
./vertex-oai start
```

---

### `stop` - 停止服务

优雅停止正在运行的服务:

```bash
$ ./vertex-oai stop
正在停止服务 (PID: 12345)...
✓ 服务已停止
```

**工作流程:**
1. 读取 PID 文件
2. 发送 SIGTERM 信号(优雅关闭)
3. 等待最多 5 秒
4. 如果未停止,发送 SIGKILL 强制终止

---

### `restart` - 重启服务

停止并重新启动服务:

```bash
$ ./vertex-oai restart
正在重启 vertex-oai...
正在停止服务 (PID: 12345)...
✓ 服务已停止
正在启动 vertex-oai 守护进程...
```

**使用场景:**
- 更新配置后重启
- 更新二进制文件后重启
- 服务异常需要重启

---

### `status` - 查看状态

查看服务运行状态:

```bash
$ ./vertex-oai status
✓ 服务正在运行
  PID:      12345
  PID 文件: /path/to/.pid
  日志文件: ./logs/vertex-oai.log
```

或者服务未运行时:

```bash
$ ./vertex-oai status
✗ 服务未运行
```

---

### 前台运行(无子命令)

直接运行,不加任何子命令:

```bash
$ ./vertex-oai
========================================
Server listening on: http://0.0.0.0:8087
Daemon mode: false
========================================
```

**特点:**
- 日志输出到控制台
- 支持 Ctrl+C 停止
- 适合开发和调试

**自定义参数:**
```bash
./vertex-oai --log-file /custom/path.log --working-dir /opt/app
```

---

## 🔧 高级用法

### 自定义日志文件

```bash
./vertex-oai start --log-file /var/log/vertex-oai/app.log
```

### 自定义工作目录

```bash
./vertex-oai start --working-dir /opt/vertex-oai
```

### 组合使用

```bash
./vertex-oai start \
  --log-file /var/log/vertex-oai.log \
  --working-dir /opt/vertex-oai
```

---

## 📝 完整示例

### 生产环境部署

```bash
# 1. 设置环境变量
export GCP_PROJECT_ID="my-gcp-project"
export PORT="8087"

# 2. 启动服务
./vertex-oai start

# 3. 检查状态
./vertex-oai status

# 4. 查看日志
tail -f ./logs/vertex-oai.log

# 5. 测试服务
curl http://localhost:8087/

# 6. 需要时重启
./vertex-oai restart

# 7. 停止服务
./vertex-oai stop
```

### 开发环境

```bash
# 前台运行,方便调试
export GCP_PROJECT_ID="dev-project"
./vertex-oai
```

---

## 🐛 故障排查

### 服务无法启动

```bash
$ ./vertex-oai start
✗ 服务已经在运行中 (PID: 12345)
```

**解决方案:**
```bash
# 先停止现有服务
./vertex-oai stop

# 或者检查是否是残留的 PID 文件
./vertex-oai status
# 如果显示未运行但有 PID 文件,手动删除
rm .pid
```

### 服务无法停止

```bash
$ ./vertex-oai stop
⚠ 服务未在预期时间内停止,强制终止...
```

这是正常的,程序会自动强制终止。

### PID 文件位置

PID 文件始终位于二进制文件同级目录:
- 开发: `./target/release/.pid`
- 生产: `/opt/vertex-oai/.pid`

---

## 🆚 对比

### vs service.sh 脚本

| 特性 | 命令行控制 | service.sh |
|------|-----------|-----------|
| 启动 | `./vertex-oai start` | `./service.sh start` |
| 停止 | `./vertex-oai stop` | `./service.sh stop` |
| 重启 | `./vertex-oai restart` | `./service.sh restart` |
| 状态 | `./vertex-oai status` | `./service.sh status` |
| 依赖 | 无需脚本 | 需要 bash |
| 跨平台 | Unix only | Unix only |

**推荐:** 使用命令行控制更简洁,无需额外脚本!

---

## 💡 提示

1. **PID 文件自动管理** - 无需手动指定,自动在二进制同级目录创建
2. **优雅关闭** - stop 命令会等待现有请求完成
3. **状态检查** - 使用 `status` 命令快速检查服务状态
4. **日志查看** - 守护进程日志默认在 `./logs/vertex-oai.log`
5. **环境变量** - 记得设置 `GCP_PROJECT_ID` 环境变量

---

## 🔗 相关文档

- [DAEMON.md](DAEMON.md) - 守护进程详细指南
- [README.md](README.md) - 项目总览
- [WINDOWS.md](WINDOWS.md) - Windows 平台指南
