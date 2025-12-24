mod gcp;
mod handlers;
mod models;
mod routes;
mod state;

use clap::Parser;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::fmt::writer::MakeWriterExt;

#[cfg(unix)]
use daemonize::Daemonize;

#[cfg(unix)]
use std::process::exit;

use crate::routes::create_routes;
use crate::state::AppState;

// ============= Unix 平台 =============
#[cfg(unix)]
/// Vertex AI OpenAI Compatible Gateway
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    /// 日志文件路径(守护进程模式)
    #[arg(long, default_value = "./logs/vertex-oai.log")]
    log_file: PathBuf,

    /// 工作目录
    #[arg(long, default_value = ".")]
    working_dir: PathBuf,
}

#[cfg(unix)]
#[derive(Parser, Debug, Clone)]
enum Command {
    /// 启动服务(守护进程模式)
    Start,
    /// 停止服务
    Stop,
    /// 重启服务
    Restart,
    /// 查看服务状态
    Status,
}

#[cfg(unix)]
impl Args {
    /// 获取 PID 文件路径(二进制文件同级目录下的 .pid)
    fn pid_file(&self) -> PathBuf {
        let exe_path = std::env::current_exe()
            .map_err(|e| {
                eprintln!("无法获取二进制文件路径: {}", e);
                exit(1);
            })
            .unwrap();

        let exe_dir = exe_path.parent().unwrap_or_else(|| {
            eprintln!("无法获取{}上级路径", exe_path.display());
            exit(1);
        });
        exe_dir.join(".pid")
    }

    /// 读取 PID 文件
    fn read_pid(&self) -> Option<u32> {
        let pid_file = self.pid_file();
        std::fs::read_to_string(&pid_file)
            .ok()
            .and_then(|s| s.trim().parse().ok())
    }

    /// 检查进程是否运行
    fn is_running(&self) -> bool {
        if let Some(pid) = self.read_pid() {
            // 使用 kill(pid, 0) 检查进程是否存在
            unsafe { libc::kill(pid as i32, 0) == 0 }
        } else {
            false
        }
    }
}

// ============= 非 Unix 平台 =============
#[cfg(not(unix))]
/// Vertex AI OpenAI Compatible Gateway
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 日志文件路径
    #[arg(long, default_value = "./logs/vertex-oai.log")]
    log_file: PathBuf,

    /// 工作目录
    #[arg(long, default_value = ".")]
    working_dir: PathBuf,
}

#[cfg(not(unix))]
impl Args {
    /// 获取 PID 文件路径(非 Unix 平台不使用,但为了兼容性提供)
    fn pid_file(&self) -> PathBuf {
        PathBuf::from(".pid")
    }
}

// ============= Unix 平台 main 函数 =============
#[cfg(unix)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 加载 .env 文件(如果存在)
    load_env();

    match args.command {
        Some(Command::Start) => start_daemon(args),
        Some(Command::Stop) => stop_daemon(args),
        Some(Command::Restart) => restart_daemon(args),
        Some(Command::Status) => show_status(args),
        None => {
            // 无子命令时,前台运行
            run_foreground(args)
        }
    }
}

// ============= 非 Unix 平台 main 函数 =============
#[cfg(not(unix))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // 加载 .env 文件(如果存在)
    load_env();

    eprintln!("╔════════════════════════════════════════════════════════════════╗");
    eprintln!("║  ℹ️  进程管理功能仅在 Unix/Linux/macOS 系统上可用            ║");
    eprintln!("╚════════════════════════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("当前系统不支持 start/stop/restart/status 命令。");
    eprintln!();
    eprintln!("Windows 用户请参考 WINDOWS.md 文档:");
    eprintln!("  - 使用 NSSM 注册为 Windows 服务");
    eprintln!("  - 使用 WinSW 包装为服务");
    eprintln!();
    eprintln!("正在以前台模式运行...");
    eprintln!();

    // 前台运行
    run_foreground(args)
}

// ============= 通用函数 =============
/// 加载 .env 文件
fn load_env() {
    match dotenvy::dotenv() {
        Ok(path) => {
            eprintln!("✓ 已加载环境变量文件: {}", path.display());
        }
        Err(dotenvy::Error::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => {
            // .env 文件不存在,这是正常的
        }
        Err(e) => {
            eprintln!("⚠ 加载 .env 文件时出错: {}", e);
        }
    }
}

// ============= Unix 平台进程管理函数 =============
#[cfg(unix)]
/// 启动守护进程
fn start_daemon(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_running() {
        eprintln!("✗ 服务已经在运行中 (PID: {})", args.read_pid().unwrap());
        exit(1);
    }

    println!("正在启动 vertex-oai 守护进程...");

    // 关键:在创建 Tokio 运行时之前先 daemonize
    daemonize_process(&args)?;

    // 初始化日志(在 daemonize 之后)
    init_logging(&args, true)?;

    // 现在启动 Tokio 运行时
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async_main(args, true))
}

#[cfg(unix)]
/// 停止守护进程
fn stop_daemon(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    if !args.is_running() {
        eprintln!("✗ 服务未运行");
        exit(1);
    }

    let pid = args.read_pid().unwrap();
    println!("正在停止服务 (PID: {})...", pid);

    // 发送 SIGTERM 信号
    unsafe {
        if libc::kill(pid as i32, libc::SIGTERM) != 0 {
            eprintln!("✗ 发送停止信号失败");
            exit(1);
        }
    }

    // 等待进程结束
    for _ in 1..=50 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if !args.is_running() {
            println!("✓ 服务已停止");
            return Ok(());
        }
    }

    eprintln!("⚠ 服务未在预期时间内停止,强制终止...");
    unsafe {
        libc::kill(pid as i32, libc::SIGKILL);
    }
    std::thread::sleep(std::time::Duration::from_millis(100));
    Ok(())
}

#[cfg(unix)]
/// 重启守护进程
fn restart_daemon(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    println!("正在重启 vertex-oai...");

    if args.is_running() {
        stop_daemon(args.clone())?;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    start_daemon(args)
}

#[cfg(unix)]
/// 显示服务状态
fn show_status(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_running() {
        let pid = args.read_pid().unwrap();
        println!("✓ 服务正在运行");
        println!("  PID:      {}", pid);
        println!("  PID 文件: {}", args.pid_file().display());
        println!("  日志文件: {}", args.log_file.display());
    } else {
        println!("✗ 服务未运行");
        if args.pid_file().exists() {
            println!("  (发现残留的 PID 文件,可能是异常退出)");
        }
    }
    Ok(())
}

// ============= 通用函数 =============
/// 前台运行
fn run_foreground(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    init_logging(&args, false)?;

    // 启动 Tokio 运行时
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(async_main(args, false))
}

async fn async_main(args: Args, daemon: bool) -> Result<(), Box<dyn std::error::Error>> {
    // 创建应用状态
    let state = Arc::new(AppState::new().await?);

    // 构建路由
    let app = create_routes(state);

    // 从环境变量获取端口,默认为 8087
    let port = std::env::var("PORT").unwrap_or_else(|_| "8087".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("========================================");
    tracing::info!("Vertex-OAI v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Build: {}", env!("BUILD_TIME"));
    tracing::info!("Commit: {} ({})", env!("GIT_HASH"), env!("GIT_BRANCH"));
    if !env!("GIT_TAG").is_empty() {
        tracing::info!("Tag: {}", env!("GIT_TAG"));
    }
    tracing::info!("Target: {}", env!("BUILD_TARGET"));
    tracing::info!("Profile: {}", env!("BUILD_PROFILE"));
    tracing::info!("========================================");
    tracing::info!("Server listening on: http://{}", addr);
    tracing::info!("Daemon mode: {}", daemon);
    if daemon {
        tracing::info!("PID file: {}", args.pid_file().display());
        tracing::info!("Log file: {}", args.log_file.display());
    }
    tracing::info!("========================================");

    // 启动服务器并处理优雅关闭
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shutdown complete");

    // 清理 PID 文件
    if daemon {
        let pid_file = args.pid_file();
        if pid_file.exists() {
            let _ = std::fs::remove_file(&pid_file);
            tracing::info!("PID file removed");
        }
    }

    Ok(())
}

/// 等待关闭信号
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal, shutting down gracefully...");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM signal, shutting down gracefully...");
        },
    }
}

/// 守护进程化 (Unix)
#[cfg(unix)]
fn daemonize_process(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let pid_file = args.pid_file();

    // 确保日志目录存在
    if let Some(parent) = args.log_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 确保 PID 文件目录存在
    if let Some(parent) = pid_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let stdout = File::create(&args.log_file)?;
    let stderr = File::create(&args.log_file)?;

    let daemonize = Daemonize::new()
        .pid_file(&pid_file) // PID 文件
        .chown_pid_file(true) // 修改 PID 文件所有者
        .working_directory(&args.working_dir) // 工作目录
        .umask(0o027) // 文件权限掩码
        .stdout(stdout) // 标准输出重定向
        .stderr(stderr) // 标准错误重定向
        .privileged_action(|| "Vertex-OAI daemon started");

    match daemonize.start() {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Failed to daemonize: {}", e);
            Err(Box::new(e))
        }
    }
}

/// 守护进程化 (Windows - 不支持)
#[cfg(not(unix))]
fn daemonize_process(_args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("╔════════════════════════════════════════════════════════════════╗");
    eprintln!("║  ⚠️  Daemon mode is not supported on Windows                  ║");
    eprintln!("╚════════════════════════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("Windows 不支持 Unix 风格的守护进程。");
    eprintln!();
    eprintln!("替代方案:");
    eprintln!("  1. 前台运行: vertex-oai.exe");
    eprintln!("  2. 使用 NSSM: https://nssm.cc/");
    eprintln!("     nssm install vertex-oai \"C:\\path\\to\\vertex-oai.exe\"");
    eprintln!("  3. 使用 WinSW: https://github.com/winsw/winsw");
    eprintln!("  4. 使用任务计划程序在后台运行");
    eprintln!();
    std::process::exit(1);
}

/// 初始化日志系统
fn init_logging(args: &Args, daemon: bool) -> Result<(), Box<dyn std::error::Error>> {
    if daemon {
        // 守护进程模式:日志输出到文件
        let log_file = File::create(&args.log_file)?;
        tracing_subscriber::fmt()
            .with_writer(log_file.and(std::io::stdout))
            .with_ansi(false) // 文件日志不需要颜色
            .init();
    } else {
        // 前台模式:日志输出到控制台
        tracing_subscriber::fmt().with_ansi(true).init();
    }
    Ok(())
}
