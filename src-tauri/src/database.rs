use rusqlite::{Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 股票基础信息
#[derive(Debug, Serialize, Deserialize)]
pub struct Stock {
    pub code: String,
    pub name: String,
    pub exchange: String,
    pub industry: Option<String>,
    pub market_cap: Option<f64>,
    pub list_date: Option<String>,
}

/// 自选股票
#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct WatchlistItem {
    pub id: i64,
    pub code: String,
    pub added_at: String,
}

/// K 线数据缓存
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct KlineCache {
    pub id: i64,
    pub code: String,
    pub period: String, // daily, weekly, monthly
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: Option<f64>,
    pub cached_at: String,
}

/// K线数据点（简化版，用于插入）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KlineDataPoint {
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: Option<f64>,
}

/// 数据库管理
pub struct Database {
    conn: Connection,
}

impl Database {
    /// 初始化数据库连接
    pub fn new(app_dir: PathBuf) -> SqliteResult<Self> {
        let db_path = app_dir.join("valuecompass.db");

        // 确保目录存在
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(&db_path)?;
        let db = Self { conn };
        db.init_tables()?;

        Ok(db)
    }

    /// 创建表结构
    fn init_tables(&self) -> SqliteResult<()> {
        // 股票基础信息表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS stocks (
                code TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                exchange TEXT,
                industry TEXT,
                market_cap REAL,
                list_date TEXT,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;

        // 用户自选表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS watchlist (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL UNIQUE,
                added_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (code) REFERENCES stocks(code)
            )",
            [],
        )?;

        // 数据同步日志
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sync_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                table_name TEXT NOT NULL,
                sync_type TEXT,
                records_count INTEGER,
                started_at TIMESTAMP,
                completed_at TIMESTAMP,
                status TEXT,
                error_message TEXT
            )",
            [],
        )?;

        // 创建索引
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_stocks_name ON stocks(name)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_stocks_industry ON stocks(industry)",
            [],
        )?;

        // K线数据缓存表
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS kline_cache (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                code TEXT NOT NULL,
                period TEXT NOT NULL,
                date TEXT NOT NULL,
                open REAL NOT NULL,
                high REAL NOT NULL,
                low REAL NOT NULL,
                close REAL NOT NULL,
                volume REAL NOT NULL,
                amount REAL,
                cached_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(code, period, date)
            )",
            [],
        )?;

        // K线缓存索引
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_kline_code_period ON kline_cache(code, period)",
            [],
        )?;
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_kline_date ON kline_cache(date)",
            [],
        )?;

        Ok(())
    }

    /// 插入或更新股票
    #[allow(dead_code)]
    pub fn upsert_stock(&self, stock: &Stock) -> SqliteResult<()> {
        self.conn.execute(
            "INSERT INTO stocks (code, name, exchange, industry, market_cap, list_date)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(code) DO UPDATE SET
                name = excluded.name,
                exchange = excluded.exchange,
                industry = excluded.industry,
                market_cap = excluded.market_cap,
                list_date = excluded.list_date,
                updated_at = CURRENT_TIMESTAMP",
            (
                &stock.code,
                &stock.name,
                &stock.exchange,
                &stock.industry,
                &stock.market_cap,
                &stock.list_date,
            ),
        )?;
        Ok(())
    }

    /// 批量插入股票
    pub fn batch_insert_stocks(&self, stocks: &[Stock]) -> SqliteResult<usize> {
        let mut count = 0;
        let tx = self.conn.unchecked_transaction()?;

        for stock in stocks {
            tx.execute(
                "INSERT INTO stocks (code, name, exchange, industry, market_cap, list_date)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(code) DO UPDATE SET
                    name = excluded.name,
                    exchange = excluded.exchange,
                    industry = excluded.industry,
                    market_cap = excluded.market_cap,
                    list_date = excluded.list_date,
                    updated_at = CURRENT_TIMESTAMP",
                (
                    &stock.code,
                    &stock.name,
                    &stock.exchange,
                    &stock.industry,
                    &stock.market_cap,
                    &stock.list_date,
                ),
            )?;
            count += 1;
        }

        tx.commit()?;
        Ok(count)
    }

    /// 获取所有股票列表
    pub fn get_all_stocks(&self) -> SqliteResult<Vec<Stock>> {
        let mut stmt = self.conn.prepare(
            "SELECT code, name, exchange, industry, market_cap, list_date
             FROM stocks
             ORDER BY code"
        )?;

        let stocks = stmt.query_map([], |row| {
            Ok(Stock {
                code: row.get(0)?,
                name: row.get(1)?,
                exchange: row.get(2)?,
                industry: row.get(3)?,
                market_cap: row.get(4)?,
                list_date: row.get(5)?,
            })
        })?;

        stocks.collect()
    }

    /// 搜索股票
    pub fn search_stocks(&self, keyword: &str) -> SqliteResult<Vec<Stock>> {
        let pattern = format!("%{}%", keyword);
        let mut stmt = self.conn.prepare(
            "SELECT code, name, exchange, industry, market_cap, list_date
             FROM stocks
             WHERE code LIKE ?1 OR name LIKE ?1
             ORDER BY code
             LIMIT 50"
        )?;

        let stocks = stmt.query_map([&pattern], |row| {
            Ok(Stock {
                code: row.get(0)?,
                name: row.get(1)?,
                exchange: row.get(2)?,
                industry: row.get(3)?,
                market_cap: row.get(4)?,
                list_date: row.get(5)?,
            })
        })?;

        stocks.collect()
    }

    /// 获取股票数量
    pub fn get_stock_count(&self) -> SqliteResult<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM stocks",
            [],
            |row| row.get(0)
        )?;
        Ok(count)
    }

    /// 添加到自选
    pub fn add_to_watchlist(&self, code: &str) -> SqliteResult<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO watchlist (code) VALUES (?1)",
            [code],
        )?;
        Ok(())
    }

    /// 从自选移除
    pub fn remove_from_watchlist(&self, code: &str) -> SqliteResult<()> {
        self.conn.execute(
            "DELETE FROM watchlist WHERE code = ?1",
            [code],
        )?;
        Ok(())
    }

    /// 获取自选列表
    pub fn get_watchlist(&self) -> SqliteResult<Vec<Stock>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.code, s.name, s.exchange, s.industry, s.market_cap, s.list_date
             FROM stocks s
             INNER JOIN watchlist w ON s.code = w.code
             ORDER BY w.added_at DESC"
        )?;

        let stocks = stmt.query_map([], |row| {
            Ok(Stock {
                code: row.get(0)?,
                name: row.get(1)?,
                exchange: row.get(2)?,
                industry: row.get(3)?,
                market_cap: row.get(4)?,
                list_date: row.get(5)?,
            })
        })?;

        stocks.collect()
    }

    /// 检查是否在自选
    pub fn is_in_watchlist(&self, code: &str) -> SqliteResult<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM watchlist WHERE code = ?1",
            [code],
            |row| row.get(0)
        )?;
        Ok(count > 0)
    }

    /// 记录同步日志
    pub fn log_sync(&self, table_name: &str, sync_type: &str,
                    records_count: i64, status: &str, error: Option<&str>) -> SqliteResult<()> {
        self.conn.execute(
            "INSERT INTO sync_log (table_name, sync_type, records_count, status, error_message, started_at, completed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'), datetime('now'))",
            [table_name, sync_type, &records_count.to_string(), status, error.unwrap_or("")],
        )?;
        Ok(())
    }

    /// 获取K线缓存数据
    pub fn get_kline_cache(&self, code: &str, period: &str, start_date: &str, end_date: &str) -> SqliteResult<Vec<KlineDataPoint>> {
        let mut stmt = self.conn.prepare(
            "SELECT date, open, high, low, close, volume, amount
             FROM kline_cache
             WHERE code = ?1 AND period = ?2 AND date >= ?3 AND date <= ?4
             ORDER BY date"
        )?;

        let klines = stmt.query_map([code, period, start_date, end_date], |row| {
            Ok(KlineDataPoint {
                date: row.get(0)?,
                open: row.get(1)?,
                high: row.get(2)?,
                low: row.get(3)?,
                close: row.get(4)?,
                volume: row.get(5)?,
                amount: row.get(6)?,
            })
        })?;

        klines.collect()
    }

    /// 批量插入或更新K线缓存
    pub fn batch_upsert_kline_cache(&self, code: &str, period: &str, klines: &[KlineDataPoint]) -> SqliteResult<usize> {
        let mut count = 0;
        let tx = self.conn.unchecked_transaction()?;

        for kline in klines {
            tx.execute(
                "INSERT INTO kline_cache (code, period, date, open, high, low, close, volume, amount)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(code, period, date) DO UPDATE SET
                    open = excluded.open,
                    high = excluded.high,
                    low = excluded.low,
                    close = excluded.close,
                    volume = excluded.volume,
                    amount = excluded.amount,
                    cached_at = CURRENT_TIMESTAMP",
                [
                    code,
                    period,
                    &kline.date,
                    &kline.open.to_string(),
                    &kline.high.to_string(),
                    &kline.low.to_string(),
                    &kline.close.to_string(),
                    &kline.volume.to_string(),
                    &kline.amount.map(|v| v.to_string()).unwrap_or_default(),
                ],
            )?;
            count += 1;
        }

        tx.commit()?;
        Ok(count)
    }

    /// 清除过期的K线缓存（保留最近2年的数据）
    #[allow(dead_code)]
    pub fn cleanup_kline_cache(&self) -> SqliteResult<usize> {
        let mut stmt = self.conn.prepare(
            "DELETE FROM kline_cache WHERE date < date('now', '-2 years')"
        )?;
        let count = stmt.execute([])?;
        Ok(count)
    }
}
