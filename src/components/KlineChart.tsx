import { useEffect, useRef, useState } from 'react';
import { createChart, IChartApi, ISeriesApi, CandlestickData, HistogramData, CandlestickSeries, HistogramSeries } from 'lightweight-charts';
import { KlineData } from '../types';

interface KlineChartProps {
  data: KlineData[];
  height?: number;
  period?: 'daily' | 'weekly' | 'monthly';
}

export function KlineChart({ data, height = 400, period: _period }: KlineChartProps) {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const candlestickSeriesRef = useRef<ISeriesApi<'Candlestick'> | null>(null);
  const volumeSeriesRef = useRef<ISeriesApi<'Histogram'> | null>(null);
  const [tooltip, setTooltip] = useState<{
    open: number;
    high: number;
    low: number;
    close: number;
    volume: number;
    date: string;
  } | null>(null);

  useEffect(() => {
    if (!chartContainerRef.current) return;

    // 创建图表
    const chart = createChart(chartContainerRef.current, {
      layout: {
        background: { color: '#ffffff' },
        textColor: '#333',
      },
      grid: {
        vertLines: { color: '#f0f0f0' },
        horzLines: { color: '#f0f0f0' },
      },
      crosshair: {
        mode: 1,
      },
      rightPriceScale: {
        borderColor: '#ddd',
      },
      timeScale: {
        borderColor: '#ddd',
        timeVisible: true,
        secondsVisible: false,
      },
      autoSize: true,
    });

    chartRef.current = chart;

    // 创建 K 线系列 (v5 API)
    const candlestickSeries = chart.addSeries(CandlestickSeries, {
      upColor: '#ef5350',
      downColor: '#26a69a',
      borderVisible: false,
      wickUpColor: '#ef5350',
      wickDownColor: '#26a69a',
    });

    candlestickSeriesRef.current = candlestickSeries;

    // 创建成交量系列 (v5 API)
    const volumeSeries = chart.addSeries(HistogramSeries, {
      color: '#26a69a',
      priceFormat: {
        type: 'volume',
      },
      priceScaleId: '',
    });

    volumeSeriesRef.current = volumeSeries;

    // 设置成交量在右侧
    volumeSeries.priceScale().applyOptions({
      scaleMargins: {
        top: 0.8,
        bottom: 0,
      },
    });

    // 鼠标移动事件
    chart.subscribeCrosshairMove((param) => {
      if (!param.point || !param.time || !candlestickSeries) {
        setTooltip(null);
        return;
      }

      const dataPoint = param.seriesData.get(candlestickSeries) as CandlestickData;
      const volumeData = param.seriesData.get(volumeSeries) as HistogramData;

      if (dataPoint) {
        setTooltip({
          open: dataPoint.open,
          high: dataPoint.high,
          low: dataPoint.low,
          close: dataPoint.close,
          volume: volumeData?.value || 0,
          date: String(param.time),
        });
      }
    });

    return () => {
      chart.remove();
      chartRef.current = null;
    };
  }, []);

  // 更新数据
  useEffect(() => {
    if (!candlestickSeriesRef.current || !volumeSeriesRef.current || data.length === 0) return;

    // 转换 K 线数据
    const candleData: CandlestickData[] = data.map((item) => ({
      time: item.date,
      open: item.open,
      high: item.high,
      low: item.low,
      close: item.close,
    }));

    // 转换成交量数据
    const volumeData: HistogramData[] = data.map((item) => ({
      time: item.date,
      value: item.volume,
      color: item.close >= item.open ? '#ef5350' : '#26a69a',
    }));

    candlestickSeriesRef.current.setData(candleData);
    volumeSeriesRef.current.setData(volumeData);

    // 自适应时间范围
    chartRef.current?.timeScale().fitContent();
  }, [data]);

  return (
    <div className="kline-chart-wrapper" style={{ position: 'relative' }}>
      <div
        ref={chartContainerRef}
        style={{ height: `${height}px`, width: '100%' }}
      />
      {tooltip && (
        <div
          style={{
            position: 'absolute',
            top: 10,
            left: 10,
            background: 'rgba(255, 255, 255, 0.95)',
            padding: '8px 12px',
            borderRadius: '4px',
            boxShadow: '0 2px 8px rgba(0,0,0,0.15)',
            fontSize: '12px',
            lineHeight: '1.5',
            zIndex: 10,
          }}
        >
          <div style={{ color: '#666' }}>{tooltip.date}</div>
          <div style={{ display: 'grid', gridTemplateColumns: 'auto auto', gap: '0 12px' }}>
            <span style={{ color: '#666' }}>开:</span>
            <span style={{ color: tooltip.open <= tooltip.close ? '#ef5350' : '#26a69a' }}>
              {tooltip.open.toFixed(2)}
            </span>
            <span style={{ color: '#666' }}>高:</span>
            <span>{tooltip.high.toFixed(2)}</span>
            <span style={{ color: '#666' }}>低:</span>
            <span>{tooltip.low.toFixed(2)}</span>
            <span style={{ color: '#666' }}>收:</span>
            <span style={{ color: tooltip.close >= tooltip.open ? '#ef5350' : '#26a69a' }}>
              {tooltip.close.toFixed(2)}
            </span>
            <span style={{ color: '#666' }}>量:</span>
            <span>{(tooltip.volume / 10000).toFixed(2)}万</span>
          </div>
        </div>
      )}
    </div>
  );
}
