use crate::database::CategorySummary;
use image::ExtendedColorType;
use image::ImageEncoder;
use image::codecs::png::PngEncoder;
use plotters::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VisualizationError {
    #[error("Chart generation failed: {0}")]
    ChartError(String),

    #[error("No data to visualize")]
    NoData,
}

const CHART_WIDTH: u32 = 800;
const CHART_HEIGHT: u32 = 600;
const COLORS: &[RGBColor] = &[
    RGBColor(75, 192, 192),
    RGBColor(255, 99, 132),
    RGBColor(255, 205, 86),
    RGBColor(54, 162, 235),
    RGBColor(153, 102, 255),
    RGBColor(255, 159, 64),
    RGBColor(199, 199, 199),
    RGBColor(83, 102, 255),
    RGBColor(255, 99, 255),
    RGBColor(99, 255, 132),
];

pub fn generate_pie_chart(data: &[CategorySummary]) -> Result<Vec<u8>, VisualizationError> {
    if data.is_empty() {
        return Err(VisualizationError::NoData);
    }

    let mut buffer = vec![0u8; (CHART_WIDTH * CHART_HEIGHT * 3) as usize];
    let mut png_buffer = Vec::new();
    {
        let root = BitMapBackend::with_buffer(&mut buffer, (CHART_WIDTH, CHART_HEIGHT))
            .into_drawing_area();

        root.fill(&WHITE)
            .map_err(|e| VisualizationError::ChartError(e.to_string()))?;

        let total: i64 = data.iter().map(|s| s.total).sum();
        let center = ((CHART_WIDTH / 2) as i32, (CHART_HEIGHT / 2) as i32);
        let radius = 180.0;

        let mut current_angle = 0.0;

        // Draw pie slices
        for (idx, summary) in data.iter().enumerate() {
            let angle_size = (summary.total as f64 / total as f64) * 360.0;
            let color = COLORS[idx % COLORS.len()];

            draw_pie_slice(&root, center, radius, current_angle, angle_size, &color)
                .map_err(|e| VisualizationError::ChartError(e.to_string()))?;

            current_angle += angle_size;
        }

        // Draw legend with text
        let legend_x = 20;
        let legend_y_start = 50;
        let legend_spacing = 35;
        let box_size = 20;

        for (idx, summary) in data.iter().enumerate() {
            let color = COLORS[idx % COLORS.len()];
            let y = legend_y_start + (idx as i32 * legend_spacing);
            let percentage = (summary.total as f64 / total as f64) * 100.0;

            // Draw colored box
            root.draw(&Rectangle::new(
                [(legend_x, y), (legend_x + box_size, y + box_size)],
                color.filled(),
            ))
            .map_err(|e| VisualizationError::ChartError(e.to_string()))?;

            // Draw text using embedded font
            let text = format!("{} ({:.1}%)", summary.category, percentage);
            root.draw(&Text::new(
                text,
                (legend_x + box_size + 10, y + 5),
                ("sans-serif", 16).into_font(),
            ))
            .map_err(|e| VisualizationError::ChartError(e.to_string()))?;
        }

        root.present()
            .map_err(|e| VisualizationError::ChartError(e.to_string()))?;
    }

    // Encode RGB buffer to PNG
    let encoder = PngEncoder::new(&mut png_buffer);
    encoder
        .write_image(&buffer, CHART_WIDTH, CHART_HEIGHT, ExtendedColorType::Rgb8)
        .map_err(|e| VisualizationError::ChartError(e.to_string()))?;

    Ok(png_buffer)
}

fn draw_pie_slice<DB: DrawingBackend>(
    root: &DrawingArea<DB, plotters::coord::Shift>,
    center: (i32, i32),
    radius: f64,
    start_angle: f64,
    angle_size: f64,
    color: &RGBColor,
) -> Result<(), DrawingAreaErrorKind<DB::ErrorType>> {
    let mut points = vec![center];

    let steps = (angle_size.abs() / 2.0).max(10.0) as i32;
    for i in 0..=steps {
        let angle = start_angle + (angle_size * i as f64 / steps as f64);
        let x = center.0 + (angle.to_radians().cos() * radius) as i32;
        let y = center.1 + (angle.to_radians().sin() * radius) as i32;
        points.push((x, y));
    }

    points.push(center);

    root.draw(&Polygon::new(points, color.filled()))?;

    Ok(())
}
