use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::time::timeout;

/// Python 服务状态
#[derive(Debug, Clone, serde::Serialize)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Failed(String),
}

/// Python 服务管理器
pub struct PythonService {
    process: Arc<Mutex<Option<Child>>>,
    port: u16,
    status: Arc<Mutex<ServiceStatus>>,
}

impl PythonService {
    /// 创建新的服务管理器
    pub fn new(port: u16) -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            port,
            status: Arc::new(Mutex::new(ServiceStatus::Stopped)),
        }
    }

    /// 获取当前状态
    pub fn get_status(&self) -> ServiceStatus {
        self.status.lock().unwrap().clone()
    }

    /// 启动 Python 服务
    pub async fn start(&self) -> Result<(), String> {
        // 检查是否已在运行
        if self.is_running().await {
            return Ok(());
        }

        // 更新状态
        *self.status.lock().unwrap() = ServiceStatus::Starting;

        // 查找 aktools 可执行文件
        let executable = self.find_aktools_executable()?;

        // 启动进程
        let child = Command::new(&executable)
            .arg(format!("--port={}", self.port))
            .arg("--host=127.0.0.1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start AKTools: {}", e))?;

        *self.process.lock().unwrap() = Some(child);

        // 等待服务就绪
        match self.wait_for_service(Duration::from_secs(30)).await {
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
        let mut process = self.process.lock().unwrap();

        if let Some(mut child) = process.take() {
            // 尝试优雅终止
            let _ = child.kill();
            let _ = child.wait();
        }

        *self.status.lock().unwrap() = ServiceStatus::Stopped;
        Ok(())
    }

    /// 查找 aktools 可执行文件
    fn find_aktools_executable(&self) -> Result<String, String> {
        // 1. 首先查找打包目录
        if let Ok(exe_path) = std::env::current_exe() {
            let exe_dir = exe_path.parent().unwrap_or_else(|| std::path::Path::new("."));

            // 检查常见位置
            let candidates = [
                exe_dir.join("aktools"),
                exe_dir.join("aktools.exe"),
                exe_dir.join("python").join("aktools"),
                exe_dir.join("python").join("aktools.exe"),
                exe_dir.join("../Resources/python/aktools"),
            ];

            for candidate in &candidates {
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

        Err("AKTools executable not found. Please install aktools or ensure it's in PATH".to_string())
    }

    /// 等待服务就绪
    async fn wait_for_service(&self, max_wait: Duration) -> Result<(), String> {
        let health_url = format!("http://127.0.0.1:{}/", self.port);

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

                // 尝试连接 HTTP 服务
                if let Ok(client) = reqwest::Client::builder()
                    .timeout(Duration::from_secs(2))
                    .build()
                {
                    if let Ok(response) = client.get(&health_url).send().await {
                        if response.status().is_success() {
                            return Ok(());
                        }
                    }
                }

                thread::sleep(Duration::from_millis(500));
            }
        }).await;

        match result {
            Ok(inner) => inner,
            Err(_) => Err(format!("Timeout waiting for service on port {}", self.port)),
        }
    }

    /// 检查服务是否正在运行
    async fn is_running(&self) -> bool {
        // 检查进程状态
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
                        // 进程仍在运行，继续检查 HTTP
                    }
                    Err(_) => return false,
                }
            } else {
                return false;
            }
        }

        // 检查 HTTP 服务
        let health_url = format!("http://127.0.0.1:{}/", self.port);
        if let Ok(client) = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
        {
            if let Ok(response) = client.get(&health_url).send().await {
                return response.status().is_success();
            }
        }

        false
    }

    /// 获取服务端口
    pub fn get_port(&self) -> u16 {
        self.port
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
