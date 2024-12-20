use std::time::Duration;

use crate::environment::{self, get_system_prompt};
use reqwest;
use serde_json::Value;

use crate::database;
use crate::environment::get_ollama_model;

/// Generate sentence using language model
pub async fn generate_sentence(user_prompt: &str) -> String {
    let base_url = environment::get_ollama_url();

    let url = base_url + "/api/chat";
    let json = serde_json::json!({
        "model": get_ollama_model(),
        "messages": [
            {
                "role": "system",
                "content": get_system_prompt()
            },
            {
                "role": "user",
                "content": user_prompt
            }
        ],
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
            return format!("Error generating response: Failed to create a client ({})", e);
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

    return content;
}
