use crate::{environment, exchange_rate::ExchangeRateMap};

pub fn render_template(
    template: &str,
    from: &str,
    to: &str,
    current_rate: f64,
    last_rate: f64,
    current_date: &str,
    last_date: &str,
) -> String {
    template
        .replace("{FROM}", from)
        .replace("{TO}", to)
        .replace("{CURR}", &format!("{:.3}", current_rate))
        .replace("{PREV}", &format!("{:.3}", last_rate))
        .replace("{DIFF}", &format!("{:.3}", current_rate - last_rate))
        .replace("{CURR_DATE}", current_date)
        .replace("{LAST_DATE}", last_date)
}

pub fn get_prompt(rates: &Vec<ExchangeRateMap>, from: &str, to: &str) -> String {
    // let failed_rate = ExchangeRateMap::failed();
    let curr_rate = rates.get(0).cloned().unwrap_or_default();
    let last_rate = rates.get(1).cloned().unwrap_or_default();

    log::debug!("Last rate: {}", last_rate);
    log::debug!("Curr rate: {}", curr_rate);

    // Calculate the difference between current and last rate values for the specific currency pair
    let curr_val: f64 = curr_rate.get_val(from, to).unwrap_or(-1.0);
    let last_val: f64 = last_rate.get_val(from, to).unwrap_or(-1.0);

    let curr_date = curr_rate.timestamp.format("%Y-%m-%d").to_string();
    let last_date = last_rate.timestamp.format("%Y-%m-%d").to_string();
    log::debug!("Current rate value: {}", curr_val);
    log::debug!("Last rate value: {}", last_val);

    // Calculate the difference
    let diff = curr_val - last_val;

    let threshold: f64 = environment::get_exchange_rate_change_threshold();

    log::debug!("Threshold: {}", threshold);
    let template = match diff.abs() {
        diff if diff.abs() < threshold => environment::get_equal_prompt_template(),
        diff if diff < 0.0 => environment::get_increase_prompt_template(),
        diff if diff > 0.0 => environment::get_decrease_prompt_template(),
        _ => environment::get_equal_prompt_template(),
    };

    // Create the appropriate prompt based on the diff and threshold
    let prompt = render_template(
        &template, &from, &to, curr_val, last_val, &curr_date, &last_date,
    );

    log::debug!("Prompt: {}", prompt);
    prompt
}
