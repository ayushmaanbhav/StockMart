//! News Service
//!
//! Generates simulated market news for the trading game.
//! Uses actual company symbols from the repository.

use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

use crate::domain::CompanyRepository;

/// News generation interval in seconds (configurable via NEWS_INTERVAL_SECS env var)
fn news_interval_secs() -> u64 {
    std::env::var("NEWS_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub id: String,
    pub headline: String,
    pub sentiment: String, // "Bullish", "Bearish", "Neutral"
    pub impact: String,    // "high", "medium", "low"
    pub symbol: Option<String>,
    pub timestamp: i64,
}

pub struct NewsService {
    news_tx: broadcast::Sender<NewsItem>,
    /// Recent news items for state sync
    recent_news: RwLock<VecDeque<NewsItem>>,
    /// Company repository to get actual symbols
    company_repo: Arc<dyn CompanyRepository>,
    /// Cached company symbols (refreshed periodically)
    cached_symbols: RwLock<Vec<String>>,
}

impl NewsService {
    pub fn new(company_repo: Arc<dyn CompanyRepository>) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            news_tx: tx,
            recent_news: RwLock::new(VecDeque::with_capacity(50)),
            company_repo,
            cached_symbols: RwLock::new(Vec::new()),
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<NewsItem> {
        self.news_tx.subscribe()
    }

    /// Get recent news items for state sync
    pub fn get_recent(&self, count: usize) -> Vec<NewsItem> {
        let news = self.recent_news.read().unwrap();
        news.iter().rev().take(count).cloned().collect()
    }

    /// Refresh the cached company symbols from the repository
    async fn refresh_symbols(&self) {
        if let Ok(companies) = self.company_repo.all().await {
            let symbols: Vec<String> = companies
                .into_iter()
                .filter(|c| !c.bankrupt) // Only non-bankrupt companies
                .map(|c| c.symbol)
                .collect();
            *self.cached_symbols.write().unwrap() = symbols;
        }
    }

    pub async fn run(&self) {
        let mut id_counter = 1u64;
        let interval = news_interval_secs();
        tracing::info!("News service started with {}s interval", interval);

        loop {
            sleep(Duration::from_secs(interval)).await;

            // Refresh symbols periodically (every news cycle)
            self.refresh_symbols().await;

            // Only generate news if we have companies
            let symbols = self.cached_symbols.read().unwrap().clone();
            if symbols.is_empty() {
                tracing::debug!("No companies available for news generation");
                continue;
            }

            let news = self.generate_news(id_counter, &symbols);
            id_counter += 1;

            // Store in recent news
            {
                let mut recent = self.recent_news.write().unwrap();
                if recent.len() >= 50 {
                    recent.pop_front();
                }
                recent.push_back(news.clone());
            }

            let _ = self.news_tx.send(news);
        }
    }

    fn generate_news(&self, id: u64, symbols: &[String]) -> NewsItem {
        let mut rng = rand::thread_rng();
        let symbol = symbols[rng.gen_range(0..symbols.len())].clone();

        let sentiments = ["Bullish", "Bearish", "Neutral"];
        let sentiment = sentiments[rng.gen_range(0..sentiments.len())].to_string();

        let impacts = ["high", "medium", "low"];
        let impact = impacts[rng.gen_range(0..impacts.len())].to_string();

        let headlines = match sentiment.as_str() {
            "Bullish" => vec![
                format!("{} beats earnings expectations!", symbol),
                format!("Analysts upgrade {} to Buy", symbol),
                format!("{} announces new breakthrough product", symbol),
                format!("Institutional investors loading up on {}", symbol),
                format!("{} reports record quarterly revenue", symbol),
                format!("{} expands into new markets", symbol),
            ],
            "Bearish" => vec![
                format!("{} misses revenue targets", symbol),
                format!("Regulatory concerns hit {}", symbol),
                format!("{} CEO sells shares", symbol),
                format!("Supply chain issues plague {}", symbol),
                format!("{} faces increased competition", symbol),
                format!("Analysts downgrade {} outlook", symbol),
            ],
            _ => vec![
                format!("{} to hold shareholder meeting", symbol),
                format!("Market awaits {} earnings report", symbol),
                format!("{} announces minor partnership", symbol),
                format!("{} maintains steady growth trajectory", symbol),
                format!("{} trading at key technical levels", symbol),
            ],
        };

        let headline = headlines[rng.gen_range(0..headlines.len())].clone();

        NewsItem {
            id: format!("news_{}", id),
            headline,
            sentiment,
            impact,
            symbol: Some(symbol),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    /// Exposed for testing
    #[cfg(test)]
    pub fn test_generate_news(&self, id: u64, symbols: &[String]) -> NewsItem {
        self.generate_news(id, symbols)
    }

    /// Store a news item manually (for testing)
    #[cfg(test)]
    pub fn test_add_news(&self, news: NewsItem) {
        let mut recent = self.recent_news.write().unwrap();
        if recent.len() >= 50 {
            recent.pop_front();
        }
        recent.push_back(news);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::persistence::InMemoryCompanyRepository;

    fn create_test_service() -> NewsService {
        let repo: Arc<dyn CompanyRepository> = Arc::new(InMemoryCompanyRepository::new());
        NewsService::new(repo)
    }

    #[test]
    fn test_news_service_new() {
        let svc = create_test_service();
        // Should be able to subscribe
        let _rx = svc.subscribe();
    }

    #[test]
    fn test_get_recent_empty() {
        let svc = create_test_service();
        assert!(svc.get_recent(10).is_empty());
    }

    #[test]
    fn test_generate_news_bullish() {
        let svc = create_test_service();
        let symbols = vec!["AAPL".to_string(), "GOOGL".to_string()];

        // Generate multiple news items to cover all sentiments
        for i in 0..20 {
            let news = svc.test_generate_news(i, &symbols);
            assert!(news.symbol.is_some());
            assert!(symbols.contains(&news.symbol.clone().unwrap()));
            assert!(!news.headline.is_empty());
            assert!(["Bullish", "Bearish", "Neutral"].contains(&news.sentiment.as_str()));
            assert!(["high", "medium", "low"].contains(&news.impact.as_str()));
            assert_eq!(news.id, format!("news_{}", i));
        }
    }

    #[test]
    fn test_add_and_get_recent() {
        let svc = create_test_service();

        // Add some news
        for i in 0..5 {
            let news = NewsItem {
                id: format!("news_{}", i),
                headline: format!("Headline {}", i),
                sentiment: "Neutral".to_string(),
                impact: "medium".to_string(),
                symbol: Some("TEST".to_string()),
                timestamp: chrono::Utc::now().timestamp() + i as i64,
            };
            svc.test_add_news(news);
        }

        let recent = svc.get_recent(3);
        assert_eq!(recent.len(), 3);
        // Should be in reverse order (most recent first)
        assert_eq!(recent[0].id, "news_4");
        assert_eq!(recent[1].id, "news_3");
        assert_eq!(recent[2].id, "news_2");
    }

    #[test]
    fn test_news_capacity_limit() {
        let svc = create_test_service();

        // Add 55 news items (capacity is 50)
        for i in 0..55 {
            let news = NewsItem {
                id: format!("news_{}", i),
                headline: format!("Headline {}", i),
                sentiment: "Neutral".to_string(),
                impact: "medium".to_string(),
                symbol: Some("TEST".to_string()),
                timestamp: chrono::Utc::now().timestamp() + i as i64,
            };
            svc.test_add_news(news);
        }

        let recent = svc.get_recent(100);
        assert_eq!(recent.len(), 50); // Capped at 50
                                      // Oldest should be news_5 (0-4 were dropped)
        assert_eq!(recent.last().unwrap().id, "news_5");
    }

    #[test]
    fn test_news_item_serialization() {
        let news = NewsItem {
            id: "news_1".to_string(),
            headline: "Test headline".to_string(),
            sentiment: "Bullish".to_string(),
            impact: "high".to_string(),
            symbol: Some("AAPL".to_string()),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&news).unwrap();
        assert!(json.contains("news_1"));
        assert!(json.contains("Test headline"));
        assert!(json.contains("Bullish"));
        assert!(json.contains("high"));
        assert!(json.contains("AAPL"));

        let deserialized: NewsItem = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, news.id);
        assert_eq!(deserialized.headline, news.headline);
    }

    #[test]
    fn test_news_interval_default() {
        // Without env var, should default to 30
        let interval = news_interval_secs();
        assert!(interval > 0);
    }
}
