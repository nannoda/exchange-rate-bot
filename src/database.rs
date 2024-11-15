use std::{
    error::Error,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::Connection;

use crate::environment;
use rusqlite::Error as RusqliteError;
/**
 * Save raw exchange rate to database
 */
pub fn save_raw_exchange_rate_result(raw: &str) {
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

pub fn get_local_last_seven_days_exchange_rate_json() -> Result<Vec<String>, RusqliteError> {
    let db_file = environment::get_db_file();
    let con = Connection::open(db_file)?;

    // Calculate the timestamp for 7 days ago, converting SystemTimeError into rusqlite::Error
    let seven_days_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Integer,
                Box::new(e),
            )
        })?
        .as_secs()
        - 7 * 24 * 60 * 60; // 7 days ago in seconds

    // Prepare the SQL query to get data from the last 7 days
    let query = "SELECT raw FROM exchange_rate_api_raw WHERE time >= ? ORDER BY time DESC";

    // Execute the query, passing the limit as a parameter
    let mut stmt = con.prepare(query)?;

    // Execute the query and get the rows
    let mut rows = stmt.query(&[&(seven_days_ago as i64)])?;

    // Closure to extract the 'raw' field from each row and return it as a String
    let mut raw_data = Vec::new();
    while let Some(row) = rows.next()? {
        let raw: String = row.get(0)?;
        raw_data.push(raw);
    }

    if raw_data.is_empty() {
        log::warn!("No data found in exchange_rate_api_raw table");
    }

    Ok(raw_data)
}
