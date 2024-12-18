use std::io;

use chrono::NaiveDate;
use image::{
    codecs::png::PngEncoder, ColorType, ExtendedColorType, ImageBuffer, ImageEncoder, ImageError,
    RgbImage,
};
use plotters::style::{
    text_anchor::{HPos, Pos, VPos},
    IntoFont, RED, WHITE,
};

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

pub fn get_error_png(error_msg: &str) -> Result<Vec<u8>, PlotError> {
    let width = 800;
    let height = 200;
    let font_size = 60;

    // Calculate the correct buffer size (width * height * 3 for RGB)
    let buffer_size = (width * height * 3) as usize;
    let mut buffer = vec![0; buffer_size];

    {
        // Create a bitmap backend to draw to the buffer
        let root = BitMapBackend::with_buffer(&mut buffer, (width, height)).into_drawing_area();

        // Fill the drawing area with white color
        root.fill(&WHITE)
            .map_err(|e| PlotError::FillError(e.to_string()))?;

        // Define the text style
        let text_style = ("sans-serif", font_size).into_font().color(&RED);

        // Draw the error message text
        root.draw_text(
            error_msg,
            &text_style,
            (30, height as i32 / 2 - (font_size / 2)),
        )
        .map_err(|e| PlotError::DrawTextError(e.to_string()))?;

        // Draw a border around the error message
        root.draw(&Rectangle::new(
            [
                (10 as i32, 10 as i32),
                (width as i32 - 10, height as i32 - 10),
            ],
            ShapeStyle {
                color: RED.to_rgba(),
                filled: false,
                stroke_width: 3,
            },
        ))
        .map_err(|e| PlotError::DrawBorderError(e.to_string()))?;

        // Finalize the drawing area
        root.present()
            .map_err(|e| PlotError::PresentError(e.to_string()))?;
    }

    // Convert the raw buffer into an ImageBuffer (for PNG encoding)
    let img = RgbImage::from_raw(width, height, buffer).ok_or(PlotError::BufferConversionError)?;

    // Encode the image into PNG format
    let mut png_data = Vec::new();
    let encoder = PngEncoder::new(&mut png_data);
    encoder
        .write_image(&img, width, height, image::ExtendedColorType::Rgb8)
        .map_err(PlotError::PngEncodingError)?;

    Ok(png_data)
}

// Assuming your existing imports and struct definitions...
pub fn get_trend_graph(
    rates: &Vec<ExchangeRateMap>,
    from: &str,
    to: &str,
) -> Result<Vec<u8>, PlotError> {
    let width = 800;
    let height = 600;

    // Extract the dates and exchange rates for the `from` and `to` currencies
    let mut data: Vec<(NaiveDate, f64)> = vec![];
    for rate_map in rates {
        if let Some(exchange_rate) = rate_map.get_val(from, to) {
            data.push((rate_map.date, exchange_rate));
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
            .x_labels(7)
            .x_label_formatter(&|date| date.format("%Y-%m-%d").to_string())
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
