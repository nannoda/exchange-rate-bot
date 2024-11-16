use std::borrow::Borrow;

use log::warn;
use rusqlite::Connection;
use serde_json::Value;
use serenity::all::CreateMessage;



use crate::{ database::exchange_rate::save_exchange_rate, environment::{self, get_exchange_rate_api_url}, exchange_rate::{get_exchange_rates, ExchangeRateMap}, llm::{generate::generate_sentence, prompt::get_prompt}, plots::{get_error_png, get_trend_graph}
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
    pub graph: Option<Vec<u8>>,
}

pub async fn get_exchange_rate_message(from: &str, to: &str) -> ExchangeRateMessage {
    // let rate_result = get_exchange_rate(from, to).await;

    let rates = get_exchange_rates().await;

    match rates {
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

            // Save rate for backward compatibility reason.
            save_exchange_rate(from, to, rate);

            // keep track how much time it takes to generate the sentence
            let start = std::time::Instant::now();

            let llm_res = generate_sentence(prompt.as_str()).await;

            let elapsed_llm = start.elapsed();

            let start_graph = std::time::Instant::now();
            let graph_result = get_trend_graph(&rates, from, to);
            let elapsed_graph = start_graph.elapsed();
            let elapsed_total = start.elapsed();

            let graph_message = match &graph_result {
                Ok(_) => String::new(), // No additional message if there's no error
                Err(err) => format!("\nGraph generation error: {}", err), // Include error message
            };

            let msg = format!(
                "{}\n\
                ```\n\
                1 {} = {} {}\n\
                Message generated in {}.{:03} seconds\n\
                Graph generated in {}.{:03} seconds{}\n\
                Generated in {}.{:03} seconds\n\
                ```",
                llm_res,
                from,
                rate,
                to,
                elapsed_llm.as_secs(),
                elapsed_llm.subsec_millis(),
                elapsed_graph.as_secs(),
                elapsed_graph.subsec_millis(),
                graph_message, // Add the error message dynamically
                elapsed_total.as_secs(),
                elapsed_total.subsec_millis(),
            );

            ExchangeRateMessage{
                message: msg,
                graph: graph_result.ok()
            }
           
        }
        Err(e) => {
            match e {
                crate::exchange_rate::GetRatesError::RemoteError(fetch_exchange_rate_error) => ExchangeRateMessage{
                    message:format!("Error fetching API. Please verify the API URL or API key. URL used: `{}`\n`Error: {:?}`", 
                    environment::get_exchange_rate_api_url(), 
                    fetch_exchange_rate_error),
                    graph: None
                },
                // crate::exchange_rate::GetRatesError::LocalError(local_exchange_rate_error) => ExchangeRateMessage{
                //     message: format!("Error reading local database.\n`Error: {:?}`", local_exchange_rate_error),
                //     graph: get_error_png("Local Error")
                // }
            }
        }
    }

}
