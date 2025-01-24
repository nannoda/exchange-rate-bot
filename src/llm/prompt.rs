use std::fmt::format;

use chrono::{DateTime, Utc};

use crate::{
    environment,
    exchange_rate::ExchangeRateMap,
    utils::search::{get_news, search_date},
};

pub fn render_template(
    template: &str,
    from: &str,
    to: &str,
    diff: f64,
    current_rate: f64,
    last_rate: f64,
    current_date: &str,
    last_date: &str,
) -> String {
    template
        .replace("{FROM}", from)
        .replace("{TO}", to)
        .replace("{CURR}", &format!("{:.4}", current_rate))
        .replace("{PREV}", &format!("{:.4}", last_rate))
        .replace("{DIFF}", &format!("{:.4}", diff))
        .replace("{CURR_DATE}", current_date)
        .replace("{LAST_DATE}", last_date)
}

pub fn get_prompt(rates: &Vec<ExchangeRateMap>, from: &str, to: &str) -> String {
    // let failed_rate = ExchangeRateMap::failed();
    let curr_rate = rates.last().cloned().unwrap_or_default();
    let last_rate = rates.get(rates.len() - 2).cloned().unwrap_or_default();

    log::debug!("Last rate: {}", last_rate);
    log::debug!("Curr rate: {}", curr_rate);

    // Calculate the difference between current and last rate values for the specific currency pair
    let curr_val: f64 = curr_rate.get_val(from, to).unwrap_or(-1.0);
    let last_val: f64 = last_rate.get_val(from, to).unwrap_or(-1.0);

    let curr_date = curr_rate.datetime.format("%Y-%m-%d").to_string();
    let last_date = last_rate.datetime.format("%Y-%m-%d").to_string();
    log::debug!("Current rate value: {}", curr_val);
    log::debug!("Last rate value: {}", last_val);

    // Calculate the difference
    let diff = curr_val - last_val;

    let threshold: f64 = environment::get_exchange_rate_change_threshold();

    log::debug!("Threshold: {}", threshold);
    let template = if diff.abs() < threshold {
        environment::get_equal_prompt_template()
    } else if diff > 0.0 {
        environment::get_increase_prompt_template()
    } else if diff < 0.0 {
        environment::get_decrease_prompt_template()
    } else {
        environment::get_equal_prompt_template()
    };

    // Create the appropriate prompt based on the diff and threshold
    let prompt = render_template(
        &template, &from, &to, diff, curr_val, last_val, &curr_date, &last_date,
    );

    log::debug!("Prompt: {}", prompt);
    prompt
}

pub async fn get_news_prompt(date: DateTime<Utc>) -> String {
    let news_list = get_news(date, 5).await;

    let mut prompt = match news_list.len() {
        0 => format!(""),
        _ => format!("News on {}", date.format("%d/%m/%Y")),
    };

    for news in news_list {
        prompt = prompt
            + format!(
                "\n# {}\n{}\n[link]({})\n",
                &news.title.unwrap_or("No Title".to_string()),
                &news.content.unwrap_or("No content...".to_string()),
                &news.url.unwrap_or("N/A".to_string())
            )
            .as_str()
    }

    prompt = prompt + "\n";

    log::debug!("News Prompt: {prompt}");
    return prompt;
}

pub async fn get_date_prompt(date: DateTime<Utc>) -> String {
    let news_list = search_date(date, 5).await;

    let mut prompt = match news_list.len() {
        0 => format!(""),
        _ => format!("Search result date [d/m]: {}", date.format("%d/%m")),
    };

    for news in news_list {
        prompt = prompt
            + format!(
                "\n# {}\n{}\n[link]({})\n",
                &news.title.unwrap_or("No Title".to_string()),
                &news.content.unwrap_or("No content...".to_string()),
                &news.url.unwrap_or("N/A".to_string())
            )
            .as_str()
    }

    prompt = prompt + "\n";

    log::debug!("Date Prompt: {prompt}");
    return prompt;
}
