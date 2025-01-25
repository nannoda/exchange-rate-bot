use chrono::{
    DateTime, NaiveDate, Utc,
};
use serde_json::Value;
use thiserror::Error;

use crate::{
    // database::historical_data::get_historical_exchange_rate_db,
    database::exchange_rate::{
        get_local_exchange_rate_fallback, save_exchange_rate_fallback,
        save_raw_exchange_rate_result,
    },
    environment::{self},
};
use std::{
    collections::HashMap,
    fmt::{self},
};

#[derive(Debug, Clone)]
pub struct ExchangeRateMap {
    pub datetime: DateTime<Utc>,
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
            self.datetime, self.base
        )?;

        // Remove the trailing comma
        write!(f, " ... }} }}")
    }
}

impl Default for ExchangeRateMap {
    fn default() -> Self {
        return ExchangeRateMap {
            datetime: Utc::now(),
            base: "EUR".to_string(),
            map: HashMap::new(),
        };
    }
}

#[derive(Debug, Error)]
pub enum FetchExchangeRateError {
    #[error("Request Error: {0}")]
    RequestError(String),

    #[error("Network Error: {0}")]
    NetworkError(String),

    #[error("Response Body Error: {0}")]
    ResponseBodyError(String),

    #[error("Failed to parse JSON: {0}")]
    ParseError(#[from] ExchangeRateParserError),

    // #[error("JSON in unexpected form: {0}")]
    // ShapeError(String),
    #[error("Missing API Key")]
    MissingKeyError,
}

impl ExchangeRateMap {
    pub fn get_date(&self) -> chrono::NaiveDate {
        self.datetime.date_naive()
    }
}

#[derive(Debug, Error)]
pub enum ExchangeRateParserError {
    #[error("Failed to parse JSON: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("JSON in unexpected form: {0}")]
    ShapeError(String),
}

impl ExchangeRateMap {
    pub fn get_val(&self, from: &str, to: &str) -> Option<f64> {
        // Convert `from` and `to` to uppercase
        let from = from.to_uppercase();
        let to = to.to_uppercase();

        // Set `from_value` or `to_value` to 1 if they match the base currency
        let from_value = if from == self.base {
            1.0
        } else {
            *self.map.get(&from)?
        };

        let to_value = if to == self.base {
            1.0
        } else {
            *self.map.get(&to)?
        };

        // Return the conversion rate
        Some(to_value / from_value)
    }

    pub fn parse_fallback_json(json: &str) -> Result<ExchangeRateMap, ExchangeRateParserError> {
        let v: Value = match serde_json::from_str(&json) {
            Ok(v) => v,
            Err(e) => return Err(ExchangeRateParserError::SerdeError(e)),
        };

        let datetime = match v
            .get("timestamp")
            .and_then(|v| v.as_i64())
            .and_then(|n| DateTime::from_timestamp(n, 0))
        {
            Some(t) => t,
            None => {
                return Err(ExchangeRateParserError::ShapeError(format!(
                    "JSON in wrong shape (datetime): {json}"
                )))
            }
        };

        let base = match v
            .get("base")
            .and_then(|v| v.as_str())
            .and_then(|s| Some(s.to_string()))
        {
            Some(t) => t,
            None => {
                return Err(ExchangeRateParserError::ShapeError(format!(
                    "JSON in wrong shape (base): {json}"
                )))
            }
        };

        let map: HashMap<String, f64> = match v.get("rates").and_then(|v| v.as_object()) {
            Some(o) => o
                .iter()
                .map(|(k, v)| {
                    let value = match v.as_f64() {
                        Some(v) => v,
                        None => {
                            return Err(ExchangeRateParserError::ShapeError(format!(
                                "JSON in wrong shape (map item): [{k}] {v}"
                            )))
                        }
                    };

                    Ok((k.clone(), value))
                })
                .collect::<Result<HashMap<_, _>, _>>()?,
            None => {
                return Err(ExchangeRateParserError::ShapeError(format!(
                    "JSON in wrong shape (map): {json}"
                )))
            }
        };

        return Ok(ExchangeRateMap {
            datetime,
            base,
            map,
        });
    }

    pub async fn get_rate_fallback(
        date: NaiveDate,
        base: Option<&String>,
    ) -> Result<ExchangeRateMap, FetchExchangeRateError> {
        let m = match get_local_exchange_rate_fallback(date) {
            Some(txt) => match ExchangeRateMap::parse_fallback_json(&txt) {
                Ok(m) => m,
                Err(e) => return Err(FetchExchangeRateError::ParseError(e)),
            },
            None => {
                let api_key = match environment::get_fallback_exchange_rate_api_key() {
                    Some(key) => key,
                    None => return Err(FetchExchangeRateError::MissingKeyError),
                };
                let api_base = environment::get_fallback_exchange_rate_api_url();

                let query_params = vec![
                    format!("access_key={}", api_key),
                    format!("base={}", base.unwrap_or(&"EUR".to_string())),
                ];
                let fetch_url = format!(
                    "{}/{}?{}",
                    api_base,
                    date.format("%Y-%m-%d"),
                    query_params.join("&")
                );

                log::debug!("Local cache missed. Getting from {fetch_url}");

                let response: reqwest::Response = match reqwest::get(&fetch_url).await {
                    Ok(r) => r,
                    Err(e) => return Err(FetchExchangeRateError::NetworkError(e.to_string())),
                };

                let text = match response.text().await {
                    Ok(t) => t,
                    Err(e) => return Err(FetchExchangeRateError::ResponseBodyError(e.to_string())),
                };

                let map = match ExchangeRateMap::parse_fallback_json(&text) {
                    Ok(map) => {
                        let timestamp = map.datetime;
                        save_exchange_rate_fallback(&text, timestamp);
                        map
                    }
                    Err(e) => return Err(FetchExchangeRateError::ParseError(e)),
                };
                map
            }
        };

        // log::debug!("JSON text for the fallback: {}", json_txt);
        Ok(m)
    }

    pub async fn get_rates(
        from_date: NaiveDate,
        base: Option<String>,
    ) -> Result<Vec<ExchangeRateMap>, FetchExchangeRateError> {
        let base: String = base.unwrap_or("EUR".to_string()).to_uppercase();

        let api_base = environment::get_exchange_rate_api_url();
        let fetch_url = format!(
            "{}/{}..?base={}",
            api_base,
            from_date.format("%Y-%m-%d"),
            &base
        );

        log::debug!("Fetching URL: {fetch_url}");

        // Perform the API request
        let result = reqwest::get(&fetch_url).await;

        if let Err(e) = &result {
            return Err(FetchExchangeRateError::NetworkError(e.to_string()));
        }

        let response = result.unwrap();

        if !response.status().is_success() {
            return Err(FetchExchangeRateError::RequestError(format!(
                "Request failed with status: {}, URL: {}",
                response.status(),
                &fetch_url
            )));
        }

        // Safely attempt to read the response body
        let text = response
            .text()
            .await
            .map_err(|e| FetchExchangeRateError::ResponseBodyError(e.to_string()))?;

        log::debug!("text: {}", text);

        // Save raw results for backward compatibility reason.
        save_raw_exchange_rate_result(&text);

        let parsed: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                return Err(FetchExchangeRateError::ParseError(
                    ExchangeRateParserError::SerdeError(e),
                ))
            }
        };

        let rates_raw = parsed
            .get("rates")
            .and_then(|v| v.as_object())
            .ok_or_else(|| {
                FetchExchangeRateError::ParseError(ExchangeRateParserError::ShapeError(
                    "rates in wrong shape".to_string(),
                ))
            })?;

        let mut rates: HashMap<NaiveDate, ExchangeRateMap> = HashMap::new();

        for (date, rate_map) in rates_raw {
            let date_with_time = format!("{}T00:00:00+00:00", date);
            let datetime = DateTime::parse_from_rfc3339(&date_with_time).map_err(|e| {
                FetchExchangeRateError::ParseError(ExchangeRateParserError::ShapeError(format!(
                    "Invalid date ({}): {:?}",
                    date_with_time, e
                )))
            })?;

            let rate_map = rate_map.as_object().ok_or_else(|| {
                FetchExchangeRateError::ParseError(ExchangeRateParserError::ShapeError(
                    "rate_map in wrong shape".to_string(),
                ))
            })?;

            let map: HashMap<String, f64> = rate_map
                .iter()
                .map(|(k, v)| {
                    let value = v.as_f64().ok_or_else(|| {
                        FetchExchangeRateError::ParseError(ExchangeRateParserError::ShapeError(
                            format!("Invalid value for key: {}", k),
                        ))
                    })?;
                    Ok((k.clone(), value))
                })
                .collect::<Result<HashMap<_, _>, FetchExchangeRateError>>()?;
            log::debug!("Map: {:?}", map);

            rates.insert(
                datetime.date_naive(),
                ExchangeRateMap {
                    datetime: datetime.into(),
                    base: base.clone(),
                    map,
                },
            );

            // rates.push(;
        }

        // log::debug!("Rates: {:?}", rates.get(0).unwrap().get_val("USD", "CNY"));

        // Sort rates by datetime
        // rates.sort_by(|a, b| a.datetime.cmp(&b.datetime));

        // The EU bank API doesn't have data on weekend.
        // Fill in empty
        let mut current_date = Utc::now().date_naive();

        while current_date >= from_date {
            if rates.get(&current_date).is_none() {
                // Date is missing
                match ExchangeRateMap::get_rate_fallback(current_date, Some(&base)).await {
                    Ok(map) => {
                        rates.insert(current_date, map);
                    }
                    Err(e) => {
                        log::warn!("Error: {e}")
                    }
                };
            }

            current_date = match current_date.pred_opt() {
                Some(date) => date,
                None => {
                    log::warn!("Unable to get previous date from {current_date}");
                    break;
                }
            }
        }

        // Convert HashMap to Vec, sorting by the date (NaiveDate)
        let mut result_vec: Vec<ExchangeRateMap> = rates.into_iter().map(|(_, v)| v).collect();

        result_vec.sort_by(|a, b| a.datetime.cmp(&b.datetime));

        Ok(result_vec)
    }
}
