use log::warn;
use rusqlite::Connection;
use serde_json::Value;
use serenity::all::CreateMessage;

use crate::{
    environment::{self, get_exchange_rate_api_url},
    exchange_rate::ExchangeRateMap,
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

    // match rate_result {
    //     Ok(rate) => {
    //         let prompt = get_prompt(rate, from, to);

    //         // keep track how much time it takes to generate the sentence
    //         let start = std::time::Instant::now();

    //         let llm_res = generate_sentence(prompt.as_str());

    //         let res_without_prompt = llm_res.await;

    //         let elapsed = start.elapsed();

    //         format!(
    //             "{}\n\
    //             ```\n\
    //             1 {} = {} {}\n\
    //             Generated in {}.{:03} seconds\n\
    //             ```",
    //             res_without_prompt,
    //             from,
    //             rate,
    //             to,
    //             elapsed.as_secs(),
    //             elapsed.subsec_millis(),
    //         )
    //     },
    //     Err(GetExchangeRateError::APIError) => format!(
    //         "Unable to retrieve exchange rate from {} to {}. This may be due to an API error. Please verify the API URL or API key. URL used: `{}`",
    //         from,
    //         to,
    //         environment::get_exchange_rate_api_url()
    //     ),
    //     Err(GetExchangeRateError::ParseError) => format!(
    //         "Exchange rate retrieval failed from {} to {} due to a parsing error. This might indicate an exceeded API usage limit or an unexpected response format.",
    //         from,
    //         to
    //     ),
    // }

    // CreateMessage::new().content(format!("From {}, to {}", from, to))
    ExchangeRateMessage {
        graph: "H".to_string(),
        message: format!("From {}, to {}", from, to),
    }
}
