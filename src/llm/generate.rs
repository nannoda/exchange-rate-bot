use std::time::Duration;

use crate::environment::{self, get_system_prompt};
use crate::llm::prompt::{get_date_prompt, get_news_prompt};
use chrono::{DateTime, Utc};
use reqwest;
use serde_json::Value;
use tokio::join;

use crate::database;
use crate::environment::get_ollama_model;

pub struct GenerationResult {
    content: String,
    search_duration: Duration,
    total_duration: Duration,
    load_duration: Duration,
    prompt_eval_duration: Duration,
    eval_duration: Duration,
}

/// Generate sentence using language model
pub async fn generate_sentence(user_prompt: &str) -> String {
    let base_url = environment::get_ollama_url();

    let datetime = Utc::now();

    let date_prompt_fut = get_date_prompt(datetime);
    let news_prompt_fut = get_news_prompt(datetime);
    let (date_prompt, news_prompt) = join!(date_prompt_fut, news_prompt_fut);

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
            return format!(
                "Error generating response: Failed to create a client ({})",
                e
            );
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
            return format!("Error generating response: Failed to send request ({})", e);
        }
    };

    let text = match res.text().await {
        Ok(body) => body,
        Err(e) => {
            log::error!("Failed to get response text: {}", e);
            return format!(
                "Error generating response: Failed to retrieve response text ({})",
                e
            );
        }
    };

    database::llm_result::save_llm_result(user_prompt, &text);

    log::debug!("text: {}", &text);

    let response: Value = match serde_json::from_str(&text) {
        Ok(parsed) => parsed,
        Err(e) => {
            log::error!("Failed to parse JSON response: {}", e);
            return format!("Error generating response: Failed to parse JSON ({})", e);
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
    return content;
}
