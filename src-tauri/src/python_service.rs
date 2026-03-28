use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::time::timeout;

/// 通过端口停止 aktools 服务
async fn stop_aktools_by_port(port: u16) -> Result<(), String> {
    // 方法1: 尝试通过 lsof 找到进程并杀死 (macOS/Linux)
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("lsof")
            .args(&["-ti", &format!(":{}", port)])
            .output();

        if let Ok(output) = output {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let pid_str = output_str.trim();
            if !pid_str.is_empty() {
                for pid in pid_str.split_whitespace() {
                    let _ = Command::new("kill")
                        .args(&["-9", pid])
                        .status();
                }
                // 等待端口释放
                tokio::time::sleep(Duration::from_millis(500)).await;
                return Ok(());
            }
        }
    }

    // 方法2: 尝试通过 pkill 停止 aktools (通用方法)
    let _ = Command::new("pkill")
        .args(&["-9", "-f", "aktools"])
        .status();

    // 等待端口释放
    tokio::time::sleep(Duration::from_millis(500)).await;

    Ok(())
}

/// Python 服务状态
#[derive(Debug, Clone, serde::Serialize)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,  // 新增：停止中
    Failed(String),
}

/// Python 服务管理器
pub struct PythonService {
    process: Arc<Mutex<Option<Child>>>,
    port: Arc<Mutex<u16>>,
    status: Arc<Mutex<ServiceStatus>>,
}

impl PythonService {
    /// 创建新的服务管理器（传入 0 表示自动选择端口）
    pub fn new(port: u16) -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            port: Arc::new(Mutex::new(port)),
            status: Arc::new(Mutex::new(ServiceStatus::Stopped)),
        }
    }

    /// 查找可用端口
    fn find_available_port(start_port: u16) -> Option<u16> {
        for port in start_port..=65535 {
            if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
                // 成功绑定，端口可用
                drop(listener);
                return Some(port);
            }
        }
        None
    }

    /// 获取当前端口
    pub fn get_port(&self) -> u16 {
        *self.port.lock().unwrap()
    }

    /// 获取当前状态
    pub fn get_status(&self) -> ServiceStatus {
        self.status.lock().unwrap().clone()
    }

    /// 启动 Python 服务
    pub async fn start(&self) -> Result<(), String> {
        // 检查是否已在运行（通过 HTTP）
        if self.is_running().await {
            println!("Service is already running on port {}", self.get_port());
            return Ok(());
        }

        // 更新状态
        *self.status.lock().unwrap() = ServiceStatus::Starting;

        // 首先检查默认端口 18080 是否已有服务在运行（可能是上次遗留的）
        if Self::check_port_in_use(18080).await {
            *self.port.lock().unwrap() = 18080;
            *self.status.lock().unwrap() = ServiceStatus::Running;
            println!("Found existing service on port 18080");
            return Ok(());
        }

        // 查找可用端口
        let current_port = *self.port.lock().unwrap();
        let port = if current_port == 0 || current_port == 18080 {
            // 从 18080 开始找可用端口
            Self::find_available_port(18080)
                .or_else(|| Self::find_available_port(18081))
                .or_else(|| Self::find_available_port(10000))
                .ok_or_else(|| "No available port found".to_string())?
        } else {
            // 检查当前端口是否可用
            if TcpListener::bind(format!("127.0.0.1:{}", current_port)).is_ok() {
                current_port
            } else {
                // 端口被占用，寻找新端口
                Self::find_available_port(current_port + 1)
                    .or_else(|| Self::find_available_port(18080))
                    .ok_or_else(|| "No available port found".to_string())?
            }
        };

        // 再次检查目标端口是否有服务在运行（可能有其他应用启动了服务）
        if Self::check_port_in_use(port).await {
            *self.port.lock().unwrap() = port;
            *self.status.lock().unwrap() = ServiceStatus::Running;
            println!("Found existing service on port {}", port);
            return Ok(());
        }

        // 更新端口
        *self.port.lock().unwrap() = port;
        println!("Starting service on port: {}", port);

        // 查找 aktools 可执行文件
        let executable = self.find_aktools_executable()?;

        // 启动进程（清除代理环境变量，避免影响东方财富 API 连接）
        let port = *self.port.lock().unwrap();
        let child = Command::new(&executable)
            .arg(format!("--port={}", port))
            .arg("--host=127.0.0.1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .env_remove("http_proxy")
            .env_remove("HTTP_PROXY")
            .env_remove("https_proxy")
            .env_remove("HTTPS_PROXY")
            .env_remove("all_proxy")
            .env_remove("ALL_PROXY")
            .env("NO_PROXY", "*")  // 禁用所有代理
            .env("no_proxy", "*")
            .spawn()
            .map_err(|e| format!("Failed to start AKTools: {}", e))?;

        *self.process.lock().unwrap() = Some(child);

        // 等待服务就绪（延长到60秒，aktools首次启动可能较慢）
        match self.wait_for_service(Duration::from_secs(60)).await {
            Ok(_) => {
                *self.status.lock().unwrap() = ServiceStatus::Running;
                Ok(())
            }
            Err(e) => {
                self.stop().await.ok();
                *self.status.lock().unwrap() = ServiceStatus::Failed(e.clone());
                Err(e)
            }
        }
    }

    /// 停止 Python 服务
    pub async fn stop(&self) -> Result<(), String> {
        // 设置停止中状态
        *self.status.lock().unwrap() = ServiceStatus::Stopping;

        // 先检查是否有保存的进程句柄
        let has_process = {
            let mut process = self.process.lock().unwrap();
            if let Some(mut child) = process.take() {
                // 尝试优雅终止
                let _ = child.kill();
                let _ = child.wait();
                true
            } else {
                false
            }
        };

        // 如果没有保存的进程句柄（可能是检测到已存在的服务）
        // 尝试通过系统命令停止 aktools 进程
        if !has_process {
            let port = *self.port.lock().unwrap();
            stop_aktools_by_port(port).await?;
        }

        // 等待服务完全停止（最多5秒）
        let port = *self.port.lock().unwrap();
        for _ in 0..50 {
            if !Self::check_port_in_use(port).await {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        *self.status.lock().unwrap() = ServiceStatus::Stopped;
        Ok(())
    }

    /// 查找 aktools 可执行文件
    fn find_aktools_executable(&self) -> Result<String, String> {
        // 1. 首先查找 Tauri sidecar 路径 (externalBin)
        if let Ok(exe_path) = std::env::current_exe() {
            let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));

            // Tauri sidecar 打包后的路径
            let sidecar_candidates = [
                // macOS: .app/Contents/MacOS/aktools
                exe_dir.join("aktools"),
                // Windows: 与 exe 同级
                exe_dir.join("aktools.exe"),
                // 开发时手动放置的路径
                exe_dir.join("..").join("Resources").join("bin").join("aktools"),
                exe_dir.join("bin").join("aktools"),
                exe_dir.join("bin").join("aktools.exe"),
            ];

            for candidate in &sidecar_candidates {
                if candidate.exists() {
                    return Ok(candidate.to_string_lossy().to_string());
                }
            }

            // 检查 bin/ 目录下的平台特定子目录
            let platform_dir = if cfg!(target_os = "macos") {
                "macos"
            } else if cfg!(target_os = "windows") {
                "windows"
            } else {
                "linux"
            };

            let platform_paths = [
                exe_dir.join("bin").join(platform_dir).join("aktools"),
                exe_dir.join("bin").join(platform_dir).join("aktools.exe"),
            ];

            for candidate in &platform_paths {
                if candidate.exists() {
                    return Ok(candidate.to_string_lossy().to_string());
                }
            }
        }

        // 2. 检查环境变量 PATH
        if let Ok(output) = Command::new("which")
            .arg("aktools")
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Ok(path);
                }
            }
        }

        // 3. Windows which 命令不同
        #[cfg(windows)]
        {
            if let Ok(output) = Command::new("where")
                .arg("aktools")
                .output()
            {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    if !path.is_empty() {
                        return Ok(path);
                    }
                }
            }
        }

        Err("AKTools executable not found. Please install aktools or run 'python build.py' in the python/ directory".to_string())
    }

    /// 等待服务就绪
    async fn wait_for_service(&self, max_wait: Duration) -> Result<(), String> {
        let port = *self.port.lock().unwrap();
        let health_url = format!("http://127.0.0.1:{}/version", port);
        println!("Waiting for service on port {}...", port);

        let start = std::time::Instant::now();
        let result = timeout(max_wait, async {
            loop {
                // 检查进程是否还在运行
                {
                    let mut process = self.process.lock().unwrap();
                    if let Some(ref mut child) = *process {
                        match child.try_wait() {
                            Ok(Some(_)) => {
                                return Err("Python process exited unexpectedly".to_string());
                            }
                            Ok(None) => {
                                // 进程仍在运行，继续检查 HTTP
                            }
                            Err(e) => {
                                return Err(format!("Failed to check process status: {}", e));
                            }
                        }
                    } else {
                        return Err("Process not found".to_string());
                    }
                }

                // 尝试连接 HTTP 服务（使用 /version 端点）
                if let Ok(client) = reqwest::Client::builder()
                    .timeout(Duration::from_secs(3))
                    .build()
                {
                    if let Ok(response) = client.get(&health_url).send().await {
                        let status = response.status();
                        if status.is_success() {
                            println!("Service is ready on port {}", port);
                            return Ok(());
                        } else {
                            println!("Health check returned status: {}", status);
                        }
                    } else {
                        // 每5秒打印一次尝试信息
                        let elapsed = start.elapsed().as_secs();
                        if elapsed % 5 == 0 {
                            println!("Still waiting for service... ({}s)", elapsed);
                        }
                    }
                }

                thread::sleep(Duration::from_millis(500));
            }
        }).await;

        match result {
            Ok(inner) => inner,
            Err(_) => Err(format!("Timeout waiting for service on port {}", port)),
        }
    }

    /// 检查服务是否正在运行（通过 HTTP /version 端点）
    async fn is_running(&self) -> bool {
        let port = *self.port.lock().unwrap();

        // 首先检查 HTTP 服务是否可达（使用 /version 端点）
        let health_url = format!("http://127.0.0.1:{}/version", port);
        if let Ok(client) = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
        {
            if let Ok(response) = client.get(&health_url).send().await {
                if response.status().is_success() {
                    return true;
                }
            }
        }

        // HTTP 不可达，检查进程是否还在运行
        {
            let mut process = self.process.lock().unwrap();
            if let Some(ref mut child) = *process {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        // 进程已退出
                        *process = None;
                        return false;
                    }
                    Ok(None) => {
                        // 进程仍在运行但 HTTP 不可达，可能是启动中
                        return false;
                    }
                    Err(_) => return false,
                }
            }
        }

        false
    }

    /// 检查指定端口是否已有服务在运行（使用 /version 端点）
    async fn check_port_in_use(port: u16) -> bool {
        let health_url = format!("http://127.0.0.1:{}/version", port);
        if let Ok(client) = reqwest::Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
        {
            if let Ok(response) = client.get(&health_url).send().await {
                return response.status().is_success();
            }
        }
        false
    }
}

impl Drop for PythonService {
    fn drop(&mut self) {
        // 尝试停止服务
        let mut process = self.process.lock().unwrap();
        if let Some(mut child) = process.take() {
            let _ = child.kill();
        }
    }
}
