import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { KlineChart } from './components/KlineChart';
import { useWatchlist } from './hooks/useTauri';
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

type Period = 'daily' | 'weekly' | 'monthly';
type ServiceStatus = 'Stopped' | 'Starting' | 'Running' | 'Stopping' | { Failed: string };
type Tab = 'all' | 'watchlist';

const PERIOD_LABELS: Record<Period, string> = {
  daily: '日K',
  weekly: '周K',
  monthly: '月K',
};

function App() {
  const [serviceStatus, setServiceStatus] = useState<ServiceStatus>('Stopped');
  const [stocks, setStocks] = useState<Stock[]>([]);
  const [selectedStock, setSelectedStock] = useState<Stock | null>(null);
  const [klineData, setKlineData] = useState<KlineData[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [klineError, setKlineError] = useState<string | null>(null);
  const [searchKeyword, setSearchKeyword] = useState('');
  const [syncing, setSyncing] = useState(false);
  const [period, setPeriod] = useState<Period>('daily');
  const [activeTab, setActiveTab] = useState<Tab>('all');
  const [watchlistCodes, setWatchlistCodes] = useState<Set<string>>(new Set());

  const {
    watchlist,
    loading: watchlistLoading,
    fetchWatchlist,
    addToWatchlist,
    removeFromWatchlist,
    isInWatchlist,
  } = useWatchlist();

  const isRunning = serviceStatus === 'Running';
  const isStarting = serviceStatus === 'Starting';
  const isStopping = serviceStatus === 'Stopping';
  const isTransitioning = isStarting || isStopping;

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
    try {
      const data = await invoke<Stock[]>('get_all_stocks');
      setStocks(data);
      setError(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  // 搜索股票
  const searchStocks = async () => {
    if (!searchKeyword.trim()) {
      await fetchStocks();
      return;
    }
    try {
      const data = await invoke<Stock[]>('search_stocks', { keyword: searchKeyword });
      setStocks(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
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
  const fetchKline = async (stock: Stock, targetPeriod: Period = period) => {
    setSelectedStock(stock);
    setKlineError(null);
    try {
      const symbol = stock.exchange.toLowerCase() + stock.code;
      const endDate = new Date().toISOString().split('T')[0];
      // 根据周期调整起始日期
      let days = 180;
      if (targetPeriod === 'weekly') days = 365;
      if (targetPeriod === 'monthly') days = 365 * 3;
      const startDate = new Date(Date.now() - days * 24 * 60 * 60 * 1000).toISOString().split('T')[0];

      const data = await invoke<KlineData[]>('get_kline', {
        symbol,
        startDate,
        endDate,
        adjust: 'qfq',
        period: targetPeriod,
      });
      setKlineData(data);
      if (data.length === 0) {
        setKlineError('该周期暂无数据，请尝试其他周期');
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      console.error('Failed to fetch kline:', msg);
      setKlineError(`获取${PERIOD_LABELS[targetPeriod]}数据失败: ${msg}`);
      setKlineData([]);
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

  // 加载自选状态到本地缓存
  const loadWatchlistStatus = async () => {
    const codes = new Set<string>();
    for (const stock of stocks) {
      const inWl = await isInWatchlist(stock.code);
      if (inWl) codes.add(stock.code);
    }
    setWatchlistCodes(codes);
  };

  // 切换自选
  const toggleWatchlist = async (stock: Stock, e: React.MouseEvent) => {
    e.stopPropagation();
    const inWl = watchlistCodes.has(stock.code);
    if (inWl) {
      await removeFromWatchlist(stock.code);
      setWatchlistCodes((prev) => {
        const next = new Set(prev);
        next.delete(stock.code);
        return next;
      });
    } else {
      await addToWatchlist(stock.code);
      setWatchlistCodes((prev) => new Set(prev).add(stock.code));
    }
  };

  // 动态轮询服务状态
  useEffect(() => {
    fetchServiceStatus();
    fetchStocks();
  }, []);

  // 股票列表变化时加载自选状态
  useEffect(() => {
    if (stocks.length > 0) {
      loadWatchlistStatus();
    }
  }, [stocks]);

  // 根据服务状态调整轮询频率
  useEffect(() => {
    // 状态变化时高频轮询 (1s)，稳定状态低频轮询 (10s)
    const intervalMs = isTransitioning ? 1000 : 10000;

    const interval = setInterval(fetchServiceStatus, intervalMs);
    return () => clearInterval(interval);
  }, [serviceStatus, isTransitioning]);

  const getStatusText = () => {
    if (typeof serviceStatus === 'string') {
      return serviceStatus;
    }
    return `Failed: ${serviceStatus.Failed}`;
  };

  return (
    <div className="app">
      <header className="app-header">
        <h1>价值罗盘</h1>
        <div className="status-bar">
          <span>AKTools 服务:</span>
          <span className={`status-${getStatusText().toLowerCase().split(':')[0]}`}>
            {getStatusText()}
          </span>
          {isRunning ? (
            <button onClick={stopService} className="btn-small btn-danger" disabled={isStopping}>
              {isStopping ? '停止中...' : '停止服务'}
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

          <div className="tab-bar">
            <button
              className={`tab ${activeTab === 'all' ? 'active' : ''}`}
              onClick={() => setActiveTab('all')}
            >
              全部股票
            </button>
            <button
              className={`tab ${activeTab === 'watchlist' ? 'active' : ''}`}
              onClick={() => {
                setActiveTab('watchlist');
                fetchWatchlist();
              }}
            >
              自选 ({watchlist.length})
            </button>
          </div>

          <div className="stock-list">
            {error && <div className="error">错误: {error}</div>}
            {activeTab === 'all' ? (
              <>
                {stocks.length === 0 && (
                  <div className="empty">暂无股票数据，请点击"同步股票列表"</div>
                )}
                {stocks.map((stock) => (
                  <div
                    key={stock.code}
                    className={`stock-item ${selectedStock?.code === stock.code ? 'active' : ''}`}
                    onClick={() => {
                      setPeriod('daily');
                      fetchKline(stock, 'daily');
                    }}
                  >
                    <span className="stock-code">{stock.code}</span>
                    <span className="stock-name">{stock.name}</span>
                    <span className="stock-exchange">{stock.exchange}</span>
                    <button
                      className={`watchlist-btn ${watchlistCodes.has(stock.code) ? 'in-watchlist' : ''}`}
                      onClick={(e) => toggleWatchlist(stock, e)}
                      title={watchlistCodes.has(stock.code) ? '移除自选' : '添加自选'}
                    >
                      {watchlistCodes.has(stock.code) ? '★' : '☆'}
                    </button>
                  </div>
                ))}
              </>
            ) : (
              <>
                {watchlistLoading && <div className="empty">加载中...</div>}
                {!watchlistLoading && watchlist.length === 0 && (
                  <div className="empty">暂无自选股票，在"全部股票"中添加</div>
                )}
                {watchlist.map((stock) => (
                  <div
                    key={stock.code}
                    className={`stock-item ${selectedStock?.code === stock.code ? 'active' : ''}`}
                    onClick={() => {
                      setPeriod('daily');
                      fetchKline(stock, 'daily');
                    }}
                  >
                    <span className="stock-code">{stock.code}</span>
                    <span className="stock-name">{stock.name}</span>
                    <span className="stock-exchange">{stock.exchange}</span>
                    <button
                      className="watchlist-btn in-watchlist"
                      onClick={(e) => toggleWatchlist(stock, e)}
                      title="移除自选"
                    >
                      ★
                    </button>
                  </div>
                ))}
              </>
            )}
          </div>
        </div>

        <div className="content">
          {selectedStock ? (
            <div className="stock-detail">
              <h2>
                {selectedStock.name}
                <span className="exchange">{selectedStock.code} · {selectedStock.exchange}</span>
                <button
                  className={`watchlist-btn-large ${watchlistCodes.has(selectedStock.code) ? 'in-watchlist' : ''}`}
                  onClick={(e) => toggleWatchlist(selectedStock, e)}
                >
                  {watchlistCodes.has(selectedStock.code) ? '★ 已自选' : '☆ 加自选'}
                </button>
              </h2>

              {klineError ? (
                <div className="kline-error">{klineError}</div>
              ) : klineData.length === 0 ? (
                <div className="loading">暂无K线数据</div>
              ) : (
                <div className="kline-wrapper">
                  <div className="kline-header">
                    <h3>K线走势</h3>
                    <div className="period-tabs">
                        {(['daily', 'weekly', 'monthly'] as Period[]).map((p) => (
                          <button
                            key={p}
                            className={`period-tab ${period === p ? 'active' : ''}`}
                            onClick={() => {
                              setPeriod(p);
                              if (selectedStock) {
                                fetchKline(selectedStock, p);
                              }
                            }}
                          >
                            {PERIOD_LABELS[p]}
                          </button>
                        ))}
                      </div>
                  </div>
                  <KlineChart
                    data={klineData}
                    height={480}
                    period={period}
                  />
                </div>
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
