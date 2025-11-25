import { useEffect, useRef } from 'react';
import { createChart, ColorType } from 'lightweight-charts';
import type { IChartApi, ISeriesApi, CandlestickData } from 'lightweight-charts';
import { useStore } from '../../store/useStore';

export const Chart = () => {
    const chartContainerRef = useRef<HTMLDivElement>(null);
    const chartRef = useRef<IChartApi | null>(null);
    const seriesRef = useRef<ISeriesApi<"Candlestick"> | null>(null);
    const { candles, activeSymbol } = useStore();

    useEffect(() => {
        if (!chartContainerRef.current) return;

        const chart = createChart(chartContainerRef.current, {
            layout: {
                background: { type: ColorType.Solid, color: '#1f2937' }, // gray-800
                textColor: '#d1d5db',
            },
            grid: {
                vertLines: { color: '#374151' },
                horzLines: { color: '#374151' },
            },
            width: chartContainerRef.current.clientWidth,
            height: 400,
        });

        const newSeries = (chart as any).addCandlestickSeries({
            upColor: '#22c55e',
            downColor: '#ef4444',
            borderVisible: false,
            wickUpColor: '#22c55e',
            wickDownColor: '#ef4444',
        });

        chartRef.current = chart;
        seriesRef.current = newSeries;

        const handleResize = () => {
            if (chartContainerRef.current) {
                chart.applyOptions({ width: chartContainerRef.current.clientWidth });
            }
        };

        window.addEventListener('resize', handleResize);

        return () => {
            window.removeEventListener('resize', handleResize);
            chart.remove();
        };
    }, []);

    useEffect(() => {
        if (seriesRef.current && candles[activeSymbol]) {
            // Sort candles by time to ensure monotonic
            const sortedCandles = [...candles[activeSymbol]].sort((a, b) => a.time - b.time);

            // Convert to lightweight-charts format
            // Our timestamp is in seconds, which lightweight-charts expects
            const data: CandlestickData[] = sortedCandles.map(c => ({
                time: c.time as any, // lightweight-charts types are strict about Time
                open: c.open / 10000,
                high: c.high / 10000,
                low: c.low / 10000,
                close: c.close / 10000,
            }));

            seriesRef.current.setData(data);
        }
    }, [candles, activeSymbol]);

    return (
        <div className="w-full bg-gray-800 p-4 rounded-xl shadow-lg">
            <h2 className="text-xl font-semibold mb-4">{activeSymbol} Chart</h2>
            <div ref={chartContainerRef} className="w-full h-[400px]" />
        </div>
    );
};
