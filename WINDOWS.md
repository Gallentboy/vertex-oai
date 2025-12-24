# Windows 平台使用指南

由于 Vertex-OAI 使用的守护进程功能基于 Unix 系统调用,Windows 平台不支持 `--daemon` 参数。

## ⚠️ 限制说明

在 Windows 上运行 `vertex-oai.exe --daemon` 会显示错误提示并退出。

## ✅ Windows 替代方案

### 方案 1: 前台运行(开发测试)

```powershell
# 设置环境变量
$env:GCP_PROJECT_ID="your-project-id"
$env:PORT="8087"

# 运行
.\vertex-oai.exe
```

### 方案 2: 使用 NSSM (推荐)

**NSSM** (Non-Sucking Service Manager) 是最简单的 Windows 服务管理工具。

#### 安装步骤:

1. **下载 NSSM**
   ```powershell
   # 使用 Chocolatey
   choco install nssm
   
   # 或从官网下载: https://nssm.cc/download
   ```

2. **安装服务**
   ```powershell
   # 以管理员身份运行
   nssm install vertex-oai "C:\path\to\vertex-oai.exe"
   
   # 设置环境变量
   nssm set vertex-oai AppEnvironmentExtra GCP_PROJECT_ID=your-project-id PORT=8087
   
   # 设置工作目录
   nssm set vertex-oai AppDirectory "C:\path\to\vertex-oai"
   
   # 设置日志
   nssm set vertex-oai AppStdout "C:\path\to\vertex-oai\logs\stdout.log"
   nssm set vertex-oai AppStderr "C:\path\to\vertex-oai\logs\stderr.log"
   
   # 启动服务
   nssm start vertex-oai
   ```

3. **管理服务**
   ```powershell
   # 查看状态
   nssm status vertex-oai
   
   # 停止服务
   nssm stop vertex-oai
   
   # 重启服务
   nssm restart vertex-oai
   
   # 卸载服务
   nssm remove vertex-oai confirm
   ```

### 方案 3: 使用 WinSW

**WinSW** (Windows Service Wrapper) 是另一个流行的服务包装工具。

#### 配置步骤:

1. **下载 WinSW**
   - 从 https://github.com/winsw/winsw/releases 下载
   - 重命名为 `vertex-oai-service.exe`

2. **创建配置文件** `vertex-oai-service.xml`:
   ```xml
   <service>
     <id>vertex-oai</id>
     <name>Vertex AI OpenAI Gateway</name>
     <description>OpenAI compatible gateway for Google Vertex AI</description>
     
     <executable>C:\path\to\vertex-oai.exe</executable>
     <workingdirectory>C:\path\to\vertex-oai</workingdirectory>
     
     <env name="GCP_PROJECT_ID" value="your-project-id"/>
     <env name="PORT" value="8087"/>
     <env name="RUST_LOG" value="info"/>
     
     <log mode="roll-by-size">
       <sizeThreshold>10240</sizeThreshold>
       <keepFiles>8</keepFiles>
     </log>
     
     <onfailure action="restart" delay="10 sec"/>
   </service>
   ```

3. **安装和管理**
   ```powershell
   # 安装服务
   .\vertex-oai-service.exe install
   
   # 启动服务
   .\vertex-oai-service.exe start
   
   # 停止服务
   .\vertex-oai-service.exe stop
   
   # 卸载服务
   .\vertex-oai-service.exe uninstall
   ```

### 方案 4: PowerShell 后台运行

简单的后台运行(不推荐用于生产):

```powershell
# 隐藏窗口运行
Start-Process -FilePath ".\vertex-oai.exe" -WindowStyle Hidden

# 或使用 Start-Job
Start-Job -ScriptBlock {
    $env:GCP_PROJECT_ID="your-project-id"
    $env:PORT="8087"
    & "C:\path\to\vertex-oai.exe"
}
```

### 方案 5: 任务计划程序

使用 Windows 任务计划程序在系统启动时自动运行:

1. 打开"任务计划程序"
2. 创建基本任务
3. 触发器: "计算机启动时"
4. 操作: 启动程序 `C:\path\to\vertex-oai.exe`
5. 配置环境变量(在批处理脚本中设置)

## 📝 推荐配置

### 使用 NSSM 的完整示例

创建 `install-service.ps1`:

```powershell
# 需要管理员权限运行

$SERVICE_NAME = "vertex-oai"
$INSTALL_DIR = "C:\Program Files\vertex-oai"
$BINARY_PATH = "$INSTALL_DIR\vertex-oai.exe"

# 设置环境变量
$GCP_PROJECT_ID = Read-Host "请输入 GCP_PROJECT_ID"
$PORT = Read-Host "请输入端口 (默认 8087)" 
if ([string]::IsNullOrWhiteSpace($PORT)) { $PORT = "8087" }

# 创建日志目录
New-Item -ItemType Directory -Force -Path "$INSTALL_DIR\logs"

# 安装服务
nssm install $SERVICE_NAME $BINARY_PATH

# 配置服务
nssm set $SERVICE_NAME AppDirectory $INSTALL_DIR
nssm set $SERVICE_NAME AppEnvironmentExtra "GCP_PROJECT_ID=$GCP_PROJECT_ID" "PORT=$PORT"
nssm set $SERVICE_NAME AppStdout "$INSTALL_DIR\logs\stdout.log"
nssm set $SERVICE_NAME AppStderr "$INSTALL_DIR\logs\stderr.log"
nssm set $SERVICE_NAME AppRotateFiles 1
nssm set $SERVICE_NAME AppRotateBytes 10485760  # 10MB

# 设置启动类型
nssm set $SERVICE_NAME Start SERVICE_AUTO_START

# 启动服务
nssm start $SERVICE_NAME

Write-Host "服务安装完成!" -ForegroundColor Green
Write-Host "服务名称: $SERVICE_NAME"
Write-Host "日志目录: $INSTALL_DIR\logs"
```

## 🔍 故障排查

### 查看服务日志

```powershell
# NSSM 日志
Get-Content "C:\path\to\vertex-oai\logs\stdout.log" -Tail 50 -Wait

# Windows 事件查看器
Get-EventLog -LogName Application -Source "vertex-oai" -Newest 20
```

### 检查服务状态

```powershell
# 使用 NSSM
nssm status vertex-oai

# 使用 Windows 服务管理器
Get-Service vertex-oai
```

## 📚 更多资源

- **NSSM 官网**: https://nssm.cc/
- **WinSW GitHub**: https://github.com/winsw/winsw
- **Windows 服务文档**: https://docs.microsoft.com/windows/win32/services/

---

## 💡 总结

对于 Windows 用户,我们推荐:
- **开发测试**: 直接前台运行
- **生产环境**: 使用 NSSM 注册为 Windows 服务

这样可以获得与 Unix 守护进程类似的功能,包括自动启动、日志管理和服务监控。
