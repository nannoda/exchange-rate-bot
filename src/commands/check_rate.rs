use log::debug;
use serenity::all::{
    CommandOptionType, CreateAttachment, CreateEmbed, CreateInteractionResponseMessage,
    CreateMessage, EditInteractionResponse,
};
use serenity::builder::{CreateAutocompleteResponse, CreateCommand, CreateCommandOption};
use serenity::model::application::{ResolvedOption, ResolvedValue};

use crate::environment::{self};
use crate::utils::get_exchange_rate_message;

pub const COMMAND_NAME: &str = "exchange-check";

pub fn register() -> CreateCommand {
    CreateCommand::new(COMMAND_NAME)
        .description("Check exchange rate between two currencies")
        // Localization for command name and description
        .name_localized("de", "wechselkurs")
        .description_localized(
            "de",
            "Überprüfen Sie den Wechselkurs zwischen zwei Währungen",
        )
        // .name_localized("hi", "मुद्रा विनिमय")
        .description_localized("hi", "दो मुद्राओं के बीच विनिमय दर की जाँच करें")
        .name_localized("ja", "為替レート")
        .description_localized("ja", "二つの通貨間の為替レートを確認します")
        .name_localized("es-ES", "tipo-de-cambio")
        .description_localized("es-ES", "Consulta el tipo de cambio entre dos monedas")
        // "From" option localization
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "from",
                "Currency to convert from",
            )
            .name_localized("de", "von")
            .description_localized("de", "Die Ausgangswährung für die Konvertierung")
            // .name_localized("hi", "मूल मुद्रा")
            .description_localized("hi", "जिस मुद्रा से परिवर्तित करना है")
            .name_localized("ja", "変換元")
            .description_localized("ja", "変換する通貨")
            .name_localized("es-ES", "de")
            .description_localized("es-ES", "Moneda de origen para la conversión")
            .required(false)
            .set_autocomplete(true),
        )
        // "To" option localization
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "to", "Currency to convert to")
                .name_localized("de", "zu")
                .description_localized("de", "Die Zielwährung für die Konvertierung")
                // .name_localized("hi", "लक्ष्य मुद्रा")
                .description_localized("hi", "जिस मुद्रा में परिवर्तित करना है")
                .name_localized("ja", "変換先")
                .description_localized("ja", "変換される通貨")
                .name_localized("es-ES", "a")
                .description_localized("es-ES", "Moneda de destino para la conversión")
                .required(false)
                .set_autocomplete(true),
        )
}

pub async fn run(options: &[ResolvedOption<'_>]) -> EditInteractionResponse {
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
    let msg = get_exchange_rate_message(from.as_str(), to.as_str()).await;
    return EditInteractionResponse::new()
        .content(msg.message)
        // .embed(CreateEmbed::new().image("attachment://graph.svg"))
        .new_attachment(CreateAttachment::bytes(msg.graph, "graph.png"));
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
