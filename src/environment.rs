use rusqlite::Connection;

use std::env;
use crate::utils::string_to_time_second;

const CREATE_EXCHANGE_RATE_RESULT_TABLE_QUERY: &str = r#"
CREATE TABLE IF NOT EXISTS exchange_rate
(
    from_currency VARCHAR(3) NOT NULL,
    to_currency VARCHAR(3) NOT NULL,
    rate DOUBLE NOT NULL,
    time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#;

const CREATE_EXCHANGE_RATE_API_RAW_TABLE_QUERY: &str = r#"
CREATE TABLE IF NOT EXISTS exchange_rate_api_raw
(
    raw TEXT NOT NULL,
    time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#;

const CREATE_LLM_RESULT_TABLE_QUERY: &str = r#"
CREATE TABLE IF NOT EXISTS llm_result
(
    prompt TEXT NOT NULL,
    result TEXT NOT NULL,
    time TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"#;

pub fn get_interval()->u64{
    let interval_str = get_and_set_env_var("INTERVAL", "24h");
    let interval_int = string_to_time_second(interval_str.as_str());
    return interval_int;
}

fn get_and_set_env_var(key: &str, default: &str) -> String {
    match env::var(key) {
        Ok(val) => val,
        Err(_) => {
            log::info!("{} not found, using default", key);
            // set environment variable
            env::set_var(key, default);
            default.to_string()
        },
    }
}

pub fn get_exchange_rate_change_threshold() -> f64 {
    let threshold_str = get_and_set_env_var("EXCHANGE_RATE_CHANGE_THRESHOLD", "0.001");
    let threshold: f64 = threshold_str.parse().unwrap();
    return threshold;
}

pub fn get_increase_prompt_template() -> String {
    return get_and_set_env_var("INCREASE_PROMPT_TEMPLATE", "Today is {DATE}. You want to write a short report informing the people about the increase in exchange from {FROM} to {TO} is now {CURR}, which is higher than the previous {PREV}. The increase is {DIFF}.");
}

pub fn get_decrease_prompt_template() -> String {
    return get_and_set_env_var("DECREASE_PROMPT_TEMPLATE", "Today is {DATE}. You want to write a short report informing the people about the decrease in exchange from {FROM} to {TO} is now {CURR}, which is lower than the previous {PREV}. The decrease is {DIFF}.");
}

pub fn get_equal_prompt_template() -> String {
    return get_and_set_env_var("EQUAL_PROMPT_TEMPLATE", "Today is {DATE}. You want to write a short report informing the people about the exchange from {FROM} to {TO} is now {CURR}, which is about the same as the previous {PREV}.");
}

pub fn get_db_file() -> String {
    return get_and_set_env_var("DB_FILE", "exchange_rate.db");
}

pub fn get_channels() -> Vec<u64> {
    let channels_str = get_and_set_env_var("CHANNELS", "");
    let channels: Vec<u64> = channels_str
        .split(",")
        .map(|s| s.parse().unwrap())
        .collect();
    return channels;
}

pub fn get_discord_token() -> String {
    match env::var("DISCORD_TOKEN") {
        Ok(val) => val,
        Err(_) => {
            // stop the program
            panic!("DISCORD_TOKEN not found in environment! Please set it in .env file or in the environment");
        }
    }
}

pub fn get_exchange_rate_api_key() -> String {
    match env::var("EXCHANGE_RATE_API_KEY") {
        Ok(val) => val,
        Err(_) => {
            // stop the program
            panic!("EXCHANGE_RATE_API_KEY not found in environment! Please set it in .env file or in the environment");
        }
    }
}

pub fn get_exchange_from() -> String {
    match env::var("EXCHANGE_FROM") {
        Ok(val) => val,
        Err(_) => {
            // stop the program
            panic!("EXCHANGE_FROM not found in environment! Please set it in .env file or in the environment");
        }
    }
}

pub fn get_exchange_to() -> String {
    match env::var("EXCHANGE_TO") {
        Ok(val) => val,
        Err(_) => {
            // stop the program
            panic!("EXCHANGE_TO not found in environment! Please set it in .env file or in the environment");
        }
    }
}

pub fn get_ollama_url()-> String {
    match env::var("OLLAMA_URL") {
        Ok(val) => val,
        Err(_) => {
            // stop the program
            panic!("OLLAMA_URL not found in environment! Please set it in .env file or in the environment");
        }
    }
}

fn ensure_db(){
    log::info!("Ensuring db");
    // get DB_FILE from environment
    let db_file = get_db_file();
    log::debug!("DB_FILE: {}", db_file);
    // create db if not exists
    let con = Connection::open(db_file).unwrap();
    con.execute(CREATE_EXCHANGE_RATE_RESULT_TABLE_QUERY, []).unwrap();
    con.execute(CREATE_EXCHANGE_RATE_API_RAW_TABLE_QUERY, []).unwrap();
    con.execute(CREATE_LLM_RESULT_TABLE_QUERY, []).unwrap();
}

/// Ensure environment variables are set
pub async fn ensure_environment(){
    log::info!("Ensuring environment");
    ensure_db();
    get_discord_token();
    get_exchange_rate_api_key();
}