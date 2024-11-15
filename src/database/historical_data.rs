use serde_json::Value;
use thiserror::Error;

use chrono::{NaiveDate, ParseError as ChronoParseError};
use rusqlite::{Connection, Error as RusqliteError};
use rusqlite::OptionalExtension;
use crate::environment;
use crate::exchange_rate::ExchangeRateMap;

pub fn get_historical_exchange_rate_db(date: &NaiveDate) -> Option<ExchangeRateMap> {
    // Get the database file path from the environment
    let db_file = environment::get_db_file();

    // Attempt to open the database connection
    let con = match Connection::open(db_file) {
        Ok(connection) => connection,
        Err(err) => {
            log::error!("Failed to open the database connection: {}", err);
            return None;
        }
    };

    // SQL query to find the latest JSON data for the given date
    let query = r#"
        SELECT json
        FROM json_data
        WHERE date = ?
        ORDER BY insert_at DESC
        LIMIT 1
    "#;

    // Prepare the SQL statement
    let mut stmt = match con.prepare(query) {
        Ok(statement) => statement,
        Err(err) => {
            log::error!("Failed to prepare SQL query: {}", err);
            return None;
        }
    };

    // Execute the query with the provided date
    let result: Result<Option<String>, RusqliteError> = stmt.query_row([date.to_string()], |row| {
        row.get(0) // Attempt to retrieve the first column as a String
    }).optional(); // Use `.optional()` to handle cases with no rows.

    // Handle the query result
    match result {
        Ok(Some(json)) => match ExchangeRateMap::from_json(&json) {
            Ok(map) => Some(map),
            Err(err) => {
                log::error!("Error querying JSON data for date {}: {}", date, err);
                None
            },
        }, // Return the JSON if found
        Ok(None) => {
            log::info!("No data found for the provided date: {}", date);
            None
        }
        Err(err) => {
            log::error!("Error querying JSON data for date {}: {}", date, err);
            None
        }
    }
}

#[derive(Error, Debug)]
pub enum SaveError {
    #[error("JSON parsing error: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("Missing or invalid 'success' field in JSON")]
    MissingSuccessField,
    #[error("Historical data is not marked as 'success'")]
    HistoricalDataNotSuccessful,
    #[error("Missing 'date' field in JSON")]
    MissingDateField,
    #[error("Invalid date format: {0}")]
    InvalidDateFormat(#[from] ChronoParseError),
    #[error("Database error: {0}")]
    DatabaseError(#[from] RusqliteError),
    #[error("Unknown error occurred")]
    UnknownError,
}

pub fn save_historical_exchange_rate(json: &str) -> Result<(), SaveError>{
    // Parse the JSON
    let parsed_json: Value = serde_json::from_str(&json)?;

    // Check if the "success" field exists and is true
    let success = parsed_json
        .get("success")
        .and_then(Value::as_bool)
        .ok_or(SaveError::MissingSuccessField)?;

    if !success {
        return Err(SaveError::HistoricalDataNotSuccessful);
    }

    // Extract and validate the "date" field
    let date_str = parsed_json
        .get("date")
        .and_then(Value::as_str)
        .ok_or(SaveError::MissingDateField)?;

    let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d")?;

    // Get the database file path
    let db_file = environment::get_db_file();
    let con = Connection::open(db_file)?;

    // Insert the JSON into the database
    let query = "INSERT INTO historical_data (json, date) VALUES (?, ?)";
    con.execute(query, &[&json, &date.to_string().as_str()])?;

    Ok(())
}
