use std::path::PathBuf;

use anyhow::Result;
use plotters::{prelude::*, style::full_palette::BROWN};

use crate::Db;

pub async fn plot(db: &Db) -> Result<()> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("plot.png");
    let root = BitMapBackend::new(&path, (640, 480)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(5)
        .caption("Håkan", ("sans-serif", 30.0))
        .build_cartesian_2d(0f64..4., 0f64..10.)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(WHITE.mix(0.3))
        .y_desc("Håkan")
        .x_desc("Tid")
        .axis_desc_style(("sans-serif", 15))
        .draw()?;

    let x_values = [(0.0f64, 2.0f64), (1., 3.), (2., 4.), (3., 6.), (4., 0.)];

    chart.draw_series(LineSeries::new(x_values, BROWN))?;

    root.present()?;
    Ok(())
}
