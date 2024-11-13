use serenity::all::{CommandOptionType, Interaction};
use serenity::builder::{CreateCommand, CreateCommandOption, CreateAutocompleteResponse};
// use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue, interaction::{Interaction, InteractionResponseType}};
use serenity::prelude::*;
use serenity::http::Http;
use log::{debug, warn};

use crate::environment::{self, ensure_environment};
use crate::utils::get_exchange_rate_message;

// Function to register the exchange command with optional "from" and "to" parameters
pub fn register() -> CreateCommand {
    CreateCommand::new("exchange-check")
        .description("Check exchange rate between two currencies")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "from", "Currency to convert from")
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
        Interaction::Command(command) if command.data.name == "exchange-check" => {
            // Get values from options or use defaults
            // let from = command
            //     .data
            //     .options
            //     .get(0)
            //     .and_then(|opt| opt.value.as_str())
            //     .unwrap_or_else(|| environment::get_exchange_from().as_str());

            // let to = command
            //     .data
            //     .options
            //     .get(1)
            //     .and_then(|opt| opt.value.as_str())
            //     .unwrap_or_else(|| environment::get_exchange_to().as_str());

            // debug!("from: {}, to: {}", from, to);

            // Generate the exchange rate message
            // let exchange_rate_message = get_exchange_rate_message(from, to).await;

            // Send the response message
            // if let Err(why) = command.create_response(
            //     &ctx.http,
            //     InteractionResponseType::ChannelMessageWithSource,
            //     |response| response.content(exchange_rate_message),
            // ).await {
            //     warn!("Error creating interaction response: {:?}", why);
            // }
        }

        // Handle autocomplete suggestions for "from" and "to" fields
        Interaction::Autocomplete(autocomplete) if autocomplete.data.name == "exchange-check" => {
            let choices = get_currency_choices();
            // let response = CreateAutocompleteResponse::new().add_choices(choices);

            // if let Err(why) = autocomplete.create_response(&ctx.http, response).await {
            //     warn!("Error creating autocomplete response: {:?}", why);
            // }
        }

        _ => {}
    }
}

// Function to generate currency choices for autocomplete
fn get_currency_choices() -> Vec<CreateAutocompleteResponse> {
    vec![
        // CreateAutocompleteResponse::new("USD", "usd"),
        // CreateAutocompleteResponse::new("EUR", "eur"),
        // CreateAutocompleteResponse::new("JPY", "jpy"),
        // Add more currencies as needed
    ]
}
