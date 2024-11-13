use serenity::all::{
    CommandOptionType, CreateInteractionResponse, CreateInteractionResponseMessage, Interaction,
};
use serenity::builder::{CreateAutocompleteResponse, CreateCommand, CreateCommandOption};
// use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue, interaction::{Interaction, InteractionResponseType}};
use log::{debug, warn};
use serenity::http::Http;
use serenity::prelude::*;

use crate::environment::{self, ensure_environment};
use crate::utils::get_exchange_rate_message;

const COMMAND_NAME: &str = "exchange-check";

// Function to register the exchange command with optional "from" and "to" parameters
pub fn register() -> CreateCommand {
    CreateCommand::new(COMMAND_NAME)
        .description("Check exchange rate between two currencies")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "from",
                "Currency to convert from",
            )
            .required(false)
            .set_autocomplete(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "to", "Currency to convert to")
                .required(false)
                .set_autocomplete(true),
        )
}

// Main function to handle interaction for the "exchange-check" command
pub async fn handle_interaction(ctx: &Context, interaction: &Interaction) {
    match interaction {
        // Handle command execution
        Interaction::Command(command) if command.data.name == COMMAND_NAME => {
            // Get values from options or use defaults
            let from: String = command
                .data
                .options
                .get(0)
                .and_then(|opt| opt.value.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| environment::get_exchange_from());

            let to: String = command
                .data
                .options
                .get(1)
                .and_then(|opt| opt.value.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| environment::get_exchange_to());

            debug!("from: {}, to: {}", from, to);

            // Generate the exchange rate message
            let exchange_rate_message = get_exchange_rate_message(from.as_str(), to.as_str()).await;

            let data = CreateInteractionResponseMessage::new().content(exchange_rate_message);
            let builder = CreateInteractionResponse::Message(data);

            // Send the response message
            if let Err(why) = command.create_response(&ctx.http, builder).await {
                log::warn!("Error creating interaction response: {:?}", why);
            }
        }

        // Handle autocomplete suggestions for "from" and "to" fields
        Interaction::Autocomplete(autocomplete) if autocomplete.data.name == COMMAND_NAME => {
            let choices = get_currency_choices();
            let response = CreateInteractionResponse::Autocomplete(choices);

            if let Err(why) = autocomplete.create_response(&ctx.http, response).await {
                warn!("Error creating autocomplete response: {:?}", why);
            }
        }

        _ => {}
    }
}

// Function to generate currency choices for autocomplete
fn get_currency_choices() -> CreateAutocompleteResponse {
    CreateAutocompleteResponse::new()
        .add_string_choice("USD", "USD")
        .add_string_choice("EUR", "EUR")
    // CreateAutocompleteResponse::new("EUR", "eur"),
    // CreateAutocompleteResponse::new("JPY", "jpy"),
    // Add more currencies as needed
}
