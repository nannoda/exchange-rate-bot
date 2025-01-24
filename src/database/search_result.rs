use rusqlite::Connection;

use crate::environment;

/**
 * Save exchange rate to database
 */
pub fn save_search_result(url: &str, res: &str) {
    let db_file = environment::get_db_file();
    let con = Connection::open(db_file).unwrap();

    let query = "INSERT INTO search_result (url, result) VALUES (?, ?)";

    con.execute(query, &[url, res]).unwrap();

    log::debug!("Saved search result");
}
