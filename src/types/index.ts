// 股票基础信息
export interface Stock {
  code: string;
  name: string;
  exchange: string;
  industry?: string;
  market_cap?: number;
  list_date?: string;
}

// K线数据
export interface KlineData {
  date: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  amount?: number;
}

// 股票详细信息
export interface StockDetail {
  code: string;
  name: string;
  exchange: string;
  industry: string;
  market_cap: number;
  pe_ratio: number;
  pb_ratio: number;
  dividend_yield: number;
}

// Python 服务状态
export type ServiceStatus = 'Stopped' | 'Starting' | 'Running' | { Failed: string };

// 同步结果
export interface SyncResult {
  total: number;
  success: number;
  failed: number;
}
