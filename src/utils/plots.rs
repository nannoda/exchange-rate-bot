use std::io;

use chrono::NaiveDate;
use image::{ImageEncoder, ImageError, RgbImage};
use plotters::style::WHITE;

use plotters::prelude::*;
use thiserror::Error;

use crate::exchange_rate::ExchangeRateMap;

#[derive(Error, Debug)]
pub enum PlotError {
    #[error("Failed to fill the drawing area: {0}")]
    FillError(String),

    #[error("Failed to draw text: {0}")]
    DrawTextError(String),

    #[error("Failed to draw border: {0}")]
    DrawBorderError(String),

    #[error("Failed to present the drawing area: {0}")]
    PresentError(String),

    #[error("Failed to create image from raw buffer")]
    BufferConversionError,

    #[error("Failed to encode PNG: {0}")]
    PngEncodingError(#[from] ImageError),

    #[error("I/O Error: {0}")]
    IoError(#[from] io::Error),

    #[error("No data available from {0} to {1}")]
    NoDataError(String, String),
}

pub fn get_trend_graph(
    rates: &Vec<ExchangeRateMap>,
    from: &str,
    to: &str,
) -> Result<Vec<u8>, PlotError> {
    let width = 800;
    let height = 400;

    // Extract the dates and exchange rates for the `from` and `to` currencies
    let mut data: Vec<(NaiveDate, f64)> = vec![];
    for rate_map in rates {
        if let Some(exchange_rate) = rate_map.get_val(from, to) {
            data.push((rate_map.datetime.date_naive(), exchange_rate));
        }
    }

    if data.is_empty() {
        return Err(PlotError::NoDataError(from.to_string(), to.to_string()));
    }

    // Sort data by date
    data.sort_by_key(|(date, _)| *date);

    // Prepare the buffer for the graph
    let mut buffer = vec![0; (width * height * 3) as usize];

    {
        // Create a drawing area using a bitmap backend
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();
        root.fill(&WHITE)
            .map_err(|e| PlotError::FillError(format!("{:?}", e)))?;

        // Dynamically determine the range of dates and rates
        let max_rate = data.iter().map(|(_, rate)| *rate).fold(f64::MIN, f64::max);
        let min_rate = data.iter().map(|(_, rate)| *rate).fold(f64::MAX, f64::min);
        let date_range = data.first().unwrap().0..data.last().unwrap().0;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                format!("Exchange Rate Trend: {} to {}", from, to),
                ("sans-serif", 20),
            )
            .margin(20)
            .x_label_area_size(35)
            .y_label_area_size(40)
            .build_cartesian_2d(date_range, min_rate..max_rate)
            .map_err(|e| PlotError::DrawBorderError(format!("{:?}", e)))?;

        chart
            .configure_mesh()
            .x_labels(10) // Adjust label count dynamically based on date range
            .x_label_formatter(&|date| date.format("%Y-%m-%d").to_string())
            .y_labels(10) // Add more granularity to the y-axis
            .y_label_formatter(&|rate| format!("{:.2}", rate)) // Format the exchange rates
            .y_desc("Exchange Rate")
            .x_desc("Date")
            .axis_desc_style(("sans-serif", 15))
            .draw()
            .map_err(|e| PlotError::DrawTextError(format!("{:?}", e)))?;

        chart
            .draw_series(LineSeries::new(
                data.iter().map(|(date, rate)| (*date, *rate)),
                &BLUE,
            ))
            .map_err(|e| PlotError::DrawTextError(format!("{:?}", e)))?
            .label(format!("{} to {}", from, to))
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLUE));

        chart
            .configure_series_labels()
            .background_style(&WHITE)
            .border_style(&BLACK)
            .draw()
            .map_err(|e| PlotError::DrawTextError(format!("{:?}", e)))?;
    }

    // Convert the raw buffer into a PNG image buffer
    let img = RgbImage::from_raw(width, height, buffer).ok_or(PlotError::BufferConversionError)?;

    let mut png_data = Vec::new();
    {
        let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
        encoder
            .write_image(&img, width, height, image::ExtendedColorType::Rgb8)
            .map_err(PlotError::PngEncodingError)?;
    }

    Ok(png_data)
}
