import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';

// 简单类型定义
interface Stock {
  code: string;
  name: string;
  exchange: string;
  industry?: string;
  market_cap?: number;
}

interface KlineData {
  date: string;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

type ServiceStatus = 'Stopped' | 'Starting' | 'Running' | { Failed: string };

function App() {
  const [serviceStatus, setServiceStatus] = useState<ServiceStatus>('Stopped');
  const [stocks, setStocks] = useState<Stock[]>([]);
  const [selectedStock, setSelectedStock] = useState<Stock | null>(null);
  const [klineData, setKlineData] = useState<KlineData[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchKeyword, setSearchKeyword] = useState('');
  const [syncing, setSyncing] = useState(false);

  // 获取服务状态
  const fetchServiceStatus = async () => {
    try {
      const status = await invoke<ServiceStatus>('get_python_service_status');
      setServiceStatus(status);
    } catch (e) {
      console.error('Failed to get service status:', e);
    }
  };

  // 获取股票列表
  const fetchStocks = async () => {
    setLoading(true);
    try {
      const data = await invoke<Stock[]>('get_all_stocks');
      setStocks(data);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  // 搜索股票
  const searchStocks = async () => {
    if (!searchKeyword.trim()) {
      await fetchStocks();
      return;
    }
    setLoading(true);
    try {
      const data = await invoke<Stock[]>('search_stocks', { keyword: searchKeyword });
      setStocks(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  // 同步股票列表
  const syncStocks = async () => {
    setSyncing(true);
    try {
      // 如果服务未运行，先尝试启动
      if (!isRunning) {
        await startService();
        // 等待服务就绪
        await new Promise(resolve => setTimeout(resolve, 3000));
      }
      await invoke('sync_stocks_from_aktools');
      await fetchStocks();
      alert('股票列表同步成功！');
    } catch (e) {
      alert('同步失败: ' + (e instanceof Error ? e.message : String(e)));
    } finally {
      setSyncing(false);
    }
  };

  // 获取 K 线数据
  const fetchKline = async (stock: Stock) => {
    setSelectedStock(stock);
    setLoading(true);
    try {
      const symbol = stock.exchange.toLowerCase() + stock.code;
      const endDate = new Date().toISOString().split('T')[0];
      const startDate = new Date(Date.now() - 180 * 24 * 60 * 60 * 1000).toISOString().split('T')[0];

      const data = await invoke<KlineData[]>('get_kline', {
        symbol,
        startDate,
        endDate,
        adjust: 'qfq',
      });
      setKlineData(data);
    } catch (e) {
      console.error('Failed to fetch kline:', e);
      setKlineData([]);
    } finally {
      setLoading(false);
    }
  };

  // 启动服务
  const startService = async () => {
    try {
      await invoke('start_python_service');
      await fetchServiceStatus();
    } catch (e) {
      alert('启动服务失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  // 停止服务
  const stopService = async () => {
    try {
      await invoke('stop_python_service');
      await fetchServiceStatus();
    } catch (e) {
      alert('停止服务失败: ' + (e instanceof Error ? e.message : String(e)));
    }
  };

  useEffect(() => {
    fetchServiceStatus();
    fetchStocks();
    const interval = setInterval(fetchServiceStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  const getStatusText = () => {
    if (typeof serviceStatus === 'string') {
      return serviceStatus;
    }
    return `Failed: ${serviceStatus.Failed}`;
  };

  const isRunning = serviceStatus === 'Running';
  const isStarting = serviceStatus === 'Starting';

  return (
    <div className="app">
      <header className="app-header">
        <h1>价值罗盘 - A股价值投资的好帮手</h1>
        <div className="status-bar">
          <span>aktools服务: </span>
          <span className={`status-${getStatusText().toLowerCase().split(':')[0]}`}>
            {getStatusText()}
          </span>
          {isRunning ? (
            <button onClick={stopService} className="btn-small btn-danger">
              停止服务
            </button>
          ) : (
            <button
              onClick={startService}
              className="btn-small"
              disabled={isStarting}
            >
              {isStarting ? '启动中...' : '启动服务'}
            </button>
          )}
        </div>
      </header>

      <main className="app-main">
        <div className="sidebar">
          <div className="toolbar">
            <div className="search-box">
              <input
                type="text"
                placeholder="搜索股票代码或名称..."
                value={searchKeyword}
                onChange={(e) => setSearchKeyword(e.target.value)}
                onKeyDown={(e) => e.key === 'Enter' && searchStocks()}
              />
              <button onClick={searchStocks}>搜索</button>
            </div>
            <button onClick={syncStocks} disabled={syncing} className="btn-primary">
              {syncing ? '同步中...' : '同步股票列表'}
            </button>
          </div>

          <div className="stock-list">
            {loading && <div className="loading">加载中...</div>}
            {error && <div className="error">错误: {error}</div>}
            {!loading && stocks.length === 0 && (
              <div className="empty">暂无股票数据，请点击"同步股票列表"</div>
            )}
            {stocks.map((stock) => (
              <div
                key={stock.code}
                className={`stock-item ${selectedStock?.code === stock.code ? 'active' : ''}`}
                onClick={() => fetchKline(stock)}
              >
                <span className="stock-code">{stock.code}</span>
                <span className="stock-name">{stock.name}</span>
                <span className="stock-exchange">{stock.exchange}</span>
              </div>
            ))}
          </div>
        </div>

        <div className="content">
          {selectedStock ? (
            <div className="stock-detail">
              <h2>
                {selectedStock.name} ({selectedStock.code})
                <span className="exchange">{selectedStock.exchange}</span>
              </h2>

              {klineData.length > 0 ? (
                <div className="kline-container">
                  <h3>K线数据 (近6个月)</h3>
                  <div className="kline-stats">
                    <div>共 {klineData.length} 个交易日</div>
                    <div>最新: {klineData[klineData.length - 1]?.close.toFixed(2)}</div>
                    <div>最高: {Math.max(...klineData.map(d => d.high)).toFixed(2)}</div>
                    <div>最低: {Math.min(...klineData.map(d => d.low)).toFixed(2)}</div>
                  </div>
                  <div className="kline-table-wrapper">
                    <table className="kline-table">
                      <thead>
                        <tr>
                          <th>日期</th>
                          <th>开盘</th>
                          <th>最高</th>
                          <th>最低</th>
                          <th>收盘</th>
                          <th>成交量</th>
                        </tr>
                      </thead>
                      <tbody>
                        {[...klineData].reverse().slice(0, 20).map((item) => (
                          <tr key={item.date}>
                            <td>{item.date.split('T')[0]}</td>
                            <td>{item.open.toFixed(2)}</td>
                            <td>{item.high.toFixed(2)}</td>
                            <td>{item.low.toFixed(2)}</td>
                            <td className={item.close >= item.open ? 'up' : 'down'}>
                              {item.close.toFixed(2)}
                            </td>
                            <td>{(item.volume / 10000).toFixed(2)}万</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              ) : (
                <div className="loading">加载K线数据中...</div>
              )}
            </div>
          ) : (
            <div className="empty-state">
              <p>请选择左侧股票查看详情</p>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}

export default App;
