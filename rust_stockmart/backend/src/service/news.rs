use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use rand::Rng;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub id: u64,
    pub headline: String,
    pub sentiment: String, // "Bullish", "Bearish", "Neutral"
    pub symbol: Option<String>,
    pub timestamp: i64,
}

pub struct NewsService {
    news_tx: broadcast::Sender<NewsItem>,
}

impl NewsService {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            news_tx: tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<NewsItem> {
        self.news_tx.subscribe()
    }

    pub async fn run(&self) {
        let mut id_counter = 1;
        loop {
            // Generate news every 30 seconds
            sleep(Duration::from_secs(30)).await;
            
            let news = self.generate_news(id_counter);
            id_counter += 1;

            let _ = self.news_tx.send(news);
        }
    }

    fn generate_news(&self, id: u64) -> NewsItem {
        let mut rng = rand::thread_rng();
        let symbols = vec!["AAPL", "GOOGL", "AMZN", "MSFT", "TSLA"];
        let symbol = symbols[rng.gen_range(0..symbols.len())].to_string();
        
        let sentiments = vec!["Bullish", "Bearish", "Neutral"];
        let sentiment = sentiments[rng.gen_range(0..sentiments.len())].to_string();

        let headlines = match sentiment.as_str() {
            "Bullish" => vec![
                format!("{} beats earnings expectations!", symbol),
                format!("Analysts upgrade {} to Buy", symbol),
                format!("{} announces new breakthrough product", symbol),
                format!("Institutional investors loading up on {}", symbol),
            ],
            "Bearish" => vec![
                format!("{} misses revenue targets", symbol),
                format!("Regulatory concerns hit {}", symbol),
                format!("{} CEO sells shares", symbol),
                format!("Supply chain issues plague {}", symbol),
            ],
            _ => vec![
                format!("{} to hold shareholder meeting", symbol),
                format!("Market awaits {} earnings report", symbol),
                format!("{} announces minor partnership", symbol),
            ],
        };

        let headline = headlines[rng.gen_range(0..headlines.len())].clone();

        NewsItem {
            id,
            headline,
            sentiment,
            symbol: Some(symbol),
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}
