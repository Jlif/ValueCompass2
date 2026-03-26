use serde::{Deserialize, Serialize};

/// K 线数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineData {
    pub date: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub amount: f64,
    pub amplitude: Option<f64>,
    pub pct_change: Option<f64>,
    pub change: Option<f64>,
    pub turnover: Option<f64>,
}

/// AKTools HTTP 客户端
pub struct AKToolsClient {
    base_url: String,
    client: reqwest::Client,
}

impl AKToolsClient {
    /// 创建新的客户端
    pub fn new(port: u16) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{}", port),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    /// 获取股票列表
    pub async fn get_stock_list(&self) -> Result<Vec<StockInfo>, String> {
        let url = format!("{}/api/stock_zh_a_spot_em", self.base_url);

        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let data: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let stocks: Vec<StockInfo> = data
            .into_iter()
            .filter_map(|v| {
                Some(StockInfo {
                    code: v.get("代码")?.as_str()?.to_string(),
                    name: v.get("名称")?.as_str()?.to_string(),
                    price: v.get("最新价")?.as_f64(),
                    change_pct: v.get("涨跌幅")?.as_f64(),
                    volume: v.get("成交量")?.as_f64(),
                    amount: v.get("成交额")?.as_f64(),
                    market_cap: v.get("总市值")?.as_f64(),
                })
            })
            .collect();

        Ok(stocks)
    }

    /// 获取 K 线数据
    pub async fn get_kline(
        &self,
        symbol: &str,
        period: &str,
        start_date: &str,
        end_date: &str,
        adjust: &str,
    ) -> Result<Vec<KlineData>, String> {
        let url = format!("{}/api/stock_zh_a_hist", self.base_url);

        let params = [
            ("symbol", symbol),
            ("period", period),
            ("start_date", start_date),
            ("end_date", end_date),
            ("adjust", adjust),
        ];

        let response = self.client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let data: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        let klines: Vec<KlineData> = data
            .into_iter()
            .filter_map(|v| {
                Some(KlineData {
                    date: v.get("日期")?.as_str()?.to_string(),
                    open: v.get("开盘")?.as_f64()?,
                    high: v.get("最高")?.as_f64()?,
                    low: v.get("最低")?.as_f64()?,
                    close: v.get("收盘")?.as_f64()?,
                    volume: v.get("成交量")?.as_f64()?,
                    amount: v.get("成交额")?.as_f64()?,
                    amplitude: v.get("振幅").and_then(|v| v.as_f64()),
                    pct_change: v.get("涨跌幅").and_then(|v| v.as_f64()),
                    change: v.get("涨跌额").and_then(|v| v.as_f64()),
                    turnover: v.get("换手率").and_then(|v| v.as_f64()),
                })
            })
            .collect();

        Ok(klines)
    }

    /// 获取个股信息
    pub async fn get_stock_info(&self, symbol: &str) -> Result<StockDetail, String> {
        let url = format!("{}/api/stock_individual_info_em", self.base_url);

        let response = self.client
            .get(&url)
            .query(&[("symbol", symbol)])
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let data: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;

        // 查找对应的股票信息
        let stock = data
            .into_iter()
            .find(|v| v.get("股票代码").and_then(|c| c.as_str()) == Some(symbol))
            .ok_or("Stock not found")?;

        Ok(StockDetail {
            code: symbol.to_string(),
            name: stock.get("股票简称")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or("Missing name field")?,
            industry: stock.get("行业")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            total_market_cap: stock.get("总市值").and_then(|v| v.as_f64()),
            float_market_cap: stock.get("流通市值").and_then(|v| v.as_f64()),
            total_shares: stock.get("总股本").and_then(|v| v.as_f64()),
            float_shares: stock.get("流通股本").and_then(|v| v.as_f64()),
            list_date: stock.get("上市时间")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .ok_or("Missing list_date field")?,
        })
    }
}

/// 股票基本信息（来自实时行情）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockInfo {
    pub code: String,
    pub name: String,
    pub price: Option<f64>,
    pub change_pct: Option<f64>,
    pub volume: Option<f64>,
    pub amount: Option<f64>,
    pub market_cap: Option<f64>,
}

/// 股票详细信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockDetail {
    pub code: String,
    pub name: String,
    pub industry: Option<String>,
    pub total_market_cap: Option<f64>,
    pub float_market_cap: Option<f64>,
    pub total_shares: Option<f64>,
    pub float_shares: Option<f64>,
    pub list_date: String,
}
