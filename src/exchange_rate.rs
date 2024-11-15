use chrono::{DateTime, ParseError, TimeZone, Utc};
use log::debug;
use rusqlite::Connection;
use serde_json::Value;

use crate::{
    database::{get_local_last_seven_days_exchange_rate_json, save_raw_exchange_rate_result},
    environment,
};
use std::{collections::HashMap, error::Error, fmt};

#[derive(Debug, Clone)]
pub struct ExchangeRateMap {
    pub success: bool,
    pub timestamp: DateTime<Utc>,
    pub base: String,
    pub map: HashMap<String, f64>,
}

#[derive(Debug)]
pub enum ExchangeRateError {
    ParseError(serde_json::Error),
    ParseDateTimeError,
    InvalidResponse(String),
    ApiError(String, String), // Error code and info message
    MissingField(String),     // Missing expected fields
    InvalidData(String),      // Invalid data or format
}

// Implement Display trait for ExchangeRate
impl fmt::Display for ExchangeRateMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ExchangeRate {{ success: {}, timestamp: {}, base: {}, rates: {{",
            self.success, self.timestamp, self.base
        )?;

        // Iterate through the rates and format them
        for (currency, rate) in &self.map {
            write!(f, " {}: {:.2},", currency, rate)?;
        }

        // Remove the trailing comma
        write!(f, " }} }}")
    }
}
#[derive(Debug)]
enum ValueError {
    MissingFrom,
    MissingTo,
}

impl Default for ExchangeRateMap {
    fn default() -> Self {
        let map = HashMap::new();
        return ExchangeRateMap {
            success: false,
            timestamp: Utc::now(),
            base: "EUR".to_string(),
            map,
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

        // Ensure 'success' is true for valid responses
        let success = parsed
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !success {
            return Err(ExchangeRateError::InvalidResponse(
                "API request was not successful".to_string(),
            ));
        }

        // Parse the other expected fields
        let timestamp = parsed
            .get("timestamp")
            .and_then(|t| t.as_i64())
            .and_then(|t| {
                Some(
                    Utc.timestamp_opt(t, 0)
                        .single()
                        .ok_or_else(|| ExchangeRateError::ParseDateTimeError),
                )
            })
            .ok_or_else(|| ExchangeRateError::MissingField("timestamp".to_string()))??;

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
            success,
            timestamp,
            base,
            map: rate_map,
        })
    }
}

#[derive(Debug)]
pub enum FetchExchangeRateError {
    RequestError(String),
    NetworkError(String),
    ResponseBodyError(String),
    ParseError(ExchangeRateError),
}

/// This function fetches and stores the exchange rate on the DB
pub async fn fetch_and_store_exchange_rate() -> Result<ExchangeRateMap, FetchExchangeRateError> {
    let api_key = environment::get_exchange_rate_api_key();
    let api_url = environment::get_exchange_rate_api_url();

    let result = reqwest::get(format!("{}?access_key={}", api_url, api_key)).await;

    if let Err(e) = &result {
        return Err(FetchExchangeRateError::NetworkError(e.to_string()));
    }

    let response = result.unwrap();

    if !response.status().is_success() {
        return Err(FetchExchangeRateError::RequestError(format!(
            "Request failed with status: {}",
            response.status()
        )));
    }

    // Safely attempt to read the response body
    let text = response
        .text()
        .await
        .map_err(|e| FetchExchangeRateError::ResponseBodyError(e.to_string()))?;

    log::debug!("text: {}", text);

    save_raw_exchange_rate_result(&text);

    match ExchangeRateMap::from_json(&text) {
        Ok(r) => Ok(r),
        Err(err) => Err(FetchExchangeRateError::ParseError(err)),
    }
}

use rusqlite::Error as RusqliteError;

#[derive(Debug)]
pub enum LocalExchangeRateError {
    SqlError(RusqliteError),
    ParseError(ExchangeRateError),
}

/// Get exchange rates from local database
pub fn get_local_exchange_rates() -> Result<Vec<ExchangeRateMap>, LocalExchangeRateError> {
    let local_rates_raw = match get_local_last_seven_days_exchange_rate_json() {
        Ok(raws) => raws,
        Err(err) => return Err(LocalExchangeRateError::SqlError(err)),
    };

    let mut rates = Vec::new();

    for rate_raw in local_rates_raw.iter() {
        match ExchangeRateMap::from_json(&rate_raw) {
            Ok(rate) => {
                rates.push(rate);
            }
            Err(err) => debug!("Error parsing rate raw: {:?}", &err),
        }
    }

    return Ok(rates);
}

pub enum GetRatesError {
    RemoteError(FetchExchangeRateError),
    LocalError(LocalExchangeRateError),
}

pub async fn get_exchange_rates() -> Result<Vec<ExchangeRateMap>, GetRatesError> {
    let old_rates = match get_local_exchange_rates() {
        Ok(rates) => rates,
        Err(e) => return Err(GetRatesError::LocalError(e)),
    };

    let new_rate = match fetch_and_store_exchange_rate().await {
        Ok(rate) => rate,
        Err(e) => return Err(GetRatesError::RemoteError(e)),
    };

    let mut rates = Vec::new();

    log::debug!("New Rate: {:?}", &new_rate);
    rates.push(new_rate);

    for old_rate in old_rates {
        log::debug!("Old rate: {:?}", &old_rate);
        rates.push(old_rate);
    }

    return Ok(rates);
}
