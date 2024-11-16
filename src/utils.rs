use std::borrow::Borrow;

use log::warn;
use plotters::prelude::*;
use rusqlite::Connection;
use serde_json::Value;
use serenity::all::CreateMessage;


use plotters::style::text_anchor::{HPos, Pos, VPos};
use image::{ColorType, ExtendedColorType, ImageBuffer, ImageEncoder, RgbImage};


use crate::{ database::exchange_rate::save_exchange_rate, environment::{self, get_exchange_rate_api_url}, exchange_rate::{get_exchange_rates, ExchangeRateMap}, llm::{generate::generate_sentence, prompt::get_prompt}
};

const DEFAULT_TIME_SECONDS: u64 = 86400;

pub fn string_to_time_second(s: &str) -> u64 {
    // convert string to lower case
    let s = s.to_lowercase();

    // if string ends with 's', remove it and convert to integer
    if s.ends_with("s") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value,
            Err(_) => {
                warn!(
                    "Failed to parse seconds from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if string ends with 'm', remove it and convert to integer
    else if s.ends_with("m") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value * 60,
            Err(_) => {
                warn!(
                    "Failed to parse minutes from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if string ends with 'h', remove it and convert to integer
    else if s.ends_with("h") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value * 60 * 60,
            Err(_) => {
                warn!(
                    "Failed to parse hours from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if string ends with 'd', remove it and convert to integer
    else if s.ends_with("d") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value * 60 * 60 * 24,
            Err(_) => {
                warn!(
                    "Failed to parse days from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if string ends with 'w', remove it and convert to integer
    else if s.ends_with("w") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value * 60 * 60 * 24 * 7,
            Err(_) => {
                warn!(
                    "Failed to parse weeks from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if string ends with 'y', remove it and convert to integer
    else if s.ends_with("y") {
        match s[..s.len() - 1].parse::<u64>() {
            Ok(value) => value * 60 * 60 * 24 * 365,
            Err(_) => {
                warn!(
                    "Failed to parse years from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
    // if no suffix, just parse as a plain integer
    else {
        match s.parse::<u64>() {
            Ok(value) => value,
            Err(_) => {
                warn!(
                    "Failed to parse integer from '{}'. Returning default value: {}",
                    s, DEFAULT_TIME_SECONDS
                );
                DEFAULT_TIME_SECONDS
            }
        }
    }
}

fn get_error_png(error_msg: &str) -> Vec<u8> {
    let width = 800;
    let height = 200;
    let font_size = 60;

    // Calculate the correct buffer size (width * height * 3 for RGB)
    let buffer_size = (width * height * 3) as usize;
    let mut buffer = vec![0; buffer_size];

  
    {
        // Create a bitmap backend to draw to the buffer
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        // Define the text style
        let text_style = ("sans-serif", font_size).into_font().color(&RED);

        // Draw the error message text
        root.draw_text(error_msg, &text_style, (30, height as i32 / 2 -(font_size/2) ))
            .unwrap();

        // Draw a border around the error message
        root.draw(&Rectangle::new(
            [(10 as i32, 10 as i32), (width as i32 - 10, height as i32 - 10)],
            ShapeStyle {
                color: RED.to_rgba(),
                filled: false,
                stroke_width: 3,
            },
        ))
        .unwrap();

        // Finalize the drawing area
        root.present().unwrap();
    }

    // Convert the raw buffer into an ImageBuffer (for PNG encoding)
    let img = RgbImage::from_raw(width, height, buffer)
        .expect("Failed to create image from raw buffer");
  // Encode the image into PNG format
  let mut png_data = Vec::new();
  {
      let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
      encoder
          .write_image(
              &img,
              width,
              height,
              ExtendedColorType::Rgb8,
          )
          .expect("Failed to encode PNG");
  }
    png_data
}

use chrono::NaiveDate;
use plotters::prelude::*;
use std::collections::HashMap;

// Assuming your existing imports and struct definitions...

fn get_trend_graph(rates: &Vec<ExchangeRateMap>, from: &str, to: &str) -> Vec<u8> {
    let width = 800;
    let height = 400;

    // Extract the dates and exchange rates for the `from` and `to` currencies
    let mut data: Vec<(NaiveDate, f64)> = vec![];
    for rate_map in rates {
        if let Some(exchange_rate) = rate_map.get_val(from, to) {
            data.push((rate_map.date, exchange_rate));
        }
    }

    if data.is_empty() {
        panic!("No exchange rate data available for the given currencies.");
    }

    // Sort data by date
    data.sort_by_key(|(date, _)| *date);

    // Prepare the buffer for the graph
    let mut buffer = vec![0; (width * height * 3) as usize];

    {
        // Create a drawing area using a bitmap backend
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let max_rate = data.iter().map(|(_, rate)| *rate).fold(f64::MIN, f64::max);
        let min_rate = data.iter().map(|(_, rate)| *rate).fold(f64::MAX, f64::min);

        let date_range = data.first().unwrap().0..data.last().unwrap().0;

        let mut chart = ChartBuilder::on(&root)
            .caption(format!("Exchange Rate Trend: {} to {}", from, to), ("sans-serif", 20))
            .margin(20)
            .x_label_area_size(35)
            .y_label_area_size(40)
            .build_cartesian_2d(date_range, min_rate..max_rate)
            .unwrap();

        chart.configure_mesh()
            .x_labels(7)
            .x_label_formatter(&|date| date.format("%Y-%m-%d").to_string())
            .y_desc("Exchange Rate")
            .x_desc("Date")
            .axis_desc_style(("sans-serif", 15))
            .draw()
            .unwrap();

        chart.draw_series(LineSeries::new(
            data.iter().map(|(date, rate)| (*date, *rate)),
            &BLUE,
        )).unwrap()
        .label(format!("{} to {}", from, to))
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        chart.configure_series_labels()
            .background_style(&WHITE)
            .border_style(&BLACK)
            .draw()
            .unwrap();
    }

    // Convert the raw buffer into a PNG image buffer
    let img = RgbImage::from_raw(width, height, buffer)
        .expect("Failed to create image from raw buffer");

    let mut png_data = Vec::new();
    {
        let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
        encoder
            .write_image(
                &img,
                width,
                height,
                image::ExtendedColorType::Rgb8,
            )
            .expect("Failed to encode PNG");
    }

    png_data
}



pub struct ExchangeRateMessage {
    pub message: String,
    pub graph: Vec<u8>,
}

pub async fn get_exchange_rate_message(from: &str, to: &str) -> ExchangeRateMessage {
    // let rate_result = get_exchange_rate(from, to).await;

    let rates = get_exchange_rates().await;

    match rates {
        Ok(rates) => {
            // Print out rates
            for r in &rates {
                log::debug!("{}",r);
            }
            

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
            let graph = get_trend_graph(&rates, from, to);
            let elapsed_graph = start_graph.elapsed();
            let elapsed_total = start.elapsed();

            ExchangeRateMessage{
                message: format!(
                    "{}\n\
                    ```\n\
                    1 {} = {} {}\n\
                    Message generated in {}.{:03} seconds\n\
                    Graph generated in {}.{:03} seconds\n\
                    Generated in {}.{:03} seconds\n\
                    ```",
                    llm_res,
                    from,
                    rate,
                    to,
                    elapsed_llm.as_secs(),
                    elapsed_llm.subsec_millis(),
                    elapsed_graph.as_secs(),
                    elapsed_graph.subsec_millis(),
                    elapsed_total.as_secs(),
                    elapsed_total.subsec_millis(),
                ),
                graph: graph
            }
           
        }
        Err(e) => {
            match e {
                crate::exchange_rate::GetRatesError::RemoteError(fetch_exchange_rate_error) => ExchangeRateMessage{
                    message:format!("Error fetching API. Please verify the API URL or API key. URL used: `{}`\n`Error: {:?}`", 
                    environment::get_exchange_rate_api_url(), 
                    fetch_exchange_rate_error),
                    graph: get_error_png("Remote Error")  
                },
                // crate::exchange_rate::GetRatesError::LocalError(local_exchange_rate_error) => ExchangeRateMessage{
                //     message: format!("Error reading local database.\n`Error: {:?}`", local_exchange_rate_error),
                //     graph: get_error_png("Local Error")
                // }
            }
        }
    }

}
