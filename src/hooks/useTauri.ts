import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Stock, ServiceStatus, SyncResult } from '../types';

// 使用 invoke 调用 Rust 命令
export function useTauriCommand() {
  const invokeCommand = async <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
    return invoke(command, args);
  };

  return { invokeCommand };
}

// 股票列表 Hook
export function useStocks() {
  const [stocks, setStocks] = useState<Stock[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchStocks = async () => {
    setLoading(true);
    setError(null);
    try {
      const data = await invoke<Stock[]>('get_all_stocks');
      setStocks(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  const searchStocks = async (keyword: string) => {
    if (!keyword.trim()) {
      await fetchStocks();
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const data = await invoke<Stock[]>('search_stocks', { keyword });
      setStocks(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  const syncStocks = async (): Promise<SyncResult> => {
    setLoading(true);
    setError(null);
    try {
      const result = await invoke<SyncResult>('sync_stocks_from_aktools');
      await fetchStocks();
      return result;
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : String(e);
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStocks();
  }, []);

  return { stocks, loading, error, fetchStocks, searchStocks, syncStocks };
}

// 自选列表 Hook
export function useWatchlist() {
  const [watchlist, setWatchlist] = useState<Stock[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchWatchlist = async () => {
    setLoading(true);
    try {
      const data = await invoke<Stock[]>('get_watchlist');
      setWatchlist(data);
    } catch (e) {
      console.error('Failed to fetch watchlist:', e);
    } finally {
      setLoading(false);
    }
  };

  const addToWatchlist = async (code: string) => {
    try {
      await invoke('add_to_watchlist', { code });
      await fetchWatchlist();
    } catch (e) {
      console.error('Failed to add to watchlist:', e);
    }
  };

  const removeFromWatchlist = async (code: string) => {
    try {
      await invoke('remove_from_watchlist', { code });
      await fetchWatchlist();
    } catch (e) {
      console.error('Failed to remove from watchlist:', e);
    }
  };

  const isInWatchlist = async (code: string): Promise<boolean> => {
    try {
      return await invoke<boolean>('is_in_watchlist', { code });
    } catch (e) {
      console.error('Failed to check watchlist:', e);
      return false;
    }
  };

  useEffect(() => {
    fetchWatchlist();
  }, []);

  return { watchlist, loading, fetchWatchlist, addToWatchlist, removeFromWatchlist, isInWatchlist };
}

// Python 服务状态 Hook
export function usePythonService() {
  const [status, setStatus] = useState<ServiceStatus>('Stopped');
  const [loading, setLoading] = useState(false);

  const fetchStatus = async () => {
    try {
      const data = await invoke<ServiceStatus>('get_python_service_status');
      setStatus(data);
    } catch (e) {
      console.error('Failed to fetch service status:', e);
    }
  };

  const startService = async () => {
    setLoading(true);
    try {
      await invoke('start_python_service');
      await fetchStatus();
    } catch (e) {
      console.error('Failed to start service:', e);
    } finally {
      setLoading(false);
    }
  };

  const stopService = async () => {
    setLoading(true);
    try {
      await invoke('stop_python_service');
      await fetchStatus();
    } catch (e) {
      console.error('Failed to stop service:', e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStatus();
    // 每 5 秒检查一次状态
    const interval = setInterval(fetchStatus, 5000);
    return () => clearInterval(interval);
  }, []);

  return { status, loading, fetchStatus, startService, stopService };
}
