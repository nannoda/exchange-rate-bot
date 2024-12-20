use chrono::NaiveDate;

use crate::{
    database::{
        exchange_rate_api_raw::save_raw_exchange_rate_result,
        historical_data::save_historical_exchange_rate,
    },
    environment,
    exchange_rate::{ExchangeRateError, ExchangeRateMap},
};

#[derive(Debug)]
pub enum FetchExchangeRateError {
    RequestError(String),
    NetworkError(String),
    ResponseBodyError(String),
    ParseError(ExchangeRateError),
}
pub enum FetchMode<'a> {
    Latest,
    Date(&'a NaiveDate),
}

pub async fn fetch_exchange_rate<'a>(
    mode: FetchMode<'a>,
) -> Result<ExchangeRateMap, FetchExchangeRateError> {
    let api_url = environment::get_exchange_rate_api_url();

    // Construct the URL with optional parameters
    let url = match mode {
        FetchMode::Latest => format!("{}/latest", api_url),
        FetchMode::Date(date) => format!("{}/{}", api_url, date.format("%Y-%m-%d")),
    };

    let query_params: Vec<String> = Vec::new();

    let full_url = format!("{}?{}", url, query_params.join("&"));

    // Perform the API request
    let result = reqwest::get(&full_url).await;

    if let Err(e) = &result {
        return Err(FetchExchangeRateError::NetworkError(
            e.to_string(),
        ));
    }

    let response = result.unwrap();

    if !response.status().is_success() {
        return Err(FetchExchangeRateError::RequestError(format!(
            "Request failed with status: {}, URL: {}",
            response.status(), &full_url
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

    match ExchangeRateMap::from_json(&text) {
        Ok(map) => {
            match save_historical_exchange_rate(&text) {
                Ok(_) => (),
                Err(err) => log::warn!("Error while saving: {}. Error {}", &text, err),
            };
            Ok(map)
        }
        Err(err) => Err(FetchExchangeRateError::ParseError(err)),
    }
}
