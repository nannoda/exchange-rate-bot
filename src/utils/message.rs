use chrono::{Duration, Utc};

use crate::{
    database::exchange_rate::save_exchange_rate,
    environment,
    exchange_rate::ExchangeRateMap,
    llm::{self, generate::generate_sentence, prompt::get_prompt},
};

use super::plots::get_trend_graph;

pub struct ExchangeRateMessage {
    pub message: String,
    pub graph: Option<Vec<u8>>,
}

pub async fn get_exchange_rate_message(from: &str, to: &str) -> ExchangeRateMessage {
    // Calculate the date 7 days ago
    let from_date = (Utc::now() - Duration::days(30)).date_naive();

    let rates = ExchangeRateMap::get_rates(from_date, Some(from.into())).await;

    match rates {
        Ok(rates) => {
            // Print out rates
            for r in &rates {
                log::debug!("{}",r);
            };
            let prompt = get_prompt(&rates, from, to);

            let rate: f64 = rates
                .get(0)
                .cloned()
                .unwrap_or_default()
                .get_val(from, to)
                .unwrap_or(-1.0);

            // Save rate for backward compatibility reason.
            save_exchange_rate(from, to, rate);

            // keep track how much time it takes to generate the sentence
            let start = std::time::Instant::now();

            let llm_res = generate_sentence(prompt.as_str()).await;

            let elapsed_llm = start.elapsed();

            let start_graph = std::time::Instant::now();
            let graph_result = get_trend_graph(&rates, from, to);
            let elapsed_graph = start_graph.elapsed();
            let elapsed_total = start.elapsed();

            let graph_message = match &graph_result {
                Ok(_) => String::new(), // No additional message if there's no error
                Err(err) => format!("\nGraph generation error: {}", err), // Include error message
            };

            let msg = format!(
                "{}\n\
                ```\n\
                1 {} = {} {}\n\
                Searched in {}.{:03} seconds\n\
                Model Loaded in {}.{:03} seconds\n\
                Evaluated in {}.{:03} seconds\n\
                Graph generated in {}.{:03} seconds{}\n\
                Generated in {}.{:03} seconds\n\
                ```",
                llm_res.content,

                from,
                rate,
                to,

                llm_res.search_duration.as_secs(),
                llm_res.search_duration.subsec_millis(),

                llm_res.load_duration.as_secs(),
                llm_res.load_duration.subsec_millis(),

                llm_res.eval_duration.as_secs(),
                llm_res.eval_duration.subsec_millis(),

                elapsed_graph.as_secs(),
                elapsed_graph.subsec_millis(),

                graph_message, // Add the error message dynamically
                
                elapsed_total.as_secs(),
                elapsed_total.subsec_millis(),
            );

            ExchangeRateMessage{
                message: msg,
                graph: graph_result.ok()
            }
        }
        Err(e) => {
            ExchangeRateMessage{
                message:format!("Error fetching API. Please verify the API URL and Internet connection. URL used: `{}`\n`Error: {:?}`", 
                environment::get_exchange_rate_api_url(), e),
                graph: None
            }
        }
    }
}
