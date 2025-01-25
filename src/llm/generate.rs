use std::time::Duration;

use crate::environment::{self, get_system_prompt};
use crate::llm::prompt::{get_date_prompt, get_news_prompt};
use chrono::Utc;
use reqwest;
use serde_json::Value;
use tokio::join;

use crate::database;
use crate::environment::get_ollama_model;

pub struct GenerationResult {
    pub content: String,
    pub search_duration: Duration,
    pub total_duration: Duration,
    pub load_duration: Duration,
    pub prompt_eval_duration: Duration,
    pub eval_duration: Duration,
}

impl GenerationResult {
    pub fn error(e: String) -> GenerationResult {
        let error_duration = Duration::from_nanos(0);
        return GenerationResult {
            content: e,
            search_duration: error_duration,
            total_duration: error_duration,
            load_duration: error_duration,
            prompt_eval_duration: error_duration,
            eval_duration: error_duration,
        };
    }
}

/// Generate sentence using language model
pub async fn generate_sentence(user_prompt: &str) -> GenerationResult {
    let base_url = environment::get_ollama_url();

    let datetime = Utc::now();
    let search_start = std::time::Instant::now();

    let date_prompt_fut = get_date_prompt(datetime);
    let news_prompt_fut = get_news_prompt(datetime);
    let (date_prompt, news_prompt) = join!(date_prompt_fut, news_prompt_fut);
    let search_duration = search_start.elapsed();

    let mut messages = vec![];

    if !news_prompt.is_empty() {
        messages.push(serde_json::json!({
            "role": "system",
            "content": news_prompt
        }));
    }

    if !date_prompt.is_empty() {
        messages.push(serde_json::json!({
            "role": "system",
            "content": date_prompt
        }));
    }

    messages.push(serde_json::json!({
        "role": "system",
        "content": get_system_prompt()
    }));

    messages.push(serde_json::json!({
        "role": "user",
        "content": user_prompt
    }));

    let url = base_url + "/api/chat";
    let json = serde_json::json!({
        "model": get_ollama_model(),
        "messages": messages,
        "stream": false
    });

    let json_string = json.to_string();
    log::debug!("json_string: {}", json_string);

    let client = match reqwest::ClientBuilder::new()
        .timeout(Duration::new(60 * 5, 0))
        .build()
    {
        Ok(client) => client,
        Err(e) => {
            log::error!("Failed to create a client: {}", e);
            return GenerationResult::error(format!(
                "Error generating response: Failed to create a client ({})",
                e
            ));
        }
    };
    let res = match client
        .post(url)
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .body(json_string)
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            log::error!("Failed to send request: {}", e);
            return GenerationResult::error(format!(
                "Error generating response: Failed to send request ({})",
                e
            ));
        }
    };

    let text = match res.text().await {
        Ok(body) => body,
        Err(e) => {
            log::error!("Failed to get response text: {}", e);
            return GenerationResult::error(format!(
                "Error generating response: Failed to retrieve response text ({})",
                e
            ));
        }
    };

    database::llm_result::save_llm_result(user_prompt, &text);

    log::debug!("text: {}", &text);

    let response: Value = match serde_json::from_str(&text) {
        Ok(parsed) => parsed,
        Err(e) => {
            log::error!("Failed to parse JSON response: {}", e);
            return GenerationResult::error(format!(
                "Error generating response: Failed to parse JSON ({})",
                e
            ));
        }
    };

    let content = response["message"]["content"]
        .as_str()
        .and_then(|content| Some(content.to_string()))
        .unwrap_or(format!(
            "Error generating response: fail to find [message][content]\nRaw JSON:\n```{}```",
            &text
        ));

    let total_duration = Duration::from_nanos(response["total_duration"].as_u64().unwrap_or(0));
    let load_duration = Duration::from_nanos(response["load_duration"].as_u64().unwrap_or(0));
    let prompt_eval_duration =
        Duration::from_nanos(response["prompt_eval_duration"].as_u64().unwrap_or(0));
    let eval_duration = Duration::from_nanos(response["eval_duration"].as_u64().unwrap_or(0));
    return GenerationResult {
        content,
        search_duration,
        total_duration,
        load_duration,
        prompt_eval_duration,
        eval_duration,
    };
}
