use serenity::all::{
    CreateCommand, EditInteractionResponse,
};

use crate::environment;

pub const COMMAND_NAME: &str = "about";

// Function to register the exchange command with optional "from" and "to" parameters
pub fn register() -> CreateCommand {
    CreateCommand::new(COMMAND_NAME).description("About the bot")
}

pub fn run() -> EditInteractionResponse {
    let content = format!(
        "
    **Exchange Rate Bot**\n\
    Version: `{}`\n\n\
    **Configuration**:\n\
    ```\n\
    - Exchange From: `{}`\n\
    - Exchange To: `{}`\n\
    - Schedule: `{}`\n\
    - Exchange Rate API: `{}`\n\
    - Fallback Exchange Rate API: `{}`\n\
    - SearXNG API: `{}`\n\
    - Ollama Model: `{}`\n\
    ```
    ",
        environment::APP_VERSION,
        environment::get_exchange_from(),
        environment::get_exchange_to(),
        environment::get_cron_expression(),
        environment::get_exchange_rate_api_url(),
        match environment::get_fallback_exchange_rate_api_key() {
            Some(_) => "ENABLED",
            None => "DISABLED",
        },
        match environment::get_searxng_url() {
            Some(url) => url,
            None => "N/A".to_string(),
        },
        environment::get_ollama_model(),
    );

    return EditInteractionResponse::new().content(content);
}
