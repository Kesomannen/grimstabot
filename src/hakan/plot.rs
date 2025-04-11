use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use chrono::NaiveDateTime;
use http::{HeaderMap, HeaderValue};
use plotters::prelude::*;
use uuid::Uuid;

use crate::AppState;

pub async fn create(state: &AppState) -> Result<PathBuf> {
    let uuid = Uuid::new_v4().to_string();
    let dir = std::env::var("PLOTS_DIRECTORY").context("PLOTS_DIRECTORY must be set")?;
    let path = PathBuf::from(dir).join(uuid).with_extension("png");

    let reports = fetch_reports(state).await?;

    if reports.is_empty() {
        bail!("no records in time range");
    }

    let last = reports.iter().last().unwrap();

    let start_date = reports[0].created_at.and_utc();
    let end_date = last.created_at.and_utc();

    let min_price = reports
        .iter()
        .map(|record| record.price)
        .fold(f64::INFINITY, |a, b| a.min(b));
    let max_price = reports
        .iter()
        .map(|record| record.price)
        .fold(f64::NEG_INFINITY, |a, b| a.max(b));

    let color = if reports[0].price > last.price {
        GREEN
    } else {
        RED
    };

    let path_2 = path.clone();
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
            reports
                .iter()
                .map(|record| (record.created_at.and_utc(), record.price)),
            color,
        )
        .point_size(3),
    )?;

    root.present()?;
    Ok(path)
}

struct Report {
    created_at: NaiveDateTime,
    price: f64,
}

async fn fetch_reports(state: &AppState) -> Result<Vec<Report>> {
    let records = sqlx::query_as!(
        Report,
        r"WITH p AS (
            SELECT 
                products.report_id,
                (products.comparative_price * ingredients.amount) price
            FROM products
            JOIN ingredients
            ON ingredients.id = products.ingredient_id
        )
        SELECT
            reports.created_at,
            SUM(p.price) price
        FROM reports
        JOIN p 
            ON p.report_id = reports.id
        GROUP BY reports.created_at
        ORDER BY created_at ASC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(records)
}

pub async fn upload(path: &Path, state: &AppState) -> Result<String> {
    let file_name = path.file_name().unwrap().to_string_lossy();
    let storage_path = format!("plots/{file_name}");
    let mut reader = tokio::fs::File::open(path).await?;

    let mut headers = HeaderMap::new();
    headers.insert("x-amz-acl", HeaderValue::from_static("public-read"));

    state
        .s3
        .with_extra_headers(headers)
        .unwrap()
        .put_object_stream_with_content_type(&mut reader, &storage_path, "image/png")
        .await?;

    let url = format!("https://mod-platform.fra1.cdn.digitaloceanspaces.com/{storage_path}");
    Ok(url)
}
