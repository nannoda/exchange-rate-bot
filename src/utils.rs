use std::borrow::Borrow;

use log::warn;
use rusqlite::Connection;
use serde_json::Value;
use serenity::all::CreateMessage;

use crate::{
    environment::{self, get_exchange_rate_api_url},
    exchange_rate::{get_exchange_rates, ExchangeRateMap},
    llm::{generate::generate_sentence, prompt::get_prompt},
};

const DEFAULT_TIME_SECONDS: u64 = 86400;

pub fn string_to_time_second(s: &str) -> u64 {
    // convert string to lower case
    let s = s.to_lowercase();

    // if string ends with 's', remove it and convert to integer
    if s.ends_with("s") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value,
            Err(_) => {
                warn!(
                    "Failed to parse seconds from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if string ends with 'm', remove it and convert to integer
    else if s.ends_with("m") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value * 60,
            Err(_) => {
                warn!(
                    "Failed to parse minutes from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if string ends with 'h', remove it and convert to integer
    else if s.ends_with("h") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value * 60 * 60,
            Err(_) => {
                warn!(
                    "Failed to parse hours from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if string ends with 'd', remove it and convert to integer
    else if s.ends_with("d") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value * 60 * 60 * 24,
            Err(_) => {
                warn!(
                    "Failed to parse days from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if string ends with 'w', remove it and convert to integer
    else if s.ends_with("w") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value * 60 * 60 * 24 * 7,
            Err(_) => {
                warn!(
                    "Failed to parse weeks from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if string ends with 'y', remove it and convert to integer
    else if s.ends_with("y") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value * 60 * 60 * 24 * 365,
            Err(_) => {
                warn!(
                    "Failed to parse years from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if no suffix, just parse as a plain integer
    else {
        match s.parse::<u64>() {
            Ok(value) => value,
            Err(_) => {
                warn!(
                    "Failed to parse integer from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
}

pub struct ExchangeRateMessage {
    pub message: String,
    pub graph: String,
}

pub async fn get_exchange_rate_message(from: &str, to: &str) -> ExchangeRateMessage {
    // let rate_result = get_exchange_rate(from, to).await;

    let rates = get_exchange_rates().await;

    let msg_str: String = match rates {
        Ok(rates) => {
            // Print out rates
            for r in &rates {
                log::debug!("{}",r);
            }


            let prompt = get_prompt(&rates, from, to);

            let rate: f64 = rates
                .get(0)
                .cloned()
                .unwrap_or_default()
                .get_val(from, to)
                .unwrap_or(-1.0);

            // keep track how much time it takes to generate the sentence
            let start = std::time::Instant::now();

            let llm_res = generate_sentence(prompt.as_str()).await;

            let elapsed = start.elapsed();

            format!(
                "{}\n\
                ```\n\
                1 {} = {} {}\n\
                Generated in {}.{:03} seconds\n\
                ```",
                llm_res,
                from,
                rate,
                to,
                elapsed.as_secs(),
                elapsed.subsec_millis(),
            )
        }
        Err(e) => {
            match e {
                crate::exchange_rate::GetRatesError::RemoteError(fetch_exchange_rate_error) => {
                    format!("Error fetching API. Please verify the API URL or API key. URL used: `{}`\n Error: {:?}", 
                    environment::get_exchange_rate_api_url(), 
                    fetch_exchange_rate_error)
                }
                crate::exchange_rate::GetRatesError::LocalError(local_exchange_rate_error) => {
                    format!("Error reading local database.\nError: {:?}", local_exchange_rate_error)
                }
            }
        }
    };

    // CreateMessage::new().content(format!("From {}, to {}", from, to))
    ExchangeRateMessage {
        graph: "H".to_string(),
        message: msg_str,
    }
}
