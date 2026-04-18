import { IndicatorType } from '../utils/indicators';

interface IndicatorPanelProps {
  activeIndicators: IndicatorType[];
  onToggle: (indicator: IndicatorType) => void;
}

const INDICATOR_CONFIG: { key: IndicatorType; label: string }[] = [
  { key: 'macd', label: 'MACD' },
  { key: 'kdj', label: 'KDJ' },
  { key: 'rsi', label: 'RSI' },
];

export function IndicatorPanel({ activeIndicators, onToggle }: IndicatorPanelProps) {
  return (
    <div className="indicator-panel">
      <span className="indicator-label">指标:</span>
      {INDICATOR_CONFIG.map((ind) => (
        <button
          key={ind.key}
          className={`indicator-btn ${activeIndicators.includes(ind.key) ? 'active' : ''}`}
          onClick={() => onToggle(ind.key)}
        >
          {ind.label}
        </button>
      ))}
    </div>
  );
}
