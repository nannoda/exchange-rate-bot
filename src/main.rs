use dotenv::dotenv;
mod environment;
mod utils;
mod bot;
mod llm;

#[tokio::main]
async fn main() {
    match dotenv() {
        Ok(_) => {
            env_logger::init();
            log::info!("Loaded .env file")},
        Err(_) => {
            env_logger::init();
            log::info!("No .env file found")
        }
    }

    log::info!("Starting bot");

    log::debug!("Log level: {}", log::max_level());

    environment::ensure_environment().await;
    bot::run_bot().await;
}
