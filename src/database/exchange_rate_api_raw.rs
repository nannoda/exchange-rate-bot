use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, Error as RusqliteError};

use crate::environment;

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