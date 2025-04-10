use std::path::PathBuf;

use anyhow::{bail, Result};
use plotters::prelude::*;
use uuid::Uuid;

use crate::Db;

pub async fn plot(db: &Db) -> Result<PathBuf> {
    let uuid = Uuid::new_v4().to_string();
    let path = PathBuf::from(r"C:\Users\bobbo\Documents\Projects\Rust\grimstabot\plots")
        .join(uuid)
        .with_extension("png");

    let path_2 = path.clone();

    let records = sqlx::query!(
        r"
        WITH p AS (
            SELECT 
            products.report_id,
            (products.comparative_price * ingredients.amount) price
            FROM products
            JOIN ingredients
            ON ingredients.id = products.ingredient_id
            )
            SELECT
            reports.id,
            reports.created_at,
            SUM(p.price) price
            FROM reports
            JOIN p 
            ON p.report_id = reports.id
            GROUP BY reports.id
            ORDER BY created_at ASC",
    )
    .fetch_all(db)
    .await?;

    //dbg!(&records);

    if records.is_empty() {
        bail!("no records in time range");
    }

    let last = records.iter().last().unwrap();

    let start_date = records[0].created_at.and_utc();
    let end_date = last.created_at.and_utc();

    let min_price = records
        .iter()
        .map(|record| record.price)
        .fold(f64::INFINITY, |a, b| a.min(b));
    let max_price = records
        .iter()
        .map(|record| record.price)
        .fold(f64::NEG_INFINITY, |a, b| a.max(b));

    let color = if records[0].price > last.price {
        GREEN
    } else {
        RED
    };

    let root = BitMapBackend::new(&path_2, (800, 600)).into_drawing_area();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 60)
        .margin(10)
        .build_cartesian_2d(start_date..end_date, min_price..max_price)?;

    chart
        .configure_mesh()
        .x_labels(5)
        .y_labels(5)
        .label_style(("sans-serif", 20))
        .x_label_formatter(&|date| date.format("%Y-%m-%d").to_string())
        .draw()?;

    chart.draw_series(
        LineSeries::new(
            records
                .iter()
                .map(|record| (record.created_at.and_utc(), record.price)),
            color,
        )
        .point_size(3),
    )?;

    root.present()?;
    Ok(path)
}
