use serenity::all::{
    CommandDataOptionValue, CommandOptionType, CreateInteractionResponse,
    CreateInteractionResponseMessage, Interaction,
};
use serenity::builder::{CreateAutocompleteResponse, CreateCommand, CreateCommandOption};
use serenity::model::application::{ResolvedOption, ResolvedValue};
// use serenity::model::application::{CommandOptionType, ResolvedOption, ResolvedValue, interaction::{Interaction, InteractionResponseType}};
use log::{debug, warn};
use serenity::prelude::*;

use crate::environment::{self};
use crate::utils::get_exchange_rate_message;

pub const COMMAND_NAME: &str = "exchange-check";

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

pub async fn run(options: &[ResolvedOption<'_>]) -> String {
    let from = options
        .iter()
        .find(|opt| opt.name == "from")
        .and_then(|opt| match &opt.value {
            ResolvedValue::String(s) => Some(s),
            _ => None,
        })
        .map(|s| s.to_string())
        .unwrap_or_else(|| environment::get_exchange_from());
    let to = options
        .iter()
        .find(|opt| opt.name == "to")
        .and_then(|opt| match &opt.value {
            ResolvedValue::String(s) => Some(s),
            _ => None,
        })
        .map(|s| s.to_string())
        .unwrap_or_else(|| environment::get_exchange_to());
    debug!("from: {}, to: {}", from, to);
    // Generate the exchange rate message
    let exchange_rate_message = get_exchange_rate_message(from.as_str(), to.as_str()).await;

    return exchange_rate_message;
}

pub fn autocomplete(input: &str) -> CreateAutocompleteResponse {
    let mut response = CreateAutocompleteResponse::new();
    let filtered_currencies = CURRENCIES
        .iter()
        .filter(|&&currency| currency.starts_with(&input.to_uppercase()))
        .take(25); // Limit results to 25

    for &currency in filtered_currencies {
        response = response.add_string_choice(currency, currency);
    }
    response
}

// Define a constant vector of currency codes
// Only provide popular one due to length limit
const CURRENCIES: &[&str] = &[
    "AED", "AFN", "ALL", "AMD", "ANG", "AOA", "ARS", "AUD", "AWG", "AZN", "BAM", "BBD", "BDT",
    "BGN", "BHD", "BIF", "BMD", "BND", "BOB", "BRL", "BSD", "BTC", "BTN", "BWP", "BYN", "BYR",
    "BZD", "CAD", "CDF", "CHF", "CLF", "CLP", "CNY", "CNH", "COP", "CRC", "CUC", "CUP", "CVE",
    "CZK", "DJF", "DKK", "DOP", "DZD", "EGP", "ERN", "ETB", "EUR", "FJD", "FKP", "GBP", "GEL",
    "GGP", "GHS", "GIP", "GMD", "GNF", "GTQ", "GYD", "HKD", "HNL", "HRK", "HTG", "HUF", "IDR",
    "ILS", "IMP", "INR", "IQD", "IRR", "ISK", "JEP", "JMD", "JOD", "JPY", "KES", "KGS", "KHR",
    "KMF", "KPW", "KRW", "KWD", "KYD", "KZT", "LAK", "LBP", "LKR", "LRD", "LSL", "LTL", "LVL",
    "LYD", "MAD", "MDL", "MGA", "MKD", "MMK", "MNT", "MOP", "MRU", "MUR", "MVR", "MWK", "MXN",
    "MYR", "MZN", "NAD", "NGN", "NIO", "NOK", "NPR", "NZD", "OMR", "PAB", "PEN", "PGK", "PHP",
    "PKR", "PLN", "PYG", "QAR", "RON", "RSD", "RUB", "RWF", "SAR", "SBD", "SCR", "SDG", "SEK",
    "SGD", "SHP", "SLE", "SLL", "SOS", "SRD", "STD", "SVC", "SYP", "SZL", "THB", "TJS", "TMT",
    "TND", "TOP", "TRY", "TTD", "TWD", "TZS", "UAH", "UGX", "USD", "UYU", "UZS", "VEF", "VES",
    "VND", "VUV", "WST", "XAF", "XAG", "XAU", "XCD", "XDR", "XOF", "XPF", "YER", "ZAR", "ZMK",
    "ZMW", "ZWL",
];
