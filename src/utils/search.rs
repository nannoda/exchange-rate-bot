use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::{environment::get_searxng_url, main, utils::parse};

pub enum SearchCategory {
    General,
    News,
}

pub async fn search(keyword: &str, category: SearchCategory) -> Option<Value> {
    let url = match get_searxng_url() {
        Some(url) => url,
        None => return None,
    };

    let query_url = format!(
        "{}/search?q={}&format=json&safesearch=0&categories={}",
        url,
        keyword,
        match category {
            SearchCategory::General => "general",
            SearchCategory::News => "news",
        }
    );

    log::debug!("Search Query: {query_url}");

    let res = match reqwest::get(&query_url).await {
        Ok(res) => res,
        Err(err) => {
            log::error!("Error fetching '{}': {}", query_url, err);
            return None;
        }
    };

    if !res.status().is_success() {
        log::error!("Error fetching '{}': {}", query_url, res.status());
        return None;
    }
    // Safely attempt to read the response body
    let text = match res.text().await {
        Ok(txt) => txt,
        Err(e) => {
            log::error!("Error getting text {e}");
            return None;
        }
    };

    let parsed: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            log::error!("Error parse json {e}");
            return None;
        }
    };

    return Some(parsed);
}

pub struct SearchResult {
    url: Option<String>,
    title: Option<String>,
    content: Option<String>,
}

pub async fn get_news(datetime: DateTime<Utc>, top: Option<u8>) -> Vec<SearchCategory> {
    let new_list = vec![];
    let v = match search(
        &format!("news on {}", datetime.format("%Y-%m-%d")),
        SearchCategory::News,
    )
    .await
    {
        Some(v) => v,
        None => return new_list,
    };

    let results = match v.get("results").and_then(|v| v.as_object()) {
        Some(r) => r,
        None => {
            log::error!("No result");
            return new_list;
        }
    };

    for n in 0..top.unwrap_or(100) {
        
    }

    return new_list;
}
