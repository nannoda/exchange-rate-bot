use crate::environment::{self, get_system_prompt};
use reqwest;
use serde_json::Value;

use crate::environment::get_ollama_model;
use crate::utils;

/// Generate sentence using language model
///
/// Currently using Cohere API
pub async fn generate_sentence(start: &str) -> String {
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
                "content": start
            }
        ],
        "stream": false
    });

    let json_string = json.to_string();
    log::debug!("json_string: {}", json_string);

    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .body(json_string)
        .send()
        .await;

    let text = res.unwrap().text().await.unwrap();

    utils::save_llm_result(start, &text);

    log::debug!("text: {}", text);

    let response: Value = serde_json::from_str(&text).unwrap();

    // let text = response["text"].as_str().unwrap();
    let text = response["message"]["content"]
        .as_str()
        .unwrap_or("ERROR GENERATING MESSAGE");

    return text.to_string();
}
