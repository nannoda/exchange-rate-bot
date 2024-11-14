use serenity::all::{
    CommandDataOption, CommandDataOptionValue, CommandOptionType, CreateInteractionResponse, CreateInteractionResponseMessage, Interaction
};
use serenity::builder::{CreateAutocompleteResponse, CreateCommand, CreateCommandOption};
// use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue, interaction::{Interaction, InteractionResponseType}};
use log::{debug, warn};
use serenity::prelude::*;

use crate::environment::{self};
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
                .iter()
                .find(|opt| opt.name =="from")
                .and_then(|opt| opt.value.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| environment::get_exchange_from());

            let to: String = command
                .data
                .options
                .iter()
                .find(|opt| opt.name =="to")
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
            if autocomplete.data.name == COMMAND_NAME {
                // Find the autocomplete option by iterating through all options
                if let Some(autocomplete_option) = autocomplete
                    .data
                    .options
                    .iter()
                    .find_map(|option| 
                        if let CommandDataOptionValue::Autocomplete { value, .. } = &option.value {
                            Some(value)
                        } else {
                            None
                        }
                    )
                {
                    // Generate the currency choices based on the user's input
                    let choices = get_currency_choices(autocomplete_option);
                    let response = CreateInteractionResponse::Autocomplete(choices);
    
                    // Send the autocomplete response
                    if let Err(why) = autocomplete.create_response(&ctx.http, response).await {
                        warn!("Error creating autocomplete response: {:?}", why);
                    }
                }
            }    
        }

        _ => {}
    }
}


// Define a constant vector of currency codes
// Only provide popular one due to length limit
const CURRENCIES: &[&str] = &[
    "AED", "AFN", "ALL", "AMD", "ANG", "AOA", "ARS", "AUD", "AWG", "AZN", "BAM", "BBD", "BDT", "BGN", "BHD", "BIF",
    "BMD", "BND", "BOB", "BRL", "BSD", "BTC", "BTN", "BWP", "BYN", "BYR", "BZD", "CAD", "CDF", "CHF", "CLF", "CLP",
    "CNY", "CNH", "COP", "CRC", "CUC", "CUP", "CVE", "CZK", "DJF", "DKK", "DOP", "DZD", "EGP", "ERN", "ETB", "EUR",
    "FJD", "FKP", "GBP", "GEL", "GGP", "GHS", "GIP", "GMD", "GNF", "GTQ", "GYD", "HKD", "HNL", "HRK", "HTG", "HUF",
    "IDR", "ILS", "IMP", "INR", "IQD", "IRR", "ISK", "JEP", "JMD", "JOD", "JPY", "KES", "KGS", "KHR", "KMF", "KPW",
    "KRW", "KWD", "KYD", "KZT", "LAK", "LBP", "LKR", "LRD", "LSL", "LTL", "LVL", "LYD", "MAD", "MDL", "MGA", "MKD",
    "MMK", "MNT", "MOP", "MRU", "MUR", "MVR", "MWK", "MXN", "MYR", "MZN", "NAD", "NGN", "NIO", "NOK", "NPR", "NZD",
    "OMR", "PAB", "PEN", "PGK", "PHP", "PKR", "PLN", "PYG", "QAR", "RON", "RSD", "RUB", "RWF", "SAR", "SBD", "SCR",
    "SDG", "SEK", "SGD", "SHP", "SLE", "SLL", "SOS", "SRD", "STD", "SVC", "SYP", "SZL", "THB", "TJS", "TMT", "TND",
    "TOP", "TRY", "TTD", "TWD", "TZS", "UAH", "UGX", "USD", "UYU", "UZS", "VEF", "VES", "VND", "VUV", "WST", "XAF",
    "XAG", "XAU", "XCD", "XDR", "XOF", "XPF", "YER", "ZAR", "ZMK", "ZMW", "ZWL",
];


fn get_currency_choices(input: &str) -> CreateAutocompleteResponse {
    let mut response = CreateAutocompleteResponse::new();
    let filtered_currencies = CURRENCIES
        .iter()
        .filter(|&&currency| currency.starts_with(&input.to_uppercase()))
        .take(25);  // Limit results to 25

    for &currency in filtered_currencies {
        response = response.add_string_choice(currency, currency);
    }
    response
}
