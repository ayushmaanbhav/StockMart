// ============================================
// News Ticker Component
// Animated marquee for news headlines
// ============================================

import React, { useMemo } from 'react';
import { Newspaper } from 'lucide-react';
import { useGameStore } from '../../../store/gameStore';
import { ANIMATION } from '../../../constants';

export const NewsTicker: React.FC = () => {
    const { news, companies } = useGameStore();

    const getSentimentColor = (sentiment: string) => {
        switch (sentiment) {
            case 'Bullish': return 'var(--color-success)';
            case 'Bearish': return 'var(--color-danger)';
            default: return 'var(--text-muted)';
        }
    };

    // Memoize stock symbols for highlighting
    const stockSymbols = useMemo(() => companies.map(c => c.symbol), [companies]);

    const keywordPatterns = useMemo(() => [
        ...stockSymbols,
        'bullish', 'bearish', 'rally', 'crash', 'surge', 'plunge',
        'profit', 'loss', 'growth', 'decline', 'buy', 'sell'
    ], [stockSymbols]);

    // Highlight keywords in headline
    const highlightHeadline = (headline: string) => {
        const regex = new RegExp(`\\b(${keywordPatterns.join('|')})\\b`, 'gi');
        const parts = headline.split(regex);

        return parts.map((part, i) => {
            const lowerPart = part.toLowerCase();
            const isSymbol = stockSymbols.some(s => s.toLowerCase() === lowerPart);
            const isBullish = ['bullish', 'rally', 'surge', 'profit', 'growth', 'buy'].includes(lowerPart);
            const isBearish = ['bearish', 'crash', 'plunge', 'loss', 'decline', 'sell'].includes(lowerPart);

            if (isSymbol) {
                return <span key={i} className="news-keyword">{part}</span>;
            } else if (isBullish) {
                return <span key={i} className="news-keyword bullish">{part}</span>;
            } else if (isBearish) {
                return <span key={i} className="news-keyword bearish">{part}</span>;
            }
            return part;
        });
    };

    // Memoize duplicated news for seamless infinite scroll
    const duplicatedNews = useMemo(() => [...news, ...news], [news]);

    // Memoize animation duration based on number of items
    const animationDuration = useMemo(() =>
        Math.max(ANIMATION.NEWS_TICKER_BASE_DURATION, news.length * ANIMATION.NEWS_TICKER_PER_ITEM),
        [news.length]
    );

    return (
        <div className="news-ticker">
            <div className="news-label">
                <Newspaper size={14} />
                <span>NEWS</span>
            </div>
            <div className="news-content">
                {news.length === 0 ? (
                    <span className="text-muted" style={{ padding: '0 var(--space-2)' }}>Waiting for news...</span>
                ) : (
                    <div
                        className="news-scroll"
                        style={{ animationDuration: `${animationDuration}s` }}
                    >
                        {duplicatedNews.map((item, i) => (
                            <span key={`${item.id}-${i}`} className="news-item">
                                <span className="sentiment-dot" style={{ background: getSentimentColor(item.sentiment) }} />
                                {highlightHeadline(item.headline)}
                            </span>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
};

export default NewsTicker;
