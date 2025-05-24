use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use http::{HeaderMap, HeaderValue};
use itertools::Itertools;
use plotters::{
    prelude::*,
    style::full_palette::{ORANGE, PINK, PURPLE},
};
use uuid::Uuid;

use crate::AppState;

pub async fn create_total(state: &AppState) -> Result<String> {
    let path = get_path().await?;
    let reports = super::db::reports(state).await?;

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
    upload(path, state).await
}

pub async fn create_by_store(state: &AppState) -> Result<String> {
    let path = get_path().await?;
    let reports = super::db::reports_by_store(state).await?;

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
                "hemkop" => PINK,
                "willys" => BLACK,
                "mathem" => BLUE,
                _ => BLACK,
            };

            (color.into(), Some(store), values)
        })
        .collect();

    draw(serieses, &path, true)?;
    upload(path, state).await
}

pub async fn create_by_ingredient(state: &AppState) -> Result<String> {
    let path = get_path().await?;
    let reports = super::db::reports_by_ingredient(state).await?;

    let mut ingredients = HashMap::new();

    for report in reports {
        ingredients
            .entry(report.ingredient_name)
            .or_insert_with(|| Vec::new())
            .push((report.created_at.and_utc(), report.price));
    }

    let serieses = ingredients
        .into_iter()
        .map(|(ingredient, values)| {
            let color = match ingredient.as_str() {
                "Vetemjöl" => RED,
                "Kakao" => ORANGE,
                "Ägg" => GREEN,
                "Smör" => BLUE,
                "Strösocker" => PURPLE,
                _ => BLACK,
            };

            (color.into(), Some(ingredient), values)
        })
        .collect();

    draw(serieses, &path, true)?;
    upload(path, state).await
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
            .label_font(("sans-serif", 20))
            .border_style(BLACK)
            .draw()?;
    }

    root.present()?;

    Ok(())
}

async fn upload(path: impl AsRef<Path>, state: &AppState) -> Result<String> {
    let file_name = path.as_ref().file_name().unwrap().to_string_lossy();
    let storage_key = format!("plots/{file_name}");

    let data = tokio::fs::read(&path).await?;
    state.storage.upload(&storage_key, data, true).await?;

    tokio::fs::remove_file(path).await?;
    let url = state.storage.object_url(storage_key);
    Ok(url)
}

fn max_float_iter(iter: impl Iterator<Item = f64>) -> f64 {
    iter.fold(f64::NEG_INFINITY, |a, b| a.max(b))
}

fn min_float_iter(iter: impl Iterator<Item = f64>) -> f64 {
    iter.fold(f64::INFINITY, |a, b| a.min(b))
}
