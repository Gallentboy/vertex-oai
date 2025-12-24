# 守护进程使用指南

Vertex-OAI 支持以守护进程(daemon)模式在后台运行,类似于 Redis、Nginx 等服务。

## 🚀 快速开始

### 前台运行(开发模式)

```bash
./vertex-oai
```

### 后台运行(守护进程模式)

```bash
./vertex-oai --daemon
```

## 📋 命令行参数

| 参数 | 短参数 | 默认值 | 说明 |
|------|--------|--------|------|
| `--daemon` | `-d` | - | 以守护进程模式运行 |
| `--pid-file` | - | `/tmp/vertex-oai.pid` | PID 文件路径 |
| `--log-file` | - | `./logs/vertex-oai.log` | 日志文件路径 |
| `--working-dir` | - | `.` | 工作目录 |

### 使用示例

```bash
# 使用默认配置启动守护进程
./vertex-oai --daemon

# 自定义 PID 文件和日志文件
./vertex-oai --daemon \
  --pid-file /var/run/vertex-oai.pid \
  --log-file /var/log/vertex-oai/app.log

# 指定工作目录
./vertex-oai --daemon \
  --working-dir /opt/vertex-oai \
  --pid-file /opt/vertex-oai/vertex-oai.pid \
  --log-file /opt/vertex-oai/logs/app.log
```

## 🔧 进程管理

### 启动服务

```bash
# 设置环境变量
export GCP_PROJECT_ID="your-project-id"
export PORT="8087"

# 启动守护进程
./vertex-oai --daemon
```

### 停止服务

```bash
# 方法 1: 使用 PID 文件
kill $(cat /tmp/vertex-oai.pid)

# 方法 2: 查找进程并停止
pkill -f vertex-oai

# 方法 3: 优雅关闭
kill -TERM $(cat /tmp/vertex-oai.pid)
```

### 重启服务

```bash
# 停止
kill $(cat /tmp/vertex-oai.pid)

# 等待进程结束
sleep 2

# 启动
./vertex-oai --daemon
```

### 查看状态

```bash
# 检查进程是否运行
ps aux | grep vertex-oai

# 或使用 PID 文件
if [ -f /tmp/vertex-oai.pid ]; then
    PID=$(cat /tmp/vertex-oai.pid)
    if ps -p $PID > /dev/null; then
        echo "服务正在运行 (PID: $PID)"
    else
        echo "服务未运行"
    fi
fi
```

### 查看日志

```bash
# 实时查看日志
tail -f ./logs/vertex-oai.log

# 查看最近 100 行
tail -n 100 ./logs/vertex-oai.log

# 搜索错误日志
grep ERROR ./logs/vertex-oai.log
```

## 📝 使用 service.sh 脚本

项目提供了 `service.sh` 脚本来简化守护进程管理:

```bash
# 启动服务(守护进程模式)
./service.sh start

# 停止服务
./service.sh stop

# 重启服务
./service.sh restart

# 查看状态
./service.sh status

# 查看实时日志
./service.sh logs
```

## 🔐 生产环境建议

### 1. 使用专用用户运行

```bash
# 创建专用用户
sudo useradd -r -s /bin/false vertex-oai

# 创建工作目录
sudo mkdir -p /opt/vertex-oai/logs
sudo chown -R vertex-oai:vertex-oai /opt/vertex-oai

# 以专用用户运行
sudo -u vertex-oai ./vertex-oai --daemon \
  --working-dir /opt/vertex-oai \
  --pid-file /opt/vertex-oai/vertex-oai.pid \
  --log-file /opt/vertex-oai/logs/app.log
```

### 2. 配置日志轮转

创建 `/etc/logrotate.d/vertex-oai`:

```
/opt/vertex-oai/logs/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0640 vertex-oai vertex-oai
    sharedscripts
    postrotate
        kill -USR1 $(cat /opt/vertex-oai/vertex-oai.pid) 2>/dev/null || true
    endscript
}
```

### 3. 设置文件权限

```bash
# 限制 PID 文件权限
chmod 644 /opt/vertex-oai/vertex-oai.pid

# 限制日志文件权限
chmod 640 /opt/vertex-oai/logs/*.log

# 限制二进制文件权限
chmod 755 /opt/vertex-oai/vertex-oai
```

## 🐛 故障排查

### 守护进程启动失败

**检查日志文件:**
```bash
cat ./logs/vertex-oai.log
```

**常见问题:**

1. **权限不足**
   ```bash
   # 确保有写入权限
   chmod 755 ./logs
   ```

2. **PID 文件已存在**
   ```bash
   # 删除旧的 PID 文件
   rm -f /tmp/vertex-oai.pid
   ```

3. **端口被占用**
   ```bash
   # 检查端口占用
   lsof -i :8087
   
   # 或使用其他端口
   export PORT=8088
   ./vertex-oai --daemon
   ```

### 进程意外退出

查看日志文件了解退出原因:
```bash
tail -n 50 ./logs/vertex-oai.log
```

## 🔄 与 systemd 集成

如果您使用 systemd,可以创建更简单的服务文件:

`/etc/systemd/system/vertex-oai.service`:
```ini
[Unit]
Description=Vertex AI OpenAI Gateway
After=network.target

[Service]
Type=forking
PIDFile=/opt/vertex-oai/vertex-oai.pid
ExecStart=/opt/vertex-oai/vertex-oai --daemon \
  --pid-file /opt/vertex-oai/vertex-oai.pid \
  --log-file /opt/vertex-oai/logs/app.log \
  --working-dir /opt/vertex-oai
Environment="GCP_PROJECT_ID=your-project-id"
Environment="PORT=8087"
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

然后使用 systemd 管理:
```bash
sudo systemctl daemon-reload
sudo systemctl enable vertex-oai
sudo systemctl start vertex-oai
sudo systemctl status vertex-oai
```

## 📊 监控建议

### 健康检查

```bash
# 定期检查服务是否响应
curl -f http://localhost:8087/ || echo "Service is down!"
```

### 进程监控

使用 `monit` 或 `supervisor` 等工具自动重启失败的进程。

**Monit 配置示例:**
```
check process vertex-oai with pidfile /opt/vertex-oai/vertex-oai.pid
    start program = "/opt/vertex-oai/vertex-oai --daemon"
    stop program = "/bin/kill -TERM $(cat /opt/vertex-oai/vertex-oai.pid)"
    if failed host 127.0.0.1 port 8087 protocol http
        request "/"
        with timeout 10 seconds
        then restart
```

---

## 📞 获取帮助

查看所有可用参数:
```bash
./vertex-oai --help
```

查看版本信息:
```bash
./vertex-oai --version
```
