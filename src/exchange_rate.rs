use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, ParseError, TimeZone, Utc};
use log::debug;
use rusqlite::Connection;
use serde_json::Value;
use serenity::futures::future::ok;
use thiserror::Error;

use crate::{
    database::{
        exchange_rate_api_raw::save_raw_exchange_rate_result,
        historical_data::get_historical_exchange_rate_db,
    },
    environment::{self, DAYS_TO_CHECK},
    exchange_rate_api::{fetch_exchange_rate, FetchExchangeRateError, FetchMode},
};
use std::{collections::HashMap, error::Error, fmt};

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

        // If both values are present, calculate the exchange rate
        let from_value = from_val.unwrap();
        let to_value = to_val.unwrap();
        Some(to_value / from_value)
    }

    // Constructor that manually parses the JSON into an ExchangeRate struct
    pub fn from_json(json: &str) -> Result<Self, ExchangeRateError> {
        let parsed: Value = match serde_json::from_str(json) {
            Ok(value) => value,
            Err(e) => return Err(ExchangeRateError::ParseError(e)),
        };

        // Check for 'success' field first, and handle possible API errors
        if let Some(error_info) = parsed.get("error") {
            if let (Some(code), Some(info)) = (error_info.get("code"), error_info.get("info")) {
                if let (Some(code_str), Some(info_str)) = (code.as_u64(), info.as_str()) {
                    return Err(ExchangeRateError::ApiError(
                        code_str.to_string(),
                        info_str.to_string(),
                    ));
                }
            }
        }

        let date = parsed
            .get("date")
            .and_then(|t| t.as_str())
            .and_then(|date_str| {
                Some(
                    NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                        .map_err(|_| ExchangeRateError::ParseDateError),
                )
            })
            .ok_or_else(|| ExchangeRateError::MissingField("date".to_string()))??;

        let base = parsed
            .get("base")
            .and_then(|b| b.as_str())
            .ok_or_else(|| ExchangeRateError::MissingField("base currency".to_string()))?
            .to_string();

        let rates = parsed
            .get("rates")
            .and_then(|r| r.as_object())
            .ok_or_else(|| ExchangeRateError::MissingField("rates".to_string()))?;

        // Convert the rates into a HashMap
        let mut rate_map = HashMap::new();
        for (currency, rate_value) in rates {
            if let Some(rate) = rate_value.as_f64() {
                rate_map.insert(currency.to_string(), rate);
            } else {
                return Err(ExchangeRateError::InvalidData(format!(
                    "Invalid rate data for currency: {}",
                    currency
                )));
            }
        }
        Ok(ExchangeRateMap {
            base,
            date,
            map: rate_map,
        })
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

pub async fn get_exchange_rate_from_date(
    date: &NaiveDate,
) -> Result<ExchangeRateMap, GetRatesError> {
    match get_historical_exchange_rate_db(date) {
        Some(map) => return Ok(map),
        None => log::debug!("Cannot find historical data on {}", date),
    };
    match fetch_exchange_rate(FetchMode::Date(date)).await {
        Ok(map) => return Ok(map),
        Err(err) => return Err(GetRatesError::RemoteError(err)),
    }
}

pub async fn get_exchange_rates() -> Result<Vec<ExchangeRateMap>, GetRatesError> {
    // Get today's date
    let today = Local::now().date_naive();

    let mut rates = Vec::new();

    // Get the current exchange rate
    match fetch_exchange_rate(FetchMode::Latest).await {
        Ok(map) => rates.push(map),
        Err(err) => return Err(GetRatesError::RemoteError(err)),
    }

    // Get for the past 7 days
    for i in 1..DAYS_TO_CHECK {
        let date = today - Duration::days(i);
        match get_exchange_rate_from_date(&date).await {
            Ok(map) => rates.push(map),
            Err(err) => return Err(err),
        }
    }

    return Ok(rates);
}
