use std::sync::BarrierWaitResult;

use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::{environment::get_searxng_url, main, utils::parse};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};

pub enum SearchCategory {
    General,
    News,
}

pub async fn search(keyword: &str, category: SearchCategory) -> Option<Value> {
    let url = match get_searxng_url() {
        Some(url) => url,
        None => return None,
    };

    // URL-encode the keyword to make it web-safe
    let encoded_keyword = utf8_percent_encode(keyword, NON_ALPHANUMERIC).to_string();

    let query_url = format!(
        "{}/search?q={}&format=json&safesearch=0&categories={}",
        url,
        encoded_keyword,
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

#[derive(Debug)]
pub struct SearchResult {
    url: Option<String>,
    title: Option<String>,
    content: Option<String>,
}

pub async fn get_news(datetime: DateTime<Utc>, max: u8) -> Vec<SearchResult> {
    let mut new_list = vec![];
    let v = match search(
        &format!("news on {}", datetime.format("%Y %m %d")),
        SearchCategory::News,
    )
    .await
    {
        Some(v) => v,
        None => return new_list,
    };

    let results = match v.get("results").and_then(|v| v.as_array()) {
        Some(r) => r,
        None => {
            log::error!("No result");
            return new_list;
        }
    };

    for i in 0..max {
        if (usize::from(i) > results.len()) {
            break;
        }

        let item = match results.get(usize::from(i)) {
            Some(i) => i,
            None => {
                log::debug!("[{i}] doesn't exist.");
                continue;
            }
        };

        let url = item
            .get("url")
            .and_then(|v| v.as_str())
            .and_then(|s| Some(s.to_owned()));
        let title = item
            .get("title")
            .and_then(|v| v.as_str())
            .and_then(|s| Some(s.to_owned()));
        let content = item
            .get("content")
            .and_then(|v| v.as_str())
            .and_then(|s| Some(s.to_owned()));

        new_list.push(SearchResult {
            url,
            title,
            content,
        });
    }

    return new_list;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    // Initialize the logger for the tests
    fn init_logger() {
        let _ = env_logger::builder()
            .is_test(true) // Ensures logs are only displayed during tests
            .filter_level(log::LevelFilter::Debug) // Set the log level to debug
            .try_init();
    }

    #[tokio::test]
    async fn test_search_general() {
        // Initialize the logger
        init_logger();
        // Mock the SearxNG URL environment variable
        std::env::set_var("SEARXNG_URL", "https://searxng.nannoda.com");

        // Call the function
        let result = search("example keyword", SearchCategory::General).await;

        println!("result: {:?}", result);

        // Assert the result
        assert!(result.is_some(), "Search should return a result");
    }

    #[tokio::test]
    async fn test_get_news() {
        // Initialize the logger
        init_logger();
        // Mock the SearxNG URL environment variable
        std::env::set_var("SEARXNG_URL", "https://searxng.nannoda.com");
        // Mock a datetime value
        let datetime = Utc::now();

        // Call the function
        let results = get_news(datetime, 5).await;

        println!("result: {:?}", results);

        // Assert the results
        assert!(
            !results.is_empty(),
            "get_news should return results (mocked case)"
        );
    }
}
