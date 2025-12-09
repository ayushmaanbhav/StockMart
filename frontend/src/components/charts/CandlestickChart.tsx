// ============================================
// Candlestick Chart Component
// Using lightweight-charts library
// ============================================

import React, { useEffect, useRef } from 'react';
import { createChart, ColorType } from 'lightweight-charts';
import type { IChartApi, ISeriesApi, CandlestickData, Time } from 'lightweight-charts';
import { useGameStore } from '../../store/gameStore';
import { useUIStore } from '../../store/uiStore';

interface CandlestickChartProps {
    symbol?: string;
    height?: number;
}

export const CandlestickChart: React.FC<CandlestickChartProps> = ({
    symbol,
    height = 300
}) => {
    const containerRef = useRef<HTMLDivElement>(null);
    const chartRef = useRef<IChartApi | null>(null);
    const seriesRef = useRef<ISeriesApi<'Candlestick'> | null>(null);
    const volumeSeriesRef = useRef<ISeriesApi<'Histogram'> | null>(null);

    const { activeSymbol, candles } = useGameStore();
    const { theme } = useUIStore();

    const currentSymbol = symbol || activeSymbol;
    const candleData = candles[currentSymbol] || [];

    // Theme colors
    const isDark = theme === 'dark';
    const chartColors = {
        background: isDark ? '#0a0a0f' : '#ffffff',
        textColor: isDark ? '#a1a1aa' : '#52525b',
        gridColor: isDark ? '#1a1a26' : '#f4f4f5',
        upColor: '#22c55e',
        downColor: '#ef4444',
        borderUpColor: '#22c55e',
        borderDownColor: '#ef4444',
        wickUpColor: '#22c55e',
        wickDownColor: '#ef4444',
    };

    // Initialize chart
    useEffect(() => {
        if (!containerRef.current) return;

        // Create chart
        const chart = createChart(containerRef.current, {
            width: containerRef.current.clientWidth,
            height,
            layout: {
                background: { type: ColorType.Solid, color: chartColors.background },
                textColor: chartColors.textColor,
            },
            grid: {
                vertLines: { color: chartColors.gridColor },
                horzLines: { color: chartColors.gridColor },
            },
            crosshair: {
                mode: 0, // Normal
                vertLine: {
                    width: 1,
                    color: isDark ? '#3f3f46' : '#d4d4d8',
                    style: 2,
                },
                horzLine: {
                    width: 1,
                    color: isDark ? '#3f3f46' : '#d4d4d8',
                    style: 2,
                },
            },
            rightPriceScale: {
                borderColor: chartColors.gridColor,
            },
            timeScale: {
                borderColor: chartColors.gridColor,
                timeVisible: true,
                secondsVisible: false,
            },
        });

        // Add candlestick series
        const candleSeries = chart.addCandlestickSeries({
            upColor: chartColors.upColor,
            downColor: chartColors.downColor,
            borderUpColor: chartColors.borderUpColor,
            borderDownColor: chartColors.borderDownColor,
            wickUpColor: chartColors.wickUpColor,
            wickDownColor: chartColors.wickDownColor,
        });

        // Add volume series
        const volumeSeries = chart.addHistogramSeries({
            color: '#6366f1',
            priceFormat: {
                type: 'volume',
            },
            priceScaleId: '',
        });

        volumeSeries.priceScale().applyOptions({
            scaleMargins: {
                top: 0.8,
                bottom: 0,
            },
        });

        chartRef.current = chart;
        seriesRef.current = candleSeries;
        volumeSeriesRef.current = volumeSeries;

        // Handle resize
        const handleResize = () => {
            if (containerRef.current && chartRef.current) {
                chartRef.current.applyOptions({
                    width: containerRef.current.clientWidth,
                });
            }
        };

        window.addEventListener('resize', handleResize);

        return () => {
            window.removeEventListener('resize', handleResize);
            chart.remove();
            chartRef.current = null;
            seriesRef.current = null;
            volumeSeriesRef.current = null;
        };
    }, [height]);

    // Update theme
    useEffect(() => {
        if (!chartRef.current) return;

        chartRef.current.applyOptions({
            layout: {
                background: { type: ColorType.Solid, color: chartColors.background },
                textColor: chartColors.textColor,
            },
            grid: {
                vertLines: { color: chartColors.gridColor },
                horzLines: { color: chartColors.gridColor },
            },
        });
    }, [theme, chartColors.background, chartColors.textColor, chartColors.gridColor]);

    // Update data
    useEffect(() => {
        if (!seriesRef.current || !volumeSeriesRef.current) return;

        if (candleData.length === 0) {
            // Clear chart when no data - show empty state
            seriesRef.current.setData([]);
            volumeSeriesRef.current.setData([]);
        } else {
            // Use real data
            const chartCandles: CandlestickData[] = candleData.map(c => ({
                time: (c.timestamp / 1000) as Time,
                open: c.open,
                high: c.high,
                low: c.low,
                close: c.close,
            }));

            const volumes = candleData.map(c => ({
                time: (c.timestamp / 1000) as Time,
                value: c.volume,
                color: c.close >= c.open
                    ? 'rgba(34, 197, 94, 0.5)'
                    : 'rgba(239, 68, 68, 0.5)',
            }));

            seriesRef.current.setData(chartCandles);
            volumeSeriesRef.current.setData(volumes);
        }

        // Fit content
        if (chartRef.current) {
            chartRef.current.timeScale().fitContent();
        }
    }, [candleData, currentSymbol]);

    return (
        <div className="chart-wrapper" style={{ position: 'relative', width: '100%', height }}>
            <div
                ref={containerRef}
                className="chart-container"
                style={{ width: '100%', height }}
            />
            {candleData.length === 0 && (
                <div
                    className="chart-empty-state"
                    style={{
                        position: 'absolute',
                        top: 0,
                        left: 0,
                        right: 0,
                        bottom: 0,
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        background: 'rgba(0,0,0,0.5)',
                        color: 'var(--text-muted)',
                        fontSize: '14px',
                        pointerEvents: 'none'
                    }}
                >
                    <div style={{ textAlign: 'center' }}>
                        <div style={{ marginBottom: '8px' }}>No chart data available</div>
                        <div style={{ fontSize: '12px', opacity: 0.7 }}>
                            Trades will generate candles
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};

export default CandlestickChart;
