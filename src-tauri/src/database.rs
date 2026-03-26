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
pub struct WatchlistItem {
    pub id: i64,
    pub code: String,
    pub added_at: String,
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

        Ok(())
    }

    /// 插入或更新股票
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
            [
                &stock.code,
                &stock.name,
                &stock.exchange,
                &stock.industry.as_deref().unwrap_or(""),
                &stock.market_cap,
                &stock.list_date.as_deref().unwrap_or(""),
            ],
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
                [
                    &stock.code,
                    &stock.name,
                    &stock.exchange,
                    &stock.industry.as_deref().unwrap_or(""),
                    &stock.market_cap,
                    &stock.list_date.as_deref().unwrap_or(""),
                ],
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
}
