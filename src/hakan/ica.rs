use anyhow::{anyhow, Context, Result};
use itertools::Itertools;
use scraper::{Html, Selector};

use crate::AppState;

use super::{Ingredient, Product};

const STORE_ID: u32 = 1004554;

pub async fn get_products(
    ingredient: &Ingredient,
    state: &AppState,
) -> Result<impl Iterator<Item = Product>> {
    let url = format!(
        "https://handlaprivatkund.ica.se/stores/{STORE_ID}/categories/{}",
        ingredient.ica_category_name
    );

    let html = state
        .http
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;

    let product_selector = Selector::parse(".sc-fuTSoq").unwrap();
    let name_selector = Selector::parse("._link-standalone_v2p9r_8").unwrap();
    let price_selector = Selector::parse(".sc-fHCFno").unwrap();

    let document = Html::parse_document(&html);

    let mut products = Vec::new();

    for product_ele in document.select(&product_selector) {
        let name_ele = product_ele
            .select(&name_selector)
            .next()
            .ok_or_else(|| anyhow!("product name element is missing"))?;

        let full_name = name_ele
            .text()
            .next()
            .ok_or_else(|| anyhow!("name element is missing text"))?;

        let mut split = full_name.split(' ');
        let manufacturer_name = split
            .next_back()
            .ok_or_else(|| anyhow!("invalid name format"))?
            .to_owned();

        let name = split
            .filter(|part| part.chars().next().is_some_and(|part| !part.is_numeric()))
            .intersperse(" ")
            .collect();

        let href = name_ele
            .attr("href")
            .ok_or_else(|| anyhow!("name is missing href"))?;
        let url = format!("https://handlaprivatkund.ica.se{href}");

        let price_ele = product_ele
            .select(&price_selector)
            .next()
            .ok_or_else(|| anyhow!("price element is missing"))?;

        let (price, price_text) = price_ele
            .text()
            .nth(2)
            .ok_or_else(|| anyhow!("price element is missing text"))?
            .split_once(' ')
            .ok_or_else(|| anyhow!("invalid price format"))?;

        let comparative_price: f64 = price
            .trim()
            .replace(',', ".")
            .parse()
            .context("invalid price number format")?;

        products.push(Product {
            name,
            manufacturer_name,
            comparative_price,
            comparative_price_text: price_text.into(),
            url,
            price: comparative_price * ingredient.amount,
        });
    }

    Ok(products.into_iter())
}
