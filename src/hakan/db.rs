use anyhow::Result;
use chrono::NaiveDateTime;

use crate::AppState;

use super::Product;

pub struct Report {
    pub created_at: NaiveDateTime,
    pub price: f64,
}

pub async fn reports(state: &AppState) -> Result<Vec<Report>> {
    let records = sqlx::query_as!(
        Report,
        r#"
SELECT
    reports.created_at,
    SUM(p.price) AS "price!"
FROM reports
JOIN cheapest_products p
    ON p.report_id = reports.id
GROUP BY reports.created_at
ORDER BY created_at ASC"#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(records)
}

pub struct ReportWithStore {
    pub created_at: NaiveDateTime,
    pub price: f64,
    pub store: String,
}

pub async fn reports_by_store(state: &AppState) -> Result<Vec<ReportWithStore>> {
    let records = sqlx::query_as!(
        ReportWithStore,
        r#"
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
    p.store,
    SUM(p.price) AS "price!"
FROM reports
JOIN p 
    ON p.report_id = reports.id
GROUP BY p.store, reports.created_at
ORDER BY created_at ASC"#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(records)
}

pub struct ReportWithIngredient {
    pub created_at: NaiveDateTime,
    pub price: f64,
    pub ingredient_name: String,
}

pub async fn reports_by_ingredient(state: &AppState) -> Result<Vec<ReportWithIngredient>> {
    let records = sqlx::query_as!(
        ReportWithIngredient,
        r#"
SELECT
  ingredients.name AS ingredient_name,
  reports.created_at,
  MIN(products.comparative_price) * amount AS "price!"
FROM ingredients
LEFT JOIN products
	ON products.ingredient_id = ingredients.id
LEFT JOIN reports
	ON products.report_id = reports.id
GROUP BY 
    ingredient_id, 
    report_id, 
    ingredients.name, 
    reports.created_at, 
    ingredients.amount"#,
    )
    .fetch_all(&state.db)
    .await?;

    Ok(records)
}

pub async fn last_products(state: &AppState) -> Result<Vec<(String, Product)>> {
    let records = sqlx::query!(
        r#"
SELECT
	p.name,
    p.manufacturer_name,
    p.comparative_price,
    p.comparative_price_text,
    p.url,
    p.price,
    ingredients.name AS ingredient_name
FROM reports
LEFT JOIN cheapest_products p
	ON p.report_id = reports.id
LEFT JOIN ingredients
	ON p.ingredient_id = ingredients.id
WHERE reports.created_at = (
    SELECT MAX(created_at) FROM reports
)"#,
    )
    .map(|record| {
        (
            record.ingredient_name.unwrap(),
            Product {
                name: record.name.unwrap(),
                manufacturer_name: record.manufacturer_name.unwrap(),
                comparative_price: record.comparative_price.unwrap(),
                comparative_price_text: record.comparative_price_text.unwrap(),
                url: record.url.unwrap(),
                price: record.price.unwrap(),
            },
        )
    })
    .fetch_all(&state.db)
    .await?;

    Ok(records)
}
