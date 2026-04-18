import { useEffect, useRef, useState, useCallback } from 'react';
import {
  createChart,
  IChartApi,
  ISeriesApi,
  CandlestickData,
  HistogramData,
  LineData,
  CandlestickSeries,
  HistogramSeries,
  LineSeries,
} from 'lightweight-charts';
import { KlineData } from '../types';
import { IndicatorType, calculateMACD, calculateKDJ, calculateRSI, MACDData, KDJData, RSIData } from '../utils/indicators';

interface KlineChartProps {
  data: KlineData[];
  height?: number;
  period?: 'daily' | 'weekly' | 'monthly';
  indicators?: IndicatorType[];
}

interface TooltipData {
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
  date: string;
  indicatorValues?: Record<string, number | undefined>;
}

export function KlineChart({ data, height = 400, period: _period, indicators = [] }: KlineChartProps) {
  const chartContainerRef = useRef<HTMLDivElement>(null);
  const chartRef = useRef<IChartApi | null>(null);
  const candlestickSeriesRef = useRef<ISeriesApi<'Candlestick'> | null>(null);
  const volumeSeriesRef = useRef<ISeriesApi<'Histogram'> | null>(null);

  // Indicator series refs
  const macdDifRef = useRef<ISeriesApi<'Line'> | null>(null);
  const macdDeaRef = useRef<ISeriesApi<'Line'> | null>(null);
  const macdHistRef = useRef<ISeriesApi<'Histogram'> | null>(null);
  const kdjKRef = useRef<ISeriesApi<'Line'> | null>(null);
  const kdjDRef = useRef<ISeriesApi<'Line'> | null>(null);
  const kdjJRef = useRef<ISeriesApi<'Line'> | null>(null);
  const rsiRef = useRef<ISeriesApi<'Line'> | null>(null);
  const rsiUpperRef = useRef<ISeriesApi<'Line'> | null>(null);
  const rsiLowerRef = useRef<ISeriesApi<'Line'> | null>(null);

  const [tooltip, setTooltip] = useState<TooltipData | null>(null);

  // Track active indicators to handle cleanup
  const activeIndicatorsRef = useRef<Set<IndicatorType>>(new Set());

  const removeIndicator = useCallback((chart: IChartApi, indicator: IndicatorType) => {
    switch (indicator) {
      case 'macd':
        if (macdDifRef.current) { chart.removeSeries(macdDifRef.current); macdDifRef.current = null; }
        if (macdDeaRef.current) { chart.removeSeries(macdDeaRef.current); macdDeaRef.current = null; }
        if (macdHistRef.current) { chart.removeSeries(macdHistRef.current); macdHistRef.current = null; }
        break;
      case 'kdj':
        if (kdjKRef.current) { chart.removeSeries(kdjKRef.current); kdjKRef.current = null; }
        if (kdjDRef.current) { chart.removeSeries(kdjDRef.current); kdjDRef.current = null; }
        if (kdjJRef.current) { chart.removeSeries(kdjJRef.current); kdjJRef.current = null; }
        break;
      case 'rsi':
        if (rsiRef.current) { chart.removeSeries(rsiRef.current); rsiRef.current = null; }
        if (rsiUpperRef.current) { chart.removeSeries(rsiUpperRef.current); rsiUpperRef.current = null; }
        if (rsiLowerRef.current) { chart.removeSeries(rsiLowerRef.current); rsiLowerRef.current = null; }
        break;
    }
    activeIndicatorsRef.current.delete(indicator);
  }, []);

  const addMACD = useCallback((chart: IChartApi, macdData: MACDData[]) => {
    const difSeries = chart.addSeries(LineSeries, {
      color: '#60a5fa',
      lineWidth: 1,
      title: 'DIF',
      priceScaleId: 'macd',
    });
    const deaSeries = chart.addSeries(LineSeries, {
      color: '#fbbf24',
      lineWidth: 1,
      title: 'DEA',
      priceScaleId: 'macd',
    });
    const histSeries = chart.addSeries(HistogramSeries, {
      priceScaleId: 'macd',
    });

    difSeries.priceScale().applyOptions({ scaleMargins: { top: 0.7, bottom: 0.05 } });

    const difData: LineData[] = [];
    const deaData: LineData[] = [];
    const histData: HistogramData[] = [];

    for (const d of macdData) {
      if (!Number.isNaN(d.dif)) difData.push({ time: d.time, value: d.dif });
      if (!Number.isNaN(d.dea)) deaData.push({ time: d.time, value: d.dea });
      if (!Number.isNaN(d.macd)) {
        histData.push({ time: d.time, value: d.macd, color: d.macd >= 0 ? '#f87171' : '#34d399' });
      }
    }

    difSeries.setData(difData);
    deaSeries.setData(deaData);
    histSeries.setData(histData);

    macdDifRef.current = difSeries;
    macdDeaRef.current = deaSeries;
    macdHistRef.current = histSeries;
    activeIndicatorsRef.current.add('macd');
  }, []);

  const addKDJ = useCallback((chart: IChartApi, kdjData: KDJData[]) => {
    const kSeries = chart.addSeries(LineSeries, {
      color: '#60a5fa',
      lineWidth: 1,
      title: 'K',
      priceScaleId: 'kdj',
    });
    const dSeries = chart.addSeries(LineSeries, {
      color: '#fbbf24',
      lineWidth: 1,
      title: 'D',
      priceScaleId: 'kdj',
    });
    const jSeries = chart.addSeries(LineSeries, {
      color: '#c084fc',
      lineWidth: 1,
      title: 'J',
      priceScaleId: 'kdj',
    });

    kSeries.priceScale().applyOptions({ scaleMargins: { top: 0.7, bottom: 0.05 } });

    const kData: LineData[] = [];
    const dData: LineData[] = [];
    const jData: LineData[] = [];

    for (const item of kdjData) {
      if (!Number.isNaN(item.k)) kData.push({ time: item.time, value: item.k });
      if (!Number.isNaN(item.d)) dData.push({ time: item.time, value: item.d });
      if (!Number.isNaN(item.j)) jData.push({ time: item.time, value: item.j });
    }

    kSeries.setData(kData);
    dSeries.setData(dData);
    jSeries.setData(jData);

    kdjKRef.current = kSeries;
    kdjDRef.current = dSeries;
    kdjJRef.current = jSeries;
    activeIndicatorsRef.current.add('kdj');
  }, []);

  const addRSI = useCallback((chart: IChartApi, rsiData: RSIData[]) => {
    const rsiSeries = chart.addSeries(LineSeries, {
      color: '#60a5fa',
      lineWidth: 1,
      title: 'RSI',
      priceScaleId: 'rsi',
    });
    const upperSeries = chart.addSeries(LineSeries, {
      color: '#f87171',
      lineWidth: 1,
      lineStyle: 2,
      title: '80',
      priceScaleId: 'rsi',
      lastValueVisible: false,
    });
    const lowerSeries = chart.addSeries(LineSeries, {
      color: '#34d399',
      lineWidth: 1,
      lineStyle: 2,
      title: '20',
      priceScaleId: 'rsi',
      lastValueVisible: false,
    });

    rsiSeries.priceScale().applyOptions({ scaleMargins: { top: 0.7, bottom: 0.05 } });

    const lineData: LineData[] = [];
    const upperData: LineData[] = [];
    const lowerData: LineData[] = [];

    for (const d of rsiData) {
      if (!Number.isNaN(d.value)) lineData.push({ time: d.time, value: d.value });
      upperData.push({ time: d.time, value: 80 });
      lowerData.push({ time: d.time, value: 20 });
    }

    rsiSeries.setData(lineData);
    upperSeries.setData(upperData);
    lowerSeries.setData(lowerData);

    rsiRef.current = rsiSeries;
    rsiUpperRef.current = upperSeries;
    rsiLowerRef.current = lowerSeries;
    activeIndicatorsRef.current.add('rsi');
  }, []);

  // Initialize chart
  useEffect(() => {
    if (!chartContainerRef.current) return;

    const chart = createChart(chartContainerRef.current, {
      layout: {
        background: { color: '#1e293b' },
        textColor: '#94a3b8',
      },
      grid: {
        vertLines: { color: '#334155' },
        horzLines: { color: '#334155' },
      },
      crosshair: {
        mode: 1,
        vertLine: {
          color: '#64748b',
          width: 1,
          style: 2,
          labelBackgroundColor: '#3b82f6',
        },
        horzLine: {
          color: '#64748b',
          width: 1,
          style: 2,
          labelBackgroundColor: '#3b82f6',
        },
      },
      rightPriceScale: {
        borderColor: '#334155',
        scaleMargins: {
          top: 0.1,
          bottom: 0.2,
        },
      },
      timeScale: {
        borderColor: '#334155',
        timeVisible: true,
        secondsVisible: false,
        tickMarkFormatter: (time: number) => {
          const date = new Date(time * 1000);
          return `${date.getMonth() + 1}/${date.getDate()}`;
        },
      },
      autoSize: true,
    });

    chartRef.current = chart;

    const candlestickSeries = chart.addSeries(CandlestickSeries, {
      upColor: '#f87171',
      downColor: '#34d399',
      borderVisible: false,
      wickUpColor: '#f87171',
      wickDownColor: '#34d399',
    });
    candlestickSeriesRef.current = candlestickSeries;

    const volumeSeries = chart.addSeries(HistogramSeries, {
      color: '#34d399',
      priceFormat: { type: 'volume' },
      priceScaleId: '',
    });
    volumeSeriesRef.current = volumeSeries;
    volumeSeries.priceScale().applyOptions({
      scaleMargins: { top: 0.8, bottom: 0 },
    });

    chart.subscribeCrosshairMove((param) => {
      if (!param.point || !param.time || !candlestickSeries) {
        setTooltip(null);
        return;
      }

      const dataPoint = param.seriesData.get(candlestickSeries) as CandlestickData;
      const volumeData = param.seriesData.get(volumeSeries) as HistogramData;

      if (dataPoint) {
        const indicatorValues: Record<string, number | undefined> = {};

        if (macdDifRef.current) {
          const v = param.seriesData.get(macdDifRef.current) as LineData | undefined;
          indicatorValues['DIF'] = v?.value;
        }
        if (macdDeaRef.current) {
          const v = param.seriesData.get(macdDeaRef.current) as LineData | undefined;
          indicatorValues['DEA'] = v?.value;
        }
        if (macdHistRef.current) {
          const v = param.seriesData.get(macdHistRef.current) as HistogramData | undefined;
          indicatorValues['MACD'] = v?.value;
        }
        if (kdjKRef.current) {
          const v = param.seriesData.get(kdjKRef.current) as LineData | undefined;
          indicatorValues['K'] = v?.value;
        }
        if (kdjDRef.current) {
          const v = param.seriesData.get(kdjDRef.current) as LineData | undefined;
          indicatorValues['D'] = v?.value;
        }
        if (kdjJRef.current) {
          const v = param.seriesData.get(kdjJRef.current) as LineData | undefined;
          indicatorValues['J'] = v?.value;
        }
        if (rsiRef.current) {
          const v = param.seriesData.get(rsiRef.current) as LineData | undefined;
          indicatorValues['RSI'] = v?.value;
        }

        setTooltip({
          open: dataPoint.open,
          high: dataPoint.high,
          low: dataPoint.low,
          close: dataPoint.close,
          volume: volumeData?.value || 0,
          date: String(param.time),
          indicatorValues: Object.keys(indicatorValues).length > 0 ? indicatorValues : undefined,
        });
      }
    });

    return () => {
      chart.remove();
      chartRef.current = null;
      activeIndicatorsRef.current.clear();
    };
  }, []);

  // Update candlestick + volume data
  useEffect(() => {
    if (!candlestickSeriesRef.current || !volumeSeriesRef.current || data.length === 0) return;

    const candleData: CandlestickData[] = data.map((item) => ({
      time: item.date,
      open: item.open,
      high: item.high,
      low: item.low,
      close: item.close,
    }));

    const volumeData: HistogramData[] = data.map((item) => ({
      time: item.date,
      value: item.volume,
      color: item.close >= item.open ? '#f87171' : '#34d399',
    }));

    candlestickSeriesRef.current.setData(candleData);
    volumeSeriesRef.current.setData(volumeData);
    chartRef.current?.timeScale().fitContent();
  }, [data]);

  // Update indicators
  useEffect(() => {
    const chart = chartRef.current;
    if (!chart || data.length === 0) return;

    const currentActive = activeIndicatorsRef.current;

    // Remove indicators no longer requested
    for (const ind of currentActive) {
      if (!indicators.includes(ind)) {
        removeIndicator(chart, ind);
      }
    }

    // Add new indicators
    for (const ind of indicators) {
      if (currentActive.has(ind)) continue;

      switch (ind) {
        case 'macd': {
          const macdData = calculateMACD(data);
          if (macdData.length > 0) addMACD(chart, macdData);
          break;
        }
        case 'kdj': {
          const kdjData = calculateKDJ(data);
          if (kdjData.length > 0) addKDJ(chart, kdjData);
          break;
        }
        case 'rsi': {
          const rsiData = calculateRSI(data);
          if (rsiData.length > 0) addRSI(chart, rsiData);
          break;
        }
      }
    }

    chart.timeScale().fitContent();
  }, [data, indicators, addMACD, addKDJ, addRSI, removeIndicator]);

  return (
    <div className="kline-chart-wrapper" style={{ position: 'relative' }}>
      <div ref={chartContainerRef} style={{ height: `${height}px`, width: '100%' }} />
      {tooltip && (
        <div className="kline-tooltip">
          <div className="kline-tooltip-date">{tooltip.date}</div>
          <div className="kline-tooltip-grid">
            <span className="kline-tooltip-label">开:</span>
            <span className={`kline-tooltip-value ${tooltip.open <= tooltip.close ? 'up' : 'down'}`}>
              {tooltip.open.toFixed(2)}
            </span>
            <span className="kline-tooltip-label">高:</span>
            <span className="kline-tooltip-value">{tooltip.high.toFixed(2)}</span>
            <span className="kline-tooltip-label">低:</span>
            <span className="kline-tooltip-value">{tooltip.low.toFixed(2)}</span>
            <span className="kline-tooltip-label">收:</span>
            <span className={`kline-tooltip-value ${tooltip.close >= tooltip.open ? 'up' : 'down'}`}>
              {tooltip.close.toFixed(2)}
            </span>
            <span className="kline-tooltip-label">量:</span>
            <span className="kline-tooltip-value">{(tooltip.volume / 10000).toFixed(2)}万</span>
          </div>
          {tooltip.indicatorValues && Object.keys(tooltip.indicatorValues).length > 0 && (
            <div className="kline-tooltip-indicators">
              {Object.entries(tooltip.indicatorValues).map(([key, value]) =>
                value !== undefined ? (
                  <div key={key} className="kline-tooltip-grid">
                    <span className="kline-tooltip-label">{key}:</span>
                    <span className="kline-tooltip-value">{value.toFixed(3)}</span>
                  </div>
                ) : null
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
