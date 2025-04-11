use std::{
    collections::HashMap,
    fmt::Display,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use http::{HeaderMap, HeaderValue};
use itertools::Itertools;
use plotters::{coord::Shift, prelude::*, style::full_palette::ORANGE};
use uuid::Uuid;

use crate::AppState;

pub async fn create_total(state: &AppState) -> Result<PathBuf> {
    let path = get_path().await?;
    let reports = fetch_reports(state).await?;

    let last = reports.iter().last().unwrap();

    let color = if reports[0].price > last.price {
        GREEN
    } else {
        RED
    };

    let series = reports
        .into_iter()
        .map(|report| (report.created_at.and_utc(), report.price))
        .collect_vec();

    draw(vec![(color.into(), None, series)], &path, false)?;

    Ok(path)
}

pub async fn create_by_store(state: &AppState) -> Result<PathBuf> {
    let path = get_path().await?;
    let reports = fetch_reports_by_store(state).await?;

    let mut stores = HashMap::new();

    for report in reports {
        stores
            .entry(report.store)
            .or_insert_with(|| Vec::new())
            .push((report.created_at.and_utc(), report.price));
    }

    let serieses = stores
        .into_iter()
        .map(|(store, values)| {
            let color = match store.as_str() {
                "coop" => GREEN,
                "ica" => RED,
                _ => BLACK,
            };

            (color.into(), Some(store), values)
        })
        .collect();

    draw(serieses, &path, true)?;

    Ok(path)
}

async fn get_path() -> Result<PathBuf> {
    let uuid = Uuid::new_v4().to_string();
    let dir = std::env::var("PLOTS_DIRECTORY").context("PLOTS_DIRECTORY must be set")?;
    tokio::fs::create_dir_all(&dir).await?;

    let path = PathBuf::from(dir).join(uuid).with_extension("png");
    Ok(path)
}

fn draw(
    serieses: Vec<(RGBAColor, Option<String>, Vec<(DateTime<Utc>, f64)>)>,
    path: &Path,
    draw_labels: bool,
) -> Result<()> {
    let root = BitMapBackend::new(path, (800, 600)).into_drawing_area();

    root.fill(&WHITE)?;

    let start_date = serieses
        .iter()
        .filter_map(|(_, _, series)| series.iter().map(|(date, _)| date).min())
        .min()
        .map(|date| *date)
        .unwrap_or_default();

    let end_date = serieses
        .iter()
        .filter_map(|(_, _, series)| series.iter().map(|(date, _)| date).max())
        .max()
        .map(|date| *date)
        .unwrap_or_default();

    let min_price = min_float_iter(
        serieses
            .iter()
            .map(|(_, _, series)| min_float_iter(series.iter().map(|(_, value)| *value))),
    );

    let max_price = max_float_iter(
        serieses
            .iter()
            .map(|(_, _, series)| max_float_iter(series.iter().map(|(_, value)| *value))),
    );

    dbg!(&start_date, &end_date, &min_price, &max_price);

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

    for (color, label, values) in serieses {
        let series = chart
            .draw_series(LineSeries::new(values, color).point_size(3))?
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));

        if let Some(label) = label {
            series.label(label);
        }
    }

    if draw_labels {
        chart
            .configure_series_labels()
            .label_font(("sans-serif", 25))
            .border_style(BLACK)
            .draw()?;
    }

    root.present()?;

    Ok(())
}

struct Report {
    created_at: NaiveDateTime,
    price: f64,
}

async fn fetch_reports(state: &AppState) -> Result<Vec<Report>> {
    let records = sqlx::query_as!(
        Report,
        r"
WITH p AS (
  WITH ranked_products AS (
    SELECT
      products.report_id,
      (products.comparative_price * ingredients.amount) AS price,
      ROW_NUMBER() OVER (
        PARTITION BY products.report_id, products.ingredient_id
        ORDER BY (products.comparative_price * ingredients.amount)
      ) AS rn
    FROM products
    JOIN ingredients
      ON ingredients.id = products.ingredient_id
  )
  SELECT
  	price,
    report_id
  FROM ranked_products
  WHERE rn = 1
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

struct ReportWithStore {
    created_at: NaiveDateTime,
    price: f64,
    store: String,
}

async fn fetch_reports_by_store(state: &AppState) -> Result<Vec<ReportWithStore>> {
    let records = sqlx::query_as!(
        ReportWithStore,
        r"
WITH p AS (
    SELECT
      products.store,
      products.report_id,
      (products.comparative_price * ingredients.amount) AS price
    FROM products
    JOIN ingredients
      ON ingredients.id = products.ingredient_id
)
SELECT
    reports.created_at,
    SUM(p.price) price,
    p.store
FROM reports
JOIN p 
    ON p.report_id = reports.id
GROUP BY p.store, reports.created_at
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
    tokio::fs::remove_file(path).await?;
    Ok(url)
}

fn max_float_iter(iter: impl Iterator<Item = f64>) -> f64 {
    iter.fold(f64::NEG_INFINITY, |a, b| a.max(b))
}

fn min_float_iter(iter: impl Iterator<Item = f64>) -> f64 {
    iter.fold(f64::INFINITY, |a, b| a.min(b))
}
