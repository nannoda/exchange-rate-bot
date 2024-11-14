use rusqlite::Connection;
use serde_json::Value;

use crate::{
    environment::{self, get_exchange_rate_api_url},
    llm::generate_sentence,
};

#[derive(Debug)]
pub enum GetExchangeRateError {
    APIError,
    ParseError,
}

impl std::fmt::Display for GetExchangeRateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error getting exchange rate")
    }
}

/**
 * Get exchange rate from API
 */
pub async fn get_exchange_rate(from: &str, to: &str) -> Result<f64, GetExchangeRateError> {
    let api_key = environment::get_exchange_rate_api_key();

    let result = reqwest::get(format!(
        "{}?access_key={}",
        get_exchange_rate_api_url(),
        api_key
    ))
    .await;

    if result.is_err() {
        return Err(GetExchangeRateError::APIError);
    }

    let text = result.unwrap().text().await.unwrap();

    log::debug!("text: {}", text);

    save_raw_exchange_rate(&text);

    let dict: Value = serde_json::from_str(&text).unwrap();

    let from_rate = match dict["rates"][from].as_f64() {
        Some(rate) => rate,
        None => {
            return Err(GetExchangeRateError::ParseError);
        }
    };

    let to_rate = match dict["rates"][to].as_f64() {
        Some(rate) => rate,
        None => {
            return Err(GetExchangeRateError::ParseError);
        }
    };

    let rate = to_rate / from_rate;

    save_exchange_rate(from, to, rate);

    Ok(rate)
}

/**
 * Save exchange rate to database
 */
pub fn save_exchange_rate(from: &str, to: &str, rate: f64) {
    let db_file = environment::get_db_file();
    let con = Connection::open(db_file).unwrap();

    let query = "INSERT INTO exchange_rate (from_currency, to_currency, rate) VALUES (?, ?, ?)";

    con.execute(query, &[from, to, rate.to_string().as_str()])
        .unwrap();

    // println!("Saved exchange rate from {} to {} as {}", from, to, rate);
    log::debug!("Saved exchange rate from {} to {} as {}", from, to, rate);
}

/**
 * Save raw exchange rate to database
 */
pub fn save_raw_exchange_rate(raw: &str) {
    let db_file = environment::get_db_file();
    let con = Connection::open(db_file).unwrap();

    let query = "INSERT INTO exchange_rate_api_raw (raw) VALUES (?)";

    con.execute(query, &[raw]).unwrap();

    log::debug!("Saved raw exchange rate: {}", raw);
}

/**
 * Save raw LLM result to database
 */
pub fn save_llm_result(prompt: &str, result: &str) {
    let db_file = environment::get_db_file();
    let con = Connection::open(db_file).unwrap();

    let query = "INSERT INTO llm_result (prompt, result) VALUES (?, ?)";

    con.execute(query, &[prompt, result]).unwrap();

    log::debug!("Saved llm result: {} -> {}", prompt, result);
}

pub fn get_last_exchange_rate(from: &str, to: &str, offset: Option<i64>) -> f64 {
    let db_file = environment::get_db_file();
    let con = Connection::open(db_file).unwrap();

    let query = "SELECT rate FROM exchange_rate WHERE from_currency = ? AND to_currency = ? ORDER BY time DESC LIMIT 1 OFFSET ?";

    let mut stmt = con.prepare(query).unwrap();

    let mut rows = stmt
        .query(&[from, to, &offset.unwrap_or(0).to_string()])
        .unwrap();

    let mut rate = || -> Result<f64, rusqlite::Error> {
        let option_row = rows.next()?;
        let row = option_row.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        let rate = row.get(0)?;
        Ok(rate)
    };

    match rate() {
        Ok(rate) => rate,
        Err(e) => {
            log::warn!("Error getting exchange rate: {}", e);
            -1.0
        }
    }
}

fn replace_template(
    template: &str,
    from: &str,
    to: &str,
    current_rate: f64,
    last_rate: f64,
    current_date: &str,
) -> String {
    template
        .replace("{FROM}", from)
        .replace("{TO}", to)
        .replace("{CURR}", &format!("{:.3}", current_rate))
        .replace("{PREV}", &format!("{:.3}", last_rate))
        .replace("{DIFF}", &format!("{:.3}", last_rate - current_rate))
        .replace("{DATE}", current_date)
}

pub fn get_prompt(current_rate: f64, from: &str, to: &str) -> String {
    // let from = environment::get_exchange_from();
    // let to = environment::get_exchange_to();

    let last_rate = get_last_exchange_rate(&from, &to, Some(1));

    log::debug!("Last rate: {}", last_rate);
    let current_date = chrono::Local::now().format("%Y-%m-%d").to_string();

    let threshold = environment::get_exchange_rate_change_threshold();

    log::debug!("Threshold: {}", threshold);

    let prompt = match current_rate {
        rate if (rate - last_rate).abs() < threshold => replace_template(
            environment::get_equal_prompt_template().as_str(),
            &from,
            &to,
            current_rate,
            last_rate,
            &current_date,
        ),
        rate if rate > last_rate => replace_template(
            environment::get_increase_prompt_template().as_str(),
            &from,
            &to,
            current_rate,
            last_rate,
            &current_date,
        ),
        rate if rate < last_rate => replace_template(
            environment::get_decrease_prompt_template().as_str(),
            &from,
            &to,
            current_rate,
            last_rate,
            &current_date,
        ),
        _ => replace_template(
            environment::get_equal_prompt_template().as_str(),
            &from,
            &to,
            current_rate,
            last_rate,
            &current_date,
        ),
    };

    log::debug!("Prompt: {}", prompt);
    prompt
}

pub fn string_to_time_second(s: &str) -> u64 {
    // convert string to lower case
    let s = s.to_lowercase();
    // if string ends with 's', remove it and convert to integer
    if s.ends_with("s") {
        s[..s.len() - 1].parse::<u64>().unwrap()
    }
    // if string ends with 'm', remove it and convert to integer
    else if s.ends_with("m") {
        s[..s.len() - 1].parse::<u64>().unwrap() * 60
    }
    // if string ends with 'h', remove it and convert to integer
    else if s.ends_with("h") {
        s[..s.len() - 1].parse::<u64>().unwrap() * 60 * 60
    }
    // if string ends with 'd', remove it and convert to integer
    else if s.ends_with("d") {
        s[..s.len() - 1].parse::<u64>().unwrap() * 60 * 60 * 24
    }
    // if string ends with 'w', remove it and convert to integer
    else if s.ends_with("w") {
        s[..s.len() - 1].parse::<u64>().unwrap() * 60 * 60 * 24 * 7
    }
    // if string ends with 'y', remove it and convert to integer
    else if s.ends_with("y") {
        s[..s.len() - 1].parse::<u64>().unwrap() * 60 * 60 * 24 * 365
    }
    // else convert to integer
    else {
        s.parse::<u64>().unwrap()
    }
}

pub async fn get_exchange_rate_message(from: &str, to: &str) -> String {
    let rate_result = get_exchange_rate(from, to).await;

    match rate_result {
        Err(GetExchangeRateError::APIError) => "GetExchangeRateError::APIError".to_string(),
        Err(GetExchangeRateError::ParseError) => "GetExchangeRateError::ParseError".to_string(),
        Ok(rate) => {
            let prompt = get_prompt(rate, from, to);

            // keep track how much time it takes to generate the sentence
            let start = std::time::Instant::now();

            let llm_res = generate_sentence(prompt.as_str());

            let res_without_prompt = llm_res.await;

            let elapsed = start.elapsed();

            format!(
                "{}\n\
                ```
                1 {} = {} {}\n\
                Generated in {}.{:03} seconds\n\
                ```",
                res_without_prompt,
                from,
                rate,
                to,
                elapsed.as_secs(),
                elapsed.subsec_millis(),
            )
        },
        _ => "Unknown exchange rate result".to_string(),
    }
}
