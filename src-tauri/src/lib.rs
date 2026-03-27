use std::sync::{Arc, Mutex};
use tauri::{Manager, State};

mod database;
mod python_service;
mod aktools_client;

use database::{Database, Stock};
use python_service::{PythonService, ServiceStatus};
use aktools_client::{AKToolsClient, KlineData, StockDetail};

// 应用状态
pub struct AppState {
    db: Arc<Mutex<Database>>,
    python_service: Arc<PythonService>,
}

// 初始化应用状态
fn setup_app_state(app: &tauri::App) -> Result<AppState, Box<dyn std::error::Error>> {
    // 获取应用数据目录
    let app_dir = app.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;

    // 初始化数据库
    let db = Database::new(app_dir)?;
    let db = Arc::new(Mutex::new(db));

    // 初始化 Python 服务（传入 0 表示自动选择可用端口）
    let python_service = Arc::new(PythonService::new(0));

    Ok(AppState {
        db,
        python_service,
    })
}

// ==================== 数据库命令 ====================

/// 获取所有股票列表
#[tauri::command]
async fn get_all_stocks(state: State<'_, AppState>) -> Result<Vec<Stock>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_all_stocks().map_err(|e| e.to_string())
}

/// 搜索股票
#[tauri::command]
async fn search_stocks(keyword: String, state: State<'_, AppState>) -> Result<Vec<Stock>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.search_stocks(&keyword).map_err(|e| e.to_string())
}

/// 获取股票数量
#[tauri::command]
async fn get_stock_count(state: State<'_, AppState>) -> Result<i64, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_stock_count().map_err(|e| e.to_string())
}

/// 添加到自选
#[tauri::command]
async fn add_to_watchlist(code: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.add_to_watchlist(&code).map_err(|e| e.to_string())
}

/// 从自选移除
#[tauri::command]
async fn remove_from_watchlist(code: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.remove_from_watchlist(&code).map_err(|e| e.to_string())
}

/// 获取自选列表
#[tauri::command]
async fn get_watchlist(state: State<'_, AppState>) -> Result<Vec<Stock>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_watchlist().map_err(|e| e.to_string())
}

/// 检查是否在自选
#[tauri::command]
async fn is_in_watchlist(code: String, state: State<'_, AppState>) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.is_in_watchlist(&code).map_err(|e| e.to_string())
}

// ==================== Python 服务命令 ====================

/// 获取 Python 服务状态
#[tauri::command]
async fn get_python_service_status(state: State<'_, AppState>) -> Result<ServiceStatus, String> {
    Ok(state.python_service.get_status())
}

/// 启动 Python 服务
#[tauri::command]
async fn start_python_service(state: State<'_, AppState>) -> Result<(), String> {
    state.python_service.start().await
}

/// 停止 Python 服务
#[tauri::command]
async fn stop_python_service(state: State<'_, AppState>) -> Result<(), String> {
    state.python_service.stop().await
}

// ==================== AKTools 命令 ====================

/// 从 AKTools 同步股票列表
#[tauri::command]
async fn sync_stocks_from_aktools(state: State<'_, AppState>) -> Result<SyncResult, String> {
    println!("Starting sync_stocks_from_aktools...");
    let port = state.python_service.get_port();
    println!("Using port: {}", port);
    let client = AKToolsClient::new(port);

    // 获取股票列表
    println!("Fetching stock list from AKTools...");
    let stocks_info = client.get_stock_list().await.map_err(|e| {
        println!("Error getting stock list: {}", e);
        e
    })?;
    println!("Got {} stocks from AKTools", stocks_info.len());

    // 转换为 Stock 结构
    let stocks: Vec<Stock> = stocks_info
        .into_iter()
        .map(|info| {
            let exchange = get_exchange(&info.code);
            Stock {
                code: info.code,
                name: info.name,
                exchange,
                industry: None,
                market_cap: info.market_cap,
                list_date: None,
            }
        })
        .collect();

    let count = stocks.len();
    println!("Converted {} stocks", count);

    // 批量插入数据库
    println!("Inserting into database...");
    let db = state.db.lock().map_err(|e| e.to_string())?;
    match db.batch_insert_stocks(&stocks) {
        Ok(inserted) => println!("Successfully inserted {} stocks", inserted),
        Err(e) => {
            println!("Error inserting stocks: {}", e);
            return Err(e.to_string());
        }
    }
    db.log_sync("stocks", "full", count as i64, "success", None)
        .map_err(|e| e.to_string())?;

    println!("Sync completed successfully");
    Ok(SyncResult {
        total: count,
        success: count,
        failed: 0,
    })
}

/// 获取 K 线数据
#[tauri::command]
async fn get_kline(
    symbol: String,
    start_date: String,
    end_date: String,
    adjust: String,
    state: State<'_, AppState>,
) -> Result<Vec<KlineData>, String> {
    let port = state.python_service.get_port();
    let client = AKToolsClient::new(port);
    client.get_kline(&symbol, "daily", &start_date, &end_date, &adjust).await
}

/// 获取股票详细信息
#[tauri::command]
async fn get_stock_detail(symbol: String, state: State<'_, AppState>) -> Result<StockDetail, String> {
    let port = state.python_service.get_port();
    let client = AKToolsClient::new(port);
    client.get_stock_info(&symbol).await
}

// ==================== 辅助函数 ====================

/// 根据股票代码判断交易所
fn get_exchange(code: &str) -> String {
    if code.starts_with("6") {
        "SH".to_string()
    } else if code.starts_with("0") || code.starts_with("3") {
        "SZ".to_string()
    } else if code.starts_with("4") || code.starts_with("8") {
        "BJ".to_string()
    } else {
        "Unknown".to_string()
    }
}

/// 同步结果
#[derive(serde::Serialize)]
pub struct SyncResult {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
}

// ==================== 应用入口 ====================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 初始化应用状态
            let state = setup_app_state(app)?;

            // 检查数据库是否为空，如果是则标记需要同步
            let need_sync = {
                let db = state.db.lock().map_err(|e| e.to_string())?;
                db.get_stock_count().unwrap_or(0) == 0
            };

            app.manage(state);

            // 启动时自动启动 Python 服务
            let handle = app.handle().clone();
            let handle_for_sync = handle.clone();
            tauri::async_runtime::spawn(async move {
                if let Some(state) = handle.try_state::<AppState>() {
                    // 尝试启动服务
                    if let Err(e) = state.python_service.start().await {
                        eprintln!("Failed to start Python service: {}", e);
                        // 服务启动失败，可能是开发环境没有打包 aktools
                        // 继续运行，用户可手动启动服务
                    }
                }
            });

            // 如果数据库为空，启动后自动同步
            if need_sync {
                tauri::async_runtime::spawn(async move {
                    // 等待服务启动
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                    if let Some(state) = handle_for_sync.try_state::<AppState>() {
                        println!("Database is empty, auto-syncing stocks...");
                        let port = state.python_service.get_port();
                        let client = AKToolsClient::new(port);

                        if let Ok(stocks_info) = client.get_stock_list().await {
                            let stocks: Vec<Stock> = stocks_info
                                .into_iter()
                                .map(|info| {
                                    let exchange = get_exchange(&info.code);
                                    Stock {
                                        code: info.code,
                                        name: info.name,
                                        exchange,
                                        industry: None,
                                        market_cap: info.market_cap,
                                        list_date: None,
                                    }
                                })
                                .collect();

                            let count = stocks.len();
                            if let Ok(db) = state.db.lock() {
                                if let Ok(inserted) = db.batch_insert_stocks(&stocks) {
                                    println!("Auto-sync completed: {} stocks inserted", inserted);
                                    let _ = db.log_sync("stocks", "auto", count as i64, "success", None);
                                }
                            }
                        }
                    }
                });
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // 窗口关闭时停止 Python 服务
            match event {
                tauri::WindowEvent::Destroyed => {
                    println!("Window destroyed, stopping Python service...");
                    if let Some(state) = window.try_state::<AppState>() {
                        tauri::async_runtime::block_on(async {
                            let _ = state.python_service.stop().await;
                        });
                    }
                }
                tauri::WindowEvent::CloseRequested { .. } => {
                    println!("Close requested, will stop Python service...");
                    if let Some(state) = window.try_state::<AppState>() {
                        tauri::async_runtime::block_on(async {
                            let _ = state.python_service.stop().await;
                        });
                    }
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            // 数据库命令
            get_all_stocks,
            search_stocks,
            get_stock_count,
            add_to_watchlist,
            remove_from_watchlist,
            get_watchlist,
            is_in_watchlist,
            // Python 服务命令
            get_python_service_status,
            start_python_service,
            stop_python_service,
            // AKTools 命令
            sync_stocks_from_aktools,
            get_kline,
            get_stock_detail,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            // 应用退出事件处理
            match event {
                tauri::RunEvent::Exit => {
                    println!("Application exiting, stopping Python service...");
                    if let Some(state) = _app_handle.try_state::<AppState>() {
                        tauri::async_runtime::block_on(async {
                            let _ = state.python_service.stop().await;
                        });
                    }
                }
                tauri::RunEvent::ExitRequested { .. } => {
                    println!("Exit requested, stopping Python service...");
                    if let Some(state) = _app_handle.try_state::<AppState>() {
                        tauri::async_runtime::block_on(async {
                            let _ = state.python_service.stop().await;
                        });
                    }
                }
                _ => {}
            }
        });
}
