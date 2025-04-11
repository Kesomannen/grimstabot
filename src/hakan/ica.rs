use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use scraper::{Html, Selector};
use uuid::Uuid;

use crate::AppState;

use super::{Ingredient, Product};

const STORE_ID: u32 = 1004554;

async fn get_products_raw(category_name: &str, http: &reqwest::Client) -> Result<Vec<Product>> {
    let url =
        format!("https://handlaprivatkund.ica.se/stores/{STORE_ID}/categories/{category_name}");
    let html = http
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let product_selector = Selector::parse(".sc-filq44-0").unwrap();
    let name_selector = Selector::parse("._link-standalone_v2p9r_8").unwrap();
    let amount_selector = Selector::parse(".kUYwXM").unwrap();

    let document = Html::parse_document(&html);

    let mut products = Vec::new();

    for product_ele in document.select(&product_selector) {
        let name_ele = product_ele
            .select(&name_selector)
            .next()
            .ok_or_else(|| anyhow!("product name element is missing"))?;

        let full_name = name_ele
            .attr("aria-label")
            .ok_or_else(|| anyhow!("name element aria-label is missing"))?;

        let mut split = full_name.split(' ');
        let manufacturer_name = split
            .next_back()
            .ok_or_else(|| anyhow!("invalid name format"))?
            .to_owned();

        let name = split.intersperse(" ").collect();
        let href = name_ele
            .attr("href")
            .ok_or_else(|| anyhow!("name is missing href"))?;
        let url = format!("https://handlaprivatkund.ica.se{href}");

        let price_ele = product_ele
            .select(&amount_selector)
            .next()
            .ok_or_else(|| anyhow!("amount element missing"))?;

        let (price, price_text) = price_ele
            .text()
            .nth(1)
            .ok_or_else(|| anyhow!("price element is missing text"))?
            .split_once('\u{a0}')
            .ok_or_else(|| anyhow!("invalid price format"))?;

        let price: f64 = price
            .trim()
            .replace(',', ".")
            .parse()
            .context("invalid price number format")?;

        products.push(Product {
            name,
            manufacturer_name,
            comparative_price: price,
            comparative_price_text: price_text.into(),
            url,
        });
    }

    Ok(products)
}

pub async fn get_products(
    ingredient: &Ingredient,
    state: &AppState,
) -> Result<impl Iterator<Item = Product>> {
    Ok(get_products_raw(&ingredient.ica_category_name, &state.http)
        .await?
        .into_iter())
}
