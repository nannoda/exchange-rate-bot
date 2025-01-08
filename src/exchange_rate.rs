use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, ParseError, TimeZone, Utc};
use serde_json::Value;
use thiserror::Error;

use crate::{
    database::historical_data::get_historical_exchange_rate_db,
    environment::{self, DAYS_TO_CHECK},
    exchange_rate_api::{fetch_exchange_rate, FetchExchangeRateError, FetchMode},
};
use std::{collections::HashMap, fmt};

#[derive(Debug, Clone)]
pub struct ExchangeRateMap {
    pub date: NaiveDate,
    pub base: String,
    pub map: HashMap<String, f64>,
}

#[derive(Debug, Error)]
pub enum ExchangeRateError {
    #[error("Failed to parse JSON: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Failed to parse datetime")]
    ParseDateTimeError,

    #[error("Failed to parse date")]
    ParseDateError,

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("API error (code: {0}, message: {1})")]
    ApiError(String, String),

    #[error("Missing field: {0}")]
    MissingField(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}
// Implement Display trait for ExchangeRate
impl fmt::Display for ExchangeRateMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ExchangeRate {{ date: {}, base: {}, rates: {{",
            self.date, self.base
        )?;

        // Iterate through the rates and format them
        // for (currency, rate) in &self.map {
        //     write!(f, " {}: {:.2},", currency, rate)?;
        // }

        // Remove the trailing comma
        write!(f, " ... }} }}")
    }
}

impl Default for ExchangeRateMap {
    fn default() -> Self {
        return ExchangeRateMap {
            date: Utc::now().date_naive(),
            base: "EUR".to_string(),
            map: HashMap::new(),
        };
    }
}

impl ExchangeRateMap {
    pub fn get_val(&self, from: &str, to: &str) -> Option<f64> {
        let from_val = self.map.get(from);
        if from_val.is_none() {
            return None;
        }
        let to_val = self.map.get(to);
        if to_val.is_none() {
            return None;
        }
        let from_value = from_val.unwrap();
        let to_value = to_val.unwrap();
        Some(to_value / from_value)
    }
}

pub struct ExchangeRateHistory {
    pub rates: ExchangeRateMap,
}

impl ExchangeRateHistory {
    pub fn create(last: DateTime<Utc>) -> ExchangeRateHistory {
        let api_url = environment::get_exchange_rate_api_url();
        let fetch_url = format!("{}/{}", api_url, last.format("%Y-%m-%d"));

        

    }
}

use rusqlite::Error as RusqliteError;

#[derive(Debug)]
pub enum LocalExchangeRateError {
    SqlError(RusqliteError),
    ParseError(ExchangeRateError),
}

pub enum GetRatesError {
    RemoteError(FetchExchangeRateError),
}
